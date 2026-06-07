use std::{collections::HashMap, sync::Arc};

use tracing::{info, warn};
use uuid::Uuid;

use crate::{
    game::resources::{Inventory, Position},
    game::{
        battle::{core::BattleCore, scenario::BattleScenario},
        behavior::GameError,
        combat_battlefield_plan::{
            validate_static_obstacles_do_not_overlap_scenario, BattleStartPlan,
        },
        combat_defense_object::defense_object_group_for_win_condition,
        combat_enemy_spawns::enemy_spawn_groups_from_preview,
        combat_mission_policy::CombatMissionPolicy,
        combat_player_spawns::{player_scenario_start_from_positions, PlayerScenarioStart},
        combat_preview::{CombatNodeType, CombatPreview},
        combat_rewards::resolve_combat_rewards_from_encounter,
        combat_scenario_groups::{push_start_spawn_group, push_timed_spawn_group},
        data::GameDataBase,
        employee::EmployeeRoster,
        enums::RewardMode,
        reward::RewardOption,
        skill_fragment::SkillFragmentInventory,
    },
};

#[cfg(test)]
use crate::game::{
    battle::{
        scenario::{ScenarioGroupId, ScenarioSpawnGroup, ScenarioUnitRef, ScenarioUnitSpawn},
        types::{BattleUnitDraft, BattleUnitSource},
    },
    combat_player_spawns::scenario_artifacts_from_inventory,
    enums::Side,
    growth::GrowthStack,
};

/// Node combat business logic helper.
pub struct CombatExecutor;

#[derive(Debug, Clone, Copy)]
#[cfg(test)]
pub struct TestPlayerAbnormalityUnit {
    pub owned_uuid: Uuid,
    pub base_uuid: Uuid,
    pub level: crate::game::enums::Tier,
}

impl CombatExecutor {
    pub fn build_battle_with_combat_preview(
        roster: &EmployeeRoster,
        inventory: &Inventory,
        skill_fragments: &SkillFragmentInventory,
        game_data: Arc<GameDataBase>,
        abnormality_id: &str,
        encounter_id: &str,
        movement_seed: u64,
        combat_preview: &CombatPreview,
        deployment_positions: &HashMap<Uuid, Position>,
    ) -> Result<BattleCore, GameError> {
        let plan = BattleStartPlan::from_preview(&game_data, encounter_id, combat_preview)?;
        Self::build_battle_with_explicit_deployment(
            roster,
            inventory,
            skill_fragments,
            game_data,
            abnormality_id,
            encounter_id,
            movement_seed,
            plan,
            combat_preview,
            deployment_positions,
        )
    }

    fn build_battle_with_explicit_deployment(
        roster: &EmployeeRoster,
        inventory: &Inventory,
        skill_fragments: &SkillFragmentInventory,
        game_data: Arc<GameDataBase>,
        abnormality_id: &str,
        encounter_id: &str,
        movement_seed: u64,
        plan: BattleStartPlan,
        combat_preview: &CombatPreview,
        deployment_positions: &HashMap<Uuid, Position>,
    ) -> Result<BattleCore, GameError> {
        info!(
            "Starting node combat for abnormality={} encounter={} field_size={:?}",
            abnormality_id, encounter_id, plan.field_size
        );

        let player_start = player_scenario_start_from_positions(
            roster,
            inventory,
            skill_fragments,
            &game_data,
            deployment_positions
                .iter()
                .map(|(uuid, position)| (*uuid, *position))
                .collect(),
        )?;
        let scenario = Self::build_battle_scenario_from_preview(
            &game_data,
            encounter_id,
            combat_preview,
            plan,
            player_start,
        )?;
        validate_static_obstacles_do_not_overlap_scenario(&scenario)?;

        Ok(BattleCore::new_from_scenario(
            scenario,
            game_data,
            movement_seed,
        ))
    }

    /// Test harness for battle scenarios that intentionally field abnormalities on the player side.
    ///
    /// This does not grant, own, buy, or store abnormalities in player inventory. It only converts
    /// explicit test fixtures into a scenario spawn group so combat behavior can be validated against
    /// abnormality metadata when needed.
    pub fn resolve_rewards(
        game_data: &GameDataBase,
        encounter_id: &str,
    ) -> Result<(RewardMode, Vec<RewardOption>), GameError> {
        let encounter = game_data
            .pve_data
            .get_by_id(encounter_id)
            .ok_or(GameError::MissingResource("PveEncounter"))?;

        resolve_combat_rewards_from_encounter(
            game_data,
            encounter,
            encounter.node_type,
            encounter.mission_variant,
        )
    }

    pub fn resolve_rewards_for_mission(
        game_data: &GameDataBase,
        encounter_id: &str,
        node_type: CombatNodeType,
        mission_variant: crate::game::combat_preview::CombatMissionVariant,
    ) -> Result<(RewardMode, Vec<RewardOption>), GameError> {
        let encounter = game_data
            .pve_data
            .get_by_id(encounter_id)
            .ok_or(GameError::MissingResource("PveEncounter"))?;

        if let Some(authored_node_type) = encounter.node_type {
            if authored_node_type != node_type {
                return Err(GameError::InvalidStaticData(format!(
                    "combat encounter '{}' resolved rewards for {:?}, but authored node_type is {:?}",
                    encounter_id, node_type, authored_node_type
                )));
            }
        }
        if let Some(authored_variant) = encounter.mission_variant {
            if authored_variant != mission_variant {
                return Err(GameError::InvalidStaticData(format!(
                    "combat encounter '{}' resolved rewards for {:?}/{:?}, but authored mission_variant is {:?}",
                    encounter_id, node_type, mission_variant, authored_variant
                )));
            }
        }

        resolve_combat_rewards_from_encounter(
            game_data,
            encounter,
            Some(node_type),
            Some(mission_variant),
        )
    }

    #[cfg(test)]
    fn build_test_player_abnormality_scenario_start_from_positions(
        player_units: &[TestPlayerAbnormalityUnit],
        inventory: &Inventory,
        deployment_positions: &HashMap<Uuid, Position>,
    ) -> Result<PlayerScenarioStart, GameError> {
        let fixtures = player_units
            .iter()
            .map(|unit| (unit.owned_uuid, *unit))
            .collect::<std::collections::HashMap<_, _>>();

        let mut placements: Vec<(Uuid, Position)> = deployment_positions
            .iter()
            .map(|(uuid, position)| (*uuid, *position))
            .collect();
        placements.sort_by(|a, b| a.0.as_bytes().cmp(b.0.as_bytes()));

        let group_id = ScenarioGroupId::new("player_initial");
        let mut spawns = Vec::new();

        for (index, (owned_uuid, pos)) in placements.into_iter().enumerate() {
            let fixture = fixtures.get(&owned_uuid).ok_or(GameError::UnitNotFound)?;
            let draft = BattleUnitDraft {
                owned_uuid,
                source: BattleUnitSource::Abnormality {
                    base_uuid: fixture.base_uuid,
                },
                level: fixture.level,
                growth_stacks: GrowthStack::new(),
                equipped_items: vec![],
                equipped_item_enhancements: vec![],
            };
            spawns.push(ScenarioUnitSpawn {
                unit_ref: ScenarioUnitRef::new(format!("{}_{}", group_id.0, index)),
                side: Side::Player,
                draft,
                position: pos,
                instance_salt: index as u32,
            });
        }

        Ok(PlayerScenarioStart {
            spawn_group: ScenarioSpawnGroup {
                id: group_id,
                side: Side::Player,
                required_for_victory: false,
                enemy_movement_plan: None,
                spawns,
            },
            artifacts: scenario_artifacts_from_inventory(inventory),
        })
    }

    fn build_battle_scenario_from_preview(
        game_data: &GameDataBase,
        encounter_id: &str,
        combat_preview: &CombatPreview,
        plan: BattleStartPlan,
        player_start: PlayerScenarioStart,
    ) -> Result<BattleScenario, GameError> {
        let encounter = game_data.pve_data.get_by_id(encounter_id).ok_or_else(|| {
            warn!("PvE encounter not found: {}", encounter_id);
            GameError::MissingResource("PveEncounter")
        })?;
        Self::validate_preview_matches_encounter(encounter_id, combat_preview, encounter)?;

        let mut groups = Vec::new();
        let mut events = Vec::new();

        push_start_spawn_group(&mut groups, &mut events, player_start.spawn_group);

        let mut tactical_plan =
            CombatMissionPolicy::default_tactical_plan_for_preview(combat_preview);
        encounter.apply_authored_tactical_plan(&mut tactical_plan);
        let win_condition = encounter.authored_win_condition().unwrap_or_else(|| {
            CombatMissionPolicy::default_win_condition_for_tactical_plan(
                combat_preview.mission_variant,
                combat_preview.mission_risk,
                &tactical_plan,
            )
        });

        if let Some(group) = defense_object_group_for_win_condition(&win_condition, &tactical_plan)
        {
            push_start_spawn_group(&mut groups, &mut events, group);
        }

        let enemy_groups =
            enemy_spawn_groups_from_preview(game_data, encounter.id.as_str(), combat_preview)?;
        for (group, time_ms) in enemy_groups {
            push_timed_spawn_group(&mut groups, &mut events, group, time_ms);
        }

        Ok(BattleScenario {
            battlefield: plan.into_battlefield_spec(),
            artifacts: player_start.artifacts,
            groups,
            events,
            win_condition,
            tactical_plan,
        })
    }

    fn validate_preview_matches_encounter(
        encounter_id: &str,
        combat_preview: &CombatPreview,
        encounter: &crate::game::data::pve_data::PveEncounter,
    ) -> Result<(), GameError> {
        match combat_preview.encounter_id.as_deref() {
            Some(preview_encounter_id) if preview_encounter_id == encounter_id => {}
            Some(preview_encounter_id) => {
                return Err(GameError::InvalidStaticData(format!(
                    "combat preview was generated for encounter '{}', but battle start requested '{}'",
                    preview_encounter_id, encounter_id
                )));
            }
            None => {
                return Err(GameError::InvalidStaticData(format!(
                    "combat preview for battle start must carry encounter_id '{}'",
                    encounter_id
                )));
            }
        }

        if let Some(authored_node_type) = encounter.node_type {
            if authored_node_type != combat_preview.node_type {
                return Err(GameError::InvalidStaticData(format!(
                    "combat preview node_type {:?} does not match authored encounter '{}' node_type {:?}",
                    combat_preview.node_type, encounter_id, authored_node_type
                )));
            }
        }
        if let Some(authored_variant) = encounter.mission_variant {
            if authored_variant != combat_preview.mission_variant {
                return Err(GameError::InvalidStaticData(format!(
                    "combat preview mission_variant {:?} does not match authored encounter '{}' mission_variant {:?}",
                    combat_preview.mission_variant, encounter_id, authored_variant
                )));
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::battle::scenario::{EnemyMovementPlan, PlayerMovementPlan, WinCondition};
    use crate::game::battle::timeline::TimelineEvent;
    use crate::game::combat_defense_object::DEFAULT_DEFENSE_OBJECT_GROUP;
    use crate::game::combat_mission_policy::DEFAULT_DEFENSE_OBJECT_REF;
    use crate::game::combat_preview::CombatPreview;
    use crate::game::data::{
        abnormality_data::AbnormalityMetadata,
        pve_data::{
            PveBattleObjectiveData, PveBattlefieldOverrideData, PveEncounter, PveEncounterDatabase,
            PveTacticalPlanData, PveTacticalPointData, PveWaveData, PveWaveEnemyData,
            PveWinConditionData,
        },
        GameDataBase, GameDataBuilder,
    };
    use crate::game::employee::{Employee, EmployeeRoster};
    use crate::game::enums::{RewardMode, RiskLevel, Side};
    use crate::game::map::{MapNodeCategory, MapNodeId};
    use crate::game::resources::Inventory;
    use std::collections::HashMap;
    use std::sync::Arc;

    fn game_data_with_abnormalities_and_pve(
        abnormalities: Vec<AbnormalityMetadata>,
        encounters: Vec<PveEncounter>,
    ) -> Arc<GameDataBase> {
        GameDataBuilder::empty()
            .with_abnormalities(abnormalities)
            .with_pve(PveEncounterDatabase::new(encounters))
            .build_arc()
    }

    fn abnormality(id: &str, uuid: Uuid, attack: u32) -> AbnormalityMetadata {
        AbnormalityMetadata {
            id: id.to_string(),
            uuid,
            name: id.to_string(),
            risk_level: RiskLevel::ZAYIN,
            price: 0,
            max_health: 30,
            attack,
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

    #[test]
    fn reward_resolution_rejects_authored_node_type_mismatch() {
        let enemy = abnormality("enemy", Uuid::from_u128(0x220), 1);
        let game_data = game_data_with_abnormalities_and_pve(
            vec![enemy],
            vec![PveEncounter {
                id: "boss_contract".to_string(),
                abnormality_id: "enemy".to_string(),
                difficulty: 1,
                risk_level: RiskLevel::ZAYIN,
                reward_mode: RewardMode::ClaimAll,
                reward_uuids: vec![],
                node_type: Some(CombatNodeType::Boss),
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
                        kind: crate::game::combat_preview::EnemyKind::Abnormality,
                        profile_id: None,
                        abnormality_id: "enemy".to_string(),
                        tier: crate::game::enums::Tier::I,
                        count: 1,
                    }],
                }],
                static_obstacles: vec![],
            }],
        );

        let err = CombatExecutor::resolve_rewards_for_mission(
            game_data.as_ref(),
            "boss_contract",
            CombatNodeType::Defense,
            crate::game::combat_preview::CombatMissionVariant::Defense,
        )
        .expect_err("reward resolution should reject a mismatched combat node type");

        assert!(matches!(
            err,
            GameError::InvalidStaticData(ref message)
                if message.contains("authored node_type")
        ));
    }

    #[test]
    fn combat_preview_drives_actual_map_battle_start_layout() {
        let employee_uuid = Uuid::from_u128(0x121);
        let enemy_base_uuid = Uuid::from_u128(0x221);
        let enemy = abnormality("enemy", enemy_base_uuid, 1);
        let game_data = game_data_with_abnormalities_and_pve(
            vec![enemy],
            vec![PveEncounter {
                id: "preview_encounter".to_string(),
                abnormality_id: "enemy".to_string(),
                difficulty: 1,
                risk_level: RiskLevel::ZAYIN,
                reward_mode: RewardMode::ClaimAll,
                reward_uuids: vec![],
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
                    enemies: vec![PveWaveEnemyData {
                        kind: crate::game::combat_preview::EnemyKind::Abnormality,
                        profile_id: None,
                        abnormality_id: "enemy".to_string(),
                        tier: crate::game::enums::Tier::I,
                        count: 1,
                    }],
                }],
                static_obstacles: vec![],
            }],
        );

        let preview = CombatPreview::generate_for_node(
            MapNodeId::new(Uuid::from_u128(0x9001)),
            MapNodeCategory::Combat,
            Some("preview_encounter"),
            game_data.as_ref(),
            11,
        );
        let inventory = Inventory::new();
        let skill_fragments = SkillFragmentInventory::new();
        let mut roster = EmployeeRoster::new();
        roster.add(Employee::new(employee_uuid, "Agent"));

        let deployment_positions =
            HashMap::from([(employee_uuid, preview.deployment_zones[0].cells[0])]);

        let battle = CombatExecutor::build_battle_with_combat_preview(
            &roster,
            &inventory,
            &skill_fragments,
            game_data,
            "enemy",
            "preview_encounter",
            7,
            &preview,
            &deployment_positions,
        )
        .expect("combat preview should provide a valid battle layout");

        assert_eq!(
            (battle.battlefield.width(), battle.battlefield.height()),
            (preview.width as u8, preview.height as u8),
            "actual combat must use the same battlefield dimensions as preview"
        );
    }

    #[test]
    fn battle_start_rejects_preview_without_spawn_waves_instead_of_falling_back_to_encounter() {
        let employee_uuid = Uuid::from_u128(0x124);
        let enemy_base_uuid = Uuid::from_u128(0x224);
        let enemy = abnormality("enemy", enemy_base_uuid, 1);
        let game_data = game_data_with_abnormalities_and_pve(
            vec![enemy],
            vec![PveEncounter {
                id: "strict_preview_encounter".to_string(),
                abnormality_id: "enemy".to_string(),
                difficulty: 1,
                risk_level: RiskLevel::ZAYIN,
                reward_mode: RewardMode::ClaimAll,
                reward_uuids: vec![],
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
                    enemies: vec![PveWaveEnemyData {
                        kind: crate::game::combat_preview::EnemyKind::Abnormality,
                        profile_id: None,
                        abnormality_id: "enemy".to_string(),
                        tier: crate::game::enums::Tier::I,
                        count: 1,
                    }],
                }],
                static_obstacles: vec![],
            }],
        );

        let mut preview = CombatPreview::generate_for_node(
            MapNodeId::new(Uuid::from_u128(0x9002)),
            MapNodeCategory::Combat,
            Some("strict_preview_encounter"),
            game_data.as_ref(),
            12,
        );
        preview.spawn_waves.clear();

        let inventory = Inventory::new();
        let skill_fragments = SkillFragmentInventory::new();
        let mut roster = EmployeeRoster::new();
        roster.add(Employee::new(employee_uuid, "Agent"));
        let deployment_positions =
            HashMap::from([(employee_uuid, preview.deployment_zones[0].cells[0])]);

        let err = match CombatExecutor::build_battle_with_combat_preview(
            &roster,
            &inventory,
            &skill_fragments,
            game_data,
            "enemy",
            "strict_preview_encounter",
            7,
            &preview,
            &deployment_positions,
        ) {
            Ok(_) => panic!("battle start must not infer hidden waves from encounter data"),
            Err(err) => err,
        };

        assert!(matches!(
            err,
            GameError::InvalidStaticData(ref message)
                if message.contains("has no spawn waves")
        ));
    }

    #[test]
    fn battle_start_rejects_preview_generated_for_different_encounter() {
        let employee_uuid = Uuid::from_u128(0x125);
        let enemy_a = abnormality("enemy_a", Uuid::from_u128(0x225), 1);
        let enemy_b = abnormality("enemy_b", Uuid::from_u128(0x226), 1);
        let encounter_a = PveEncounter {
            id: "encounter_a".to_string(),
            abnormality_id: "enemy_a".to_string(),
            difficulty: 1,
            risk_level: RiskLevel::ZAYIN,
            reward_mode: RewardMode::ClaimAll,
            reward_uuids: vec![],
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
                    kind: crate::game::combat_preview::EnemyKind::Abnormality,
                    profile_id: None,
                    abnormality_id: "enemy_a".to_string(),
                    tier: crate::game::enums::Tier::I,
                    count: 1,
                }],
            }],
            static_obstacles: vec![],
        };
        let mut encounter_b = encounter_a.clone();
        encounter_b.id = "encounter_b".to_string();
        encounter_b.abnormality_id = "enemy_b".to_string();
        encounter_b.waves[0].enemies[0].abnormality_id = "enemy_b".to_string();
        let game_data = game_data_with_abnormalities_and_pve(
            vec![enemy_a, enemy_b],
            vec![encounter_a, encounter_b],
        );

        let preview = CombatPreview::generate_for_node(
            MapNodeId::new(Uuid::from_u128(0x9002_1)),
            MapNodeCategory::Combat,
            Some("encounter_a"),
            game_data.as_ref(),
            12,
        );
        let mut roster = EmployeeRoster::new();
        roster.add(Employee::new(employee_uuid, "Agent"));
        let deployment_positions =
            HashMap::from([(employee_uuid, preview.deployment_zones[0].cells[0])]);

        let err = match CombatExecutor::build_battle_with_combat_preview(
            &roster,
            &Inventory::new(),
            &SkillFragmentInventory::new(),
            game_data,
            "enemy_b",
            "encounter_b",
            7,
            &preview,
            &deployment_positions,
        ) {
            Ok(_) => panic!("battle start must reject preview/encounter id mismatch"),
            Err(err) => err,
        };

        assert!(matches!(
            err,
            GameError::InvalidStaticData(ref message)
                if message.contains("generated for encounter 'encounter_a'")
                    && message.contains("requested 'encounter_b'")
        ));
    }

    #[test]
    fn battle_start_rejects_preview_node_type_mismatch_with_authored_encounter() {
        let employee_uuid = Uuid::from_u128(0x126);
        let enemy = abnormality("enemy", Uuid::from_u128(0x227), 1);
        let game_data = game_data_with_abnormalities_and_pve(
            vec![enemy],
            vec![PveEncounter {
                id: "defense_encounter".to_string(),
                abnormality_id: "enemy".to_string(),
                difficulty: 1,
                risk_level: RiskLevel::ZAYIN,
                reward_mode: RewardMode::ClaimAll,
                reward_uuids: vec![],
                node_type: Some(CombatNodeType::Defense),
                mission_variant: None,
                battlefield: Some(PveBattlefieldOverrideData {
                    archetype: Some(crate::game::combat_preview::BattlefieldArchetype::ChokePoint),
                    size_class: Some(crate::game::combat_preview::BattlefieldSizeClass::Small),
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
                        kind: crate::game::combat_preview::EnemyKind::Abnormality,
                        profile_id: None,
                        abnormality_id: "enemy".to_string(),
                        tier: crate::game::enums::Tier::I,
                        count: 1,
                    }],
                }],
                static_obstacles: vec![],
            }],
        );

        let mut preview = CombatPreview::generate_for_node(
            MapNodeId::new(Uuid::from_u128(0x9002_2)),
            MapNodeCategory::Combat,
            Some("defense_encounter"),
            game_data.as_ref(),
            12,
        );
        preview.node_type = CombatNodeType::Boss;
        let mut roster = EmployeeRoster::new();
        roster.add(Employee::new(employee_uuid, "Agent"));
        let deployment_positions =
            HashMap::from([(employee_uuid, preview.deployment_zones[0].cells[0])]);

        let err = match CombatExecutor::build_battle_with_combat_preview(
            &roster,
            &Inventory::new(),
            &SkillFragmentInventory::new(),
            game_data,
            "enemy",
            "defense_encounter",
            7,
            &preview,
            &deployment_positions,
        ) {
            Ok(_) => panic!("battle start must reject preview node_type mismatch"),
            Err(err) => err,
        };

        assert!(matches!(
            err,
            GameError::InvalidStaticData(ref message)
                if message.contains("preview node_type Boss")
                    && message.contains("authored encounter 'defense_encounter' node_type Defense")
        ));
    }

    #[test]
    fn combat_preview_spawn_waves_drive_battle_scenario_spawn_times() {
        let employee_uuid = Uuid::from_u128(0x122);
        let enemy_base_uuid = Uuid::from_u128(0x222);
        let enemy = abnormality("enemy", enemy_base_uuid, 1);
        let game_data = game_data_with_abnormalities_and_pve(
            vec![enemy],
            vec![PveEncounter {
                id: "wave_encounter".to_string(),
                abnormality_id: "enemy".to_string(),
                difficulty: 1,
                risk_level: RiskLevel::ZAYIN,
                reward_mode: RewardMode::ClaimAll,
                reward_uuids: vec![],
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
                    enemies: vec![PveWaveEnemyData {
                        kind: crate::game::combat_preview::EnemyKind::Abnormality,
                        profile_id: None,
                        abnormality_id: "enemy".to_string(),
                        tier: crate::game::enums::Tier::I,
                        count: 1,
                    }],
                }],
                static_obstacles: vec![],
            }],
        );

        let node_id = MapNodeId::new(Uuid::from_u128(0x9003));
        let wave_entry = crate::game::combat_preview::SpawnWaveEnemyEntry {
            kind: crate::game::combat_preview::EnemyKind::Abnormality,
            profile_id: None,
            abnormality_id: "enemy".to_string(),
            tier: crate::game::enums::Tier::I,
            count: 1,
            appearance_seeds: Vec::new(),
        };
        let preview = CombatPreview {
            node_id,
            encounter_id: Some("wave_encounter".to_string()),
            battlefield_template_id: "test_wave_field".to_string(),
            node_type: crate::game::combat_preview::CombatNodeType::Defense,
            mission_variant: crate::game::combat_preview::CombatMissionVariant::Encirclement,
            mission_risk: crate::game::combat_preview::CombatMissionRisk::Controlled,
            archetype: crate::game::combat_preview::BattlefieldArchetype::Ambush,
            size_class: crate::game::combat_preview::BattlefieldSizeClass::Small,
            width: 5,
            height: 5,
            tiles: vec![
                crate::game::combat_preview::BattlefieldTile {
                    position: Position::new(1, 1),
                    kind: crate::game::combat_preview::BattlefieldTileKind::Ground,
                },
                crate::game::combat_preview::BattlefieldTile {
                    position: Position::new(1, 2),
                    kind: crate::game::combat_preview::BattlefieldTileKind::Ground,
                },
                crate::game::combat_preview::BattlefieldTile {
                    position: Position::new(2, 1),
                    kind: crate::game::combat_preview::BattlefieldTileKind::Ground,
                },
            ],
            valid_tiles: vec![
                Position::new(1, 1),
                Position::new(1, 2),
                Position::new(2, 1),
            ],
            deployment_zones: vec![crate::game::combat_preview::DeploymentZone {
                id: "deploy".to_string(),
                label: "Deploy".to_string(),
                kind: crate::game::combat_preview::DeploymentZoneKind::Ground,
                cells: vec![Position::new(1, 1)],
            }],
            spawn_zones: vec![
                crate::game::combat_preview::SpawnZone {
                    id: "entry".to_string(),
                    label: "Entry".to_string(),
                    kind: crate::game::combat_preview::SpawnZoneKind::Entry,
                    confidence: crate::game::combat_preview::ZoneConfidence::Confirmed,
                    cells: vec![Position::new(1, 2)],
                    revealed_details: Vec::new(),
                },
                crate::game::combat_preview::SpawnZone {
                    id: "reinforcement".to_string(),
                    label: "Reinforcement".to_string(),
                    kind: crate::game::combat_preview::SpawnZoneKind::Ambush,
                    confidence: crate::game::combat_preview::ZoneConfidence::Likely,
                    cells: vec![Position::new(2, 1)],
                    revealed_details: Vec::new(),
                },
            ],
            routes: Vec::new(),
            spawn_waves: vec![
                crate::game::combat_preview::SpawnWave {
                    id: "wave_0".to_string(),
                    time_ms: 0,
                    spawn_zone_ids: vec!["entry".to_string()],
                    route_id: None,
                    enemy_entries: vec![wave_entry.clone()],
                    required_for_victory: true,
                },
                crate::game::combat_preview::SpawnWave {
                    id: "wave_1".to_string(),
                    time_ms: 1_000,
                    spawn_zone_ids: vec!["reinforcement".to_string()],
                    route_id: None,
                    enemy_entries: vec![wave_entry],
                    required_for_victory: true,
                },
            ],
            obstacles: Vec::new(),
            enemy_briefing: Vec::new(),
            threat_warnings: Vec::new(),
        };

        let inventory = Inventory::new();
        let skill_fragments = SkillFragmentInventory::new();
        let mut roster = EmployeeRoster::new();
        roster.add(Employee::new(employee_uuid, "Agent"));
        let deployment_positions = HashMap::from([(employee_uuid, Position::new(1, 1))]);

        let mut battle = CombatExecutor::build_battle_with_combat_preview(
            &roster,
            &inventory,
            &skill_fragments,
            game_data,
            "enemy",
            "wave_encounter",
            7,
            &preview,
            &deployment_positions,
        )
        .expect("preview waves should convert into battle scenario waves");
        let mut execution = battle
            .start_battle_execution()
            .expect("live execution should start");
        battle
            .step_battle_execution_until(&mut execution, 1_000)
            .expect("second wave time should be reachable");

        assert!(battle.timeline.entries.iter().any(|entry| {
            entry.time_ms == 1_000
                && matches!(
                    entry.event,
                    TimelineEvent::UnitSpawned {
                        owner: Side::Opponent,
                        ..
                    }
                )
        }));
    }

    #[test]
    fn authored_protect_unit_tactical_plan_overrides_default_defense_contract() {
        let player_owned_uuid = Uuid::from_u128(0x140);
        let player_base_uuid = Uuid::from_u128(0x240);
        let enemy_base_uuid = Uuid::from_u128(0x241);
        let player = abnormality("player_fixture", player_base_uuid, 37);
        let enemy = abnormality("enemy", enemy_base_uuid, 1);
        let game_data = game_data_with_abnormalities_and_pve(
            vec![player, enemy],
            vec![PveEncounter {
                id: "tactical_encounter".to_string(),
                abnormality_id: "enemy".to_string(),
                difficulty: 1,
                risk_level: RiskLevel::ZAYIN,
                reward_mode: RewardMode::ClaimAll,
                reward_uuids: vec![],
                node_type: None,
                mission_variant: None,
                battlefield: None,
                tactical_plan: Some(PveTacticalPlanData {
                    points: vec![PveTacticalPointData {
                        id: "black_box_anchor".to_string(),
                        position: crate::game::data::pve_data::PvePosition { x: 4, y: 8 },
                    }],
                    objective: Some(PveBattleObjectiveData::ProtectUnit {
                        unit_ref: "custom_black_box".to_string(),
                    }),
                }),
                win_condition: Some(PveWinConditionData::ProtectUnit {
                    unit_ref: "custom_black_box".to_string(),
                }),
                waves: vec![PveWaveData {
                    id: "wave_0".to_string(),
                    time_ms: 0,
                    spawn_zone_ids: Vec::new(),
                    route_id: Some("black_box_breach_main".to_string()),
                    required_for_victory: true,
                    source: None,
                    enemies: vec![PveWaveEnemyData {
                        kind: crate::game::combat_preview::EnemyKind::Abnormality,
                        profile_id: None,
                        abnormality_id: "enemy".to_string(),
                        tier: crate::game::enums::Tier::I,
                        count: 1,
                    }],
                }],
                static_obstacles: vec![],
            }],
        );
        let preview = CombatPreview::generate_for_node(
            MapNodeId::new(Uuid::from_u128(0x9004)),
            MapNodeCategory::Combat,
            Some("tactical_encounter"),
            game_data.as_ref(),
            2,
        );
        assert_eq!(
            preview.archetype,
            crate::game::combat_preview::BattlefieldArchetype::ChokePoint
        );

        let plan =
            BattleStartPlan::from_preview(game_data.as_ref(), "tactical_encounter", &preview)
                .expect("preview should convert to battle start plan");
        let player_start =
            CombatExecutor::build_test_player_abnormality_scenario_start_from_positions(
                &[TestPlayerAbnormalityUnit {
                    owned_uuid: player_owned_uuid,
                    base_uuid: player_base_uuid,
                    level: crate::game::enums::Tier::I,
                }],
                &Inventory::new(),
                &HashMap::from([(player_owned_uuid, preview.deployment_zones[0].cells[0])]),
            )
            .expect("test player scenario start");
        let scenario = CombatExecutor::build_battle_scenario_from_preview(
            game_data.as_ref(),
            "tactical_encounter",
            &preview,
            plan,
            player_start,
        )
        .expect("battle scenario should build");

        assert!(matches!(
            scenario.tactical_plan.player_plan,
            PlayerMovementPlan::FixedDefense
        ));
        assert_eq!(scenario.tactical_plan.points.len(), 1);
        assert!(matches!(
            scenario.tactical_plan.enemy_plan,
            EnemyMovementPlan::PathAlongCells { ref cells } if !cells.is_empty()
        ));
        assert!(matches!(
            scenario.win_condition,
            WinCondition::ProtectUnit { ref unit_ref } if unit_ref.0 == "custom_black_box"
        ));
        assert!(scenario.groups.iter().any(|group| {
            group.id.0 == DEFAULT_DEFENSE_OBJECT_GROUP
                && group
                    .spawns
                    .iter()
                    .any(|spawn| spawn.unit_ref.0 == "custom_black_box")
        }));
    }

    #[test]
    fn defense_node_type_builds_default_black_box_defense_objective() {
        let player_owned_uuid = Uuid::from_u128(0x131);
        let player_base_uuid = Uuid::from_u128(0x232);
        let enemy_base_uuid = Uuid::from_u128(0x233);
        let player = abnormality("player_fixture", player_base_uuid, 37);
        let enemy = abnormality("enemy", enemy_base_uuid, 1);
        let game_data = game_data_with_abnormalities_and_pve(
            vec![player, enemy],
            vec![PveEncounter {
                id: "default_defense_encounter".to_string(),
                abnormality_id: "enemy".to_string(),
                difficulty: 1,
                risk_level: RiskLevel::ZAYIN,
                reward_mode: RewardMode::ClaimAll,
                reward_uuids: vec![],
                node_type: Some(CombatNodeType::Defense),
                mission_variant: None,
                battlefield: Some(PveBattlefieldOverrideData {
                    archetype: Some(crate::game::combat_preview::BattlefieldArchetype::ChokePoint),
                    size_class: Some(crate::game::combat_preview::BattlefieldSizeClass::Small),
                }),
                tactical_plan: None,
                win_condition: None,
                waves: vec![PveWaveData {
                    id: "wave_0".to_string(),
                    time_ms: 0,
                    spawn_zone_ids: Vec::new(),
                    route_id: Some("black_box_breach_main".to_string()),
                    required_for_victory: true,
                    source: None,
                    enemies: vec![PveWaveEnemyData {
                        kind: crate::game::combat_preview::EnemyKind::Abnormality,
                        profile_id: None,
                        abnormality_id: "enemy".to_string(),
                        tier: crate::game::enums::Tier::I,
                        count: 1,
                    }],
                }],
                static_obstacles: vec![],
            }],
        );
        let preview = CombatPreview::generate_for_node(
            MapNodeId::new(Uuid::from_u128(0x9005)),
            MapNodeCategory::Combat,
            Some("default_defense_encounter"),
            game_data.as_ref(),
            0,
        );
        assert_eq!(preview.node_type, CombatNodeType::Defense);

        let plan = BattleStartPlan::from_preview(
            game_data.as_ref(),
            "default_defense_encounter",
            &preview,
        )
        .expect("preview should convert to battle start plan");
        let player_start =
            CombatExecutor::build_test_player_abnormality_scenario_start_from_positions(
                &[TestPlayerAbnormalityUnit {
                    owned_uuid: player_owned_uuid,
                    base_uuid: player_base_uuid,
                    level: crate::game::enums::Tier::I,
                }],
                &Inventory::new(),
                &HashMap::from([(player_owned_uuid, preview.deployment_zones[0].cells[0])]),
            )
            .expect("test player scenario start");
        let scenario = CombatExecutor::build_battle_scenario_from_preview(
            game_data.as_ref(),
            "default_defense_encounter",
            &preview,
            plan,
            player_start,
        )
        .expect("battle scenario should build");

        assert!(matches!(
            scenario.tactical_plan.objective,
            crate::game::battle::scenario::BattleObjective::ProtectUnit {
                ref unit_ref,
            } if unit_ref.0 == DEFAULT_DEFENSE_OBJECT_REF
        ));
        assert!(matches!(
            scenario.tactical_plan.player_plan,
            PlayerMovementPlan::FixedDefense
        ));
        let EnemyMovementPlan::PathAlongCells { ref cells } = scenario.tactical_plan.enemy_plan;
        assert_eq!(
            cells.last().copied(),
            preview.routes.first().map(|route| route.end)
        );
        let enemy_group = scenario
            .groups
            .iter()
            .find(|group| group.side == Side::Opponent)
            .expect("defense wave should become an opponent spawn group");
        let Some(EnemyMovementPlan::PathAlongCells { cells: wave_cells }) =
            &enemy_group.enemy_movement_plan
        else {
            panic!("defense wave route_id should select a route movement plan");
        };
        assert_eq!(
            wave_cells.last().copied(),
            preview.routes.first().map(|route| route.end)
        );
        assert!(matches!(
            scenario.win_condition,
            WinCondition::ProtectUnit {
                ref unit_ref,
            } if unit_ref.0 == DEFAULT_DEFENSE_OBJECT_REF
        ));
        assert!(scenario
            .groups
            .iter()
            .any(|group| group.id.0 == DEFAULT_DEFENSE_OBJECT_GROUP
                && group.spawns.iter().any(|spawn| {
                    spawn.unit_ref.0 == DEFAULT_DEFENSE_OBJECT_REF
                        && matches!(spawn.draft.source, BattleUnitSource::DefenseObject { .. })
                })));
        assert!(scenario
            .groups
            .iter()
            .any(|group| group.id.0 == DEFAULT_DEFENSE_OBJECT_GROUP
                && group.spawns.iter().any(|spawn| {
                    Some(spawn.position) == preview.routes.first().map(|route| route.end)
                })));
    }

    #[test]
    fn defense_node_type_builds_fixed_defense_objective() {
        let player_owned_uuid = Uuid::from_u128(0x161);
        let player_base_uuid = Uuid::from_u128(0x262);
        let enemy_base_uuid = Uuid::from_u128(0x263);
        let player = abnormality("player_fixture", player_base_uuid, 37);
        let enemy = abnormality("enemy", enemy_base_uuid, 1);
        let game_data = game_data_with_abnormalities_and_pve(
            vec![player, enemy],
            vec![PveEncounter {
                id: "default_defense_encounter".to_string(),
                abnormality_id: "enemy".to_string(),
                difficulty: 1,
                risk_level: RiskLevel::ZAYIN,
                reward_mode: RewardMode::ClaimAll,
                reward_uuids: vec![],
                node_type: Some(CombatNodeType::Defense),
                mission_variant: None,
                battlefield: Some(PveBattlefieldOverrideData {
                    archetype: Some(crate::game::combat_preview::BattlefieldArchetype::Corridor),
                    size_class: Some(crate::game::combat_preview::BattlefieldSizeClass::Small),
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
                        kind: crate::game::combat_preview::EnemyKind::Abnormality,
                        profile_id: None,
                        abnormality_id: "enemy".to_string(),
                        tier: crate::game::enums::Tier::I,
                        count: 1,
                    }],
                }],
                static_obstacles: vec![],
            }],
        );
        let preview = CombatPreview::generate_for_node(
            MapNodeId::new(Uuid::from_u128(0x9007)),
            MapNodeCategory::Combat,
            Some("default_defense_encounter"),
            game_data.as_ref(),
            0,
        );

        let plan = BattleStartPlan::from_preview(
            game_data.as_ref(),
            "default_defense_encounter",
            &preview,
        )
        .expect("preview should convert to battle start plan");
        let player_start =
            CombatExecutor::build_test_player_abnormality_scenario_start_from_positions(
                &[TestPlayerAbnormalityUnit {
                    owned_uuid: player_owned_uuid,
                    base_uuid: player_base_uuid,
                    level: crate::game::enums::Tier::I,
                }],
                &Inventory::new(),
                &HashMap::from([(player_owned_uuid, preview.deployment_zones[0].cells[0])]),
            )
            .expect("test player scenario start");
        let scenario = CombatExecutor::build_battle_scenario_from_preview(
            game_data.as_ref(),
            "default_defense_encounter",
            &preview,
            plan,
            player_start,
        )
        .expect("battle scenario should build");

        assert!(matches!(
            scenario.tactical_plan.objective,
            crate::game::battle::scenario::BattleObjective::ProtectUnit { .. }
        ));
        assert!(matches!(
            scenario.tactical_plan.player_plan,
            PlayerMovementPlan::FixedDefense
        ));
        assert!(matches!(
            scenario.win_condition,
            WinCondition::ProtectUnit { .. }
        ));
    }

    #[test]
    fn encirclement_mission_variant_builds_survival_hold_objective() {
        let player_owned_uuid = Uuid::from_u128(0x171);
        let player_base_uuid = Uuid::from_u128(0x272);
        let enemy_base_uuid = Uuid::from_u128(0x273);
        let player = abnormality("player_fixture", player_base_uuid, 37);
        let enemy = abnormality("enemy", enemy_base_uuid, 1);
        let game_data = game_data_with_abnormalities_and_pve(
            vec![player, enemy],
            vec![PveEncounter {
                id: "default_encirclement_encounter".to_string(),
                abnormality_id: "enemy".to_string(),
                difficulty: 1,
                risk_level: RiskLevel::ZAYIN,
                reward_mode: RewardMode::ClaimAll,
                reward_uuids: vec![],
                node_type: Some(CombatNodeType::Defense),
                mission_variant: Some(
                    crate::game::combat_preview::CombatMissionVariant::Encirclement,
                ),
                battlefield: Some(PveBattlefieldOverrideData {
                    archetype: Some(crate::game::combat_preview::BattlefieldArchetype::Surrounded),
                    size_class: Some(crate::game::combat_preview::BattlefieldSizeClass::Small),
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
                        kind: crate::game::combat_preview::EnemyKind::Abnormality,
                        profile_id: None,
                        abnormality_id: "enemy".to_string(),
                        tier: crate::game::enums::Tier::I,
                        count: 1,
                    }],
                }],
                static_obstacles: vec![],
            }],
        );
        let preview = CombatPreview::generate_for_node(
            MapNodeId::new(Uuid::from_u128(0x9008)),
            MapNodeCategory::Combat,
            Some("default_encirclement_encounter"),
            game_data.as_ref(),
            0,
        );

        let plan = BattleStartPlan::from_preview(
            game_data.as_ref(),
            "default_encirclement_encounter",
            &preview,
        )
        .expect("preview should convert to battle start plan");
        let player_start =
            CombatExecutor::build_test_player_abnormality_scenario_start_from_positions(
                &[TestPlayerAbnormalityUnit {
                    owned_uuid: player_owned_uuid,
                    base_uuid: player_base_uuid,
                    level: crate::game::enums::Tier::I,
                }],
                &Inventory::new(),
                &HashMap::from([(player_owned_uuid, preview.deployment_zones[0].cells[0])]),
            )
            .expect("test player scenario start");
        let scenario = CombatExecutor::build_battle_scenario_from_preview(
            game_data.as_ref(),
            "default_encirclement_encounter",
            &preview,
            plan,
            player_start,
        )
        .expect("battle scenario should build");

        assert!(matches!(
            scenario.tactical_plan.objective,
            crate::game::battle::scenario::BattleObjective::Survive { time_ms: 45_000 }
        ));
        assert!(matches!(
            scenario.tactical_plan.player_plan,
            PlayerMovementPlan::FixedDefense
        ));
        assert!(matches!(
            scenario.win_condition,
            WinCondition::SurviveUntil { time_ms: 45_000 }
        ));
    }

    #[test]
    fn test_harness_can_field_player_side_abnormality_without_inventory_ownership() {
        let player_owned_uuid = Uuid::from_u128(0x130);
        let player_base_uuid = Uuid::from_u128(0x230);
        let enemy_base_uuid = Uuid::from_u128(0x231);
        let player = abnormality("player_fixture", player_base_uuid, 37);
        let enemy = abnormality("enemy", enemy_base_uuid, 1);
        let game_data = game_data_with_abnormalities_and_pve(
            vec![player, enemy],
            vec![PveEncounter {
                id: "fixture_encounter".to_string(),
                abnormality_id: "enemy".to_string(),
                difficulty: 1,
                risk_level: RiskLevel::ZAYIN,
                reward_mode: RewardMode::ClaimAll,
                reward_uuids: vec![],
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
                    enemies: vec![PveWaveEnemyData {
                        kind: crate::game::combat_preview::EnemyKind::Abnormality,
                        profile_id: None,
                        abnormality_id: "enemy".to_string(),
                        tier: crate::game::enums::Tier::I,
                        count: 1,
                    }],
                }],
                static_obstacles: vec![],
            }],
        );

        let inventory = Inventory::new();
        let preview = CombatPreview::generate_for_node(
            MapNodeId::new(Uuid::from_u128(0x9002)),
            MapNodeCategory::Combat,
            Some("fixture_encounter"),
            game_data.as_ref(),
            11,
        );
        let deployment_positions =
            HashMap::from([(player_owned_uuid, preview.deployment_zones[0].cells[0])]);

        let plan = BattleStartPlan::from_preview(game_data.as_ref(), "fixture_encounter", &preview)
            .expect("preview should convert to battle start plan");
        let player_start =
            CombatExecutor::build_test_player_abnormality_scenario_start_from_positions(
                &[TestPlayerAbnormalityUnit {
                    owned_uuid: player_owned_uuid,
                    base_uuid: player_base_uuid,
                    level: crate::game::enums::Tier::I,
                }],
                &inventory,
                &deployment_positions,
            )
            .expect("test-only player abnormality fixture should build scenario start");
        let scenario = CombatExecutor::build_battle_scenario_from_preview(
            game_data.as_ref(),
            "fixture_encounter",
            &preview,
            plan,
            player_start,
        )
        .expect("test-only player abnormality fixture should build battle scenario");

        let player_spawn = scenario
            .groups
            .iter()
            .flat_map(|group| group.spawns.iter())
            .find(|spawn| {
                spawn.side == Side::Player
                    && matches!(
                        spawn.draft.source,
                        BattleUnitSource::Abnormality { base_uuid } if base_uuid == player_base_uuid
                    )
            })
            .expect("scenario should include player abnormality fixture");
        assert_eq!(player_spawn.draft.owned_uuid, player_owned_uuid);
    }
}
