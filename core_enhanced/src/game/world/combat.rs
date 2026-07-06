use std::collections::HashMap;

use tracing::info;
use uuid::Uuid;

use super::{
    node_flow::CombatResultNodeCompletion,
    state::{
        ActiveBattleSession, LiveBattleDeployedUnitState, LiveBattleDeploymentState,
        LiveBattleRedeployState,
    },
    GameCore,
};
use crate::game::ability::{DeliveryDef, SkillTarget};
use crate::game::abnormality_research::RunAbnormalityResearchState;
use crate::game::battle::{
    core::sim::{
        BattleDeployCurrentHpPolicy, BattleLiveCommand, BattleLiveCommandOutcome, BattleStepOutcome,
    },
    event_log::BattleEventLog,
    result_stats::collect_battle_result_stats,
    tile_range::{FacingDirection, TileRangePolicy},
    types::{BattleUnitSource, BattleWinner, ParticipantBattleResult},
};
use crate::game::behavior::{
    BattlePlaybackState, BehaviorResult, CombatOutcomeSummary, DeploymentRangePreviewFacingDto,
    DeploymentRangePreviewFacingsDto, DeploymentRangePreviewResultDto, GameError,
    LiveBattleRangePreviewsDto, NodeOutcomeEmployeeChange, NodeOutcomeSummary,
};
use crate::game::combat_preview::{CombatPreview, DeploymentZoneKind};
use crate::game::combat_setup::player_spawns::battle_unit_draft_for_employee;
use crate::game::employee::{EmployeeInjury, EmployeeLifeState, EmployeeRoster};
use crate::game::employee_trust::EmployeeTrustResolver;
use crate::game::enums::{RewardMode, Side};
use crate::game::events::combat::CombatExecutor;
use crate::game::managers::uuid_manager::UuidManager;
use crate::game::map::{GameMode, MapNodeCategory, MapNodeExecutor, MapNodeId, MapNodePayload};
use crate::game::pve_bonus_objectives::{evaluate_pve_bonus_objectives, satisfied_research_bonus};
use crate::game::range_preview::range_previews_for_combat_profile;
use crate::game::resources::{
    ActiveNodeContent, CombatBattleState, Enkephalin, GameState, Inventory, InventoryDiffDto,
    Position, RunFailureReason,
};
use crate::game::reward::{
    EmployeeExperienceDiffDto, ExperienceTargetPolicy, GrantExecutionContext, GrantExecutor,
    RewardEffect, SkillFragmentGrantDiffDto, SkillFragmentResearchDiffDto,
};
use crate::game::skill_fragment::SkillFragmentInventory;

#[derive(Debug, Clone, Copy)]
struct EmployeeOutcomeBefore {
    run_hp: u32,
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
}

enum PlannedPlayerVictoryCombatResultCompletion {
    RunFailure {
        staged: StagedCombatResultState,
        reason: RunFailureReason,
    },
    RewardsGranted {
        staged: StagedCombatResultState,
        abnormality_research_after_victory: Option<RunAbnormalityResearchState>,
        node_completion: CombatResultNodeCompletion,
        endless_response_complete: bool,
        enkephalin: u32,
        inventory_diff: InventoryDiffDto,
        skill_fragment_diffs: Vec<SkillFragmentGrantDiffDto>,
        skill_fragment_research_diffs: Vec<SkillFragmentResearchDiffDto>,
        employee_experience_diffs: Vec<EmployeeExperienceDiffDto>,
        outcome: NodeOutcomeSummary,
    },
}

impl GameCore {
    pub(super) fn combat_preview_for_node(
        &mut self,
        node_id: MapNodeId,
    ) -> Result<Option<CombatPreview>, GameError> {
        let (category, encounter_id, existing, floor_index) = {
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
            )
        };

        if let Some(existing) = existing {
            return Ok(Some(existing));
        }
        if !matches!(category, MapNodeCategory::Combat | MapNodeCategory::Boss) {
            return Ok(None);
        }

        let seed = self.node_seed(node_id, 0x5052_4556); // "PREV"
        let preview = CombatPreview::try_generate_for_node_at_floor(
            node_id,
            category,
            encounter_id.as_deref(),
            self.game_data.as_ref(),
            seed,
            floor_index,
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
    ) -> Result<BehaviorResult, GameError> {
        let category = self
            .run_state()?
            .map
            .node(node_id)
            .ok_or(GameError::InvalidAction)?
            .category;
        self.handle_explicit_combat_node(node_id, category, primary_abnormality_id, encounter_id)
    }

    pub(super) fn handle_event_combat_node(
        &mut self,
        node_id: MapNodeId,
        primary_abnormality_id: Option<String>,
        encounter_id: String,
    ) -> Result<BehaviorResult, GameError> {
        self.handle_explicit_combat_node(
            node_id,
            MapNodeCategory::Combat,
            primary_abnormality_id,
            encounter_id,
        )
    }

    fn explicit_combat_preview_for_node(
        &mut self,
        node_id: MapNodeId,
        category: MapNodeCategory,
        encounter_id: &str,
    ) -> Result<CombatPreview, GameError> {
        if let Some(existing) = self.run_state()?.combat_previews.get(&node_id).cloned() {
            return Ok(existing);
        }
        let floor_index = self.run_state()?.run_progression.floor_index();
        let seed = self.node_seed(node_id, 0x5052_4556);
        let preview = CombatPreview::try_generate_for_node_at_floor(
            node_id,
            category,
            Some(encounter_id),
            self.game_data.as_ref(),
            seed,
            floor_index,
        )?;
        self.run_state_mut()?
            .combat_previews
            .insert(node_id, preview.clone());
        Ok(preview)
    }

    fn handle_explicit_combat_node(
        &mut self,
        node_id: MapNodeId,
        category: MapNodeCategory,
        primary_abnormality_id: Option<String>,
        encounter_id: String,
    ) -> Result<BehaviorResult, GameError> {
        if let Some(result) = self.block_or_fail_undeployable_combat_selection()? {
            return Ok(result);
        }
        let selected_uuid = node_id.0;
        info!(
            "Starting map combat node for primary_abnormality={:?} encounter={} node={}",
            primary_abnormality_id, encounter_id, selected_uuid
        );

        let combat_preview =
            self.explicit_combat_preview_for_node(node_id, category, &encounter_id)?;
        let node_type = combat_preview.node_type;
        let mission_variant = combat_preview.mission_variant;
        let (reward_mode, rewards) = CombatExecutor::resolve_rewards_for_mission(
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

        if crate::game::combat_setup::mission_policy::CombatMissionPolicy::starts_as_live_battle(
            node_type,
            mission_variant,
        ) {
            let empty_deployment = HashMap::new();
            let mut battle = CombatExecutor::build_battle_with_combat_preview(
                self.roster()?,
                self.inventory()?,
                &self.state.skill_fragments,
                self.game_data.clone(),
                primary_abnormality_id.as_deref(),
                &encounter_id,
                self.run_seed,
                &combat_preview,
                &empty_deployment,
            )?;
            let execution = battle.start_battle_execution()?;
            if self
                .run_state_mut()?
                .start_abnormality_attempt(node_id)
                .is_none()
            {
                return Err(GameError::InvalidAction);
            }
            let live_deployment_policy = self.run_policy().live_deployment;
            self.state.active_battle = Some(ActiveBattleSession {
                battle_uuid: selected_uuid,
                primary_abnormality_id,
                encounter_id,
                node_type,
                mission_variant,
                reward_mode,
                rewards,
                combat_preview,
                battle,
                execution,
                last_pushed_event_log_seq: None,
                live_deployment: Some(LiveBattleDeploymentState::new(live_deployment_policy)),
                playback: BattlePlaybackState::default(),
                playback_delta_remainder: 0,
            });
            self.transition_to(GameState::InBattle {
                battle_uuid: selected_uuid,
            })?;
            return self.advance_active_battle_by_internal(0, true);
        }

        Err(GameError::InvalidStaticData(format!(
            "combat encounter '{}' uses unsupported non-live combat mission {:?}/{:?}; official combat flow is DefenseRoute live battle only",
            encounter_id, node_type, mission_variant
        )))
    }

    fn is_final_boss_node(&self, node_id: MapNodeId) -> Result<bool, GameError> {
        let run = self.run_state()?;
        let is_standard_final_boss = run.run_progression.is_final_standard_floor()
            && run.map.terminal_node_id == node_id
            && run
                .map
                .node(node_id)
                .is_some_and(|node| node.category == MapNodeCategory::Boss);
        Ok(is_standard_final_boss || run.boss_omen.forced_boss_node_id() == Some(node_id))
    }

    pub(super) fn apply_post_battle_resolution(
        &mut self,
        participant_results: &[ParticipantBattleResult],
    ) -> Result<(), GameError> {
        let mut staged = self.staged_combat_result_state();
        self.apply_post_battle_resolution_to_state(&mut staged, participant_results)?;
        self.commit_staged_combat_result_state(staged);
        self.sync_roster_order_with_owned_units()?;
        Ok(())
    }

    fn staged_combat_result_state(&self) -> StagedCombatResultState {
        StagedCombatResultState {
            inventory: self.state.inventory.clone(),
            skill_fragments: self.state.skill_fragments.clone(),
            roster: self.state.roster.clone(),
            uuid_manager: self.state.uuid_manager.clone(),
            enkephalin: self.state.enkephalin.clone(),
        }
    }

    fn commit_staged_combat_result_state(&mut self, staged: StagedCombatResultState) {
        self.state.inventory = staged.inventory;
        self.state.skill_fragments = staged.skill_fragments;
        self.state.roster = staged.roster;
        self.state.uuid_manager = staged.uuid_manager;
        self.state.enkephalin = staged.enkephalin;
    }

    fn apply_post_battle_resolution_to_state(
        &self,
        staged: &mut StagedCombatResultState,
        participant_results: &[ParticipantBattleResult],
    ) -> Result<(), GameError> {
        let trust_policy = self.state.employee_trust_policy.clone();
        let post_battle_policy = self.run_policy().post_battle;
        let post_battle_survival_xp = self.run_policy().growth.post_battle_survival_xp;
        if staged.roster.is_empty() {
            return Ok(());
        }

        let mut survival_xp_employee_ids = Vec::new();
        for participant in participant_results
            .iter()
            .filter(|participant| participant.side == Side::Player)
        {
            let Some(employee) = staged.roster.get_mut(&participant.owned_uuid) else {
                continue;
            };

            if participant.became_incapacitated {
                let trauma_outcome = EmployeeTrustResolver::modify_trauma(
                    &employee.trust,
                    employee.uuid,
                    post_battle_policy.incapacitation_trauma,
                    &trust_policy,
                );
                employee.trust.apply_reaction(&trauma_outcome.reaction);
                employee.apply_incapacitation(
                    trauma_outcome.final_amount,
                    post_battle_policy.incapacitation_run_hp_loss_percent,
                    crate::game::employee::EmployeeInjury {
                        id: "battle_incapacitation".to_string(),
                        severity: 1,
                    },
                );
                info!(
                    "Employee {} incapacitated: trauma={}, life_state={:?}",
                    employee.uuid, employee.trauma, employee.life_state
                );
            } else if participant.survived {
                survival_xp_employee_ids.push(employee.uuid);
            } else {
                employee.apply_incapacitation(
                    post_battle_policy.incapacitation_trauma,
                    post_battle_policy.incapacitation_run_hp_loss_percent,
                    EmployeeInjury {
                        id: "battle_loss".to_string(),
                        severity: 1,
                    },
                );
            }
        }

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

    fn decrement_consumables_after_combat_node(&mut self) -> Result<(), GameError> {
        let mut staged = self.staged_combat_result_state();
        Self::decrement_consumables_after_combat_node_in_roster(&mut staged.roster);
        self.commit_staged_combat_result_state(staged);
        Ok(())
    }

    fn decrement_consumables_after_combat_node_in_roster(roster: &mut EmployeeRoster) {
        for employee in roster.iter_mut() {
            employee.decrement_consumable_after_combat_node();
        }
    }

    fn employee_outcome_before(
        &self,
        participant_results: &[ParticipantBattleResult],
    ) -> Result<HashMap<Uuid, EmployeeOutcomeBefore>, GameError> {
        let roster = self.roster()?;
        let mut before = HashMap::new();
        for participant in participant_results
            .iter()
            .filter(|participant| participant.side == Side::Player)
        {
            if let Some(employee) = roster.get(&participant.owned_uuid) {
                before.insert(
                    participant.owned_uuid,
                    EmployeeOutcomeBefore {
                        run_hp: employee.health.current_hp,
                        trauma: employee.trauma,
                        experience: employee.experience,
                        alive: employee.life_state == EmployeeLifeState::Alive,
                    },
                );
            }
        }
        Ok(before)
    }

    fn employee_outcome_changes(
        &self,
        before: HashMap<Uuid, EmployeeOutcomeBefore>,
        participant_results: &[ParticipantBattleResult],
    ) -> Result<Vec<NodeOutcomeEmployeeChange>, GameError> {
        let roster = self.roster()?;
        Self::employee_outcome_changes_from_roster(roster, before, participant_results)
    }

    fn employee_outcome_changes_from_roster(
        roster: &EmployeeRoster,
        before: HashMap<Uuid, EmployeeOutcomeBefore>,
        participant_results: &[ParticipantBattleResult],
    ) -> Result<Vec<NodeOutcomeEmployeeChange>, GameError> {
        let mut changes = Vec::new();
        for participant in participant_results
            .iter()
            .filter(|participant| participant.side == Side::Player)
        {
            let Some(before) = before.get(&participant.owned_uuid) else {
                continue;
            };
            let Some(employee) = roster.get(&participant.owned_uuid) else {
                continue;
            };
            changes.push(NodeOutcomeEmployeeChange {
                employee_uuid: participant.owned_uuid,
                survived: participant.survived,
                became_incapacitated: participant.became_incapacitated,
                run_hp_before: before.run_hp,
                run_hp_after: employee.health.current_hp,
                trauma_before: before.trauma,
                trauma_after: employee.trauma,
                experience_before: before.experience,
                experience_after: employee.experience,
                was_alive: before.alive,
                is_alive: employee.life_state == EmployeeLifeState::Alive,
            });
        }
        Ok(changes)
    }

    fn combat_experience_context_from_roster(
        roster: &EmployeeRoster,
        participant_results: &[ParticipantBattleResult],
    ) -> GrantExecutionContext {
        let eligible_employee_ids = participant_results
            .iter()
            .filter(|participant| {
                participant.side == Side::Player
                    && participant.survived
                    && !participant.became_incapacitated
                    && roster
                        .get(&participant.owned_uuid)
                        .is_some_and(|employee| employee.life_state == EmployeeLifeState::Alive)
            })
            .map(|participant| participant.owned_uuid)
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
            research_deliveries: Vec::new(),
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

        if !roster
            .iter()
            .any(|employee| employee.life_state == EmployeeLifeState::Alive)
        {
            return Some(RunFailureReason::NoLivingEmployees);
        }

        if roster.available_employee_ids().is_empty() {
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
        progression.complete_current_node(&mut map).is_ok()
    }

    fn can_recover_no_deployable_with_checkpoint(&self) -> bool {
        self.state.run_checkpoint.can_load()
    }

    pub(super) fn block_or_fail_undeployable_combat_selection(
        &mut self,
    ) -> Result<Option<BehaviorResult>, GameError> {
        match self.current_run_failure_reason()? {
            Some(RunFailureReason::NoLivingEmployees) => {
                self.fail_run(RunFailureReason::NoLivingEmployees).map(Some)
            }
            Some(RunFailureReason::NoDeployableEmployees) => {
                if self.can_recover_no_deployable_with_checkpoint() {
                    Err(GameError::InvalidAction)
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
        self.state.node_session = None;
        self.state.active_node_content = None;
        self.state.active_battle = None;
        self.transition_to(GameState::RunFailed { reason })?;
        Ok(BehaviorResult::RunFailed { reason, outcome })
    }

    fn fail_run_if_needed(&mut self) -> Result<Option<BehaviorResult>, GameError> {
        match self.current_run_failure_reason()? {
            Some(RunFailureReason::NoLivingEmployees) => {
                self.fail_run(RunFailureReason::NoLivingEmployees).map(Some)
            }
            Some(RunFailureReason::NoDeployableEmployees) => {
                if self.can_recover_no_deployable_with_checkpoint() {
                    Ok(None)
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

    fn active_battle_advanced_result(
        active: &mut ActiveBattleSession,
        roster: &EmployeeRoster,
        finished: bool,
        include_setup_snapshot: bool,
    ) -> BehaviorResult {
        active.refresh_live_deployment_cost();
        active.apply_live_signals_to_deployment();
        let battle_setup_snapshot =
            include_setup_snapshot.then(|| active.battle_setup_snapshot_dto());
        let battle_update = active.drain_battle_update_dto(roster);
        BehaviorResult::BattleAdvanced {
            battle_uuid: active.battle_uuid,
            encounter_id: active.encounter_id.clone(),
            node_type: active.node_type,
            mission_variant: active.mission_variant,
            battle_setup_snapshot,
            battle_update,
            finished,
        }
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
            abnormality_uuid: active.battle_uuid,
            winner,
            event_log,
            result_stats,
            bonus_objectives,
            reward_mode: active.reward_mode,
            rewards: active.rewards.clone(),
            participant_results,
        }
    }

    fn transition_finished_live_battle_to_result(
        &mut self,
        battle: CombatBattleState,
    ) -> Result<(), GameError> {
        let battle_uuid = battle.abnormality_uuid;
        self.store_and_export_abnormality_battle_record(&battle)?;
        self.state.active_node_content = Some(ActiveNodeContent::CombatBattle(battle));
        self.state.active_battle = None;
        self.transition_to(GameState::CombatResult { battle_uuid })
    }

    fn store_and_export_abnormality_battle_record(
        &mut self,
        battle: &CombatBattleState,
    ) -> Result<(), GameError> {
        let inserted = self
            .run_state_mut()?
            .store_abnormality_battle_record(battle);
        if inserted {
            self.run_state()?
                .export_abnormality_battle_record_debug_json(battle)?;
        }
        Ok(())
    }

    pub fn advance_active_battle_by(&mut self, delta_ms: u64) -> Result<BehaviorResult, GameError> {
        self.advance_active_battle_by_internal(delta_ms, false)
    }

    fn advance_active_battle_by_internal(
        &mut self,
        delta_ms: u64,
        include_setup_snapshot: bool,
    ) -> Result<BehaviorResult, GameError> {
        let roster = self.state.roster.clone();
        let result_stats_policy = self.run_policy().battle_result_stats.clone();
        let game_data = self.game_data.clone();
        let (result, completed_battle) = {
            let active = self
                .state
                .active_battle
                .as_mut()
                .ok_or(GameError::InvalidAction)?;
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
            let result = Self::active_battle_advanced_result(
                active,
                &roster,
                completed_battle.is_some(),
                include_setup_snapshot,
            );
            (result, completed_battle)
        };
        if let Some(battle) = completed_battle {
            self.transition_finished_live_battle_to_result(battle)?;
        }
        Ok(result)
    }

    pub fn advance_active_battle_for_server_tick(
        &mut self,
        raw_delta_ms: u64,
    ) -> Result<Option<BehaviorResult>, GameError> {
        let Some(delta_ms) = self
            .state
            .active_battle
            .as_mut()
            .ok_or(GameError::InvalidAction)?
            .simulation_delta_for_tick(raw_delta_ms)
        else {
            return Ok(None);
        };
        self.advance_active_battle_by(delta_ms).map(Some)
    }

    pub fn advance_active_battle_to_next_event_bucket(
        &mut self,
    ) -> Result<BehaviorResult, GameError> {
        let roster = self.state.roster.clone();
        let result_stats_policy = self.run_policy().battle_result_stats.clone();
        let game_data = self.game_data.clone();
        let (result, completed_battle) = {
            let active = self
                .state
                .active_battle
                .as_mut()
                .ok_or(GameError::InvalidAction)?;
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
            let result = Self::active_battle_advanced_result(
                active,
                &roster,
                completed_battle.is_some(),
                false,
            );
            (result, completed_battle)
        };
        if let Some(battle) = completed_battle {
            self.transition_finished_live_battle_to_result(battle)?;
        }
        Ok(result)
    }

    pub(super) fn handle_request_battle_state(
        &mut self,
        since_seq: Option<u64>,
    ) -> Result<BehaviorResult, GameError> {
        let roster = self.state.roster.clone();
        let active = self
            .state
            .active_battle
            .as_mut()
            .ok_or(GameError::InvalidAction)?;
        active.reconcile_live_deployment_with_battle_state();
        let battle_update = active.battle_update_dto_after(since_seq, &roster)?;
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

    pub(super) fn handle_recover_battle_setup_loss(&mut self) -> Result<BehaviorResult, GameError> {
        let (node_id, kind_id, category, payload, session, combat_preview) = {
            let Some(active) = self.state.active_battle.as_ref() else {
                return Err(GameError::InvalidAction);
            };
            let node_id = MapNodeId(active.battle_uuid);
            let combat_preview = active.combat_preview.clone();
            let run = self.run_state()?;
            let node = run.map.node(node_id).ok_or(GameError::InvalidAction)?;
            let enter_result = MapNodeExecutor::enter(node);
            (
                node_id,
                enter_result.kind_id,
                enter_result.category,
                enter_result.payload,
                enter_result.session,
                combat_preview,
            )
        };

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
        })
    }

    pub(super) fn handle_pause_battle(&mut self) -> Result<BehaviorResult, GameError> {
        let roster = self.state.roster.clone();
        let active = self
            .state
            .active_battle
            .as_mut()
            .ok_or(GameError::InvalidAction)?;
        active.playback.paused = true;
        let battle_update = active.drain_battle_update_dto(&roster);
        Ok(BehaviorResult::BattlePlaybackChanged {
            battle_uuid: active.battle_uuid,
            encounter_id: active.encounter_id.clone(),
            node_type: active.node_type,
            mission_variant: active.mission_variant,
            battle_update,
        })
    }

    pub(super) fn handle_resume_battle(&mut self) -> Result<BehaviorResult, GameError> {
        let roster = self.state.roster.clone();
        let active = self
            .state
            .active_battle
            .as_mut()
            .ok_or(GameError::InvalidAction)?;
        active.playback.paused = false;
        let battle_update = active.drain_battle_update_dto(&roster);
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
    ) -> Result<BehaviorResult, GameError> {
        let roster = self.state.roster.clone();
        let active = self
            .state
            .active_battle
            .as_mut()
            .ok_or(GameError::InvalidAction)?;
        active.playback.speed = speed;
        active.playback_delta_remainder = 0;
        let battle_update = active.drain_battle_update_dto(&roster);
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
                redeploy_lock.map(|lock| lock.current_hp_policy),
            )
        };
        let deploy_cost = self
            .roster()?
            .get(&employee_uuid)
            .ok_or(GameError::UnitNotFound)?
            .deployment_cost_after_consumable(base_deploy_cost);
        let roster = self.state.roster.clone();
        let active = self
            .state
            .active_battle
            .as_mut()
            .ok_or(GameError::InvalidAction)?;
        let deployment = active
            .live_deployment
            .as_mut()
            .ok_or(GameError::InvalidAction)?;
        if deployment.deployed_units.contains_key(&employee_uuid) {
            return Err(GameError::UnitAlreadyPlaced);
        }
        if let Some(lock) = deployment.redeploy_locks.get(&employee_uuid) {
            if active.execution.last_event_time_ms() < lock.ready_at_ms {
                return Err(GameError::InvalidAction);
            }
        }
        if deployment.current_cost < deploy_cost {
            return Err(GameError::InsufficientResources);
        }
        let instance_salt = deployment.next_instance_salt;
        deployment.next_instance_salt = deployment.next_instance_salt.saturating_add(1);
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
        let battle_update = active.drain_battle_update_dto(&roster);
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
        for step in &skill.steps {
            if matches!(
                step.target,
                SkillTarget::EnemySingle { .. } | SkillTarget::CastTarget
            ) || matches!(step.delivery, DeliveryDef::TileArea { .. })
            {
                if step.range_policy == TileRangePolicy::WholeFieldValidTiles
                    || step.defense_tile_range.is_some()
                {
                    continue;
                }
                return Err(GameError::InvalidStaticData(format!(
                    "DefenseRoute player skill '{}' step '{}' is missing defense_tile_range or WholeFieldValidTiles range_policy",
                    skill_id, step.id
                )));
            }
        }
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
    ) -> Result<BehaviorResult, GameError> {
        let roster = self.state.roster.clone();
        let active = self
            .state
            .active_battle
            .as_mut()
            .ok_or(GameError::InvalidAction)?;
        let deployment = active
            .live_deployment
            .as_mut()
            .ok_or(GameError::InvalidAction)?;
        let deployed = deployment
            .deployed_units
            .remove(&employee_uuid)
            .ok_or(GameError::UnitNotFound)?;
        let unit_id = deployed.unit_instance_id;
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
                current_hp_policy,
            },
        );
        let battle_update = active.drain_battle_update_dto(&roster);
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
    ) -> Result<BehaviorResult, GameError> {
        let roster = self.state.roster.clone();
        let active = self
            .state
            .active_battle
            .as_mut()
            .ok_or(GameError::InvalidAction)?;
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
        let battle_update = active.drain_battle_update_dto(&roster);
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
        let (node_id, mut combat_preview, attempts_exhausted, final_boss) = {
            let Some(active) = self.state.active_battle.as_ref() else {
                return Err(GameError::InvalidAction);
            };
            let node_id = MapNodeId(active.battle_uuid);
            let attempts_exhausted = self
                .run_state()?
                .abnormality_attempt_state(node_id)
                .is_exhausted();
            let final_boss = self.is_final_boss_node(node_id)?;
            (
                node_id,
                active.combat_preview.clone(),
                attempts_exhausted,
                final_boss,
            )
        };

        if !attempts_exhausted {
            combat_preview.disprove_rumor_warnings();
            return self.return_to_node_confirm_for_combat_retry(node_id, combat_preview);
        }

        let battle = {
            let active = self
                .state
                .active_battle
                .as_ref()
                .ok_or(GameError::InvalidAction)?;
            let result_stats_policy = self.run_policy().battle_result_stats.clone();
            let game_data = self.game_data.clone();
            Self::combat_battle_state_from_live_result(
                active,
                BattleWinner::Draw,
                active.battle.event_log.clone(),
                Vec::new(),
                &result_stats_policy,
                game_data.as_ref(),
            )
        };
        self.state.active_battle = None;
        self.store_and_export_abnormality_battle_record(&battle)?;
        let outcome = self.combat_node_outcome_summary_with_resolution(
            &battle,
            false,
            BattleWinner::Draw,
            true,
            Vec::new(),
            InventoryDiffDto::default(),
        )?;
        self.decrement_consumables_after_combat_node()?;
        if final_boss {
            return self.fail_run_with_outcome(RunFailureReason::BossDefeated, Some(outcome));
        }
        let completion = self.handle_complete_node()?;
        Ok(match completion {
            BehaviorResult::NodeCompleted { map, .. } => BehaviorResult::NodeCompleted {
                map,
                outcome: Some(outcome),
            },
            other => other,
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
        })
    }

    pub(super) fn handle_complete_combat_result(&mut self) -> Result<BehaviorResult, GameError> {
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

        let employee_before = self.employee_outcome_before(&battle.participant_results)?;

        if self.is_final_boss_node(battle_node_id)? && battle.winner != BattleWinner::Player {
            self.apply_post_battle_resolution(&battle.participant_results)?;
            self.decrement_consumables_after_combat_node()?;
            let employee_changes =
                self.employee_outcome_changes(employee_before, &battle.participant_results)?;
            let outcome = self
                .combat_node_outcome_summary(
                    &battle,
                    false,
                    employee_changes,
                    InventoryDiffDto::default(),
                )
                .ok();
            return self.fail_run_with_outcome(RunFailureReason::BossDefeated, outcome);
        }

        if battle.winner != BattleWinner::Player {
            self.apply_post_battle_resolution(&battle.participant_results)?;
            self.decrement_consumables_after_combat_node()?;
            let employee_changes =
                self.employee_outcome_changes(employee_before, &battle.participant_results)?;
            info!(
                "Combat result finished without player victory (winner={:?}); skipping rewards",
                battle.winner
            );
            if self.state.node_session.is_some() {
                let node_id = battle_node_id;
                if !self
                    .run_state()?
                    .abnormality_attempt_state(node_id)
                    .is_exhausted()
                {
                    let combat_preview = self
                        .combat_preview_for_node(node_id)?
                        .ok_or(GameError::InvalidAction)?;
                    return self.return_to_node_confirm_for_combat_retry(node_id, combat_preview);
                }

                let failure_reason = self.current_run_failure_reason()?;
                match failure_reason {
                    Some(
                        reason @ (RunFailureReason::NoLivingEmployees
                        | RunFailureReason::BossDefeated),
                    ) => {
                        let outcome = self.combat_node_outcome_summary(
                            &battle,
                            false,
                            employee_changes,
                            InventoryDiffDto::default(),
                        )?;
                        return self.fail_run_with_outcome(reason, Some(outcome));
                    }
                    Some(RunFailureReason::NoDeployableEmployees) => {
                        let outcome = self.combat_node_outcome_summary(
                            &battle,
                            false,
                            employee_changes,
                            InventoryDiffDto::default(),
                        )?;
                        let completion = self.handle_complete_node()?;
                        if self.current_run_failure_reason()?
                            == Some(RunFailureReason::NoDeployableEmployees)
                            && !self.can_recover_no_deployable_with_checkpoint()
                        {
                            return self.fail_run_with_outcome(
                                RunFailureReason::NoDeployableEmployees,
                                Some(outcome),
                            );
                        }
                        return Ok(match completion {
                            BehaviorResult::NodeCompleted { map, .. } => {
                                BehaviorResult::NodeCompleted {
                                    map,
                                    outcome: Some(outcome),
                                }
                            }
                            other => other,
                        });
                    }
                    None => {
                        let outcome = self.combat_node_outcome_summary(
                            &battle,
                            false,
                            employee_changes,
                            InventoryDiffDto::default(),
                        )?;
                        let completion = self.handle_complete_node()?;
                        return Ok(match completion {
                            BehaviorResult::NodeCompleted { map, .. } => {
                                BehaviorResult::NodeCompleted {
                                    map,
                                    outcome: Some(outcome),
                                }
                            }
                            other => other,
                        });
                    }
                }
            }
            return Err(GameError::InvalidAction);
        }

        if let Some(result) = self.fail_run_if_needed()? {
            return Ok(result);
        }

        let planned = self.plan_player_victory_combat_result_completion(
            &battle,
            battle_node_id,
            employee_before,
        )?;
        self.commit_planned_player_victory_combat_result_completion(planned)
    }

    fn plan_player_victory_combat_result_completion(
        &mut self,
        battle: &CombatBattleState,
        battle_node_id: MapNodeId,
        employee_before: HashMap<Uuid, EmployeeOutcomeBefore>,
    ) -> Result<PlannedPlayerVictoryCombatResultCompletion, GameError> {
        let node_completion =
            CombatResultNodeCompletion::try_from(self.plan_complete_current_node(false)?)?;
        let mut abnormality_research_after_victory = None;
        let mut abnormality_research_reward_effects = Vec::new();
        let mut endless_response_complete = false;
        if self.run_state()?.run_progression.game_mode == GameMode::Endless {
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
        self.apply_post_battle_resolution_to_state(&mut staged, &battle.participant_results)?;
        Self::decrement_consumables_after_combat_node_in_roster(&mut staged.roster);

        match Self::current_run_failure_reason_for_roster(&staged.roster) {
            Some(RunFailureReason::NoLivingEmployees) => {
                return Ok(PlannedPlayerVictoryCombatResultCompletion::RunFailure {
                    staged,
                    reason: RunFailureReason::NoLivingEmployees,
                });
            }
            Some(RunFailureReason::NoDeployableEmployees)
                if !self.can_recover_no_deployable_with_checkpoint() =>
            {
                return Ok(PlannedPlayerVictoryCombatResultCompletion::RunFailure {
                    staged,
                    reason: RunFailureReason::NoDeployableEmployees,
                });
            }
            Some(RunFailureReason::BossDefeated) => {
                return Ok(PlannedPlayerVictoryCombatResultCompletion::RunFailure {
                    staged,
                    reason: RunFailureReason::BossDefeated,
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
            &battle.participant_results,
        );
        let (
            inventory_diff,
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
        let mut inventory_diff = inventory_diff;
        let mut skill_fragment_diffs = skill_fragment_diffs;
        let mut skill_fragment_research_diffs = skill_fragment_research_diffs;
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
            &battle.participant_results,
        )?;
        let outcome = self.combat_node_outcome_summary(
            &battle,
            true,
            employee_changes,
            inventory_diff.clone(),
        )?;

        Ok(PlannedPlayerVictoryCombatResultCompletion::RewardsGranted {
            staged,
            abnormality_research_after_victory,
            node_completion,
            endless_response_complete,
            enkephalin,
            inventory_diff,
            skill_fragment_diffs,
            skill_fragment_research_diffs,
            employee_experience_diffs,
            outcome,
        })
    }

    fn commit_planned_player_victory_combat_result_completion(
        &mut self,
        planned: PlannedPlayerVictoryCombatResultCompletion,
    ) -> Result<BehaviorResult, GameError> {
        match planned {
            PlannedPlayerVictoryCombatResultCompletion::RunFailure { staged, reason } => {
                self.commit_staged_combat_result_state(staged);
                self.fail_run(reason)
            }
            PlannedPlayerVictoryCombatResultCompletion::RewardsGranted {
                staged,
                abnormality_research_after_victory,
                node_completion,
                endless_response_complete,
                enkephalin,
                inventory_diff,
                skill_fragment_diffs,
                skill_fragment_research_diffs,
                employee_experience_diffs,
                outcome,
            } => {
                self.commit_staged_combat_result_state(staged);
                if let Some(research) = abnormality_research_after_victory {
                    self.run_state_mut()?.abnormality_research = research;
                }
                let completion = self.commit_combat_result_node_completion(node_completion)?;
                let completion = if endless_response_complete {
                    self.transition_to(GameState::RunComplete)?;
                    BehaviorResult::RunComplete {
                        map: self.current_map_view()?,
                    }
                } else {
                    completion
                };
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
