use std::collections::HashMap;

use tracing::{info, warn};
use uuid::Uuid;

use super::{
    node_flow::CombatResultNodeCompletion,
    state::{
        ActiveBattleSession, BattleEmployeeAttemptState, BattleEntrySnapshot,
        CombatRecoveryCapability, CombatRecoveryRestriction, LiveBattleDeployedUnitState,
        LiveBattleDeploymentState, LiveBattleRedeployState,
    },
    GameCore,
};
use crate::game::abnormality_research::RunAbnormalityResearchState;
use crate::game::battle::{
    core::sim::{
        BattleDeployCurrentHpPolicy, BattleLiveCommand, BattleLiveCommandOutcome, BattleStepOutcome,
    },
    event_log::BattleEventLog,
    result_stats::{collect_battle_result_stats, BattleResultEmployeeStatsDto},
    tile_range::FacingDirection,
    types::{BattleUnitSource, BattleWinner, ParticipantBattleResult},
};
use crate::game::behavior::{
    BattlePlaybackState, BehaviorResult, CombatOutcomeSummary, DeploymentRangePreviewFacingDto,
    DeploymentRangePreviewFacingsDto, DeploymentRangePreviewResultDto, GameError,
    LiveBattleRangePreviewsDto, NodeOutcomeEmployeeChange, NodeOutcomeSummary,
};
use crate::game::combat_preview::{CombatPreview, DeploymentZoneKind};
use crate::game::combat_setup::player_spawns::battle_unit_draft_for_employee;
use crate::game::combat_setup::rewards::apply_combat_reward_multiplier;
use crate::game::data::run_policy_data::PostBattleResolutionPolicy;
use crate::game::employee::EmployeeRoster;
use crate::game::encounter_compendium::EncounterCompendiumProfileState;
use crate::game::enums::RewardMode;
use crate::game::events::combat::CombatExecutor;
use crate::game::managers::uuid_manager::UuidManager;
use crate::game::map::{
    GameMode, MapNode, MapNodeCategory, MapNodeExecutor, MapNodeId, MapNodePayload, MapProgression,
    RunMap,
};
use crate::game::pve_bonus_objectives::{evaluate_pve_bonus_objectives, satisfied_research_bonus};
use crate::game::range_preview::range_previews_for_combat_profile;
use crate::game::resources::{
    ActiveNodeContent, CombatBattleState, Enkephalin, GameState, Inventory, InventoryDiffDto,
    InventoryItemDto, Position, RunFailureReason,
};
use crate::game::reward::{
    EmployeeExperienceDiffDto, ExperienceTargetPolicy, GrantExecutionContext, GrantExecutor,
    RewardEffect, SkillFragmentGrantDiffDto, SkillFragmentResearchDiffDto,
};
use crate::game::skill_fragment::SkillFragmentInventory;

#[derive(Debug, Clone, Copy)]
struct EmployeeOutcomeBefore {
    physical_damage: u8,
    trauma: u32,
    experience: u32,
    alive: bool,
}

struct StagedCombatResultState {
    inventory: Inventory,
    skill_fragments: SkillFragmentInventory,
    roster: EmployeeRoster,
    uuid_manager: UuidManager,
    enkephalin: Enkephalin,
    condition_inventory_diff: InventoryDiffDto,
}

pub(super) struct PlannedLiveBattleStart {
    combat_preview: CombatPreview,
    active_battle: ActiveBattleSession,
    entry_snapshot: Option<BattleEntrySnapshot>,
}

impl ActiveBattleSession {
    fn reconcile_employee_attempt(
        &mut self,
        post_battle_policy: PostBattleResolutionPolicy,
    ) -> Result<(), GameError> {
        let incapacitations = self.unprocessed_deployed_employee_incapacitations();
        for (employee_uuid, _) in &incapacitations {
            if self.employee_attempt.roster.get(employee_uuid).is_none() {
                return Err(GameError::UnitNotFound);
            }
        }

        for (employee_uuid, unit_instance_id) in incapacitations {
            let employee = self
                .employee_attempt
                .roster
                .get_mut(&employee_uuid)
                .expect("employee incapacitations were prevalidated");
            employee.apply_incapacitation(post_battle_policy.incapacitation_trauma);
            self.employee_attempt
                .processed_incapacitations
                .insert(unit_instance_id);
            info!(
                "Employee {} tactically incapacitated in unit {}: physical_damage={}, trauma={}",
                employee.uuid,
                unit_instance_id,
                employee.condition.physical_damage,
                employee.condition.trauma
            );
        }

        self.reconcile_live_deployment_with_battle_state();
        Ok(())
    }
}

enum PlannedPlayerVictoryCombatResultCompletion {
    RunFailure {
        staged: StagedCombatResultState,
        reason: RunFailureReason,
        encounter_discoveries: EncounterCompendiumProfileState,
    },
    RewardsGranted {
        staged: StagedCombatResultState,
        encounter_discoveries: EncounterCompendiumProfileState,
        research_deliveries: Vec<crate::game::skill_fragment::SkillFragmentResearchDelivery>,
        abnormality_research_after_victory: Option<RunAbnormalityResearchState>,
        node_completion: CombatResultNodeCompletion,
        endless_response_complete: bool,
        enkephalin: u32,
        inventory_diff: InventoryDiffDto,
        skill_fragment_diffs: Vec<SkillFragmentGrantDiffDto>,
        skill_fragment_research_diffs: Vec<SkillFragmentResearchDiffDto>,
        employee_experience_diffs: Vec<EmployeeExperienceDiffDto>,
        outcome: NodeOutcomeSummary,
        recovery_restriction: Option<PlannedCombatRecoveryRestriction>,
    },
}

struct PlannedCombatRecoveryRestriction {
    allowed_node_ids: Vec<MapNodeId>,
    path_exists: bool,
}

#[derive(Default)]
struct MonolithVictoryPlan {
    managed: bool,
    reward_effects: Vec<RewardEffect>,
}

impl GameCore {
    pub(super) fn combat_preview_for_node(
        &mut self,
        node_id: MapNodeId,
    ) -> Result<Option<CombatPreview>, GameError> {
        let (category, encounter_id, existing, floor_index, difficulty) = {
            let run = self.run_state()?;
            let node = run.map.node(node_id).ok_or(GameError::InvalidAction)?;
            let encounter_id = match &node.payload {
                MapNodePayload::Encounter { encounter_id } => encounter_id.as_deref(),
                _ => None,
            };
            (
                node.category,
                encounter_id.map(str::to_string),
                run.combat_previews.get(&node_id).cloned(),
                run.run_progression.floor_index(),
                node.combat_difficulty.clone(),
            )
        };

        // Event nodes can start an authored combat while retaining their Event category.
        // That flow stores the authoritative preview at battle start, so retry must be
        // allowed to reuse it before the manifested Combat/Boss category gate below.
        if let Some(existing) = existing {
            return Ok(Some(existing));
        }
        if !matches!(category, MapNodeCategory::Combat | MapNodeCategory::Boss) {
            return Ok(None);
        }
        let difficulty = difficulty.as_ref().ok_or_else(|| {
            GameError::InvalidStaticData(format!(
                "manifested {:?} node '{}' is missing combat difficulty authority",
                category, node_id.0
            ))
        })?;
        self.validate_live_monolith_difficulty_authority(node_id, Some(difficulty))?;
        let seed = self.node_seed(node_id, 0x5052_4556); // "PREV"
        let preview = CombatPreview::try_generate_for_node_at_floor_with_difficulty(
            node_id,
            category,
            encounter_id.as_deref(),
            self.game_data.as_ref(),
            seed,
            floor_index,
            Some(difficulty),
        )?;
        self.run_state_mut()?
            .combat_previews
            .insert(node_id, preview.clone());
        Ok(Some(preview))
    }

    fn deployment_zone_kind_for_cell(
        combat_preview: &CombatPreview,
        position: Position,
    ) -> Option<DeploymentZoneKind> {
        combat_preview
            .deployment_zones
            .iter()
            .find(|zone| zone.cells.contains(&position))
            .map(|zone| zone.kind)
    }

    fn validate_employee_deployment_cell(
        &self,
        combat_preview: &CombatPreview,
        employee_uuid: Uuid,
        position: Position,
    ) -> Result<(), GameError> {
        let zone_kind = Self::deployment_zone_kind_for_cell(combat_preview, position)
            .ok_or(GameError::OutOfBounds)?;
        let employee = self
            .roster()?
            .get(&employee_uuid)
            .ok_or(GameError::UnitNotFound)?;
        let profile = employee.combat_profile_for_battle(
            &self.game_data.skill_fragment_data,
            &self.state.skill_fragments,
        )?;
        if !zone_kind.supports_affinity(profile.deployment_affinity) {
            return Err(GameError::InvalidAction);
        }
        Ok(())
    }

    pub(super) fn handle_map_combat_node(
        &mut self,
        node_id: MapNodeId,
        primary_abnormality_id: Option<String>,
        encounter_id: String,
        entry_snapshot: BattleEntrySnapshot,
    ) -> Result<BehaviorResult, GameError> {
        let category = self
            .run_state()?
            .map
            .node(node_id)
            .ok_or(GameError::InvalidAction)?
            .category;
        self.handle_explicit_combat_node(
            node_id,
            category,
            primary_abnormality_id,
            encounter_id,
            Some(entry_snapshot),
        )
    }

    pub(super) fn handle_event_combat_node(
        &mut self,
        node_id: MapNodeId,
        primary_abnormality_id: Option<String>,
        encounter_id: String,
    ) -> Result<BehaviorResult, GameError> {
        let enter_result = {
            let run = self.run_state()?;
            let node = run.map.node(node_id).ok_or(GameError::InvalidAction)?;
            MapNodeExecutor::enter(node)
        };
        let entry_snapshot = BattleEntrySnapshot::capture(&self.state).as_event_combat_reentry(
            node_id,
            enter_result.kind_id,
            enter_result.category,
            enter_result.session,
        );
        self.handle_explicit_combat_node(
            node_id,
            MapNodeCategory::Combat,
            primary_abnormality_id,
            encounter_id,
            Some(entry_snapshot),
        )
    }

    fn explicit_combat_preview_for_node(
        &self,
        node_id: MapNodeId,
        category: MapNodeCategory,
        encounter_id: &str,
    ) -> Result<CombatPreview, GameError> {
        if let Some(existing) = self.run_state()?.combat_previews.get(&node_id).cloned() {
            return Ok(existing);
        }
        let run = self.run_state()?;
        let floor_index = run.run_progression.floor_index();
        let node = run.map.node(node_id).ok_or(GameError::InvalidAction)?;
        let difficulty = node.combat_difficulty.as_ref();
        if difficulty.is_none()
            && matches!(
                node.category,
                MapNodeCategory::Combat | MapNodeCategory::Boss
            )
        {
            return Err(GameError::InvalidStaticData(format!(
                "manifested {:?} node '{}' is missing combat difficulty authority",
                node.category, node_id.0
            )));
        }
        let seed = self.node_seed(node_id, 0x5052_4556);
        let preview = CombatPreview::try_generate_for_node_at_floor_with_difficulty(
            node_id,
            category,
            Some(encounter_id),
            self.game_data.as_ref(),
            seed,
            floor_index,
            difficulty,
        )?;
        Ok(preview)
    }

    fn handle_explicit_combat_node(
        &mut self,
        node_id: MapNodeId,
        category: MapNodeCategory,
        primary_abnormality_id: Option<String>,
        encounter_id: String,
        entry_snapshot: Option<BattleEntrySnapshot>,
    ) -> Result<BehaviorResult, GameError> {
        if let Some(result) = self.block_or_fail_undeployable_combat_selection()? {
            return Ok(result);
        }
        let planned = self.plan_explicit_combat_node_with_state(
            node_id,
            category,
            primary_abnormality_id,
            encounter_id,
            &self.state.inventory,
            &self.state.skill_fragments,
            &self.state.roster,
        )?;
        let mut planned = planned;
        planned.entry_snapshot = entry_snapshot;
        self.commit_planned_live_battle_start(node_id, planned)
    }

    pub(super) fn plan_event_combat_node_with_state(
        &self,
        node_id: MapNodeId,
        primary_abnormality_id: Option<String>,
        encounter_id: String,
        inventory: &Inventory,
        skill_fragments: &SkillFragmentInventory,
        roster: &EmployeeRoster,
    ) -> Result<PlannedLiveBattleStart, GameError> {
        self.plan_explicit_combat_node_with_state(
            node_id,
            MapNodeCategory::Combat,
            primary_abnormality_id,
            encounter_id,
            inventory,
            skill_fragments,
            roster,
        )
    }

    fn plan_explicit_combat_node_with_state(
        &self,
        node_id: MapNodeId,
        category: MapNodeCategory,
        primary_abnormality_id: Option<String>,
        encounter_id: String,
        inventory: &Inventory,
        skill_fragments: &SkillFragmentInventory,
        roster: &EmployeeRoster,
    ) -> Result<PlannedLiveBattleStart, GameError> {
        info!(
            "Starting map combat node for primary_abnormality={:?} encounter={} node={}",
            primary_abnormality_id, encounter_id, node_id.0
        );

        let combat_preview =
            self.explicit_combat_preview_for_node(node_id, category, &encounter_id)?;
        let node_type = combat_preview.node_type;
        let mission_variant = combat_preview.mission_variant;
        let (reward_mode, mut rewards) = CombatExecutor::resolve_rewards_for_mission(
            self.game_data.as_ref(),
            &encounter_id,
            node_type,
            mission_variant,
        )
        .map_err(|_| {
            GameError::InvalidStaticData(format!(
                "combat encounter '{}' has invalid reward configuration for {:?}/{:?}",
                encounter_id, node_type, mission_variant
            ))
        })?;
        let reward_multiplier_percent = self.combat_reward_multiplier_for_node(node_id)?;
        apply_combat_reward_multiplier(&mut rewards, reward_multiplier_percent)?;

        if crate::game::combat_setup::mission_policy::CombatMissionPolicy::starts_as_live_battle(
            node_type,
            mission_variant,
        ) {
            if self
                .run_state()?
                .abnormality_attempt_state(node_id)
                .remaining_attempts()
                == 0
            {
                return Err(GameError::InvalidAction);
            }
            let empty_deployment = HashMap::new();
            let mut battle = CombatExecutor::build_battle_with_combat_preview(
                roster,
                inventory,
                skill_fragments,
                self.game_data.clone(),
                primary_abnormality_id.as_deref(),
                &encounter_id,
                self.run_seed,
                &combat_preview,
                &empty_deployment,
            )?;
            let execution = battle.start_battle_execution()?;
            let live_deployment_policy = self.run_policy().live_deployment;
            let active_battle = ActiveBattleSession {
                node_id,
                battle_uuid: Uuid::nil(),
                battle_state_revision: 0,
                primary_abnormality_id,
                encounter_id,
                node_type,
                mission_variant,
                reward_mode,
                rewards,
                combat_preview: combat_preview.clone(),
                battle,
                execution,
                entry_snapshot: None,
                employee_attempt: BattleEmployeeAttemptState::new(roster),
                live_deployment: Some(LiveBattleDeploymentState::new(live_deployment_policy)),
                playback: BattlePlaybackState::default(),
                playback_delta_remainder: 0,
            };
            return Ok(PlannedLiveBattleStart {
                combat_preview,
                active_battle,
                entry_snapshot: None,
            });
        }

        Err(GameError::InvalidStaticData(format!(
            "combat encounter '{}' uses unsupported non-live combat mission {:?}/{:?}; official combat flow is DefenseRoute live battle only",
            encounter_id, node_type, mission_variant
        )))
    }

    fn combat_reward_multiplier_for_node(&self, node_id: MapNodeId) -> Result<u32, GameError> {
        let node = self
            .run_state()?
            .map
            .node(node_id)
            .ok_or(GameError::InvalidAction)?;
        let Some(difficulty) = node.combat_difficulty.as_ref() else {
            if matches!(
                node.category,
                MapNodeCategory::Combat | MapNodeCategory::Boss
            ) {
                return Err(GameError::InvalidStaticData(format!(
                    "manifested {:?} node '{}' is missing combat difficulty authority",
                    node.category, node.id.0
                )));
            }
            // Event-authored combat is not a manifested Combat/Boss node and keeps its
            // authored reward contract unchanged.
            return Ok(100);
        };
        self.validate_live_monolith_difficulty_authority(node_id, Some(difficulty))?;
        self.run_policy()
            .resolved_tuning_for_manifested_node(difficulty)
            .map(|tuning| tuning.reward_multiplier_percent)
            .ok_or_else(|| {
                GameError::InvalidStaticData(format!(
                    "manifested combat difficulty '{}@{}' does not match live reward policy",
                    difficulty.difficulty_profile_id, difficulty.power_model_version
                ))
            })
    }

    pub(super) fn commit_planned_live_battle_start(
        &mut self,
        node_id: MapNodeId,
        mut planned: PlannedLiveBattleStart,
    ) -> Result<BehaviorResult, GameError> {
        let certified = crate::game::combat_power::certified_combat_power_for_roster(
            &self.state.roster,
            &self.state.inventory,
            &self.state.skill_fragments,
            self.game_data.as_ref(),
        )?;
        let entry_snapshot = if let Some(snapshot) = planned.entry_snapshot.take() {
            snapshot
        } else if self
            .state
            .node_session
            .as_ref()
            .is_some_and(|session| session.category == MapNodeCategory::Event)
        {
            let enter_result = {
                let run = self.run_state()?;
                let node = run.map.node(node_id).ok_or(GameError::InvalidAction)?;
                MapNodeExecutor::enter(node)
            };
            BattleEntrySnapshot::capture(&self.state).as_event_combat_reentry(
                node_id,
                enter_result.kind_id,
                enter_result.category,
                enter_result.session,
            )
        } else {
            BattleEntrySnapshot::capture(&self.state)
        };
        if self
            .run_state_mut()?
            .start_abnormality_attempt(node_id)
            .is_none()
        {
            return Err(GameError::InvalidAction);
        }
        self.state
            .run_power_authority
            .record_combat_entry(certified.power_model_version, certified.total);
        self.run_state_mut()?
            .combat_previews
            .insert(node_id, planned.combat_preview);

        planned.active_battle.battle_uuid = self.state.uuid_manager.next_battle_session();
        planned.active_battle.entry_snapshot = Some(entry_snapshot);
        let battle_uuid = planned.active_battle.battle_uuid;
        self.state.active_battle = Some(planned.active_battle);
        self.transition_to(GameState::InBattle { battle_uuid })?;
        self.advance_active_battle_by_internal(0, true, 0)
    }

    fn is_final_boss_node(&self, node_id: MapNodeId) -> Result<bool, GameError> {
        let run = self.run_state()?;
        let is_standard_final_boss = run.run_progression.is_final_standard_floor()
            && run.map.terminal_resolution(node_id)
                == Some(crate::game::map::FacilityTerminalResolution::RunFinalBoss)
            && run
                .map
                .node(node_id)
                .is_some_and(|node| node.category == MapNodeCategory::Boss);
        Ok(is_standard_final_boss || run.boss_omen.forced_boss_node_id() == Some(node_id))
    }

    fn stage_committed_combat_resolution(
        &self,
        battle: &CombatBattleState,
    ) -> Result<StagedCombatResultState, GameError> {
        let mut staged = self.staged_combat_result_state();
        self.apply_post_battle_resolution_to_state(&mut staged, battle)?;
        Self::decrement_consumables_after_combat_node_in_roster(&mut staged.roster);
        Ok(staged)
    }

    fn staged_combat_result_state(&self) -> StagedCombatResultState {
        StagedCombatResultState {
            inventory: self.state.inventory.clone(),
            skill_fragments: self.state.skill_fragments.clone(),
            roster: self.state.roster.clone(),
            uuid_manager: self.state.uuid_manager.clone(),
            enkephalin: self.state.enkephalin.clone(),
            condition_inventory_diff: InventoryDiffDto::default(),
        }
    }

    fn commit_staged_combat_result_state(&mut self, staged: StagedCombatResultState) {
        self.state.inventory = staged.inventory;
        self.state.skill_fragments = staged.skill_fragments;
        self.state.roster = staged.roster;
        self.state.uuid_manager = staged.uuid_manager;
        self.state.enkephalin = staged.enkephalin;
    }

    fn validated_encounter_discoveries(
        &self,
        battle: &CombatBattleState,
    ) -> Result<EncounterCompendiumProfileState, GameError> {
        let discoveries = EncounterCompendiumProfileState::from_consumed_battle_result(
            &battle.event_log,
            battle.winner == BattleWinner::Player,
            battle.primary_abnormality_id.as_deref(),
        );
        discoveries
            .validate_current_catalog_commit(self.game_data.as_ref())
            .map_err(GameError::InvalidStaticData)?;
        Ok(discoveries)
    }

    fn commit_encounter_discoveries(&mut self, discoveries: EncounterCompendiumProfileState) {
        self.state.encounter_compendium.merge(discoveries);
    }

    fn apply_post_battle_resolution_to_state(
        &self,
        staged: &mut StagedCombatResultState,
        battle: &CombatBattleState,
    ) -> Result<(), GameError> {
        let post_battle_survival_xp = self.run_policy().growth.post_battle_survival_xp;
        staged.roster = battle.employee_conditions.clone();
        staged.condition_inventory_diff =
            self.recover_assets_for_new_deaths(&mut staged.roster, &mut staged.inventory)?;
        if staged.roster.is_empty() {
            return Ok(());
        }

        let survival_xp_employee_ids = battle
            .result_stats
            .employees
            .iter()
            .filter(|result| {
                result.survived
                    && staged
                        .roster
                        .get(&result.employee_uuid)
                        .is_some_and(|employee| employee.is_alive())
            })
            .map(|result| result.employee_uuid)
            .collect::<Vec<_>>();

        if post_battle_survival_xp > 0 {
            for employee_uuid in survival_xp_employee_ids {
                GrantExecutor::grant_effects_with_state(
                    &mut staged.inventory,
                    &mut staged.skill_fragments,
                    &mut staged.roster,
                    &mut staged.uuid_manager,
                    &mut staged.enkephalin,
                    &self.game_data,
                    &self.state.skill_fragment_policy,
                    &GrantExecutionContext {
                        selected_employee_id: Some(employee_uuid),
                        ..GrantExecutionContext::default()
                    },
                    &[RewardEffect::GrantExperience {
                        amount: post_battle_survival_xp,
                        target: ExperienceTargetPolicy::SelectedEmployee,
                    }],
                    self.run_seed ^ 0x4752_414e_5446_5853,
                )?;
            }
        }
        Ok(())
    }

    pub(super) fn recover_assets_for_new_deaths(
        &self,
        roster: &mut EmployeeRoster,
        inventory: &mut Inventory,
    ) -> Result<InventoryDiffDto, GameError> {
        let newly_dead_ids = roster
            .iter()
            .filter(|employee| {
                employee.condition.is_dead()
                    && self
                        .state
                        .roster
                        .get(&employee.uuid)
                        .is_some_and(|before| before.is_alive())
            })
            .map(|employee| employee.uuid)
            .collect::<Vec<_>>();
        let mut diff = InventoryDiffDto::default();
        for employee_uuid in newly_dead_ids {
            let employee = roster
                .get_mut(&employee_uuid)
                .ok_or(GameError::UnitNotFound)?;
            let equipped = employee.loadout.item_slot.unequip_all();
            employee.skill_fragments.unequip_shared_fragment();
            employee.active_consumable_modifier = None;
            for equipped_ref in equipped {
                let owned = inventory
                    .equipments
                    .get_item_mut(&equipped_ref.instance_uuid)
                    .ok_or(GameError::InventoryItemNotFound)?;
                if owned.equipped_to != Some(employee_uuid) {
                    return Err(GameError::InvalidAction);
                }
                owned.equipped_to = None;
                diff.updated.push(InventoryItemDto::Equipment(
                    crate::game::resources::EquipmentItemDto::from_owned_equipment(owned),
                ));
            }
        }
        Ok(diff)
    }

    fn decrement_consumables_after_combat_node_in_roster(roster: &mut EmployeeRoster) {
        for employee in roster.iter_mut() {
            employee.decrement_consumable_after_combat_node();
        }
    }

    fn employee_outcome_before(
        &self,
        employee_results: &[BattleResultEmployeeStatsDto],
    ) -> Result<HashMap<Uuid, EmployeeOutcomeBefore>, GameError> {
        let roster = &self.state.roster;
        let mut before = HashMap::new();
        for result in employee_results {
            if let Some(employee) = roster.get(&result.employee_uuid) {
                before.insert(
                    result.employee_uuid,
                    EmployeeOutcomeBefore {
                        physical_damage: employee.condition.physical_damage,
                        trauma: employee.condition.trauma,
                        experience: employee.experience,
                        alive: employee.is_alive(),
                    },
                );
            }
        }
        Ok(before)
    }

    fn employee_outcome_changes_from_roster(
        roster: &EmployeeRoster,
        before: HashMap<Uuid, EmployeeOutcomeBefore>,
        employee_results: &[BattleResultEmployeeStatsDto],
    ) -> Result<Vec<NodeOutcomeEmployeeChange>, GameError> {
        let mut changes = Vec::new();
        for result in employee_results {
            let Some(before) = before.get(&result.employee_uuid) else {
                continue;
            };
            let Some(employee) = roster.get(&result.employee_uuid) else {
                continue;
            };
            changes.push(NodeOutcomeEmployeeChange {
                employee_uuid: result.employee_uuid,
                survived: result.survived,
                became_incapacitated: result.became_incapacitated,
                physical_damage_before: before.physical_damage,
                physical_damage_after: employee.condition.physical_damage,
                trauma_before: before.trauma,
                trauma_after: employee.condition.trauma,
                experience_before: before.experience,
                experience_after: employee.experience,
                was_alive: before.alive,
                is_alive: employee.is_alive(),
            });
        }
        Ok(changes)
    }

    fn combat_experience_context_from_roster(
        roster: &EmployeeRoster,
        employee_results: &[BattleResultEmployeeStatsDto],
    ) -> GrantExecutionContext {
        let eligible_employee_ids = employee_results
            .iter()
            .filter(|result| {
                result.survived
                    && roster
                        .get(&result.employee_uuid)
                        .is_some_and(|employee| employee.is_alive())
            })
            .map(|result| result.employee_uuid)
            .collect::<Vec<_>>();

        GrantExecutionContext {
            combat_participant_employee_ids: Some(eligible_employee_ids),
            selected_employee_id: None,
        }
    }

    fn combat_node_outcome_summary(
        &self,
        battle: &CombatBattleState,
        mission_success: bool,
        employee_changes: Vec<NodeOutcomeEmployeeChange>,
        inventory_diff: InventoryDiffDto,
    ) -> Result<NodeOutcomeSummary, GameError> {
        self.combat_node_outcome_summary_with_resolution(
            battle,
            mission_success,
            battle.winner,
            false,
            employee_changes,
            inventory_diff,
        )
    }

    fn combat_node_outcome_summary_with_resolution(
        &self,
        battle: &CombatBattleState,
        mission_success: bool,
        winner: BattleWinner,
        retreated: bool,
        employee_changes: Vec<NodeOutcomeEmployeeChange>,
        inventory_diff: InventoryDiffDto,
    ) -> Result<NodeOutcomeSummary, GameError> {
        let session = self
            .state
            .node_session
            .as_ref()
            .ok_or(GameError::InvalidAction)?;
        Ok(NodeOutcomeSummary {
            node_id: session.node_id,
            kind_id: session.kind_id.clone(),
            category: session.category,
            mission_success,
            combat: Some(CombatOutcomeSummary {
                node_type: battle.node_type,
                mission_variant: battle.mission_variant,
                winner,
                retreated,
            }),
            employee_changes,
            inventory_diff,
        })
    }

    fn current_run_failure_reason(&self) -> Result<Option<RunFailureReason>, GameError> {
        let roster = self.roster()?;
        Ok(Self::current_run_failure_reason_for_roster(roster))
    }

    fn current_run_failure_reason_for_roster(roster: &EmployeeRoster) -> Option<RunFailureReason> {
        if roster.is_empty() {
            return None;
        }

        if !roster.iter().any(|employee| employee.is_alive()) {
            return Some(RunFailureReason::NoLivingEmployees);
        }

        if roster.combat_eligible_employee_ids().is_empty() {
            return Some(RunFailureReason::NoDeployableEmployees);
        }

        None
    }

    pub(super) fn can_complete_combat_result_locally(&self) -> bool {
        let Some(session) = self.state.node_session.as_ref() else {
            return false;
        };
        if !matches!(
            session.category,
            MapNodeCategory::Combat | MapNodeCategory::Boss | MapNodeCategory::Event
        ) {
            return false;
        }
        let Some(content) = self.state.active_node_content.as_ref() else {
            return false;
        };
        let Ok(battle) = content.as_combat_battle() else {
            return false;
        };
        if battle.abnormality_uuid != session.node_id.0 {
            return false;
        }
        if battle.winner == BattleWinner::Player && battle.reward_mode != RewardMode::ClaimAll {
            return false;
        }
        let Ok(run) = self.run_state() else {
            return false;
        };
        if run.map_progression.current_node_id != Some(session.node_id) {
            return false;
        }
        if run.map.node(session.node_id).is_none() {
            return false;
        }

        let mut map = run.map.clone();
        let mut progression = run.map_progression.clone();
        let Ok(template) = self.facility_template_for_map(&map) else {
            return false;
        };
        progression
            .complete_current_node(&mut map, &template)
            .is_ok()
    }

    fn combat_recovery_capability_for_node(
        &self,
        node: &MapNode,
    ) -> Option<CombatRecoveryCapability> {
        match (&node.category, &node.payload) {
            (
                MapNodeCategory::HeadquartersContact,
                MapNodePayload::HeadquartersContact {
                    candidate_count, ..
                },
            ) if *candidate_count > 0
                && self
                    .state
                    .run
                    .as_ref()
                    .is_some_and(|run| run.recruitment.has_queued_candidates()) =>
            {
                Some(CombatRecoveryCapability::Recruitment)
            }
            _ => None,
        }
    }

    fn immediately_available_treatment_amount(inventory: &Inventory) -> u32 {
        inventory
            .consumables
            .iter()
            .filter_map(|stack| match stack.meta.effect {
                crate::game::data::consumable_data::ConsumableEffect::ReducePhysicalDamage {
                    amount,
                } => Some(u32::from(amount).saturating_mul(stack.quantity)),
                _ => None,
            })
            .fold(0u32, u32::saturating_add)
    }

    fn roster_can_recover_deployability(
        &self,
        roster: &EmployeeRoster,
        inventory: &Inventory,
        recovery: Option<crate::game::data::run_policy_data::TraumaRecovery>,
    ) -> bool {
        let treatment_amount = Self::immediately_available_treatment_amount(inventory);
        roster.iter().any(|employee| {
            if !employee.is_alive() {
                return false;
            }
            let physical_damage = u32::from(employee.condition.physical_damage)
                .saturating_sub(treatment_amount)
                .min(u32::from(u8::MAX)) as u8;
            let trauma = recovery
                .map(|policy| policy.apply(employee.condition.trauma))
                .unwrap_or(employee.condition.trauma);
            crate::game::employee::EmployeeCondition::new(physical_damage, trauma)
                .eligibility(crate::game::employee::EmployeeActivity::Combat)
                .eligible
        })
    }

    fn recovery_node_ids_and_path_exists(
        &self,
        map: &RunMap,
        progression: &MapProgression,
        roster: &EmployeeRoster,
        inventory: &Inventory,
    ) -> (Vec<MapNodeId>, bool) {
        let Ok(template) = self.facility_template_for_map(map) else {
            return (Vec::new(), false);
        };
        let selectable = progression.selectable_node_ids(map, &template);
        let has_treatment_path = self.roster_can_recover_deployability(roster, inventory, None);
        let mut has_node_path = false;
        let mut allowed_node_ids = Vec::new();
        for node_id in selectable {
            let Some(node) = map.node(node_id) else {
                continue;
            };
            match (&node.category, &node.payload) {
                (MapNodeCategory::SavePoint, MapNodePayload::SavePoint) => {
                    allowed_node_ids.push(node_id);
                    has_node_path |= self.roster_can_recover_deployability(
                        roster,
                        inventory,
                        Some(self.run_policy().save_point_trauma_recovery),
                    );
                }
                (MapNodeCategory::Gate, MapNodePayload::None) => {
                    allowed_node_ids.push(node_id);
                    has_node_path |= self.roster_can_recover_deployability(
                        roster,
                        inventory,
                        Some(self.run_policy().gate_trauma_recovery),
                    );
                }
                _ if self.combat_recovery_capability_for_node(node).is_some() => {
                    allowed_node_ids.push(node_id);
                    has_node_path = true;
                }
                _ => {}
            }
        }
        (allowed_node_ids, has_treatment_path || has_node_path)
    }

    fn recovery_path_for_completion(
        &self,
        completion: &CombatResultNodeCompletion,
        roster: &EmployeeRoster,
        inventory: &Inventory,
    ) -> (Vec<MapNodeId>, bool) {
        match completion {
            CombatResultNodeCompletion::NodeCompleted {
                map, progression, ..
            }
            | CombatResultNodeCompletion::RunComplete {
                map, progression, ..
            } => self.recovery_node_ids_and_path_exists(map, progression, roster, inventory),
            CombatResultNodeCompletion::FloorAdvanced {
                next_map,
                next_progression,
                ..
            } => self.recovery_node_ids_and_path_exists(
                next_map,
                next_progression,
                roster,
                inventory,
            ),
        }
    }

    fn current_forward_recovery_path(&self) -> Result<(Vec<MapNodeId>, bool), GameError> {
        let run = self.run_state()?;
        Ok(self.recovery_node_ids_and_path_exists(
            &run.map,
            &run.map_progression,
            self.roster()?,
            self.inventory()?,
        ))
    }

    pub(super) fn selectable_map_node_ids(&self) -> Vec<MapNodeId> {
        let Some(run) = self.state.run.as_ref() else {
            return Vec::new();
        };
        let Ok(template) = self.facility_template_for_map(&run.map) else {
            return Vec::new();
        };
        let selectable = run.map_progression.selectable_node_ids(&run.map, &template);
        let Some(restriction) = run.combat_recovery_restriction.as_ref() else {
            return selectable;
        };
        selectable
            .into_iter()
            .filter(|node_id| restriction.allowed_node_ids.contains(node_id))
            .collect()
    }

    pub(super) fn has_combat_recovery_restriction(&self) -> bool {
        self.state
            .run
            .as_ref()
            .is_some_and(|run| run.combat_recovery_restriction.is_some())
    }

    fn set_combat_recovery_restriction(&mut self, mut allowed_node_ids: Vec<MapNodeId>) {
        allowed_node_ids.sort_by_key(|node_id| node_id.0.as_u128());
        allowed_node_ids.dedup();
        if let Some(run) = self.state.run.as_mut() {
            run.combat_recovery_restriction = Some(CombatRecoveryRestriction { allowed_node_ids });
        }
    }

    pub(super) fn finalize_combat_recovery_node_resolution(
        &mut self,
        completion: BehaviorResult,
    ) -> Result<BehaviorResult, GameError> {
        if !self.has_combat_recovery_restriction() {
            return Ok(completion);
        }
        if let Some(run) = self.state.run.as_mut() {
            run.combat_recovery_restriction = None;
        }
        match self.current_run_failure_reason()? {
            Some(reason @ RunFailureReason::NoLivingEmployees) => self.fail_run(reason),
            Some(reason @ RunFailureReason::NoDeployableEmployees) => {
                let (allowed_node_ids, path_exists) = self.current_forward_recovery_path()?;
                if path_exists {
                    self.set_combat_recovery_restriction(allowed_node_ids);
                    Ok(completion)
                } else {
                    self.fail_run(reason)
                }
            }
            Some(RunFailureReason::BossDefeated) | None => Ok(completion),
        }
    }

    pub(super) fn block_or_fail_undeployable_combat_selection(
        &mut self,
    ) -> Result<Option<BehaviorResult>, GameError> {
        let reason = self.current_run_failure_reason()?;
        self.block_or_fail_undeployable_combat_selection_for_reason(reason)
    }

    fn block_or_fail_undeployable_combat_selection_for_reason(
        &mut self,
        reason: Option<RunFailureReason>,
    ) -> Result<Option<BehaviorResult>, GameError> {
        match reason {
            Some(RunFailureReason::NoLivingEmployees) => {
                self.fail_run(RunFailureReason::NoLivingEmployees).map(Some)
            }
            Some(RunFailureReason::NoDeployableEmployees) => {
                let (allowed_node_ids, path_exists) = self.current_forward_recovery_path()?;
                if path_exists {
                    self.set_combat_recovery_restriction(allowed_node_ids);
                    self.handle_request_map_data().map(Some)
                } else {
                    self.fail_run(RunFailureReason::NoDeployableEmployees)
                        .map(Some)
                }
            }
            Some(RunFailureReason::BossDefeated) => {
                self.fail_run(RunFailureReason::BossDefeated).map(Some)
            }
            None => Ok(None),
        }
    }

    fn fail_run(&mut self, reason: RunFailureReason) -> Result<BehaviorResult, GameError> {
        self.fail_run_with_outcome(reason, None)
    }

    fn fail_run_with_outcome(
        &mut self,
        reason: RunFailureReason,
        outcome: Option<NodeOutcomeSummary>,
    ) -> Result<BehaviorResult, GameError> {
        self.fail_run_with_reasons(reason, None, outcome)
    }

    fn fail_run_with_reasons(
        &mut self,
        reason: RunFailureReason,
        contributing_reason: Option<RunFailureReason>,
        outcome: Option<NodeOutcomeSummary>,
    ) -> Result<BehaviorResult, GameError> {
        self.state.node_session = None;
        self.state.active_node_content = None;
        self.state.active_battle = None;
        if self.state.run_checkpoint.payload.is_some()
            && self.state.run_checkpoint.loads_used
                < crate::game::world::state::RUN_CHECKPOINT_MAX_LOADS
        {
            self.transition_to(GameState::AwaitingRunFailureDecision {
                reason,
                contributing_reason,
            })?;
            return Ok(BehaviorResult::RunFailureDecisionRequired {
                reason,
                contributing_reason,
                outcome,
            });
        }
        self.commit_run_failure(reason, outcome)
    }

    fn commit_run_failure(
        &mut self,
        reason: RunFailureReason,
        outcome: Option<NodeOutcomeSummary>,
    ) -> Result<BehaviorResult, GameError> {
        self.state.node_session = None;
        self.state.active_node_content = None;
        self.state.active_battle = None;
        self.transition_to(GameState::RunFailed { reason })?;
        Ok(BehaviorResult::RunFailed { reason, outcome })
    }

    pub(super) fn handle_confirm_run_failure(&mut self) -> Result<BehaviorResult, GameError> {
        let GameState::AwaitingRunFailureDecision { reason, .. } = self.state.game_state else {
            return Err(GameError::InvalidAction);
        };
        self.commit_run_failure(reason, None)
    }

    fn fail_run_if_needed(&mut self) -> Result<Option<BehaviorResult>, GameError> {
        match self.current_run_failure_reason()? {
            Some(RunFailureReason::NoLivingEmployees) => {
                self.fail_run(RunFailureReason::NoLivingEmployees).map(Some)
            }
            Some(RunFailureReason::NoDeployableEmployees) => self
                .fail_run(RunFailureReason::NoDeployableEmployees)
                .map(Some),
            Some(RunFailureReason::BossDefeated) => {
                self.fail_run(RunFailureReason::BossDefeated).map(Some)
            }
            None => Ok(None),
        }
    }

    fn active_battle_advanced_result(
        active: &mut ActiveBattleSession,
        post_battle_policy: PostBattleResolutionPolicy,
        finished: bool,
        include_setup_snapshot: bool,
        after_seq: u64,
    ) -> Result<BehaviorResult, GameError> {
        active.refresh_live_deployment_cost();
        active.apply_live_signals_to_deployment();
        active.reconcile_employee_attempt(post_battle_policy)?;
        active.commit_battle_projection()?;
        let battle_setup_snapshot =
            include_setup_snapshot.then(|| active.battle_setup_snapshot_dto());
        let battle_update = active.battle_update_dto_after(after_seq, active.employee_roster())?;
        Ok(BehaviorResult::BattleAdvanced {
            battle_uuid: active.battle_uuid,
            encounter_id: active.encounter_id.clone(),
            node_type: active.node_type,
            mission_variant: active.mission_variant,
            battle_setup_snapshot,
            battle_update,
            finished,
        })
    }

    fn combat_battle_state_from_live_result(
        active: &ActiveBattleSession,
        winner: BattleWinner,
        event_log: BattleEventLog,
        participant_results: Vec<ParticipantBattleResult>,
        result_stats_policy: &crate::game::data::run_policy_data::BattleResultStatsPolicy,
        game_data: &crate::game::data::GameDataBase,
    ) -> CombatBattleState {
        let result_stats = collect_battle_result_stats(
            winner,
            &event_log,
            &participant_results,
            result_stats_policy,
        );
        let bonus_objectives = game_data
            .pve_data
            .get_by_id(&active.encounter_id)
            .map(|encounter| {
                evaluate_pve_bonus_objectives(encounter, winner, &event_log, &result_stats)
            })
            .unwrap_or_default();
        CombatBattleState {
            primary_abnormality_id: active.primary_abnormality_id.clone(),
            encounter_id: active.encounter_id.clone(),
            node_type: active.node_type,
            mission_variant: active.mission_variant,
            abnormality_uuid: active.node_id.0,
            winner,
            event_log,
            result_stats,
            bonus_objectives,
            reward_mode: active.reward_mode,
            rewards: active.rewards.clone(),
            participant_results,
            employee_conditions: active.employee_attempt.roster.clone(),
        }
    }

    pub(super) fn transition_finished_live_battle_to_result(
        &mut self,
        battle_uuid: Uuid,
        battle: CombatBattleState,
    ) -> Result<(), GameError> {
        if let Err(error) =
            super::combat_trace::export_debug_combat_trace(self.run_seed, battle_uuid, &battle)
        {
            warn!("Ignoring non-authoritative debug combat trace export failure: {error:?}");
        }
        self.state.active_node_content = Some(ActiveNodeContent::CombatBattle(battle));
        self.state.active_battle = None;
        self.transition_to(GameState::CombatResult { battle_uuid })
    }

    pub fn advance_active_battle_by(&mut self, delta_ms: u64) -> Result<BehaviorResult, GameError> {
        self.advance_active_battle_by_after_seq(delta_ms, 0)
    }

    pub fn advance_active_battle_by_after_seq(
        &mut self,
        delta_ms: u64,
        after_seq: u64,
    ) -> Result<BehaviorResult, GameError> {
        self.advance_active_battle_by_internal(delta_ms, false, after_seq)
    }

    fn advance_active_battle_by_internal(
        &mut self,
        delta_ms: u64,
        include_setup_snapshot: bool,
        after_seq: u64,
    ) -> Result<BehaviorResult, GameError> {
        let result_stats_policy = self.run_policy().battle_result_stats.clone();
        let post_battle_policy = self.run_policy().post_battle;
        let game_data = self.game_data.clone();
        let (mut completed_battle, trigger_context) = {
            let active = self
                .state
                .active_battle
                .as_mut()
                .ok_or(GameError::InvalidAction)?;
            active.ensure_battle_projection_commit_available()?;
            let previous_battle_time_ms = active.execution.last_event_time_ms();
            let outcome = active
                .battle
                .step_battle_execution_by(&mut active.execution, delta_ms)?;
            let completed_battle = match outcome {
                BattleStepOutcome::Running => None,
                BattleStepOutcome::Finished(finish) => {
                    let battle_result = active
                        .battle
                        .finalize_battle_execution(&mut active.execution, finish)?;
                    Some(Self::combat_battle_state_from_live_result(
                        active,
                        battle_result.winner,
                        battle_result.event_log,
                        battle_result.participant_results,
                        &result_stats_policy,
                        game_data.as_ref(),
                    ))
                }
            };
            let current_battle_time_ms = active.execution.last_event_time_ms();
            let trigger_context = (
                active.battle_uuid,
                active.encounter_id.clone(),
                active.node_type,
                active.mission_variant,
                previous_battle_time_ms,
                current_battle_time_ms,
            );
            (completed_battle, trigger_context)
        };
        let (
            battle_uuid,
            encounter_id,
            node_type,
            mission_variant,
            previous_battle_time_ms,
            current_battle_time_ms,
        ) = trigger_context;
        if include_setup_snapshot {
            self.evaluate_visual_novel_triggers(
                super::visual_novel_triggers::VisualNovelTriggerEvent::BattleStarted {
                    battle_uuid,
                    encounter_id: encounter_id.clone(),
                    node_type,
                    mission_variant,
                },
            )?;
        }
        self.evaluate_visual_novel_triggers(
            super::visual_novel_triggers::VisualNovelTriggerEvent::BattleTimeReached {
                battle_uuid,
                previous_battle_time_ms,
                current_battle_time_ms,
            },
        )?;
        let mut result = {
            let active = self
                .state
                .active_battle
                .as_mut()
                .ok_or(GameError::InvalidAction)?;
            Self::active_battle_advanced_result(
                active,
                post_battle_policy,
                completed_battle.is_some(),
                include_setup_snapshot,
                after_seq,
            )?
        };
        if let Some(battle) = completed_battle.as_mut() {
            battle.employee_conditions = self
                .state
                .active_battle
                .as_ref()
                .ok_or(GameError::InvalidAction)?
                .employee_attempt
                .roster
                .clone();
        }
        self.attach_visual_novel_to_battle_result(&mut result)?;
        if let Some(battle) = completed_battle {
            self.transition_finished_live_battle_to_result(battle_uuid, battle)?;
        }
        Ok(result)
    }

    pub fn advance_active_battle_for_server_tick(
        &mut self,
        raw_delta_ms: u64,
    ) -> Result<Option<BehaviorResult>, GameError> {
        self.advance_active_battle_for_server_tick_after_seq(raw_delta_ms, 0)
    }

    pub fn advance_active_battle_for_server_tick_after_seq(
        &mut self,
        raw_delta_ms: u64,
        after_seq: u64,
    ) -> Result<Option<BehaviorResult>, GameError> {
        if self
            .state
            .visual_novel
            .active
            .as_ref()
            .is_some_and(|session| session.blocking)
        {
            return Ok(None);
        }
        let Some(delta_ms) = self
            .state
            .active_battle
            .as_mut()
            .ok_or(GameError::InvalidAction)?
            .simulation_delta_for_tick(raw_delta_ms)
        else {
            return Ok(None);
        };
        self.advance_active_battle_by_after_seq(delta_ms, after_seq)
            .map(Some)
    }

    pub fn advance_active_battle_to_next_event_bucket(
        &mut self,
    ) -> Result<BehaviorResult, GameError> {
        let result_stats_policy = self.run_policy().battle_result_stats.clone();
        let post_battle_policy = self.run_policy().post_battle;
        let game_data = self.game_data.clone();
        let (mut completed_battle, trigger_context) = {
            let active = self
                .state
                .active_battle
                .as_mut()
                .ok_or(GameError::InvalidAction)?;
            active.ensure_battle_projection_commit_available()?;
            let previous_battle_time_ms = active.execution.last_event_time_ms();
            let outcome = active.battle.step_battle_execution(&mut active.execution)?;
            let completed_battle = match outcome {
                BattleStepOutcome::Running => None,
                BattleStepOutcome::Finished(finish) => {
                    let battle_result = active
                        .battle
                        .finalize_battle_execution(&mut active.execution, finish)?;
                    Some(Self::combat_battle_state_from_live_result(
                        active,
                        battle_result.winner,
                        battle_result.event_log,
                        battle_result.participant_results,
                        &result_stats_policy,
                        game_data.as_ref(),
                    ))
                }
            };
            let current_battle_time_ms = active.execution.last_event_time_ms();
            let trigger_context = (
                active.battle_uuid,
                previous_battle_time_ms,
                current_battle_time_ms,
            );
            (completed_battle, trigger_context)
        };
        let (battle_uuid, previous_battle_time_ms, current_battle_time_ms) = trigger_context;
        self.evaluate_visual_novel_triggers(
            super::visual_novel_triggers::VisualNovelTriggerEvent::BattleTimeReached {
                battle_uuid,
                previous_battle_time_ms,
                current_battle_time_ms,
            },
        )?;
        let mut result = {
            let active = self
                .state
                .active_battle
                .as_mut()
                .ok_or(GameError::InvalidAction)?;
            Self::active_battle_advanced_result(
                active,
                post_battle_policy,
                completed_battle.is_some(),
                false,
                0,
            )?
        };
        if let Some(battle) = completed_battle.as_mut() {
            battle.employee_conditions = self
                .state
                .active_battle
                .as_ref()
                .ok_or(GameError::InvalidAction)?
                .employee_attempt
                .roster
                .clone();
        }
        self.attach_visual_novel_to_battle_result(&mut result)?;
        if let Some(battle) = completed_battle {
            self.transition_finished_live_battle_to_result(battle_uuid, battle)?;
        }
        Ok(result)
    }

    pub(super) fn handle_request_battle_state(
        &mut self,
        since_seq: Option<u64>,
    ) -> Result<BehaviorResult, GameError> {
        let roster = self.roster()?.clone();
        let active = self
            .state
            .active_battle
            .as_ref()
            .ok_or(GameError::InvalidAction)?;
        let battle_update = active.battle_update_dto_after(since_seq.unwrap_or(0), &roster)?;
        Ok(BehaviorResult::BattleState {
            battle_uuid: active.battle_uuid,
            node_type: active.node_type,
            mission_variant: active.mission_variant,
            encounter_id: active.encounter_id.clone(),
            combat_preview: active.combat_preview.clone(),
            battle_update,
            finished: active.execution.is_finished(),
        })
    }

    pub(super) fn handle_recover_battle_setup_loss(
        &self,
        after_seq: u64,
    ) -> Result<BehaviorResult, GameError> {
        let active = self
            .state
            .active_battle
            .as_ref()
            .ok_or(GameError::InvalidAction)?;
        let battle_update = active.battle_update_dto_after(after_seq, active.employee_roster())?;
        Ok(BehaviorResult::BattleAdvanced {
            battle_uuid: active.battle_uuid,
            encounter_id: active.encounter_id.clone(),
            node_type: active.node_type,
            mission_variant: active.mission_variant,
            battle_setup_snapshot: Some(active.battle_setup_snapshot_dto()),
            battle_update,
            finished: active.execution.is_finished(),
        })
    }

    pub(super) fn handle_pause_battle(
        &mut self,
        after_seq: u64,
    ) -> Result<BehaviorResult, GameError> {
        let roster = self.roster()?.clone();
        let active = self
            .state
            .active_battle
            .as_mut()
            .ok_or(GameError::InvalidAction)?;
        if active.playback.paused {
            return Err(GameError::ActionUnavailable {
                action: crate::game::behavior::ActionKind::PauseBattle,
                reason: crate::game::behavior::ActionDisabledReasonCode::AlreadyActive,
            });
        }
        active.ensure_battle_projection_commit_available()?;
        active.playback.paused = true;
        active.commit_battle_projection()?;
        let battle_update = active.battle_update_dto_after(after_seq, &roster)?;
        Ok(BehaviorResult::BattlePlaybackChanged {
            battle_uuid: active.battle_uuid,
            encounter_id: active.encounter_id.clone(),
            node_type: active.node_type,
            mission_variant: active.mission_variant,
            battle_update,
        })
    }

    pub(super) fn handle_resume_battle(
        &mut self,
        after_seq: u64,
    ) -> Result<BehaviorResult, GameError> {
        let roster = self.roster()?.clone();
        let active = self
            .state
            .active_battle
            .as_mut()
            .ok_or(GameError::InvalidAction)?;
        if !active.playback.paused {
            return Err(GameError::ActionUnavailable {
                action: crate::game::behavior::ActionKind::ResumeBattle,
                reason: crate::game::behavior::ActionDisabledReasonCode::AlreadyActive,
            });
        }
        active.ensure_battle_projection_commit_available()?;
        active.playback.paused = false;
        active.commit_battle_projection()?;
        let battle_update = active.battle_update_dto_after(after_seq, &roster)?;
        Ok(BehaviorResult::BattlePlaybackChanged {
            battle_uuid: active.battle_uuid,
            encounter_id: active.encounter_id.clone(),
            node_type: active.node_type,
            mission_variant: active.mission_variant,
            battle_update,
        })
    }

    pub(super) fn handle_set_battle_speed(
        &mut self,
        speed: crate::game::behavior::BattlePlaybackSpeed,
        after_seq: u64,
    ) -> Result<BehaviorResult, GameError> {
        let roster = self.roster()?.clone();
        let active = self
            .state
            .active_battle
            .as_mut()
            .ok_or(GameError::InvalidAction)?;
        if active.playback.speed == speed {
            return Err(GameError::ActionUnavailable {
                action: crate::game::behavior::ActionKind::SetBattleSpeed,
                reason: crate::game::behavior::ActionDisabledReasonCode::AlreadyActive,
            });
        }
        active.ensure_battle_projection_commit_available()?;
        active.playback.speed = speed;
        active.playback_delta_remainder = 0;
        active.commit_battle_projection()?;
        let battle_update = active.battle_update_dto_after(after_seq, &roster)?;
        Ok(BehaviorResult::BattlePlaybackChanged {
            battle_uuid: active.battle_uuid,
            encounter_id: active.encounter_id.clone(),
            node_type: active.node_type,
            mission_variant: active.mission_variant,
            battle_update,
        })
    }

    pub(super) fn handle_deploy_unit(
        &mut self,
        employee_uuid: Uuid,
        position: Position,
        facing: crate::game::battle::tile_range::FacingDirection,
        source_command_id: Option<&str>,
        after_seq: u64,
    ) -> Result<BehaviorResult, GameError> {
        let combat_preview = self
            .state
            .active_battle
            .as_ref()
            .ok_or(GameError::InvalidAction)?
            .combat_preview
            .clone();
        self.validate_employee_deployment_cell(&combat_preview, employee_uuid, position)?;
        let draft = battle_unit_draft_for_employee(
            self.roster()?,
            self.inventory()?,
            &self.state.skill_fragments,
            self.game_data.as_ref(),
            employee_uuid,
        )?;
        self.validate_defense_route_deployable_profile(&draft)?;
        let (base_deploy_cost, current_hp_policy) = {
            let active = self
                .state
                .active_battle
                .as_ref()
                .ok_or(GameError::InvalidAction)?;
            let deployment = active
                .live_deployment
                .as_ref()
                .ok_or(GameError::InvalidAction)?;
            let redeploy_lock = deployment.redeploy_locks.get(&employee_uuid);
            (
                redeploy_lock
                    .map(|lock| lock.deploy_cost)
                    .unwrap_or(deployment.base_deploy_cost),
                redeploy_lock.and_then(|lock| lock.current_hp_policy),
            )
        };
        let deploy_cost = self
            .roster()?
            .get(&employee_uuid)
            .ok_or(GameError::UnitNotFound)?
            .deployment_cost_after_consumable(base_deploy_cost);
        if let Some(reason) = self
            .state
            .active_battle
            .as_ref()
            .ok_or(GameError::InvalidAction)?
            .deployment_candidate_eligibility(employee_uuid, deploy_cost)
            .disabled_reason_code
        {
            return Err(GameError::ActionUnavailable {
                action: crate::game::behavior::ActionKind::DeployUnit,
                reason,
            });
        }
        let roster = self.roster()?.clone();
        let post_battle_policy = self.run_policy().post_battle;
        let active = self
            .state
            .active_battle
            .as_mut()
            .ok_or(GameError::InvalidAction)?;
        active.ensure_battle_projection_commit_available()?;
        let deployment = active
            .live_deployment
            .as_mut()
            .ok_or(GameError::InvalidAction)?;
        let instance_salt = deployment.next_instance_salt;
        let next_instance_salt = deployment.next_instance_salt.saturating_add(1);
        let time_ms = active.execution.last_event_time_ms();
        let outcome = active.battle.apply_live_command_with_source_command_id(
            BattleLiveCommand::DeployPlayerUnit {
                draft,
                position,
                facing,
                instance_salt,
                time_ms,
                current_hp_policy,
            },
            source_command_id,
        )?;
        let BattleLiveCommandOutcome::UnitDeployed { unit_id } = outcome else {
            return Err(GameError::InvalidAction);
        };
        deployment.next_instance_salt = next_instance_salt;
        deployment.current_cost = deployment.current_cost.saturating_sub(deploy_cost);
        deployment.redeploy_locks.remove(&employee_uuid);
        deployment.deployed_units.insert(
            employee_uuid,
            LiveBattleDeployedUnitState {
                unit_instance_id: unit_id,
                facing,
            },
        );
        active.apply_live_signals_to_deployment();
        active.reconcile_employee_attempt(post_battle_policy)?;
        active.commit_battle_projection()?;
        let battle_update = active.battle_update_dto_after(after_seq, &roster)?;
        Ok(BehaviorResult::BattleUnitDeployed {
            battle_uuid: active.battle_uuid,
            encounter_id: active.encounter_id.clone(),
            node_type: active.node_type,
            mission_variant: active.mission_variant,
            battle_update,
            employee_uuid,
            unit_instance_id: unit_id,
        })
    }

    fn validate_defense_route_deployable_profile(
        &self,
        draft: &crate::game::battle::types::BattleUnitDraft,
    ) -> Result<(), GameError> {
        let Some(active) = self.state.active_battle.as_ref() else {
            return Err(GameError::InvalidAction);
        };
        if !crate::game::combat_setup::mission_policy::CombatMissionPolicy::starts_as_live_battle(
            active.node_type,
            active.mission_variant,
        ) {
            return Ok(());
        }
        let BattleUnitSource::Employee(profile) = &draft.source else {
            return Ok(());
        };
        let Some(skill_id) = &profile.skill_id else {
            return Ok(());
        };
        let skill = self
            .game_data
            .skill_data
            .get_by_id(skill_id.as_str())
            .ok_or_else(|| {
                GameError::InvalidStaticData(format!(
                    "DefenseRoute player skill '{}' is missing from skill database",
                    skill_id
                ))
            })?;
        let _ = skill;
        Ok(())
    }

    pub fn deployment_range_preview_dto(
        &self,
        employee_uuid: Uuid,
        position: Position,
        facing: FacingDirection,
    ) -> Result<LiveBattleRangePreviewsDto, GameError> {
        let active = self
            .state
            .active_battle
            .as_ref()
            .ok_or(GameError::InvalidAction)?;
        self.validate_employee_deployment_cell(&active.combat_preview, employee_uuid, position)?;
        let draft = battle_unit_draft_for_employee(
            self.roster()?,
            self.inventory()?,
            &self.state.skill_fragments,
            self.game_data.as_ref(),
            employee_uuid,
        )?;
        self.validate_defense_route_deployable_profile(&draft)?;
        let BattleUnitSource::Employee(profile) = &draft.source else {
            return Err(GameError::InvalidAction);
        };
        Ok(range_previews_for_combat_profile(
            profile,
            self.game_data.as_ref(),
            &active.battle.battlefield,
            position,
            facing,
        ))
    }

    pub(super) fn handle_request_deployment_range_preview(
        &self,
        employee_uuid: Uuid,
        position: Position,
    ) -> Result<BehaviorResult, GameError> {
        Ok(BehaviorResult::DeploymentRangePreview {
            result: DeploymentRangePreviewResultDto {
                employee_uuid,
                position,
                deploy_eligibility: self
                    .state
                    .active_battle
                    .as_ref()
                    .and_then(|active| active.live_deployment_dto(active.employee_roster()))
                    .and_then(|deployment| {
                        deployment
                            .unit_deploy_costs
                            .into_iter()
                            .find(|candidate| candidate.employee_uuid == employee_uuid)
                    })
                    .map(|candidate| candidate.eligibility)
                    .unwrap_or_else(|| {
                        crate::game::behavior::BattlePayloadEligibilityDto::disabled(
                            crate::game::behavior::ActionDisabledReasonCode::UnitUnavailable,
                        )
                    }),
                facings: DeploymentRangePreviewFacingsDto {
                    up: DeploymentRangePreviewFacingDto {
                        range_previews: self.deployment_range_preview_dto(
                            employee_uuid,
                            position,
                            FacingDirection::Up,
                        )?,
                    },
                    right: DeploymentRangePreviewFacingDto {
                        range_previews: self.deployment_range_preview_dto(
                            employee_uuid,
                            position,
                            FacingDirection::Right,
                        )?,
                    },
                    down: DeploymentRangePreviewFacingDto {
                        range_previews: self.deployment_range_preview_dto(
                            employee_uuid,
                            position,
                            FacingDirection::Down,
                        )?,
                    },
                    left: DeploymentRangePreviewFacingDto {
                        range_previews: self.deployment_range_preview_dto(
                            employee_uuid,
                            position,
                            FacingDirection::Left,
                        )?,
                    },
                },
            },
        })
    }

    pub(super) fn handle_withdraw_unit(
        &mut self,
        employee_uuid: Uuid,
        source_command_id: Option<&str>,
        after_seq: u64,
    ) -> Result<BehaviorResult, GameError> {
        let roster = self.roster()?.clone();
        let post_battle_policy = self.run_policy().post_battle;
        let active = self
            .state
            .active_battle
            .as_mut()
            .ok_or(GameError::InvalidAction)?;
        active.ensure_battle_projection_commit_available()?;
        let deployment = active
            .live_deployment
            .as_mut()
            .ok_or(GameError::InvalidAction)?;
        let unit_id = deployment
            .deployed_units
            .get(&employee_uuid)
            .map(|deployed| deployed.unit_instance_id)
            .ok_or(GameError::UnitNotFound)?;
        let time_ms = active.execution.last_event_time_ms();
        let outcome = active.battle.apply_live_command_with_source_command_id(
            BattleLiveCommand::WithdrawUnit { unit_id, time_ms },
            source_command_id,
        )?;
        if !matches!(outcome, BattleLiveCommandOutcome::UnitWithdrawn { .. }) {
            return Err(GameError::InvalidAction);
        }
        let current_hp_policy = active
            .battle
            .units
            .get(&unit_id)
            .map(|unit| {
                BattleDeployCurrentHpPolicy::withdraw_redeploy(
                    unit.stats.current_health,
                    unit.stats.max_health,
                )
            })
            .ok_or(GameError::UnitNotFound)?;
        deployment.deployed_units.remove(&employee_uuid);
        let deploy_cost = deployment
            .base_deploy_cost
            .saturating_mul(deployment.redeploy_cost_multiplier_pct)
            / 100;
        deployment.redeploy_locks.insert(
            employee_uuid,
            LiveBattleRedeployState {
                ready_at_ms: active
                    .execution
                    .last_event_time_ms()
                    .saturating_add(deployment.withdraw_redeploy_cooldown_ms),
                deploy_cost,
                current_hp_policy: Some(current_hp_policy),
            },
        );
        active.apply_live_signals_to_deployment();
        active.reconcile_employee_attempt(post_battle_policy)?;
        active.commit_battle_projection()?;
        let battle_update = active.battle_update_dto_after(after_seq, &roster)?;
        Ok(BehaviorResult::BattleUnitWithdrawn {
            battle_uuid: active.battle_uuid,
            encounter_id: active.encounter_id.clone(),
            node_type: active.node_type,
            mission_variant: active.mission_variant,
            battle_update,
            employee_uuid,
            unit_instance_id: unit_id,
        })
    }

    pub(super) fn handle_activate_skill(
        &mut self,
        employee_uuid: Uuid,
        skill_id: crate::game::ability::SkillId,
        target: Option<crate::game::battle::event_log::SkillCastTarget>,
        source_command_id: Option<&str>,
        after_seq: u64,
    ) -> Result<BehaviorResult, GameError> {
        let roster = self.roster()?.clone();
        let post_battle_policy = self.run_policy().post_battle;
        let active = self
            .state
            .active_battle
            .as_mut()
            .ok_or(GameError::InvalidAction)?;
        active.ensure_battle_projection_commit_available()?;
        let unit_id = active
            .live_deployment
            .as_ref()
            .and_then(|deployment| deployment.deployed_units.get(&employee_uuid))
            .map(|deployed| deployed.unit_instance_id)
            .ok_or(GameError::UnitNotFound)?;
        let time_ms = active.execution.last_event_time_ms();
        active.battle.validate_manual_skill_activation(
            time_ms,
            unit_id,
            &skill_id,
            target.clone(),
        )?;
        let outcome = active.battle.apply_live_command_with_source_command_id(
            BattleLiveCommand::ActivateSkill {
                unit_id,
                skill_id: skill_id.clone(),
                target,
                time_ms,
            },
            source_command_id,
        )?;
        if !matches!(outcome, BattleLiveCommandOutcome::SkillActivated { .. }) {
            return Err(GameError::InvalidAction);
        }
        active.apply_live_signals_to_deployment();
        active.reconcile_employee_attempt(post_battle_policy)?;
        active.commit_battle_projection()?;
        let battle_update = active.battle_update_dto_after(after_seq, &roster)?;
        Ok(BehaviorResult::BattleSkillActivated {
            battle_uuid: active.battle_uuid,
            encounter_id: active.encounter_id.clone(),
            node_type: active.node_type,
            mission_variant: active.mission_variant,
            battle_update,
            employee_uuid,
            unit_instance_id: unit_id,
            skill_id,
        })
    }

    pub(super) fn handle_retreat_battle(&mut self) -> Result<BehaviorResult, GameError> {
        let active = self
            .state
            .active_battle
            .take()
            .ok_or(GameError::InvalidAction)?;
        let snapshot = active.entry_snapshot.ok_or(GameError::InvalidAction)?;
        snapshot.restore(&mut self.state);

        let (node_id, kind_id, category) = match &self.state.game_state {
            GameState::NodeConfirm {
                node_id,
                kind_id,
                category,
            } => (*node_id, kind_id.clone(), *category),
            _ => return Err(GameError::InvalidAction),
        };
        let session = self
            .state
            .node_session
            .clone()
            .ok_or(GameError::InvalidAction)?;
        let payload = session.payload.clone();
        let combat_preview = if matches!(category, MapNodeCategory::Combat | MapNodeCategory::Boss)
        {
            self.combat_preview_for_node(node_id)?
        } else {
            None
        };
        Ok(BehaviorResult::NodePreview {
            node_id,
            kind_id,
            category,
            payload,
            session,
            map: self.current_map_view()?,
            combat_preview,
            trauma_recovery_preview: None,
        })
    }

    fn return_to_node_confirm_for_combat_retry(
        &mut self,
        node_id: MapNodeId,
        combat_preview: CombatPreview,
    ) -> Result<BehaviorResult, GameError> {
        let (kind_id, category, payload, session) = {
            let run = self.run_state()?;
            let node = run.map.node(node_id).ok_or(GameError::InvalidAction)?;
            let enter_result = MapNodeExecutor::enter(node);
            (
                enter_result.kind_id,
                enter_result.category,
                enter_result.payload,
                enter_result.session,
            )
        };
        self.run_state_mut()?
            .combat_previews
            .insert(node_id, combat_preview.clone());
        self.state.active_battle = None;
        self.state.active_node_content = None;
        self.state.node_session = Some(session.clone());
        self.transition_to(GameState::NodeConfirm {
            node_id,
            kind_id: kind_id.clone(),
            category,
        })?;
        Ok(BehaviorResult::NodePreview {
            node_id,
            kind_id,
            category,
            payload,
            session,
            map: self.current_map_view()?,
            combat_preview: Some(combat_preview),
            trauma_recovery_preview: None,
        })
    }

    pub(super) fn handle_complete_combat_result(&mut self) -> Result<BehaviorResult, GameError> {
        self.with_state_transaction(Self::handle_complete_combat_result_inner)
    }

    fn handle_complete_combat_result_inner(&mut self) -> Result<BehaviorResult, GameError> {
        let battle = {
            let content = self
                .state
                .active_node_content
                .as_ref()
                .ok_or(GameError::InvalidAction)?;
            content.as_combat_battle()?.clone()
        };
        let battle_node_id = self
            .state
            .node_session
            .as_ref()
            .map(|session| session.node_id)
            .ok_or(GameError::InvalidAction)?;

        let employee_before = self.employee_outcome_before(&battle.result_stats.employees)?;

        if battle.winner != BattleWinner::Player {
            let mut staged = self.stage_committed_combat_resolution(&battle)?;
            let employee_changes = Self::employee_outcome_changes_from_roster(
                &staged.roster,
                employee_before,
                &battle.result_stats.employees,
            )?;
            let outcome = self.combat_node_outcome_summary(
                &battle,
                false,
                employee_changes,
                staged.condition_inventory_diff.clone(),
            )?;
            let failure_reason = Self::current_run_failure_reason_for_roster(&staged.roster);
            let is_final_boss = self.is_final_boss_node(battle_node_id)?;
            info!(
                "Combat result finished without player victory (winner={:?}); skipping rewards",
                battle.winner
            );
            if is_final_boss || failure_reason == Some(RunFailureReason::NoLivingEmployees) {
                let reason = if is_final_boss {
                    RunFailureReason::BossDefeated
                } else {
                    RunFailureReason::NoLivingEmployees
                };
                let encounter_discoveries = self.validated_encounter_discoveries(&battle)?;
                self.commit_staged_combat_result_state(staged);
                let contributing_reason = failure_reason.filter(|derived| *derived != reason);
                let result =
                    self.fail_run_with_reasons(reason, contributing_reason, Some(outcome))?;
                self.commit_encounter_discoveries(encounter_discoveries);
                return Ok(result);
            }

            let attempts_exhausted = self
                .run_state()?
                .abnormality_attempt_state(battle_node_id)
                .is_exhausted();
            if failure_reason.is_none() && !attempts_exhausted {
                let combat_preview = self
                    .combat_preview_for_node(battle_node_id)?
                    .ok_or(GameError::InvalidAction)?;
                self.commit_staged_combat_result_state(staged);
                return self
                    .return_to_node_confirm_for_combat_retry(battle_node_id, combat_preview);
            }

            self.resolve_failed_monolith_managed_combat(battle_node_id, attempts_exhausted)?;

            let node_completion =
                CombatResultNodeCompletion::try_from(self.plan_complete_current_node(false)?)?;
            let recovery_restriction =
                (failure_reason == Some(RunFailureReason::NoDeployableEmployees)).then(|| {
                    let (allowed_node_ids, path_exists) = self.recovery_path_for_completion(
                        &node_completion,
                        &staged.roster,
                        &staged.inventory,
                    );
                    PlannedCombatRecoveryRestriction {
                        allowed_node_ids,
                        path_exists,
                    }
                });
            let returns_to_planning = matches!(
                node_completion,
                CombatResultNodeCompletion::NodeCompleted { .. }
                    | CombatResultNodeCompletion::FloorAdvanced { .. }
            ) && recovery_restriction
                .as_ref()
                .is_none_or(|restriction| restriction.path_exists);
            let research_deliveries = if returns_to_planning {
                self.deliver_pending_research_in_staged_inventory(&mut staged.skill_fragments)?
            } else {
                Vec::new()
            };
            let encounter_discoveries = self.validated_encounter_discoveries(&battle)?;
            self.commit_staged_combat_result_state(staged);
            let completion =
                self.commit_combat_result_node_completion(node_completion, research_deliveries)?;
            self.commit_encounter_discoveries(encounter_discoveries);
            if let Some(restriction) = recovery_restriction {
                if !restriction.path_exists {
                    return self.fail_run_with_outcome(
                        RunFailureReason::NoDeployableEmployees,
                        Some(outcome),
                    );
                }
                self.set_combat_recovery_restriction(restriction.allowed_node_ids);
            }
            if let BehaviorResult::NodeCompleted {
                research_deliveries,
                ..
            } = completion
            {
                let map = self.current_map_view()?;
                return Ok(BehaviorResult::NodeCompleted {
                    map,
                    outcome: Some(outcome),
                    research_deliveries,
                });
            }
            return Ok(completion);
        }

        if let Some(result) = self.fail_run_if_needed()? {
            return Ok(result);
        }

        let monolith_victory =
            self.resolve_victorious_monolith_managed_combat(&battle, battle_node_id)?;
        let planned = self.plan_player_victory_combat_result_completion(
            &battle,
            battle_node_id,
            employee_before,
            monolith_victory,
        )?;
        self.commit_planned_player_victory_combat_result_completion(planned)
    }

    fn resolve_failed_monolith_managed_combat(
        &mut self,
        node_id: MapNodeId,
        attempts_exhausted: bool,
    ) -> Result<(), GameError> {
        if self.run_state()?.run_progression.game_mode != GameMode::Endless {
            return Ok(());
        }
        if self
            .run_state()?
            .monolith
            .remanifestation(node_id)
            .is_some()
        {
            self.run_state_mut()?
                .monolith
                .resolve_remanifestation_failure(node_id)
                .map_err(|_| GameError::InvalidAction)?;
            return Ok(());
        }
        if !attempts_exhausted {
            return Ok(());
        }
        let abnormality_id = {
            let run = self.run_state()?;
            let node = run.map.node(node_id).ok_or(GameError::InvalidAction)?;
            if node.category != MapNodeCategory::Combat || !node.kind_id.as_str().contains("elite")
            {
                return Ok(());
            }
            let MapNodePayload::Encounter {
                encounter_id: Some(encounter_id),
            } = &node.payload
            else {
                return Ok(());
            };
            self.game_data
                .pve_data
                .get_by_id(encounter_id)
                .and_then(|encounter| encounter.primary_abnormality_id())
                .map(str::to_string)
        };
        let Some(abnormality_id) = abnormality_id else {
            return Ok(());
        };
        let already_resolved = self
            .run_state()?
            .monolith
            .progress(&abnormality_id)
            .is_some_and(|progress| progress.is_naturally_excluded());
        if !already_resolved {
            self.run_state_mut()?
                .monolith
                .record_natural_failure(&abnormality_id)
                .map_err(|_| GameError::InvalidAction)?;
        }
        Ok(())
    }

    fn resolve_victorious_monolith_managed_combat(
        &mut self,
        battle: &CombatBattleState,
        node_id: MapNodeId,
    ) -> Result<MonolithVictoryPlan, GameError> {
        if self.run_state()?.run_progression.game_mode != GameMode::Endless {
            return Ok(MonolithVictoryPlan::default());
        }
        let remanifestation = self.run_state()?.monolith.remanifestation(node_id).cloned();
        if let Some(remanifestation) = remanifestation {
            let threshold = self.run_policy().monolith.residual_exchange_threshold;
            let resolution = self
                .run_state_mut()?
                .monolith
                .resolve_remanifestation_victory(node_id, threshold)
                .map_err(|_| GameError::InvalidAction)?;
            let mut reward_effects = Vec::new();
            if resolution.grant_unique_fragment {
                let fragment_id = self
                    .game_data
                    .abnormality_data
                    .get_by_id(&remanifestation.abnormality_id)
                    .and_then(|metadata| metadata.response_complete_skill_fragment_id.clone())
                    .ok_or_else(|| {
                        GameError::InvalidStaticData(format!(
                            "monolith abnormality '{}' has no unique response-complete fragment",
                            remanifestation.abnormality_id
                        ))
                    })?;
                reward_effects.push(RewardEffect::GrantSkillFragment { fragment_id });
            }
            return Ok(MonolithVictoryPlan {
                managed: true,
                reward_effects,
            });
        }

        let is_natural_elite = {
            let run = self.run_state()?;
            run.map.node(node_id).is_some_and(|node| {
                node.category == MapNodeCategory::Combat && node.kind_id.as_str().contains("elite")
            })
        };
        if !is_natural_elite {
            return Ok(MonolithVictoryPlan::default());
        }
        let Some(abnormality_id) = battle.primary_abnormality_id.as_deref() else {
            return Ok(MonolithVictoryPlan::default());
        };
        let already_resolved = self
            .run_state()?
            .monolith
            .progress(abnormality_id)
            .is_some_and(|progress| progress.is_naturally_excluded());
        if !already_resolved {
            self.run_state_mut()?
                .monolith
                .record_natural_victory(abnormality_id)
                .map_err(|_| GameError::InvalidAction)?;
        }
        Ok(MonolithVictoryPlan {
            managed: true,
            ..MonolithVictoryPlan::default()
        })
    }

    fn plan_player_victory_combat_result_completion(
        &mut self,
        battle: &CombatBattleState,
        battle_node_id: MapNodeId,
        employee_before: HashMap<Uuid, EmployeeOutcomeBefore>,
        monolith_victory: MonolithVictoryPlan,
    ) -> Result<PlannedPlayerVictoryCombatResultCompletion, GameError> {
        let node_completion =
            CombatResultNodeCompletion::try_from(self.plan_complete_current_node(false)?)?;
        let encounter_discoveries = self.validated_encounter_discoveries(battle)?;
        let mut abnormality_research_after_victory = None;
        let mut abnormality_research_reward_effects = Vec::new();
        let mut endless_response_complete = false;
        if self.run_state()?.run_progression.game_mode == GameMode::Endless
            && !monolith_victory.managed
        {
            let mut research = self.run_state()?.abnormality_research.clone();
            let bonus_research_gain = satisfied_research_bonus(&battle.bonus_objectives);
            if let Some(primary_abnormality_id) = battle.primary_abnormality_id.as_deref() {
                let outcome = research
                    .apply_suppression_victory_with_bonus(
                        self.game_data.as_ref(),
                        primary_abnormality_id,
                        self.run_state()?.run_progression.floor_index(),
                        self.is_final_boss_node(battle_node_id)?,
                        bonus_research_gain,
                    )
                    .map_err(GameError::InvalidStaticData)?;
                abnormality_research_reward_effects = outcome.reward_effects;
                endless_response_complete = research.all_response_complete();
                abnormality_research_after_victory = Some(research);
            }
        }
        let mut staged = self.staged_combat_result_state();
        self.apply_post_battle_resolution_to_state(&mut staged, battle)?;
        Self::decrement_consumables_after_combat_node_in_roster(&mut staged.roster);

        match Self::current_run_failure_reason_for_roster(&staged.roster) {
            Some(RunFailureReason::NoLivingEmployees) => {
                return Ok(PlannedPlayerVictoryCombatResultCompletion::RunFailure {
                    staged,
                    reason: RunFailureReason::NoLivingEmployees,
                    encounter_discoveries,
                });
            }
            Some(RunFailureReason::BossDefeated) => {
                return Ok(PlannedPlayerVictoryCombatResultCompletion::RunFailure {
                    staged,
                    reason: RunFailureReason::BossDefeated,
                    encounter_discoveries,
                });
            }
            Some(RunFailureReason::NoDeployableEmployees) | None => {}
        }

        let reward_session = self.build_reward_session(
            battle.abnormality_uuid,
            battle.reward_mode,
            battle.rewards.clone(),
            false,
        );
        let rewards_to_apply = match reward_session.mode {
            RewardMode::ClaimAll => reward_session.rewards.clone(),
            RewardMode::ChooseOne => {
                return Err(GameError::InvalidStaticData(format!(
                    "combat result '{}' uses ChooseOne reward mode, but combat result rewards are automatically granted and must use ClaimAll",
                    battle.encounter_id
                )));
            }
        };
        let experience_context = Self::combat_experience_context_from_roster(
            &staged.roster,
            &battle.result_stats.employees,
        );
        let (
            reward_inventory_diff,
            skill_fragment_diffs,
            skill_fragment_research_diffs,
            employee_experience_diffs,
        ) = self.apply_reward_options_with_context_to_state(
            reward_session.stage_uuid,
            &rewards_to_apply,
            &experience_context,
            &mut staged.inventory,
            &mut staged.skill_fragments,
            &mut staged.roster,
            &mut staged.uuid_manager,
            &mut staged.enkephalin,
        )?;
        let mut inventory_diff = staged.condition_inventory_diff.clone();
        Self::merge_inventory_diff(&mut inventory_diff, reward_inventory_diff);
        let mut skill_fragment_diffs = skill_fragment_diffs;
        let mut skill_fragment_research_diffs = skill_fragment_research_diffs;
        abnormality_research_reward_effects.extend(monolith_victory.reward_effects);
        if !abnormality_research_reward_effects.is_empty() {
            let granted = GrantExecutor::grant_effects_with_state(
                &mut staged.inventory,
                &mut staged.skill_fragments,
                &mut staged.roster,
                &mut staged.uuid_manager,
                &mut staged.enkephalin,
                &self.game_data,
                &self.state.skill_fragment_policy,
                &GrantExecutionContext::default(),
                &abnormality_research_reward_effects,
                self.run_seed ^ 0xABAD_0BAD_ABAD_0BAD,
            )?;
            Self::merge_inventory_diff(&mut inventory_diff, granted.inventory_diff);
            skill_fragment_diffs.extend(granted.skill_fragment_diffs);
            skill_fragment_research_diffs.extend(granted.skill_fragment_research_diffs);
        }
        let enkephalin = staged.enkephalin.amount;
        let employee_changes = Self::employee_outcome_changes_from_roster(
            &staged.roster,
            employee_before,
            &battle.result_stats.employees,
        )?;
        let outcome = self.combat_node_outcome_summary(
            battle,
            true,
            employee_changes,
            inventory_diff.clone(),
        )?;
        let recovery_restriction = (Self::current_run_failure_reason_for_roster(&staged.roster)
            == Some(RunFailureReason::NoDeployableEmployees))
        .then(|| {
            let (allowed_node_ids, path_exists) = self.recovery_path_for_completion(
                &node_completion,
                &staged.roster,
                &staged.inventory,
            );
            PlannedCombatRecoveryRestriction {
                allowed_node_ids,
                path_exists,
            }
        });
        let returns_to_planning = matches!(
            node_completion,
            CombatResultNodeCompletion::NodeCompleted { .. }
                | CombatResultNodeCompletion::FloorAdvanced { .. }
        ) && match recovery_restriction.as_ref() {
            Some(restriction) => restriction.path_exists,
            None => !endless_response_complete,
        };
        let research_deliveries = if returns_to_planning {
            self.deliver_pending_research_in_staged_inventory(&mut staged.skill_fragments)?
        } else {
            Vec::new()
        };

        Ok(PlannedPlayerVictoryCombatResultCompletion::RewardsGranted {
            staged,
            encounter_discoveries,
            research_deliveries,
            abnormality_research_after_victory,
            node_completion,
            endless_response_complete,
            enkephalin,
            inventory_diff,
            skill_fragment_diffs,
            skill_fragment_research_diffs,
            employee_experience_diffs,
            outcome,
            recovery_restriction,
        })
    }

    fn commit_planned_player_victory_combat_result_completion(
        &mut self,
        planned: PlannedPlayerVictoryCombatResultCompletion,
    ) -> Result<BehaviorResult, GameError> {
        match planned {
            PlannedPlayerVictoryCombatResultCompletion::RunFailure {
                staged,
                reason,
                encounter_discoveries,
            } => {
                self.commit_staged_combat_result_state(staged);
                let result = self.fail_run(reason)?;
                self.commit_encounter_discoveries(encounter_discoveries);
                Ok(result)
            }
            PlannedPlayerVictoryCombatResultCompletion::RewardsGranted {
                staged,
                encounter_discoveries,
                research_deliveries,
                abnormality_research_after_victory,
                node_completion,
                endless_response_complete,
                enkephalin,
                inventory_diff,
                skill_fragment_diffs,
                skill_fragment_research_diffs,
                employee_experience_diffs,
                outcome,
                recovery_restriction,
            } => {
                self.commit_staged_combat_result_state(staged);
                if let Some(research) = abnormality_research_after_victory {
                    self.run_state_mut()?.abnormality_research = research;
                }
                let mut completion = self
                    .commit_combat_result_node_completion(node_completion, research_deliveries)?;
                self.commit_encounter_discoveries(encounter_discoveries);
                if let Some(restriction) = recovery_restriction {
                    if !restriction.path_exists {
                        completion = self.fail_run(RunFailureReason::NoDeployableEmployees)?;
                    } else {
                        self.set_combat_recovery_restriction(restriction.allowed_node_ids);
                        if let BehaviorResult::NodeCompleted {
                            research_deliveries,
                            ..
                        } = completion
                        {
                            completion = BehaviorResult::NodeCompleted {
                                map: self.current_map_view()?,
                                outcome: None,
                                research_deliveries,
                            };
                        }
                    }
                } else if endless_response_complete {
                    self.transition_to(GameState::RunComplete)?;
                    completion = BehaviorResult::RunComplete {
                        map: self.current_map_view()?,
                    };
                }
                Ok(BehaviorResult::CombatRewardsGranted {
                    enkephalin,
                    inventory_diff,
                    skill_fragment_diffs,
                    skill_fragment_research_diffs,
                    employee_experience_diffs,
                    outcome,
                    completion: Box::new(completion),
                })
            }
        }
    }
}
