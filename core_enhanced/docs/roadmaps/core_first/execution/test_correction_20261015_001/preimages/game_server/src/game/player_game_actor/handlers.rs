use actix::{ActorContext, AsyncContext, Handler};
use serde_json::Value;

use game_core::application::{
    ApplicationBattleAuthority, ApplicationCommandKind, ApplicationDocument, ApplicationError,
    ApplicationErrorKind, ApplicationExecutionContext, ApplicationResultKind,
    ApplicationSnapshotKind,
};
use game_core::game::behavior::{HostRecoveryAction, RunSnapshotDto};
use tracing::{info, warn};

use crate::game::player_game_actor::{
    messages::{
        AttachSession, AttachSessionResult, CommandExecutionResult, DetachSession,
        EnsureLiveBattleTick, ExecuteAdminCommand, ExecutePlayerBehavior, ForceDisconnect,
        GetEncounterCompendiumProfile, PlayerGameServerMessage, PushServerMessage, QuitPlayerActor,
    },
    state::{
        battle_payload, battle_update_from_behavior_result, behavior_result_to_command_result,
        ActiveRunContinueSnapshotDto, HostSnapshotOverlay, PlayerGameActorError,
        PlayerProfilePersistenceSnapshotDto, PlayerStateSnapshotDto,
    },
    BattleEventDeliveryCursor, PlayerGameActor,
};
use crate::persistence::active_run_store::{ActiveRunLoad, StoredCheckpoint};
use crate::persistence::player_profile_store::{
    PlayerProfilePersistOutcome, PlayerProfilePersistenceState,
};

impl Handler<AttachSession> for PlayerGameActor {
    type Result = Result<AttachSessionResult, PlayerGameActorError>;

    fn handle(&mut self, msg: AttachSession, ctx: &mut Self::Context) -> Self::Result {
        if let Some(timer) = self.disconnect_timer.take() {
            ctx.cancel_future(timer);
        }

        let replacing_active_session =
            self.active_session_id.is_some() && self.active_session_id != Some(msg.session_id);
        self.stop_live_battle_tick(ctx);
        if replacing_active_session
            && self.has_active_live_battle()
            && self.game_core.battle_fault().is_none()
        {
            if let Err(error) = self.game_core.pause_active_battle_for_disconnect() {
                if self.game_core.battle_fault().is_none() {
                    return Err(error.into());
                }
            }
        }

        if self.active_session_id != Some(msg.session_id) {
            if let Some(control) = self.session_control.take() {
                control.do_send(ForceDisconnect {
                    code: "session_replaced".to_string(),
                    message: "A newer connection replaced this session".to_string(),
                });
            }
        }

        self.stop_live_battle_tick(ctx);
        self.active_session_id = Some(msg.session_id);
        self.socket = Some(msg.socket);
        self.session_control = Some(msg.control);
        self.battle_event_cursor = None;

        if self.game_core.battle_fault().is_some() {
            return Ok(AttachSessionResult {
                state_snapshot: self.build_state_snapshot()?,
                side_messages: Vec::new(),
            });
        }
        self.sync_player_profile_from_persistence_for_snapshot()?;
        let state_snapshot = self.build_state_snapshot()?;
        let mut side_messages = Vec::new();
        if let Some(persistence) = self.player_profile_persistence.as_ref() {
            if persistence
                .lock()
                .map_err(|_| {
                    PlayerGameActorError::new(
                        "player_profile_lock_poisoned",
                        "Player profile persistence lock is poisoned",
                    )
                })?
                .take_recovery_notification()
            {
                side_messages.push(PlayerGameServerMessage::Notification {
                    notification_type: "player_profile_recovered_from_backup".to_string(),
                    payload: serde_json::json!({}),
                });
            }
        }
        if self.has_active_live_battle() {
            let recovery = self
                .game_core
                .active_battle_reconnect(0)
                .map_err(PlayerGameActorError::from)?;
            let projection = battle_payload(&recovery)?.ok_or_else(|| {
                PlayerGameActorError::new(
                    "invalid_battle_reconnect_projection",
                    "Active battle reconnect did not produce an update",
                )
            })?;
            let setup = projection.battle_setup_snapshot.ok_or_else(|| {
                PlayerGameActorError::new(
                    "invalid_battle_reconnect_projection",
                    "Active battle reconnect did not produce immutable setup",
                )
            })?;
            let battle_update = projection.battle_update;
            side_messages.push(PlayerGameServerMessage::battle_setup_snapshot(setup));
            side_messages.push(PlayerGameServerMessage::battle_update(battle_update));
            for message in &side_messages {
                self.record_battle_message_delivery(message);
            }
        }

        Ok(AttachSessionResult {
            state_snapshot,
            side_messages,
        })
    }
}

impl Handler<DetachSession> for PlayerGameActor {
    type Result = ();

    fn handle(&mut self, msg: DetachSession, ctx: &mut Self::Context) -> Self::Result {
        if self.active_session_id == Some(msg.session_id) {
            let preserve_active_battle = self.has_active_live_battle();
            if preserve_active_battle && self.game_core.battle_fault().is_none() {
                if let Err(error) = self.game_core.pause_active_battle_for_disconnect() {
                    warn!(
                        player_id = %self.player_id,
                        error = ?error,
                        "failed to auto-pause active battle during session detach"
                    );
                }
            }

            self.active_session_id = None;
            self.socket = None;
            self.session_control = None;
            self.battle_event_cursor = None;
            self.stop_live_battle_tick(ctx);

            if let Some(timer) = self.disconnect_timer.take() {
                ctx.cancel_future(timer);
            }

            // A live battle has no disconnect TTL while this server process is
            // alive. Process death still discards it through the existing
            // checkpoint-only restore path.
            if preserve_active_battle {
                return;
            }

            let ttl = self.disconnect_ttl;
            self.disconnect_timer = Some(ctx.run_later(ttl, |act, ctx| {
                act.disconnect_timer = None;
                if act.active_session_id.is_none() {
                    ctx.stop();
                }
            }));
        }
    }
}

impl Handler<ExecutePlayerBehavior> for PlayerGameActor {
    type Result = Result<CommandExecutionResult, PlayerGameActorError>;

    fn handle(&mut self, msg: ExecutePlayerBehavior, ctx: &mut Self::Context) -> Self::Result {
        self.ensure_active_session(msg.session_id)?;
        self.ensure_battle_not_faulted(ctx)?;
        let action_kind = msg.behavior.kind();
        let host_recovery = match action_kind {
            ApplicationCommandKind::ContinueRun => Some(HostRecoveryAction::ContinueRun),
            ApplicationCommandKind::ResetActiveRun => Some(HostRecoveryAction::ResetActiveRun),
            ApplicationCommandKind::RetryPlayerProfile => {
                Some(HostRecoveryAction::RetryPlayerProfile)
            }
            ApplicationCommandKind::ResetPlayerProfile => {
                Some(HostRecoveryAction::ResetPlayerProfile)
            }
            _ => None,
        };
        if let Some(operation) = host_recovery {
            match operation {
                HostRecoveryAction::ContinueRun
                | HostRecoveryAction::ResetActiveRun
                | HostRecoveryAction::RetryPlayerProfile
                | HostRecoveryAction::ResetPlayerProfile => {}
            }
        }
        if matches!(
            host_recovery,
            Some(HostRecoveryAction::RetryPlayerProfile | HostRecoveryAction::ResetPlayerProfile)
        ) {
            let execution_result = self.handle_player_profile_management(
                msg.request_id,
                matches!(host_recovery, Some(HostRecoveryAction::ResetPlayerProfile)),
            )?;
            self.emit_command_execution_result(&execution_result);
            return Ok(execution_result);
        }
        let query = matches!(
            action_kind,
            ApplicationCommandKind::RequestMapData
                | ApplicationCommandKind::RequestBattleState
                | ApplicationCommandKind::PreviewSkillTarget
                | ApplicationCommandKind::RequestDeploymentRangePreview
        );
        if query {
            self.sync_player_profile_from_persistence_for_snapshot()?;
        } else if !matches!(host_recovery, Some(HostRecoveryAction::ResetActiveRun)) {
            self.sync_player_profile_from_persistence()?;
        }
        if matches!(host_recovery, Some(HostRecoveryAction::ResetActiveRun)) {
            if !self.game_core.is_not_started() { return Err(PlayerGameActorError::new("active_run_reset_requires_title", "Corrupt active reset requires title state")); }
            let store = self.active_run_store.clone().ok_or_else(|| PlayerGameActorError::new("active_run_persistence_unavailable", "Active store is unavailable"))?;
            let persistence = self.player_profile_persistence.clone().ok_or_else(|| PlayerGameActorError::new("player_profile_persistence_unavailable", "Paired active reset requires profile persistence"))?;
            let mut profile = persistence.lock().map_err(|_| PlayerGameActorError::new("player_profile_lock_poisoned", "Profile lock is poisoned"))?;
            let active = store.lock().map_err(|_| PlayerGameActorError::new("active_run_lock_poisoned", "Active lock is poisoned"))?;
            let mut next_profile = profile.profile().map_err(|e| PlayerGameActorError::new("player_profile_recovery_required", e))?;
            next_profile.lifecycle.active_expedition = None;
            let prepared = self.game_core.prepare_authoritative_profile_install(ApplicationDocument::new(serde_json::to_value(next_profile).map_err(|e| PlayerGameActorError::new("serialization_failed", e.to_string()))?))?;
            let accepted = profile.accept_corrupt_active_reset(&active).map_err(|e| PlayerGameActorError::new("active_run_reset_failed", e.to_string()))?;
            prepared.commit();
            self.active_run_revision = Some(accepted.revision);
            drop(active);
            drop(profile);
            let execution_result = CommandExecutionResult {
                response: PlayerGameServerMessage::CommandResult {
                    request_id: msg.request_id,
                    ok: true,
                    result_type: "ActiveRunReset".to_string(),
                    payload: serde_json::json!({ "recovery_pending": accepted.recovery_pending }),
                },
                side_messages: Vec::new(),
                state_snapshot: Some(self.build_state_snapshot()?),
            };
            self.emit_command_execution_result(&execution_result);
            return Ok(execution_result);
        }
        let battle_event_after_seq = self.battle_event_after_seq();
        let checkpoint_before = self
            .game_core
            .export_active_run_checkpoint()
            .map_err(PlayerGameActorError::from)?;
        let is_continue = matches!(host_recovery, Some(HostRecoveryAction::ContinueRun));
        let is_new_game = msg.behavior.kind() == ApplicationCommandKind::StartNewGame;
        let replacement_requested = is_new_game
            && msg.behavior.document().as_value()["replace_active_run"]
                .as_bool()
                .unwrap_or(false);
        if is_new_game && !replacement_requested {
            if let Some(store) = self.active_run_store.as_ref() {
                match store
                    .lock()
                    .map_err(|_| {
                        PlayerGameActorError::new(
                            "active_run_lock_poisoned",
                            "Active-run persistence lock is poisoned",
                        )
                    })?
                    .load()
                {
                    Ok(ActiveRunLoad::Loaded { .. }) => {
                        return Err(PlayerGameActorError::new(
                            "active_run_requires_continue_and_abandon",
                            "Continue and explicitly abandon the saved expedition before starting a new run",
                        ));
                    }
                    Ok(ActiveRunLoad::Missing) => {}
                    Err(error) => {
                        return Err(PlayerGameActorError::new(
                            "active_run_load_failed",
                            error.to_string(),
                        ));
                    }
                }
            }
        }
        let mut recovered_from_backup = false;
        let mut durable_recovery_pending = false;
        let active_store = self.active_run_store.clone();
        let profile_persistence = self.player_profile_persistence.clone();
        let execution = if is_continue {
            let store = self.active_run_store.clone().ok_or_else(|| {
                PlayerGameActorError::new(
                    "active_run_persistence_unavailable",
                    "Active-run persistence is unavailable",
                )
            })?;
            let loaded = store
                .lock()
                .map_err(|_| {
                    PlayerGameActorError::new(
                        "active_run_lock_poisoned",
                        "Active-run persistence lock is poisoned",
                    )
                })?
                .snapshot()
                .map_err(|error| {
                    PlayerGameActorError::new("active_run_load_failed", error.to_string())
                })?;
            let checkpoint = loaded.checkpoint.ok_or_else(|| {
                PlayerGameActorError::new(
                    "active_run_unavailable",
                    "No valid active run is available to continue",
                )
            })?;
            recovered_from_backup = loaded.recovered_from_backup;
            let result = self
                .game_core
                .continue_from_active_run_checkpoint(self.player_id, &checkpoint.to_application());
            if result.is_ok() {
                self.active_run_revision = Some(loaded.revision);
                if let Some(required) = loaded.required_configuration {
                    self.active_required = Some(required);
                }
            }
            result
        } else {
            let context = ApplicationExecutionContext {
                source_command_id: Some(msg.request_id.clone()),
                battle_event_after_seq,
                expected_battle_authority: msg.battle_authority.map(|authority| {
                    ApplicationBattleAuthority {
                        battle_uuid: authority.battle_uuid,
                        battle_state_revision: authority.battle_state_revision,
                    }
                }),
            };
            if (active_store.is_some() || profile_persistence.is_some()) && !query {
                let staged = if is_new_game { self.approved_host.as_ref().map(|host| host.future_generation()).transpose().map_err(|e| PlayerGameActorError::new("approved_selection_unavailable", e.to_string()))? } else { None };
                let required = staged.as_ref().map(|(_, r)| r.clone()).or_else(|| self.active_required.clone());
                let generation = staged.map(|(g, _)| g);
                match self
                    .game_core
                    .prepare_command_with_generation(self.player_id, msg.behavior, context, generation)
                {
                    Ok(prepared) => {
                        let (result, pending) = publish_prepared_application_bound(
                            prepared,
                            self.approved_persistence.as_ref(), required.as_ref(),
                            active_store.as_ref(),
                            profile_persistence.as_ref(),
                            &mut self.active_run_revision,
                            checkpoint_before.as_ref(),
                            is_new_game,
                        )?;
                        self.active_required = required;
                        durable_recovery_pending = pending;
                        Ok(result)
                    }
                    Err(error) => Err(error),
                }
            } else {
                self.game_core
                    .execute(self.player_id, msg.behavior, context)
            }
        };
        let result = match execution {
            Ok(result) => result,
            Err(error)
                if matches!(
                    error.kind(),
                    ApplicationErrorKind::MissingBattleStateAuthority
                        | ApplicationErrorKind::StaleBattleStateAuthority
                ) =>
            {
                let execution_result = self.battle_authority_rejection_result(
                    error,
                    msg.request_id,
                    battle_event_after_seq,
                )?;
                self.emit_command_execution_result(&execution_result);
                return Ok(execution_result);
            }
            Err(error) => {
                self.publish_battle_fault(ctx);
                return Err(PlayerGameActorError::from(error));
            }
        };
        let compendium_persist_outcome = if active_store.is_some() {
            PlayerProfilePersistOutcome::Unchanged
        } else {
            self.persist_player_profile()?
        };
        let mut command_result =
            behavior_result_to_command_result(msg.request_id, result, msg.battle_side_message)?;
        if recovered_from_backup {
            command_result
                .side_messages
                .push(PlayerGameServerMessage::Notification {
                    notification_type: "active_run_recovered_from_backup".to_string(),
                    payload: serde_json::json!({}),
                });
        }
        if compendium_persist_outcome == PlayerProfilePersistOutcome::BecameDirty {
            command_result
                .side_messages
                .push(PlayerGameServerMessage::Notification {
                    notification_type: "player_profile_save_failed".to_string(),
                    payload: serde_json::json!({ "unsaved_progress_may_be_lost": true }),
                });
        }
        if durable_recovery_pending {
            command_result
                .side_messages
                .push(PlayerGameServerMessage::Notification {
                    notification_type: "application_recovery_pending".to_string(),
                    payload: serde_json::json!({}),
                });
        }
        let state_snapshot = command_result
            .send_state_snapshot
            .then(|| self.build_state_snapshot())
            .transpose()?;
        let execution_result = CommandExecutionResult {
            response: command_result.response,
            side_messages: command_result.side_messages,
            state_snapshot,
        };
        self.emit_command_execution_result(&execution_result);
        Ok(execution_result)
    }
}

impl Handler<ExecuteAdminCommand> for PlayerGameActor {
    type Result = Result<CommandExecutionResult, PlayerGameActorError>;

    fn handle(&mut self, msg: ExecuteAdminCommand, ctx: &mut Self::Context) -> Self::Result {
        self.ensure_active_session(msg.session_id)?;
        self.ensure_battle_not_faulted(ctx)?;
        self.sync_player_profile_from_persistence()?;
        let active = self.active_run_store.clone();
        let profile = self.player_profile_persistence.clone();
        let before = self.game_core.export_active_run_checkpoint()?;
        let (result, recovery_pending) = if active.is_some() {
            let prepared = self.game_core.prepare_admin_command(msg.command)?;
            publish_prepared_application_bound(
                prepared,
                self.approved_persistence.as_ref(), self.active_required.as_ref(),
                active.as_ref(),
                profile.as_ref(),
                &mut self.active_run_revision,
                before.as_ref(),
                false,
            )?
        } else {
            (self.game_core.execute_admin(msg.command)?, false)
        };
        let response = PlayerGameServerMessage::AdminResult {
            request_id: msg.request_id,
            ok: true,
            result_type: result.meaning().to_string(),
            payload: result.payload().as_value().clone(),
        };
        let state_snapshot = self.build_state_snapshot()?;
        Ok(CommandExecutionResult {
            response,
            side_messages: if recovery_pending {
                vec![PlayerGameServerMessage::Notification {
                    notification_type: "application_recovery_pending".into(),
                    payload: serde_json::json!({}),
                }]
            } else {
                Vec::new()
            },
            state_snapshot: Some(state_snapshot),
        })
    }
}

impl Handler<PushServerMessage> for PlayerGameActor {
    type Result = ();

    fn handle(&mut self, msg: PushServerMessage, _ctx: &mut Self::Context) -> Self::Result {
        self.push_to_active_socket(msg.message);
    }
}

impl Handler<EnsureLiveBattleTick> for PlayerGameActor {
    type Result = ();

    fn handle(&mut self, _msg: EnsureLiveBattleTick, ctx: &mut Self::Context) -> Self::Result {
        self.ensure_live_battle_tick(ctx);
    }
}

impl Handler<QuitPlayerActor> for PlayerGameActor {
    type Result = ();

    fn handle(&mut self, _msg: QuitPlayerActor, ctx: &mut Self::Context) -> Self::Result {
        info!(
            "PlayerGameActor quitting on client request: player {}",
            self.player_id
        );
        if let Some(timer) = self.disconnect_timer.take() {
            ctx.cancel_future(timer);
        }
        self.stop_live_battle_tick(ctx);
        self.active_session_id = None;
        self.socket = None;
        self.session_control = None;
        self.battle_event_cursor = None;
        // stopped() 에서 LoadBalance Deregister 가 호출됨 → 다음 접속 시 새 Actor 생성.
        ctx.stop();
    }
}

impl Handler<GetEncounterCompendiumProfile> for PlayerGameActor {
    type Result = actix::MessageResult<GetEncounterCompendiumProfile>;

    fn handle(
        &mut self,
        _msg: GetEncounterCompendiumProfile,
        _ctx: &mut Self::Context,
    ) -> Self::Result {
        actix::MessageResult(self.read_snapshot(ApplicationSnapshotKind::EncounterCompendium))
    }
}

impl PlayerGameActor {
    fn sync_player_profile_from_persistence_for_snapshot(
        &mut self,
    ) -> Result<(), PlayerGameActorError> {
        self.game_core.ensure_battle_not_faulted()?;
        let Some(persistence) = self.player_profile_persistence.clone() else {
            return Ok(());
        };
        let profile = {
            let persistence = persistence.lock().map_err(|_| {
                PlayerGameActorError::new(
                    "player_profile_lock_poisoned",
                    "Player profile persistence lock is poisoned",
                )
            })?;
            match persistence.state() {
                PlayerProfilePersistenceState::ApplicationRecoveryPending { .. }
                | PlayerProfilePersistenceState::Protected { .. }
                | PlayerProfilePersistenceState::Fatal { .. } => None,
                _ => Some(persistence.profile().map_err(|issue| {
                    PlayerGameActorError::new("player_profile_recovery_required", issue)
                })?),
            }
        };
        if let Some(profile) = profile {
            self.game_core
                .install_authoritative_player_profile(ApplicationDocument::new(
                    serde_json::to_value(profile).map_err(|error| {
                        PlayerGameActorError::new("serialization_failed", error.to_string())
                    })?,
                ))
                .map_err(PlayerGameActorError::from)?;
        }
        Ok(())
    }

    fn sync_player_profile_from_persistence(&mut self) -> Result<(), PlayerGameActorError> {
        self.game_core.ensure_battle_not_faulted()?;
        let Some(persistence) = self.player_profile_persistence.as_ref() else { return Ok(()); };
        let persistence = persistence.lock().map_err(|_| PlayerGameActorError::new("player_profile_lock_poisoned", "Player profile persistence lock is poisoned"))?;
        let profile = persistence.profile().map_err(|issue| PlayerGameActorError::new("player_profile_recovery_required", issue))?;
        if let Some(active) = self.active_run_store.as_ref() {
            let active = active.lock().map_err(|_| PlayerGameActorError::new("active_run_lock_poisoned", "Active store lock is poisoned"))?;
            if self.active_run_revision.is_none() {
                let snapshot = active.snapshot().map_err(|e| PlayerGameActorError::new("active_run_load_failed", e.to_string()))?;
                self.active_run_revision = Some(snapshot.revision);
            }
        }
        self.game_core.install_authoritative_player_profile(ApplicationDocument::new(
            serde_json::to_value(profile).map_err(|e| PlayerGameActorError::new("serialization_failed", e.to_string()))?
        )).map_err(PlayerGameActorError::from)
    }

    fn persist_player_profile(
        &mut self,
    ) -> Result<PlayerProfilePersistOutcome, PlayerGameActorError> {
        self.game_core.ensure_battle_not_faulted()?;
        let Some(persistence) = self.player_profile_persistence.as_ref() else {
            return Ok(PlayerProfilePersistOutcome::Unchanged);
        };
        persistence
            .lock()
            .map_err(|_| {
                PlayerGameActorError::new(
                    "player_profile_lock_poisoned",
                    "Player profile persistence lock is poisoned",
                )
            })?
            .merge_and_persist(self.read_player_profile()?)
            .map_err(|e| PlayerGameActorError::new("player_profile_persistence_failed", e.to_string()))
    }

    fn handle_player_profile_management(
        &mut self,
        request_id: String,
        reset: bool,
    ) -> Result<CommandExecutionResult, PlayerGameActorError> {
        self.game_core.ensure_battle_not_faulted()?;
        if reset && !self.game_core.is_not_started() {
            return Err(PlayerGameActorError::new(
                "player_profile_reset_requires_title",
                "Player profile reset is only available before a run starts",
            ));
        }
        let persistence = self.player_profile_persistence.clone().ok_or_else(|| {
            PlayerGameActorError::new(
                "player_profile_persistence_unavailable",
                "Player profile persistence is unavailable",
            )
        })?;
        let mut persist_outcome = PlayerProfilePersistOutcome::Unchanged;
        let mut recovered_from_backup = false;
        let profile_clean;
        {
            let mut persistence = persistence.lock().map_err(|_| {
                PlayerGameActorError::new(
                    "player_profile_lock_poisoned",
                    "Player profile persistence lock is poisoned",
                )
            })?;
            if reset {
                if let Some(active) = self.active_run_store.as_ref() {
                    let active = active.lock().map_err(|_| PlayerGameActorError::new("active_run_lock_poisoned", "Active lock is poisoned"))?;
                    let snapshot = active.snapshot().map_err(|e| PlayerGameActorError::new("active_run_requires_explicit_recovery", e.to_string()))?;
                    if snapshot.checkpoint.is_some() { return Err(PlayerGameActorError::new("profile_reset_would_orphan_active_run", "Continue and explicitly abandon the saved run before resetting the profile")); }
                }
                if matches!(
                    persistence.state(),
                    PlayerProfilePersistenceState::ApplicationRecoveryPending { .. }
                ) {
                    return Err(PlayerGameActorError::new(
                        "application_recovery_pending",
                        "Finish the pending application publication before resetting the player profile",
                    ));
                }
                persistence.reset().map_err(|error| {
                    PlayerGameActorError::new("player_profile_reset_failed", error.to_string())
                })?;
                self.game_core.reset_player_profile()?;
            } else if matches!(
                persistence.state(),
                PlayerProfilePersistenceState::ApplicationRecoveryPending { .. }
            ) {
                let active_run_store = self.active_run_store.as_ref().ok_or_else(|| {
                    PlayerGameActorError::new(
                        "active_run_persistence_unavailable",
                        "Active-run persistence is unavailable",
                    )
                })?;
                let active_run_store = active_run_store.lock().map_err(|_| {
                    PlayerGameActorError::new(
                        "active_run_lock_poisoned",
                        "Active-run persistence lock is poisoned",
                    )
                })?;
                persistence
                    .recover_pending_application(&active_run_store)
                    .map_err(|error| {
                        PlayerGameActorError::new("application_recovery_failed", error.to_string())
                    })?;
                recovered_from_backup = persistence.take_recovery_notification();
            } else if matches!(
                persistence.state(),
                PlayerProfilePersistenceState::Fatal { .. }
                    | PlayerProfilePersistenceState::Protected { .. }
            ) {
                persistence.reload().map_err(|error| {
                    PlayerGameActorError::new("player_profile_retry_failed", error.to_string())
                })?;
                recovered_from_backup = persistence.take_recovery_notification();
            } else {
                persist_outcome = persistence.retry().map_err(|e| PlayerGameActorError::new("player_profile_persistence_failed", e.to_string()))?;
            }
            if let Ok(profile) = persistence.profile() {
                self.game_core
                    .install_authoritative_player_profile(ApplicationDocument::new(
                        serde_json::to_value(profile).map_err(|error| {
                            PlayerGameActorError::new("serialization_failed", error.to_string())
                        })?,
                    ))
                    .map_err(PlayerGameActorError::from)?;
            }
            profile_clean = matches!(persistence.state(), PlayerProfilePersistenceState::Clean);
        }
        let mut side_messages = Vec::new();
        if recovered_from_backup {
            side_messages.push(PlayerGameServerMessage::Notification {
                notification_type: "player_profile_recovered_from_backup".to_string(),
                payload: serde_json::json!({}),
            });
        }
        if persist_outcome == PlayerProfilePersistOutcome::BecameDirty {
            side_messages.push(PlayerGameServerMessage::Notification {
                notification_type: "player_profile_save_failed".to_string(),
                payload: serde_json::json!({ "unsaved_progress_may_be_lost": true }),
            });
        }
        Ok(CommandExecutionResult {
            response: PlayerGameServerMessage::CommandResult {
                request_id,
                ok: true,
                result_type: if reset {
                    "PlayerProfileReset"
                } else {
                    "PlayerProfileRetry"
                }
                .to_string(),
                payload: serde_json::json!({
                    "saved": profile_clean,
                }),
            },
            side_messages,
            state_snapshot: Some(self.build_state_snapshot()?),
        })
    }

    fn ensure_active_session(&self, session_id: uuid::Uuid) -> Result<(), PlayerGameActorError> {
        if self.active_session_id == Some(session_id) {
            return Ok(());
        }

        Err(PlayerGameActorError::new(
            "stale_session",
            "This socket is no longer bound to the active player session",
        ))
    }

    fn build_state_snapshot(&self) -> Result<Value, PlayerGameActorError> {
        if let Some(fault) = self.game_core.battle_fault() {
            // This projection never loads a save (backup recovery is a write),
            // merges a profile, or depends on the potentially failing Run DTO.
            return Ok(serde_json::json!({
                "game_state": "battle_faulted",
                "battle_fault": fault,
                "action_capabilities": [],
                "recovery": "quit_then_continue_valid_checkpoint"
            }));
        }
        let snapshot: RunSnapshotDto = self.read_snapshot(ApplicationSnapshotKind::Run)?;
        // Terminal diagnostic history is not part of the production result API.
        let compressed_event_log = None;
        let title_screen = self.game_core.is_not_started();
        let active_run_continue = if title_screen {
            self.active_run_store
                .as_ref()
                .map(|store| match store.lock() {
                    Ok(store) => match store.load() {
                        Ok(ActiveRunLoad::Missing) => ActiveRunContinueSnapshotDto::missing(),
                        Ok(ActiveRunLoad::Loaded {
                            recovered_from_backup,
                            ..
                        }) => ActiveRunContinueSnapshotDto::loaded(recovered_from_backup),
                        Err(error) if error.is_corrupt() => {
                            ActiveRunContinueSnapshotDto::reset_required(error.to_string())
                        }
                        Err(error) => ActiveRunContinueSnapshotDto::protected(error.to_string()),
                    },
                    Err(_) => ActiveRunContinueSnapshotDto::reset_required(
                        "active-run persistence lock is poisoned",
                    ),
                })
        } else {
            None
        };
        let player_profile_persistence = if let Some(persistence) =
            self.player_profile_persistence.as_ref()
        {
            let persistence = persistence.lock().map_err(|_| {
                PlayerGameActorError::new(
                    "player_profile_lock_poisoned",
                    "Player profile persistence lock is poisoned",
                )
            })?;
            Some(match persistence.state() {
                PlayerProfilePersistenceState::Clean => {
                    PlayerProfilePersistenceSnapshotDto::clean()
                }
                PlayerProfilePersistenceState::RecoveredFromBackup => {
                    PlayerProfilePersistenceSnapshotDto::recovered_from_backup()
                }
                PlayerProfilePersistenceState::Dirty { issue } => {
                    PlayerProfilePersistenceSnapshotDto::dirty(issue.clone())
                }
                PlayerProfilePersistenceState::ApplicationRecoveryPending { issue } => {
                    PlayerProfilePersistenceSnapshotDto::application_recovery_pending(issue.clone())
                }
                PlayerProfilePersistenceState::Fatal { issue } => {
                    PlayerProfilePersistenceSnapshotDto::fatal(issue.clone())
                }
                PlayerProfilePersistenceState::Protected { issue } => {
                    PlayerProfilePersistenceSnapshotDto::protected(issue.clone())
                }
            })
        } else {
            None
        };
        let overlay = HostSnapshotOverlay::new(
            title_screen,
            active_run_continue,
            player_profile_persistence,
        );
        serde_json::to_value(
            PlayerStateSnapshotDto::new(snapshot, compressed_event_log).with_host_overlay(overlay),
        )
        .map_err(|error| PlayerGameActorError::new("serialization_failed", error.to_string()))
    }

    fn battle_event_after_seq(&self) -> u64 {
        if !self.has_active_live_battle() {
            return 0;
        }
        self.battle_event_cursor
            .map(|cursor| cursor.last_sent_seq)
            .unwrap_or(0)
    }

    fn battle_authority_rejection_result(
        &self,
        error: ApplicationError,
        request_id: String,
        battle_event_after_seq: u64,
    ) -> Result<CommandExecutionResult, PlayerGameActorError> {
        debug_assert!(matches!(
            error.kind(),
            ApplicationErrorKind::MissingBattleStateAuthority
                | ApplicationErrorKind::StaleBattleStateAuthority
        ));
        let mapped = PlayerGameActorError::from(error);
        let recovery = self
            .game_core
            .active_battle_reconnect(battle_event_after_seq)
            .map_err(PlayerGameActorError::from)?;
        let battle_update = battle_payload(&recovery)?
            .ok_or_else(|| {
                PlayerGameActorError::new(
                    "invalid_battle_resync_projection",
                    "Battle authority recovery did not produce a battle update",
                )
            })?
            .battle_update;
        Ok(CommandExecutionResult {
            response: mapped.to_server_message(Some(request_id)),
            side_messages: vec![PlayerGameServerMessage::battle_resync(battle_update)],
            state_snapshot: None,
        })
    }

    fn record_battle_message_delivery(&mut self, message: &PlayerGameServerMessage) {
        let delivered = match message {
            PlayerGameServerMessage::BattleUpdate(update) => {
                Some((update.battle_uuid, update.events_delta.to_seq))
            }
            PlayerGameServerMessage::BattleResync {
                battle_uuid,
                update,
            } => Some((*battle_uuid, update.events_delta.to_seq)),
            _ => None,
        };
        let Some((battle_uuid, delivered_seq)) = delivered else {
            return;
        };
        let last_sent_seq = self
            .battle_event_cursor
            .filter(|cursor| cursor.battle_uuid == battle_uuid)
            .map(|cursor| cursor.last_sent_seq.max(delivered_seq))
            .unwrap_or(delivered_seq);
        self.battle_event_cursor = Some(BattleEventDeliveryCursor {
            battle_uuid,
            last_sent_seq,
        });
    }

    fn push_to_active_socket(&mut self, message: PlayerGameServerMessage) {
        if let Some(socket) = self.socket.clone() {
            self.record_battle_message_delivery(&message);
            socket.do_send(message);
        }
    }

    fn emit_command_execution_result(&mut self, result: &CommandExecutionResult) {
        self.push_to_active_socket(result.response.clone());
        for message in &result.side_messages {
            self.push_to_active_socket(message.clone());
        }
        if let Some(state) = &result.state_snapshot {
            self.push_to_active_socket(PlayerGameServerMessage::StateSnapshot {
                state: state.clone(),
            });
        }
    }

    fn ensure_battle_not_faulted(
        &mut self,
        ctx: &mut <Self as actix::Actor>::Context,
    ) -> Result<(), PlayerGameActorError> {
        if self.game_core.battle_fault().is_some() {
            self.stop_live_battle_tick(ctx);
        }
        self.game_core
            .ensure_battle_not_faulted()
            .map_err(Into::into)
    }

    fn publish_battle_fault(&mut self, ctx: &mut <Self as actix::Actor>::Context) {
        if self.game_core.battle_fault().is_some() {
            self.stop_live_battle_tick(ctx);
            if let Ok(state) = self.build_state_snapshot() {
                self.push_to_active_socket(PlayerGameServerMessage::StateSnapshot { state });
            }
        }
    }

    fn durable_recovery_pending(&self) -> bool {
        self.player_profile_persistence
            .as_ref()
            .is_some_and(|persistence| {
                persistence.lock().map_or(true, |persistence| {
                    matches!(
                        persistence.state(),
                        PlayerProfilePersistenceState::ApplicationRecoveryPending { .. }
                            | PlayerProfilePersistenceState::Protected { .. }
                            | PlayerProfilePersistenceState::Fatal { .. }
                    )
                })
            })
    }

    fn ensure_live_battle_tick(&mut self, ctx: &mut <Self as actix::Actor>::Context) {
        if self.durable_recovery_pending() {
            return;
        }
        if self.ensure_battle_not_faulted(ctx).is_err() {
            return;
        }
        if self.live_battle_tick.is_some()
            || self.socket.is_none()
            || !self.has_active_live_battle()
        {
            return;
        }

        let interval = self.live_battle_tick_interval;
        self.live_battle_tick = Some(ctx.run_interval(interval, |act, ctx| {
            act.tick_live_battle(ctx);
        }));
    }

    fn stop_live_battle_tick(&mut self, ctx: &mut <Self as actix::Actor>::Context) {
        if let Some(handle) = self.live_battle_tick.take() {
            ctx.cancel_future(handle);
        }
    }

    fn tick_live_battle(&mut self, ctx: &mut <Self as actix::Actor>::Context) {
        if self.durable_recovery_pending() {
            self.stop_live_battle_tick(ctx);
            return;
        }
        if self.ensure_battle_not_faulted(ctx).is_err() {
            return;
        }
        if self.socket.is_none() || !self.has_active_live_battle() {
            self.stop_live_battle_tick(ctx);
            return;
        }

        let delta_ms = self.live_battle_tick_interval.as_millis();
        let delta_ms = u64::try_from(delta_ms).unwrap_or(u64::MAX).max(1);
        let battle_event_after_seq = self.battle_event_after_seq();
        let active_store = self.active_run_store.clone();
        let profile_persistence = self.player_profile_persistence.clone();
        let checkpoint_before = match self.game_core.export_active_run_checkpoint() {
            Ok(checkpoint) => checkpoint,
            Err(error) => {
                self.push_to_active_socket(
                    PlayerGameActorError::from(error).to_server_message(None),
                );
                self.stop_live_battle_tick(ctx);
                return;
            }
        };
        let execution = if active_store.is_some() {
            match self
                .game_core
                .prepare_active_battle_for_host_tick(delta_ms, battle_event_after_seq)
            {
                Ok(prepared) => publish_prepared_application_bound(
                    prepared,
                    self.approved_persistence.as_ref(), self.active_required.as_ref(),
                    active_store.as_ref(),
                    profile_persistence.as_ref(),
                    &mut self.active_run_revision,
                    checkpoint_before.as_ref(),
                    false,
                ),
                Err(error) => Err(PlayerGameActorError::from(error)),
            }
        } else {
            self.game_core
                .advance_active_battle_for_host_tick(delta_ms, battle_event_after_seq)
                .map(|result| (result, false))
                .map_err(PlayerGameActorError::from)
        };
        let (result, recovery_pending) = match execution {
            Ok(result) => result,
            Err(error) => {
                warn!(
                    "Live battle tick failed for player {}: {:?}",
                    self.player_id, error
                );
                self.push_to_active_socket(error.to_server_message(None));
                self.stop_live_battle_tick(ctx);
                self.publish_battle_fault(ctx);
                return;
            }
        };
        if recovery_pending {
            self.stop_live_battle_tick(ctx);
            self.push_to_active_socket(PlayerGameServerMessage::Notification {
                notification_type: "application_recovery_pending".into(),
                payload: serde_json::json!({}),
            });
        }
        let Some(result) = result else {
            return;
        };

        let finished = result.kind() == ApplicationResultKind::BattleAdvanced
            && result.payload().as_value()["finished"]
                .as_bool()
                .unwrap_or(false);

        if let Some(battle_update) = battle_update_from_behavior_result(&result) {
            self.push_to_active_socket(PlayerGameServerMessage::battle_update(battle_update));
        } else {
            self.push_to_active_socket(
                PlayerGameActorError::new(
                    "missing_battle_update",
                    "Live battle tick did not produce a battle_update payload",
                )
                .to_server_message(None),
            );
            self.stop_live_battle_tick(ctx);
            return;
        }

        if finished {
            match self.build_state_snapshot() {
                Ok(state) => {
                    self.push_to_active_socket(PlayerGameServerMessage::StateSnapshot { state });
                }
                Err(error) => {
                    self.push_to_active_socket(error.to_server_message(None));
                    self.stop_live_battle_tick(ctx);
                    return;
                }
            }
        }

        if finished || !self.has_active_live_battle() {
            self.stop_live_battle_tick(ctx);
        }
    }
}

// Legacy raw-data fixtures require Core fixture APIs in release tests.
#[cfg(all(test, any(debug_assertions, feature = "test-fixtures")))]
mod tests {
    mod durable;
    mod fault;
    mod u06;
    use super::*;
    use crate::game::{
        load_balance_actor::{messages::Register, LoadBalanceActor},
        player_game_actor::{
            messages::{
                AttachSession, BattleSideMessageKind, DetachSession, EnsureLiveBattleTick,
                ForceDisconnect, PlayerGameServerMessage,
            },
            PlayerGameActor,
        },
    };
    use crate::persistence::active_run_store::StoredCheckpoint as ActiveRunCheckpointExport;
    use crate::persistence::{
        active_run_store::ActiveRunStore,
        player_profile_store::{PlayerProfilePersistence, PlayerProfileStore},
    };
    use crate::test_support::{command, ApplicationFixture};
    use actix::{Actor, Context};
    use actix_web::rt::time;
    use game_core::application::{ApplicationResult, CoreApplication};
    use game_core::game::{
        behavior::GameStateContextDto,
        combat_preview::CombatNodeType,
        data::{
            employee_data::EmployeeDefinitionDatabase,
            equipment_data::{
                EquipmentMetadata, EquipmentType, GroundDefenseFocusProfile, WeaponCombatProfile,
            },
            pve_data::PveEncounterDatabase,
            skill_fragment_data::{
                SkillFragmentAcquisitionSource, SkillFragmentCompatibilityRequirements,
                SkillFragmentDatabase, SkillFragmentEffectDef, SkillFragmentEquipLimit,
                SkillFragmentId, SkillFragmentMetadata, SkillFragmentOrigin, SkillFragmentRarity,
            },
            visual_novel_data::{
                VisualNovelBackground, VisualNovelDatabase, VisualNovelDefinition, VisualNovelId,
                VisualNovelPresentationMode, VisualNovelSceneDefinition, VisualNovelSceneId,
                VisualNovelSceneNext, VisualNovelTextKind,
            },
            visual_novel_trigger_data::{
                VisualNovelTrigger, VisualNovelTriggerCheckpointPolicy, VisualNovelTriggerDatabase,
                VisualNovelTriggerId, VisualNovelTriggerRepeat, VisualNovelTriggerSource,
            },
            GameDataBase, GameDataBuilder,
        },
        employee::{
            CombatAptitude, EmployeeBaseCombatStats, EmployeeDefinition, EmployeeInitialLoadout,
            EmployeePersonality, EmployeeProfile, EmployeeRiskDisposition,
            EmployeeSocialDisposition, EventAptitude,
        },
        enums::RiskLevel,
        map::{GameMode, MapNodeCategory, MapNodeState},
    };
    use std::{
        sync::{Arc, Mutex},
        time::Duration,
    };
    use uuid::Uuid;

    struct ProbeSession {
        disconnects: Arc<Mutex<Vec<(String, String)>>>,
        messages: Arc<Mutex<Vec<PlayerGameServerMessage>>>,
    }

    impl Actor for ProbeSession {
        type Context = Context<Self>;
    }

    impl Handler<ForceDisconnect> for ProbeSession {
        type Result = ();

        fn handle(&mut self, msg: ForceDisconnect, _ctx: &mut Self::Context) -> Self::Result {
            self.disconnects
                .lock()
                .expect("disconnects mutex should be lockable")
                .push((msg.code, msg.message));
        }
    }

    impl Handler<PlayerGameServerMessage> for ProbeSession {
        type Result = ();

        fn handle(
            &mut self,
            msg: PlayerGameServerMessage,
            _ctx: &mut Self::Context,
        ) -> Self::Result {
            self.messages
                .lock()
                .expect("messages mutex should be lockable")
                .push(msg);
        }
    }

    fn spawn_probe() -> (
        actix::Addr<ProbeSession>,
        Arc<Mutex<Vec<(String, String)>>>,
        Arc<Mutex<Vec<PlayerGameServerMessage>>>,
    ) {
        let disconnects = Arc::new(Mutex::new(Vec::new()));
        let messages = Arc::new(Mutex::new(Vec::new()));
        let addr = ProbeSession {
            disconnects: disconnects.clone(),
            messages: messages.clone(),
        }
        .start();

        (addr, disconnects, messages)
    }

    fn starter_test_loadout(index: usize) -> EmployeeInitialLoadout {
        EmployeeInitialLoadout {
            active_skill_fragment_id: None,
            equipment_ids: vec![
                match index / 2 {
                    0 => "standard_training_sword",
                    1 => "standard_training_shield",
                    _ => "standard_training_healing_staff",
                }.to_string(),
                "standard_armor".to_string(),
            ],
            baseline_skill_fragment_ids: vec![SkillFragmentId::from(
                "starter_basic_attack_enhancement",
            )],
        }
    }

    fn test_employee_definition(index: usize, starter_eligible: bool) -> EmployeeDefinition {
        EmployeeDefinition {
            employee_definition_id: if starter_eligible {
                format!("candidate_{index}")
            } else {
                format!("recruit_{index}")
            },
            name: format!("Candidate {index}"),
            role: match index / 2 {
                0 => game_core::game::employee::EmployeeRole::Dealer,
                1 => game_core::game::employee::EmployeeRole::Tank,
                _ => game_core::game::employee::EmployeeRole::Healer,
            },
            starter_eligible,
            initial_loadout: if starter_eligible {
                starter_test_loadout(index)
            } else {
                EmployeeInitialLoadout::default()
            },
            profile: EmployeeProfile {
                role: "Test Role".to_string(),
                background: "Test Background".to_string(),
                personality: EmployeePersonality {
                    risk: EmployeeRiskDisposition::Cautious,
                    social: EmployeeSocialDisposition::Obedient,
                },
                combat_aptitude: CombatAptitude::Balanced,
                event_aptitude: EventAptitude::Maintenance,
                base_combat_stats: EmployeeBaseCombatStats::default(),
                basic_attack_speed_percent: 100,
            },
        }
    }

    fn starter_test_equipment() -> EquipmentMetadata {
        EquipmentMetadata {
            id: "standard_armor".to_string(),
            uuid: Uuid::parse_str("650e8400-e29b-41d4-a716-446655440031")
                .expect("standard armor uuid should be valid"),
            name: "Standard Training E.G.O Armor".to_string(),
            equipment_type: EquipmentType::Armor,
            rarity: RiskLevel::TETH,
            price: 80,
            allow_duplicate_equip: true,
            bound: false,
            cannot_unequip_reason: "equipment_bound".to_string(),
            triggered_effects: Default::default(),
            ability_activations: Vec::new(),
            weapon_profile: None,
            event_check_modifiers: vec![],
            event_choice_keys: vec![],
        }
    }

    fn starter_test_weapon() -> EquipmentMetadata {
        let mut weapon_profile = WeaponCombatProfile::default();
        weapon_profile.block_capacity = 1;
        weapon_profile.ground_defense_focus = Some(GroundDefenseFocusProfile {
            block_capacity_bonus: 1,
            incoming_damage_reduction_percent: 10,
        });
        EquipmentMetadata {
            id: "standard_training_sword".to_string(),
            uuid: Uuid::parse_str("650e8400-e29b-41d4-a716-446655440030")
                .expect("standard training sword uuid should be valid"),
            name: "Standard Training Sword".to_string(),
            equipment_type: EquipmentType::Weapon,
            rarity: RiskLevel::ZAYIN,
            price: 0,
            allow_duplicate_equip: true,
            bound: false,
            cannot_unequip_reason: "equipment_bound".to_string(),
            triggered_effects: Default::default(),
            ability_activations: Vec::new(),
            weapon_profile: Some(weapon_profile),
            event_check_modifiers: vec![],
            event_choice_keys: vec![],
        }
    }

    fn starter_test_skill_fragment() -> SkillFragmentMetadata {
        SkillFragmentMetadata {
            corrosion_class:
                game_core::game::data::skill_fragment_data::FragmentCorrosionClass::Ordinary,
            id: SkillFragmentId::from("starter_basic_attack_enhancement"),
            uuid: Uuid::parse_str("53544152-5445-525f-4652-414700000001")
                .expect("starter basic attack fragment uuid should be valid"),
            name: "Starter Basic Attack Enhancement".to_string(),
            description:
                "A baseline fragment that lets employees perform reinforced basic attacks."
                    .to_string(),
            rarity: SkillFragmentRarity::Common,
            equip_limit: SkillFragmentEquipLimit::OwnedCopies,
            origin: Some(SkillFragmentOrigin::Concept {
                concept_id: "employee_baseline_training".to_string(),
            }),
            sources: vec![SkillFragmentAcquisitionSource::DependentConcept {
                concept_id: "employee_baseline_training".to_string(),
            }],
            dependencies: Vec::new(),
            compatibility: SkillFragmentCompatibilityRequirements::default(),
            effect: SkillFragmentEffectDef::BasicAttackModifier {
                attack_bonus: 2,
                attack_interval_ms_reduction: 0,
            },
        }
    }

    fn starter_policy_game_data_builder() -> GameDataBuilder {
        let (visual_novels, visual_novel_triggers) = test_hq_briefing_visual_novel_fixture();
        let mut policy = game_core::game::data::run_policy_data::RunPolicyData::builtin();
        policy.tutorial_employee_definition_ids = vec!["candidate_0".into(), "candidate_2".into(), "candidate_4".into()];
        let mut shield = starter_test_weapon();
        shield.id = "standard_training_shield".into();
        shield.uuid = Uuid::from_u128(0x650e_8400_e29b_41d4_a716_4466_5544_9032);
        shield.weapon_profile.as_mut().unwrap().weapon_archetype = game_core::game::data::equipment_data::WeaponArchetype::Shield;
        let mut staff = starter_test_weapon();
        staff.id = "standard_training_healing_staff".into();
        staff.uuid = Uuid::from_u128(0x650e_8400_e29b_41d4_a716_4466_5544_9033);
        let profile = staff.weapon_profile.as_mut().unwrap();
        profile.weapon_archetype = game_core::game::data::equipment_data::WeaponArchetype::Staff;
        profile.range_role = game_core::game::data::equipment_data::WeaponRangeRole::Ranged;
        profile.action = game_core::game::data::abnormality_data::BasicAttackAction::Heal {
            flat: 8, spell_power_percent: 100, footprint: None,
        };
        GameDataBuilder::empty()
            .with_run_policy(policy)
            .with_equipment(vec![starter_test_weapon(), starter_test_equipment(), shield, staff])
            .with_skill_fragments(SkillFragmentDatabase::new(vec![
                starter_test_skill_fragment(),
            ]))
            .with_visual_novels(visual_novels)
            .with_visual_novel_triggers(visual_novel_triggers)
    }

    fn test_hq_briefing_visual_novel_fixture() -> (VisualNovelDatabase, VisualNovelTriggerDatabase)
    {
        let standard_vn_id = "hq_briefing_standard_intro";
        let endless_vn_id = "hq_briefing_endless_intro";
        (
            VisualNovelDatabase::new(vec![
                test_hq_briefing_visual_novel_definition(standard_vn_id),
                test_hq_briefing_visual_novel_definition(endless_vn_id),
            ]),
            VisualNovelTriggerDatabase::new(vec![
                test_run_start_visual_novel_trigger(
                    "run_start_standard_hq_briefing",
                    GameMode::Tutorial,
                    standard_vn_id,
                ),
                test_run_start_visual_novel_trigger(
                    "run_start_endless_hq_briefing",
                    GameMode::Expedition,
                    endless_vn_id,
                ),
            ]),
        )
    }

    fn test_hq_briefing_visual_novel_definition(id: &str) -> VisualNovelDefinition {
        VisualNovelDefinition {
            id: VisualNovelId::new(id),
            entry_scene_id: VisualNovelSceneId::new("start"),
            default_presentation_mode: VisualNovelPresentationMode::Cinematic,
            scenes: vec![VisualNovelSceneDefinition {
                id: VisualNovelSceneId::new("start"),
                text_kind: VisualNovelTextKind::Narration,
                background: VisualNovelBackground::CurrentScreenDimmed,
                script_id: id.to_string(),
                speakers: vec![],
                next: VisualNovelSceneNext::End,
            }],
        }
    }

    fn test_run_start_visual_novel_trigger(
        id: &str,
        game_mode: GameMode,
        vn_id: &str,
    ) -> VisualNovelTrigger {
        VisualNovelTrigger {
            id: VisualNovelTriggerId::new(id),
            source: VisualNovelTriggerSource::RunStart { game_mode },
            vn_id: VisualNovelId::new(vn_id),
            repeat: VisualNovelTriggerRepeat::OncePerRun,
            checkpoint_policy: VisualNovelTriggerCheckpointPolicy::PersistAcrossCheckpointLoad,
            priority: 100,
            blocking: true,
        }
    }

    fn empty_game_data() -> Arc<GameDataBase> {
        let definitions = (0..6)
            .map(|index| test_employee_definition(index, true))
            .collect();

        starter_policy_game_data_builder()
            .with_employee_definitions(EmployeeDefinitionDatabase::new(definitions))
            .build_arc()
    }

    fn defense_game_data() -> Arc<GameDataBase> {
        defense_game_data_with_survive_timer(None)
    }

    fn defense_game_data_with_survive_timer(survive_timer_ms: Option<u64>) -> Arc<GameDataBase> {
        defense_game_data_with_profile(
            survive_timer_ms,
            game_core::runtime_safety::RuntimeSafetySettings::embedded_shared(),
        )
    }

    fn defense_game_data_with_profile(
        survive_timer_ms: Option<u64>,
        profile: Arc<game_core::runtime_safety::RuntimeSafetySettings>,
    ) -> Arc<GameDataBase> {
        let base = game_core::game::data::ValidatedGameData::load_live_embedded()
            .map(|data| data.into_fixture_database_arc())
            .expect("embedded live gameplay RON must be valid");
        let mut policy = base.run_policy.as_ref().clone();
        policy.tutorial_employee_definition_ids = vec!["candidate_0".into(), "candidate_2".into(), "candidate_4".into()];
        let mut pve_encounters = base.pve_data.encounters.clone();
        if let Some(survive_timer_ms) = survive_timer_ms {
            for encounter in &mut pve_encounters {
                if encounter.node_type == Some(CombatNodeType::Defense) {
                    let victory = encounter
                        .terminal_outcomes
                        .iter_mut()
                        .find(|outcome| {
                            outcome.verdict
                                == game_core::game::battle::types::BattleVerdict::Victory
                        })
                        .expect("Defense fixture Victory outcome");
                    victory.condition =
                        game_core::game::data::pve_data::PveTerminalConditionData::TimeReached {
                            time_ms: survive_timer_ms,
                        };
                    for wave in &mut encounter.waves {
                        wave.required_for_victory = false;
                    }
                }
            }
        }

        Arc::new(
            starter_policy_game_data_builder()
                .with_abnormality_data(base.abnormality_data.clone())
                .with_corroded_employee_data(base.corroded_employee_data.clone())
                .with_corroded_wave_data(base.corroded_wave_data.clone())
                .with_employee_definitions(EmployeeDefinitionDatabase::new(
                    (0..6)
                        .map(|index| test_employee_definition(index, true))
                        .collect(),
                ))
                .with_consumable_data(base.consumable_data.clone())
                .with_equipment_data(base.equipment_data.clone())
                .with_shop_data(base.shop_data.clone())
                .with_reward_data(base.reward_data.clone())
                .with_event_data(base.event_data.clone())
                .with_visual_novel_data(base.visual_novel_data.clone())
                .with_pve(PveEncounterDatabase::new(pve_encounters))
                .with_boss_runtime_data(base.boss_runtime_data.clone())
                .with_boss_omen_data(base.boss_omen_data.clone())
                .with_run_policy(policy)
                .with_skill_data(base.skill_data.clone())
                .with_buff_data(base.buff_data.clone())
                .with_skill_fragment_data(base.skill_fragment_data.clone())
                .build_with_runtime_safety(profile)
                .unwrap(),
        )
    }

    #[test]
    fn prepared_battle_entry_drop_preserves_root_and_commit_matches_direct_execution() {
        let player = Uuid::from_u128(0xC252);
        let mut app = game_core_waiting_on_live_defense_node(player);
        let before = app.snapshot(ApplicationSnapshotKind::Run).unwrap();
        let checkpoint = app.export_active_run_checkpoint().unwrap();
        let prepared = app
            .prepare_non_battle_command(
                player,
                command!(ConfirmEnterNode),
                ApplicationExecutionContext::default(),
            )
            .unwrap();
        assert!(prepared.enters_battle());
        let expected = prepared.result().clone();
        drop(prepared);
        assert!(!app.has_active_battle());
        assert_eq!(app.snapshot(ApplicationSnapshotKind::Run).unwrap(), before);
        assert_eq!(app.export_active_run_checkpoint().unwrap(), checkpoint);
        let prepared = app
            .prepare_non_battle_command(
                player,
                command!(ConfirmEnterNode),
                ApplicationExecutionContext::default(),
            )
            .unwrap();
        let projected_checkpoint = prepared.checkpoint().cloned();
        assert_eq!(prepared.commit(), expected);
        assert!(app.has_active_battle());
        assert_eq!(
            app.export_active_run_checkpoint().unwrap(),
            projected_checkpoint
        );
        let after = app.active_battle_reconnect(0).unwrap();
        let error = match app.prepare_non_battle_command(
            player,
            command!(RequestBattleState),
            ApplicationExecutionContext::default(),
        ) {
            Ok(_) => panic!("active battle must not admit preparation"),
            Err(error) => error,
        };
        assert_eq!(error.code(), "active_battle_preparation");
        assert_eq!(app.active_battle_reconnect(0).unwrap(), after);
        let mut direct = game_core_waiting_on_live_defense_node(player);
        assert_eq!(
            direct
                .execute_fixture(player, command!(ConfirmEnterNode))
                .unwrap(),
            expected
        );
        assert_eq!(
            direct.snapshot(ApplicationSnapshotKind::Run).unwrap(),
            app.snapshot(ApplicationSnapshotKind::Run).unwrap()
        );
    }

    #[test]
    fn first_public_battle_entry_uses_host_work_limit_and_active_battle_has_no_rollback_token() {
        let player = Uuid::from_u128(0xC250);
        let raw = std::str::from_utf8(game_core::runtime_safety::EMBEDDED_RUNTIME_SAFETY_RON)
            .unwrap()
            .replacen("max_work: 10000000", "max_work: 1", 1);
        let profile = Arc::new(
            game_core::runtime_safety::RuntimeSafetySettings::from_ron(
                "small-host",
                raw.as_bytes(),
            )
            .unwrap(),
        );
        let mut limited = game_core_waiting_on_live_defense_node_with_data(
            player,
            defense_game_data_with_profile(None, profile),
        );
        let before = limited.snapshot(ApplicationSnapshotKind::Run).unwrap();
        let failure = limited
            .execute_fixture(player, command!(ConfirmEnterNode))
            .unwrap_err();
        assert_eq!(failure.code(), "battle_runtime_work_limit");
        assert_eq!(
            limited.snapshot(ApplicationSnapshotKind::Run).unwrap(),
            before
        );
        assert!(!limited.has_active_battle());
        let failure = match limited.prepare_non_battle_command(
            player,
            command!(ConfirmEnterNode),
            ApplicationExecutionContext::default(),
        ) {
            Ok(_) => panic!("preparation must retain the same native work limit"),
            Err(error) => error,
        };
        assert_eq!(failure.code(), "battle_runtime_work_limit");
        assert_eq!(
            limited.snapshot(ApplicationSnapshotKind::Run).unwrap(),
            before
        );
        assert!(!limited.has_active_battle());
        let mut normal = game_core_waiting_on_live_defense_node(player);
        let before_entry = normal.capture_persistence_rollback().unwrap().unwrap();
        normal
            .execute_fixture(player, command!(ConfirmEnterNode))
            .unwrap();
        assert!(normal.has_active_battle());
        assert!(normal.capture_persistence_rollback().unwrap().is_none());
        let run_after_entry = normal.snapshot(ApplicationSnapshotKind::Run).unwrap();
        let battle_after_entry = normal.active_battle_reconnect(0).unwrap();
        assert_eq!(
            normal
                .restore_persistence_rollback(before_entry)
                .unwrap_err()
                .code(),
            "active_battle_persistence_rollback"
        );
        assert!(normal.has_active_battle());
        assert_eq!(
            normal.snapshot(ApplicationSnapshotKind::Run).unwrap(),
            run_after_entry
        );
        assert_eq!(
            normal.active_battle_reconnect(0).unwrap(),
            battle_after_entry
        );
    }

    fn start_new_game_with_default_starters(
        core: &mut CoreApplication,
        player_id: Uuid,
    ) -> ApplicationResult {
        let start = core
            .execute_fixture(
                player_id,
                command!(StartNewGame {
                    game_mode: GameMode::Expedition,
                    replace_active_run: false,
                }),
            )
            .expect("start new game should open pre-run briefing");
        assert_eq!(start.kind(), ApplicationResultKind::RunStartFlowStarted);
        let visual_novel = &start.payload().as_value()["visual_novel"];
        assert!(visual_novel.is_object());
        let advanced = core
            .execute_fixture(
                player_id,
                command!(AdvanceVisualNovel {
                    session_id: visual_novel["session_id"].clone(),
                    current_scene_id: visual_novel["current_scene_id"].clone(),
                }),
            )
            .expect("HQ briefing advance should open starter selection");
        assert!(
            advanced.kind() == ApplicationResultKind::VisualNovelState
                && advanced.payload().as_value()["visual_novel"].is_null(),
            "single-scene test HQ briefing should complete"
        );
        let snapshot = core.run_snapshot_fixture();
        let GameStateContextDto::SelectingStarterEmployees {
            candidates,
            required_count,
        } = snapshot.game_state_context
        else {
            panic!("HQ briefing completion should expose starter selection context");
        };
        let employee_definition_ids = candidates
            .into_iter()
            .take(required_count)
            .map(|definition| definition.employee_definition_id)
            .collect::<Vec<_>>();
        core.execute_fixture(
            player_id,
            command!(SelectStarterEmployees {
                employee_definition_ids,
                loan_copy_ids: Vec::<Uuid>::new(),
            }),
        )
        .expect("default starter employee selection should start the run")
    }

    fn game_core_waiting_on_live_defense_node(player_id: Uuid) -> CoreApplication {
        game_core_waiting_on_live_defense_node_with_data(player_id, defense_game_data())
    }

    fn game_core_waiting_on_live_survival_defense_node(player_id: Uuid) -> CoreApplication {
        game_core_waiting_on_live_defense_node_with_data(
            player_id,
            defense_game_data_with_survive_timer(Some(1)),
        )
    }

    fn game_core_waiting_on_live_defense_node_with_data(
        player_id: Uuid,
        game_data: Arc<GameDataBase>,
    ) -> CoreApplication {
        for seed in 0..1_000 {
            let mut core = CoreApplication::new(game_data.clone(), seed);
            let selected = start_new_game_with_default_starters(&mut core, player_id);
            assert_eq!(
                selected.kind(),
                ApplicationResultKind::StarterEmployeesSelected
            );
            let mut map: game_core::game::map::MapViewDto =
                serde_json::from_value(selected.payload().as_value()["map"].clone()).unwrap();

            for _ in 0..3 {
                let available_node_ids = map
                    .nodes
                    .iter()
                    .filter(|node| {
                        node.state == MapNodeState::Available
                            && node.category == MapNodeCategory::Combat
                    })
                    .map(|node| node.id)
                    .collect::<Vec<_>>();
                for node_id in available_node_ids {
                    let preview = core
                        .execute_fixture(player_id, command!(SelectMapNode { node_id }))
                        .expect("available node preview should succeed");
                    let is_live_defense = preview.kind() == ApplicationResultKind::NodePreview
                        && preview.payload().as_value()["combat_preview"]["node_type"]
                            == serde_json::to_value(CombatNodeType::Defense).unwrap();
                    if is_live_defense {
                        return core;
                    }

                    core.execute_fixture(player_id, command!(CancelSelectedNode))
                        .expect("non-defense node preview should be cancellable");
                }

                let Some(start_node_id) = map
                    .nodes
                    .iter()
                    .find(|node| {
                        node.state == MapNodeState::Available
                            && node.category == MapNodeCategory::Start
                    })
                    .map(|node| node.id)
                else {
                    break;
                };
                let preview = core
                    .execute_fixture(
                        player_id,
                        command!(SelectMapNode {
                            node_id: start_node_id,
                        }),
                    )
                    .expect("start node preview should succeed");
                assert_eq!(preview.kind(), ApplicationResultKind::NodePreview);
                assert_eq!(
                    preview.payload().as_value()["category"],
                    serde_json::to_value(MapNodeCategory::Start).unwrap()
                );
                let entered = core
                    .execute_fixture(player_id, command!(ConfirmEnterNode))
                    .expect("start node confirm should succeed");
                assert_eq!(entered.kind(), ApplicationResultKind::NodeEntered);
                assert_eq!(
                    entered.payload().as_value()["category"],
                    serde_json::to_value(MapNodeCategory::Start).unwrap()
                );
                let completed = core
                    .execute_fixture(player_id, command!(CompleteNode))
                    .expect("start node complete should succeed");
                assert_eq!(completed.kind(), ApplicationResultKind::NodeCompleted);
                let next_map =
                    serde_json::from_value(completed.payload().as_value()["map"].clone()).unwrap();
                map = next_map;
            }
        }

        panic!("expected at least one generated run seed to start near a live Defense node");
    }

    #[test]
    fn player_state_snapshot_preserves_core_snapshot_root_fields() {
        let core = CoreApplication::new(empty_game_data(), 7);
        let core_snapshot = core.run_snapshot_fixture();
        let core_json =
            serde_json::to_value(&core_snapshot).expect("core snapshot should serialize");
        let player_json = serde_json::to_value(PlayerStateSnapshotDto::new(core_snapshot, None))
            .expect("player snapshot should serialize");

        let core_fields = core_json
            .as_object()
            .expect("core snapshot should be an object")
            .keys()
            .cloned()
            .collect::<std::collections::BTreeSet<_>>();
        let player_fields = player_json
            .as_object()
            .expect("player snapshot should be an object")
            .keys()
            .cloned()
            .collect::<std::collections::BTreeSet<_>>();

        assert_eq!(
            player_fields, core_fields,
            "server snapshot wrapper must not maintain a second root field list"
        );

        for field in core_fields {
            assert_eq!(
                player_json.get(&field),
                core_json.get(&field),
                "server snapshot should preserve core field `{field}` when no transport attachment is added"
            );
        }
    }

    #[derive(Debug, Clone, Copy)]
    enum SnapshotOverlayGameState {
        Title,
        OrdinaryNonBattle,
        ActiveBattle,
    }

    #[derive(Debug, Clone, Copy)]
    enum SnapshotOverlayActiveRun {
        Unconfigured,
        Missing,
        LoadedCurrent,
        LoadedBackup,
        Corrupt,
    }

    #[derive(Debug, Clone, Copy)]
    enum SnapshotOverlayCompendium {
        Unconfigured,
        Clean,
        Recovered,
        Dirty,
        Fatal,
    }

    struct ActiveRunSnapshotFixture {
        _directory: Option<tempfile::TempDir>,
        store: Option<Arc<Mutex<ActiveRunStore>>>,
        status: Option<Value>,
        action: Option<&'static str>,
    }

    struct CompendiumSnapshotFixture {
        _directory: Option<tempfile::TempDir>,
        persistence: Option<Arc<Mutex<PlayerProfilePersistence>>>,
        status: Option<Value>,
        dirty: bool,
        reset_required: bool,
    }

    fn snapshot_overlay_core(state: SnapshotOverlayGameState, player_id: Uuid) -> CoreApplication {
        match state {
            SnapshotOverlayGameState::Title => CoreApplication::new(empty_game_data(), 7),
            SnapshotOverlayGameState::OrdinaryNonBattle => {
                let mut core = CoreApplication::new(empty_game_data(), 7);
                let result = core
                    .execute_fixture(
                        player_id,
                        command!(StartNewGame {
                            game_mode: GameMode::Tutorial,
                            replace_active_run: false,
                        }),
                    )
                    .expect("ordinary non-battle fixture should enter pre-run briefing");
                assert_eq!(result.kind(), ApplicationResultKind::RunStartFlowStarted);
                core
            }
            SnapshotOverlayGameState::ActiveBattle => {
                let mut core = game_core_waiting_on_live_defense_node(player_id);
                let result = core
                    .execute_fixture(player_id, command!(ConfirmEnterNode))
                    .expect("active-battle fixture should enter the selected defense node");
                assert_eq!(result.kind(), ApplicationResultKind::BattleAdvanced);
                assert!(!result.payload().as_value()["battle_setup_snapshot"].is_null());
                core
            }
        }
    }

    fn active_run_snapshot_fixture(state: SnapshotOverlayActiveRun) -> ActiveRunSnapshotFixture {
        if matches!(state, SnapshotOverlayActiveRun::Unconfigured) {
            return ActiveRunSnapshotFixture {
                _directory: None,
                store: None,
                status: None,
                action: None,
            };
        }

        let directory = tempfile::tempdir().unwrap();
        let store = ActiveRunStore::open(directory.path()).unwrap();
        let checkpoint = |loads_used| ActiveRunCheckpointExport {
            core_payload_json: serde_json::to_vec(
                &serde_json::json!({"version":12,"fixture_value":loads_used}),
            )
            .unwrap(),
            loads_used,
            run_peak_certified_power: 100 + u64::from(loads_used),
        };
        let (status, action) = match state {
            SnapshotOverlayActiveRun::Unconfigured => unreachable!(),
            SnapshotOverlayActiveRun::Missing => (
                serde_json::json!({
                    "available": false,
                    "recovered_from_backup": false,
                    "reset_required": false,
                }),
                None,
            ),
            SnapshotOverlayActiveRun::LoadedCurrent => {
                store.publish(&checkpoint(1)).unwrap();
                (
                    serde_json::json!({
                        "available": true,
                        "recovered_from_backup": false,
                        "reset_required": false,
                    }),
                    Some("ContinueRun"),
                )
            }
            SnapshotOverlayActiveRun::LoadedBackup => {
                store.publish(&checkpoint(1)).unwrap();
                store.publish(&checkpoint(2)).unwrap();
                std::fs::write(directory.path().join("active_run.json"), b"corrupt").unwrap();
                (
                    serde_json::json!({
                        "available": true,
                        "recovered_from_backup": true,
                        "reset_required": false,
                    }),
                    Some("ContinueRun"),
                )
            }
            SnapshotOverlayActiveRun::Corrupt => {
                store.publish(&checkpoint(1)).unwrap();
                std::fs::write(directory.path().join("active_run.json"), b"corrupt").unwrap();
                std::fs::write(
                    directory.path().join("active_run.backup.json"),
                    b"corrupt backup",
                )
                .unwrap();
                let system_issue = store
                    .load()
                    .expect_err("corrupt fixture should have no valid backup")
                    .to_string();
                (
                    serde_json::json!({
                        "available": false,
                        "recovered_from_backup": false,
                        "reset_required": true,
                        "system_issue": system_issue,
                    }),
                    Some("ResetActiveRun"),
                )
            }
        };

        ActiveRunSnapshotFixture {
            _directory: Some(directory),
            store: Some(Arc::new(Mutex::new(store))),
            status: Some(status),
            action,
        }
    }

    fn compendium_snapshot_fixture(state: SnapshotOverlayCompendium) -> CompendiumSnapshotFixture {
        if matches!(state, SnapshotOverlayCompendium::Unconfigured) {
            return CompendiumSnapshotFixture {
                _directory: None,
                persistence: None,
                status: None,
                dirty: false,
                reset_required: false,
            };
        }

        let directory = tempfile::tempdir().unwrap();
        let store = PlayerProfileStore::open(directory.path()).unwrap();
        let profile =
            |id: &str| game_core::game::encounter_compendium::EncounterCompendiumProfileState {
                registered_ids: std::collections::BTreeSet::from([id.to_string()]),
                ..Default::default()
            };
        match state {
            SnapshotOverlayCompendium::Unconfigured => unreachable!(),
            SnapshotOverlayCompendium::Clean => {}
            SnapshotOverlayCompendium::Recovered => {
                store.publish(&profile("first")).unwrap();
                store.publish(&profile("second")).unwrap();
                std::fs::write(directory.path().join("player_profile.json"), b"corrupt").unwrap();
            }
            SnapshotOverlayCompendium::Dirty => {}
            SnapshotOverlayCompendium::Fatal => {
                store.publish(&profile("fatal")).unwrap();
                std::fs::write(directory.path().join("player_profile.json"), b"corrupt").unwrap();
                std::fs::write(
                    directory.path().join("player_profile.backup.json"),
                    b"corrupt backup",
                )
                .unwrap();
            }
        }
        let mut persistence = PlayerProfilePersistence::bootstrap(store);
        if matches!(state, SnapshotOverlayCompendium::Dirty) {
            std::fs::create_dir(directory.path().join("player_profile.tmp")).unwrap();
            assert_eq!(
                persistence.merge_and_persist(profile("dirty")).unwrap(),
                PlayerProfilePersistOutcome::BecameDirty
            );
        }
        let (status, dirty, reset_required) = match persistence.state() {
            PlayerProfilePersistenceState::Clean => (
                serde_json::json!({
                    "state": "clean",
                    "dirty": false,
                    "recovered_from_backup": false,
                    "reset_required": false,
                    "system_issue": null,
                }),
                false,
                false,
            ),
            PlayerProfilePersistenceState::RecoveredFromBackup => (
                serde_json::json!({
                    "state": "recovered",
                    "dirty": true,
                    "recovered_from_backup": true,
                    "reset_required": false,
                    "system_issue": null,
                }),
                true,
                false,
            ),
            PlayerProfilePersistenceState::Dirty { issue } => (
                serde_json::json!({
                    "state": "dirty",
                    "dirty": true,
                    "recovered_from_backup": false,
                    "reset_required": false,
                    "system_issue": issue,
                }),
                true,
                false,
            ),
            PlayerProfilePersistenceState::ApplicationRecoveryPending { issue } => (
                serde_json::json!({
                    "state": "application_recovery_pending",
                    "dirty": true,
                    "recovered_from_backup": false,
                    "reset_required": true,
                    "system_issue": issue,
                }),
                true,
                true,
            ),
            PlayerProfilePersistenceState::Fatal { issue } => (
                serde_json::json!({
                    "state": "error",
                    "dirty": false,
                    "recovered_from_backup": false,
                    "reset_required": true,
                    "system_issue": issue,
                }),
                false,
                true,
            ),
            PlayerProfilePersistenceState::Protected { issue } => (
                serde_json::json!({"state":"protected", "dirty":false, "recovered_from_backup":false, "reset_required":false, "system_issue":issue}),
                true,
                true,
            ),
        };

        CompendiumSnapshotFixture {
            _directory: Some(directory),
            persistence: Some(Arc::new(Mutex::new(persistence))),
            status: Some(status),
            dirty,
            reset_required,
        }
    }

    fn expected_host_capability(action: &'static str) -> Value {
        serde_json::json!({
            "action": action,
            "scope": "recovery",
            "enabled": true,
        })
    }

    #[actix_web::test]
    async fn state_snapshot_overlay_matrix_preserves_old_full_json_order_and_uniqueness() {
        let game_states = [
            SnapshotOverlayGameState::Title,
            SnapshotOverlayGameState::OrdinaryNonBattle,
            SnapshotOverlayGameState::ActiveBattle,
        ];
        let active_run_states = [
            SnapshotOverlayActiveRun::Unconfigured,
            SnapshotOverlayActiveRun::Missing,
            SnapshotOverlayActiveRun::LoadedCurrent,
            SnapshotOverlayActiveRun::LoadedBackup,
            SnapshotOverlayActiveRun::Corrupt,
        ];
        let compendium_states = [
            SnapshotOverlayCompendium::Unconfigured,
            SnapshotOverlayCompendium::Clean,
            SnapshotOverlayCompendium::Recovered,
            SnapshotOverlayCompendium::Dirty,
            SnapshotOverlayCompendium::Fatal,
        ];
        let load_balance = LoadBalanceActor::new().start();
        let player_id = Uuid::new_v4();
        let mut rows = 0;

        for game_state in game_states {
            let core = snapshot_overlay_core(game_state, player_id);
            let mut actor = PlayerGameActor::new(player_id, core, load_balance.clone());
            for active_run_state in active_run_states {
                for compendium_state in compendium_states {
                    let active_run = active_run_snapshot_fixture(active_run_state);
                    let compendium = compendium_snapshot_fixture(compendium_state);
                    actor.active_run_store = active_run.store.clone();
                    actor.player_profile_persistence = compendium.persistence.clone();

                    let core_snapshot = actor.game_core.run_snapshot_fixture();
                    let mut expected =
                        serde_json::to_value(PlayerStateSnapshotDto::new(core_snapshot, None))
                            .expect("matrix Core snapshot should serialize");
                    let is_title = matches!(game_state, SnapshotOverlayGameState::Title);

                    if is_title {
                        if let Some(status) = active_run.status.clone() {
                            expected["active_run_continue"] = status;
                        }
                        if let Some(action) = active_run.action {
                            expected["action_capabilities"]
                                .as_array_mut()
                                .unwrap()
                                .push(expected_host_capability(action));
                        }
                    }
                    if let Some(status) = compendium.status.clone() {
                        expected["player_profile_persistence"] = status;
                        if is_title && compendium.reset_required {
                            expected["action_capabilities"] = serde_json::json!([]);
                        }
                        if is_title {
                            let capabilities =
                                expected["action_capabilities"].as_array_mut().unwrap();
                            if compendium.dirty || compendium.reset_required {
                                capabilities.push(expected_host_capability("RetryPlayerProfile"));
                            }
                            capabilities.push(expected_host_capability("ResetPlayerProfile"));
                        }
                    }

                    let actual = actor
                        .build_state_snapshot()
                        .expect("matrix server snapshot should build");
                    assert_eq!(
                        actual, expected,
                        "full JSON mismatch for {game_state:?} × {active_run_state:?} × {compendium_state:?}"
                    );
                    assert_eq!(
                        serde_json::to_vec(&actual).unwrap(),
                        serde_json::to_vec(&expected).unwrap(),
                        "serialized JSON mismatch for {game_state:?} × {active_run_state:?} × {compendium_state:?}"
                    );

                    let actions = actual["action_capabilities"]
                        .as_array()
                        .expect("capabilities should remain an array")
                        .iter()
                        .map(|capability| {
                            capability["action"]
                                .as_str()
                                .expect("each capability should have a typed action")
                        })
                        .collect::<Vec<_>>();
                    let unique_actions = actions
                        .iter()
                        .copied()
                        .collect::<std::collections::BTreeSet<_>>();
                    assert_eq!(
                        actions.len(),
                        unique_actions.len(),
                        "duplicate action for {game_state:?} × {active_run_state:?} × {compendium_state:?}: {actions:?}"
                    );
                    rows += 1;
                }
            }
        }

        assert_eq!(rows, 75, "the complete 3 × 5 × 5 matrix must execute");
    }

    #[actix_web::test]
    async fn execute_behavior_returns_updated_state_snapshot() {
        let load_balance = LoadBalanceActor::new().start();
        let actor = PlayerGameActor::new(
            Uuid::new_v4(),
            CoreApplication::new(empty_game_data(), 7),
            load_balance,
        )
        .start();

        let (probe, _, _) = spawn_probe();
        let session_id = Uuid::new_v4();

        let attach = actor
            .send(AttachSession {
                session_id,
                socket: probe.clone().recipient(),
                control: probe.recipient(),
            })
            .await
            .expect("attach request should complete")
            .expect("attach should succeed");

        assert_eq!(attach.state_snapshot["game_state"], "not_started");
        assert!(attach.side_messages.is_empty());

        let result = actor
            .send(ExecutePlayerBehavior {
                session_id,
                request_id: "req-1".to_string(),
                behavior: command!(StartNewGame {
                    game_mode: GameMode::Tutorial,
                    replace_active_run: false,
                }),
                battle_authority: None,
                battle_side_message: BattleSideMessageKind::BattleUpdate,
            })
            .await
            .expect("execute request should complete")
            .expect("execute should succeed");

        match result.response {
            PlayerGameServerMessage::CommandResult {
                request_id,
                result_type,
                ..
            } => {
                assert_eq!(request_id, "req-1");
                assert_eq!(result_type, "RunStartFlowStarted");
            }
            other => panic!("expected command_result, got {other:?}"),
        }

        assert_eq!(
            result
                .state_snapshot
                .expect("non-battle commands should send state_snapshot")["game_state"],
            "pre_run_briefing"
        );
    }

    #[actix_web::test]
    async fn fatal_compendium_bootstrap_gates_title_to_retry_and_confirmed_reset() {
        let directory = tempfile::tempdir().unwrap();
        let store = PlayerProfileStore::open(directory.path()).unwrap();
        store
            .publish(
                &game_core::game::encounter_compendium::EncounterCompendiumProfileState::default(),
            )
            .unwrap();
        std::fs::write(directory.path().join("player_profile.json"), b"corrupt").unwrap();
        std::fs::write(
            directory.path().join("player_profile.backup.json"),
            b"corrupt backup",
        )
        .unwrap();
        let persistence = Arc::new(Mutex::new(PlayerProfilePersistence::bootstrap(store)));

        let load_balance = LoadBalanceActor::new().start();
        let actor = PlayerGameActor::new(
            Uuid::new_v4(),
            CoreApplication::new(empty_game_data(), 7),
            load_balance,
        )
        .with_player_profile_persistence(persistence)
        .start();
        let (probe, _, _) = spawn_probe();

        let attach = actor
            .send(AttachSession {
                session_id: Uuid::new_v4(),
                socket: probe.clone().recipient(),
                control: probe.recipient(),
            })
            .await
            .unwrap()
            .unwrap();

        assert_eq!(
            attach.state_snapshot["player_profile_persistence"]["state"],
            "error"
        );
        let actions = attach.state_snapshot["action_capabilities"]
            .as_array()
            .unwrap()
            .iter()
            .map(|capability| capability["action"].as_str().unwrap())
            .collect::<std::collections::BTreeSet<_>>();
        assert_eq!(
            actions,
            std::collections::BTreeSet::from(["ResetPlayerProfile", "RetryPlayerProfile",])
        );
    }

    #[actix_web::test]
    async fn clean_title_exposes_confirmable_full_compendium_reset_without_retry() {
        let directory = tempfile::tempdir().unwrap();
        let persistence = Arc::new(Mutex::new(PlayerProfilePersistence::bootstrap(
            PlayerProfileStore::open(directory.path()).unwrap(),
        )));
        let load_balance = LoadBalanceActor::new().start();
        let actor = PlayerGameActor::new(
            Uuid::new_v4(),
            CoreApplication::new(empty_game_data(), 7),
            load_balance,
        )
        .with_player_profile_persistence(persistence)
        .start();
        let (probe, _, _) = spawn_probe();

        let attach = actor
            .send(AttachSession {
                session_id: Uuid::new_v4(),
                socket: probe.clone().recipient(),
                control: probe.recipient(),
            })
            .await
            .unwrap()
            .unwrap();
        let actions = attach.state_snapshot["action_capabilities"]
            .as_array()
            .unwrap();
        assert!(actions
            .iter()
            .any(|capability| capability["action"] == "ResetPlayerProfile"));
        assert!(actions
            .iter()
            .all(|capability| capability["action"] != "RetryPlayerProfile"));
    }

    #[actix_web::test]
    async fn pending_application_blocks_continue_and_retries_in_process() {
        let active_directory = tempfile::tempdir().unwrap();
        let active_store = Arc::new(Mutex::new(
            ActiveRunStore::open(active_directory.path()).unwrap(),
        ));
        active_store
            .lock()
            .unwrap()
            .publish(&ActiveRunCheckpointExport {
                core_payload_json: br#"{"version":12,"fixture":"pending-standard-completion"}"#
                    .to_vec(),
                loads_used: 0,
                run_peak_certified_power: 100,
            })
            .unwrap();

        let profile_directory = tempfile::tempdir().unwrap();
        let mut profile_persistence = PlayerProfilePersistence::bootstrap(
            PlayerProfileStore::open(profile_directory.path()).unwrap(),
        );
        std::fs::create_dir(profile_directory.path().join("player_profile.tmp")).unwrap();
        let expected = active_store.lock().unwrap().snapshot().unwrap().revision;
        let accepted = profile_persistence
            .accept_application(
                &active_store.lock().unwrap(),
                expected,
                None,
                &game_core::game::encounter_compendium::PlayerProfileState {
                    highest_floor_reached: 7,
                    ..Default::default()
                },
            )
            .unwrap();
        assert!(accepted.recovery_pending);
        let profile_persistence = Arc::new(Mutex::new(profile_persistence));

        let load_balance = LoadBalanceActor::new().start();
        let actor = PlayerGameActor::new(
            Uuid::new_v4(),
            CoreApplication::new(empty_game_data(), 7),
            load_balance,
        )
        .with_active_run_store(active_store.clone())
        .with_player_profile_persistence(profile_persistence.clone())
        .start();
        let (probe, _, _) = spawn_probe();
        let session_id = Uuid::new_v4();
        let attach = actor
            .send(AttachSession {
                session_id,
                socket: probe.clone().recipient(),
                control: probe.recipient(),
            })
            .await
            .unwrap()
            .unwrap();

        assert_eq!(
            attach.state_snapshot["player_profile_persistence"]["state"],
            "application_recovery_pending"
        );
        let actions = attach.state_snapshot["action_capabilities"]
            .as_array()
            .unwrap()
            .iter()
            .map(|capability| capability["action"].as_str().unwrap())
            .collect::<Vec<_>>();
        assert_eq!(actions, vec!["RetryPlayerProfile"]);

        std::fs::remove_dir(profile_directory.path().join("player_profile.tmp")).unwrap();
        let retried = actor
            .send(ExecutePlayerBehavior {
                session_id,
                request_id: "recover-standard-completion".to_string(),
                behavior: command!(RetryPlayerProfile),
                battle_authority: None,
                battle_side_message: BattleSideMessageKind::BattleUpdate,
            })
            .await
            .unwrap()
            .unwrap();

        assert_eq!(
            active_store.lock().unwrap().load().unwrap(),
            ActiveRunLoad::Missing
        );
        assert!(matches!(
            profile_persistence.lock().unwrap().state(),
            PlayerProfilePersistenceState::Clean
        ));
        assert_eq!(
            retried.state_snapshot.unwrap()["player_profile"]["highest_floor_reached"],
            7
        );
    }

    #[actix_web::test]
    async fn attach_session_merges_durable_compendium_before_building_snapshot() {
        let game_data = game_core::game::data::ValidatedGameData::load_live_embedded()
            .map(|data| data.into_fixture_database_arc())
            .expect("embedded live gameplay RON must be valid");
        let abnormality_id = game_data
            .abnormality_data
            .items
            .first()
            .expect("live catalog must contain an abnormality")
            .id
            .clone();
        let profile = game_core::game::encounter_compendium::EncounterCompendiumProfileState {
            registered_ids: std::collections::BTreeSet::from([abnormality_id.clone()]),
            ..Default::default()
        };
        let directory = tempfile::tempdir().unwrap();
        let store = PlayerProfileStore::open(directory.path()).unwrap();
        store.publish(&profile).unwrap();
        let persistence = Arc::new(Mutex::new(PlayerProfilePersistence::bootstrap(store)));

        let load_balance = LoadBalanceActor::new().start();
        let actor = PlayerGameActor::new(
            Uuid::new_v4(),
            CoreApplication::new(game_data, 7),
            load_balance,
        )
        .with_player_profile_persistence(persistence)
        .start();
        let (probe, _, _) = spawn_probe();

        let attach = actor
            .send(AttachSession {
                session_id: Uuid::new_v4(),
                socket: probe.clone().recipient(),
                control: probe.recipient(),
            })
            .await
            .unwrap()
            .unwrap();

        let projected = attach.state_snapshot["encounter_compendium"]["abnormalities"]
            .as_array()
            .expect("compendium abnormalities must be an array")
            .iter()
            .find(|entry| entry["abnormality_id"] == abnormality_id)
            .expect("durable registered abnormality must be projected on attach");
        assert_eq!(projected["registered"], true);
    }

    #[actix_web::test]
    async fn application_preintent_write_failure_keeps_core_and_profile_unchanged() {
        let active_directory = tempfile::tempdir().unwrap();
        let active_store = Arc::new(Mutex::new(
            ActiveRunStore::open(active_directory.path()).unwrap(),
        ));

        let profile_directory = tempfile::tempdir().unwrap();
        let profile_persistence = Arc::new(Mutex::new(PlayerProfilePersistence::bootstrap(
            PlayerProfileStore::open(profile_directory.path()).unwrap(),
        )));
        std::fs::create_dir(
            profile_directory
                .path()
                .join("application_commit.pending.tmp"),
        )
        .unwrap();
        let mut core = CoreApplication::new(empty_game_data(), 7);
        core.merge_compendium_fixture(
            game_core::game::encounter_compendium::EncounterCompendiumProfileState {
                registered_ids: std::collections::BTreeSet::from([
                    "dormant_before_active_failure".to_string()
                ]),
                ..Default::default()
            },
        )
        .unwrap();

        let load_balance = LoadBalanceActor::new().start();
        let actor = PlayerGameActor::new(Uuid::new_v4(), core, load_balance)
            .with_active_run_store(active_store)
            .with_player_profile_persistence(profile_persistence.clone())
            .start();
        let (probe, _, _) = spawn_probe();
        let session_id = Uuid::new_v4();
        actor
            .send(AttachSession {
                session_id,
                socket: probe.clone().recipient(),
                control: probe.clone().recipient(),
            })
            .await
            .unwrap()
            .unwrap();

        let error = actor
            .send(ExecutePlayerBehavior {
                session_id,
                request_id: "independent-profile-publish".to_string(),
                behavior: command!(StartNewGame {
                    game_mode: GameMode::Tutorial,
                    replace_active_run: true,
                }),
                battle_authority: None,
                battle_side_message: BattleSideMessageKind::BattleUpdate,
            })
            .await
            .unwrap()
            .unwrap_err();

        assert_eq!(error.code, "application_persistence_failed");
        assert_eq!(
            profile_persistence.lock().unwrap().profile().unwrap(),
            Default::default()
        );
        assert!(!profile_directory
            .path()
            .join("player_profile.json")
            .exists());
        assert!(!profile_directory
            .path()
            .join("application_commit.pending.json")
            .exists());
        let reattached = actor
            .send(AttachSession {
                session_id: Uuid::new_v4(),
                socket: probe.clone().recipient(),
                control: probe.recipient(),
            })
            .await
            .unwrap()
            .unwrap();
        assert_eq!(reattached.state_snapshot["game_state"], "not_started");
    }

    #[actix_web::test]
    async fn accepted_application_profile_write_failure_commits_core_and_requires_recovery() {
        let active_directory = tempfile::tempdir().unwrap();
        let active_store = Arc::new(Mutex::new(
            ActiveRunStore::open(active_directory.path()).unwrap(),
        ));
        active_store
            .lock()
            .unwrap()
            .publish(&ActiveRunCheckpointExport {
                core_payload_json: br#"{"version":12,"fixture":"existing-active-run"}"#.to_vec(),
                loads_used: 0,
                run_peak_certified_power: 100,
            })
            .unwrap();

        let profile_directory = tempfile::tempdir().unwrap();
        let profile_persistence = Arc::new(Mutex::new(PlayerProfilePersistence::bootstrap(
            PlayerProfileStore::open(profile_directory.path()).unwrap(),
        )));
        std::fs::create_dir(profile_directory.path().join("player_profile.tmp")).unwrap();
        let mut core = CoreApplication::new(empty_game_data(), 7);
        core.merge_compendium_fixture(
            game_core::game::encounter_compendium::EncounterCompendiumProfileState {
                registered_ids: std::collections::BTreeSet::from([
                    "dirty_but_committed_in_memory".to_string()
                ]),
                ..Default::default()
            },
        )
        .unwrap();

        let load_balance = LoadBalanceActor::new().start();
        let actor = PlayerGameActor::new(Uuid::new_v4(), core, load_balance)
            .with_active_run_store(active_store.clone())
            .with_player_profile_persistence(profile_persistence.clone())
            .start();
        let (probe, _, _) = spawn_probe();
        let session_id = Uuid::new_v4();
        actor
            .send(AttachSession {
                session_id,
                socket: probe.clone().recipient(),
                control: probe.recipient(),
            })
            .await
            .unwrap()
            .unwrap();

        let result = actor
            .send(ExecutePlayerBehavior {
                session_id,
                request_id: "profile-failure-independent".to_string(),
                behavior: command!(StartNewGame {
                    game_mode: GameMode::Tutorial,
                    replace_active_run: true,
                }),
                battle_authority: None,
                battle_side_message: BattleSideMessageKind::BattleUpdate,
            })
            .await
            .unwrap()
            .expect("profile failure must not fail the gameplay command");

        assert_eq!(
            result.state_snapshot.as_ref().unwrap()["game_state"],
            "pre_run_briefing"
        );
        assert!(matches!(
            active_store.lock().unwrap().load().unwrap(),
            ActiveRunLoad::Loaded { .. }
        ));
        assert!(result.side_messages.iter().any(|message| matches!(message,
            PlayerGameServerMessage::Notification { notification_type, .. }
                if notification_type == "application_recovery_pending")));
        assert!(matches!(
            profile_persistence.lock().unwrap().state(),
            PlayerProfilePersistenceState::ApplicationRecoveryPending { .. }
        ));
        assert!(profile_persistence.lock().unwrap().profile().is_err());
        assert!(profile_directory
            .path()
            .join("application_commit.pending.json")
            .is_file());
        std::fs::remove_dir(profile_directory.path().join("player_profile.tmp")).unwrap();
        actor
            .send(ExecutePlayerBehavior {
                session_id,
                request_id: "recover-accepted".into(),
                behavior: command!(RetryPlayerProfile),
                battle_authority: None,
                battle_side_message: BattleSideMessageKind::BattleUpdate,
            })
            .await
            .unwrap()
            .unwrap();
        assert_eq!(
            active_store.lock().unwrap().load().unwrap(),
            ActiveRunLoad::Missing
        );
        let persistence = profile_persistence.lock().unwrap();
        assert!(matches!(
            persistence.state(),
            PlayerProfilePersistenceState::Clean
        ));
        assert!(persistence
            .profile()
            .unwrap()
            .encounter_compendium
            .registered_ids
            .contains("dirty_but_committed_in_memory"));
    }

    #[actix_web::test]
    async fn backup_recovery_notification_is_emitted_once_across_reconnects() {
        let directory = tempfile::tempdir().unwrap();
        let store = PlayerProfileStore::open(directory.path()).unwrap();
        let first = game_core::game::encounter_compendium::EncounterCompendiumProfileState {
            registered_ids: std::collections::BTreeSet::from(["first_generation".to_string()]),
            ..Default::default()
        };
        let second = game_core::game::encounter_compendium::EncounterCompendiumProfileState {
            registered_ids: std::collections::BTreeSet::from([
                "first_generation".to_string(),
                "second_generation".to_string(),
            ]),
            ..Default::default()
        };
        store.publish(&first).unwrap();
        store.publish(&second).unwrap();
        std::fs::write(directory.path().join("player_profile.json"), b"corrupt").unwrap();
        let persistence = Arc::new(Mutex::new(PlayerProfilePersistence::bootstrap(store)));

        let load_balance = LoadBalanceActor::new().start();
        let actor = PlayerGameActor::new(
            Uuid::new_v4(),
            CoreApplication::new(empty_game_data(), 7),
            load_balance,
        )
        .with_player_profile_persistence(persistence)
        .start();
        let (probe, _, _) = spawn_probe();

        let first_attach = actor
            .send(AttachSession {
                session_id: Uuid::new_v4(),
                socket: probe.clone().recipient(),
                control: probe.clone().recipient(),
            })
            .await
            .unwrap()
            .unwrap();
        let second_attach = actor
            .send(AttachSession {
                session_id: Uuid::new_v4(),
                socket: probe.clone().recipient(),
                control: probe.recipient(),
            })
            .await
            .unwrap()
            .unwrap();

        assert_eq!(
            first_attach
                .side_messages
                .iter()
                .filter(|message| matches!(
                    message,
                    PlayerGameServerMessage::Notification { notification_type, .. }
                        if notification_type == "player_profile_recovered_from_backup"
                ))
                .count(),
            1
        );
        assert!(second_attach.side_messages.iter().all(|message| !matches!(
            message,
            PlayerGameServerMessage::Notification { notification_type, .. }
                if notification_type == "player_profile_recovered_from_backup"
        )));
    }

    #[actix_web::test]
    async fn local_active_run_is_projected_and_requires_explicit_new_run_replacement() {
        let directory = tempfile::tempdir().unwrap();
        let store = Arc::new(Mutex::new(ActiveRunStore::open(directory.path()).unwrap()));
        store
            .lock()
            .unwrap()
            .publish(&ActiveRunCheckpointExport {
                core_payload_json: br#"{"version":12,"fixture":"opaque-core-payload"}"#.to_vec(),
                loads_used: 2,
                run_peak_certified_power: 100,
            })
            .unwrap();
        let load_balance = LoadBalanceActor::new().start();
        let actor = PlayerGameActor::new(
            Uuid::new_v4(),
            CoreApplication::new(empty_game_data(), 7),
            load_balance,
        )
        .with_player_profile_persistence(Arc::new(Mutex::new(PlayerProfilePersistence::bootstrap(
            PlayerProfileStore::open(directory.path()).unwrap(),
        ))))
        .with_active_run_store(store.clone())
        .start();
        let (probe, _, _) = spawn_probe();
        let session_id = Uuid::new_v4();

        let attach = actor
            .send(AttachSession {
                session_id,
                socket: probe.clone().recipient(),
                control: probe.recipient(),
            })
            .await
            .unwrap()
            .unwrap();
        assert_eq!(
            attach.state_snapshot["active_run_continue"]["available"],
            true
        );
        assert!(attach.state_snapshot["action_capabilities"]
            .as_array()
            .unwrap()
            .iter()
            .any(|capability| capability["action"] == "ContinueRun"));

        let error = actor
            .send(ExecutePlayerBehavior {
                session_id,
                request_id: "replace-check".to_string(),
                behavior: command!(StartNewGame {
                    game_mode: GameMode::Tutorial,
                    replace_active_run: false,
                }),
                battle_authority: None,
                battle_side_message: BattleSideMessageKind::BattleUpdate,
            })
            .await
            .unwrap()
            .unwrap_err();
        assert_eq!(error.code, "active_run_replacement_confirmation_required");

        actor
            .send(ExecutePlayerBehavior {
                session_id,
                request_id: "replace-confirmed".to_string(),
                behavior: command!(StartNewGame {
                    game_mode: GameMode::Tutorial,
                    replace_active_run: true,
                }),
                battle_authority: None,
                battle_side_message: BattleSideMessageKind::BattleUpdate,
            })
            .await
            .unwrap()
            .unwrap();
        assert_eq!(
            store.lock().unwrap().load().unwrap(),
            ActiveRunLoad::Missing
        );
    }

    #[actix_web::test]
    async fn corrupt_local_active_run_requires_explicit_reset() {
        let directory = tempfile::tempdir().unwrap();
        let store = Arc::new(Mutex::new(ActiveRunStore::open(directory.path()).unwrap()));
        store
            .lock()
            .unwrap()
            .publish(&ActiveRunCheckpointExport {
                core_payload_json: br#"{"version":12,"fixture":"opaque-core-payload"}"#.to_vec(),
                loads_used: 0,
                run_peak_certified_power: 100,
            })
            .unwrap();
        std::fs::write(directory.path().join("active_run.json"), b"corrupt").unwrap();
        std::fs::write(
            directory.path().join("active_run.backup.json"),
            b"corrupt backup",
        )
        .unwrap();
        let load_balance = LoadBalanceActor::new().start();
        let actor = PlayerGameActor::new(
            Uuid::new_v4(),
            CoreApplication::new(empty_game_data(), 7),
            load_balance,
        )
        .with_active_run_store(store)
        .start();
        let (probe, _, _) = spawn_probe();
        let session_id = Uuid::new_v4();

        let attach = actor
            .send(AttachSession {
                session_id,
                socket: probe.clone().recipient(),
                control: probe.recipient(),
            })
            .await
            .unwrap()
            .unwrap();
        assert_eq!(
            attach.state_snapshot["active_run_continue"]["reset_required"],
            true
        );

        let reset = actor
            .send(ExecutePlayerBehavior {
                session_id,
                request_id: "reset-corrupt".to_string(),
                behavior: command!(ResetActiveRun),
                battle_authority: None,
                battle_side_message: BattleSideMessageKind::BattleUpdate,
            })
            .await
            .unwrap()
            .unwrap();
        assert!(matches!(
            reset.response,
            PlayerGameServerMessage::CommandResult { ref result_type, .. }
                if result_type == "ActiveRunReset"
        ));
        assert_eq!(
            reset.state_snapshot.unwrap()["active_run_continue"]["available"],
            false
        );
    }

    #[actix_web::test]
    async fn attach_replaces_previous_session_immediately() {
        let load_balance = LoadBalanceActor::new().start();
        let actor = PlayerGameActor::new(
            Uuid::new_v4(),
            CoreApplication::new(empty_game_data(), 9),
            load_balance,
        )
        .start();

        let (probe_one, disconnects_one, _) = spawn_probe();
        let (probe_two, _, _) = spawn_probe();

        actor
            .send(AttachSession {
                session_id: Uuid::new_v4(),
                socket: probe_one.clone().recipient(),
                control: probe_one.recipient(),
            })
            .await
            .expect("first attach request should complete")
            .expect("first attach should succeed");

        actor
            .send(AttachSession {
                session_id: Uuid::new_v4(),
                socket: probe_two.clone().recipient(),
                control: probe_two.recipient(),
            })
            .await
            .expect("second attach request should complete")
            .expect("second attach should succeed");

        time::sleep(Duration::from_millis(25)).await;

        let disconnects = disconnects_one
            .lock()
            .expect("disconnects mutex should be lockable");
        assert_eq!(disconnects.len(), 1);
        assert_eq!(disconnects[0].0, "session_replaced");
    }

    #[actix_web::test]
    async fn detached_actor_stops_after_ttl_and_deregisters() {
        let load_balance = LoadBalanceActor::new().start();
        let player_id = Uuid::new_v4();
        let actor = PlayerGameActor::new(
            player_id,
            CoreApplication::new(empty_game_data(), 13),
            load_balance.clone(),
        )
        .with_disconnect_ttl(Duration::from_millis(20))
        .start();

        load_balance
            .send(Register {
                player_id,
                addr: actor.clone(),
            })
            .await
            .expect("register should succeed");

        let (probe, _, _) = spawn_probe();
        let session_id = Uuid::new_v4();

        actor
            .send(AttachSession {
                session_id,
                socket: probe.clone().recipient(),
                control: probe.recipient(),
            })
            .await
            .expect("attach request should complete")
            .expect("attach should succeed");

        actor.do_send(DetachSession { session_id });

        time::sleep(Duration::from_millis(80)).await;

        assert_eq!(
            load_balance
                .send(crate::game::load_balance_actor::messages::GetPlayerCount)
                .await
                .expect("player count query should succeed"),
            0
        );
    }

    #[actix_web::test]
    async fn detached_live_battle_has_no_ttl_and_reconnects_paused() {
        let load_balance = LoadBalanceActor::new().start();
        let player_id = Uuid::from_u128(0xD3F3_51DC);
        let actor = PlayerGameActor::new(
            player_id,
            game_core_waiting_on_live_defense_node(player_id),
            load_balance.clone(),
        )
        .with_disconnect_ttl(Duration::from_millis(20))
        .with_live_battle_tick_interval(Duration::from_secs(1))
        .start();
        load_balance
            .send(Register {
                player_id,
                addr: actor.clone(),
            })
            .await
            .expect("register should succeed");

        let (first_probe, _, _) = spawn_probe();
        let first_session_id = Uuid::new_v4();
        actor
            .send(AttachSession {
                session_id: first_session_id,
                socket: first_probe.clone().recipient(),
                control: first_probe.recipient(),
            })
            .await
            .unwrap()
            .unwrap();
        actor
            .send(ExecutePlayerBehavior {
                session_id: first_session_id,
                request_id: "enter-before-detach".to_owned(),
                behavior: command!(ConfirmEnterNode),
                battle_authority: None,
                battle_side_message: BattleSideMessageKind::BattleUpdate,
            })
            .await
            .unwrap()
            .unwrap();

        actor.do_send(DetachSession {
            session_id: first_session_id,
        });
        time::sleep(Duration::from_millis(80)).await;
        assert_eq!(
            load_balance
                .send(crate::game::load_balance_actor::messages::GetPlayerCount)
                .await
                .unwrap(),
            1,
            "an active battle must outlive the ordinary disconnect TTL"
        );

        let (second_probe, _, _) = spawn_probe();
        let second_session_id = Uuid::new_v4();
        let reconnect = actor
            .send(AttachSession {
                session_id: second_session_id,
                socket: second_probe.clone().recipient(),
                control: second_probe.recipient(),
            })
            .await
            .unwrap()
            .unwrap();
        let paused_update = reconnect
            .side_messages
            .iter()
            .find_map(|message| match message {
                PlayerGameServerMessage::BattleUpdate(update) => Some(update.clone()),
                _ => None,
            })
            .expect("reconnect should include the current battle update");
        assert!(paused_update.checkpoint.playback.paused);
        assert_eq!(paused_update.checkpoint.battle_time_ms, 0);

        let resumed = actor
            .send(ExecutePlayerBehavior {
                session_id: second_session_id,
                request_id: "resume-after-reconnect".to_owned(),
                behavior: command!(ResumeBattle),
                battle_authority: Some(game_core::game::behavior::BattleStateAuthorityDto {
                    battle_uuid: paused_update.battle_uuid,
                    battle_state_revision: paused_update.battle_state_revision,
                }),
                battle_side_message: BattleSideMessageKind::BattleUpdate,
            })
            .await
            .unwrap()
            .unwrap();
        let resumed_update = resumed
            .side_messages
            .iter()
            .find_map(|message| match message {
                PlayerGameServerMessage::BattleUpdate(update) => Some(update),
                _ => None,
            })
            .expect("explicit resume should include a battle update");
        assert!(!resumed_update.checkpoint.playback.paused);
    }

    #[actix_web::test]
    async fn live_battle_tick_pushes_update_without_state_snapshot_after_confirm_enter() {
        let load_balance = LoadBalanceActor::new().start();
        let player_id = Uuid::from_u128(0xD3F3_51DE);
        let actor = PlayerGameActor::new(
            player_id,
            game_core_waiting_on_live_defense_node(player_id),
            load_balance,
        )
        .with_live_battle_tick_interval(Duration::from_millis(5))
        .start();

        let (probe, _, messages) = spawn_probe();
        let session_id = Uuid::new_v4();

        actor
            .send(AttachSession {
                session_id,
                socket: probe.clone().recipient(),
                control: probe.recipient(),
            })
            .await
            .expect("attach request should complete")
            .expect("attach should succeed");

        let result = actor
            .send(ExecutePlayerBehavior {
                session_id,
                request_id: "enter-live-defense".to_string(),
                behavior: command!(ConfirmEnterNode),
                battle_authority: None,
                battle_side_message: BattleSideMessageKind::BattleUpdate,
            })
            .await
            .expect("confirm enter request should complete")
            .expect("confirm enter should start live battle");

        match result.response {
            PlayerGameServerMessage::CommandResult {
                result_type,
                payload,
                ..
            } => {
                assert_eq!(result_type, "CommandAccepted");
                assert_eq!(payload["command_id"], "enter-live-defense");
                assert_eq!(payload["accepted_at_battle_time_ms"], 0);
            }
            other => panic!("expected CommandAccepted command result, got {other:?}"),
        }
        assert!(matches!(
            result.side_messages.as_slice(),
            [
                PlayerGameServerMessage::BattleSetupSnapshot(setup),
                PlayerGameServerMessage::BattleUpdate(update)
            ] if !update.battle_uuid.is_nil()
                && setup.battle_uuid == update.battle_uuid
                && setup.setup_version == 2
                && !setup.routes.is_empty()
                && !setup.deployment_zones.is_empty()
                && update.events_delta.after_seq == 0
                && update.battle_state_revision == 1
                && update.action_capabilities.iter().any(|capability| {
                    capability.action == game_core::game::behavior::ActionKind::PauseBattle
                        && capability.enabled
                })
                && update.events_delta.to_seq == update.checkpoint.at_seq
                && update.checkpoint.battle_time_ms == 0
                && update.server_battle_time_ms == 0
                && update.visual_novel.is_none()
        ));
        assert!(
            result.state_snapshot.is_none(),
            "battle start is represented by battle_setup_snapshot plus battle_update, not full state_snapshot"
        );
        let (started_battle_uuid, started_revision, started_to_seq) = result
            .side_messages
            .iter()
            .find_map(|message| match message {
                PlayerGameServerMessage::BattleUpdate(update) => Some((
                    update.battle_uuid,
                    update.battle_state_revision,
                    update.events_delta.to_seq,
                )),
                _ => None,
            })
            .expect("battle start should expose its delivered cursor");
        time::sleep(Duration::from_millis(20)).await;
        {
            let mut messages = messages.lock().expect("messages mutex should be lockable");
            assert!(matches!(
                messages.as_slice(),
                [
                    PlayerGameServerMessage::CommandResult { request_id, payload, .. },
                    PlayerGameServerMessage::BattleSetupSnapshot(setup),
                    PlayerGameServerMessage::BattleUpdate(update)
                ] if request_id == "enter-live-defense"
                    && payload["battle_state_revision"] == 1
                    && setup.battle_uuid == update.battle_uuid
                    && update.battle_state_revision == 1
                    && update.events_delta.to_seq == started_to_seq
            ));
            messages.clear();
        }
        actor.do_send(EnsureLiveBattleTick);

        let resync = actor
            .send(ExecutePlayerBehavior {
                session_id,
                request_id: "resync-live-defense".to_string(),
                behavior: command!(RequestBattleState { since_seq: Some(0) }),
                battle_authority: None,
                battle_side_message: BattleSideMessageKind::BattleResync,
            })
            .await
            .expect("resync request should complete")
            .expect("resync should succeed");
        match resync.response {
            PlayerGameServerMessage::CommandResult {
                result_type,
                payload,
                ..
            } => {
                assert_eq!(result_type, "CommandAccepted");
                assert_eq!(payload["command_id"], "resync-live-defense");
                assert_eq!(payload["battle_state_revision"], started_revision);
            }
            other => panic!("expected CommandAccepted resync result, got {other:?}"),
        }
        assert!(resync.side_messages.iter().any(|message| matches!(
            message,
            PlayerGameServerMessage::BattleResync {
                update,
                ..
            } if update.events_delta.after_seq == 0
                && update.events_delta.to_seq == update.checkpoint.at_seq
                && update.battle_uuid == started_battle_uuid
                && update.battle_state_revision == started_revision
                && !update.events_delta.events.is_empty()
        )));
        assert!(
            resync.state_snapshot.is_none(),
            "catch-up battle_resync is a battle side message and must not be followed by full state_snapshot"
        );

        time::sleep(Duration::from_millis(60)).await;

        let messages = messages.lock().expect("messages mutex should be lockable");
        let battle_update = messages.iter().find_map(|message| match message {
            PlayerGameServerMessage::BattleUpdate(update) => Some((
                &update.battle_uuid,
                &update.events_delta,
                &update.checkpoint,
            )),
            _ => None,
        });
        let (battle_uuid, events_delta, checkpoint) =
            battle_update.expect("live battle tick should push battle_update");
        assert!(!battle_uuid.is_nil());
        assert_eq!(*battle_uuid, started_battle_uuid);
        assert_eq!(events_delta.after_seq, started_to_seq);
        assert_eq!(events_delta.to_seq, checkpoint.at_seq);

        assert!(!messages
            .iter()
            .any(|message| matches!(message, PlayerGameServerMessage::StateSnapshot { .. })));
    }

    #[actix_web::test]
    async fn second_mutation_using_same_revision_is_stale_after_first_commit() {
        let load_balance = LoadBalanceActor::new().start();
        let player_id = Uuid::from_u128(0xD3F3_57A1E);
        let actor = PlayerGameActor::new(
            player_id,
            game_core_waiting_on_live_defense_node(player_id),
            load_balance,
        )
        .start();
        let (probe, _, messages) = spawn_probe();
        let session_id = Uuid::new_v4();
        actor
            .send(AttachSession {
                session_id,
                socket: probe.clone().recipient(),
                control: probe.recipient(),
            })
            .await
            .expect("attach request should complete")
            .expect("attach should succeed");
        let entered = actor
            .send(ExecutePlayerBehavior {
                session_id,
                request_id: "enter-before-stale".to_string(),
                behavior: command!(ConfirmEnterNode),
                battle_authority: None,
                battle_side_message: BattleSideMessageKind::BattleUpdate,
            })
            .await
            .expect("enter request should complete")
            .expect("enter should succeed");
        let (battle_uuid, revision) = entered
            .side_messages
            .iter()
            .find_map(|message| match message {
                PlayerGameServerMessage::BattleUpdate(update) => {
                    Some((update.battle_uuid, update.battle_state_revision))
                }
                _ => None,
            })
            .expect("battle update authority");
        let committed = actor
            .send(ExecutePlayerBehavior {
                session_id,
                request_id: "first-pause".to_string(),
                behavior: command!(PauseBattle),
                battle_authority: Some(game_core::game::behavior::BattleStateAuthorityDto {
                    battle_uuid,
                    battle_state_revision: revision,
                }),
                battle_side_message: BattleSideMessageKind::BattleUpdate,
            })
            .await
            .expect("first mutation should complete")
            .expect("first mutation should commit");
        let committed_revision = committed
            .side_messages
            .iter()
            .find_map(|message| match message {
                PlayerGameServerMessage::BattleUpdate(update)
                    if update.checkpoint.playback.paused =>
                {
                    Some(update.battle_state_revision)
                }
                _ => None,
            })
            .expect("first mutation should expose paused authority");
        assert_eq!(committed_revision, revision + 1);
        time::sleep(Duration::from_millis(20)).await;
        messages.lock().expect("messages mutex").clear();

        let rejected = actor
            .send(ExecutePlayerBehavior {
                session_id,
                request_id: "stale-resume".to_string(),
                behavior: command!(ResumeBattle),
                battle_authority: Some(game_core::game::behavior::BattleStateAuthorityDto {
                    battle_uuid,
                    battle_state_revision: revision,
                }),
                battle_side_message: BattleSideMessageKind::BattleUpdate,
            })
            .await
            .expect("stale request should complete as an ordered rejection batch")
            .expect("stale rejection should remain transportable");

        assert!(matches!(
            rejected.response,
            PlayerGameServerMessage::Error { ref request_id, ref code, .. }
                if request_id.as_deref() == Some("stale-resume")
                    && code == "stale_battle_state_authority"
        ));
        assert!(matches!(
            rejected.side_messages.as_slice(),
            [PlayerGameServerMessage::BattleResync {
                battle_uuid: actual_uuid,
                update,
            }] if *actual_uuid == battle_uuid
                && update.battle_state_revision == committed_revision
                && update.checkpoint.playback.paused
        ));
        time::sleep(Duration::from_millis(20)).await;
        let delivered = messages.lock().expect("messages mutex");
        assert!(matches!(
            delivered.as_slice(),
            [
                PlayerGameServerMessage::Error { request_id, code, .. },
                PlayerGameServerMessage::BattleResync { battle_uuid: actual_uuid, update }
            ] if request_id.as_deref() == Some("stale-resume")
                && code == "stale_battle_state_authority"
                && *actual_uuid == battle_uuid
                && update.battle_state_revision == committed_revision
                && update.checkpoint.playback.paused
        ));
    }

    #[actix_web::test]
    async fn missing_battle_authority_emits_error_then_current_resync_without_mutation() {
        let load_balance = LoadBalanceActor::new().start();
        let player_id = Uuid::from_u128(0xD3F3_57A2E);
        let actor = PlayerGameActor::new(
            player_id,
            game_core_waiting_on_live_defense_node(player_id),
            load_balance,
        )
        .start();
        let (probe, _, messages) = spawn_probe();
        let session_id = Uuid::new_v4();
        actor
            .send(AttachSession {
                session_id,
                socket: probe.clone().recipient(),
                control: probe.recipient(),
            })
            .await
            .expect("attach request should complete")
            .expect("attach should succeed");
        let entered = actor
            .send(ExecutePlayerBehavior {
                session_id,
                request_id: "enter-before-missing".to_string(),
                behavior: command!(ConfirmEnterNode),
                battle_authority: None,
                battle_side_message: BattleSideMessageKind::BattleUpdate,
            })
            .await
            .expect("enter request should complete")
            .expect("enter should succeed");
        let (battle_uuid, revision) = entered
            .side_messages
            .iter()
            .find_map(|message| match message {
                PlayerGameServerMessage::BattleUpdate(update) => {
                    Some((update.battle_uuid, update.battle_state_revision))
                }
                _ => None,
            })
            .expect("battle update authority");
        time::sleep(Duration::from_millis(20)).await;
        messages.lock().expect("messages mutex").clear();

        let rejected = actor
            .send(ExecutePlayerBehavior {
                session_id,
                request_id: "missing-pause".to_string(),
                behavior: command!(PauseBattle),
                battle_authority: None,
                battle_side_message: BattleSideMessageKind::BattleUpdate,
            })
            .await
            .expect("missing-authority request should complete as an ordered rejection batch")
            .expect("missing-authority rejection should remain transportable");

        assert!(matches!(
            rejected.response,
            PlayerGameServerMessage::Error { ref request_id, ref code, .. }
                if request_id.as_deref() == Some("missing-pause")
                    && code == "missing_battle_state_authority"
        ));
        assert!(matches!(
            rejected.side_messages.as_slice(),
            [PlayerGameServerMessage::BattleResync {
                battle_uuid: actual_uuid,
                update,
            }] if *actual_uuid == battle_uuid
                && update.battle_state_revision == revision
                && !update.checkpoint.playback.paused
        ));
        time::sleep(Duration::from_millis(20)).await;
        let delivered = messages.lock().expect("messages mutex");
        assert!(matches!(
            delivered.as_slice(),
            [
                PlayerGameServerMessage::Error { request_id, code, .. },
                PlayerGameServerMessage::BattleResync { battle_uuid: actual_uuid, update }
            ] if request_id.as_deref() == Some("missing-pause")
                && code == "missing_battle_state_authority"
                && *actual_uuid == battle_uuid
                && update.battle_state_revision == revision
        ));
    }

    #[actix_web::test]
    async fn finished_live_battle_tick_pushes_combat_result_snapshot_after_final_update() {
        let load_balance = LoadBalanceActor::new().start();
        let player_id = Uuid::from_u128(0xD3F3_51E2);
        let actor = PlayerGameActor::new(
            player_id,
            game_core_waiting_on_live_survival_defense_node(player_id),
            load_balance,
        )
        .with_live_battle_tick_interval(Duration::from_millis(5))
        .start();

        let (probe, _, messages) = spawn_probe();
        let session_id = Uuid::new_v4();

        actor
            .send(AttachSession {
                session_id,
                socket: probe.clone().recipient(),
                control: probe.recipient(),
            })
            .await
            .expect("attach request should complete")
            .expect("attach should succeed");

        actor
            .send(ExecutePlayerBehavior {
                session_id,
                request_id: "enter-live-survival-defense".to_string(),
                behavior: command!(ConfirmEnterNode),
                battle_authority: None,
                battle_side_message: BattleSideMessageKind::BattleUpdate,
            })
            .await
            .expect("confirm enter request should complete")
            .expect("confirm enter should start live battle");

        actor.do_send(EnsureLiveBattleTick);

        for _ in 0..20 {
            time::sleep(Duration::from_millis(20)).await;
            let messages = messages.lock().expect("messages mutex should be lockable");
            if messages
                .iter()
                .any(|message| matches!(message, PlayerGameServerMessage::StateSnapshot { .. }))
            {
                break;
            }
        }

        let messages = messages.lock().expect("messages mutex should be lockable");
        let final_snapshot_index = messages
            .iter()
            .position(|message| {
                matches!(
                    message,
                    PlayerGameServerMessage::StateSnapshot { state }
                        if state["game_state_context"]["type"] == "combat_result"
                            && state["selected_event"]["type"] == "combat_battle"
                            && state["selected_event"]["result_stats"].is_object()
                            && state["selected_event"].get("compressed_event_log").is_none()
                )
            })
            .expect("finished live tick should push combat_result state_snapshot");
        let final_update_index = messages
            .iter()
            .position(|message| matches!(message, PlayerGameServerMessage::BattleUpdate(_)))
            .expect("finished live tick should push final battle_update");

        assert!(
            final_update_index < final_snapshot_index,
            "final battle_update must arrive before combat_result state_snapshot"
        );
    }

    #[actix_web::test]
    async fn live_battle_resync_rejects_future_since_seq() {
        let load_balance = LoadBalanceActor::new().start();
        let player_id = Uuid::from_u128(0xD3F3_51E0);
        let actor = PlayerGameActor::new(
            player_id,
            game_core_waiting_on_live_defense_node(player_id),
            load_balance,
        )
        .start();

        let (probe, _, _) = spawn_probe();
        let session_id = Uuid::new_v4();

        actor
            .send(AttachSession {
                session_id,
                socket: probe.clone().recipient(),
                control: probe.recipient(),
            })
            .await
            .expect("attach request should complete")
            .expect("attach should succeed");

        actor
            .send(ExecutePlayerBehavior {
                session_id,
                request_id: "enter-live-defense".to_string(),
                behavior: command!(ConfirmEnterNode),
                battle_authority: None,
                battle_side_message: BattleSideMessageKind::BattleUpdate,
            })
            .await
            .expect("confirm enter request should complete")
            .expect("confirm enter should start live battle");

        let err = actor
            .send(ExecutePlayerBehavior {
                session_id,
                request_id: "future-resync".to_string(),
                behavior: command!(RequestBattleState {
                    since_seq: Some(u64::MAX),
                }),
                battle_authority: None,
                battle_side_message: BattleSideMessageKind::BattleResync,
            })
            .await
            .expect("future resync request should complete")
            .expect_err("future since_seq should be rejected");

        assert_eq!(err.code, "invalid_battle_resync_seq");
    }

    #[actix_web::test]
    async fn attaching_replacement_session_auto_pauses_and_returns_recovery_batch() {
        let load_balance = LoadBalanceActor::new().start();
        let player_id = Uuid::from_u128(0xD3F3_51E1);
        let actor = PlayerGameActor::new(
            player_id,
            game_core_waiting_on_live_defense_node(player_id),
            load_balance,
        )
        .with_live_battle_tick_interval(Duration::from_millis(50))
        .start();

        let (probe_one, disconnects_one, _) = spawn_probe();
        let session_one = Uuid::new_v4();

        actor
            .send(AttachSession {
                session_id: session_one,
                socket: probe_one.clone().recipient(),
                control: probe_one.recipient(),
            })
            .await
            .expect("first attach request should complete")
            .expect("first attach should succeed");

        actor
            .send(ExecutePlayerBehavior {
                session_id: session_one,
                request_id: "enter-live-defense".to_string(),
                behavior: command!(ConfirmEnterNode),
                battle_authority: None,
                battle_side_message: BattleSideMessageKind::BattleUpdate,
            })
            .await
            .expect("confirm enter request should complete")
            .expect("confirm enter should start live battle");

        let (probe_two, _, _) = spawn_probe();
        let session_two = Uuid::new_v4();
        let attach = actor
            .send(AttachSession {
                session_id: session_two,
                socket: probe_two.clone().recipient(),
                control: probe_two.recipient(),
            })
            .await
            .expect("second attach request should complete")
            .expect("second attach should succeed");

        assert_eq!(
            attach.state_snapshot["game_state_context"]["type"],
            "in_battle"
        );
        assert_eq!(
            attach.state_snapshot["game_state_context"]["battle_state_revision"],
            2
        );
        assert!(matches!(
            attach.side_messages.as_slice(),
            [
                PlayerGameServerMessage::BattleSetupSnapshot(setup),
                PlayerGameServerMessage::BattleUpdate(update)
            ] if setup.battle_uuid == update.battle_uuid
                && update.battle_state_revision == 2
                && update.checkpoint.playback.paused
                && update.events_delta.after_seq == 0
                && update.events_delta.to_seq > 0
        ));

        let disconnects = disconnects_one
            .lock()
            .expect("disconnects mutex should be lockable");
        assert!(disconnects
            .iter()
            .any(|(code, _)| code == "session_replaced"));
    }

    #[actix_web::test]
    async fn paused_live_battle_tick_does_not_push_delta_until_resumed() {
        let load_balance = LoadBalanceActor::new().start();
        let player_id = Uuid::from_u128(0xD3F3_51DF);
        let actor = PlayerGameActor::new(
            player_id,
            game_core_waiting_on_live_defense_node(player_id),
            load_balance,
        )
        .with_live_battle_tick_interval(Duration::from_millis(50))
        .start();

        let (probe, _, messages) = spawn_probe();
        let session_id = Uuid::new_v4();

        actor
            .send(AttachSession {
                session_id,
                socket: probe.clone().recipient(),
                control: probe.recipient(),
            })
            .await
            .expect("attach request should complete")
            .expect("attach should succeed");

        let entered = actor
            .send(ExecutePlayerBehavior {
                session_id,
                request_id: "enter-live-defense".to_string(),
                behavior: command!(ConfirmEnterNode),
                battle_authority: None,
                battle_side_message: BattleSideMessageKind::BattleUpdate,
            })
            .await
            .expect("confirm enter request should complete")
            .expect("confirm enter should start live battle");
        let (battle_uuid, entered_revision) = entered
            .side_messages
            .iter()
            .find_map(|message| match message {
                PlayerGameServerMessage::BattleUpdate(update) => {
                    Some((update.battle_uuid, update.battle_state_revision))
                }
                _ => None,
            })
            .expect("enter should expose battle authority");
        actor.do_send(EnsureLiveBattleTick);

        let paused = actor
            .send(ExecutePlayerBehavior {
                session_id,
                request_id: "pause-live-defense".to_string(),
                behavior: command!(PauseBattle),
                battle_authority: Some(game_core::game::behavior::BattleStateAuthorityDto {
                    battle_uuid,
                    battle_state_revision: entered_revision,
                }),
                battle_side_message: BattleSideMessageKind::BattleUpdate,
            })
            .await
            .expect("pause request should complete")
            .expect("pause should succeed");
        match paused.response {
            PlayerGameServerMessage::CommandResult {
                result_type,
                payload,
                ..
            } => {
                assert_eq!(result_type, "CommandAccepted");
                assert_eq!(payload["command_id"], "pause-live-defense");
            }
            other => panic!("expected CommandAccepted command result, got {other:?}"),
        }
        assert!(paused.side_messages.iter().any(|message| matches!(
            message,
            PlayerGameServerMessage::BattleUpdate(update)
                if update.checkpoint.playback.paused
        )));
        assert!(
            paused.state_snapshot.is_none(),
            "battle playback commands must not be followed by full state_snapshot"
        );
        assert!(!paused
            .side_messages
            .iter()
            .any(|message| matches!(message, PlayerGameServerMessage::BattleSetupSnapshot(_))));

        let (paused_revision, paused_to_seq) = paused
            .side_messages
            .iter()
            .find_map(|message| match message {
                PlayerGameServerMessage::BattleUpdate(update)
                    if update.checkpoint.playback.paused =>
                {
                    Some((update.battle_state_revision, update.events_delta.to_seq))
                }
                _ => None,
            })
            .expect("pause should expose its revision and cursor");
        time::sleep(Duration::from_millis(20)).await;
        {
            let mut messages = messages.lock().expect("messages mutex should be lockable");
            let pause_ack_index = messages
                .iter()
                .position(|message| {
                    matches!(
                        message,
                        PlayerGameServerMessage::CommandResult { request_id, payload, .. }
                            if request_id == "pause-live-defense"
                                && payload["battle_state_revision"] == paused_revision
                    )
                })
                .expect("actor outbox should emit the pause acknowledgement");
            let pause_update_index = messages
                .iter()
                .position(|message| {
                    matches!(
                        message,
                        PlayerGameServerMessage::BattleUpdate(update)
                            if update.battle_state_revision == paused_revision
                                && update.checkpoint.playback.paused
                    )
                })
                .expect("actor outbox should emit the pause update");
            assert!(pause_ack_index < pause_update_index);
            messages.clear();
        }

        time::sleep(Duration::from_millis(120)).await;
        {
            let messages = messages.lock().expect("messages mutex should be lockable");
            assert!(!messages
                .iter()
                .any(|message| matches!(message, PlayerGameServerMessage::BattleUpdate(_))));
        }

        actor
            .send(ExecutePlayerBehavior {
                session_id,
                request_id: "resume-live-defense".to_string(),
                behavior: command!(ResumeBattle),
                battle_authority: Some(game_core::game::behavior::BattleStateAuthorityDto {
                    battle_uuid,
                    battle_state_revision: paused_revision,
                }),
                battle_side_message: BattleSideMessageKind::BattleUpdate,
            })
            .await
            .expect("resume request should complete")
            .expect("resume should succeed");

        time::sleep(Duration::from_millis(120)).await;
        let messages = messages.lock().expect("messages mutex should be lockable");
        assert!(messages.iter().any(|message| matches!(
            message,
            PlayerGameServerMessage::BattleUpdate(update)
                if update.battle_state_revision > paused_revision
                    && update.events_delta.after_seq >= paused_to_seq
                    && update.events_delta.to_seq == update.checkpoint.at_seq
        )));
    }
    #[cfg(feature = "runtime-observation")]
    #[test]
    fn runtime_observation_real_battle_preparation_failure_commit_and_end_are_distinct() {
        use game_core::{
            application::{ApplicationBattleAuthority, ApplicationExecutionContext},
            runtime_observation::{self as obs, Outcome},
        };
        let player = Uuid::from_u128(0x5209);
        let mut app = game_core_waiting_on_live_defense_node(player);
        let mut baseline = game_core_waiting_on_live_defense_node(player);
        let (sender, receiver) = obs::channel();
        let (observer, reader) = sender.session(10);
        app.set_runtime_observer(Some(observer));
        let before = app.snapshot(ApplicationSnapshotKind::Run).unwrap();
        let prepared = app
            .prepare_non_battle_command(
                player,
                command!(ConfirmEnterNode),
                ApplicationExecutionContext::default(),
            )
            .unwrap();
        assert!(prepared.enters_battle());
        assert_eq!(reader.snapshot(false).unwrap().commits, 0);
        drop(prepared);
        assert_eq!(app.snapshot(ApplicationSnapshotKind::Run).unwrap(), before);
        let discarded = receiver.try_recv().unwrap();
        assert_eq!(discarded.outcome, Outcome::Discarded);
        assert!(discarded.work > 0 && discarded.admitted_bytes > 0);
        assert!(discarded.peak.0[1] > 0 && discarded.peak.0[4] > 0);
        let expected = baseline
            .execute_fixture(player, command!(ConfirmEnterNode))
            .unwrap();
        let prepared = app
            .prepare_non_battle_command(
                player,
                command!(ConfirmEnterNode),
                ApplicationExecutionContext::default(),
            )
            .unwrap();
        assert_eq!(prepared.commit(), expected);
        let accepted = receiver.try_recv().unwrap();
        assert_eq!(accepted.outcome, Outcome::Accepted);
        let live = reader.snapshot(false).unwrap();
        assert_eq!(live.commits, 1);
        assert!(live.current.battle.is_some());
        assert!(live.current.usage.0[0] > 0 && live.current.usage.0[4] > 0);
        let failure = app
            .execute_fixture(player, command!(PauseBattle))
            .unwrap_err();
        assert_eq!(
            failure.kind(),
            ApplicationErrorKind::MissingBattleStateAuthority
        );
        let rejected = receiver.try_recv().unwrap();
        assert_eq!(rejected.outcome, Outcome::Rejected);
        assert_eq!(reader.snapshot(false).unwrap().current, live.current);
        let projection = app.active_battle_reconnect(0).unwrap();
        let update = &projection.payload().as_value()["battle_update"];
        let authority = ApplicationBattleAuthority {
            battle_uuid: serde_json::from_value(update["battle_uuid"].clone()).unwrap(),
            battle_state_revision: update["battle_state_revision"].as_u64().unwrap(),
        };
        app.execute(
            player,
            command!(RetreatBattle),
            ApplicationExecutionContext {
                expected_battle_authority: Some(authority),
                ..Default::default()
            },
        )
        .unwrap();
        assert!(reader.snapshot(false).unwrap().current.battle.is_none());
        assert_eq!(reader.snapshot(false).unwrap().commits, 2);
        assert_eq!(receiver.counters().reasons.iter().sum::<u64>(), 1);
        assert_eq!(receiver.counters().commits, 2);
    }
}

/// Lock order is profile -> active everywhere. The exclusive prepared borrow
/// keeps the candidate private until a conditional durable intent is accepted.
fn publish_prepared_application<T>(
    prepared: game_core::application::PreparedApplication<'_, T>,
    active_store: Option<
        &std::sync::Arc<std::sync::Mutex<crate::persistence::active_run_store::ActiveRunStore>>,
    >,
    profile_persistence: Option<
        &std::sync::Arc<
            std::sync::Mutex<crate::persistence::player_profile_store::PlayerProfilePersistence>,
        >,
    >,
    expected_revision: &mut Option<crate::persistence::active_run_store::ActiveRunRevision>,
    checkpoint_before: Option<&game_core::application::ApplicationCheckpoint>,
    force_revision: bool,
) -> Result<(T, bool), PlayerGameActorError> {
    let Some(active_store) = active_store else {
        if let Some(persistence) = profile_persistence {
            let profile = persistence.lock().map_err(|_| PlayerGameActorError::new("player_profile_lock_poisoned", "Profile lock is poisoned"))?;
            let current = profile.profile().map_err(|e| PlayerGameActorError::new("player_profile_recovery_required", e))?;
            if prepared.player_profile().lifecycle != current.lifecycle {
                return Err(PlayerGameActorError::new("paired_active_persistence_required", "Expedition ownership changes require paired active/profile persistence"));
            }
        }
        return Ok((prepared.commit(), false));
    };
    let profile_persistence = profile_persistence.ok_or_else(|| {
        PlayerGameActorError::new(
            "player_profile_persistence_unavailable",
            "Durable application requires player-profile persistence",
        )
    })?;
    let mut profile = profile_persistence.lock().map_err(|_| {
        PlayerGameActorError::new(
            "player_profile_lock_poisoned",
            "Player profile persistence lock is poisoned",
        )
    })?;
    let current_profile = profile
        .profile()
        .map_err(|issue| PlayerGameActorError::new("application_recovery_required", issue))?;
    let mut merged = prepared.player_profile().clone();
    merged.validate_persisted().map_err(|e| PlayerGameActorError::new("invalid_player_profile", e))?;
    merged.merge_knowledge(&current_profile);
    let active = active_store.lock().map_err(|_| PlayerGameActorError::new("active_run_lock_poisoned", "Active store lock is poisoned"))?;
    let expected = expected_revision.unwrap_or(uuid::Uuid::nil());
    let actual = active.snapshot().map_err(|e| PlayerGameActorError::new("active_run_load_failed", e.to_string()))?.revision;
    if expected != actual { return Err(PlayerGameActorError::new("active_run_revision_conflict", format!("expected {expected}, actual {actual}"))); }
    if !force_revision && !prepared.terminal() && prepared.checkpoint() == checkpoint_before && merged == current_profile {
        return Ok((prepared.commit(), false));
    }
    let checkpoint = prepared.checkpoint().map(StoredCheckpoint::from);
    let accepted = profile.accept_application(&active, expected, checkpoint.as_ref(), &merged).map_err(|error| {
        let code = if matches!(error, crate::persistence::player_profile_store::PlayerProfileStoreError::ActiveRun(
            crate::persistence::active_run_store::ActiveRunStoreError::RevisionConflict { .. })) {
            "active_run_revision_conflict"
        } else { "application_persistence_failed" };
        PlayerGameActorError::new(code, error.to_string())
    })?;
    *expected_revision = Some(accepted.revision);
    Ok((prepared.commit(), accepted.recovery_pending))
}

fn publish_prepared_application_bound<T>(
    prepared: game_core::application::PreparedApplication<'_, T>,
    approved: Option<&std::sync::Arc<crate::mod_admission::ApprovedPersistence>>,
    required: Option<&game_core::mod_validation::RequiredConfiguration>,
    active_store: Option<&std::sync::Arc<std::sync::Mutex<crate::persistence::active_run_store::ActiveRunStore>>>,
    profile_persistence: Option<&std::sync::Arc<std::sync::Mutex<crate::persistence::player_profile_store::PlayerProfilePersistence>>>,
    expected_revision: &mut Option<crate::persistence::active_run_store::ActiveRunRevision>,
    checkpoint_before: Option<&game_core::application::ApplicationCheckpoint>, force_revision: bool,
) -> Result<(T, bool), PlayerGameActorError> {
    if let Some(approved) = approved {
        let required = required.ok_or_else(|| PlayerGameActorError::new("required_configuration_missing", "approved actor has no pinned configuration"))?;
        approved.publish(prepared, required, expected_revision, checkpoint_before, force_revision).map_err(|e| PlayerGameActorError::new("application_persistence_failed", e))
    } else {
        publish_prepared_application(prepared, active_store, profile_persistence, expected_revision, checkpoint_before, force_revision)
    }
}
