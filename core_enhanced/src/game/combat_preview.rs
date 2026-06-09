use crate::game::{
    battle::types::DeploymentAffinity,
    behavior::GameError,
    combat_balance::{is_fast_breakthrough_speed, is_high_defense, is_high_magic_resist},
    combat_mission_policy::CombatMissionPolicy,
    data::{
        corroded_wave_data::{CorrodedWavePreset, CorrodedWaveRoleWeight},
        pve_data::{PveEncounter, PveWaveData},
        GameDataBase,
    },
    determinism,
    enums::{RiskLevel, Tier},
    map::{MapNodeCategory, MapNodeId},
    resources::Position,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet, VecDeque},
    sync::OnceLock,
};

const CORRODED_APPEARANCE_SEED_NS: u64 = 0x434F_5241_5050_4541; // "CORAPPEA"
const THREAT_RUMOR_SEED_NS: u64 = 0x5448_5254_5255_4D52; // "THRTRUMR"
const THREAT_RUMOR_CHANCE_PERCENT: u8 = 20;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BattlefieldArchetype {
    OpenHall,
    Corridor,
    ChokePoint,
    Ambush,
    Surrounded,
    SplitRoom,
    ObstacleRoom,
    BossArena,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CombatNodeType {
    Defense,
    Boss,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CombatMissionVariant {
    Defense,
    Encirclement,
    SplitRoom,
    Boss,
}

impl CombatMissionVariant {
    pub fn default_for_node_type(node_type: CombatNodeType) -> Self {
        match node_type {
            CombatNodeType::Defense => Self::Defense,
            CombatNodeType::Boss => Self::Boss,
        }
    }

    pub fn is_compatible_with(self, node_type: CombatNodeType) -> bool {
        matches!(
            (node_type, self),
            (CombatNodeType::Defense, Self::Defense)
                | (CombatNodeType::Defense, Self::Encirclement)
                | (CombatNodeType::Defense, Self::SplitRoom)
                | (CombatNodeType::Boss, Self::Boss)
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CombatMissionRisk {
    Controlled,
    Unstable,
    Collapse,
}

impl CombatMissionRisk {
    pub fn from_risk_level(risk_level: RiskLevel) -> Self {
        CombatMissionPolicy::mission_risk_from_risk_level(risk_level)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BattlefieldSizeClass {
    Small,
    Medium,
    Large,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ZoneConfidence {
    Confirmed,
    Likely,
    Suspected,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SpawnZoneKind {
    Entry,
    Interior,
    BossAnchor,
    Ambush,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BattlefieldTileKind {
    Ground,
    Platform,
    Obstacle,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BattlefieldTile {
    pub position: Position,
    pub kind: BattlefieldTileKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EnemyKind {
    CorrodedEmployee,
    Abnormality,
    FacilityEntity,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeploymentZone {
    pub id: String,
    pub label: String,
    pub kind: DeploymentZoneKind,
    pub cells: Vec<Position>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DeploymentZoneKind {
    Ground,
    Platform,
}

impl DeploymentZoneKind {
    pub fn supports_affinity(self, affinity: DeploymentAffinity) -> bool {
        match self {
            DeploymentZoneKind::Ground => affinity.allows_ground(),
            DeploymentZoneKind::Platform => affinity.allows_platform(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpawnZone {
    pub id: String,
    pub label: String,
    pub kind: SpawnZoneKind,
    pub confidence: ZoneConfidence,
    pub cells: Vec<Position>,
    pub revealed_details: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EnemyBriefing {
    pub kind: EnemyKind,
    pub abnormality_id: String,
    pub display_name: String,
    pub risk_level: RiskLevel,
    pub role: String,
    pub count_hint: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ThreatWarningTag {
    ArmoredEnemyPossible,
    HighMagicResistEnemyPossible,
    AirEnemyPossible,
    HardToBlockEnemyPossible,
    ShieldedEnemyPossible,
    RegeneratingEnemyPossible,
    FastBreakthroughEnemyPossible,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ThreatWarningStatus {
    Unverified,
    Disproved,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ThreatWarningSource {
    Briefing,
    Rumor,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ThreatWarning {
    pub tag: ThreatWarningTag,
    pub status: ThreatWarningStatus,
    pub source: ThreatWarningSource,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpawnWaveEnemyEntry {
    pub kind: EnemyKind,
    pub profile_id: Option<String>,
    pub abnormality_id: String,
    pub tier: Tier,
    pub count: u32,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub appearance_seeds: Vec<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpawnWave {
    pub id: String,
    pub time_ms: u32,
    pub spawn_zone_ids: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub route_id: Option<String>,
    pub enemy_entries: Vec<SpawnWaveEnemyEntry>,
    pub required_for_victory: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BattlefieldRoute {
    pub id: String,
    pub start: Position,
    pub end: Position,
    pub cells: Vec<Position>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BattlefieldInstance {
    pub battlefield_template_id: String,
    pub node_type: CombatNodeType,
    pub mission_variant: CombatMissionVariant,
    pub mission_risk: CombatMissionRisk,
    pub archetype: BattlefieldArchetype,
    pub size_class: BattlefieldSizeClass,
    pub width: i32,
    pub height: i32,
    pub tiles: Vec<BattlefieldTile>,
    pub valid_tiles: Vec<Position>,
    pub deployment_zones: Vec<DeploymentZone>,
    pub spawn_zones: Vec<SpawnZone>,
    pub routes: Vec<BattlefieldRoute>,
    pub spawn_waves: Vec<SpawnWave>,
    pub obstacles: Vec<Position>,
    pub enemy_briefing: Vec<EnemyBriefing>,
    pub threat_warnings: Vec<ThreatWarning>,
}

#[derive(Debug, Clone, Copy)]
pub struct BattlefieldGenerationRequest<'a> {
    pub category: MapNodeCategory,
    pub encounter_id: Option<&'a str>,
    pub seed: u64,
}

pub struct BattlefieldGenerator;

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
    fn builtin() -> &'static Self {
        static DATABASE: OnceLock<BattlefieldGenerationPolicyDatabase> = OnceLock::new();
        DATABASE.get_or_init(|| {
            let database: BattlefieldGenerationPolicyDatabase = ron::de::from_str(include_str!(
                "../../../game_resources/data/map/battlefield_archetypes.ron"
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
        }
        for archetype in expected_archetypes {
            if !archetypes.contains(&archetype) {
                return Err(format!(
                    "battlefield policy is missing archetype {:?}",
                    archetype
                ));
            }
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
}

impl BattlefieldTemplateDatabase {
    fn builtin() -> &'static Self {
        static DATABASE: OnceLock<BattlefieldTemplateDatabase> = OnceLock::new();
        DATABASE.get_or_init(|| {
            let templates: Vec<BattlefieldTemplateDefinition> = ron::de::from_str(include_str!(
                "../../../game_resources/data/map/battlefield_templates.ron"
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
            parse_battlefield_template(template)?;
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
            .unwrap_or_else(|| {
                CombatMissionPolicy::fallback_mission_variant_for_archetype(node_type, archetype)
            });
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
        ensure_defense_route(&mut battlefield_template, node_type);
        let spawn_waves = spawn_waves_for(
            node_type,
            archetype,
            &battlefield_template.spawn_zones,
            &battlefield_template.routes,
            encounter,
            game_data,
            request.seed,
        );
        let enemy_briefing = enemy_briefing(encounter, game_data, request.category, &spawn_waves);
        let threat_warnings = threat_warnings_for(&spawn_waves, game_data, request.seed);
        Ok(BattlefieldInstance {
            battlefield_template_id: battlefield_template.id,
            node_type,
            mission_variant,
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
        })
    }
}

fn ensure_defense_route(
    battlefield_template: &mut crate::game::combat_preview::ParsedBattlefieldTemplate,
    node_type: CombatNodeType,
) {
    if node_type != CombatNodeType::Defense || !battlefield_template.routes.is_empty() {
        return;
    }

    let start = battlefield_template
        .spawn_zones
        .first()
        .and_then(|zone| zone.cells.first())
        .copied()
        .unwrap_or(Position::new(battlefield_template.width / 2, 0));
    let end = default_generated_route_end(battlefield_template);
    battlefield_template.routes.push(BattlefieldRoute {
        id: "generated_defense_route".to_string(),
        start,
        end,
        cells: generated_route_cells(start, end),
    });
}

fn default_generated_route_end(
    battlefield_template: &crate::game::combat_preview::ParsedBattlefieldTemplate,
) -> Position {
    let cells = battlefield_template
        .deployment_zones
        .first()
        .map(|zone| zone.cells.as_slice())
        .unwrap_or(&[]);
    if cells.is_empty() {
        return Position::new(
            battlefield_template.width / 2,
            battlefield_template.height.saturating_sub(1),
        );
    }

    let sum_x = cells.iter().map(|cell| cell.x).sum::<i32>();
    let sum_y = cells.iter().map(|cell| cell.y).sum::<i32>();
    Position::new(
        sum_x / i32::try_from(cells.len()).unwrap_or(1),
        sum_y / i32::try_from(cells.len()).unwrap_or(1),
    )
}

fn generated_route_cells(start: Position, end: Position) -> Vec<Position> {
    if start == end {
        vec![start]
    } else {
        vec![start, end]
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CombatPreview {
    pub node_id: MapNodeId,
    pub encounter_id: Option<String>,
    pub battlefield_template_id: String,
    pub node_type: CombatNodeType,
    pub mission_variant: CombatMissionVariant,
    pub mission_risk: CombatMissionRisk,
    pub archetype: BattlefieldArchetype,
    pub size_class: BattlefieldSizeClass,
    pub width: i32,
    pub height: i32,
    pub tiles: Vec<BattlefieldTile>,
    pub valid_tiles: Vec<Position>,
    pub deployment_zones: Vec<DeploymentZone>,
    pub spawn_zones: Vec<SpawnZone>,
    pub routes: Vec<BattlefieldRoute>,
    pub spawn_waves: Vec<SpawnWave>,
    pub obstacles: Vec<Position>,
    pub enemy_briefing: Vec<EnemyBriefing>,
    #[serde(default)]
    pub threat_warnings: Vec<ThreatWarning>,
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
            },
            game_data,
        )?;

        Ok(Self {
            node_id,
            encounter_id: encounter_id.map(str::to_string),
            battlefield_template_id: instance.battlefield_template_id,
            node_type: instance.node_type,
            mission_variant: instance.mission_variant,
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

    match seed % 6 {
        0 => BattlefieldArchetype::OpenHall,
        1 => BattlefieldArchetype::Corridor,
        2 => BattlefieldArchetype::ChokePoint,
        3 => BattlefieldArchetype::Ambush,
        4 => BattlefieldArchetype::Surrounded,
        _ => BattlefieldArchetype::ObstacleRoom,
    }
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

fn parse_battlefield_template(
    template: &BattlefieldTemplateDefinition,
) -> Result<ParsedBattlefieldTemplate, String> {
    if template.rows.is_empty() {
        return Err(format!(
            "battlefield template '{}' has no rows",
            template.id
        ));
    }

    let height = i32::try_from(template.rows.len()).map_err(|_| {
        format!(
            "battlefield template '{}' height does not fit i32",
            template.id
        )
    })?;
    let width = template
        .rows
        .iter()
        .map(|row| row.chars().count())
        .max()
        .and_then(|width| i32::try_from(width).ok())
        .ok_or_else(|| format!("battlefield template '{}' has invalid width", template.id))?;
    if width <= 0 || height <= 0 {
        return Err(format!(
            "battlefield template '{}' dimensions must be positive",
            template.id
        ));
    }

    let mut valid_tiles = Vec::new();
    let mut tiles = Vec::new();
    let mut ground_deployment_cells = Vec::new();
    let mut platform_deployment_cells = Vec::new();
    let mut obstacles = Vec::new();
    let mut north_entry = Vec::new();
    let mut north_west_entry = Vec::new();
    let mut north_east_entry = Vec::new();
    let mut side_ambush = Vec::new();
    let mut boss_anchor = Vec::new();
    let mut west_reinforcement = Vec::new();
    let mut east_reinforcement = Vec::new();

    for (y, row) in template.rows.iter().enumerate() {
        for (x, tile) in row.chars().enumerate() {
            if tile == ' ' {
                continue;
            }

            let position = Position::new(x as i32, y as i32);
            valid_tiles.push(position);
            let tile_kind = match tile {
                '.' | 'P' | 'N' | 'L' | 'Q' | 'A' | 'B' | 'W' | 'R' | 'X' | 'Y' | 'Z' => {
                    BattlefieldTileKind::Ground
                }
                'T' => BattlefieldTileKind::Platform,
                '#' => BattlefieldTileKind::Obstacle,
                other => {
                    return Err(format!(
                        "battlefield template '{}' contains unsupported tile '{}'",
                        template.id, other
                    ));
                }
            };
            tiles.push(BattlefieldTile {
                position,
                kind: tile_kind,
            });
            match tile {
                '.' => {}
                '#' => obstacles.push(position),
                'P' => ground_deployment_cells.push(position),
                'T' => platform_deployment_cells.push(position),
                'N' => north_entry.push(position),
                'L' => north_west_entry.push(position),
                'Q' => north_east_entry.push(position),
                'A' => side_ambush.push(position),
                'B' => boss_anchor.push(position),
                'W' => west_reinforcement.push(position),
                'R' => east_reinforcement.push(position),
                'X' | 'Y' | 'Z' => {}
                _ => unreachable!("unsupported battlefield tile was rejected above"),
            }
        }
    }

    let valid_tiles = unique_positions(valid_tiles);
    let tiles = unique_tiles(tiles);
    let ground_deployment_cells = unique_positions(ground_deployment_cells);
    let platform_deployment_cells = unique_positions(platform_deployment_cells);
    if ground_deployment_cells.is_empty() && platform_deployment_cells.is_empty() {
        return Err(format!(
            "battlefield template '{}' must contain at least one deployment tile",
            template.id
        ));
    }

    let mut deployment_zones = Vec::new();
    push_deployment_zone(
        &mut deployment_zones,
        "ground_deployment",
        "Ground Deployment",
        DeploymentZoneKind::Ground,
        ground_deployment_cells,
    );
    push_deployment_zone(
        &mut deployment_zones,
        "platform_deployment",
        "Platform Deployment",
        DeploymentZoneKind::Platform,
        platform_deployment_cells,
    );
    let mut spawn_zones = Vec::new();
    push_spawn_zone(
        &mut spawn_zones,
        "north_entry",
        "North Entry",
        SpawnZoneKind::Entry,
        ZoneConfidence::Confirmed,
        north_entry,
    );
    push_spawn_zone(
        &mut spawn_zones,
        "north_west_entry",
        "North-West Entry",
        SpawnZoneKind::Entry,
        ZoneConfidence::Likely,
        north_west_entry,
    );
    push_spawn_zone(
        &mut spawn_zones,
        "north_east_entry",
        "North-East Entry",
        SpawnZoneKind::Entry,
        ZoneConfidence::Likely,
        north_east_entry,
    );
    push_spawn_zone(
        &mut spawn_zones,
        "side_ambush",
        "Side Ambush",
        SpawnZoneKind::Ambush,
        confidence_for_hidden(),
        side_ambush,
    );
    push_spawn_zone(
        &mut spawn_zones,
        "boss_anchor",
        "Boss Anchor",
        SpawnZoneKind::BossAnchor,
        ZoneConfidence::Confirmed,
        boss_anchor,
    );
    push_spawn_zone(
        &mut spawn_zones,
        "west_reinforcement",
        "West Reinforcement",
        SpawnZoneKind::Entry,
        confidence_for_hidden(),
        west_reinforcement,
    );
    push_spawn_zone(
        &mut spawn_zones,
        "east_reinforcement",
        "East Reinforcement",
        SpawnZoneKind::Entry,
        confidence_for_hidden(),
        east_reinforcement,
    );
    if spawn_zones.is_empty() {
        return Err(format!(
            "battlefield template '{}' must contain at least one spawn tile",
            template.id
        ));
    }
    let obstacle_positions = obstacles.iter().copied().collect::<HashSet<_>>();
    let valid_positions = valid_tiles.iter().copied().collect::<HashSet<_>>();
    let routes = parse_battlefield_routes(
        template,
        width,
        height,
        &valid_positions,
        &obstacle_positions,
    )?;

    Ok(ParsedBattlefieldTemplate {
        id: template.id.clone(),
        archetype: template.archetype,
        size_class: template.size_class,
        width,
        height,
        tiles,
        valid_tiles,
        deployment_zones,
        spawn_zones,
        routes,
        obstacles: unique_positions(obstacles),
    })
}

fn parse_battlefield_routes(
    template: &BattlefieldTemplateDefinition,
    width: i32,
    height: i32,
    valid_tiles: &HashSet<Position>,
    obstacles: &HashSet<Position>,
) -> Result<Vec<BattlefieldRoute>, String> {
    let mut route_ids = HashSet::new();
    let mut routes = Vec::new();
    for route in &template.routes {
        if route.id.trim().is_empty() {
            return Err(format!(
                "battlefield template '{}' has route with empty id",
                template.id
            ));
        }
        if !route_ids.insert(route.id.as_str()) {
            return Err(format!(
                "battlefield template '{}' has duplicate route id '{}'",
                template.id, route.id
            ));
        }
        routes.push(parse_battlefield_route(
            template,
            route,
            width,
            height,
            valid_tiles,
            obstacles,
        )?);
    }
    Ok(routes)
}

fn parse_battlefield_route(
    template: &BattlefieldTemplateDefinition,
    route: &BattlefieldRouteDefinition,
    width: i32,
    height: i32,
    valid_tiles: &HashSet<Position>,
    obstacles: &HashSet<Position>,
) -> Result<BattlefieldRoute, String> {
    if route.overlay.len() != height as usize {
        return Err(format!(
            "route '{}' in battlefield template '{}' must have {} rows",
            route.id, template.id, height
        ));
    }

    let mut route_chars: HashMap<Position, char> = HashMap::new();
    for (y, row) in route.overlay.iter().enumerate() {
        let row_width = row.chars().count();
        if row_width != width as usize {
            return Err(format!(
                "route '{}' in battlefield template '{}' row {} width must be {} but was {}",
                route.id, template.id, y, width, row_width
            ));
        }
        for (x, ch) in row.chars().enumerate() {
            if ch == ' ' {
                continue;
            }
            let position = Position::new(x as i32, y as i32);
            if !valid_tiles.contains(&position) {
                return Err(format!(
                    "route '{}' in battlefield template '{}' places '{}' outside valid terrain at ({}, {})",
                    route.id, template.id, ch, position.x, position.y
                ));
            }
            if obstacles.contains(&position) {
                return Err(format!(
                    "route '{}' in battlefield template '{}' places '{}' on obstacle at ({}, {})",
                    route.id, template.id, ch, position.x, position.y
                ));
            }
            if !is_route_arrow(ch) && !is_route_marker(ch) {
                return Err(format!(
                    "route '{}' in battlefield template '{}' contains unsupported marker '{}'",
                    route.id, template.id, ch
                ));
            }
            if is_route_marker(ch) {
                let terrain = terrain_char_at(template, position);
                if terrain != Some(ch) {
                    return Err(format!(
                        "route '{}' in battlefield template '{}' marker '{}' must match terrain marker at ({}, {})",
                        route.id, template.id, ch, position.x, position.y
                    ));
                }
            }
            route_chars.insert(position, ch);
        }
    }

    if route_chars.len() < 2 {
        return Err(format!(
            "route '{}' in battlefield template '{}' must contain at least a start and end marker",
            route.id, template.id
        ));
    }

    let mut outgoing: HashMap<Position, Position> = HashMap::new();
    let mut incoming_counts: HashMap<Position, usize> = HashMap::new();
    let mut marker_positions = Vec::new();

    for (&position, &ch) in &route_chars {
        if is_route_arrow(ch) {
            let next = route_arrow_destination(position, ch);
            if !route_chars.contains_key(&next) {
                return Err(format!(
                    "route '{}' in battlefield template '{}' arrow '{}' at ({}, {}) points outside route",
                    route.id, template.id, ch, position.x, position.y
                ));
            }
            outgoing.insert(position, next);
            *incoming_counts.entry(next).or_default() += 1;
        } else {
            marker_positions.push(position);
        }
    }

    if marker_positions.len() != 2 {
        return Err(format!(
            "route '{}' in battlefield template '{}' must contain exactly two endpoint markers",
            route.id, template.id
        ));
    }

    for &marker in &marker_positions {
        let candidates = route_marker_outgoing_candidates(marker, &route_chars);
        if candidates.len() > 1 {
            return Err(format!(
                "route '{}' in battlefield template '{}' marker at ({}, {}) branches to multiple arrows",
                route.id, template.id, marker.x, marker.y
            ));
        }
        if let Some(next) = candidates.first().copied() {
            outgoing.insert(marker, next);
            *incoming_counts.entry(next).or_default() += 1;
        }
    }

    let starts = marker_positions
        .iter()
        .copied()
        .filter(|position| {
            incoming_counts.get(position).copied().unwrap_or(0) == 0
                && outgoing.contains_key(position)
        })
        .collect::<Vec<_>>();
    let ends = marker_positions
        .iter()
        .copied()
        .filter(|position| {
            incoming_counts.get(position).copied().unwrap_or(0) == 1
                && !outgoing.contains_key(position)
        })
        .collect::<Vec<_>>();

    if starts.len() != 1 || ends.len() != 1 {
        return Err(format!(
            "route '{}' in battlefield template '{}' must resolve to one start and one end",
            route.id, template.id
        ));
    }

    for &position in route_chars.keys() {
        let incoming = incoming_counts.get(&position).copied().unwrap_or(0);
        if incoming > 1 {
            return Err(format!(
                "route '{}' in battlefield template '{}' branches into ({}, {})",
                route.id, template.id, position.x, position.y
            ));
        }
        if is_route_arrow(route_chars[&position]) && !outgoing.contains_key(&position) {
            return Err(format!(
                "route '{}' in battlefield template '{}' arrow at ({}, {}) has no outgoing edge",
                route.id, template.id, position.x, position.y
            ));
        }
    }

    let start = starts[0];
    let end = ends[0];
    let mut cells = Vec::new();
    let mut visited = HashSet::new();
    let mut current = start;
    loop {
        if !visited.insert(current) {
            return Err(format!(
                "route '{}' in battlefield template '{}' contains a loop",
                route.id, template.id
            ));
        }
        cells.push(current);
        if current == end {
            break;
        }
        current = *outgoing.get(&current).ok_or_else(|| {
            format!(
                "route '{}' in battlefield template '{}' is disconnected before reaching end",
                route.id, template.id
            )
        })?;
    }

    if cells.len() != route_chars.len() {
        return Err(format!(
            "route '{}' in battlefield template '{}' has disconnected cells",
            route.id, template.id
        ));
    }

    Ok(BattlefieldRoute {
        id: route.id.clone(),
        start,
        end,
        cells,
    })
}

fn terrain_char_at(template: &BattlefieldTemplateDefinition, position: Position) -> Option<char> {
    template
        .rows
        .get(position.y as usize)
        .and_then(|row| row.chars().nth(position.x as usize))
}

fn is_route_arrow(ch: char) -> bool {
    matches!(ch, '>' | '<' | '^' | 'v')
}

fn is_route_marker(ch: char) -> bool {
    ch.is_ascii_uppercase()
}

fn route_arrow_destination(position: Position, arrow: char) -> Position {
    match arrow {
        '>' => Position::new(position.x + 1, position.y),
        '<' => Position::new(position.x - 1, position.y),
        '^' => Position::new(position.x, position.y - 1),
        'v' => Position::new(position.x, position.y + 1),
        _ => position,
    }
}

fn route_marker_outgoing_candidates(
    marker: Position,
    route_chars: &HashMap<Position, char>,
) -> Vec<Position> {
    [
        (Position::new(marker.x + 1, marker.y), '>'),
        (Position::new(marker.x - 1, marker.y), '<'),
        (Position::new(marker.x, marker.y - 1), '^'),
        (Position::new(marker.x, marker.y + 1), 'v'),
    ]
    .into_iter()
    .filter_map(|(position, expected)| {
        route_chars
            .get(&position)
            .is_some_and(|ch| *ch == expected)
            .then_some(position)
    })
    .collect()
}

fn push_deployment_zone(
    zones: &mut Vec<DeploymentZone>,
    id: &str,
    label: &str,
    kind: DeploymentZoneKind,
    cells: Vec<Position>,
) {
    if cells.is_empty() {
        return;
    }

    zones.push(DeploymentZone {
        id: id.to_string(),
        label: label.to_string(),
        kind,
        cells: unique_positions(cells),
    });
}

fn push_spawn_zone(
    zones: &mut Vec<SpawnZone>,
    id: &str,
    label: &str,
    kind: SpawnZoneKind,
    confidence: ZoneConfidence,
    cells: Vec<Position>,
) {
    if cells.is_empty() {
        return;
    }

    zones.push(SpawnZone {
        id: id.to_string(),
        label: label.to_string(),
        kind,
        confidence,
        cells: unique_positions(cells),
        revealed_details: Vec::new(),
    });
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
    routes: &[BattlefieldRoute],
    encounter: Option<&PveEncounter>,
    game_data: &GameDataBase,
    preview_seed: u64,
) -> Vec<SpawnWave> {
    let fallback_route_id = fallback_spawn_route_id(node_type, routes);
    let Some(encounter) = encounter else {
        return vec![SpawnWave {
            id: "wave_0".to_string(),
            time_ms: 0,
            spawn_zone_ids: spawn_zone_ids_for_wave(archetype, spawn_zones, 0),
            route_id: fallback_route_id,
            enemy_entries: Vec::new(),
            required_for_victory: true,
        }];
    };

    let authored_waves = encounter.wave_definitions();
    if authored_waves.is_empty() {
        return vec![SpawnWave {
            id: "wave_0".to_string(),
            time_ms: 0,
            spawn_zone_ids: spawn_zone_ids_for_wave(archetype, spawn_zones, 0),
            route_id: fallback_route_id,
            enemy_entries: Vec::new(),
            required_for_victory: true,
        }];
    }

    authored_waves
        .iter()
        .enumerate()
        .map(|(index, wave)| SpawnWave {
            id: wave.id.clone(),
            time_ms: wave.time_ms,
            spawn_zone_ids: if wave.spawn_zone_ids.is_empty() {
                spawn_zone_ids_for_wave(archetype, spawn_zones, index)
            } else {
                wave.spawn_zone_ids.clone()
            },
            route_id: wave.route_id.clone().or_else(|| fallback_route_id.clone()),
            enemy_entries: wave_enemy_entries(wave, game_data, preview_seed, index),
            required_for_victory: wave.required_for_victory,
        })
        .collect()
}

fn fallback_spawn_route_id(
    node_type: CombatNodeType,
    routes: &[BattlefieldRoute],
) -> Option<String> {
    if node_type != CombatNodeType::Defense {
        return None;
    }
    routes.first().map(|route| route.id.clone())
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
) -> Vec<SpawnWaveEnemyEntry> {
    resolve_wave_enemy_data(wave, game_data, preview_seed, wave_index)
        .iter()
        .enumerate()
        .map(|(enemy_index, enemy)| {
            let count = enemy.count.max(1);
            SpawnWaveEnemyEntry {
                kind: enemy.kind,
                profile_id: enemy.profile_id.clone(),
                abnormality_id: enemy.abnormality_id.clone(),
                tier: enemy.tier,
                count,
                appearance_seeds: appearance_seeds_for_enemy(
                    preview_seed,
                    wave_index,
                    enemy_index,
                    enemy.kind,
                    enemy.profile_id.as_deref(),
                    &enemy.abnormality_id,
                    count,
                ),
            }
        })
        .collect()
}

fn resolve_wave_enemy_data(
    wave: &PveWaveData,
    game_data: &GameDataBase,
    preview_seed: u64,
    wave_index: usize,
) -> Vec<crate::game::data::pve_data::PveWaveEnemyData> {
    use crate::game::data::pve_data::PveWaveSource;

    match &wave.source {
        Some(PveWaveSource::Manual(enemies)) => enemies.clone(),
        Some(PveWaveSource::GeneratedCorroded {
            preset_id,
            budget_override,
            seed_salt,
        }) => {
            let preset = game_data
                .corroded_wave_data
                .get_by_id(preset_id)
                .unwrap_or_else(|| {
                    panic!(
                        "generated corroded wave '{}' references missing preset '{}'",
                        wave.id, preset_id
                    )
                });
            generate_corroded_wave(
                preset,
                *budget_override,
                *seed_salt,
                preview_seed,
                wave_index,
            )
        }
        None => wave.enemies.clone(),
    }
}

fn generate_corroded_wave(
    preset: &CorrodedWavePreset,
    budget_override: Option<u32>,
    seed_salt: Option<u64>,
    preview_seed: u64,
    wave_index: usize,
) -> Vec<crate::game::data::pve_data::PveWaveEnemyData> {
    let mut remaining_budget = budget_override.unwrap_or(preset.budget);
    let mut counts = vec![0_u32; preset.role_mix.len()];

    for (index, role) in preset.role_mix.iter().enumerate() {
        let min_count = role.min_count;
        counts[index] = min_count;
        remaining_budget = remaining_budget.saturating_sub(role.cost.saturating_mul(min_count));
    }

    let target_count = generated_corroded_target_count(preset, preview_seed, seed_salt, wave_index);
    while counts.iter().sum::<u32>() < target_count {
        let Some(index) = choose_corroded_role_index(
            &preset.role_mix,
            &counts,
            remaining_budget,
            preview_seed,
            seed_salt,
            wave_index,
            counts.iter().sum::<u32>(),
        ) else {
            break;
        };
        counts[index] += 1;
        remaining_budget = remaining_budget.saturating_sub(preset.role_mix[index].cost);
    }

    preset
        .role_mix
        .iter()
        .zip(counts)
        .filter(|(_, count)| *count > 0)
        .map(|(role, count)| role.to_enemy_data(count))
        .collect()
}

fn generated_corroded_target_count(
    preset: &CorrodedWavePreset,
    preview_seed: u64,
    seed_salt: Option<u64>,
    wave_index: usize,
) -> u32 {
    let min = preset.count_range.min;
    let max = preset.count_range.max.max(min);
    let span = max - min + 1;
    min + (generated_corroded_seed(preview_seed, seed_salt, wave_index, 0) % u64::from(span)) as u32
}

fn choose_corroded_role_index(
    roles: &[CorrodedWaveRoleWeight],
    counts: &[u32],
    remaining_budget: u32,
    preview_seed: u64,
    seed_salt: Option<u64>,
    wave_index: usize,
    pick_index: u32,
) -> Option<usize> {
    let candidates = roles
        .iter()
        .enumerate()
        .filter(|(index, role)| {
            remaining_budget >= role.cost
                && role
                    .max_count
                    .is_none_or(|max_count| counts[*index] < max_count)
        })
        .collect::<Vec<_>>();
    let total_weight = candidates
        .iter()
        .map(|(_, role)| u64::from(role.weight))
        .sum::<u64>();
    if total_weight == 0 {
        return None;
    }

    let mut roll = generated_corroded_seed(
        preview_seed,
        seed_salt,
        wave_index,
        u64::from(pick_index) + 1,
    ) % total_weight;
    for (index, role) in candidates {
        let weight = u64::from(role.weight);
        if roll < weight {
            return Some(index);
        }
        roll -= weight;
    }

    None
}

fn generated_corroded_seed(
    preview_seed: u64,
    seed_salt: Option<u64>,
    wave_index: usize,
    step: u64,
) -> u64 {
    const CORRODED_WAVE_SEED_NS: u64 = 0x434F_5257_4156_4547; // "CORWAVEG"
    let mut seed = determinism::seed_with_namespace(preview_seed, CORRODED_WAVE_SEED_NS);
    seed = determinism::seed_with_namespace(seed, seed_salt.unwrap_or(0));
    seed = determinism::seed_with_namespace(seed, wave_index as u64);
    determinism::seed_with_namespace(seed, step)
}

fn appearance_seeds_for_enemy(
    preview_seed: u64,
    wave_index: usize,
    enemy_index: usize,
    kind: EnemyKind,
    profile_id: Option<&str>,
    abnormality_id: &str,
    count: u32,
) -> Vec<u64> {
    if kind != EnemyKind::CorrodedEmployee {
        return Vec::new();
    }

    let profile_seed = stable_str_seed(profile_id.unwrap_or(abnormality_id));
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

#[derive(Debug, Clone, Copy)]
struct EnemyThreatStats {
    defense: i32,
    magic_resist: i32,
    speed_units_per_ms: u32,
    mobility_kind: crate::game::battle::types::MobilityKind,
}

fn threat_warnings_for(
    spawn_waves: &[SpawnWave],
    game_data: &GameDataBase,
    preview_seed: u64,
) -> Vec<ThreatWarning> {
    let tags = required_briefing_warning_tags_for_spawn_waves(spawn_waves, game_data);
    let mut warnings = detectable_threat_warning_tags()
        .iter()
        .copied()
        .filter(|tag| tags.contains(tag))
        .map(|tag| ThreatWarning {
            tag,
            status: ThreatWarningStatus::Unverified,
            source: ThreatWarningSource::Briefing,
        })
        .collect::<Vec<_>>();

    if let Some(tag) = rumor_threat_warning_tag(preview_seed, &tags) {
        warnings.push(ThreatWarning {
            tag,
            status: ThreatWarningStatus::Unverified,
            source: ThreatWarningSource::Rumor,
        });
    }

    warnings
}

pub fn required_briefing_warning_tags_for_spawn_waves(
    spawn_waves: &[SpawnWave],
    game_data: &GameDataBase,
) -> HashSet<ThreatWarningTag> {
    let mut tags = HashSet::new();
    for entry in spawn_waves
        .iter()
        .flat_map(|wave| wave.enemy_entries.iter())
    {
        let Some(stats) = threat_stats_for_enemy(entry, game_data) else {
            continue;
        };
        if is_high_defense(stats.defense) {
            tags.insert(ThreatWarningTag::ArmoredEnemyPossible);
        }
        if is_high_magic_resist(stats.magic_resist) {
            tags.insert(ThreatWarningTag::HighMagicResistEnemyPossible);
        }
        if is_fast_breakthrough_speed(stats.speed_units_per_ms) {
            tags.insert(ThreatWarningTag::FastBreakthroughEnemyPossible);
        }
        if stats.mobility_kind.is_airborne() {
            tags.insert(ThreatWarningTag::AirEnemyPossible);
        }
    }

    tags
}

fn detectable_threat_warning_tags() -> &'static [ThreatWarningTag] {
    &[
        ThreatWarningTag::ArmoredEnemyPossible,
        ThreatWarningTag::HighMagicResistEnemyPossible,
        ThreatWarningTag::AirEnemyPossible,
        ThreatWarningTag::FastBreakthroughEnemyPossible,
    ]
}

fn rumor_threat_warning_tag(
    preview_seed: u64,
    real_tags: &HashSet<ThreatWarningTag>,
) -> Option<ThreatWarningTag> {
    let seed = determinism::seed_with_namespace(preview_seed, THREAT_RUMOR_SEED_NS);
    if (seed % 100) as u8 >= THREAT_RUMOR_CHANCE_PERCENT {
        return None;
    }

    let candidates = detectable_threat_warning_tags()
        .iter()
        .copied()
        .filter(|tag| !real_tags.contains(tag))
        .collect::<Vec<_>>();
    if candidates.is_empty() {
        return None;
    }

    let pick_seed = determinism::seed_with_namespace(seed, 1);
    Some(candidates[(pick_seed as usize) % candidates.len()])
}

fn threat_stats_for_enemy(
    entry: &SpawnWaveEnemyEntry,
    game_data: &GameDataBase,
) -> Option<EnemyThreatStats> {
    match entry.kind {
        EnemyKind::CorrodedEmployee => {
            let profile_id = entry.profile_id.as_deref().unwrap_or(&entry.abnormality_id);
            game_data
                .corroded_employee_data
                .get_by_id(profile_id)
                .map(|profile| EnemyThreatStats {
                    defense: profile.defense,
                    magic_resist: profile.magic_resist,
                    speed_units_per_ms: profile.movement.speed_units_per_ms,
                    mobility_kind: Default::default(),
                })
        }
        EnemyKind::Abnormality | EnemyKind::FacilityEntity => game_data
            .abnormality_data
            .get_by_id(&entry.abnormality_id)
            .map(|abnormality| EnemyThreatStats {
                defense: abnormality.defense,
                magic_resist: abnormality.magic_resist,
                speed_units_per_ms: abnormality.movement.speed_units_per_ms,
                mobility_kind: abnormality.mobility_kind,
            }),
    }
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
    let abnormality = game_data
        .abnormality_data
        .get_by_id(&encounter.abnormality_id);
    let kind = primary_enemy_kind(encounter, category);
    let count_hint = spawn_waves
        .iter()
        .flat_map(|wave| &wave.enemy_entries)
        .map(|enemy| enemy.count)
        .sum::<u32>()
        .max(1);

    vec![EnemyBriefing {
        kind,
        abnormality_id: encounter.abnormality_id.clone(),
        display_name: abnormality
            .map(|abnormality| abnormality.name.clone())
            .unwrap_or_else(|| encounter.abnormality_id.clone()),
        risk_level: abnormality
            .map(|abnormality| abnormality.risk_level)
            .unwrap_or(encounter.risk_level),
        role: role_for(category).to_string(),
        count_hint,
    }]
}

fn primary_enemy_kind(encounter: &PveEncounter, category: MapNodeCategory) -> EnemyKind {
    if category == MapNodeCategory::Boss {
        return EnemyKind::Abnormality;
    }

    encounter
        .wave_definitions()
        .first()
        .and_then(|wave| wave.enemies.first())
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

fn validate_instance(instance: &BattlefieldInstance) -> Result<(), &'static str> {
    if instance.width <= 0 || instance.height <= 0 {
        return Err("battlefield dimensions must be positive");
    }
    if instance.deployment_zones.is_empty() {
        return Err("battlefield must have at least one deployment zone");
    }
    if instance.spawn_zones.is_empty() {
        return Err("battlefield must have at least one spawn zone");
    }

    let valid_cells = instance.valid_tiles.iter().copied().collect::<HashSet<_>>();
    if valid_cells.is_empty() {
        return Err("battlefield must have at least one valid tile");
    }
    if valid_cells.len() != instance.valid_tiles.len() {
        return Err("duplicate valid battlefield tile");
    }
    if instance
        .valid_tiles
        .iter()
        .any(|cell| !in_bounds(*cell, instance.width, instance.height))
    {
        return Err("valid tile is out of battlefield bounds");
    }
    let tile_positions = instance
        .tiles
        .iter()
        .map(|tile| tile.position)
        .collect::<HashSet<_>>();
    if tile_positions.is_empty() {
        return Err("battlefield must expose canonical tiles");
    }
    if tile_positions.len() != instance.tiles.len() {
        return Err("duplicate canonical battlefield tile");
    }
    if tile_positions != valid_cells {
        return Err("canonical battlefield tiles must match valid tiles");
    }
    if instance
        .tiles
        .iter()
        .any(|tile| !in_bounds(tile.position, instance.width, instance.height))
    {
        return Err("canonical battlefield tile is out of battlefield bounds");
    }

    let obstacle_cells = instance.obstacles.iter().copied().collect::<HashSet<_>>();
    if obstacle_cells.len() != instance.obstacles.len() {
        return Err("duplicate battlefield obstacle");
    }
    if instance
        .obstacles
        .iter()
        .any(|cell| !in_bounds(*cell, instance.width, instance.height))
    {
        return Err("obstacle is out of battlefield bounds");
    }
    if obstacle_cells
        .iter()
        .any(|cell| !valid_cells.contains(cell))
    {
        return Err("obstacle is outside valid battlefield tiles");
    }
    let obstacle_tile_cells = instance
        .tiles
        .iter()
        .filter(|tile| tile.kind == BattlefieldTileKind::Obstacle)
        .map(|tile| tile.position)
        .collect::<HashSet<_>>();
    if obstacle_tile_cells != obstacle_cells {
        return Err("canonical obstacle tiles must match obstacles");
    }
    if has_duplicate_ids(
        instance
            .deployment_zones
            .iter()
            .map(|zone| zone.id.as_str()),
    ) {
        return Err("duplicate deployment zone id");
    }
    if has_duplicate_ids(instance.spawn_zones.iter().map(|zone| zone.id.as_str())) {
        return Err("duplicate spawn zone id");
    }
    if has_duplicate_ids(instance.routes.iter().map(|route| route.id.as_str())) {
        return Err("duplicate route id");
    }
    let deployment_cells = instance
        .deployment_zones
        .iter()
        .flat_map(|zone| zone.cells.iter().copied())
        .collect::<HashSet<_>>();
    let spawn_cells = instance
        .spawn_zones
        .iter()
        .flat_map(|zone| zone.cells.iter().copied())
        .collect::<HashSet<_>>();

    if deployment_cells
        .iter()
        .any(|cell| !in_bounds(*cell, instance.width, instance.height))
    {
        return Err("deployment zone is out of battlefield bounds");
    }
    if spawn_cells
        .iter()
        .any(|cell| !in_bounds(*cell, instance.width, instance.height))
    {
        return Err("spawn zone is out of battlefield bounds");
    }
    if deployment_cells
        .iter()
        .any(|cell| !valid_cells.contains(cell))
    {
        return Err("deployment zone is outside valid battlefield tiles");
    }
    if spawn_cells.iter().any(|cell| !valid_cells.contains(cell)) {
        return Err("spawn zone is outside valid battlefield tiles");
    }
    if deployment_cells
        .iter()
        .any(|cell| obstacle_cells.contains(cell))
    {
        return Err("deployment zone overlaps obstacle");
    }
    if spawn_cells.iter().any(|cell| obstacle_cells.contains(cell)) {
        return Err("spawn zone overlaps obstacle");
    }
    if deployment_cells
        .iter()
        .any(|cell| spawn_cells.contains(cell))
    {
        return Err("deployment zone overlaps spawn zone");
    }
    for route in &instance.routes {
        if route.cells.is_empty() {
            return Err("route has no cells");
        }
        if route.cells.first().copied() != Some(route.start) {
            return Err("route first cell must be route start");
        }
        if route.cells.last().copied() != Some(route.end) {
            return Err("route last cell must be route end");
        }
        if route.cells.iter().any(|cell| !valid_cells.contains(cell)) {
            return Err("route is outside valid battlefield tiles");
        }
        if route.cells.iter().any(|cell| obstacle_cells.contains(cell)) {
            return Err("route overlaps obstacle");
        }
    }
    if instance.node_type == CombatNodeType::Defense && !instance.routes.is_empty() {
        let defense_endpoint = instance.routes[0].end;
        if instance
            .routes
            .iter()
            .any(|route| route.end != defense_endpoint)
        {
            return Err("defense routes must share one defense object endpoint");
        }
    }

    let spawn_zone_ids = instance
        .spawn_zones
        .iter()
        .map(|zone| zone.id.as_str())
        .collect::<HashSet<_>>();
    let route_ids = instance
        .routes
        .iter()
        .map(|route| route.id.as_str())
        .collect::<HashSet<_>>();
    for wave in &instance.spawn_waves {
        if wave
            .spawn_zone_ids
            .iter()
            .any(|zone_id| !spawn_zone_ids.contains(zone_id.as_str()))
        {
            return Err("spawn wave references missing spawn zone");
        }
        if let Some(route_id) = &wave.route_id {
            if !route_ids.contains(route_id.as_str()) {
                return Err("spawn wave references missing route");
            }
        } else if instance.node_type == CombatNodeType::Defense {
            return Err("defense spawn wave must reference a route");
        }
    }

    let start = deployment_cells
        .iter()
        .next()
        .copied()
        .ok_or("deployment zone has no cells")?;
    let target = spawn_cells
        .iter()
        .next()
        .copied()
        .ok_or("spawn zone has no cells")?;
    if !has_path(
        start,
        target,
        instance.width,
        instance.height,
        &valid_cells,
        &obstacle_cells,
    ) {
        return Err("deployment zone cannot path to spawn zone");
    }

    Ok(())
}

fn has_duplicate_ids<'a>(mut ids: impl Iterator<Item = &'a str>) -> bool {
    let mut seen = HashSet::new();
    ids.any(|id| !seen.insert(id))
}

fn in_bounds(position: Position, width: i32, height: i32) -> bool {
    position.x >= 0 && position.y >= 0 && position.x < width && position.y < height
}

fn has_path(
    start: Position,
    target: Position,
    width: i32,
    height: i32,
    valid_cells: &HashSet<Position>,
    blocked: &HashSet<Position>,
) -> bool {
    if !valid_cells.contains(&start)
        || !valid_cells.contains(&target)
        || blocked.contains(&start)
        || blocked.contains(&target)
    {
        return false;
    }

    let mut visited = HashSet::new();
    let mut queue = VecDeque::from([start]);
    while let Some(position) = queue.pop_front() {
        if position == target {
            return true;
        }
        if !visited.insert(position) {
            continue;
        }
        for next in [
            Position::new(position.x + 1, position.y),
            Position::new(position.x - 1, position.y),
            Position::new(position.x, position.y + 1),
            Position::new(position.x, position.y - 1),
        ] {
            if next.x < 0
                || next.y < 0
                || next.x >= width
                || next.y >= height
                || !valid_cells.contains(&next)
                || blocked.contains(&next)
                || visited.contains(&next)
            {
                continue;
            }
            queue.push_back(next);
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    use crate::game::data::{
        abnormality_data::AbnormalityMetadata,
        corroded_employee_data::{
            CorrodedEmployeeProfileDatabase, CorrodedEmployeeProfileMetadata,
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

    fn empty_data() -> GameDataBase {
        GameDataBuilder::empty().build()
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
            max_health: 10,
            attack: 1,
            defense: 0,
            magic_resist: 0,
            movement: Default::default(),
            basic_attack: Default::default(),
            resonance: Default::default(),
            skill_id: None,
        }
    }

    #[test]
    fn combat_preview_serializes_typed_threat_warnings() {
        let mut armored = preview_test_abnormality("armored_enemy", 0xA111);
        armored.defense = crate::game::combat_balance::HIGH_DEFENSE_WARNING_THRESHOLD;
        armored.magic_resist = crate::game::combat_balance::HIGH_MAGIC_RESIST_WARNING_THRESHOLD;
        let data = GameDataBuilder::empty()
            .with_abnormalities(vec![armored])
            .with_pve(PveEncounterDatabase::new(vec![PveEncounter {
                id: "threat_encounter".to_string(),
                abnormality_id: "armored_enemy".to_string(),
                difficulty: 1,
                risk_level: RiskLevel::ZAYIN,
                reward_mode: crate::game::enums::RewardMode::ClaimAll,
                reward_uuids: Vec::new(),
                node_type: Some(CombatNodeType::Defense),
                mission_variant: None,
                battlefield: None,
                tactical_plan: None,
                win_condition: None,
                waves: vec![PveWaveData {
                    id: "wave_0".to_string(),
                    time_ms: 0,
                    spawn_zone_ids: Vec::new(),
                    route_id: None,
                    required_for_victory: true,
                    source: None,
                    enemies: vec![PveWaveEnemyData {
                        kind: EnemyKind::Abnormality,
                        profile_id: None,
                        abnormality_id: "armored_enemy".to_string(),
                        tier: Tier::I,
                        count: 1,
                    }],
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
                abnormality_id: "airborne_enemy".to_string(),
                difficulty: 1,
                risk_level: RiskLevel::ZAYIN,
                reward_mode: crate::game::enums::RewardMode::ClaimAll,
                reward_uuids: Vec::new(),
                node_type: Some(CombatNodeType::Defense),
                mission_variant: None,
                battlefield: None,
                tactical_plan: None,
                win_condition: None,
                waves: vec![PveWaveData {
                    id: "wave_0".to_string(),
                    time_ms: 0,
                    spawn_zone_ids: Vec::new(),
                    route_id: None,
                    required_for_victory: true,
                    source: None,
                    enemies: vec![PveWaveEnemyData {
                        kind: EnemyKind::Abnormality,
                        profile_id: None,
                        abnormality_id: "airborne_enemy".to_string(),
                        tier: Tier::I,
                        count: 1,
                    }],
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
        let data = empty_data();
        let request = BattlefieldGenerationRequest {
            category: MapNodeCategory::Combat,
            encounter_id: None,
            seed: 42,
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
            mission_variant: CombatMissionVariant::Encirclement,
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
        };

        let result = validate_instance(&instance);

        assert_eq!(result, Err("battlefield dimensions must be positive"));
    }

    #[test]
    fn generator_covers_all_combat_archetypes_with_valid_instances() {
        let data = empty_data();
        let mut archetypes = HashSet::new();

        for seed in 0..6 {
            let instance = BattlefieldGenerator::generate(
                BattlefieldGenerationRequest {
                    category: MapNodeCategory::Combat,
                    encounter_id: None,
                    seed,
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
        let data = empty_data();

        let instance = BattlefieldGenerator::generate(
            BattlefieldGenerationRequest {
                category: MapNodeCategory::Combat,
                encounter_id: None,
                seed: 1,
            },
            &data,
        );

        assert_eq!(instance.archetype, BattlefieldArchetype::Corridor);
        assert_eq!((instance.width, instance.height), (9, 9));
        assert!(!instance.valid_tiles.contains(&Position::new(8, 0)));
        assert!(instance.valid_tiles.contains(&Position::new(0, 0)));
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
            if template.archetype == BattlefieldArchetype::SplitRoom {
                continue;
            }
            let mut parsed = parse_battlefield_template(template).unwrap();
            ensure_defense_route(&mut parsed, CombatNodeType::Defense);
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
        };

        let result = validate_instance(&instance);

        assert_eq!(result, Err("defense spawn wave must reference a route"));
    }

    #[test]
    fn validate_instance_requires_defense_routes_to_share_endpoint() {
        let instance = BattlefieldInstance {
            battlefield_template_id: "defense_split_endpoint".to_string(),
            node_type: CombatNodeType::Defense,
            mission_variant: CombatMissionVariant::Defense,
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
            .with_abnormalities(vec![
                preview_test_abnormality("display_abno", 0xD15A),
                preview_test_abnormality("enemy_a", 0xE0A),
                preview_test_abnormality("enemy_b", 0xE0B),
            ])
            .with_pve(PveEncounterDatabase::new(vec![PveEncounter {
                id: "encounter".to_string(),
                abnormality_id: "display_abno".to_string(),
                difficulty: 1,
                risk_level: RiskLevel::ZAYIN,
                reward_mode: crate::game::enums::RewardMode::ClaimAll,
                reward_uuids: Vec::new(),
                node_type: None,
                mission_variant: None,
                battlefield: None,
                tactical_plan: None,
                win_condition: None,
                waves: vec![PveWaveData {
                    id: "wave_0".to_string(),
                    time_ms: 0,
                    spawn_zone_ids: Vec::new(),
                    route_id: None,
                    required_for_victory: true,
                    source: None,
                    enemies: vec![
                        PveWaveEnemyData {
                            kind: EnemyKind::Abnormality,
                            profile_id: None,
                            abnormality_id: "enemy_a".to_string(),
                            tier: Tier::I,
                            count: 2,
                        },
                        PveWaveEnemyData {
                            kind: EnemyKind::Abnormality,
                            profile_id: None,
                            abnormality_id: "enemy_b".to_string(),
                            tier: Tier::II,
                            count: 1,
                        },
                    ],
                }],
                static_obstacles: Vec::new(),
            }]))
            .build();

        let instance = BattlefieldGenerator::generate(
            BattlefieldGenerationRequest {
                category: MapNodeCategory::Combat,
                encounter_id: Some("encounter"),
                seed: 0,
            },
            &data,
        );

        assert_eq!(instance.enemy_briefing[0].abnormality_id, "display_abno");
        assert_eq!(
            instance.spawn_waves[0].enemy_entries,
            vec![
                SpawnWaveEnemyEntry {
                    kind: EnemyKind::Abnormality,
                    profile_id: None,
                    abnormality_id: "enemy_a".to_string(),
                    tier: Tier::I,
                    count: 2,
                    appearance_seeds: Vec::new(),
                },
                SpawnWaveEnemyEntry {
                    kind: EnemyKind::Abnormality,
                    profile_id: None,
                    abnormality_id: "enemy_b".to_string(),
                    tier: Tier::II,
                    count: 1,
                    appearance_seeds: Vec::new(),
                },
            ]
        );
    }

    #[test]
    fn generated_corroded_wave_source_resolves_during_preview_generation() {
        let data = GameDataBuilder::empty()
            .with_abnormalities(vec![preview_test_abnormality("display_abno", 0xD15A)])
            .with_corroded_employee_profiles(CorrodedEmployeeProfileDatabase::new(vec![
                preview_test_corroded_profile("corroded_guard", 0xC01),
                preview_test_corroded_profile("corroded_rusher", 0xC02),
            ]))
            .with_corroded_wave_presets(CorrodedWavePresetDatabase::new(vec![CorrodedWavePreset {
                id: "test_corroded_mix".to_string(),
                difficulty: 1,
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
                abnormality_id: "display_abno".to_string(),
                difficulty: 1,
                risk_level: RiskLevel::ZAYIN,
                reward_mode: crate::game::enums::RewardMode::ClaimAll,
                reward_uuids: Vec::new(),
                node_type: Some(CombatNodeType::Defense),
                mission_variant: None,
                battlefield: None,
                tactical_plan: None,
                win_condition: None,
                waves: vec![PveWaveData {
                    id: "wave_0".to_string(),
                    time_ms: 0,
                    spawn_zone_ids: Vec::new(),
                    route_id: None,
                    required_for_victory: true,
                    source: Some(PveWaveSource::GeneratedCorroded {
                        preset_id: "test_corroded_mix".to_string(),
                        budget_override: None,
                        seed_salt: Some(7),
                    }),
                    enemies: Vec::new(),
                }],
                static_obstacles: Vec::new(),
            }]))
            .build();

        let first = BattlefieldGenerator::generate(
            BattlefieldGenerationRequest {
                category: MapNodeCategory::Combat,
                encounter_id: Some("generated_corroded_encounter"),
                seed: 44,
            },
            &data,
        );
        let second = BattlefieldGenerator::generate(
            BattlefieldGenerationRequest {
                category: MapNodeCategory::Combat,
                encounter_id: Some("generated_corroded_encounter"),
                seed: 44,
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
                abnormality_id: "display_abno".to_string(),
                difficulty: 1,
                risk_level: RiskLevel::ZAYIN,
                reward_mode: crate::game::enums::RewardMode::ClaimAll,
                reward_uuids: Vec::new(),
                node_type: Some(CombatNodeType::Defense),
                mission_variant: None,
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
                    route_id: None,
                    required_for_victory: false,
                    source: None,
                    enemies: vec![PveWaveEnemyData {
                        kind: EnemyKind::Abnormality,
                        profile_id: None,
                        abnormality_id: "enemy_a".to_string(),
                        tier: Tier::I,
                        count: 1,
                    }],
                }],
                static_obstacles: Vec::new(),
            }]))
            .build();

        let instance = BattlefieldGenerator::generate(
            BattlefieldGenerationRequest {
                category: MapNodeCategory::Combat,
                encounter_id: Some("authored_route"),
                seed: 0,
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
                abnormality_id: "display_abno".to_string(),
                difficulty: 1,
                risk_level: RiskLevel::ZAYIN,
                reward_mode: crate::game::enums::RewardMode::ClaimAll,
                reward_uuids: Vec::new(),
                node_type: Some(CombatNodeType::Defense),
                mission_variant: None,
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
                    route_id: None,
                    required_for_victory: true,
                    source: None,
                    enemies: vec![PveWaveEnemyData {
                        kind: EnemyKind::Abnormality,
                        profile_id: None,
                        abnormality_id: "enemy_a".to_string(),
                        tier: Tier::I,
                        count: 1,
                    }],
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
        let data = empty_data();

        let instance = BattlefieldGenerator::generate(
            BattlefieldGenerationRequest {
                category: MapNodeCategory::Boss,
                encounter_id: None,
                seed: 99,
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
