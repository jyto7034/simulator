use std::{collections::HashMap, sync::Arc};

use tracing::{info, warn};
use uuid::Uuid;

use crate::{
    game::resources::{Inventory, Position},
    game::{
        battle::{
            core::BattleCore,
            scenario::{
                BattleScenario, ScenarioGroupId, ScenarioSpawnGroup, ScenarioUnitRef,
                ScenarioUnitSpawn,
            },
            types::{BattleResult, BattleUnitDraft, BattleUnitSource},
        },
        behavior::GameError,
        combat_battlefield_plan::{
            validate_static_obstacles_do_not_overlap_scenario, BattleStartPlan,
        },
        combat_defense_object::defense_object_group_for_win_condition,
        combat_enemy_spawns::enemy_spawn_groups_from_preview,
        combat_mission_policy::CombatMissionPolicy,
        combat_player_spawns::{
            player_scenario_start_from_positions, scenario_artifacts_from_inventory,
            PlayerScenarioStart,
        },
        combat_preview::{CombatNodeType, CombatPreview},
        combat_rewards::resolve_combat_rewards_from_encounter,
        combat_scenario_groups::{push_start_spawn_group, push_timed_spawn_group},
        data::GameDataBase,
        employee::EmployeeRoster,
        enums::{RewardMode, Side},
        growth::GrowthStack,
        reward::RewardOption,
        skill_fragment::SkillFragmentInventory,
    },
};

/// Node combat business logic helper.
pub struct CombatExecutor;

#[derive(Debug, Clone, Copy)]
pub struct TestPlayerAbnormalityUnit {
    pub owned_uuid: Uuid,
    pub base_uuid: Uuid,
    pub level: crate::game::enums::Tier,
}

impl CombatExecutor {
    pub fn start_battle_with_combat_preview(
        roster: &EmployeeRoster,
        inventory: &Inventory,
        skill_fragments: &SkillFragmentInventory,
        game_data: Arc<GameDataBase>,
        abnormality_id: &str,
        encounter_id: &str,
        movement_seed: u64,
        combat_preview: &CombatPreview,
        deployment_positions: &HashMap<Uuid, Position>,
    ) -> Result<BattleResult, GameError> {
        let plan = BattleStartPlan::from_preview(&game_data, encounter_id, combat_preview)?;
        Self::start_battle_with_explicit_deployment(
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

    fn start_battle_with_explicit_deployment(
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
    ) -> Result<BattleResult, GameError> {
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

        let mut battle = BattleCore::new_from_scenario(scenario, game_data, movement_seed);
        let result = battle.run_battle()?;

        info!("Node combat completed");

        Ok(result)
    }

    /// Test harness for battle scenarios that intentionally field abnormalities on the player side.
    ///
    /// This does not grant, own, buy, or store abnormalities in player inventory. It only converts
    /// explicit test fixtures into a scenario spawn group so combat behavior can be validated against
    /// abnormality metadata when needed.
    pub fn start_test_battle_with_player_abnormalities(
        player_units: &[TestPlayerAbnormalityUnit],
        inventory: &Inventory,
        game_data: Arc<GameDataBase>,
        abnormality_id: &str,
        encounter_id: &str,
        movement_seed: u64,
        combat_preview: &CombatPreview,
        deployment_positions: &HashMap<Uuid, Position>,
    ) -> Result<BattleResult, GameError> {
        info!(
            "Starting test node combat with player-side abnormality fixtures for abnormality={} encounter={}",
            abnormality_id, encounter_id
        );

        let player_start = Self::build_test_player_abnormality_scenario_start_from_positions(
            player_units,
            inventory,
            deployment_positions,
        )?;
        let plan = BattleStartPlan::from_preview(&game_data, encounter_id, combat_preview)?;
        let scenario = Self::build_battle_scenario_from_preview(
            &game_data,
            encounter_id,
            combat_preview,
            plan,
            player_start,
        )?;
        validate_static_obstacles_do_not_overlap_scenario(&scenario)?;

        let mut battle = BattleCore::new_from_scenario(scenario, game_data, movement_seed);
        battle.run_battle()
    }

    pub fn resolve_rewards(
        game_data: &GameDataBase,
        encounter_id: &str,
    ) -> Result<(RewardMode, Vec<RewardOption>), GameError> {
        let encounter = game_data
            .pve_data
            .get_by_id(encounter_id)
            .ok_or(GameError::MissingResource("PveEncounter"))?;

        resolve_combat_rewards_from_encounter(game_data, encounter, encounter.node_type)
    }

    pub fn resolve_rewards_for_node_type(
        game_data: &GameDataBase,
        encounter_id: &str,
        node_type: CombatNodeType,
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

        resolve_combat_rewards_from_encounter(game_data, encounter, Some(node_type))
    }

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

        let mut tactical_plan = CombatMissionPolicy::default_tactical_plan_for_preview(
            combat_preview.node_type,
            combat_preview,
        );
        encounter.apply_authored_tactical_plan(&mut tactical_plan);
        let win_condition = encounter.authored_win_condition().unwrap_or_else(|| {
            CombatMissionPolicy::default_win_condition_for_tactical_plan(
                combat_preview.node_type,
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
            PveEnemyMovementPlanData, PveTacticalPlanData, PveTacticalPointData, PveWaveData,
            PveWaveEnemyData, PveWinConditionData,
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
                battlefield: None,
                tactical_plan: None,
                win_condition: None,
                waves: vec![PveWaveData {
                    id: "wave_0".to_string(),
                    time_ms: 0,
                    spawn_zone_ids: Vec::new(),
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

        let err = CombatExecutor::resolve_rewards_for_node_type(
            game_data.as_ref(),
            "boss_contract",
            CombatNodeType::Suppression,
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
                battlefield: None,
                tactical_plan: None,
                win_condition: None,
                waves: vec![PveWaveData {
                    id: "wave_0".to_string(),
                    time_ms: 0,
                    spawn_zone_ids: Vec::new(),
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
            false,
            11,
        );
        let inventory = Inventory::new();
        let skill_fragments = SkillFragmentInventory::new();
        let mut roster = EmployeeRoster::new();
        roster.add(Employee::new(employee_uuid, "Agent"));

        let deployment_positions =
            HashMap::from([(employee_uuid, preview.deployment_zones[0].cells[0])]);

        let result = CombatExecutor::start_battle_with_combat_preview(
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

        let battle_start = result
            .timeline
            .entries
            .iter()
            .find_map(|entry| match entry.event {
                TimelineEvent::BattleStart { width, height } => Some((width, height)),
                _ => None,
            })
            .expect("timeline should contain BattleStart");
        assert_eq!(
            battle_start,
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
                battlefield: None,
                tactical_plan: None,
                win_condition: None,
                waves: vec![PveWaveData {
                    id: "wave_0".to_string(),
                    time_ms: 0,
                    spawn_zone_ids: Vec::new(),
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
            false,
            12,
        );
        preview.spawn_waves.clear();

        let inventory = Inventory::new();
        let skill_fragments = SkillFragmentInventory::new();
        let mut roster = EmployeeRoster::new();
        roster.add(Employee::new(employee_uuid, "Agent"));
        let deployment_positions =
            HashMap::from([(employee_uuid, preview.deployment_zones[0].cells[0])]);

        let err = match CombatExecutor::start_battle_with_combat_preview(
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
            node_type: Some(CombatNodeType::Suppression),
            battlefield: None,
            tactical_plan: None,
            win_condition: None,
            waves: vec![PveWaveData {
                id: "wave_0".to_string(),
                time_ms: 0,
                spawn_zone_ids: Vec::new(),
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
            false,
            12,
        );
        let mut roster = EmployeeRoster::new();
        roster.add(Employee::new(employee_uuid, "Agent"));
        let deployment_positions =
            HashMap::from([(employee_uuid, preview.deployment_zones[0].cells[0])]);

        let err = match CombatExecutor::start_battle_with_combat_preview(
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
            false,
            12,
        );
        preview.node_type = CombatNodeType::Suppression;
        let mut roster = EmployeeRoster::new();
        roster.add(Employee::new(employee_uuid, "Agent"));
        let deployment_positions =
            HashMap::from([(employee_uuid, preview.deployment_zones[0].cells[0])]);

        let err = match CombatExecutor::start_battle_with_combat_preview(
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
                if message.contains("preview node_type Suppression")
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
                battlefield: None,
                tactical_plan: None,
                win_condition: None,
                waves: vec![PveWaveData {
                    id: "wave_0".to_string(),
                    time_ms: 0,
                    spawn_zone_ids: Vec::new(),
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
            node_type: crate::game::combat_preview::CombatNodeType::Suppression,
            mission_risk: crate::game::combat_preview::CombatMissionRisk::Controlled,
            archetype: crate::game::combat_preview::BattlefieldArchetype::Ambush,
            size_class: crate::game::combat_preview::BattlefieldSizeClass::Small,
            width: 5,
            height: 5,
            valid_tiles: Vec::new(),
            deployment_zones: vec![crate::game::combat_preview::DeploymentZone {
                id: "deploy".to_string(),
                label: "Deploy".to_string(),
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
            spawn_waves: vec![
                crate::game::combat_preview::SpawnWave {
                    id: "wave_0".to_string(),
                    time_ms: 0,
                    spawn_zone_ids: vec!["entry".to_string()],
                    enemy_entries: vec![wave_entry.clone()],
                    required_for_victory: true,
                    revealed_by_recon: true,
                },
                crate::game::combat_preview::SpawnWave {
                    id: "wave_1".to_string(),
                    time_ms: 1_000,
                    spawn_zone_ids: vec!["reinforcement".to_string()],
                    enemy_entries: vec![wave_entry],
                    required_for_victory: true,
                    revealed_by_recon: false,
                },
            ],
            obstacles: Vec::new(),
            enemy_briefing: Vec::new(),
            recon_available: true,
            recon_revealed: false,
        };

        let inventory = Inventory::new();
        let skill_fragments = SkillFragmentInventory::new();
        let mut roster = EmployeeRoster::new();
        roster.add(Employee::new(employee_uuid, "Agent"));
        let deployment_positions = HashMap::from([(employee_uuid, Position::new(1, 1))]);

        let result = CombatExecutor::start_battle_with_combat_preview(
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

        assert!(result.timeline.entries.iter().any(|entry| {
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
                battlefield: None,
                tactical_plan: Some(PveTacticalPlanData {
                    points: vec![PveTacticalPointData {
                        id: "black_box_recovery".to_string(),
                        position: crate::game::data::pve_data::PvePosition { x: 4, y: 8 },
                    }],
                    objective: Some(PveBattleObjectiveData::ProtectUnit {
                        unit_ref: "custom_black_box".to_string(),
                    }),
                    enemy_plan: Some(PveEnemyMovementPlanData::PathToPoint {
                        point_id: "black_box_recovery".to_string(),
                    }),
                }),
                win_condition: Some(PveWinConditionData::ProtectUnit {
                    unit_ref: "custom_black_box".to_string(),
                }),
                waves: vec![PveWaveData {
                    id: "wave_0".to_string(),
                    time_ms: 0,
                    spawn_zone_ids: Vec::new(),
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
            false,
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
            PlayerMovementPlan::HoldDeployment { .. }
        ));
        assert_eq!(scenario.tactical_plan.points.len(), 1);
        assert!(matches!(
            scenario.tactical_plan.enemy_plan,
            EnemyMovementPlan::PathToPoint { ref point_id } if point_id.0 == "black_box_recovery"
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
            false,
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
            crate::game::battle::scenario::BattleObjective::ProtectUnitForDuration {
                ref unit_ref,
                time_ms: 30_000,
                cleanup_required: false,
            } if unit_ref.0 == DEFAULT_DEFENSE_OBJECT_REF
        ));
        assert!(matches!(
            scenario.tactical_plan.player_plan,
            PlayerMovementPlan::HoldDeployment { .. }
        ));
        assert!(matches!(
            scenario.tactical_plan.enemy_plan,
            EnemyMovementPlan::PathToPoint { ref point_id }
                if point_id.0 == "black_box_recovery"
        ));
        assert!(matches!(
            scenario.win_condition,
            WinCondition::ProtectUnitForDuration {
                ref unit_ref,
                time_ms: 30_000,
                cleanup_required: false,
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
        assert!(preview.deployment_zones[0]
            .cells
            .contains(&scenario.tactical_plan.points[0].position));
    }

    #[test]
    fn recovery_node_type_builds_default_recover_and_extract_objective() {
        let player_owned_uuid = Uuid::from_u128(0x151);
        let player_base_uuid = Uuid::from_u128(0x252);
        let enemy_base_uuid = Uuid::from_u128(0x253);
        let player = abnormality("player_fixture", player_base_uuid, 37);
        let enemy = abnormality("enemy", enemy_base_uuid, 1);
        let game_data = game_data_with_abnormalities_and_pve(
            vec![player, enemy],
            vec![PveEncounter {
                id: "default_recovery_encounter".to_string(),
                abnormality_id: "enemy".to_string(),
                difficulty: 1,
                risk_level: RiskLevel::ZAYIN,
                reward_mode: RewardMode::ClaimAll,
                reward_uuids: vec![],
                node_type: Some(CombatNodeType::Recovery),
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
            MapNodeId::new(Uuid::from_u128(0x9006)),
            MapNodeCategory::Combat,
            Some("default_recovery_encounter"),
            game_data.as_ref(),
            false,
            0,
        );
        assert_eq!(preview.node_type, CombatNodeType::Recovery);

        let plan = BattleStartPlan::from_preview(
            game_data.as_ref(),
            "default_recovery_encounter",
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
            "default_recovery_encounter",
            &preview,
            plan,
            player_start,
        )
        .expect("battle scenario should build");

        assert!(matches!(
            scenario.tactical_plan.objective,
            crate::game::battle::scenario::BattleObjective::RecoverHoldAndExtract {
                ref target_point_id,
                ref extraction_point_id,
                hold_duration_ms: 5_000,
            } if target_point_id.0 == "recovery_target"
                && extraction_point_id.0 == "extraction_point"
        ));
        assert!(matches!(
            scenario.tactical_plan.player_plan,
            PlayerMovementPlan::CautiousEngage { .. }
        ));
        assert!(scenario
            .tactical_plan
            .group_plans
            .iter()
            .any(|group| matches!(
                group.objective,
                crate::game::battle::scenario::GroupObjective::AdvanceAlongPath { ref point_ids }
                    if point_ids.len() == 2
                        && point_ids[0].0 == "recovery_target"
                        && point_ids[1].0 == "extraction_point"
            )));
        assert!(matches!(
            scenario.win_condition,
            WinCondition::RecoverHoldAndExtract {
                ref target_point_id,
                ref extraction_point_id,
                target_radius,
                extraction_radius,
                hold_duration_ms: 5_000,
            } if target_point_id.0 == "recovery_target"
                && extraction_point_id.0 == "extraction_point"
                && (target_radius - 0.75).abs() <= f32::EPSILON
                && (extraction_radius - 0.75).abs() <= f32::EPSILON
        ));
    }

    #[test]
    fn frontline_node_type_builds_limited_forward_engagement_objective() {
        let player_owned_uuid = Uuid::from_u128(0x161);
        let player_base_uuid = Uuid::from_u128(0x262);
        let enemy_base_uuid = Uuid::from_u128(0x263);
        let player = abnormality("player_fixture", player_base_uuid, 37);
        let enemy = abnormality("enemy", enemy_base_uuid, 1);
        let game_data = game_data_with_abnormalities_and_pve(
            vec![player, enemy],
            vec![PveEncounter {
                id: "default_frontline_encounter".to_string(),
                abnormality_id: "enemy".to_string(),
                difficulty: 1,
                risk_level: RiskLevel::ZAYIN,
                reward_mode: RewardMode::ClaimAll,
                reward_uuids: vec![],
                node_type: Some(CombatNodeType::Frontline),
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
            Some("default_frontline_encounter"),
            game_data.as_ref(),
            false,
            0,
        );

        let plan = BattleStartPlan::from_preview(
            game_data.as_ref(),
            "default_frontline_encounter",
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
            "default_frontline_encounter",
            &preview,
            plan,
            player_start,
        )
        .expect("battle scenario should build");

        assert!(matches!(
            scenario.tactical_plan.objective,
            crate::game::battle::scenario::BattleObjective::SuppressAll
        ));
        assert!(matches!(
            scenario.tactical_plan.player_plan,
            PlayerMovementPlan::CautiousEngage {
                leash_radius,
                chase_radius,
            } if (leash_radius - 5.0).abs() <= f32::EPSILON
                && (chase_radius - 1.5).abs() <= f32::EPSILON
        ));
        assert!(matches!(
            scenario.win_condition,
            WinCondition::AllRequiredEnemyGroupsDefeated
        ));
    }

    #[test]
    fn encirclement_node_type_builds_survival_hold_objective() {
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
                node_type: Some(CombatNodeType::Encirclement),
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
            false,
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
            PlayerMovementPlan::HoldDeployment { .. }
        ));
        assert!(scenario
            .tactical_plan
            .group_plans
            .iter()
            .any(|group| matches!(
                group.objective,
                crate::game::battle::scenario::GroupObjective::HoldArea { ref point_id }
                    if point_id.0 == "survival_anchor"
            )));
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
                battlefield: None,
                tactical_plan: None,
                win_condition: None,
                waves: vec![PveWaveData {
                    id: "wave_0".to_string(),
                    time_ms: 0,
                    spawn_zone_ids: Vec::new(),
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
            false,
            11,
        );
        let deployment_positions =
            HashMap::from([(player_owned_uuid, preview.deployment_zones[0].cells[0])]);

        let result = CombatExecutor::start_test_battle_with_player_abnormalities(
            &[TestPlayerAbnormalityUnit {
                owned_uuid: player_owned_uuid,
                base_uuid: player_base_uuid,
                level: crate::game::enums::Tier::I,
            }],
            &inventory,
            game_data,
            "enemy",
            "fixture_encounter",
            7,
            &preview,
            &deployment_positions,
        )
        .expect("test-only player abnormality fixture should start battle");

        let player_result = result
            .participant_results
            .iter()
            .find(|participant| {
                participant.side == Side::Player && participant.owned_uuid == player_owned_uuid
            })
            .expect("participant summary should include player abnormality fixture");
        assert_eq!(player_result.max_hp, 30);
        assert!(player_result.survived);
    }
}
