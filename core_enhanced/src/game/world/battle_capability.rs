use super::GameCore;
use crate::game::ability::SkillCastSelection;
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
        if !matches!(self.state.lifecycle.phase(), GameState::InBattle { .. }) {
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
                state.deployed_units.iter().any(|unit| {
                    if !unit.skill_readiness.eligibility.enabled {
                        return false;
                    }
                    match unit.skill_readiness.selection {
                        Some(SkillCastSelection::PlayerUnit | SkillCastSelection::PlayerTile) => {
                            active.tactical_focus.as_ref().is_some_and(|focus| {
                                focus.caster_unit_instance_id == unit.unit_instance_id
                                    && unit.skill_readiness.skill_id.as_ref()
                                        == Some(&focus.skill_id)
                            })
                        }
                        _ => active.tactical_focus.is_none(),
                    }
                })
            }) {
                BattleActionEvaluation::Enabled
            } else {
                BattleActionEvaluation::Disabled(ActionDisabledReasonCode::NoEligiblePayload)
            },
        );
        let begin_skill_targeting = deployment_action(
            if active.tactical_focus.is_none()
                && deployment.as_ref().is_some_and(|state| {
                    state.deployed_units.iter().any(|unit| {
                        unit.skill_readiness.eligibility.enabled
                            && matches!(
                                unit.skill_readiness.selection,
                                Some(
                                    SkillCastSelection::PlayerUnit | SkillCastSelection::PlayerTile
                                )
                            )
                    })
                })
            {
                BattleActionEvaluation::Enabled
            } else {
                BattleActionEvaluation::Disabled(ActionDisabledReasonCode::NoEligiblePayload)
            },
        );
        let cancel_skill_targeting = deployment_action(if active.tactical_focus.is_some() {
            BattleActionEvaluation::Enabled
        } else {
            BattleActionEvaluation::Irrelevant(ActionDisabledReasonCode::ContextIrrelevant)
        });
        let preview_skill_target = cancel_skill_targeting;
        let set_engagement_stance = deployment_action(
            if deployment.as_ref().is_some_and(|state| {
                state.deployed_units.iter().any(|unit| {
                    unit.tactical_state.stance_eligibility.attack.enabled
                        || unit.tactical_state.stance_eligibility.defense.enabled
                })
            }) {
                BattleActionEvaluation::Enabled
            } else {
                BattleActionEvaluation::Disabled(ActionDisabledReasonCode::NoEligiblePayload)
            },
        );
        let tactical_relocate = deployment_action(
            if deployment.as_ref().is_some_and(|state| {
                state
                    .deployed_units
                    .iter()
                    .any(|unit| unit.tactical_state.relocation_eligibility.enabled)
            }) {
                BattleActionEvaluation::Enabled
            } else {
                BattleActionEvaluation::Disabled(ActionDisabledReasonCode::NoEligiblePayload)
            },
        );
        let tactical_swap = deployment_action(
            if deployment.as_ref().is_some_and(|state| {
                state
                    .deployed_units
                    .iter()
                    .filter(|unit| unit.tactical_state.relocation_eligibility.enabled)
                    .take(2)
                    .count()
                    == 2
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

        let mut evaluations = vec![
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
                ordinary(if self.active_battle_retreat_available() {
                    BattleActionEvaluation::Enabled
                } else {
                    BattleActionEvaluation::Disabled(
                        ActionDisabledReasonCode::ProgressionUnavailable,
                    )
                }),
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
                ordinary(if active.has_selection() {
                    BattleActionEvaluation::Disabled(ActionDisabledReasonCode::SelectionActive)
                } else {
                    BattleActionEvaluation::Enabled
                }),
            ),
            (ActionKind::DeployUnit, deploy),
            (ActionKind::RequestDeploymentRangePreview, deploy),
            (ActionKind::WithdrawUnit, withdraw),
            (ActionKind::SetEngagementStance, set_engagement_stance),
            (ActionKind::TacticalRelocateUnit, tactical_relocate),
            (ActionKind::TacticalSwapUnits, tactical_swap),
            (
                ActionKind::FocusBattleUnit,
                ordinary(
                    if active
                        .battle
                        .units
                        .values()
                        .any(|unit| unit.is_active() && unit.is_combatant())
                    {
                        BattleActionEvaluation::Enabled
                    } else {
                        BattleActionEvaluation::Disabled(
                            ActionDisabledReasonCode::NoEligiblePayload,
                        )
                    },
                ),
            ),
            (
                ActionKind::ClearBattleFocus,
                ordinary(if active.interaction.focused_unit_instance_id.is_some() {
                    BattleActionEvaluation::Enabled
                } else {
                    BattleActionEvaluation::Irrelevant(ActionDisabledReasonCode::ContextIrrelevant)
                }),
            ),
            (ActionKind::BeginDeploymentSelection, deploy),
            (
                ActionKind::CancelDeploymentSelection,
                ordinary(if active.interaction.deployment_employee_uuid.is_some() {
                    BattleActionEvaluation::Enabled
                } else {
                    BattleActionEvaluation::Irrelevant(ActionDisabledReasonCode::ContextIrrelevant)
                }),
            ),
            (
                ActionKind::OpenBattleSettings,
                ordinary(if active.settings_open() {
                    BattleActionEvaluation::Disabled(ActionDisabledReasonCode::AlreadyActive)
                } else {
                    BattleActionEvaluation::Enabled
                }),
            ),
            (
                ActionKind::CloseBattleSettings,
                ordinary(if active.settings_open() {
                    BattleActionEvaluation::Enabled
                } else {
                    BattleActionEvaluation::Irrelevant(ActionDisabledReasonCode::ContextIrrelevant)
                }),
            ),
            (ActionKind::BeginSkillTargeting, begin_skill_targeting),
            (ActionKind::CancelSkillTargeting, cancel_skill_targeting),
            (ActionKind::PreviewSkillTarget, preview_skill_target),
            (
                ActionKind::PreviewTacticalCommand,
                ordinary(BattleActionEvaluation::Enabled),
            ),
            (ActionKind::ActivateSkill, activate_skill),
            (ActionKind::AdvanceVisualNovel, advance_visual_novel),
            (
                ActionKind::SelectVisualNovelChoice,
                select_visual_novel_choice,
            ),
        ];
        if active.settings_open() && !finished {
            for (action, evaluation) in &mut evaluations {
                if !super::battle_interaction::settings_allows_action(*action) {
                    *evaluation =
                        BattleActionEvaluation::Disabled(ActionDisabledReasonCode::SettingsOpen);
                }
            }
        }
        debug_assert!(
            evaluations
                .iter()
                .all(|(action, _)| action.route().uses_battle_preflight()),
            "battle evaluator contains a command owned by another route"
        );
        debug_assert!(
            ActionKind::ALL
                .iter()
                .filter(|action| action.route().uses_battle_preflight())
                .all(|action| evaluations.iter().any(|(candidate, _)| candidate == action)),
            "battle route descriptor contains a command missing from its evaluator"
        );
        Ok(Some(evaluations))
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
        if !action.route().uses_battle_preflight() {
            return Ok(false);
        }
        let Some((_, evaluation)) = evaluations
            .into_iter()
            .find(|(candidate, _)| *candidate == action)
        else {
            debug_assert!(false, "battle-routed action is missing from its evaluator");
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
