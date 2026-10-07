use std::{collections::HashMap, sync::Arc};

use tracing::info;
use uuid::Uuid;

#[cfg(test)]
use tracing::warn;

use crate::{
    game::resources::{Inventory, Position},
    game::{
        battle::{
            core::{sim::BattleExecutionState, BattleCore},
            scenario::BattleScenario,
        },
        behavior::GameError,
        combat_preview::{CombatNodeType, CombatPreview},
        combat_setup::battlefield_plan::{
            validate_static_obstacles_do_not_overlap_scenario, BattleStartPlan,
        },
        combat_setup::boss_runtime::resolve_boss_runtime_setup_with_preview,
        combat_setup::defense_object::defense_object_group_for_tactical_plan,
        combat_setup::enemy_spawns::enemy_spawn_groups_from_preview_with_overrides,
        combat_setup::mission_policy::CombatMissionPolicy,
        combat_setup::player_spawns::{player_scenario_start_from_positions, PlayerScenarioStart},
        combat_setup::rewards::{
            resolve_combat_outcome_policies, resolve_combat_rewards_from_encounter,
            ResolvedCombatOutcomePolicy,
        },
        combat_setup::scenario_groups::{push_start_spawn_group, push_wave_spawn_group},
        data::GameDataBase,
        employee::EmployeeRoster,
        enums::RewardMode,
        map::AuthoredCombatNodePolicy,
        reward::RewardOption,
        skill_fragment::SkillFragmentInventory,
    },
};

#[cfg(test)]
use crate::game::{
    battle::{
        scenario::{ScenarioGroupId, ScenarioSpawnGroup, ScenarioUnitRef, ScenarioUnitSpawn},
        types::{BattleUnitDraft, BattleUnitSource, BattleUnitThreatClass},
    },
    enums::Side,
    growth::GrowthStack,
};

/// Node combat business logic helper.
pub struct CombatExecutor;

/// A crate-private typed handoff from an already activated World relic owner.
/// It registers the additional battle owner, never selects safety limits.
pub(crate) struct RelicBattleRuntimeOwner;

/// Fully validated private handoff from combat setup into the world battle-start
/// transaction. Active-session identity and world lifecycle remain world-owned.
pub(crate) struct ResolvedCombatSetupPlan {
    pub(crate) node_type: CombatNodeType,
    pub(crate) mission_variant: crate::game::combat_preview::CombatMissionVariant,
    pub(crate) outcome_policies: Vec<ResolvedCombatOutcomePolicy>,
    pub(crate) battle: BattleCore,
    pub(crate) execution: BattleExecutionState,
}

#[derive(Debug, Clone, Copy)]
#[cfg(test)]
pub struct TestPlayerAbnormalityUnit {
    pub owned_uuid: Uuid,
    pub base_uuid: Uuid,
    pub level: crate::game::enums::Tier,
}

impl CombatExecutor {
    pub(crate) fn validate_live_combat_setup_feasibility(
        game_data: Arc<GameDataBase>,
        encounter_id: &str,
        movement_seed: u64,
        combat_preview: &CombatPreview,
        reward_multiplier_percent: u32,
    ) -> Result<(), GameError> {
        let primary_abnormality_id = game_data
            .pve_data
            .get_by_id(encounter_id)
            .ok_or(GameError::MissingResource("PveEncounter"))?
            .primary_abnormality_id()
            .map(str::to_string);
        Self::resolve_live_combat_setup_plan(
            &EmployeeRoster::new(),
            &Inventory::new(),
            &SkillFragmentInventory::new(),
            game_data,
            primary_abnormality_id.as_deref(),
            encounter_id,
            movement_seed,
            combat_preview,
            &HashMap::new(),
            reward_multiplier_percent,
        )
        .map(|_| ())
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn resolve_live_combat_setup_plan(
        roster: &EmployeeRoster,
        inventory: &Inventory,
        skill_fragments: &SkillFragmentInventory,
        game_data: Arc<GameDataBase>,
        primary_abnormality_id: Option<&str>,
        encounter_id: &str,
        movement_seed: u64,
        combat_preview: &CombatPreview,
        deployment_positions: &HashMap<Uuid, Position>,
        reward_multiplier_percent: u32,
    ) -> Result<ResolvedCombatSetupPlan, GameError> {
        Self::resolve_live_combat_setup_plan_with_authored_policy(
            roster,
            inventory,
            skill_fragments,
            game_data,
            None,
            primary_abnormality_id,
            encounter_id,
            movement_seed,
            combat_preview,
            deployment_positions,
            reward_multiplier_percent,
            None,
            None,
            None,
            None,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn resolve_live_combat_setup_plan_with_authored_policy(
        roster: &EmployeeRoster,
        inventory: &Inventory,
        skill_fragments: &SkillFragmentInventory,
        game_data: Arc<GameDataBase>,
        content_identity: Option<Arc<crate::game::data::content_pack::ContentIdentity>>,
        primary_abnormality_id: Option<&str>,
        encounter_id: &str,
        movement_seed: u64,
        combat_preview: &CombatPreview,
        deployment_positions: &HashMap<Uuid, Position>,
        reward_multiplier_percent: u32,
        authored_policy: Option<&AuthoredCombatNodePolicy>,
        authoring_observation: Option<(crate::authoring::trace::TraceOptions, bool)>,
        relic_runtime_owner: Option<RelicBattleRuntimeOwner>,
        relic_numeric_owner: Option<
            Arc<crate::game::battle::core::relic_numeric::RelicNumericOwner>,
        >,
    ) -> Result<ResolvedCombatSetupPlan, GameError> {
        use crate::authoring::control::{self, Resource};
        control::charge(1, "combat_setup_entry")?;
        let encounter = game_data
            .pve_data
            .get_by_id(encounter_id)
            .ok_or(GameError::MissingResource("PveEncounter"))?;
        Self::validate_preview_matches_encounter(encounter_id, combat_preview, encounter)?;
        let invalid_reward_configuration = || {
            GameError::InvalidStaticData(format!(
                "combat encounter '{}' has invalid reward configuration for {:?}/{:?}",
                encounter_id, combat_preview.node_type, combat_preview.mission_variant
            ))
        };
        Self::validate_mission_matches_encounter(
            encounter_id,
            encounter,
            combat_preview.node_type,
            combat_preview.mission_variant,
        )
        .map_err(|_| invalid_reward_configuration())?;

        let outcome_policies = resolve_combat_outcome_policies(
            &game_data,
            encounter,
            authored_policy,
            combat_preview.node_type,
            combat_preview.mission_variant,
            reward_multiplier_percent,
        )
        .map_err(|_| invalid_reward_configuration())?;

        info!(
            "Resolving combat setup for primary_abnormality={:?} encounter={}",
            primary_abnormality_id, encounter_id
        );
        let plan = BattleStartPlan::from_resolved_preview(combat_preview)?;
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
        let scenario = Self::build_battle_scenario_for_encounter(
            &game_data,
            encounter,
            combat_preview,
            plan,
            player_start,
            authored_policy,
        )?;
        let mut spawn_count = 0;
        for group in &scenario.groups {
            control::admit(
                Resource::RetainedEntities,
                spawn_count,
                group.spawns.len(),
                "scenario_spawns",
            )?;
            control::charge(group.spawns.len() as u64 + 1, "scenario_group")?;
            spawn_count += group.spawns.len();
        }
        control::charge(
            u64::from(scenario.battlefield.width) * u64::from(scenario.battlefield.height),
            "scenario_board",
        )?;
        control::charge(
            scenario.battlefield.obstacles.len() as u64,
            "scenario_obstacles",
        )?;
        control::poll("battle_constructor")?;
        validate_static_obstacles_do_not_overlap_scenario(&scenario)?;
        scenario.validate()?;

        let mut battle = match relic_runtime_owner {
            Some(owner) => BattleCore::new_for_relic_owner(
                scenario,
                game_data,
                content_identity,
                movement_seed,
                owner,
            )?,
            None => {
                BattleCore::new_from_database(scenario, game_data, content_identity, movement_seed)
            }
        };
        battle.install_relic_numeric_owner(relic_numeric_owner)?;
        if let Some((trace, amount_observation_enabled)) = authoring_observation {
            battle.enable_authoring_observation_with_options(trace, amount_observation_enabled);
        }
        let execution = battle.start_battle_execution_in_candidate()?;
        control::poll("combat_setup_complete")?;
        Ok(ResolvedCombatSetupPlan {
            node_type: combat_preview.node_type,
            mission_variant: combat_preview.mission_variant,
            outcome_policies,
            battle,
            execution,
        })
    }

    #[cfg(test)]
    fn build_battle_with_combat_preview(
        roster: &EmployeeRoster,
        inventory: &Inventory,
        skill_fragments: &SkillFragmentInventory,
        game_data: Arc<GameDataBase>,
        primary_abnormality_id: Option<&str>,
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
            primary_abnormality_id,
            encounter_id,
            movement_seed,
            plan,
            combat_preview,
            deployment_positions,
        )
    }

    #[cfg(test)]
    fn build_battle_with_explicit_deployment(
        roster: &EmployeeRoster,
        inventory: &Inventory,
        skill_fragments: &SkillFragmentInventory,
        game_data: Arc<GameDataBase>,
        primary_abnormality_id: Option<&str>,
        encounter_id: &str,
        movement_seed: u64,
        plan: BattleStartPlan,
        combat_preview: &CombatPreview,
        deployment_positions: &HashMap<Uuid, Position>,
    ) -> Result<BattleCore, GameError> {
        info!(
            "Starting node combat for primary_abnormality={:?} encounter={} field_size={:?}",
            primary_abnormality_id, encounter_id, plan.field_size
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

        Ok(BattleCore::new_from_database(
            scenario,
            game_data,
            None,
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

        Self::validate_mission_matches_encounter(
            encounter_id,
            encounter,
            node_type,
            mission_variant,
        )?;

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
        _inventory: &Inventory,
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
                threat_class: BattleUnitThreatClass::Elite,
                level: fixture.level,
                stat_scale: Default::default(),
                growth_stacks: GrowthStack::new(),
                equipment_layout: None,
                equipped_items: vec![],
                equipped_item_enhancements: vec![],
                employee_start_condition: None,
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
        })
    }

    #[cfg(test)]
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

        Self::build_battle_scenario_for_encounter(
            game_data,
            encounter,
            combat_preview,
            plan,
            player_start,
            None,
        )
    }

    fn build_battle_scenario_for_encounter(
        game_data: &GameDataBase,
        encounter: &crate::game::data::pve_data::PveEncounter,
        combat_preview: &CombatPreview,
        plan: BattleStartPlan,
        player_start: PlayerScenarioStart,
        authored_policy: Option<&AuthoredCombatNodePolicy>,
    ) -> Result<BattleScenario, GameError> {
        let mut groups = Vec::new();
        let mut events = Vec::new();

        push_start_spawn_group(&mut groups, &mut events, player_start.spawn_group);

        let mut tactical_plan =
            CombatMissionPolicy::default_tactical_plan_for_preview(combat_preview);
        encounter.apply_authored_tactical_plan(&mut tactical_plan);
        let terminal_outcomes = encounter.authored_terminal_outcomes();

        if let Some(group) = defense_object_group_for_tactical_plan(&tactical_plan) {
            push_start_spawn_group(&mut groups, &mut events, group);
        }

        let enemy_groups = enemy_spawn_groups_from_preview_with_overrides(
            game_data,
            encounter.id.as_str(),
            combat_preview,
            authored_policy.map_or(&[], |policy| policy.actor_stat_overrides.as_slice()),
        )?;
        let boss_runtime = resolve_boss_runtime_setup_with_preview(
            game_data,
            encounter,
            &enemy_groups,
            combat_preview,
        )?;
        for (group, activation) in enemy_groups {
            push_wave_spawn_group(&mut groups, &mut events, group, activation);
        }

        Ok(BattleScenario {
            battlefield: plan.into_battlefield_spec(),
            groups,
            events,
            terminal_outcomes,
            tactical_plan,
            boss_runtime,
        })
    }

    fn validate_mission_matches_encounter(
        encounter_id: &str,
        encounter: &crate::game::data::pve_data::PveEncounter,
        node_type: CombatNodeType,
        mission_variant: crate::game::combat_preview::CombatMissionVariant,
    ) -> Result<(), GameError> {
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
        Ok(())
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
    use crate::game::battle::core::{movement::types::WorldVec2, BattleCore};
    use crate::game::battle::event_log::BattleLogEvent;
    use crate::game::battle::scenario::{
        EnemyMovementPlan, PlayerMovementPlan, ScenarioAction, ScenarioTrigger, TerminalCondition,
    };
    use crate::game::combat_preview::{BattlefieldArchetype, BattlefieldSizeClass, CombatPreview};
    use crate::game::combat_setup::defense_object::DEFAULT_DEFENSE_OBJECT_GROUP;
    use crate::game::combat_setup::mission_policy::DEFAULT_DEFENSE_OBJECT_REF;
    use crate::game::data::{
        abnormality_data::AbnormalityMetadata,
        pve_data::{
            PveBattleObjectiveData, PveBattlefieldOverrideData, PveEncounter, PveEncounterDatabase,
            PveTacticalPlanData, PveTacticalPointData, PveWaveData, PveWaveEnemyData,
            PveWaveSource,
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

    fn route_progress(cells: &[Position], position: WorldVec2) -> f32 {
        let centers = cells
            .iter()
            .copied()
            .map(WorldVec2::from_tile_center)
            .collect::<Vec<_>>();
        if centers.len() < 2 {
            return 0.0;
        }

        let mut cumulative = 0.0;
        let mut best_distance_sq = f32::MAX;
        let mut best_progress = 0.0;
        for segment in centers.windows(2) {
            let start = segment[0];
            let end = segment[1];
            let delta = end - start;
            let length_sq = delta.length_squared();
            if length_sq <= f32::EPSILON {
                continue;
            }
            let segment_length = length_sq.sqrt();
            let t = dot_world(position - start, delta) / length_sq;
            let t = t.clamp(0.0, 1.0);
            let projected = start + delta * t;
            let distance_sq = position.distance_squared(projected);
            let progress = cumulative + segment_length * t;
            if distance_sq < best_distance_sq - f32::EPSILON
                || ((distance_sq - best_distance_sq).abs() <= f32::EPSILON
                    && progress > best_progress)
            {
                best_distance_sq = distance_sq;
                best_progress = progress;
            }
            cumulative += segment_length;
        }
        best_progress
    }

    fn dot_world(left: WorldVec2, right: WorldVec2) -> f32 {
        left.x * right.x + left.y * right.y
    }

    #[test]
    fn reward_resolution_rejects_authored_node_type_mismatch() {
        let enemy = abnormality("enemy", Uuid::from_u128(0x220), 1);
        let game_data = game_data_with_abnormalities_and_pve(
            vec![enemy],
            vec![PveEncounter {
                id: "boss_contract".to_string(),
                encounter_class: crate::game::data::pve_data::PveEncounterClass::Elite,
                primary_abnormality_id: Some("enemy".to_string()),
                boss_runtime: None,
                risk_level: RiskLevel::ZAYIN,
                outcome_policies: PveEncounter::fixture_outcome_policies(
                    RewardMode::ClaimAll,
                    vec![],
                ),
                node_type: Some(CombatNodeType::Boss),
                mission_variant: None,
                terminal_outcomes: PveEncounter::fixture_terminal_outcomes(None),
                battlefield: None,
                tactical_plan: None,
                waves: vec![PveWaveData {
                    id: "wave_0".to_string(),
                    activation: crate::game::data::pve_data::PveWaveActivationData::at_time(
                        "wave_0", 0,
                    ),
                    spawn_zone_ids: Vec::new(),
                    route_id: Some("defense_main".to_string()),
                    required_for_victory: true,
                    source: PveWaveSource::Manual(vec![PveWaveEnemyData::Abnormality {
                        abnormality_id: "enemy".to_string(),
                        tier: crate::game::enums::Tier::I,
                        count: 1,
                    }]),
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
                encounter_class: crate::game::data::pve_data::PveEncounterClass::Elite,
                primary_abnormality_id: Some("enemy".to_string()),
                boss_runtime: None,
                risk_level: RiskLevel::ZAYIN,
                outcome_policies: PveEncounter::fixture_outcome_policies(
                    RewardMode::ClaimAll,
                    vec![],
                ),
                node_type: None,
                mission_variant: None,
                terminal_outcomes: PveEncounter::fixture_terminal_outcomes(None),
                battlefield: None,
                tactical_plan: None,
                waves: vec![PveWaveData {
                    id: "wave_0".to_string(),
                    activation: crate::game::data::pve_data::PveWaveActivationData::at_time(
                        "wave_0", 0,
                    ),
                    spawn_zone_ids: Vec::new(),
                    route_id: Some("defense_main".to_string()),
                    required_for_victory: true,
                    source: PveWaveSource::Manual(vec![PveWaveEnemyData::Abnormality {
                        abnormality_id: "enemy".to_string(),
                        tier: crate::game::enums::Tier::I,
                        count: 1,
                    }]),
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
        roster.add(Employee::new(employee_uuid, "test_agent", "Agent"));

        let deployment_positions =
            HashMap::from([(employee_uuid, preview.deployment_zones[0].cells[0])]);

        let battle = CombatExecutor::build_battle_with_combat_preview(
            &roster,
            &inventory,
            &skill_fragments,
            game_data,
            Some("enemy"),
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
                encounter_class: crate::game::data::pve_data::PveEncounterClass::Elite,
                primary_abnormality_id: Some("enemy".to_string()),
                boss_runtime: None,
                risk_level: RiskLevel::ZAYIN,
                outcome_policies: PveEncounter::fixture_outcome_policies(
                    RewardMode::ClaimAll,
                    vec![],
                ),
                node_type: None,
                mission_variant: None,
                terminal_outcomes: PveEncounter::fixture_terminal_outcomes(None),
                battlefield: None,
                tactical_plan: None,
                waves: vec![PveWaveData {
                    id: "wave_0".to_string(),
                    activation: crate::game::data::pve_data::PveWaveActivationData::at_time(
                        "wave_0", 0,
                    ),
                    spawn_zone_ids: Vec::new(),
                    route_id: Some("defense_main".to_string()),
                    required_for_victory: true,
                    source: PveWaveSource::Manual(vec![PveWaveEnemyData::Abnormality {
                        abnormality_id: "enemy".to_string(),
                        tier: crate::game::enums::Tier::I,
                        count: 1,
                    }]),
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
        roster.add(Employee::new(employee_uuid, "test_agent", "Agent"));
        let deployment_positions =
            HashMap::from([(employee_uuid, preview.deployment_zones[0].cells[0])]);

        let err = match CombatExecutor::build_battle_with_combat_preview(
            &roster,
            &inventory,
            &skill_fragments,
            game_data,
            Some("enemy"),
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
            encounter_class: crate::game::data::pve_data::PveEncounterClass::Elite,
            primary_abnormality_id: Some("enemy_a".to_string()),
            boss_runtime: None,
            risk_level: RiskLevel::ZAYIN,
            outcome_policies: PveEncounter::fixture_outcome_policies(RewardMode::ClaimAll, vec![]),
            node_type: Some(CombatNodeType::Defense),
            mission_variant: None,
            terminal_outcomes: PveEncounter::fixture_terminal_outcomes(None),
            battlefield: None,
            tactical_plan: None,
            waves: vec![PveWaveData {
                id: "wave_0".to_string(),
                activation: crate::game::data::pve_data::PveWaveActivationData::at_time(
                    "wave_0", 0,
                ),
                spawn_zone_ids: Vec::new(),
                route_id: Some("defense_main".to_string()),
                required_for_victory: true,
                source: PveWaveSource::Manual(vec![PveWaveEnemyData::Abnormality {
                    abnormality_id: "enemy_a".to_string(),
                    tier: crate::game::enums::Tier::I,
                    count: 1,
                }]),
            }],
            static_obstacles: vec![],
        };
        let mut encounter_b = encounter_a.clone();
        encounter_b.id = "encounter_b".to_string();
        encounter_b.primary_abnormality_id = Some("enemy_b".to_string());
        let PveWaveSource::Manual(enemies) = &mut encounter_b.waves[0].source else {
            panic!("test encounter should use a manual wave");
        };
        let PveWaveEnemyData::Abnormality { abnormality_id, .. } = &mut enemies[0] else {
            panic!("test encounter should use an abnormality enemy");
        };
        *abnormality_id = "enemy_b".to_string();
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
        roster.add(Employee::new(employee_uuid, "test_agent", "Agent"));
        let deployment_positions =
            HashMap::from([(employee_uuid, preview.deployment_zones[0].cells[0])]);

        let err = match CombatExecutor::build_battle_with_combat_preview(
            &roster,
            &Inventory::new(),
            &SkillFragmentInventory::new(),
            game_data,
            Some("enemy_b"),
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
                encounter_class: crate::game::data::pve_data::PveEncounterClass::Elite,
                primary_abnormality_id: Some("enemy".to_string()),
                boss_runtime: None,
                risk_level: RiskLevel::ZAYIN,
                outcome_policies: PveEncounter::fixture_outcome_policies(
                    RewardMode::ClaimAll,
                    vec![],
                ),
                node_type: Some(CombatNodeType::Defense),
                mission_variant: None,
                terminal_outcomes: PveEncounter::fixture_terminal_outcomes(None),
                battlefield: Some(PveBattlefieldOverrideData {
                    archetype: Some(crate::game::combat_preview::BattlefieldArchetype::ChokePoint),
                    size_class: Some(crate::game::combat_preview::BattlefieldSizeClass::Small),
                }),
                tactical_plan: None,
                waves: vec![PveWaveData {
                    id: "wave_0".to_string(),
                    activation: crate::game::data::pve_data::PveWaveActivationData::at_time(
                        "wave_0", 0,
                    ),
                    spawn_zone_ids: Vec::new(),
                    route_id: Some("defense_main".to_string()),
                    required_for_victory: true,
                    source: PveWaveSource::Manual(vec![PveWaveEnemyData::Abnormality {
                        abnormality_id: "enemy".to_string(),
                        tier: crate::game::enums::Tier::I,
                        count: 1,
                    }]),
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
        roster.add(Employee::new(employee_uuid, "test_agent", "Agent"));
        let deployment_positions =
            HashMap::from([(employee_uuid, preview.deployment_zones[0].cells[0])]);

        let err = match CombatExecutor::build_battle_with_combat_preview(
            &roster,
            &Inventory::new(),
            &SkillFragmentInventory::new(),
            game_data,
            Some("enemy"),
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
                encounter_class: crate::game::data::pve_data::PveEncounterClass::Elite,
                primary_abnormality_id: Some("enemy".to_string()),
                boss_runtime: None,
                risk_level: RiskLevel::ZAYIN,
                outcome_policies: PveEncounter::fixture_outcome_policies(
                    RewardMode::ClaimAll,
                    vec![],
                ),
                node_type: None,
                mission_variant: None,
                terminal_outcomes: PveEncounter::fixture_terminal_outcomes(None),
                battlefield: None,
                tactical_plan: None,
                waves: vec![PveWaveData {
                    id: "wave_0".to_string(),
                    activation: crate::game::data::pve_data::PveWaveActivationData::at_time(
                        "wave_0", 0,
                    ),
                    spawn_zone_ids: Vec::new(),
                    route_id: Some("defense_main".to_string()),
                    required_for_victory: true,
                    source: PveWaveSource::Manual(vec![PveWaveEnemyData::Abnormality {
                        abnormality_id: "enemy".to_string(),
                        tier: crate::game::enums::Tier::I,
                        count: 1,
                    }]),
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
        let valid_tiles = (0..5)
            .flat_map(|y| (0..5).map(move |x| Position::new(x, y)))
            .collect::<Vec<_>>();
        let tiles = valid_tiles
            .iter()
            .copied()
            .map(|position| crate::game::combat_preview::BattlefieldTile {
                position,
                kind: crate::game::combat_preview::BattlefieldTileKind::Ground,
            })
            .collect::<Vec<_>>();
        let preview = CombatPreview {
            node_id,
            encounter_id: Some("wave_encounter".to_string()),
            battlefield_template_id: "test_wave_field".to_string(),
            node_type: crate::game::combat_preview::CombatNodeType::Defense,
            mission_variant: crate::game::combat_preview::CombatMissionVariant::Defense,
            survive_timer_ms: None,
            mission_risk: crate::game::combat_preview::CombatMissionRisk::Controlled,
            archetype: crate::game::combat_preview::BattlefieldArchetype::Ambush,
            size_class: crate::game::combat_preview::BattlefieldSizeClass::Small,
            width: 5,
            height: 5,
            tiles,
            valid_tiles,
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
                    cells: vec![Position::new(4, 4)],
                    revealed_details: Vec::new(),
                },
            ],
            routes: Vec::new(),
            spawn_waves: vec![
                crate::game::combat_preview::SpawnWave {
                    id: "wave_0".to_string(),
                    activation: crate::game::data::pve_data::PveWaveActivationData::at_time(
                        "wave_0", 0,
                    ),
                    spawn_zone_ids: vec!["entry".to_string()],
                    route_id: None,
                    enemy_entries: vec![wave_entry.clone()],
                    required_for_victory: true,
                },
                crate::game::combat_preview::SpawnWave {
                    id: "wave_1".to_string(),
                    activation: crate::game::data::pve_data::PveWaveActivationData::at_time(
                        "wave_1", 1_000,
                    ),
                    spawn_zone_ids: vec!["reinforcement".to_string()],
                    route_id: None,
                    enemy_entries: vec![wave_entry],
                    required_for_victory: true,
                },
            ],
            obstacles: Vec::new(),
            enemy_briefing: Vec::new(),
            threat_warnings: Vec::new(),
            enemy_stat_scale: Default::default(),
        };

        let inventory = Inventory::new();
        let skill_fragments = SkillFragmentInventory::new();
        let mut roster = EmployeeRoster::new();
        roster.add(Employee::new(employee_uuid, "test_agent", "Agent"));
        let deployment_positions = HashMap::from([(employee_uuid, Position::new(1, 1))]);

        let plan = BattleStartPlan::from_preview(&game_data, "wave_encounter", &preview)
            .expect("preview should build a battle start plan");
        let player_start = player_scenario_start_from_positions(
            &roster,
            &inventory,
            &skill_fragments,
            &game_data,
            deployment_positions
                .iter()
                .map(|(uuid, position)| (*uuid, *position))
                .collect(),
        )
        .expect("player deployment should build scenario start");
        let scenario = CombatExecutor::build_battle_scenario_from_preview(
            &game_data,
            "wave_encounter",
            &preview,
            plan,
            player_start,
        )
        .expect("preview waves should convert into battle scenario waves");

        let timed_spawn_group_id =
            scenario
                .events
                .iter()
                .find_map(|event| match (&event.trigger, &event.action) {
                    (
                        ScenarioTrigger::Wave(activation),
                        ScenarioAction::SpawnGroup { group_id },
                    ) if matches!(
                        activation.primary,
                        crate::game::battle::scenario::ScenarioWavePrimaryTrigger::AtTimeMs {
                            time_ms: 1_000
                        }
                    ) =>
                    {
                        Some(group_id)
                    }
                    _ => None,
                });
        assert!(
            timed_spawn_group_id.is_some(),
            "second preview wave should become a timed scenario spawn event at 1000ms"
        );
        assert!(scenario.groups.iter().any(|group| {
            Some(&group.id) == timed_spawn_group_id && group.side == Side::Opponent
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
                encounter_class: crate::game::data::pve_data::PveEncounterClass::Elite,
                primary_abnormality_id: Some("enemy".to_string()),
                boss_runtime: None,
                risk_level: RiskLevel::ZAYIN,
                outcome_policies: PveEncounter::fixture_outcome_policies(
                    RewardMode::ClaimAll,
                    vec![],
                ),
                node_type: Some(CombatNodeType::Defense),
                mission_variant: None,
                terminal_outcomes: PveEncounter::fixture_protected_terminal_outcomes(
                    "custom_black_box",
                    None,
                ),
                battlefield: Some(PveBattlefieldOverrideData {
                    archetype: Some(crate::game::combat_preview::BattlefieldArchetype::ChokePoint),
                    size_class: Some(crate::game::combat_preview::BattlefieldSizeClass::Small),
                }),
                tactical_plan: Some(PveTacticalPlanData {
                    points: vec![PveTacticalPointData {
                        id: "black_box_anchor".to_string(),
                        position: crate::game::data::pve_data::PvePosition { x: 4, y: 8 },
                    }],
                    objective: Some(PveBattleObjectiveData::ProtectUnit {
                        unit_ref: "custom_black_box".to_string(),
                    }),
                    field_work_objects: Vec::new(),
                }),
                waves: vec![PveWaveData {
                    id: "wave_0".to_string(),
                    activation: crate::game::data::pve_data::PveWaveActivationData::at_time(
                        "wave_0", 0,
                    ),
                    spawn_zone_ids: Vec::new(),
                    route_id: Some("defense_main".to_string()),
                    required_for_victory: true,
                    source: PveWaveSource::Manual(vec![PveWaveEnemyData::Abnormality {
                        abnormality_id: "enemy".to_string(),
                        tier: crate::game::enums::Tier::I,
                        count: 1,
                    }]),
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
        assert!(scenario.terminal_outcomes.iter().any(|outcome| matches!(
            &outcome.condition,
            TerminalCondition::UnitDefeated { unit_ref } if unit_ref.0 == "custom_black_box"
        )));
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
                encounter_class: crate::game::data::pve_data::PveEncounterClass::Elite,
                primary_abnormality_id: Some("enemy".to_string()),
                boss_runtime: None,
                risk_level: RiskLevel::ZAYIN,
                outcome_policies: PveEncounter::fixture_outcome_policies(
                    RewardMode::ClaimAll,
                    vec![],
                ),
                node_type: Some(CombatNodeType::Defense),
                mission_variant: None,
                terminal_outcomes: PveEncounter::fixture_protected_terminal_outcomes(
                    DEFAULT_DEFENSE_OBJECT_REF,
                    None,
                ),
                battlefield: Some(PveBattlefieldOverrideData {
                    archetype: Some(crate::game::combat_preview::BattlefieldArchetype::ChokePoint),
                    size_class: Some(crate::game::combat_preview::BattlefieldSizeClass::Small),
                }),
                tactical_plan: None,
                waves: vec![PveWaveData {
                    id: "wave_0".to_string(),
                    activation: crate::game::data::pve_data::PveWaveActivationData::at_time(
                        "wave_0", 0,
                    ),
                    spawn_zone_ids: Vec::new(),
                    route_id: Some("defense_main".to_string()),
                    required_for_victory: true,
                    source: PveWaveSource::Manual(vec![PveWaveEnemyData::Abnormality {
                        abnormality_id: "enemy".to_string(),
                        tier: crate::game::enums::Tier::I,
                        count: 1,
                    }]),
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
        let preview_route_cells = &preview
            .routes
            .first()
            .expect("defense preview should expose a route")
            .cells;
        assert_eq!(cells, preview_route_cells);
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
        assert_eq!(wave_cells, preview_route_cells);
        assert!(scenario.terminal_outcomes.iter().any(|outcome| matches!(
            &outcome.condition,
            TerminalCondition::UnitDefeated { unit_ref }
                if unit_ref.0 == DEFAULT_DEFENSE_OBJECT_REF
        )));
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
    fn authored_defense_route_enemy_progresses_past_spawn_boundary() {
        let player_owned_uuid = Uuid::from_u128(0x9101);
        let player_base_uuid = Uuid::from_u128(0x9102);
        let enemy_base_uuid = Uuid::from_u128(0x9103);
        let player = abnormality("route_player", player_base_uuid, 1);
        let enemy = abnormality("route_enemy", enemy_base_uuid, 1);
        let game_data = game_data_with_abnormalities_and_pve(
            vec![player, enemy],
            vec![PveEncounter {
                id: "generated_route_encounter".to_string(),
                encounter_class: crate::game::data::pve_data::PveEncounterClass::Elite,
                primary_abnormality_id: Some("route_enemy".to_string()),
                boss_runtime: None,
                risk_level: RiskLevel::ZAYIN,
                outcome_policies: PveEncounter::fixture_outcome_policies(
                    RewardMode::ClaimAll,
                    vec![],
                ),
                node_type: Some(CombatNodeType::Defense),
                mission_variant: None,
                terminal_outcomes: PveEncounter::fixture_terminal_outcomes(None),
                battlefield: Some(PveBattlefieldOverrideData {
                    archetype: Some(BattlefieldArchetype::Corridor),
                    size_class: Some(BattlefieldSizeClass::Medium),
                }),
                tactical_plan: None,
                waves: vec![PveWaveData {
                    id: "wave_0".to_string(),
                    activation: crate::game::data::pve_data::PveWaveActivationData::at_time(
                        "wave_0", 0,
                    ),
                    spawn_zone_ids: Vec::new(),
                    route_id: Some("defense_main".to_string()),
                    required_for_victory: true,
                    source: PveWaveSource::Manual(vec![PveWaveEnemyData::Abnormality {
                        abnormality_id: "route_enemy".to_string(),
                        tier: crate::game::enums::Tier::I,
                        count: 1,
                    }]),
                }],
                static_obstacles: vec![],
            }],
        );
        let preview = CombatPreview::generate_for_node(
            MapNodeId::new(Uuid::from_u128(0x9104)),
            MapNodeCategory::Combat,
            Some("generated_route_encounter"),
            game_data.as_ref(),
            2,
        );
        assert_eq!(preview.battlefield_template_id, "corridor_switchback_01");
        let route_cells = preview
            .routes
            .first()
            .expect("authored route should exist")
            .cells
            .clone();
        assert!(
            route_cells.len() > 2,
            "authored route should contain adjacent waypoints, not only endpoints"
        );

        let plan = BattleStartPlan::from_preview(
            game_data.as_ref(),
            "generated_route_encounter",
            &preview,
        )
        .expect("preview should convert to battle start plan");
        let route_end = preview
            .routes
            .first()
            .expect("authored route should exist")
            .end;
        let player_position = preview.deployment_zones[0]
            .cells
            .iter()
            .copied()
            .find(|cell| *cell != route_end)
            .expect("test deployment zone should include a non-object cell");
        let player_start =
            CombatExecutor::build_test_player_abnormality_scenario_start_from_positions(
                &[TestPlayerAbnormalityUnit {
                    owned_uuid: player_owned_uuid,
                    base_uuid: player_base_uuid,
                    level: crate::game::enums::Tier::I,
                }],
                &Inventory::new(),
                &HashMap::from([(player_owned_uuid, player_position)]),
            )
            .expect("test player scenario start");
        let scenario = CombatExecutor::build_battle_scenario_from_preview(
            game_data.as_ref(),
            "generated_route_encounter",
            &preview,
            plan,
            player_start,
        )
        .expect("battle scenario should build");

        let mut core = BattleCore::new_from_scenario(scenario, game_data, 0x9105);
        let mut execution = core.start_battle_execution().unwrap();
        core.step_battle_execution_until(&mut execution, 2_000)
            .unwrap();

        let enemy_position = core
            .units
            .values()
            .find(|unit| unit.owner == Side::Opponent)
            .expect("enemy should have spawned")
            .world_position();
        let progress = route_progress(&route_cells, enemy_position);
        assert!(
            progress > 1.0,
            "enemy should progress beyond the spawn-row boundary on authored route: position={enemy_position:?}, progress={progress}"
        );
        assert!(core
            .event_log
            .entries
            .iter()
            .any(|entry| matches!(entry.event, BattleLogEvent::MovementSegmentStarted { .. })));
    }

    #[test]
    fn survival_timer_builds_protected_timer_objective() {
        let player_owned_uuid = Uuid::from_u128(0x171);
        let player_base_uuid = Uuid::from_u128(0x272);
        let enemy_base_uuid = Uuid::from_u128(0x273);
        let player = abnormality("player_fixture", player_base_uuid, 37);
        let enemy = abnormality("enemy", enemy_base_uuid, 1);
        let game_data = game_data_with_abnormalities_and_pve(
            vec![player, enemy],
            vec![PveEncounter {
                id: "survival_timer_encounter".to_string(),
                encounter_class: crate::game::data::pve_data::PveEncounterClass::Elite,
                primary_abnormality_id: Some("enemy".to_string()),
                boss_runtime: None,
                risk_level: RiskLevel::ZAYIN,
                outcome_policies: PveEncounter::fixture_outcome_policies(
                    RewardMode::ClaimAll,
                    vec![],
                ),
                node_type: Some(CombatNodeType::Defense),
                mission_variant: None,
                terminal_outcomes: PveEncounter::fixture_protected_terminal_outcomes(
                    DEFAULT_DEFENSE_OBJECT_REF,
                    Some(45_000),
                ),
                battlefield: Some(PveBattlefieldOverrideData {
                    archetype: Some(crate::game::combat_preview::BattlefieldArchetype::Surrounded),
                    size_class: Some(crate::game::combat_preview::BattlefieldSizeClass::Small),
                }),
                tactical_plan: None,
                waves: vec![PveWaveData {
                    id: "wave_0".to_string(),
                    activation: crate::game::data::pve_data::PveWaveActivationData::at_time(
                        "wave_0", 0,
                    ),
                    spawn_zone_ids: Vec::new(),
                    route_id: Some("defense_main".to_string()),
                    required_for_victory: false,
                    source: PveWaveSource::Manual(vec![PveWaveEnemyData::Abnormality {
                        abnormality_id: "enemy".to_string(),
                        tier: crate::game::enums::Tier::I,
                        count: 1,
                    }]),
                }],
                static_obstacles: vec![],
            }],
        );
        let preview = CombatPreview::generate_for_node(
            MapNodeId::new(Uuid::from_u128(0x9008)),
            MapNodeCategory::Combat,
            Some("survival_timer_encounter"),
            game_data.as_ref(),
            0,
        );

        let plan =
            BattleStartPlan::from_preview(game_data.as_ref(), "survival_timer_encounter", &preview)
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
            "survival_timer_encounter",
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
        assert!(scenario.terminal_outcomes.iter().any(|outcome| matches!(
            outcome.condition,
            TerminalCondition::TimeReached { time_ms: 45_000 }
        )));
        assert!(scenario.terminal_outcomes.iter().any(|outcome| matches!(
            &outcome.condition,
            TerminalCondition::UnitDefeated { unit_ref }
                if unit_ref.0 == DEFAULT_DEFENSE_OBJECT_REF
        )));
    }

    #[test]
    fn surrounded_battlefield_remains_default_defense_and_does_not_end_before_deployment() {
        let enemy_base_uuid = Uuid::from_u128(0x283);
        let enemy = abnormality("enemy", enemy_base_uuid, 1);
        let game_data = game_data_with_abnormalities_and_pve(
            vec![enemy],
            vec![PveEncounter {
                id: "surrounded_default_defense".to_string(),
                encounter_class: crate::game::data::pve_data::PveEncounterClass::Elite,
                primary_abnormality_id: Some("enemy".to_string()),
                boss_runtime: None,
                risk_level: RiskLevel::ZAYIN,
                outcome_policies: PveEncounter::fixture_outcome_policies(
                    RewardMode::ClaimAll,
                    vec![],
                ),
                node_type: Some(CombatNodeType::Defense),
                mission_variant: None,
                terminal_outcomes: PveEncounter::fixture_terminal_outcomes(None),
                battlefield: Some(PveBattlefieldOverrideData {
                    archetype: Some(crate::game::combat_preview::BattlefieldArchetype::Surrounded),
                    size_class: Some(crate::game::combat_preview::BattlefieldSizeClass::Medium),
                }),
                tactical_plan: None,
                waves: vec![PveWaveData {
                    id: "wave_0".to_string(),
                    activation: crate::game::data::pve_data::PveWaveActivationData::at_time(
                        "wave_0", 0,
                    ),
                    spawn_zone_ids: Vec::new(),
                    route_id: Some("defense_main".to_string()),
                    required_for_victory: true,
                    source: PveWaveSource::Manual(vec![PveWaveEnemyData::Abnormality {
                        abnormality_id: "enemy".to_string(),
                        tier: crate::game::enums::Tier::I,
                        count: 1,
                    }]),
                }],
                static_obstacles: vec![],
            }],
        );
        let preview = CombatPreview::generate_for_node(
            MapNodeId::new(Uuid::from_u128(0x9009)),
            MapNodeCategory::Combat,
            Some("surrounded_default_defense"),
            game_data.as_ref(),
            4,
        );

        assert_eq!(
            preview.mission_variant,
            crate::game::combat_preview::CombatMissionVariant::Defense
        );
        assert_eq!(preview.survive_timer_ms, None);
        assert_eq!(
            preview.archetype,
            crate::game::combat_preview::BattlefieldArchetype::Surrounded
        );

        let plan = BattleStartPlan::from_preview(
            game_data.as_ref(),
            "surrounded_default_defense",
            &preview,
        )
        .expect("preview should convert to battle start plan");
        let empty_player_start =
            CombatExecutor::build_test_player_abnormality_scenario_start_from_positions(
                &[],
                &Inventory::new(),
                &HashMap::new(),
            )
            .expect("empty deployment player start should build");
        let scenario = CombatExecutor::build_battle_scenario_from_preview(
            game_data.as_ref(),
            "surrounded_default_defense",
            &preview,
            plan,
            empty_player_start,
        )
        .expect("battle scenario should build");

        assert!(scenario.terminal_outcomes.iter().any(|outcome| matches!(
            outcome.condition,
            TerminalCondition::AllRequiredEnemyGroupsDefeated
        )));

        let mut core = BattleCore::new_from_scenario(scenario, game_data, 123);
        let mut execution = core
            .start_battle_execution()
            .expect("battle execution should start");
        let outcome = core
            .step_battle_execution_by(&mut execution, 0)
            .expect("initial update should not fail");

        assert!(matches!(
            outcome,
            crate::game::battle::core::sim::BattleStepOutcome::Running
        ));
        assert!(!core.event_log.entries.iter().any(|entry| {
            matches!(
                entry.event,
                crate::game::battle::event_log::BattleLogEvent::BattleEnd { .. }
            )
        }));
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
                encounter_class: crate::game::data::pve_data::PveEncounterClass::Elite,
                primary_abnormality_id: Some("enemy".to_string()),
                boss_runtime: None,
                risk_level: RiskLevel::ZAYIN,
                outcome_policies: PveEncounter::fixture_outcome_policies(
                    RewardMode::ClaimAll,
                    vec![],
                ),
                node_type: None,
                mission_variant: None,
                terminal_outcomes: PveEncounter::fixture_terminal_outcomes(None),
                battlefield: None,
                tactical_plan: None,
                waves: vec![PveWaveData {
                    id: "wave_0".to_string(),
                    activation: crate::game::data::pve_data::PveWaveActivationData::at_time(
                        "wave_0", 0,
                    ),
                    spawn_zone_ids: Vec::new(),
                    route_id: Some("defense_main".to_string()),
                    required_for_victory: true,
                    source: PveWaveSource::Manual(vec![PveWaveEnemyData::Abnormality {
                        abnormality_id: "enemy".to_string(),
                        tier: crate::game::enums::Tier::I,
                        count: 1,
                    }]),
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

    #[test]
    fn resolved_setup_plan_preserves_boss_class_no_binding_scenario_identity() {
        let enemy_base_uuid = Uuid::from_u128(0xB055_0201);
        let mut boss_fixture = abnormality("boss_fixture", enemy_base_uuid, 3);
        boss_fixture.threat_class = crate::game::battle::types::BattleUnitThreatClass::Boss;
        let game_data = game_data_with_abnormalities_and_pve(
            vec![boss_fixture],
            vec![PveEncounter {
                id: "boss_without_binding".to_string(),
                encounter_class: crate::game::data::pve_data::PveEncounterClass::NormalBoss,
                primary_abnormality_id: Some("boss_fixture".to_string()),
                boss_runtime: None,
                risk_level: RiskLevel::ALEPH,
                outcome_policies: PveEncounter::fixture_outcome_policies(
                    RewardMode::ClaimAll,
                    vec![],
                ),
                node_type: Some(CombatNodeType::Boss),
                mission_variant: Some(crate::game::combat_preview::CombatMissionVariant::Boss),
                terminal_outcomes: PveEncounter::fixture_terminal_outcomes(None),
                battlefield: None,
                tactical_plan: None,
                waves: vec![PveWaveData {
                    id: "wave_0".to_string(),
                    activation: crate::game::data::pve_data::PveWaveActivationData::at_time(
                        "wave_0", 0,
                    ),
                    spawn_zone_ids: Vec::new(),
                    route_id: None,
                    required_for_victory: true,
                    source: PveWaveSource::Manual(vec![PveWaveEnemyData::Abnormality {
                        abnormality_id: "boss_fixture".to_string(),
                        tier: crate::game::enums::Tier::I,
                        count: 1,
                    }]),
                }],
                static_obstacles: vec![],
            }],
        );
        let preview = CombatPreview::generate_for_node(
            MapNodeId::new(Uuid::from_u128(0xB055_0202)),
            MapNodeCategory::Boss,
            Some("boss_without_binding"),
            game_data.as_ref(),
            0xB055_0203,
        );
        let roster = EmployeeRoster::new();
        let inventory = Inventory::new();
        let skill_fragments = SkillFragmentInventory::new();
        let deployment = HashMap::new();

        let direct = CombatExecutor::build_battle_with_combat_preview(
            &roster,
            &inventory,
            &skill_fragments,
            game_data.clone(),
            Some("boss_fixture"),
            "boss_without_binding",
            77,
            &preview,
            &deployment,
        )
        .expect("legacy-compatible builder should resolve the no-binding Boss encounter");
        let resolved = CombatExecutor::resolve_live_combat_setup_plan(
            &roster,
            &inventory,
            &skill_fragments,
            game_data,
            Some("boss_fixture"),
            "boss_without_binding",
            77,
            &preview,
            &deployment,
            100,
        )
        .expect("aggregate setup should resolve the no-binding Boss encounter");

        let signature = |scenario: &BattleScenario| {
            (
                scenario.battlefield.width,
                scenario.battlefield.height,
                scenario
                    .groups
                    .iter()
                    .map(|group| {
                        (
                            group.id.0.clone(),
                            group
                                .spawns
                                .iter()
                                .map(|spawn| spawn.unit_ref.0.clone())
                                .collect::<Vec<_>>(),
                        )
                    })
                    .collect::<Vec<_>>(),
                scenario
                    .events
                    .iter()
                    .map(|event| event.id.0.clone())
                    .collect::<Vec<_>>(),
            )
        };
        assert_eq!(
            signature(direct.scenario()),
            signature(resolved.battle.scenario())
        );
        assert!(direct.scenario().boss_runtime.is_none());
        assert!(resolved.battle.scenario().boss_runtime.is_none());
        assert_eq!(resolved.node_type, CombatNodeType::Boss);
        assert_eq!(
            resolved.mission_variant,
            crate::game::combat_preview::CombatMissionVariant::Boss
        );
        assert_eq!(resolved.outcome_policies.len(), 2);
        let victory = resolved
            .outcome_policies
            .iter()
            .find(|policy| policy.verdict == crate::game::battle::types::BattleVerdict::Victory)
            .expect("resolved Victory policy");
        assert_eq!(victory.reward_mode, RewardMode::ClaimAll);
        assert!(victory.rewards.is_empty());
    }

    #[test]
    fn resolved_setup_plan_carries_deterministic_bound_actor_identity() {
        let enemy_base_uuid = Uuid::from_u128(0xB055_0221);
        let boss_runtime = crate::game::data::boss_runtime_data::tests::valid_catalog()
            .compile()
            .expect("Boss fixture catalog should compile");
        let encounter = PveEncounter {
            id: "bound_tracking_encounter".to_string(),
            encounter_class: crate::game::data::pve_data::PveEncounterClass::Elite,
            primary_abnormality_id: Some("tracking_fixture".to_string()),
            boss_runtime: Some(crate::game::data::pve_data::PveBossRuntimeBindingData {
                encounter_id: crate::game::data::boss_runtime_data::BossEncounterRuntimeId::new(
                    "tracking_encounter",
                ),
                actors: vec![crate::game::data::pve_data::PveBossActorBindingData {
                    actor_slot_id: crate::game::data::boss_runtime_data::BossActorSlotId::new(
                        "primary",
                    ),
                    wave_id: "wave_0".to_string(),
                    spawn_index: 0,
                }],
            }),
            risk_level: RiskLevel::WAW,
            outcome_policies: PveEncounter::fixture_outcome_policies(RewardMode::ClaimAll, vec![]),
            node_type: Some(CombatNodeType::Defense),
            mission_variant: Some(crate::game::combat_preview::CombatMissionVariant::Defense),
            terminal_outcomes: PveEncounter::fixture_terminal_outcomes(None),
            battlefield: None,
            tactical_plan: None,
            waves: vec![PveWaveData {
                id: "wave_0".to_string(),
                activation: crate::game::data::pve_data::PveWaveActivationData::at_time(
                    "wave_0", 0,
                ),
                spawn_zone_ids: Vec::new(),
                route_id: Some("defense_main".to_string()),
                required_for_victory: true,
                source: PveWaveSource::Manual(vec![PveWaveEnemyData::Abnormality {
                    abnormality_id: "tracking_fixture".to_string(),
                    tier: crate::game::enums::Tier::I,
                    count: 1,
                }]),
            }],
            static_obstacles: vec![],
        };
        let game_data = GameDataBuilder::empty()
            .with_abnormalities(vec![abnormality("tracking_fixture", enemy_base_uuid, 3)])
            .with_boss_runtime_data(Arc::new(boss_runtime))
            .with_pve(PveEncounterDatabase::new(vec![encounter]))
            .build_arc();
        let preview = CombatPreview::generate_for_node(
            MapNodeId::new(Uuid::from_u128(0xB055_0222)),
            MapNodeCategory::Combat,
            Some("bound_tracking_encounter"),
            game_data.as_ref(),
            0xB055_0223,
        );
        let resolve = || {
            CombatExecutor::resolve_live_combat_setup_plan(
                &EmployeeRoster::new(),
                &Inventory::new(),
                &SkillFragmentInventory::new(),
                game_data.clone(),
                Some("tracking_fixture"),
                "bound_tracking_encounter",
                91,
                &preview,
                &HashMap::new(),
                100,
            )
            .expect("bound tracking setup should resolve")
        };

        let first = resolve();
        let second = resolve();
        let first_setup = first
            .battle
            .scenario()
            .boss_runtime
            .as_ref()
            .expect("first scenario should carry Boss setup");
        let second_setup = second
            .battle
            .scenario()
            .boss_runtime
            .as_ref()
            .expect("second scenario should carry Boss setup");
        assert_eq!(first_setup.actors, second_setup.actors);
        assert_eq!(first_setup.actors[0].unit_ref.0, "enemy_wave_0_0");
        assert!(Arc::ptr_eq(
            &first_setup.definition,
            &second_setup.definition
        ));
        assert!(first
            .battle
            .scenario()
            .groups
            .iter()
            .flat_map(|group| group.spawns.iter())
            .any(|spawn| spawn.unit_ref == first_setup.actors[0].unit_ref));
    }
}
