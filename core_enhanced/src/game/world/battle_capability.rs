use super::GameCore;
use crate::game::behavior::{
    ActionCapabilityDto, ActionDisabledReasonCode, ActionKind, GameError, PlayerBehavior,
    VisualNovelSnapshotDto,
};
use crate::game::resources::GameState;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BattleActionEvaluation {
    Irrelevant(ActionDisabledReasonCode),
    Disabled(ActionDisabledReasonCode),
    Enabled,
}

impl GameCore {
    fn evaluate_battle_context_actions(
        &self,
        visual_novel: Option<&VisualNovelSnapshotDto>,
    ) -> Result<Option<Vec<(ActionKind, BattleActionEvaluation)>>, GameError> {
        if !matches!(self.state.game_state, GameState::InBattle { .. }) {
            return Ok(None);
        }

        let active = self
            .state
            .active_battle
            .as_ref()
            .ok_or(GameError::InvalidAction)?;
        let finished = active.execution.is_finished();
        let blocking_visual_novel = visual_novel.is_some_and(|session| session.blocking);
        let deployment = (!finished && !blocking_visual_novel)
            .then(|| active.live_deployment_dto(active.employee_roster()))
            .flatten();

        let ordinary = |evaluation| {
            if finished || blocking_visual_novel {
                BattleActionEvaluation::Irrelevant(ActionDisabledReasonCode::ContextIrrelevant)
            } else {
                evaluation
            }
        };
        let deployment_action = |evaluation| match deployment.as_ref() {
            Some(_) => ordinary(evaluation),
            None => BattleActionEvaluation::Irrelevant(ActionDisabledReasonCode::ContextIrrelevant),
        };

        let any_deployable = deployment.as_ref().is_some_and(|state| {
            state
                .unit_deploy_costs
                .iter()
                .any(|candidate| candidate.eligibility.enabled)
        });
        let deploy = deployment_action(if any_deployable {
            BattleActionEvaluation::Enabled
        } else {
            BattleActionEvaluation::Disabled(ActionDisabledReasonCode::NoEligiblePayload)
        });
        let withdraw = deployment_action(
            if deployment
                .as_ref()
                .is_some_and(|state| !state.deployed_units.is_empty())
            {
                BattleActionEvaluation::Enabled
            } else {
                BattleActionEvaluation::Disabled(ActionDisabledReasonCode::NoEligiblePayload)
            },
        );
        let activate_skill = deployment_action(
            if deployment.as_ref().is_some_and(|state| {
                state
                    .deployed_units
                    .iter()
                    .any(|unit| unit.skill_readiness.eligibility.enabled)
            }) {
                BattleActionEvaluation::Enabled
            } else {
                BattleActionEvaluation::Disabled(ActionDisabledReasonCode::NoEligiblePayload)
            },
        );

        let (advance_visual_novel, select_visual_novel_choice) = if finished {
            (
                BattleActionEvaluation::Irrelevant(ActionDisabledReasonCode::ContextIrrelevant),
                BattleActionEvaluation::Irrelevant(ActionDisabledReasonCode::ContextIrrelevant),
            )
        } else {
            match visual_novel {
                Some(session) if session.choices.is_empty() => (
                    BattleActionEvaluation::Enabled,
                    BattleActionEvaluation::Irrelevant(ActionDisabledReasonCode::ContextIrrelevant),
                ),
                Some(_) => (
                    BattleActionEvaluation::Irrelevant(ActionDisabledReasonCode::ContextIrrelevant),
                    BattleActionEvaluation::Enabled,
                ),
                None => (
                    BattleActionEvaluation::Irrelevant(ActionDisabledReasonCode::ContextIrrelevant),
                    BattleActionEvaluation::Irrelevant(ActionDisabledReasonCode::ContextIrrelevant),
                ),
            }
        };

        Ok(Some(vec![
            (
                ActionKind::RequestBattleState,
                if finished {
                    BattleActionEvaluation::Irrelevant(ActionDisabledReasonCode::ContextIrrelevant)
                } else {
                    BattleActionEvaluation::Enabled
                },
            ),
            (
                ActionKind::RecoverBattleSetupLoss,
                if finished {
                    BattleActionEvaluation::Irrelevant(ActionDisabledReasonCode::ContextIrrelevant)
                } else {
                    BattleActionEvaluation::Enabled
                },
            ),
            (
                ActionKind::RetreatBattle,
                ordinary(BattleActionEvaluation::Enabled),
            ),
            (
                ActionKind::PauseBattle,
                ordinary(if active.playback.paused {
                    BattleActionEvaluation::Irrelevant(ActionDisabledReasonCode::AlreadyActive)
                } else {
                    BattleActionEvaluation::Enabled
                }),
            ),
            (
                ActionKind::ResumeBattle,
                ordinary(if active.playback.paused {
                    BattleActionEvaluation::Enabled
                } else {
                    BattleActionEvaluation::Irrelevant(ActionDisabledReasonCode::AlreadyActive)
                }),
            ),
            (
                ActionKind::SetBattleSpeed,
                ordinary(BattleActionEvaluation::Enabled),
            ),
            (ActionKind::DeployUnit, deploy),
            (ActionKind::RequestDeploymentRangePreview, deploy),
            (ActionKind::WithdrawUnit, withdraw),
            (ActionKind::ActivateSkill, activate_skill),
            (ActionKind::AdvanceVisualNovel, advance_visual_novel),
            (
                ActionKind::SelectVisualNovelChoice,
                select_visual_novel_choice,
            ),
        ]))
    }

    pub(super) fn project_battle_action_capabilities(
        &self,
        visual_novel: Option<&VisualNovelSnapshotDto>,
    ) -> Result<Vec<ActionCapabilityDto>, GameError> {
        let evaluations = self
            .evaluate_battle_context_actions(visual_novel)?
            .ok_or(GameError::InvalidAction)?;
        Ok(evaluations
            .into_iter()
            .filter_map(|(action, evaluation)| match evaluation {
                BattleActionEvaluation::Irrelevant(_) => None,
                BattleActionEvaluation::Disabled(reason) => Some(ActionCapabilityDto::disabled(
                    action,
                    action.scope(),
                    reason,
                )),
                BattleActionEvaluation::Enabled => {
                    Some(ActionCapabilityDto::enabled(action, action.scope()))
                }
            })
            .collect())
    }

    /// Returns true when the behavior belongs to the battle context capability slice.
    pub(super) fn preflight_battle_capability_behavior(
        &self,
        behavior: &PlayerBehavior,
    ) -> Result<bool, GameError> {
        let visual_novel = self.visual_novel_snapshot_dto()?;
        let Some(evaluations) = self.evaluate_battle_context_actions(visual_novel.as_ref())? else {
            return Ok(false);
        };
        let action = behavior.kind();
        let Some((_, evaluation)) = evaluations
            .into_iter()
            .find(|(candidate, _)| *candidate == action)
        else {
            return Ok(false);
        };

        match evaluation {
            BattleActionEvaluation::Enabled => Ok(true),
            // A context family can be disabled because no concrete payload is currently enabled.
            // The owning domain preflight still revalidates the submitted payload and returns its
            // precise typed reason (for example cooldown, resources, or target eligibility).
            BattleActionEvaluation::Disabled(_) => Ok(true),
            BattleActionEvaluation::Irrelevant(reason) => {
                Err(GameError::ActionUnavailable { action, reason })
            }
        }
    }
}
