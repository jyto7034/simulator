use std::collections::{BTreeSet, HashMap};

use tracing::info;
use uuid::Uuid;

use super::{
    state::{
        ActiveBattleSession, LiveBattleDeployedUnitState, LiveBattleDeploymentState,
        LiveBattleRedeployState,
    },
    GameCore, RUN_SYSTEM_POLICY,
};
use crate::game::ability::{DeliveryDef, SkillTarget};
use crate::game::battle::{
    core::sim::{BattleLiveCommand, BattleLiveCommandOutcome, BattleStepOutcome},
    timeline::Timeline,
    types::{BattleWinner, ParticipantBattleResult},
};
use crate::game::behavior::{
    BattlePlaybackState, BehaviorResult, CombatOutcomeSummary, GameError,
    NodeOutcomeEmployeeChange, NodeOutcomeSummary,
};
use crate::game::combat_player_spawns::battle_unit_draft_for_employee;
use crate::game::combat_preview::{CombatNodeType, CombatPreview, DeploymentZoneKind};
use crate::game::employee::{EmployeeInjury, EmployeeLifeState, EmployeeRoster};
use crate::game::employee_trust::EmployeeTrustResolver;
use crate::game::enums::Side;
use crate::game::events::combat::CombatExecutor;
use crate::game::managers::qliphoth_manager::QliphothManager;
use crate::game::map::{
    MapNodeCategory, MapNodeExecutor, MapNodeId, MapNodePayload, SupportNodeMode,
};
use crate::game::resources::{
    ActiveNodeContent, CombatBattleState, GameState, InventoryDiffDto, Position,
    RewardSessionState, RunFailureReason,
};
use crate::game::reward::RewardEffect;

#[derive(Debug, Clone, Copy)]
struct EmployeeOutcomeBefore {
    run_hp: u32,
    trauma: u32,
    experience: u32,
    alive: bool,
}

impl GameCore {
    pub(super) fn combat_preview_for_node(
        &mut self,
        node_id: MapNodeId,
    ) -> Result<Option<CombatPreview>, GameError> {
        let (category, encounter_id, existing) = {
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
            )
        };

        if !matches!(category, MapNodeCategory::Combat | MapNodeCategory::Boss) {
            return Ok(None);
        }
        if let Some(existing) = existing {
            return Ok(Some(existing));
        }

        let seed = self.node_seed(node_id, 0x5052_4556); // "PREV"
        let preview = CombatPreview::try_generate_for_node(
            node_id,
            category,
            encounter_id.as_deref(),
            self.game_data.as_ref(),
            seed,
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
        abnormality_id: String,
        encounter_id: String,
    ) -> Result<BehaviorResult, GameError> {
        if let Some(result) = self.block_or_fail_undeployable_combat_selection()? {
            return Ok(result);
        }
        let selected_uuid = node_id.0;
        info!(
            "Starting map combat node for abnormality={} encounter={} node={}",
            abnormality_id, encounter_id, selected_uuid
        );

        let combat_preview = self
            .combat_preview_for_node(node_id)?
            .ok_or(GameError::InvalidAction)?;
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

        if crate::game::combat_mission_policy::CombatMissionPolicy::starts_as_live_battle(
            node_type,
            mission_variant,
        ) {
            let empty_deployment = HashMap::new();
            let mut battle = CombatExecutor::build_battle_with_combat_preview(
                self.roster()?,
                self.inventory()?,
                &self.state.skill_fragments,
                self.game_data.clone(),
                &abnormality_id,
                &encounter_id,
                self.run_seed,
                &combat_preview,
                &empty_deployment,
            )?;
            let execution = battle.start_battle_execution()?;
            if node_type != CombatNodeType::Boss
                && self
                    .run_state_mut()?
                    .start_abnormality_attempt(node_id)
                    .is_none()
            {
                return Err(GameError::InvalidAction);
            }
            self.state.active_battle = Some(ActiveBattleSession {
                battle_uuid: selected_uuid,
                abnormality_id,
                encounter_id,
                node_type,
                mission_variant,
                reward_mode,
                rewards,
                combat_preview,
                battle,
                execution,
                last_pushed_timeline_seq: None,
                live_deployment: Some(LiveBattleDeploymentState::new(
                    RUN_SYSTEM_POLICY.live_deployment,
                )),
                playback: BattlePlaybackState::default(),
                playback_delta_remainder: 0,
            });
            self.transition_to(GameState::InBattle {
                battle_uuid: selected_uuid,
            })?;
            return self.advance_active_battle_by(0);
        }

        Err(GameError::InvalidStaticData(format!(
            "combat encounter '{}' uses unsupported non-live combat mission {:?}/{:?}; official combat flow is DefenseRoute live battle only",
            encounter_id, node_type, mission_variant
        )))
    }

    pub(super) fn apply_post_battle_resolution(
        &mut self,
        participant_results: &[ParticipantBattleResult],
    ) -> Result<(), GameError> {
        let trust_policy = self.state.employee_trust_policy.clone();
        let roster = self.roster_mut()?;
        if roster.is_empty() {
            return Ok(());
        }

        for participant in participant_results
            .iter()
            .filter(|participant| participant.side == Side::Player)
        {
            let Some(employee) = roster.get_mut(&participant.owned_uuid) else {
                continue;
            };

            if participant.became_incapacitated {
                let trauma_outcome = EmployeeTrustResolver::modify_trauma(
                    &employee.trust,
                    employee.uuid,
                    RUN_SYSTEM_POLICY.post_battle.incapacitation_trauma,
                    &trust_policy,
                );
                employee.trust.apply_reaction(&trauma_outcome.reaction);
                employee.apply_incapacitation(
                    trauma_outcome.final_amount,
                    RUN_SYSTEM_POLICY
                        .post_battle
                        .incapacitation_run_hp_loss_percent,
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
                employee.add_experience(RUN_SYSTEM_POLICY.post_battle.survival_xp);
            } else {
                employee.apply_incapacitation(
                    RUN_SYSTEM_POLICY.post_battle.incapacitation_trauma,
                    RUN_SYSTEM_POLICY
                        .post_battle
                        .incapacitation_run_hp_loss_percent,
                    EmployeeInjury {
                        id: "battle_loss".to_string(),
                        severity: 1,
                    },
                );
            }
        }

        let _ = roster;
        self.sync_roster_order_with_owned_units()?;
        Ok(())
    }

    fn decrement_consumables_after_combat_node(&mut self) -> Result<(), GameError> {
        let roster = self.roster_mut()?;
        for employee in roster.iter_mut() {
            employee.decrement_consumable_after_combat_node();
        }
        Ok(())
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

    fn combat_experience_reward_amount(
        &self,
        reward: &RewardSessionState,
    ) -> Result<u32, GameError> {
        let rewards_to_apply = match reward.mode {
            crate::game::enums::RewardMode::ClaimAll => reward.rewards.clone(),
            crate::game::enums::RewardMode::ChooseOne => vec![reward
                .get_selected_reward()
                .cloned()
                .ok_or(GameError::InvalidAction)?],
        };

        let mut amount = 0_u32;
        for reward in rewards_to_apply {
            for effect in reward.effects {
                if let RewardEffect::GrantExperience {
                    amount: effect_amount,
                } = effect
                {
                    amount = amount
                        .checked_add(effect_amount)
                        .ok_or(GameError::InvalidAction)?;
                }
            }
        }
        Ok(amount)
    }

    fn apply_combat_experience_reward(
        &mut self,
        participant_results: &[ParticipantBattleResult],
        amount: u32,
    ) -> Result<(), GameError> {
        if amount == 0 {
            return Ok(());
        }

        let eligible_employee_ids = {
            let roster = self.roster()?;
            participant_results
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
                .collect::<BTreeSet<_>>()
        };

        if eligible_employee_ids.is_empty() {
            return Ok(());
        }

        let eligible_count =
            u32::try_from(eligible_employee_ids.len()).map_err(|_| GameError::InvalidAction)?;
        let base_amount = amount / eligible_count;
        let mut remainder = amount % eligible_count;
        let roster = self.roster_mut()?;
        for employee_id in eligible_employee_ids {
            let extra = u32::from(remainder > 0);
            remainder = remainder.saturating_sub(1);
            let employee = roster
                .get_mut(&employee_id)
                .ok_or(GameError::UnitNotFound)?;
            employee.add_experience(base_amount.saturating_add(extra));
        }
        Ok(())
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
        if roster.is_empty() {
            return Ok(None);
        }

        if !roster
            .iter()
            .any(|employee| employee.life_state == EmployeeLifeState::Alive)
        {
            return Ok(Some(RunFailureReason::NoLivingEmployees));
        }

        if roster.available_employee_ids().is_empty() {
            return Ok(Some(RunFailureReason::NoDeployableEmployees));
        }

        Ok(None)
    }

    fn has_available_medical_support_node(&self) -> Result<bool, GameError> {
        let run = self.run_state()?;
        Ok(run
            .map_progression
            .available_node_ids
            .iter()
            .any(|node_id| {
                run.map.node(*node_id).is_some_and(|node| {
                    if node.category != MapNodeCategory::Support {
                        return false;
                    }
                    match &node.payload {
                        MapNodePayload::Support {
                            support_type,
                            support_mode,
                            choices,
                        } => match support_mode {
                            SupportNodeMode::Known => {
                                *support_type == crate::game::map::SupportNodeType::Medical
                            }
                            SupportNodeMode::LimitedChoice => {
                                choices.contains(&crate::game::map::SupportNodeType::Medical)
                            }
                            SupportNodeMode::FullChoice => true,
                        },
                        _ => false,
                    }
                })
            }))
    }

    pub(super) fn block_or_fail_undeployable_combat_selection(
        &mut self,
    ) -> Result<Option<BehaviorResult>, GameError> {
        match self.current_run_failure_reason()? {
            Some(RunFailureReason::NoLivingEmployees) => {
                self.fail_run(RunFailureReason::NoLivingEmployees).map(Some)
            }
            Some(RunFailureReason::NoDeployableEmployees) => {
                if self.has_available_medical_support_node()? {
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
                if self.has_available_medical_support_node()? {
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
    ) -> BehaviorResult {
        active.refresh_live_deployment_cost();
        active.apply_live_signals_to_deployment();
        let timeline_delta = active.drain_event_log_delta();
        BehaviorResult::BattleAdvanced {
            battle_uuid: active.battle_uuid,
            encounter_id: active.encounter_id.clone(),
            node_type: active.node_type,
            mission_variant: active.mission_variant,
            playback: active.playback_state(),
            battle_time_ms: active.execution.last_event_time_ms(),
            timeline_delta,
            last_timeline_seq: active.last_event_log_seq(),
            finished,
            deployment: active.live_deployment_dto(roster),
        }
    }

    fn combat_battle_state_from_live_result(
        active: &ActiveBattleSession,
        winner: BattleWinner,
        timeline: Timeline,
        participant_results: Vec<ParticipantBattleResult>,
    ) -> CombatBattleState {
        CombatBattleState {
            abnormality_id: active.abnormality_id.clone(),
            encounter_id: active.encounter_id.clone(),
            node_type: active.node_type,
            mission_variant: active.mission_variant,
            abnormality_uuid: active.battle_uuid,
            winner,
            timeline,
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
        self.state.active_node_content = Some(ActiveNodeContent::CombatBattle(battle));
        self.state.active_battle = None;
        self.transition_to(GameState::CombatResult { battle_uuid })
    }

    pub fn advance_active_battle_by(&mut self, delta_ms: u64) -> Result<BehaviorResult, GameError> {
        let roster = self.state.roster.clone();
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
                        battle_result.timeline,
                        battle_result.participant_results,
                    ))
                }
            };
            let result =
                Self::active_battle_advanced_result(active, &roster, completed_battle.is_some());
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
                        battle_result.timeline,
                        battle_result.participant_results,
                    ))
                }
            };
            let result =
                Self::active_battle_advanced_result(active, &roster, completed_battle.is_some());
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
        let roster = self.roster()?;
        let active = self
            .state
            .active_battle
            .as_ref()
            .ok_or(GameError::InvalidAction)?;
        Ok(BehaviorResult::BattleState {
            battle_uuid: active.battle_uuid,
            node_type: active.node_type,
            mission_variant: active.mission_variant,
            encounter_id: active.encounter_id.clone(),
            combat_preview: active.combat_preview.clone(),
            playback: active.playback_state(),
            battle_time_ms: active.execution.last_event_time_ms(),
            timeline_delta: active.event_log_delta_after(since_seq),
            last_timeline_seq: active.last_event_log_seq(),
            finished: active.execution.is_finished(),
            deployment: active.live_deployment_dto(roster),
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
        Ok(BehaviorResult::BattlePlaybackChanged {
            battle_uuid: active.battle_uuid,
            encounter_id: active.encounter_id.clone(),
            node_type: active.node_type,
            mission_variant: active.mission_variant,
            playback: active.playback_state(),
            battle_time_ms: active.execution.last_event_time_ms(),
            last_timeline_seq: active.last_event_log_seq(),
            deployment: active.live_deployment_dto(&roster),
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
        Ok(BehaviorResult::BattlePlaybackChanged {
            battle_uuid: active.battle_uuid,
            encounter_id: active.encounter_id.clone(),
            node_type: active.node_type,
            mission_variant: active.mission_variant,
            playback: active.playback_state(),
            battle_time_ms: active.execution.last_event_time_ms(),
            last_timeline_seq: active.last_event_log_seq(),
            deployment: active.live_deployment_dto(&roster),
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
        Ok(BehaviorResult::BattlePlaybackChanged {
            battle_uuid: active.battle_uuid,
            encounter_id: active.encounter_id.clone(),
            node_type: active.node_type,
            mission_variant: active.mission_variant,
            playback: active.playback_state(),
            battle_time_ms: active.execution.last_event_time_ms(),
            last_timeline_seq: active.last_event_log_seq(),
            deployment: active.live_deployment_dto(&roster),
        })
    }

    pub(super) fn handle_deploy_unit(
        &mut self,
        employee_uuid: Uuid,
        position: Position,
        facing: crate::game::battle::tile_range::FacingDirection,
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
        let base_deploy_cost = {
            let active = self
                .state
                .active_battle
                .as_ref()
                .ok_or(GameError::InvalidAction)?;
            let deployment = active
                .live_deployment
                .as_ref()
                .ok_or(GameError::InvalidAction)?;
            deployment
                .redeploy_locks
                .get(&employee_uuid)
                .map(|lock| lock.deploy_cost)
                .unwrap_or(deployment.base_deploy_cost)
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
        let outcome = active
            .battle
            .apply_live_command(BattleLiveCommand::DeployPlayerUnit {
                draft,
                position,
                facing,
                instance_salt,
                time_ms,
            })?;
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
        let timeline_delta = active.drain_event_log_delta();
        let deployment = active
            .live_deployment_dto(&roster)
            .ok_or(GameError::InvalidAction)?;
        Ok(BehaviorResult::BattleUnitDeployed {
            battle_uuid: active.battle_uuid,
            encounter_id: active.encounter_id.clone(),
            node_type: active.node_type,
            mission_variant: active.mission_variant,
            playback: active.playback_state(),
            employee_uuid,
            unit_instance_id: unit_id,
            timeline_delta,
            last_timeline_seq: active.last_event_log_seq(),
            deployment,
        })
    }

    fn validate_defense_route_deployable_profile(
        &self,
        draft: &crate::game::battle::types::BattleUnitDraft,
    ) -> Result<(), GameError> {
        let Some(active) = self.state.active_battle.as_ref() else {
            return Err(GameError::InvalidAction);
        };
        if !crate::game::combat_mission_policy::CombatMissionPolicy::starts_as_live_battle(
            active.node_type,
            active.mission_variant,
        ) {
            return Ok(());
        }
        let crate::game::battle::types::BattleUnitSource::Employee(profile) = &draft.source else {
            return Ok(());
        };
        if profile.basic_attack.defense_tile_range.is_none() {
            return Err(GameError::InvalidStaticData(
                "DefenseRoute player basic attack is missing defense_tile_range".to_string(),
            ));
        }
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
                if step.defense_tile_range.is_some() {
                    continue;
                }
                return Err(GameError::InvalidStaticData(format!(
                    "DefenseRoute player skill '{}' step '{}' is missing defense_tile_range",
                    skill_id, step.id
                )));
            }
        }
        Ok(())
    }

    pub(super) fn handle_withdraw_unit(
        &mut self,
        employee_uuid: Uuid,
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
        let outcome = active
            .battle
            .apply_live_command(BattleLiveCommand::WithdrawUnit { unit_id })?;
        if !matches!(outcome, BattleLiveCommandOutcome::UnitWithdrawn { .. }) {
            return Err(GameError::InvalidAction);
        }
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
                    .saturating_add(deployment.redeploy_cooldown_ms),
                deploy_cost,
            },
        );
        let timeline_delta = active.drain_event_log_delta();
        let deployment = active
            .live_deployment_dto(&roster)
            .ok_or(GameError::InvalidAction)?;
        Ok(BehaviorResult::BattleUnitWithdrawn {
            battle_uuid: active.battle_uuid,
            encounter_id: active.encounter_id.clone(),
            node_type: active.node_type,
            mission_variant: active.mission_variant,
            playback: active.playback_state(),
            employee_uuid,
            unit_instance_id: unit_id,
            timeline_delta,
            last_timeline_seq: active.last_event_log_seq(),
            deployment,
        })
    }

    pub(super) fn handle_activate_skill(
        &mut self,
        employee_uuid: Uuid,
        skill_id: crate::game::ability::SkillId,
        target: Option<crate::game::battle::timeline::SkillCastTarget>,
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
        let outcome = active
            .battle
            .apply_live_command(BattleLiveCommand::ActivateSkill {
                unit_id,
                skill_id: skill_id.clone(),
                target,
                time_ms,
            })?;
        if !matches!(outcome, BattleLiveCommandOutcome::SkillActivated { .. }) {
            return Err(GameError::InvalidAction);
        }
        let timeline_delta = active.drain_event_log_delta();
        let deployment = active
            .live_deployment_dto(&roster)
            .ok_or(GameError::InvalidAction)?;
        Ok(BehaviorResult::BattleSkillActivated {
            battle_uuid: active.battle_uuid,
            encounter_id: active.encounter_id.clone(),
            node_type: active.node_type,
            mission_variant: active.mission_variant,
            playback: active.playback_state(),
            employee_uuid,
            unit_instance_id: unit_id,
            skill_id,
            timeline_delta,
            last_timeline_seq: active.last_event_log_seq(),
            deployment,
        })
    }

    pub(super) fn handle_retreat_battle(&mut self) -> Result<BehaviorResult, GameError> {
        let (node_id, combat_preview, attempts_exhausted) = {
            let Some(active) = self.state.active_battle.as_ref() else {
                return Err(GameError::InvalidAction);
            };
            if active.node_type == CombatNodeType::Boss {
                return Err(GameError::InvalidAction);
            }
            let node_id = MapNodeId(active.battle_uuid);
            let attempts_exhausted = self
                .run_state()?
                .abnormality_attempt_state(node_id)
                .is_exhausted();
            (node_id, active.combat_preview.clone(), attempts_exhausted)
        };

        if !attempts_exhausted {
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
            self.state.active_battle = None;
            self.state.active_node_content = None;
            self.state.node_session = Some(session.clone());
            self.transition_to(GameState::NodeConfirm {
                node_id,
                kind_id: kind_id.clone(),
                category,
            })?;
            return Ok(BehaviorResult::NodePreview {
                node_id,
                kind_id,
                category,
                payload,
                session,
                map: self.current_map_view()?,
                combat_preview: Some(combat_preview),
            });
        }

        let battle = {
            let active = self
                .state
                .active_battle
                .as_ref()
                .ok_or(GameError::InvalidAction)?;
            Self::combat_battle_state_from_live_result(
                active,
                BattleWinner::Draw,
                active.battle.timeline.clone(),
                Vec::new(),
            )
        };
        self.state.active_battle = None;
        QliphothManager::apply_suppress_failure(&mut self.state.qliphoth);
        let outcome = self.combat_node_outcome_summary_with_resolution(
            &battle,
            false,
            BattleWinner::Draw,
            true,
            Vec::new(),
            InventoryDiffDto::default(),
        )?;
        self.decrement_consumables_after_combat_node()?;
        let completion = self.handle_complete_node()?;
        Ok(match completion {
            BehaviorResult::NodeCompleted { map, .. } => BehaviorResult::NodeCompleted {
                map,
                outcome: Some(outcome),
            },
            other => other,
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

        match battle.winner {
            BattleWinner::Player => {
                QliphothManager::apply_suppress_success(&mut self.state.qliphoth)
            }
            BattleWinner::Opponent | BattleWinner::Draw => {
                QliphothManager::apply_suppress_failure(&mut self.state.qliphoth)
            }
        }
        let employee_before = self.employee_outcome_before(&battle.participant_results)?;
        self.apply_post_battle_resolution(&battle.participant_results)?;
        self.decrement_consumables_after_combat_node()?;

        if battle.node_type == CombatNodeType::Boss && battle.winner != BattleWinner::Player {
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
            let employee_changes =
                self.employee_outcome_changes(employee_before, &battle.participant_results)?;
            info!(
                "Combat result finished without player victory (winner={:?}); skipping rewards",
                battle.winner
            );
            if self.state.node_session.is_some() {
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
                            && !self.has_available_medical_support_node()?
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

        if self.state.node_session.is_none() {
            return Err(GameError::InvalidAction);
        }

        let reward_session = self.build_reward_session(
            battle.abnormality_uuid,
            battle.reward_mode,
            battle.rewards.clone(),
            false,
        );
        let experience_reward_amount = self.combat_experience_reward_amount(&reward_session)?;
        let (enkephalin, inventory_diff) = self.apply_reward_session(&reward_session)?;
        self.apply_combat_experience_reward(&battle.participant_results, experience_reward_amount)?;
        let employee_changes =
            self.employee_outcome_changes(employee_before, &battle.participant_results)?;
        let outcome = self.combat_node_outcome_summary(
            &battle,
            true,
            employee_changes,
            inventory_diff.clone(),
        )?;
        let completion = self.handle_complete_node()?;
        Ok(BehaviorResult::CombatRewardsGranted {
            enkephalin,
            inventory_diff,
            outcome,
            completion: Box::new(completion),
        })
    }
}
