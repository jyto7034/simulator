mod template;
mod threat;
mod types;
mod validation;

use template::parse_battlefield_template;
pub use threat::required_briefing_warning_tags_for_spawn_waves;
use threat::threat_warnings_for;
#[cfg(test)]
use threat::{
    detectable_threat_warning_tags, rumor_threat_warning_tag, THREAT_RUMOR_CHANCE_PERCENT,
};
pub use types::*;
use validation::validate_instance;

use crate::game::{
    battle::types::BattleUnitStatScale,
    behavior::GameError,
    combat_setup::mission_policy::CombatMissionPolicy,
    data::{pve_data::PveEncounter, pve_data::PveWaveData, GameDataBase},
    determinism,
    enums::RiskLevel,
    map::{MapNodeCategory, MapNodeId},
    resources::Position,
    wave_resolution::resolve_pve_wave_enemy_data_with_budget_multiplier,
};
use serde::{Deserialize, Serialize};
use std::{collections::HashSet, sync::OnceLock};

const CORRODED_APPEARANCE_SEED_NS: u64 = 0x434F_5241_5050_4541; // "CORAPPEA"

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct BattlefieldSizeClassDefinition {
    size_class: BattlefieldSizeClass,
    id: String,
    width: i32,
    height: i32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct BattlefieldArchetypeGenerationDefinition {
    archetype: BattlefieldArchetype,
    id: String,
    default_size_class: BattlefieldSizeClass,
    enabled: bool,
    weight: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct BattlefieldGenerationPolicyDatabase {
    size_classes: Vec<BattlefieldSizeClassDefinition>,
    archetypes: Vec<BattlefieldArchetypeGenerationDefinition>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct BattlefieldTemplateDefinition {
    id: String,
    archetype: BattlefieldArchetype,
    size_class: BattlefieldSizeClass,
    rows: Vec<String>,
    #[serde(default)]
    routes: Vec<BattlefieldRouteDefinition>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct BattlefieldRouteDefinition {
    id: String,
    overlay: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ParsedBattlefieldTemplate {
    id: String,
    archetype: BattlefieldArchetype,
    size_class: BattlefieldSizeClass,
    width: i32,
    height: i32,
    tiles: Vec<BattlefieldTile>,
    valid_tiles: Vec<Position>,
    deployment_zones: Vec<DeploymentZone>,
    spawn_zones: Vec<SpawnZone>,
    routes: Vec<BattlefieldRoute>,
    obstacles: Vec<Position>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct BattlefieldTemplateDatabase {
    templates: Vec<BattlefieldTemplateDefinition>,
}

impl BattlefieldGenerationPolicyDatabase {
    /// Load the embedded battlefield archetype/size selection policy.
    ///
    /// Combat preview owns this domain builtin because it is used to resolve a
    /// playable battlefield before battle setup DTOs are produced. It is not an
    /// alternate `GameDataBase` live bundle loader.
    fn builtin() -> &'static Self {
        static DATABASE: OnceLock<BattlefieldGenerationPolicyDatabase> = OnceLock::new();
        DATABASE.get_or_init(|| {
            let database: BattlefieldGenerationPolicyDatabase = ron::de::from_str(include_str!(
                "../../../../game_resources/data/map/battlefield_archetypes.ron"
            ))
            .expect("built-in battlefield archetype generation policy must be valid RON");
            database
                .validate_contract()
                .expect("built-in battlefield archetype generation policy must satisfy contract");
            database
        })
    }

    fn validate_contract(&self) -> Result<(), String> {
        if self.size_classes.is_empty() {
            return Err("battlefield policy must define at least one size class".to_string());
        }
        if self.archetypes.is_empty() {
            return Err("battlefield policy must define at least one archetype".to_string());
        }

        let mut size_classes = HashSet::new();
        for definition in &self.size_classes {
            if definition.id.trim().is_empty() {
                return Err("battlefield size class id must not be empty".to_string());
            }
            if definition.width <= 0 || definition.height <= 0 {
                return Err(format!(
                    "battlefield size class '{}' dimensions must be positive",
                    definition.id
                ));
            }
            if !size_classes.insert(definition.size_class) {
                return Err(format!(
                    "duplicate battlefield size class {:?}",
                    definition.size_class
                ));
            }
        }

        let expected_archetypes = [
            BattlefieldArchetype::OpenHall,
            BattlefieldArchetype::Corridor,
            BattlefieldArchetype::ChokePoint,
            BattlefieldArchetype::Ambush,
            BattlefieldArchetype::Surrounded,
            BattlefieldArchetype::SplitRoom,
            BattlefieldArchetype::ObstacleRoom,
            BattlefieldArchetype::BossArena,
        ];
        let mut archetypes = HashSet::new();
        let mut enabled_random_weight = 0_u32;
        for definition in &self.archetypes {
            if definition.id.trim().is_empty() {
                return Err("battlefield archetype id must not be empty".to_string());
            }
            if !size_classes.contains(&definition.default_size_class) {
                return Err(format!(
                    "battlefield archetype '{}' references missing default size class {:?}",
                    definition.id, definition.default_size_class
                ));
            }
            if !archetypes.insert(definition.archetype) {
                return Err(format!(
                    "duplicate battlefield archetype {:?}",
                    definition.archetype
                ));
            }
            if definition.enabled {
                if definition.archetype == BattlefieldArchetype::SplitRoom {
                    return Err(
                        "SplitRoom battlefield archetype must stay disabled until implemented"
                            .to_string(),
                    );
                }
                if definition.archetype != BattlefieldArchetype::BossArena {
                    if definition.weight == 0 {
                        return Err(format!(
                            "enabled battlefield archetype '{}' must have positive weight",
                            definition.id
                        ));
                    }
                    enabled_random_weight = enabled_random_weight.saturating_add(definition.weight);
                }
            }
        }
        for archetype in expected_archetypes {
            if !archetypes.contains(&archetype) {
                return Err(format!(
                    "battlefield policy is missing archetype {:?}",
                    archetype
                ));
            }
        }
        if enabled_random_weight == 0 {
            return Err(
                "battlefield policy must enable at least one non-boss random archetype".to_string(),
            );
        }

        Ok(())
    }

    #[cfg(test)]
    fn size_class(&self, size_class: BattlefieldSizeClass) -> &BattlefieldSizeClassDefinition {
        self.size_classes
            .iter()
            .find(|definition| definition.size_class == size_class)
            .expect("validated battlefield size class should exist")
    }

    fn archetype(
        &self,
        archetype: BattlefieldArchetype,
    ) -> &BattlefieldArchetypeGenerationDefinition {
        self.archetypes
            .iter()
            .find(|definition| definition.archetype == archetype)
            .expect("validated battlefield archetype should exist")
    }

    fn random_archetype(&self, seed: u64) -> BattlefieldArchetype {
        let candidates = self
            .archetypes
            .iter()
            .filter(|definition| {
                definition.enabled && definition.archetype != BattlefieldArchetype::BossArena
            })
            .collect::<Vec<_>>();
        let total_weight = candidates.iter().fold(0_u64, |total, definition| {
            total + u64::from(definition.weight)
        });
        debug_assert!(total_weight > 0, "validated policy has enabled weight");
        let mut roll = determinism::seed_with_namespace(seed, 0x4246_4152_4348_4554) % total_weight;
        for definition in candidates {
            let weight = u64::from(definition.weight);
            if roll < weight {
                return definition.archetype;
            }
            roll -= weight;
        }
        unreachable!("validated weighted archetype policy should select a candidate")
    }
}

impl BattlefieldTemplateDatabase {
    /// Load the embedded battlefield template catalog.
    ///
    /// The combat preview/battlefield domain owns this catalog. It is validated
    /// here because template ASCII rows, valid tiles, deployment zones, routes,
    /// and obstacles are interpreted by the battlefield parser rather than by
    /// generic game-data loading.
    fn builtin() -> &'static Self {
        static DATABASE: OnceLock<BattlefieldTemplateDatabase> = OnceLock::new();
        DATABASE.get_or_init(|| {
            let templates: Vec<BattlefieldTemplateDefinition> = ron::de::from_str(include_str!(
                "../../../../game_resources/data/map/battlefield_templates.ron"
            ))
            .expect("built-in battlefield templates must be valid RON");
            let database = BattlefieldTemplateDatabase { templates };
            database
                .validate_contract()
                .expect("built-in battlefield templates must satisfy contract");
            database
        })
    }

    fn validate_contract(&self) -> Result<(), String> {
        if self.templates.is_empty() {
            return Err(
                "battlefield template database must define at least one template".to_string(),
            );
        }

        let mut ids = HashSet::new();
        let mut archetype_size_pairs = HashSet::new();
        for template in &self.templates {
            if template.id.trim().is_empty() {
                return Err("battlefield template id must not be empty".to_string());
            }
            if !ids.insert(template.id.as_str()) {
                return Err(format!(
                    "duplicate battlefield template id '{}'",
                    template.id
                ));
            }
            archetype_size_pairs.insert((template.archetype, template.size_class));
            let parsed = parse_battlefield_template(template)?;
            if !matches!(
                template.archetype,
                BattlefieldArchetype::BossArena | BattlefieldArchetype::SplitRoom
            ) && !parsed.routes.iter().any(|route| route.id == "defense_main")
            {
                return Err(format!(
                    "battlefield template '{}' must define defense_main route",
                    template.id
                ));
            }
        }

        for archetype in [
            BattlefieldArchetype::OpenHall,
            BattlefieldArchetype::Corridor,
            BattlefieldArchetype::ChokePoint,
            BattlefieldArchetype::Ambush,
            BattlefieldArchetype::Surrounded,
            BattlefieldArchetype::SplitRoom,
            BattlefieldArchetype::ObstacleRoom,
            BattlefieldArchetype::BossArena,
        ] {
            let default_size = BattlefieldGenerationPolicyDatabase::builtin()
                .archetype(archetype)
                .default_size_class;
            if !archetype_size_pairs.contains(&(archetype, default_size)) {
                return Err(format!(
                    "battlefield template database is missing default template for {:?}/{:?}",
                    archetype, default_size
                ));
            }
        }

        Ok(())
    }

    fn select(
        &self,
        archetype: BattlefieldArchetype,
        size_class: BattlefieldSizeClass,
        seed: u64,
    ) -> &BattlefieldTemplateDefinition {
        let candidates = self
            .templates
            .iter()
            .filter(|template| template.archetype == archetype && template.size_class == size_class)
            .collect::<Vec<_>>();
        if !candidates.is_empty() {
            return candidates[(seed as usize) % candidates.len()];
        }

        let default_size = BattlefieldGenerationPolicyDatabase::builtin()
            .archetype(archetype)
            .default_size_class;
        self.templates
            .iter()
            .find(|template| template.archetype == archetype && template.size_class == default_size)
            .expect("validated default battlefield template should exist")
    }
}

impl BattlefieldGenerator {
    pub fn try_generate(
        request: BattlefieldGenerationRequest<'_>,
        game_data: &GameDataBase,
    ) -> Result<BattlefieldInstance, GameError> {
        let instance = Self::build_instance(request, game_data)?;
        validate_instance(&instance).map_err(|message| {
            GameError::InvalidStaticData(format!("invalid battlefield instance: {message}"))
        })?;
        Ok(instance)
    }

    pub fn generate(
        request: BattlefieldGenerationRequest<'_>,
        game_data: &GameDataBase,
    ) -> BattlefieldInstance {
        Self::try_generate(request, game_data).expect("battlefield instance should be valid")
    }

    fn build_instance(
        request: BattlefieldGenerationRequest<'_>,
        game_data: &GameDataBase,
    ) -> Result<BattlefieldInstance, GameError> {
        let encounter = request
            .encounter_id
            .and_then(|id| game_data.pve_data.get_by_id(id));
        let archetype = encounter
            .and_then(|encounter| {
                encounter
                    .battlefield
                    .as_ref()
                    .and_then(|battlefield| battlefield.archetype)
            })
            .unwrap_or_else(|| archetype_for(request.category, request.seed));
        let node_type = encounter
            .and_then(|encounter| encounter.node_type)
            .map(Ok)
            .unwrap_or_else(|| {
                CombatMissionPolicy::try_fallback_node_type_for_archetype(
                    request.category,
                    archetype,
                )
            })?;
        let mission_variant = encounter
            .and_then(|encounter| encounter.mission_variant)
            .unwrap_or_else(|| CombatMissionVariant::default_for_node_type(node_type));
        let survive_timer_ms = encounter.and_then(|encounter| encounter.survive_timer_ms);
        let mission_risk = encounter
            .map(|encounter| CombatMissionRisk::from_risk_level(encounter.risk_level))
            .unwrap_or(CombatMissionRisk::Controlled);
        let size_class = encounter
            .and_then(|encounter| {
                encounter
                    .battlefield
                    .as_ref()
                    .and_then(|battlefield| battlefield.size_class)
            })
            .unwrap_or_else(|| size_for(request.category, archetype));
        let template =
            BattlefieldTemplateDatabase::builtin().select(archetype, size_class, request.seed);
        let mut battlefield_template =
            parse_battlefield_template(template).unwrap_or_else(|message| {
                panic!(
                    "validated battlefield template '{}' failed to parse: {}",
                    template.id, message
                )
            });
        apply_authored_static_obstacles(&mut battlefield_template, encounter);
        ensure_defense_route(&mut battlefield_template, node_type)?;
        let scaling_stage = game_data
            .run_policy
            .floor_combat_scaling_for_floor_index(request.floor_index);
        let enemy_stat_scale = BattleUnitStatScale {
            max_health_percent: scaling_stage.max_health_multiplier_percent,
            attack_percent: scaling_stage.attack_multiplier_percent,
            defense_percent: scaling_stage.defense_multiplier_percent,
        };
        let spawn_waves = spawn_waves_for(
            node_type,
            archetype,
            &battlefield_template.spawn_zones,
            encounter,
            game_data,
            request.seed,
            scaling_stage.generated_wave_budget_multiplier_percent,
            &scaling_stage.extra_waves,
        )?;
        let enemy_briefing = enemy_briefing(encounter, game_data, request.category, &spawn_waves);
        let threat_warnings = threat_warnings_for(&spawn_waves, game_data, request.seed);
        Ok(BattlefieldInstance {
            battlefield_template_id: battlefield_template.id,
            node_type,
            mission_variant,
            survive_timer_ms,
            mission_risk,
            archetype: battlefield_template.archetype,
            size_class: battlefield_template.size_class,
            width: battlefield_template.width,
            height: battlefield_template.height,
            tiles: battlefield_template.tiles,
            valid_tiles: battlefield_template.valid_tiles,
            deployment_zones: battlefield_template.deployment_zones,
            spawn_zones: battlefield_template.spawn_zones,
            routes: battlefield_template.routes,
            spawn_waves,
            obstacles: battlefield_template.obstacles,
            enemy_briefing,
            threat_warnings,
            enemy_stat_scale,
        })
    }
}

fn ensure_defense_route(
    battlefield_template: &mut crate::game::combat_preview::ParsedBattlefieldTemplate,
    node_type: CombatNodeType,
) -> Result<(), GameError> {
    if node_type != CombatNodeType::Defense || !battlefield_template.routes.is_empty() {
        return Ok(());
    }

    Err(GameError::InvalidStaticData(format!(
        "defense battlefield template '{}' must define at least one authored route",
        battlefield_template.id
    )))
}

impl CombatPreview {
    pub fn try_generate_for_node(
        node_id: MapNodeId,
        category: MapNodeCategory,
        encounter_id: Option<&str>,
        game_data: &GameDataBase,
        seed: u64,
    ) -> Result<Self, GameError> {
        let instance = BattlefieldGenerator::try_generate(
            BattlefieldGenerationRequest {
                category,
                encounter_id,
                seed,
                floor_index: 0,
            },
            game_data,
        )?;

        Ok(Self {
            node_id,
            encounter_id: encounter_id.map(str::to_string),
            battlefield_template_id: instance.battlefield_template_id,
            node_type: instance.node_type,
            mission_variant: instance.mission_variant,
            survive_timer_ms: instance.survive_timer_ms,
            mission_risk: instance.mission_risk,
            archetype: instance.archetype,
            size_class: instance.size_class,
            width: instance.width,
            height: instance.height,
            tiles: instance.tiles,
            valid_tiles: instance.valid_tiles,
            deployment_zones: instance.deployment_zones,
            spawn_zones: instance.spawn_zones,
            routes: instance.routes,
            spawn_waves: instance.spawn_waves,
            obstacles: instance.obstacles,
            enemy_briefing: instance.enemy_briefing,
            threat_warnings: instance.threat_warnings,
            enemy_stat_scale: instance.enemy_stat_scale,
        })
    }

    pub fn try_generate_for_node_at_floor(
        node_id: MapNodeId,
        category: MapNodeCategory,
        encounter_id: Option<&str>,
        game_data: &GameDataBase,
        seed: u64,
        floor_index: u32,
    ) -> Result<Self, GameError> {
        let instance = BattlefieldGenerator::try_generate(
            BattlefieldGenerationRequest {
                category,
                encounter_id,
                seed,
                floor_index,
            },
            game_data,
        )?;

        Ok(Self {
            node_id,
            encounter_id: encounter_id.map(str::to_string),
            battlefield_template_id: instance.battlefield_template_id,
            node_type: instance.node_type,
            mission_variant: instance.mission_variant,
            survive_timer_ms: instance.survive_timer_ms,
            mission_risk: instance.mission_risk,
            archetype: instance.archetype,
            size_class: instance.size_class,
            width: instance.width,
            height: instance.height,
            tiles: instance.tiles,
            valid_tiles: instance.valid_tiles,
            deployment_zones: instance.deployment_zones,
            spawn_zones: instance.spawn_zones,
            routes: instance.routes,
            spawn_waves: instance.spawn_waves,
            obstacles: instance.obstacles,
            enemy_briefing: instance.enemy_briefing,
            threat_warnings: instance.threat_warnings,
            enemy_stat_scale: instance.enemy_stat_scale,
        })
    }

    pub fn generate_for_node(
        node_id: MapNodeId,
        category: MapNodeCategory,
        encounter_id: Option<&str>,
        game_data: &GameDataBase,
        seed: u64,
    ) -> Self {
        Self::try_generate_for_node(node_id, category, encounter_id, game_data, seed)
            .expect("combat preview battlefield should be valid")
    }

    pub fn generate_for_node_at_floor(
        node_id: MapNodeId,
        category: MapNodeCategory,
        encounter_id: Option<&str>,
        game_data: &GameDataBase,
        seed: u64,
        floor_index: u32,
    ) -> Self {
        Self::try_generate_for_node_at_floor(
            node_id,
            category,
            encounter_id,
            game_data,
            seed,
            floor_index,
        )
        .expect("combat preview battlefield should be valid")
    }

    pub fn disprove_rumor_warnings(&mut self) {
        for warning in &mut self.threat_warnings {
            if warning.source == ThreatWarningSource::Rumor {
                warning.status = ThreatWarningStatus::Disproved;
            }
        }
    }
}

fn archetype_for(category: MapNodeCategory, seed: u64) -> BattlefieldArchetype {
    if category == MapNodeCategory::Boss {
        return BattlefieldArchetype::BossArena;
    }

    BattlefieldGenerationPolicyDatabase::builtin().random_archetype(seed)
}

fn size_for(category: MapNodeCategory, archetype: BattlefieldArchetype) -> BattlefieldSizeClass {
    if category == MapNodeCategory::Boss {
        BattlefieldSizeClass::Large
    } else {
        BattlefieldGenerationPolicyDatabase::builtin()
            .archetype(archetype)
            .default_size_class
    }
}

fn apply_authored_static_obstacles(
    template: &mut ParsedBattlefieldTemplate,
    encounter: Option<&PveEncounter>,
) {
    let Some(authored_obstacles) = encounter
        .map(|encounter| {
            encounter
                .static_obstacles
                .iter()
                .map(|obstacle| obstacle.position.into())
                .collect::<Vec<_>>()
        })
        .filter(|obstacles| !obstacles.is_empty())
    else {
        return;
    };

    let obstacles = unique_positions(authored_obstacles);
    let obstacle_positions = obstacles.iter().copied().collect::<HashSet<_>>();
    for tile in &mut template.tiles {
        tile.kind = if obstacle_positions.contains(&tile.position) {
            BattlefieldTileKind::Obstacle
        } else if tile.kind == BattlefieldTileKind::Obstacle {
            BattlefieldTileKind::Ground
        } else {
            tile.kind
        };
    }
    template.obstacles = obstacles;
}

fn spawn_waves_for(
    node_type: CombatNodeType,
    archetype: BattlefieldArchetype,
    spawn_zones: &[SpawnZone],
    encounter: Option<&PveEncounter>,
    game_data: &GameDataBase,
    preview_seed: u64,
    generated_wave_budget_multiplier_percent: u32,
    extra_waves: &[PveWaveData],
) -> Result<Vec<SpawnWave>, GameError> {
    let Some(encounter) = encounter else {
        return Err(GameError::InvalidStaticData(
            "combat preview requires an authored pve encounter".to_string(),
        ));
    };

    let mut authored_waves = encounter.wave_definitions();
    authored_waves.extend(extra_waves.iter().cloned());
    if authored_waves.is_empty() {
        return Err(GameError::InvalidStaticData(format!(
            "pve encounter '{}' must define at least one wave",
            encounter.id
        )));
    }

    authored_waves
        .iter()
        .enumerate()
        .map(|(index, wave)| {
            let route_id = if node_type == CombatNodeType::Defense {
                Some(wave.route_id.clone().ok_or_else(|| {
                    GameError::InvalidStaticData(format!(
                        "defense pve encounter '{}' wave '{}' must define route_id",
                        encounter.id, wave.id
                    ))
                })?)
            } else {
                wave.route_id.clone()
            };
            Ok(SpawnWave {
                id: wave.id.clone(),
                time_ms: wave.time_ms,
                spawn_zone_ids: if wave.spawn_zone_ids.is_empty() {
                    spawn_zone_ids_for_wave(archetype, spawn_zones, index)
                } else {
                    wave.spawn_zone_ids.clone()
                },
                route_id,
                enemy_entries: wave_enemy_entries(
                    wave,
                    game_data,
                    preview_seed,
                    index,
                    generated_wave_budget_multiplier_percent,
                ),
                required_for_victory: wave.required_for_victory,
            })
        })
        .collect()
}

fn spawn_zone_ids_for_wave(
    archetype: BattlefieldArchetype,
    spawn_zones: &[SpawnZone],
    wave_index: usize,
) -> Vec<String> {
    if wave_index == 0 {
        return spawn_zones
            .first()
            .map(|zone| vec![zone.id.clone()])
            .unwrap_or_else(|| vec!["north_entry".to_string()]);
    }

    if matches!(
        archetype,
        BattlefieldArchetype::Ambush
            | BattlefieldArchetype::Surrounded
            | BattlefieldArchetype::BossArena
    ) {
        let reinforcement_zones = spawn_zones
            .iter()
            .skip(1)
            .map(|zone| zone.id.clone())
            .collect::<Vec<_>>();
        if !reinforcement_zones.is_empty() {
            return reinforcement_zones;
        }
    }

    spawn_zones
        .first()
        .map(|zone| vec![zone.id.clone()])
        .unwrap_or_default()
}

fn wave_enemy_entries(
    wave: &PveWaveData,
    game_data: &GameDataBase,
    preview_seed: u64,
    wave_index: usize,
    generated_wave_budget_multiplier_percent: u32,
) -> Vec<SpawnWaveEnemyEntry> {
    resolve_pve_wave_enemy_data_with_budget_multiplier(
        wave,
        game_data,
        preview_seed,
        wave_index,
        generated_wave_budget_multiplier_percent,
    )
    .iter()
    .enumerate()
    .map(|(enemy_index, enemy)| {
        let count = enemy.count().max(1);
        match enemy {
            crate::game::data::pve_data::PveWaveEnemyData::Abnormality {
                abnormality_id, ..
            } => SpawnWaveEnemyEntry {
                kind: EnemyKind::Abnormality,
                profile_id: None,
                abnormality_id: abnormality_id.clone(),
                tier: enemy.tier(),
                count,
                appearance_seeds: Vec::new(),
            },
            crate::game::data::pve_data::PveWaveEnemyData::CorrodedEmployee {
                profile_id, ..
            } => SpawnWaveEnemyEntry {
                kind: EnemyKind::CorrodedEmployee,
                profile_id: Some(profile_id.clone()),
                abnormality_id: String::new(),
                tier: enemy.tier(),
                count,
                appearance_seeds: appearance_seeds_for_corroded_employee(
                    preview_seed,
                    wave_index,
                    enemy_index,
                    profile_id,
                    count,
                ),
            },
            crate::game::data::pve_data::PveWaveEnemyData::FacilityEntity {
                profile_id, ..
            } => SpawnWaveEnemyEntry {
                kind: EnemyKind::FacilityEntity,
                profile_id: Some(profile_id.clone()),
                abnormality_id: String::new(),
                tier: enemy.tier(),
                count,
                appearance_seeds: Vec::new(),
            },
        }
    })
    .collect()
}

fn appearance_seeds_for_corroded_employee(
    preview_seed: u64,
    wave_index: usize,
    enemy_index: usize,
    profile_id: &str,
    count: u32,
) -> Vec<u64> {
    let profile_seed = stable_str_seed(profile_id);
    let mut base = determinism::seed_with_namespace(preview_seed, CORRODED_APPEARANCE_SEED_NS);
    base = determinism::seed_with_namespace(base, profile_seed);
    base = determinism::seed_with_namespace(base, wave_index as u64);
    base = determinism::seed_with_namespace(base, enemy_index as u64);

    (0..count)
        .map(|spawn_index| determinism::seed_with_namespace(base, u64::from(spawn_index)))
        .collect()
}

fn stable_str_seed(value: &str) -> u64 {
    value.bytes().fold(0xcbf2_9ce4_8422_2325, |hash, byte| {
        (hash ^ u64::from(byte)).wrapping_mul(0x0000_0100_0000_01b3)
    })
}

fn enemy_briefing(
    encounter: Option<&PveEncounter>,
    game_data: &GameDataBase,
    category: MapNodeCategory,
    spawn_waves: &[SpawnWave],
) -> Vec<EnemyBriefing> {
    let Some(encounter) = encounter else {
        return vec![EnemyBriefing {
            kind: EnemyKind::CorrodedEmployee,
            abnormality_id: "unknown_abnormality".to_string(),
            display_name: "Corroded Expedition Member".to_string(),
            risk_level: RiskLevel::ZAYIN,
            role: role_for(category).to_string(),
            count_hint: 1,
        }];
    };
    let abnormality = encounter
        .primary_abnormality_id()
        .and_then(|id| game_data.abnormality_data.get_by_id(id));
    let kind = primary_enemy_kind(category, spawn_waves);
    let count_hint = spawn_waves
        .iter()
        .flat_map(|wave| &wave.enemy_entries)
        .map(|enemy| enemy.count)
        .sum::<u32>()
        .max(1);

    vec![EnemyBriefing {
        kind,
        abnormality_id: encounter.primary_abnormality_id.clone().unwrap_or_default(),
        display_name: abnormality
            .map(|abnormality| abnormality.name.clone())
            .unwrap_or_else(|| "Corroded Employees".to_string()),
        risk_level: abnormality
            .map(|abnormality| abnormality.risk_level)
            .unwrap_or(encounter.risk_level),
        role: role_for(category).to_string(),
        count_hint,
    }]
}

fn primary_enemy_kind(category: MapNodeCategory, spawn_waves: &[SpawnWave]) -> EnemyKind {
    if category == MapNodeCategory::Boss {
        return EnemyKind::Abnormality;
    }

    spawn_waves
        .iter()
        .flat_map(|wave| wave.enemy_entries.iter())
        .next()
        .map(|enemy| enemy.kind)
        .unwrap_or(EnemyKind::CorrodedEmployee)
}

fn confidence_for_hidden() -> ZoneConfidence {
    ZoneConfidence::Suspected
}

fn unique_positions(positions: Vec<Position>) -> Vec<Position> {
    let mut seen = HashSet::new();
    positions
        .into_iter()
        .filter(|position| seen.insert(*position))
        .collect()
}

fn unique_tiles(tiles: Vec<BattlefieldTile>) -> Vec<BattlefieldTile> {
    let mut seen = HashSet::new();
    tiles
        .into_iter()
        .filter(|tile| seen.insert(tile.position))
        .collect()
}

fn role_for(category: MapNodeCategory) -> &'static str {
    match category {
        MapNodeCategory::Boss => "Boss",
        _ => "PrimaryThreat",
    }
}

#[cfg(test)]
fn archetype_id(archetype: BattlefieldArchetype) -> &'static str {
    BattlefieldGenerationPolicyDatabase::builtin()
        .archetype(archetype)
        .id
        .as_str()
}

#[cfg(test)]
fn size_class_id(size_class: BattlefieldSizeClass) -> &'static str {
    BattlefieldGenerationPolicyDatabase::builtin()
        .size_class(size_class)
        .id
        .as_str()
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    use crate::game::data::{
        abnormality_data::AbnormalityMetadata,
        corroded_employee_data::{
            CorrodedEmployeeBasicAttackRangePresetDef, CorrodedEmployeeProfileDatabase,
            CorrodedEmployeeProfileMetadata,
        },
        corroded_wave_data::{
            CorrodedWaveCountRange, CorrodedWavePreset, CorrodedWavePresetDatabase,
            CorrodedWaveRoleWeight,
        },
        pve_data::{
            PveBattlefieldOverrideData, PveEncounter, PveEncounterDatabase, PvePosition,
            PveStaticObstacleData, PveWaveData, PveWaveEnemyData, PveWaveSource,
        },
        GameDataBuilder,
    };
    use crate::game::enums::Tier;

    fn empty_data() -> GameDataBase {
        GameDataBuilder::empty().build()
    }

    fn preview_wave(abnormality_id: &str, route_id: Option<&str>) -> PveWaveData {
        PveWaveData {
            id: "wave_0".to_string(),
            time_ms: 0,
            spawn_zone_ids: Vec::new(),
            route_id: route_id.map(str::to_string),
            required_for_victory: true,
            source: PveWaveSource::Manual(vec![PveWaveEnemyData::Abnormality {
                abnormality_id: abnormality_id.to_string(),
                tier: Tier::I,
                count: 1,
            }]),
        }
    }

    fn preview_data_with_encounter(
        encounter_id: &str,
        node_type: CombatNodeType,
        route_id: Option<&str>,
    ) -> GameDataBase {
        let abnormality_id = format!("{encounter_id}_enemy");
        GameDataBuilder::empty()
            .with_abnormalities(vec![preview_test_abnormality(&abnormality_id, 0xDADA)])
            .with_pve(PveEncounterDatabase::new(vec![PveEncounter {
                id: encounter_id.to_string(),
                encounter_class: crate::game::data::pve_data::PveEncounterClass::Elite,
                primary_abnormality_id: Some(abnormality_id.clone()),
                risk_level: RiskLevel::ZAYIN,
                reward_mode: crate::game::enums::RewardMode::ClaimAll,
                reward_uuids: Vec::new(),
                suppression_research: None,
                node_type: Some(node_type),
                mission_variant: None,
                survive_timer_ms: None,
                battlefield: None,
                tactical_plan: None,
                win_condition: None,
                waves: vec![preview_wave(&abnormality_id, route_id)],
                static_obstacles: Vec::new(),
            }]))
            .build()
    }

    fn assert_cardinal_route_cells(cells: &[Position]) {
        assert!(
            !cells.is_empty(),
            "route cells should contain at least the start cell"
        );
        for window in cells.windows(2) {
            let distance = (window[0].x - window[1].x).abs() + (window[0].y - window[1].y).abs();
            assert_eq!(
                distance, 1,
                "route cells should be cardinal-adjacent: {:?} -> {:?}",
                window[0], window[1]
            );
        }
    }

    fn preview_test_abnormality(id: &str, uuid: u128) -> AbnormalityMetadata {
        AbnormalityMetadata {
            id: id.to_string(),
            uuid: Uuid::from_u128(uuid),
            name: id.to_string(),
            risk_level: RiskLevel::ZAYIN,
            price: 1,
            max_health: 10,
            attack: 1,
            defense: 0,
            magic_resist: 0,
            threat_class: crate::game::battle::types::BattleUnitThreatClass::Elite,
            response_complete_skill_fragment_id: None,
            omen_chain_id: None,
            movement: Default::default(),
            basic_attack: Default::default(),
            resonance: Default::default(),
            skill_id: None,
            mobility_kind: Default::default(),
            target_traits: Vec::new(),
        }
    }

    fn preview_test_corroded_profile(id: &str, uuid: u128) -> CorrodedEmployeeProfileMetadata {
        CorrodedEmployeeProfileMetadata {
            id: id.to_string(),
            uuid: Uuid::from_u128(uuid),
            name: id.to_string(),
            profile_role: Default::default(),
            basic_attack_range_preset: Some("melee_front_1".to_string()),
            max_health: 10,
            attack: 1,
            defense: 0,
            magic_resist: 0,
            movement: Default::default(),
            basic_attack: crate::game::data::abnormality_data::BasicAttackDef {
                defense_tile_range: Some(crate::game::battle::tile_range::TileRangePattern {
                    include_anchor_tile: true,
                    rows: vec![".X.".to_string(), ".@.".to_string(), "...".to_string()],
                }),
                ..Default::default()
            },
            resonance: Default::default(),
            skill_id: None,
        }
    }

    fn preview_test_corroded_range_presets() -> Vec<CorrodedEmployeeBasicAttackRangePresetDef> {
        vec![CorrodedEmployeeBasicAttackRangePresetDef {
            id: "melee_front_1".to_string(),
            range: crate::game::battle::tile_range::TileRangePattern {
                include_anchor_tile: true,
                rows: vec![".X.".to_string(), ".@.".to_string(), "...".to_string()],
            },
        }]
    }

    #[test]
    fn combat_preview_serializes_typed_threat_warnings() {
        let mut armored = preview_test_abnormality("armored_enemy", 0xA111);
        armored.defense = crate::game::combat_setup::balance::HIGH_DEFENSE_WARNING_THRESHOLD;
        armored.magic_resist =
            crate::game::combat_setup::balance::HIGH_MAGIC_RESIST_WARNING_THRESHOLD;
        let data = GameDataBuilder::empty()
            .with_abnormalities(vec![armored])
            .with_pve(PveEncounterDatabase::new(vec![PveEncounter {
                id: "threat_encounter".to_string(),
                encounter_class: crate::game::data::pve_data::PveEncounterClass::Elite,
                primary_abnormality_id: Some("armored_enemy".to_string()),
                risk_level: RiskLevel::ZAYIN,
                reward_mode: crate::game::enums::RewardMode::ClaimAll,
                reward_uuids: Vec::new(),
                suppression_research: None,
                node_type: Some(CombatNodeType::Defense),
                mission_variant: None,
                survive_timer_ms: None,
                battlefield: None,
                tactical_plan: None,
                win_condition: None,
                waves: vec![PveWaveData {
                    id: "wave_0".to_string(),
                    time_ms: 0,
                    spawn_zone_ids: Vec::new(),
                    route_id: Some("defense_main".to_string()),
                    required_for_victory: true,
                    source: PveWaveSource::Manual(vec![PveWaveEnemyData::Abnormality {
                        abnormality_id: "armored_enemy".to_string(),
                        tier: Tier::I,
                        count: 1,
                    }]),
                }],
                static_obstacles: Vec::new(),
            }]))
            .build();

        let preview = CombatPreview::generate_for_node(
            MapNodeId::new(Uuid::from_u128(0xCAFE)),
            MapNodeCategory::Combat,
            Some("threat_encounter"),
            &data,
            0,
        );

        assert!(preview.threat_warnings.iter().any(|warning| {
            warning.tag == ThreatWarningTag::ArmoredEnemyPossible
                && warning.status == ThreatWarningStatus::Unverified
                && warning.source == ThreatWarningSource::Briefing
        }));
        assert!(preview.threat_warnings.iter().any(|warning| {
            warning.tag == ThreatWarningTag::HighMagicResistEnemyPossible
                && warning.status == ThreatWarningStatus::Unverified
                && warning.source == ThreatWarningSource::Briefing
        }));

        let value = serde_json::to_value(&preview).expect("serialize combat preview");
        assert!(value.get("threat_warning_tags").is_none());
        assert!(value["threat_warnings"]
            .as_array()
            .expect("threat_warnings should be an array")
            .iter()
            .any(|warning| {
                warning
                    == &serde_json::json!({
                        "tag": "armored_enemy_possible",
                        "status": "Unverified",
                        "source": "Briefing",
                    })
            }));
    }

    #[test]
    fn combat_preview_warns_when_airborne_enemy_can_appear() {
        let mut drone = preview_test_abnormality("airborne_enemy", 0xA112);
        drone.mobility_kind = crate::game::battle::types::MobilityKind::Airborne;
        let data = GameDataBuilder::empty()
            .with_abnormalities(vec![drone])
            .with_pve(PveEncounterDatabase::new(vec![PveEncounter {
                id: "airborne_encounter".to_string(),
                encounter_class: crate::game::data::pve_data::PveEncounterClass::Elite,
                primary_abnormality_id: Some("airborne_enemy".to_string()),
                risk_level: RiskLevel::ZAYIN,
                reward_mode: crate::game::enums::RewardMode::ClaimAll,
                reward_uuids: Vec::new(),
                suppression_research: None,
                node_type: Some(CombatNodeType::Defense),
                mission_variant: None,
                survive_timer_ms: None,
                battlefield: None,
                tactical_plan: None,
                win_condition: None,
                waves: vec![PveWaveData {
                    id: "wave_0".to_string(),
                    time_ms: 0,
                    spawn_zone_ids: Vec::new(),
                    route_id: Some("defense_main".to_string()),
                    required_for_victory: true,
                    source: PveWaveSource::Manual(vec![PveWaveEnemyData::Abnormality {
                        abnormality_id: "airborne_enemy".to_string(),
                        tier: Tier::I,
                        count: 1,
                    }]),
                }],
                static_obstacles: Vec::new(),
            }]))
            .build();

        let preview = CombatPreview::generate_for_node(
            MapNodeId::new(Uuid::from_u128(0xCAFF)),
            MapNodeCategory::Combat,
            Some("airborne_encounter"),
            &data,
            0,
        );

        assert!(preview.threat_warnings.iter().any(|warning| {
            warning.tag == ThreatWarningTag::AirEnemyPossible
                && warning.status == ThreatWarningStatus::Unverified
                && warning.source == ThreatWarningSource::Briefing
        }));
    }

    #[test]
    fn rumor_threat_warning_is_seeded_and_limited_to_one_entry() {
        assert_eq!(THREAT_RUMOR_CHANCE_PERCENT, 20);
        let seed = (0..10_000)
            .find(|seed| rumor_threat_warning_tag(*seed, &HashSet::new()).is_some())
            .expect("test should find a rumor-producing seed");
        let first = threat_warnings_for(&[], &empty_data(), seed);
        let second = threat_warnings_for(&[], &empty_data(), seed);

        assert_eq!(first, second);
        assert_eq!(first.len(), 1);
        assert_eq!(first[0].status, ThreatWarningStatus::Unverified);
        assert_eq!(first[0].source, ThreatWarningSource::Rumor);
        assert!(detectable_threat_warning_tags().contains(&first[0].tag));
    }

    #[test]
    fn rumor_threat_warning_does_not_duplicate_real_briefing_tag() {
        let real_tags = HashSet::from([ThreatWarningTag::ArmoredEnemyPossible]);
        for seed in 0..10_000 {
            if let Some(tag) = rumor_threat_warning_tag(seed, &real_tags) {
                assert_ne!(tag, ThreatWarningTag::ArmoredEnemyPossible);
                return;
            }
        }
        panic!("test should find a rumor-producing seed");
    }

    #[test]
    fn combat_preview_enemy_stat_scale_is_internal_not_wire_contract() {
        let data = GameDataBuilder::empty()
            .with_abnormalities(vec![preview_test_abnormality("scaled_enemy", 0xC0_51C1)])
            .with_pve(PveEncounterDatabase::new(vec![PveEncounter {
                id: "scaled_encounter".to_string(),
                encounter_class: crate::game::data::pve_data::PveEncounterClass::Elite,
                primary_abnormality_id: Some("scaled_enemy".to_string()),
                risk_level: RiskLevel::ZAYIN,
                reward_mode: crate::game::enums::RewardMode::ClaimAll,
                reward_uuids: Vec::new(),
                suppression_research: None,
                node_type: Some(CombatNodeType::Defense),
                mission_variant: None,
                survive_timer_ms: None,
                battlefield: None,
                tactical_plan: None,
                win_condition: None,
                waves: vec![preview_wave("scaled_enemy", Some("defense_main"))],
                static_obstacles: Vec::new(),
            }]))
            .build();
        let preview = CombatPreview::try_generate_for_node_at_floor(
            MapNodeId::new(Uuid::from_u128(0xC0_51C1)),
            MapNodeCategory::Combat,
            Some("scaled_encounter"),
            &data,
            51,
            6,
        )
        .expect("combat preview should generate from live data");

        assert_ne!(preview.enemy_stat_scale, BattleUnitStatScale::default());

        let value = serde_json::to_value(&preview).expect("serialize combat preview");
        assert!(value.get("enemy_stat_scale").is_none());

        let round_tripped: CombatPreview =
            serde_json::from_value(value).expect("deserialize combat preview");
        assert_eq!(
            round_tripped.enemy_stat_scale,
            BattleUnitStatScale::default()
        );
    }

    #[test]
    fn rumor_threat_warning_uses_only_currently_detectable_missing_tags() {
        let implemented_tags = HashSet::from([
            ThreatWarningTag::ArmoredEnemyPossible,
            ThreatWarningTag::HighMagicResistEnemyPossible,
            ThreatWarningTag::AirEnemyPossible,
            ThreatWarningTag::FastBreakthroughEnemyPossible,
        ]);
        assert_eq!(
            detectable_threat_warning_tags()
                .iter()
                .copied()
                .collect::<HashSet<_>>(),
            implemented_tags
        );
        assert!(rumor_threat_warning_tag(0, &implemented_tags).is_none());
    }

    #[test]
    fn generator_is_deterministic_for_same_request() {
        let data = preview_data_with_encounter(
            "deterministic_encounter",
            CombatNodeType::Defense,
            Some("defense_main"),
        );
        let request = BattlefieldGenerationRequest {
            category: MapNodeCategory::Combat,
            encounter_id: Some("deterministic_encounter"),
            seed: 42,
            floor_index: 0,
        };

        let left = BattlefieldGenerator::generate(request, &data);
        let right = BattlefieldGenerator::generate(request, &data);

        assert_eq!(left, right);
    }

    #[test]
    fn validate_instance_rejects_invalid_battlefield_contracts() {
        let instance = BattlefieldInstance {
            battlefield_template_id: "broken".to_string(),
            node_type: CombatNodeType::Defense,
            mission_variant: CombatMissionVariant::Defense,
            survive_timer_ms: None,
            mission_risk: CombatMissionRisk::Controlled,
            archetype: BattlefieldArchetype::OpenHall,
            size_class: BattlefieldSizeClass::Small,
            width: 0,
            height: 6,
            tiles: Vec::new(),
            valid_tiles: Vec::new(),
            deployment_zones: vec![DeploymentZone {
                id: "deploy".to_string(),
                label: "Deploy".to_string(),
                kind: DeploymentZoneKind::Ground,
                cells: vec![Position::new(0, 0)],
            }],
            spawn_zones: vec![SpawnZone {
                id: "spawn".to_string(),
                label: "Spawn".to_string(),
                kind: SpawnZoneKind::Entry,
                confidence: ZoneConfidence::Confirmed,
                cells: vec![Position::new(0, 5)],
                revealed_details: Vec::new(),
            }],
            routes: Vec::new(),
            spawn_waves: vec![SpawnWave {
                id: "wave_0".to_string(),
                time_ms: 0,
                spawn_zone_ids: vec!["spawn".to_string()],
                route_id: None,
                enemy_entries: Vec::new(),
                required_for_victory: true,
            }],
            obstacles: Vec::new(),
            enemy_briefing: Vec::new(),
            threat_warnings: Vec::new(),
            enemy_stat_scale: Default::default(),
        };

        let result = validate_instance(&instance);

        assert_eq!(result, Err("battlefield dimensions must be positive"));
    }

    #[test]
    fn generator_covers_all_combat_archetypes_with_valid_instances() {
        let data = preview_data_with_encounter(
            "archetype_encounter",
            CombatNodeType::Defense,
            Some("defense_main"),
        );
        let mut archetypes = HashSet::new();

        for seed in 0..128 {
            let instance = BattlefieldGenerator::generate(
                BattlefieldGenerationRequest {
                    category: MapNodeCategory::Combat,
                    encounter_id: Some("archetype_encounter"),
                    seed,
                    floor_index: 0,
                },
                &data,
            );
            validate_instance(&instance).unwrap();
            archetypes.insert(instance.archetype);
            assert!(!instance.deployment_zones.is_empty());
            assert!(!instance.spawn_zones.is_empty());
            assert!(!instance.spawn_waves.is_empty());
            assert_eq!(instance.spawn_waves[0].time_ms, 0);
        }

        assert_eq!(archetypes.len(), 6);
        assert!(!archetypes.contains(&BattlefieldArchetype::SplitRoom));
        assert!(!archetypes.contains(&BattlefieldArchetype::BossArena));
    }

    #[test]
    fn battlefield_generation_policy_is_loaded_from_shared_ron() {
        let policy = BattlefieldGenerationPolicyDatabase::builtin();

        assert_eq!(
            policy
                .archetype(BattlefieldArchetype::BossArena)
                .default_size_class,
            BattlefieldSizeClass::Large
        );
        assert!(!policy.archetype(BattlefieldArchetype::SplitRoom).enabled);
        assert_eq!(policy.archetype(BattlefieldArchetype::SplitRoom).weight, 0);
        assert!(policy.archetype(BattlefieldArchetype::OpenHall).enabled);
        assert_eq!(policy.archetype(BattlefieldArchetype::OpenHall).weight, 1);
        assert_eq!(
            (
                policy.size_class(BattlefieldSizeClass::Medium).width,
                policy.size_class(BattlefieldSizeClass::Medium).height
            ),
            (9, 9)
        );
        assert_eq!(archetype_id(BattlefieldArchetype::SplitRoom), "split_room");
        assert_eq!(size_class_id(BattlefieldSizeClass::Small), "small");
    }

    #[test]
    fn ascii_battlefield_templates_define_non_rectangular_valid_tiles() {
        let data = preview_data_with_encounter(
            "ascii_template_encounter",
            CombatNodeType::Defense,
            Some("defense_main"),
        );

        let instance = BattlefieldGenerator::generate(
            BattlefieldGenerationRequest {
                category: MapNodeCategory::Combat,
                encounter_id: Some("ascii_template_encounter"),
                seed: 1,
                floor_index: 0,
            },
            &data,
        );

        assert_eq!(instance.archetype, BattlefieldArchetype::Corridor);
        assert_eq!((instance.width, instance.height), (9, 9));
        assert!(!instance.valid_tiles.contains(&Position::new(8, 0)));
        assert!(!instance.valid_tiles.contains(&Position::new(0, 0)));
        assert!(instance.valid_tiles.contains(&Position::new(2, 0)));
        assert_eq!(
            instance
                .tiles
                .iter()
                .map(|tile| tile.position)
                .collect::<HashSet<_>>(),
            instance.valid_tiles.iter().copied().collect::<HashSet<_>>()
        );
        assert!(instance
            .deployment_zones
            .iter()
            .flat_map(|zone| zone.cells.iter())
            .all(|cell| instance.valid_tiles.contains(cell)));
        assert!(instance
            .spawn_zones
            .iter()
            .flat_map(|zone| zone.cells.iter())
            .all(|cell| instance.valid_tiles.contains(cell)));
    }

    #[test]
    fn battlefield_template_database_selects_seeded_variants() {
        let database = BattlefieldTemplateDatabase::builtin();

        let first = database.select(
            BattlefieldArchetype::Corridor,
            BattlefieldSizeClass::Medium,
            0,
        );
        let second = database.select(
            BattlefieldArchetype::Corridor,
            BattlefieldSizeClass::Medium,
            1,
        );

        assert_ne!(first.id, second.id);
        assert!(
            database
                .templates
                .iter()
                .filter(|template| {
                    template.archetype == BattlefieldArchetype::Ambush
                        && template.size_class == BattlefieldSizeClass::Medium
                })
                .count()
                >= 3
        );
    }

    #[test]
    fn every_authored_battlefield_template_satisfies_instance_contract() {
        for template in &BattlefieldTemplateDatabase::builtin().templates {
            if matches!(
                template.archetype,
                BattlefieldArchetype::SplitRoom | BattlefieldArchetype::BossArena
            ) {
                continue;
            }
            let mut parsed = parse_battlefield_template(template).unwrap();
            ensure_defense_route(&mut parsed, CombatNodeType::Defense).unwrap();
            let first_spawn_zone_id = parsed
                .spawn_zones
                .first()
                .expect("validated template should have a spawn zone")
                .id
                .clone();
            let route_id = parsed.routes.first().map(|route| route.id.clone());
            let instance = BattlefieldInstance {
                battlefield_template_id: parsed.id,
                node_type: CombatNodeType::Defense,
                mission_variant: CombatMissionVariant::Defense,
                survive_timer_ms: None,
                mission_risk: CombatMissionRisk::Controlled,
                archetype: parsed.archetype,
                size_class: parsed.size_class,
                width: parsed.width,
                height: parsed.height,
                tiles: parsed.tiles,
                valid_tiles: parsed.valid_tiles,
                deployment_zones: parsed.deployment_zones,
                spawn_zones: parsed.spawn_zones,
                routes: parsed.routes,
                spawn_waves: vec![SpawnWave {
                    id: "wave_0".to_string(),
                    time_ms: 0,
                    spawn_zone_ids: vec![first_spawn_zone_id],
                    route_id,
                    enemy_entries: Vec::new(),
                    required_for_victory: true,
                }],
                obstacles: parsed.obstacles,
                enemy_briefing: Vec::new(),
                threat_warnings: Vec::new(),
                enemy_stat_scale: Default::default(),
            };

            validate_instance(&instance).unwrap_or_else(|error| {
                panic!(
                    "authored battlefield template '{}' should satisfy instance contract: {}",
                    instance.battlefield_template_id, error
                )
            });
        }
    }

    #[test]
    fn authored_defense_route_for_non_rectangular_corridor_is_contiguous() {
        let template = BattlefieldTemplateDatabase::builtin()
            .templates
            .iter()
            .find(|template| template.id == "corridor_switchback_01")
            .expect("switchback corridor template should exist");
        let parsed = parse_battlefield_template(template).unwrap();

        let route = parsed
            .routes
            .first()
            .expect("authored defense template should define a route");
        assert_eq!(route.id, "defense_main");
        assert_eq!(route.cells.first().copied(), Some(route.start));
        assert_eq!(route.cells.last().copied(), Some(route.end));
        assert_ne!(
            route.cells,
            vec![route.start, route.end],
            "authored route must not collapse to a two-point diagonal path"
        );
        assert_cardinal_route_cells(&route.cells);
        assert!(route
            .cells
            .iter()
            .all(|cell| parsed.valid_tiles.contains(cell)));
        assert!(route
            .cells
            .iter()
            .all(|cell| !parsed.obstacles.contains(cell)));
    }

    #[test]
    fn battlefield_template_route_overlay_parses_ordered_route() {
        let template = BattlefieldTemplateDefinition {
            id: "route_test".to_string(),
            archetype: BattlefieldArchetype::ChokePoint,
            size_class: BattlefieldSizeClass::Small,
            rows: vec![
                "  N  ".to_string(),
                "  .  ".to_string(),
                "  X  ".to_string(),
                "PPPPP".to_string(),
            ],
            routes: vec![BattlefieldRouteDefinition {
                id: "main_breach".to_string(),
                overlay: vec![
                    "  N  ".to_string(),
                    "  v  ".to_string(),
                    "  X  ".to_string(),
                    "     ".to_string(),
                ],
            }],
        };

        let parsed = parse_battlefield_template(&template).unwrap();

        assert_eq!(
            parsed.routes,
            vec![BattlefieldRoute {
                id: "main_breach".to_string(),
                start: Position::new(2, 0),
                end: Position::new(2, 2),
                cells: vec![
                    Position::new(2, 0),
                    Position::new(2, 1),
                    Position::new(2, 2),
                ],
            }]
        );
    }

    #[test]
    fn battlefield_template_route_overlay_rejects_obstacle_overlap() {
        let template = BattlefieldTemplateDefinition {
            id: "route_obstacle_test".to_string(),
            archetype: BattlefieldArchetype::ChokePoint,
            size_class: BattlefieldSizeClass::Small,
            rows: vec![
                "  N  ".to_string(),
                "  #  ".to_string(),
                "  X  ".to_string(),
                "PPPPP".to_string(),
            ],
            routes: vec![BattlefieldRouteDefinition {
                id: "main_breach".to_string(),
                overlay: vec![
                    "  N  ".to_string(),
                    "  v  ".to_string(),
                    "  X  ".to_string(),
                    "     ".to_string(),
                ],
            }],
        };

        let err = parse_battlefield_template(&template).unwrap_err();

        assert!(err.contains("on obstacle"));
    }

    #[test]
    fn battlefield_template_exposes_ground_and_platform_deployment_zones() {
        let template = BattlefieldTemplateDefinition {
            id: "deployment_kind_test".to_string(),
            archetype: BattlefieldArchetype::OpenHall,
            size_class: BattlefieldSizeClass::Small,
            rows: vec![
                "NNNNN".to_string(),
                ".....".to_string(),
                "PP.TT".to_string(),
            ],
            routes: Vec::new(),
        };

        let parsed = parse_battlefield_template(&template).unwrap();

        let ground = parsed
            .deployment_zones
            .iter()
            .find(|zone| zone.kind == DeploymentZoneKind::Ground)
            .expect("ground deployment zone");
        let platform = parsed
            .deployment_zones
            .iter()
            .find(|zone| zone.kind == DeploymentZoneKind::Platform)
            .expect("platform deployment zone");
        assert_eq!(ground.cells, vec![Position::new(0, 2), Position::new(1, 2)]);
        assert_eq!(
            platform.cells,
            vec![Position::new(3, 2), Position::new(4, 2)]
        );
        assert_eq!(
            parsed
                .tiles
                .iter()
                .find(|tile| tile.position == Position::new(0, 2))
                .map(|tile| tile.kind),
            Some(BattlefieldTileKind::Ground)
        );
        assert_eq!(
            parsed
                .tiles
                .iter()
                .find(|tile| tile.position == Position::new(3, 2))
                .map(|tile| tile.kind),
            Some(BattlefieldTileKind::Platform)
        );
    }

    #[test]
    fn validate_instance_requires_defense_spawn_wave_route() {
        let instance = BattlefieldInstance {
            battlefield_template_id: "defense_without_route".to_string(),
            node_type: CombatNodeType::Defense,
            mission_variant: CombatMissionVariant::Defense,
            survive_timer_ms: None,
            mission_risk: CombatMissionRisk::Controlled,
            archetype: BattlefieldArchetype::ChokePoint,
            size_class: BattlefieldSizeClass::Small,
            width: 1,
            height: 2,
            tiles: vec![
                BattlefieldTile {
                    position: Position::new(0, 0),
                    kind: BattlefieldTileKind::Ground,
                },
                BattlefieldTile {
                    position: Position::new(0, 1),
                    kind: BattlefieldTileKind::Ground,
                },
            ],
            valid_tiles: vec![Position::new(0, 0), Position::new(0, 1)],
            deployment_zones: vec![DeploymentZone {
                id: "deploy".to_string(),
                label: "Deploy".to_string(),
                kind: DeploymentZoneKind::Ground,
                cells: vec![Position::new(0, 0)],
            }],
            spawn_zones: vec![SpawnZone {
                id: "spawn".to_string(),
                label: "Spawn".to_string(),
                kind: SpawnZoneKind::Entry,
                confidence: ZoneConfidence::Confirmed,
                cells: vec![Position::new(0, 1)],
                revealed_details: Vec::new(),
            }],
            routes: Vec::new(),
            spawn_waves: vec![SpawnWave {
                id: "wave_0".to_string(),
                time_ms: 0,
                spawn_zone_ids: vec!["spawn".to_string()],
                route_id: None,
                enemy_entries: Vec::new(),
                required_for_victory: true,
            }],
            obstacles: Vec::new(),
            enemy_briefing: Vec::new(),
            threat_warnings: Vec::new(),
            enemy_stat_scale: Default::default(),
        };

        let result = validate_instance(&instance);

        assert_eq!(result, Err("defense spawn wave must reference a route"));
    }

    #[test]
    fn validate_instance_rejects_non_contiguous_route_cells() {
        let instance = BattlefieldInstance {
            battlefield_template_id: "jump_route".to_string(),
            node_type: CombatNodeType::Defense,
            mission_variant: CombatMissionVariant::Defense,
            survive_timer_ms: None,
            mission_risk: CombatMissionRisk::Controlled,
            archetype: BattlefieldArchetype::OpenHall,
            size_class: BattlefieldSizeClass::Small,
            width: 3,
            height: 3,
            tiles: (0..3)
                .flat_map(|y| {
                    (0..3).map(move |x| BattlefieldTile {
                        position: Position::new(x, y),
                        kind: BattlefieldTileKind::Ground,
                    })
                })
                .collect(),
            valid_tiles: (0..3)
                .flat_map(|y| (0..3).map(move |x| Position::new(x, y)))
                .collect(),
            deployment_zones: vec![DeploymentZone {
                id: "deploy".to_string(),
                label: "Deploy".to_string(),
                kind: DeploymentZoneKind::Ground,
                cells: vec![Position::new(2, 2)],
            }],
            spawn_zones: vec![SpawnZone {
                id: "spawn".to_string(),
                label: "Spawn".to_string(),
                kind: SpawnZoneKind::Entry,
                confidence: ZoneConfidence::Confirmed,
                cells: vec![Position::new(0, 0)],
                revealed_details: Vec::new(),
            }],
            routes: vec![BattlefieldRoute {
                id: "jump".to_string(),
                start: Position::new(0, 0),
                end: Position::new(2, 2),
                cells: vec![Position::new(0, 0), Position::new(2, 2)],
            }],
            spawn_waves: vec![SpawnWave {
                id: "wave_0".to_string(),
                time_ms: 0,
                spawn_zone_ids: vec!["spawn".to_string()],
                route_id: Some("jump".to_string()),
                enemy_entries: Vec::new(),
                required_for_victory: true,
            }],
            obstacles: Vec::new(),
            enemy_briefing: Vec::new(),
            threat_warnings: Vec::new(),
            enemy_stat_scale: Default::default(),
        };

        let result = validate_instance(&instance);

        assert_eq!(result, Err("route cells must be cardinal-adjacent"));
    }

    #[test]
    fn validate_instance_requires_defense_routes_to_share_endpoint() {
        let instance = BattlefieldInstance {
            battlefield_template_id: "defense_split_endpoint".to_string(),
            node_type: CombatNodeType::Defense,
            mission_variant: CombatMissionVariant::Defense,
            survive_timer_ms: None,
            mission_risk: CombatMissionRisk::Controlled,
            archetype: BattlefieldArchetype::ChokePoint,
            size_class: BattlefieldSizeClass::Small,
            width: 3,
            height: 3,
            tiles: (0..3)
                .flat_map(|y| {
                    (0..3).map(move |x| BattlefieldTile {
                        position: Position::new(x, y),
                        kind: BattlefieldTileKind::Ground,
                    })
                })
                .collect(),
            valid_tiles: vec![
                Position::new(0, 0),
                Position::new(1, 0),
                Position::new(2, 0),
                Position::new(0, 1),
                Position::new(1, 1),
                Position::new(2, 1),
                Position::new(0, 2),
                Position::new(1, 2),
                Position::new(2, 2),
            ],
            deployment_zones: vec![DeploymentZone {
                id: "deploy".to_string(),
                label: "Deploy".to_string(),
                kind: DeploymentZoneKind::Ground,
                cells: vec![Position::new(1, 1)],
            }],
            spawn_zones: vec![SpawnZone {
                id: "spawn".to_string(),
                label: "Spawn".to_string(),
                kind: SpawnZoneKind::Entry,
                confidence: ZoneConfidence::Confirmed,
                cells: vec![Position::new(0, 0)],
                revealed_details: Vec::new(),
            }],
            routes: vec![
                BattlefieldRoute {
                    id: "lane_a".to_string(),
                    start: Position::new(0, 0),
                    end: Position::new(2, 0),
                    cells: vec![
                        Position::new(0, 0),
                        Position::new(1, 0),
                        Position::new(2, 0),
                    ],
                },
                BattlefieldRoute {
                    id: "lane_b".to_string(),
                    start: Position::new(0, 2),
                    end: Position::new(2, 2),
                    cells: vec![
                        Position::new(0, 2),
                        Position::new(1, 2),
                        Position::new(2, 2),
                    ],
                },
            ],
            spawn_waves: vec![SpawnWave {
                id: "wave_0".to_string(),
                time_ms: 0,
                spawn_zone_ids: vec!["spawn".to_string()],
                route_id: Some("lane_a".to_string()),
                enemy_entries: Vec::new(),
                required_for_victory: true,
            }],
            obstacles: Vec::new(),
            enemy_briefing: Vec::new(),
            threat_warnings: Vec::new(),
            enemy_stat_scale: Default::default(),
        };

        let result = validate_instance(&instance);

        assert_eq!(
            result,
            Err("defense routes must share one defense object endpoint")
        );
    }

    #[test]
    fn spawn_waves_store_explicit_enemy_entries_separate_from_briefing() {
        let data = GameDataBuilder::empty()
            .with_abnormalities(vec![preview_test_abnormality("display_abno", 0xD15A)])
            .with_corroded_employee_profiles(CorrodedEmployeeProfileDatabase::with_range_presets(
                preview_test_corroded_range_presets(),
                vec![
                    preview_test_corroded_profile("enemy_a", 0xE0A),
                    preview_test_corroded_profile("enemy_b", 0xE0B),
                ],
            ))
            .with_pve(PveEncounterDatabase::new(vec![PveEncounter {
                id: "encounter".to_string(),
                encounter_class: crate::game::data::pve_data::PveEncounterClass::Elite,
                primary_abnormality_id: Some("display_abno".to_string()),
                risk_level: RiskLevel::ZAYIN,
                reward_mode: crate::game::enums::RewardMode::ClaimAll,
                reward_uuids: Vec::new(),
                suppression_research: None,
                node_type: None,
                mission_variant: None,
                survive_timer_ms: None,
                battlefield: None,
                tactical_plan: None,
                win_condition: None,
                waves: vec![PveWaveData {
                    id: "wave_0".to_string(),
                    time_ms: 0,
                    spawn_zone_ids: Vec::new(),
                    route_id: Some("defense_main".to_string()),
                    required_for_victory: true,
                    source: PveWaveSource::Manual(vec![
                        PveWaveEnemyData::Abnormality {
                            abnormality_id: "display_abno".to_string(),
                            tier: Tier::I,
                            count: 1,
                        },
                        PveWaveEnemyData::CorrodedEmployee {
                            profile_id: "enemy_a".to_string(),
                            tier: Tier::I,
                            count: 2,
                        },
                        PveWaveEnemyData::CorrodedEmployee {
                            profile_id: "enemy_b".to_string(),
                            tier: Tier::II,
                            count: 1,
                        },
                    ]),
                }],
                static_obstacles: Vec::new(),
            }]))
            .build();

        let instance = BattlefieldGenerator::generate(
            BattlefieldGenerationRequest {
                category: MapNodeCategory::Combat,
                encounter_id: Some("encounter"),
                seed: 0,
                floor_index: 0,
            },
            &data,
        );

        assert_eq!(instance.enemy_briefing[0].abnormality_id, "display_abno");
        assert_eq!(instance.enemy_briefing[0].kind, EnemyKind::Abnormality);
        let entries = &instance.spawn_waves[0].enemy_entries;
        assert_eq!(
            entries[0],
            SpawnWaveEnemyEntry {
                kind: EnemyKind::Abnormality,
                profile_id: None,
                abnormality_id: "display_abno".to_string(),
                tier: Tier::I,
                count: 1,
                appearance_seeds: Vec::new(),
            }
        );
        assert_eq!(entries[1].kind, EnemyKind::CorrodedEmployee);
        assert_eq!(entries[1].profile_id.as_deref(), Some("enemy_a"));
        assert_eq!(entries[1].count, 2);
        assert_eq!(entries[1].appearance_seeds.len(), 2);
        assert_eq!(entries[2].kind, EnemyKind::CorrodedEmployee);
        assert_eq!(entries[2].profile_id.as_deref(), Some("enemy_b"));
        assert_eq!(entries[2].count, 1);
        assert_eq!(entries[2].appearance_seeds.len(), 1);
    }

    #[test]
    fn generated_corroded_wave_source_resolves_during_preview_generation() {
        let data = GameDataBuilder::empty()
            .with_abnormalities(vec![preview_test_abnormality("display_abno", 0xD15A)])
            .with_corroded_employee_profiles(CorrodedEmployeeProfileDatabase::with_range_presets(
                preview_test_corroded_range_presets(),
                vec![
                    preview_test_corroded_profile("corroded_guard", 0xC01),
                    preview_test_corroded_profile("corroded_rusher", 0xC02),
                ],
            ))
            .with_corroded_wave_presets(CorrodedWavePresetDatabase::new(vec![CorrodedWavePreset {
                id: "test_corroded_mix".to_string(),
                pressure: "test".to_string(),
                preferred_node_types: vec![CombatNodeType::Defense],
                preferred_risk_levels: vec![RiskLevel::ZAYIN],
                budget: 3,
                count_range: CorrodedWaveCountRange { min: 2, max: 3 },
                role_mix: vec![
                    CorrodedWaveRoleWeight {
                        profile_id: "corroded_guard".to_string(),
                        weight: 3,
                        cost: 1,
                        min_count: 1,
                        max_count: Some(2),
                        tier: Tier::I,
                    },
                    CorrodedWaveRoleWeight {
                        profile_id: "corroded_rusher".to_string(),
                        weight: 1,
                        cost: 1,
                        min_count: 0,
                        max_count: Some(2),
                        tier: Tier::I,
                    },
                ],
            }]))
            .with_pve(PveEncounterDatabase::new(vec![PveEncounter {
                id: "generated_corroded_encounter".to_string(),
                encounter_class: crate::game::data::pve_data::PveEncounterClass::Normal,
                primary_abnormality_id: None,
                risk_level: RiskLevel::ZAYIN,
                reward_mode: crate::game::enums::RewardMode::ClaimAll,
                reward_uuids: Vec::new(),
                suppression_research: None,
                node_type: Some(CombatNodeType::Defense),
                mission_variant: None,
                survive_timer_ms: None,
                battlefield: None,
                tactical_plan: None,
                win_condition: None,
                waves: vec![PveWaveData {
                    id: "wave_0".to_string(),
                    time_ms: 0,
                    spawn_zone_ids: Vec::new(),
                    route_id: Some("defense_main".to_string()),
                    required_for_victory: true,
                    source: PveWaveSource::GeneratedCorroded {
                        preset_id: "test_corroded_mix".to_string(),
                        budget_override: None,
                        seed_salt: Some(7),
                    },
                }],
                static_obstacles: Vec::new(),
            }]))
            .build();

        let first = BattlefieldGenerator::generate(
            BattlefieldGenerationRequest {
                category: MapNodeCategory::Combat,
                encounter_id: Some("generated_corroded_encounter"),
                seed: 44,
                floor_index: 0,
            },
            &data,
        );
        let second = BattlefieldGenerator::generate(
            BattlefieldGenerationRequest {
                category: MapNodeCategory::Combat,
                encounter_id: Some("generated_corroded_encounter"),
                seed: 44,
                floor_index: 0,
            },
            &data,
        );

        assert_eq!(first.spawn_waves, second.spawn_waves);
        let entries = &first.spawn_waves[0].enemy_entries;
        assert!(entries
            .iter()
            .all(|entry| entry.kind == EnemyKind::CorrodedEmployee));
        assert!(entries.iter().any(|entry| {
            entry.profile_id.as_deref() == Some("corroded_guard") && entry.count >= 1
        }));
        assert_eq!(
            entries.iter().map(|entry| entry.count).sum::<u32>() as usize,
            entries
                .iter()
                .map(|entry| entry.appearance_seeds.len())
                .sum::<usize>()
        );
        assert_eq!(
            first.enemy_briefing[0].count_hint,
            entries.iter().map(|entry| entry.count).sum::<u32>()
        );
        assert_eq!(first.enemy_briefing[0].kind, EnemyKind::CorrodedEmployee);

        let combat_preview = CombatPreview::generate_for_node(
            MapNodeId::new(Uuid::from_u128(0xC0_660D)),
            MapNodeCategory::Combat,
            Some("generated_corroded_encounter"),
            &data,
            44,
        );
        assert_eq!(combat_preview.spawn_waves, first.spawn_waves);
        let spawn_groups =
            crate::game::combat_setup::enemy_spawns::enemy_spawn_groups_from_preview(
                &data,
                "generated_corroded_encounter",
                &combat_preview,
            )
            .expect("generated corroded preview should build runtime enemy spawn groups");
        let spawned_sources = spawn_groups
            .iter()
            .flat_map(|(group, _)| group.spawns.iter())
            .map(|spawn| &spawn.draft.source)
            .collect::<Vec<_>>();
        assert_eq!(
            spawned_sources.len() as u32,
            entries.iter().map(|entry| entry.count).sum::<u32>()
        );
        assert!(spawned_sources.iter().all(|source| matches!(
            source,
            crate::game::battle::types::BattleUnitSource::CorrodedEmployee { .. }
        )));
    }

    #[test]
    fn authored_encounter_battlefield_overrides_drive_generated_preview() {
        let data = GameDataBuilder::empty()
            .with_abnormalities(vec![
                preview_test_abnormality("display_abno", 0xD15A),
                preview_test_abnormality("enemy_a", 0xE0A),
            ])
            .with_pve(PveEncounterDatabase::new(vec![PveEncounter {
                id: "authored_route".to_string(),
                encounter_class: crate::game::data::pve_data::PveEncounterClass::Elite,
                primary_abnormality_id: Some("enemy_a".to_string()),
                risk_level: RiskLevel::ZAYIN,
                reward_mode: crate::game::enums::RewardMode::ClaimAll,
                reward_uuids: Vec::new(),
                suppression_research: None,
                node_type: Some(CombatNodeType::Defense),
                mission_variant: None,
                survive_timer_ms: None,
                battlefield: Some(PveBattlefieldOverrideData {
                    archetype: Some(BattlefieldArchetype::ChokePoint),
                    size_class: Some(BattlefieldSizeClass::Small),
                }),
                tactical_plan: None,
                win_condition: None,
                waves: vec![PveWaveData {
                    id: "wave_0".to_string(),
                    time_ms: 0,
                    spawn_zone_ids: vec!["north_entry".to_string()],
                    route_id: Some("defense_main".to_string()),
                    required_for_victory: false,
                    source: PveWaveSource::Manual(vec![PveWaveEnemyData::Abnormality {
                        abnormality_id: "enemy_a".to_string(),
                        tier: Tier::I,
                        count: 1,
                    }]),
                }],
                static_obstacles: Vec::new(),
            }]))
            .build();

        let instance = BattlefieldGenerator::generate(
            BattlefieldGenerationRequest {
                category: MapNodeCategory::Combat,
                encounter_id: Some("authored_route"),
                seed: 0,
                floor_index: 0,
            },
            &data,
        );

        assert_eq!(instance.node_type, CombatNodeType::Defense);
        assert_eq!(instance.archetype, BattlefieldArchetype::ChokePoint);
        assert_eq!(instance.size_class, BattlefieldSizeClass::Small);
        assert_eq!((instance.width, instance.height), (7, 7));
        assert_eq!(instance.spawn_waves[0].spawn_zone_ids, ["north_entry"]);
        assert!(!instance.spawn_waves[0].required_for_victory);
    }

    #[test]
    fn authored_static_obstacles_replace_archetype_default_obstacles() {
        let data = GameDataBuilder::empty()
            .with_abnormalities(vec![
                preview_test_abnormality("display_abno", 0xD15A),
                preview_test_abnormality("enemy_a", 0xE0A),
            ])
            .with_pve(PveEncounterDatabase::new(vec![PveEncounter {
                id: "authored_obstacles".to_string(),
                encounter_class: crate::game::data::pve_data::PveEncounterClass::Elite,
                primary_abnormality_id: Some("enemy_a".to_string()),
                risk_level: RiskLevel::ZAYIN,
                reward_mode: crate::game::enums::RewardMode::ClaimAll,
                reward_uuids: Vec::new(),
                suppression_research: None,
                node_type: Some(CombatNodeType::Defense),
                mission_variant: None,
                survive_timer_ms: None,
                battlefield: Some(PveBattlefieldOverrideData {
                    archetype: Some(BattlefieldArchetype::ObstacleRoom),
                    size_class: Some(BattlefieldSizeClass::Medium),
                }),
                tactical_plan: None,
                win_condition: None,
                waves: vec![PveWaveData {
                    id: "wave_0".to_string(),
                    time_ms: 0,
                    spawn_zone_ids: Vec::new(),
                    route_id: Some("defense_main".to_string()),
                    required_for_victory: true,
                    source: PveWaveSource::Manual(vec![PveWaveEnemyData::Abnormality {
                        abnormality_id: "enemy_a".to_string(),
                        tier: Tier::I,
                        count: 1,
                    }]),
                }],
                static_obstacles: vec![
                    PveStaticObstacleData {
                        position: PvePosition { x: 3, y: 3 },
                    },
                    PveStaticObstacleData {
                        position: PvePosition { x: 3, y: 3 },
                    },
                    PveStaticObstacleData {
                        position: PvePosition { x: 5, y: 3 },
                    },
                ],
            }]))
            .build();

        let instance = BattlefieldGenerator::generate(
            BattlefieldGenerationRequest {
                category: MapNodeCategory::Combat,
                encounter_id: Some("authored_obstacles"),
                seed: 0,
                floor_index: 0,
            },
            &data,
        );

        assert_eq!(
            instance.obstacles,
            vec![Position::new(3, 3), Position::new(5, 3)]
        );
        assert!(!instance.obstacles.contains(&Position::new(2, 1)));
        assert_eq!(
            instance
                .tiles
                .iter()
                .find(|tile| tile.position == Position::new(3, 3))
                .map(|tile| tile.kind),
            Some(BattlefieldTileKind::Obstacle)
        );
        assert_ne!(
            instance
                .tiles
                .iter()
                .find(|tile| tile.position == Position::new(2, 1))
                .map(|tile| tile.kind),
            Some(BattlefieldTileKind::Obstacle)
        );
    }

    #[test]
    fn boss_generation_uses_boss_arena_and_boss_anchor() {
        let data = preview_data_with_encounter("boss_encounter", CombatNodeType::Boss, None);

        let instance = BattlefieldGenerator::generate(
            BattlefieldGenerationRequest {
                category: MapNodeCategory::Boss,
                encounter_id: Some("boss_encounter"),
                seed: 99,
                floor_index: 0,
            },
            &data,
        );

        validate_instance(&instance).unwrap();
        assert_eq!(instance.archetype, BattlefieldArchetype::BossArena);
        assert_eq!(instance.size_class, BattlefieldSizeClass::Large);
        assert!(instance
            .spawn_zones
            .iter()
            .any(|zone| zone.kind == SpawnZoneKind::BossAnchor));
    }
}
