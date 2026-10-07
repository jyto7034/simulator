use super::{state::ActiveBattleSession, GameCore};
use crate::game::behavior::{
    ActionDisabledReasonCode, ActionKind, BattlePlaybackSpeed, BehaviorResult, GameError,
    LiveBattleInteractionDto, LiveBattleTacticalFocusDto, PlayerBehavior,
};

impl ActiveBattleSession {
    pub(super) fn has_selection(&self) -> bool {
        self.interaction.focused_unit_instance_id.is_some()
            || self.interaction.deployment_employee_uuid.is_some()
            || self.tactical_focus.is_some()
    }

    pub(super) fn settings_open(&self) -> bool {
        self.interaction.settings_restore_playback.is_some()
    }

    pub(super) fn sync_selection_playback(&mut self) {
        let before_speed = self.playback.speed;
        if self.has_selection() {
            self.interaction.restore_playback.get_or_insert(
                self.interaction
                    .settings_restore_playback
                    .unwrap_or(self.playback),
            );
            self.playback.speed = BattlePlaybackSpeed::X0_5;
            if let Some(saved) = self.interaction.settings_restore_playback.as_mut() {
                saved.speed = BattlePlaybackSpeed::X0_5;
            }
        } else if let Some(saved) = self.interaction.restore_playback.take() {
            // Pause has its own owner. Ending a selection never resumes battle.
            self.playback.speed = saved.speed;
            if let Some(settings) = self.interaction.settings_restore_playback.as_mut() {
                settings.speed = saved.speed;
            }
        }
        if before_speed != self.playback.speed {
            self.playback_delta_remainder = 0;
        }
    }

    pub(super) fn remember_manual_pause(&mut self, paused: bool) {
        if let Some(saved) = self.interaction.restore_playback.as_mut() {
            saved.paused = paused;
        }
        if let Some(saved) = self.interaction.settings_restore_playback.as_mut() {
            saved.paused = paused;
        }
    }

    pub(super) fn reconcile_tactical_focus(&mut self) {
        let valid_unit = |id| {
            self.battle
                .units
                .get(&id)
                .is_some_and(|unit| unit.is_active())
        };
        if self
            .tactical_focus
            .as_ref()
            .is_some_and(|focus| !valid_unit(focus.caster_unit_instance_id))
        {
            self.tactical_focus = None;
        }
        if self
            .interaction
            .focused_unit_instance_id
            .is_some_and(|id| !valid_unit(id))
        {
            self.interaction.focused_unit_instance_id = None;
        }
        if self
            .interaction
            .deployment_employee_uuid
            .is_some_and(|employee| {
                self.employee_roster()
                    .get(&employee)
                    .is_none_or(|value| value.battle_deployment_disabled_reason().is_some())
                    || self
                        .live_deployment
                        .as_ref()
                        .is_none_or(|deployment| deployment.deployed_units.contains_key(&employee))
            })
        {
            self.interaction.deployment_employee_uuid = None;
        }
        self.sync_selection_playback();
    }

    pub(super) fn tactical_focus_dto(&self) -> Option<LiveBattleTacticalFocusDto> {
        self.tactical_focus
            .as_ref()
            .map(|focus| LiveBattleTacticalFocusDto {
                purpose: focus.purpose,
                caster_unit_instance_id: focus.caster_unit_instance_id,
                skill_id: focus.skill_id.clone(),
                restore_playback: self
                    .interaction
                    .restore_playback
                    .expect("a skill focus owns a shared selection restore"),
            })
    }

    pub(super) fn interaction_dto(&self) -> LiveBattleInteractionDto {
        LiveBattleInteractionDto {
            focused_unit_instance_id: self.interaction.focused_unit_instance_id,
            deployment_employee_uuid: self.interaction.deployment_employee_uuid,
            restore_playback: self.interaction.restore_playback,
            settings_restore_playback: self.interaction.settings_restore_playback,
        }
    }
}

impl GameCore {
    pub(super) fn handle_battle_interaction(
        &mut self,
        behavior: PlayerBehavior,
        after_seq: u64,
    ) -> Result<BehaviorResult, GameError> {
        let action = behavior.kind();
        let active = self
            .state
            .active_battle
            .as_mut()
            .ok_or(GameError::InvalidAction)?;
        active.ensure_battle_projection_commit_available()?;
        let unavailable = |reason| GameError::ActionUnavailable { action, reason };
        match behavior {
            PlayerBehavior::FocusBattleUnit { unit_instance_id } => {
                if !active
                    .battle
                    .units
                    .get(&unit_instance_id)
                    .is_some_and(|unit| unit.is_active() && unit.is_combatant())
                {
                    return Err(unavailable(ActionDisabledReasonCode::UnitUnavailable));
                }
                active.interaction.focused_unit_instance_id = Some(unit_instance_id);
            }
            PlayerBehavior::ClearBattleFocus => {
                active.interaction.focused_unit_instance_id = None;
            }
            PlayerBehavior::BeginDeploymentSelection { employee_uuid } => {
                let eligibility = active
                    .live_deployment_dto(active.employee_roster())
                    .and_then(|deployment| {
                        deployment
                            .unit_deploy_costs
                            .into_iter()
                            .find(|row| row.employee_uuid == employee_uuid)
                    })
                    .ok_or_else(|| unavailable(ActionDisabledReasonCode::UnitUnavailable))?
                    .eligibility;
                if let Some(reason) = eligibility.disabled_reason_code {
                    return Err(unavailable(reason));
                }
                active.interaction.deployment_employee_uuid = Some(employee_uuid);
            }
            PlayerBehavior::CancelDeploymentSelection => {
                active.interaction.deployment_employee_uuid = None;
            }
            PlayerBehavior::OpenBattleSettings => {
                if active.settings_open() {
                    return Err(unavailable(ActionDisabledReasonCode::AlreadyActive));
                }
                active.interaction.settings_restore_playback = Some(active.playback);
                active.playback.paused = true;
            }
            PlayerBehavior::CloseBattleSettings => {
                active.playback = active
                    .interaction
                    .settings_restore_playback
                    .take()
                    .ok_or_else(|| unavailable(ActionDisabledReasonCode::ContextIrrelevant))?;
            }
            _ => unreachable!("battle interaction routing is exhaustive"),
        }
        active.sync_selection_playback();
        active.commit_battle_projection()?;
        let battle_update =
            active.battle_update_dto_after_or_resync(after_seq, active.employee_roster())?;
        Ok(BehaviorResult::BattlePlaybackChanged {
            battle_uuid: active.battle_uuid,
            encounter_id: active.encounter_id.clone(),
            node_type: active.node_type,
            mission_variant: active.mission_variant,
            battle_update,
        })
    }
}

/// Settings owns gameplay input, while read-only previews and existing retreat remain available.
pub(super) fn settings_allows_action(action: ActionKind) -> bool {
    matches!(
        action,
        ActionKind::RequestBattleState
            | ActionKind::RecoverBattleSetupLoss
            | ActionKind::PreviewSkillTarget
            | ActionKind::PreviewTacticalCommand
            | ActionKind::RequestDeploymentRangePreview
            | ActionKind::OpenBattleSettings
            | ActionKind::CloseBattleSettings
            | ActionKind::RetreatBattle
    )
}
