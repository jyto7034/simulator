use std::collections::{BTreeSet, HashMap};

use tracing::info;
use uuid::Uuid;

use super::{GameCore, RUN_SYSTEM_POLICY};
use crate::game::battle::types::{BattleWinner, ParticipantBattleResult};
use crate::game::behavior::{
    BehaviorResult, CombatOutcomeSummary, GameError, NodeOutcomeEmployeeChange, NodeOutcomeSummary,
};
use crate::game::combat_preview::{
    CombatDeployment, CombatNodeType, CombatPreview, DeploymentZoneKind,
};
use crate::game::employee::{EmployeeInjury, EmployeeLifeState};
use crate::game::employee_trust::EmployeeTrustResolver;
use crate::game::enums::Side;
use crate::game::events::combat::CombatExecutor;
use crate::game::managers::qliphoth_manager::QliphothManager;
use crate::game::map::{MapNodeCategory, MapNodeId, MapNodePayload, SupportNodeMode};
use crate::game::resources::{
    CombatBattleState, GameState, InventoryDiffDto, Position, RewardSessionState, RunFailureReason,
    SelectedEvent, SelectedEventState,
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
        reveal_recon: bool,
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
            if !reveal_recon || existing.recon_revealed {
                return Ok(Some(existing));
            }
        }

        let seed = self.node_seed(node_id, 0x5245_434F_4E); // "RECON"
        let preview = CombatPreview::try_generate_for_node(
            node_id,
            category,
            encounter_id.as_deref(),
            self.game_data.as_ref(),
            reveal_recon,
            seed,
        )?;
        self.run_state_mut()?
            .combat_previews
            .insert(node_id, preview.clone());
        Ok(Some(preview))
    }

    pub(super) fn combat_deployment_for_node(
        &mut self,
        node_id: MapNodeId,
    ) -> Result<Option<CombatDeployment>, GameError> {
        if self.combat_preview_for_node(node_id, false)?.is_none() {
            return Ok(None);
        }
        let run = self.run_state_mut()?;
        let deployment = run
            .combat_deployments
            .entry(node_id)
            .or_insert_with(|| CombatDeployment::new(node_id))
            .clone();
        Ok(Some(deployment))
    }

    fn combat_deployment_zone_kind_for_cell(
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
        let zone_kind = Self::combat_deployment_zone_kind_for_cell(combat_preview, position)
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

    pub(super) fn handle_move_combat_deployment_unit(
        &mut self,
        node_id: MapNodeId,
        target_unit_uuid: Uuid,
        dest_pos: Position,
        swap_with_unit_uuid: Option<Uuid>,
    ) -> Result<BehaviorResult, GameError> {
        self.validate_owned_unit_exists(target_unit_uuid)?;
        if let Some(swap_unit_uuid) = swap_with_unit_uuid {
            self.validate_owned_unit_exists(swap_unit_uuid)?;
        }

        let combat_preview = self
            .combat_preview_for_node(node_id, false)?
            .ok_or(GameError::InvalidAction)?;
        self.validate_employee_deployment_cell(&combat_preview, target_unit_uuid, dest_pos)?;

        let source_position = self
            .run_state()?
            .combat_deployments
            .get(&node_id)
            .and_then(|deployment| deployment.position_of(target_unit_uuid));
        let occupant_uuid = self
            .run_state()?
            .combat_deployments
            .get(&node_id)
            .and_then(|deployment| deployment.employee_at(dest_pos));
        if let (Some(swap_unit_uuid), Some(source_position)) =
            (swap_with_unit_uuid, source_position)
        {
            if occupant_uuid == Some(swap_unit_uuid) {
                self.validate_employee_deployment_cell(
                    &combat_preview,
                    swap_unit_uuid,
                    source_position,
                )?;
            }
        }

        let deployment = self
            .run_state_mut()?
            .combat_deployments
            .entry(node_id)
            .or_insert_with(|| CombatDeployment::new(node_id));
        deployment.place(target_unit_uuid, dest_pos, swap_with_unit_uuid)?;
        Ok(BehaviorResult::MoveUnit {
            combat_deployment: deployment.clone(),
        })
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
        let deployment_positions = self
            .run_state()?
            .combat_deployments
            .get(&node_id)
            .map(|deployment| deployment.positions())
            .unwrap_or_default();
        if deployment_positions.is_empty() {
            return Err(GameError::InvalidAction);
        }

        let selected_uuid = node_id.0;
        info!(
            "Starting map combat node for abnormality={} encounter={} node={}",
            abnormality_id, encounter_id, selected_uuid
        );

        let (battle_result, node_type) = {
            let combat_preview = self
                .combat_preview_for_node(node_id, false)?
                .ok_or(GameError::InvalidAction)?;
            let node_type = combat_preview.node_type;
            let inventory = self.inventory()?;
            let roster = self.roster()?;
            let skill_fragments = &self.state.skill_fragments;
            let battle_result = CombatExecutor::start_battle_with_combat_preview(
                roster,
                inventory,
                skill_fragments,
                self.game_data.clone(),
                &abnormality_id,
                &encounter_id,
                self.run_seed,
                &combat_preview,
                &deployment_positions,
            )?;
            (battle_result, node_type)
        };

        let (reward_mode, rewards) = CombatExecutor::resolve_rewards_for_node_type(
            self.game_data.as_ref(),
            &encounter_id,
            node_type,
        )
        .map_err(|_| {
            GameError::InvalidStaticData(format!(
                "combat encounter '{}' has invalid reward configuration for {:?}",
                encounter_id, node_type
            ))
        })?;
        let winner = battle_result.winner;
        let timeline = battle_result.timeline;
        let participant_results = battle_result.participant_results;

        self.state.selected_event = Some(SelectedEvent::new(SelectedEventState::CombatBattle(
            CombatBattleState {
                abnormality_id,
                encounter_id,
                node_type,
                abnormality_uuid: selected_uuid,
                winner,
                timeline: timeline.clone(),
                reward_mode,
                rewards,
                participant_results,
            },
        )));
        self.transition_to(GameState::InCombatReplay {
            battle_uuid: selected_uuid,
        })?;

        Ok(BehaviorResult::CombatResolved { winner, timeline })
    }

    pub(super) fn ensure_combat_deployment_ready(
        &mut self,
        node_id: MapNodeId,
    ) -> Result<(), GameError> {
        let placements = self
            .run_state()?
            .combat_deployments
            .get(&node_id)
            .ok_or(GameError::InvalidAction)?
            .placements
            .clone();
        if placements.is_empty() {
            return Err(GameError::InvalidAction);
        }

        let combat_preview = self
            .combat_preview_for_node(node_id, false)?
            .ok_or(GameError::InvalidAction)?;
        for placement in placements {
            self.validate_owned_unit_exists(placement.employee_uuid)?;
            self.validate_employee_deployment_cell(
                &combat_preview,
                placement.employee_uuid,
                placement.position,
            )?;
        }

        Ok(())
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
                    RUN_SYSTEM_POLICY.post_battle_incapacitation_trauma,
                    &trust_policy,
                );
                employee.trust.apply_reaction(&trauma_outcome.reaction);
                employee.apply_incapacitation(
                    trauma_outcome.final_amount,
                    RUN_SYSTEM_POLICY.post_battle_incapacitation_run_hp_loss_percent,
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
                employee.add_experience(RUN_SYSTEM_POLICY.post_battle_survival_xp);
            } else {
                employee.apply_incapacitation(
                    RUN_SYSTEM_POLICY.post_battle_incapacitation_trauma,
                    RUN_SYSTEM_POLICY.post_battle_incapacitation_run_hp_loss_percent,
                    EmployeeInjury {
                        id: "battle_loss".to_string(),
                        severity: 1,
                    },
                );
            }
        }

        let _ = roster;
        self.sync_bench_with_owned_units()?;
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

    fn has_available_recovery_map_node(&self) -> Result<bool, GameError> {
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
                if self.has_available_recovery_map_node()? {
                    Err(GameError::InvalidAction)
                } else {
                    self.fail_run(RunFailureReason::NoDeployableEmployees)
                        .map(Some)
                }
            }
            Some(RunFailureReason::CombatTeamUnavailable) => self
                .fail_run(RunFailureReason::CombatTeamUnavailable)
                .map(Some),
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
        self.state.selected_event = None;
        self.transition_to(GameState::RunFailed { reason })?;
        Ok(BehaviorResult::RunFailed { reason, outcome })
    }

    fn fail_run_if_needed(&mut self) -> Result<Option<BehaviorResult>, GameError> {
        match self.current_run_failure_reason()? {
            Some(RunFailureReason::NoLivingEmployees) => {
                self.fail_run(RunFailureReason::NoLivingEmployees).map(Some)
            }
            Some(RunFailureReason::NoDeployableEmployees) => {
                if self.has_available_recovery_map_node()? {
                    Ok(None)
                } else {
                    self.fail_run(RunFailureReason::NoDeployableEmployees)
                        .map(Some)
                }
            }
            Some(RunFailureReason::CombatTeamUnavailable) => self
                .fail_run(RunFailureReason::CombatTeamUnavailable)
                .map(Some),
            Some(RunFailureReason::BossDefeated) => {
                self.fail_run(RunFailureReason::BossDefeated).map(Some)
            }
            None => Ok(None),
        }
    }

    pub(super) fn handle_finish_combat_replay(&mut self) -> Result<BehaviorResult, GameError> {
        let battle = {
            let selected = self
                .state
                .selected_event
                .as_ref()
                .ok_or(GameError::InvalidAction)?;
            selected.as_combat_battle()?.clone()
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
                "Suppression replay finished without player victory (winner={:?}); skipping rewards",
                battle.winner
            );
            if self.state.node_session.is_some() {
                let failure_reason = self.current_run_failure_reason()?;
                match failure_reason {
                    Some(
                        reason @ (RunFailureReason::NoLivingEmployees
                        | RunFailureReason::CombatTeamUnavailable
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
                            && !self.has_available_recovery_map_node()?
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

    pub(super) fn handle_retreat_combat(&mut self) -> Result<BehaviorResult, GameError> {
        let battle = {
            let selected = self
                .state
                .selected_event
                .as_ref()
                .ok_or(GameError::InvalidAction)?;
            selected.as_combat_battle()?.clone()
        };

        if battle.node_type == CombatNodeType::Boss {
            return Err(GameError::InvalidAction);
        }
        if self.state.node_session.is_none() {
            return Err(GameError::InvalidAction);
        }

        QliphothManager::apply_suppress_failure(&mut self.state.qliphoth);
        let outcome = self.combat_node_outcome_summary_with_resolution(
            &battle,
            false,
            BattleWinner::Draw,
            true,
            Vec::new(),
            InventoryDiffDto::default(),
        )?;
        let completion = self.handle_complete_node()?;
        Ok(match completion {
            BehaviorResult::NodeCompleted { map, .. } => BehaviorResult::NodeCompleted {
                map,
                outcome: Some(outcome),
            },
            other => other,
        })
    }
}
