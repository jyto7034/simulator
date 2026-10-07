use crate::game::behavior::{
    ActionCapabilityDto, ActionDisabledReasonCode, ActionKind, BehaviorResult, GameError,
    MasteredEliteResolutionDto, MasteredEliteSweepResultDto, MasteredEliteSweepRewardBranchDto,
};
use crate::game::data::pve_data::PveEncounterClass;
use crate::game::encounter_progression::{
    EncounterProgressionState, MasteredEliteSweepRewardBranch,
};
use crate::game::enums::RewardMode;
use crate::game::map::{GameMode, MapNodeCategory, MapNodeId, MapNodePayload, MapNodeState};
use crate::game::resources::GameState;
use crate::game::reward::{
    EmployeeExperienceDiffDto, ExperienceTargetPolicy, GrantExecutionContext, RewardEffect,
    SkillFragmentGrantDiffDto,
};

use super::grant_state::EconomyGrantState;
use super::node_flow::CombatResultNodeCompletion;
use super::GameCore;

#[derive(Clone)]
pub(super) struct MasteredEliteSweepCommitPlan {
    pub(super) result: MasteredEliteSweepResultDto,
    pub(super) grant_state: EconomyGrantState,
    pub(super) encounter_progression_after: EncounterProgressionState,
    pub(super) node_completion: CombatResultNodeCompletion,
}

#[derive(Clone)]
pub(super) struct MasteredEliteResolutionEvaluation {
    pub(super) node_id: MapNodeId,
    pub(super) encounter_id: String,
    pub(super) reward_source_id: String,
    pub(super) dto: MasteredEliteResolutionDto,
}

impl GameCore {
    pub(super) fn mastered_elite_resolution_evaluation(
        &self,
    ) -> Option<MasteredEliteResolutionEvaluation> {
        let GameState::NodeConfirm {
            node_id, category, ..
        } = self.state.lifecycle.phase()
        else {
            return None;
        };
        let node_id = *node_id;
        let category = *category;
        let run = self.state.run.as_ref()?;
        if run.run_progression.game_mode != GameMode::Endless || category != MapNodeCategory::Combat
        {
            return None;
        }
        let node = run.map.node(node_id)?;
        if node.state != MapNodeState::Available || node.category != MapNodeCategory::Combat {
            return None;
        }
        let MapNodePayload::Encounter {
            encounter_id: Some(encounter_id),
        } = &node.payload
        else {
            return None;
        };
        let encounter = self.game_data.pve_data.get_by_id(encounter_id)?;
        if encounter.encounter_class != PveEncounterClass::Elite {
            return None;
        }
        let abnormality_id = encounter.primary_abnormality_id()?;
        if !run
            .monolith
            .progress(abnormality_id)
            .is_some_and(|progress| progress.victories == 3)
        {
            return None;
        }
        let source = self
            .game_data
            .pve_data
            .get_reward_source_by_encounter_id(encounter_id)?;

        let has_deployable_employee = self
            .state
            .roster
            .iter()
            .any(|employee| employee.is_battle_deployable());
        let direct_battle = if has_deployable_employee {
            crate::game::behavior::ActionCapabilityDto::enabled(
                ActionKind::ConfirmEnterNode,
                ActionKind::ConfirmEnterNode.scope(),
            )
        } else {
            ActionCapabilityDto::disabled(
                ActionKind::ConfirmEnterNode,
                ActionKind::ConfirmEnterNode.scope(),
                ActionDisabledReasonCode::NoEligiblePayload,
            )
        };
        let has_exact_session = self.state.node_session.as_ref().is_some_and(|session| {
            session.node_id == node_id
                && session.category == MapNodeCategory::Combat
                && session.payload == node.payload
        });
        let source_progression_valid = run
            .completed_reward_source_ids
            .contains(&source.reward_source_id)
            && run
                .mastered_elite_cycle
                .assigned_source_ids
                .iter()
                .any(|assigned| assigned == &source.reward_source_id);
        let sweep = if !has_exact_session || !source_progression_valid {
            ActionCapabilityDto::disabled(
                ActionKind::SweepMasteredElite,
                ActionKind::SweepMasteredElite.scope(),
                ActionDisabledReasonCode::ProgressionUnavailable,
            )
        } else if !has_deployable_employee {
            ActionCapabilityDto::disabled(
                ActionKind::SweepMasteredElite,
                ActionKind::SweepMasteredElite.scope(),
                ActionDisabledReasonCode::NoEligiblePayload,
            )
        } else {
            ActionCapabilityDto::enabled(
                ActionKind::SweepMasteredElite,
                ActionKind::SweepMasteredElite.scope(),
            )
        };

        Some(MasteredEliteResolutionEvaluation {
            node_id,
            encounter_id: encounter_id.clone(),
            reward_source_id: source.reward_source_id.clone(),
            dto: MasteredEliteResolutionDto {
                direct_battle,
                sweep,
            },
        })
    }

    pub(super) fn mastered_elite_resolution_dto(&self) -> Option<MasteredEliteResolutionDto> {
        self.mastered_elite_resolution_evaluation()
            .map(|evaluation| evaluation.dto)
    }

    pub(super) fn handle_sweep_mastered_elite(&mut self) -> Result<BehaviorResult, GameError> {
        self.with_state_transaction(Self::handle_sweep_mastered_elite_inner)
    }

    fn handle_sweep_mastered_elite_inner(&mut self) -> Result<BehaviorResult, GameError> {
        let evaluation = self
            .mastered_elite_resolution_evaluation()
            .ok_or(GameError::InvalidAction)?;
        if !evaluation.dto.sweep.enabled {
            return Err(GameError::ActionUnavailable {
                action: ActionKind::SweepMasteredElite,
                reason: evaluation
                    .dto
                    .sweep
                    .disabled_reason_code
                    .unwrap_or(ActionDisabledReasonCode::ProgressionUnavailable),
            });
        }

        // Staging may use the existing node-completion planner, but it must leave the live
        // run exactly at NodeConfirm. Capture and restore the complete non-battle state after
        // the immutable commit plan has been built.
        let pristine_profile =
            crate::authoring::clone_shape::controlled_clone(&self.state.player_profile)?;
        let pristine = self
            .state
            .clone_non_battle_transaction_state()?
            .ok_or(GameError::InvalidAction)?;

        let original_boss_omen = self.run_state()?.boss_omen.clone();
        let mut map = self.run_state()?.map.clone();
        let mut progression = self.run_state()?.map_progression.clone();
        let template = self.facility_template_for_map(&map)?;
        progression
            .enter_node(&mut map, &template, evaluation.node_id)
            .map_err(|_| GameError::InvalidAction)?;
        {
            let run = self.run_state_mut()?;
            run.map = map;
            run.map_progression = progression;
        }

        let owned_fragment_count = {
            let source = self
                .game_data
                .pve_data
                .get_reward_source_by_encounter_id(&evaluation.encounter_id)
                .ok_or_else(|| {
                    GameError::InvalidStaticData(format!(
                        "mastered Elite encounter '{}' lost its reward source",
                        evaluation.encounter_id
                    ))
                })?;
            self.state.skill_fragments.count(&source.fragment_id)
        };
        let sweep_reward = self
            .run_state()?
            .encounter_progression
            .plan_mastered_elite_sweep_reward(
                &self.game_data.pve_data,
                &evaluation.encounter_id,
                owned_fragment_count,
            )
            .map_err(GameError::InvalidStaticData)?;
        if sweep_reward.reward_source_id != evaluation.reward_source_id {
            return Err(GameError::InvalidAction);
        }
        let mut encounter_progression_after = self.run_state()?.encounter_progression.clone();
        encounter_progression_after
            .commit_mastered_elite_sweep_reward(&sweep_reward)
            .map_err(GameError::InvalidStaticData)?;

        let (reward_mode, base_rewards) =
            self.resolved_sweep_base_rewards(evaluation.node_id, &evaluation.encounter_id)?;
        if reward_mode != RewardMode::ClaimAll {
            return Err(GameError::InvalidStaticData(format!(
                "mastered Elite sweep encounter '{}' must use ClaimAll base rewards",
                evaluation.encounter_id
            )));
        }

        let mut grant_state = EconomyGrantState::capture(&self.state)?;
        let sweep_uuid = grant_state.uuid_manager.next_mastered_elite_sweep_result();
        let (mut inventory_diff, mut skill_fragment_diffs, mut employee_experience_diffs) = self
            .apply_reward_options_with_context_to_state(
                sweep_uuid,
                &base_rewards,
                &GrantExecutionContext::default(),
                &mut grant_state,
            )?;
        let exclusive = grant_state.grant_effects(
            &self.game_data,
            &self.state.skill_fragment_policy,
            &GrantExecutionContext::default(),
            &sweep_reward.effects,
            self.run_seed ^ 0x5357_4545_505f_5843,
        )?;
        Self::merge_inventory_diff(&mut inventory_diff, exclusive.inventory_diff);
        skill_fragment_diffs.extend(exclusive.skill_fragment_diffs);
        employee_experience_diffs.extend(exclusive.employee_experience_diffs);

        let sweep_xp = self.run_policy().growth.post_battle_survival_xp;
        let mut living_employee_ids = grant_state
            .roster
            .iter()
            .filter(|employee| employee.is_alive())
            .map(|employee| employee.uuid)
            .collect::<Vec<_>>();
        living_employee_ids.sort_by_key(|uuid| uuid.as_u128());
        if sweep_xp > 0 {
            for employee_uuid in living_employee_ids {
                let granted = grant_state.grant_effects(
                    &self.game_data,
                    &self.state.skill_fragment_policy,
                    &GrantExecutionContext {
                        selected_employee_id: Some(employee_uuid),
                        ..GrantExecutionContext::default()
                    },
                    &[RewardEffect::GrantExperience {
                        amount: sweep_xp,
                        target: ExperienceTargetPolicy::SelectedEmployee,
                    }],
                    self.run_seed ^ 0x5357_4545_505f_5850,
                )?;
                Self::merge_inventory_diff(&mut inventory_diff, granted.inventory_diff);
                skill_fragment_diffs.extend(granted.skill_fragment_diffs);
                employee_experience_diffs.extend(granted.employee_experience_diffs);
            }
        }
        Self::decrement_consumables_after_combat_node_in_roster(&mut grant_state.roster);

        let mut node_completion = CombatResultNodeCompletion::try_from(
            self.plan_complete_current_node_with_encounter_progression(
                false,
                Some(&encounter_progression_after),
            )?,
        )?;
        match &mut node_completion {
            CombatResultNodeCompletion::NodeCompleted { boss_omen, .. } => {
                // Sweep is not a BossOmen condition event. Preserve its ledger exactly while
                // still reusing the authoritative map/session completion commit.
                *boss_omen = Some(original_boss_omen);
            }
            CombatResultNodeCompletion::FloorAdvanced { .. }
            | CombatResultNodeCompletion::RunComplete { .. } => {
                return Err(GameError::InvalidStaticData(
                    "a mastered Elite sweep cannot resolve a terminal node".to_string(),
                ));
            }
        }

        let result = MasteredEliteSweepResultDto {
            sweep_uuid,
            node_id: evaluation.node_id,
            encounter_id: evaluation.encounter_id,
            reward_source_id: sweep_reward.reward_source_id,
            fragment_id: sweep_reward.fragment_id,
            reward_branch: match sweep_reward.branch {
                MasteredEliteSweepRewardBranch::ReacquireFragment => {
                    MasteredEliteSweepRewardBranchDto::ReacquireFragment
                }
                MasteredEliteSweepRewardBranch::RepeatReward => {
                    MasteredEliteSweepRewardBranchDto::RepeatReward
                }
            },
            base_rewards,
            exclusive_effects: sweep_reward.effects,
            enkephalin_after: grant_state.enkephalin.amount,
            inventory_diff,
            skill_fragment_diffs,
            employee_experience_diffs,
        };
        let plan = MasteredEliteSweepCommitPlan {
            result: result.clone(),
            grant_state,
            encounter_progression_after,
            node_completion,
        };

        self.state = pristine;
        self.state.player_profile = pristine_profile;
        self.state.pending_mastered_elite_sweep = Some(plan);
        self.transition_to(GameState::MasteredEliteSweepResult { sweep_uuid })?;
        Ok(BehaviorResult::MasteredEliteSweepResultStaged { result })
    }

    pub(super) fn handle_complete_mastered_elite_sweep(
        &mut self,
    ) -> Result<BehaviorResult, GameError> {
        self.with_state_transaction(Self::handle_complete_mastered_elite_sweep_inner)
    }

    fn handle_complete_mastered_elite_sweep_inner(&mut self) -> Result<BehaviorResult, GameError> {
        let sweep_uuid = match self.state.lifecycle.phase() {
            GameState::MasteredEliteSweepResult { sweep_uuid } => *sweep_uuid,
            _ => return Err(GameError::InvalidAction),
        };
        let plan = self
            .state
            .pending_mastered_elite_sweep
            .take()
            .ok_or(GameError::InvalidAction)?;
        let session_is_current = self.state.node_session.as_ref().is_some_and(|session| {
            session.node_id == plan.result.node_id
                && session.category == MapNodeCategory::Combat
                && matches!(
                    &session.payload,
                    MapNodePayload::Encounter {
                        encounter_id: Some(encounter_id),
                    } if encounter_id == &plan.result.encounter_id
                )
        });
        let run_state_is_current = self.state.run.as_ref().is_some_and(|run| {
            let node_is_current = run.map.node(plan.result.node_id).is_some_and(|node| {
                node.state == MapNodeState::Available
                    && node.category == MapNodeCategory::Combat
                    && matches!(
                        &node.payload,
                        MapNodePayload::Encounter {
                            encounter_id: Some(encounter_id),
                        } if encounter_id == &plan.result.encounter_id
                    )
            });
            let reward_source_is_current = run
                .completed_reward_source_ids
                .contains(&plan.result.reward_source_id)
                && run
                    .mastered_elite_cycle
                    .assigned_source_ids
                    .iter()
                    .any(|assigned| assigned == &plan.result.reward_source_id);
            let mastery_is_current = self
                .game_data
                .pve_data
                .get_by_id(&plan.result.encounter_id)
                .and_then(|encounter| encounter.primary_abnormality_id())
                .and_then(|abnormality_id| run.monolith.progress(abnormality_id))
                .is_some_and(|progress| progress.victories == 3);
            node_is_current && reward_source_is_current && mastery_is_current
        });
        let has_deployable_employee = self
            .state
            .roster
            .iter()
            .any(|employee| employee.is_battle_deployable());
        if plan.result.sweep_uuid != sweep_uuid
            || !session_is_current
            || !run_state_is_current
            || !has_deployable_employee
        {
            return Err(GameError::InvalidAction);
        }

        let enkephalin = plan.result.enkephalin_after;
        let inventory_diff = plan.result.inventory_diff.clone();
        let skill_fragment_diffs: Vec<SkillFragmentGrantDiffDto> =
            plan.result.skill_fragment_diffs.clone();
        let employee_experience_diffs: Vec<EmployeeExperienceDiffDto> =
            plan.result.employee_experience_diffs.clone();
        let player_profile = self.state.player_profile.clone();
        plan.grant_state.commit_into(&mut self.state);
        self.run_state_mut()?.encounter_progression = plan.encounter_progression_after;
        let completion = self.commit_combat_result_node_completion(plan.node_completion)?;
        // Ordinary Endless node completion opportunistically refreshes the permanent
        // all-FinalBoss aggregate. A sweep is explicitly run-local and cannot publish a
        // profile mutation, so preserve the exact pre-confirmation profile.
        self.state.player_profile = player_profile;

        Ok(BehaviorResult::MasteredEliteSweepCompleted {
            sweep_uuid,
            enkephalin,
            inventory_diff,
            skill_fragment_diffs,
            employee_experience_diffs,
            completion: Box::new(completion),
        })
    }

    pub(super) fn staged_mastered_elite_sweep_result(
        &self,
    ) -> Option<&MasteredEliteSweepResultDto> {
        self.state
            .pending_mastered_elite_sweep
            .as_ref()
            .map(|plan| &plan.result)
    }
}

crate::authoring::clone_shape::struct_shape!(MasteredEliteSweepCommitPlan {
    result,
    grant_state,
    encounter_progression_after,
    node_completion
});
