use actix::{ActorContext, AsyncContext, Handler};
use serde_json::Value;

use game_core::game::behavior::{
    ActionCapabilityDto, ActionKind, ActionScope, BehaviorResult, GameError,
};
use tracing::{info, warn};

use crate::game::player_game_actor::{
    messages::{
        AttachSession, AttachSessionResult, CommandExecutionResult, DetachSession,
        EnsureLiveBattleTick, ExecuteAdminCommand, ExecutePlayerBehavior, ForceDisconnect,
        GetEncounterCompendiumProfile, PlayerGameServerMessage, PushServerMessage, QuitPlayerActor,
    },
    state::{
        battle_update_from_behavior_result, behavior_result_to_command_result,
        compress_combat_result_event_log_attachment, PlayerGameActorError, PlayerStateSnapshotDto,
    },
    BattleEventDeliveryCursor, PlayerGameActor,
};
use crate::persistence::active_run_store::ActiveRunLoad;
use crate::persistence::encounter_compendium_store::{
    EncounterCompendiumPersistOutcome, EncounterCompendiumPersistenceState,
};

impl Handler<AttachSession> for PlayerGameActor {
    type Result = Result<AttachSessionResult, PlayerGameActorError>;

    fn handle(&mut self, msg: AttachSession, ctx: &mut Self::Context) -> Self::Result {
        if let Some(timer) = self.disconnect_timer.take() {
            ctx.cancel_future(timer);
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

        self.sync_encounter_compendium_from_persistence_for_snapshot()?;
        let state_snapshot = self.build_state_snapshot()?;
        let mut side_messages = Vec::new();
        if let Some(persistence) = self.encounter_compendium_persistence.as_ref() {
            if persistence
                .lock()
                .map_err(|_| {
                    PlayerGameActorError::new(
                        "encounter_compendium_lock_poisoned",
                        "Encounter compendium persistence lock is poisoned",
                    )
                })?
                .take_recovery_notification()
            {
                side_messages.push(PlayerGameServerMessage::Notification {
                    notification_type: "encounter_compendium_recovered_from_backup".to_string(),
                    payload: serde_json::json!({}),
                });
            }
        }
        if matches!(
            self.game_core.get_state(),
            game_core::game::resources::GameState::InBattle { .. }
        ) {
            let recovery = self
                .game_core
                .active_battle_reconnect_projection(0)
                .map_err(PlayerGameActorError::from)?;
            let game_core::game::behavior::BehaviorResult::BattleAdvanced {
                battle_setup_snapshot: Some(setup),
                battle_update,
                ..
            } = recovery
            else {
                return Err(PlayerGameActorError::new(
                    "invalid_battle_reconnect_projection",
                    "Active battle reconnect did not produce immutable setup and update",
                ));
            };
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
            self.active_session_id = None;
            self.socket = None;
            self.session_control = None;
            self.battle_event_cursor = None;
            self.stop_live_battle_tick(ctx);

            if let Some(timer) = self.disconnect_timer.take() {
                ctx.cancel_future(timer);
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

    fn handle(&mut self, msg: ExecutePlayerBehavior, _ctx: &mut Self::Context) -> Self::Result {
        self.ensure_active_session(msg.session_id)?;
        if matches!(
            &msg.behavior,
            game_core::game::behavior::PlayerBehavior::RetryEncounterCompendium
                | game_core::game::behavior::PlayerBehavior::ResetEncounterCompendium
        ) {
            let execution_result = self.handle_encounter_compendium_management(
                msg.request_id,
                matches!(
                    &msg.behavior,
                    game_core::game::behavior::PlayerBehavior::ResetEncounterCompendium
                ),
            )?;
            self.emit_command_execution_result(&execution_result);
            return Ok(execution_result);
        }
        self.sync_encounter_compendium_from_persistence()?;
        if matches!(
            &msg.behavior,
            game_core::game::behavior::PlayerBehavior::ResetActiveRun
        ) {
            let store = self.active_run_store.clone().ok_or_else(|| {
                PlayerGameActorError::new(
                    "active_run_persistence_unavailable",
                    "Active-run persistence is unavailable",
                )
            })?;
            {
                let store = store.lock().map_err(|_| {
                    PlayerGameActorError::new(
                        "active_run_lock_poisoned",
                        "Active-run persistence lock is poisoned",
                    )
                })?;
                match store.load() {
                    Err(_) => store.invalidate().map_err(|error| {
                        PlayerGameActorError::new(
                            "active_run_persistence_failed",
                            error.to_string(),
                        )
                    })?,
                    Ok(_) => {
                        return Err(PlayerGameActorError::new(
                            "active_run_reset_not_required",
                            "Active-run reset is only available for corrupt data",
                        ));
                    }
                }
            }
            let execution_result = CommandExecutionResult {
                response: PlayerGameServerMessage::CommandResult {
                    request_id: msg.request_id,
                    ok: true,
                    result_type: "ActiveRunReset".to_string(),
                    payload: serde_json::json!({}),
                },
                side_messages: Vec::new(),
                state_snapshot: Some(self.build_state_snapshot()?),
            };
            self.emit_command_execution_result(&execution_result);
            return Ok(execution_result);
        }
        let battle_event_after_seq = self.battle_event_after_seq();
        let rollback = self.game_core.capture_active_run_transaction_rollback();
        let checkpoint_before = self
            .game_core
            .export_active_run_checkpoint()
            .map_err(PlayerGameActorError::from)?;
        let is_continue = matches!(
            &msg.behavior,
            game_core::game::behavior::PlayerBehavior::ContinueRun
        );
        let replacement_requested = matches!(
            &msg.behavior,
            game_core::game::behavior::PlayerBehavior::StartNewGame {
                replace_active_run: true,
                ..
            }
        );
        if matches!(
            &msg.behavior,
            game_core::game::behavior::PlayerBehavior::StartNewGame {
                replace_active_run: false,
                ..
            }
        ) {
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
                            "active_run_replacement_confirmation_required",
                            "Starting a new run will replace the existing active run",
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
                .load()
                .map_err(|error| {
                    PlayerGameActorError::new("active_run_load_failed", error.to_string())
                })?;
            let ActiveRunLoad::Loaded {
                checkpoint,
                recovered_from_backup: recovered,
            } = loaded
            else {
                return Err(PlayerGameActorError::new(
                    "active_run_unavailable",
                    "No valid active run is available to continue",
                ));
            };
            recovered_from_backup = recovered;
            self.game_core.continue_from_active_run_checkpoint(
                self.player_id,
                &checkpoint.core_payload_json,
                checkpoint.loads_used,
                checkpoint.run_peak_certified_power,
            )
        } else {
            self.game_core.execute_with_transport_context(
                self.player_id,
                msg.behavior,
                Some(&msg.request_id),
                battle_event_after_seq,
                msg.battle_authority,
            )
        };
        let result = match execution {
            Ok(result) => result,
            Err(
                error @ (GameError::MissingBattleStateAuthority { .. }
                | GameError::StaleBattleStateAuthority { .. }),
            ) => {
                let execution_result = self.battle_authority_rejection_result(
                    error,
                    msg.request_id,
                    battle_event_after_seq,
                )?;
                self.emit_command_execution_result(&execution_result);
                return Ok(execution_result);
            }
            Err(error) => return Err(PlayerGameActorError::from(error)),
        };
        if !is_continue {
            if let Some(store) = self.active_run_store.as_ref() {
                let checkpoint_after = self
                    .game_core
                    .export_active_run_checkpoint()
                    .map_err(PlayerGameActorError::from)?;
                let terminal = matches!(
                    &result,
                    BehaviorResult::RunComplete { .. }
                        | BehaviorResult::RunFailed { .. }
                        | BehaviorResult::RunAbandoned
                );
                let persistence_result = {
                    let store = store.lock().map_err(|_| {
                        PlayerGameActorError::new(
                            "active_run_lock_poisoned",
                            "Active-run persistence lock is poisoned",
                        )
                    })?;
                    if terminal || replacement_requested {
                        store.invalidate()
                    } else if checkpoint_after != checkpoint_before {
                        checkpoint_after
                            .as_ref()
                            .map_or(Ok(()), |checkpoint| store.publish(checkpoint))
                    } else {
                        Ok(())
                    }
                };
                if let Err(error) = persistence_result {
                    let rollback = rollback.ok_or_else(|| {
                        PlayerGameActorError::new(
                            "active_run_rollback_unavailable",
                            "Durable active-run mutation had no rollback snapshot",
                        )
                    })?;
                    self.game_core
                        .restore_active_run_transaction_rollback(rollback);
                    // The local compendium is a separate aggregate and transaction.
                    // Preserve and publish its latest full union even when the active-run
                    // transaction must roll back.
                    if self.persist_encounter_compendium_profile()?
                        == EncounterCompendiumPersistOutcome::BecameDirty
                    {
                        self.push_to_active_socket(PlayerGameServerMessage::Notification {
                            notification_type: "encounter_compendium_save_failed".to_string(),
                            payload: serde_json::json!({ "unsaved_progress_may_be_lost": true }),
                        });
                    }
                    return Err(PlayerGameActorError::new(
                        "active_run_persistence_failed",
                        error.to_string(),
                    ));
                }
            }
        }
        let compendium_persist_outcome = self.persist_encounter_compendium_profile()?;
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
        if compendium_persist_outcome == EncounterCompendiumPersistOutcome::BecameDirty {
            command_result
                .side_messages
                .push(PlayerGameServerMessage::Notification {
                    notification_type: "encounter_compendium_save_failed".to_string(),
                    payload: serde_json::json!({ "unsaved_progress_may_be_lost": true }),
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

    fn handle(&mut self, msg: ExecuteAdminCommand, _ctx: &mut Self::Context) -> Self::Result {
        self.ensure_active_session(msg.session_id)?;
        let result = self
            .game_core
            .execute_admin_command(msg.command)
            .map_err(PlayerGameActorError::from)?;
        let response = PlayerGameServerMessage::AdminResult {
            request_id: msg.request_id,
            ok: true,
            result_type: result.result_type.to_string(),
            payload: result.payload,
        };
        let state_snapshot = self.build_state_snapshot()?;
        Ok(CommandExecutionResult {
            response,
            side_messages: Vec::new(),
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
        actix::MessageResult(self.game_core.encounter_compendium_profile())
    }
}

impl PlayerGameActor {
    fn sync_encounter_compendium_from_persistence_for_snapshot(
        &mut self,
    ) -> Result<(), PlayerGameActorError> {
        let Some(persistence) = self.encounter_compendium_persistence.clone() else {
            return Ok(());
        };
        let profile = {
            let persistence = persistence.lock().map_err(|_| {
                PlayerGameActorError::new(
                    "encounter_compendium_lock_poisoned",
                    "Encounter compendium persistence lock is poisoned",
                )
            })?;
            match persistence.state() {
                EncounterCompendiumPersistenceState::Fatal { .. } => None,
                _ => Some(persistence.profile().map_err(|issue| {
                    PlayerGameActorError::new("encounter_compendium_recovery_required", issue)
                })?),
            }
        };
        if let Some(profile) = profile {
            self.game_core
                .merge_encounter_compendium_profile(profile)
                .map_err(PlayerGameActorError::from)?;
        }
        Ok(())
    }

    fn sync_encounter_compendium_from_persistence(&mut self) -> Result<(), PlayerGameActorError> {
        let Some(persistence) = self.encounter_compendium_persistence.as_ref() else {
            return Ok(());
        };
        let profile = persistence
            .lock()
            .map_err(|_| {
                PlayerGameActorError::new(
                    "encounter_compendium_lock_poisoned",
                    "Encounter compendium persistence lock is poisoned",
                )
            })?
            .profile()
            .map_err(|issue| {
                PlayerGameActorError::new("encounter_compendium_recovery_required", issue)
            })?;
        self.game_core
            .merge_encounter_compendium_profile(profile)
            .map_err(PlayerGameActorError::from)
    }

    fn persist_encounter_compendium_profile(
        &mut self,
    ) -> Result<EncounterCompendiumPersistOutcome, PlayerGameActorError> {
        let Some(persistence) = self.encounter_compendium_persistence.as_ref() else {
            return Ok(EncounterCompendiumPersistOutcome::Unchanged);
        };
        Ok(persistence
            .lock()
            .map_err(|_| {
                PlayerGameActorError::new(
                    "encounter_compendium_lock_poisoned",
                    "Encounter compendium persistence lock is poisoned",
                )
            })?
            .merge_and_persist(self.game_core.encounter_compendium_profile()))
    }

    fn handle_encounter_compendium_management(
        &mut self,
        request_id: String,
        reset: bool,
    ) -> Result<CommandExecutionResult, PlayerGameActorError> {
        if reset
            && !matches!(
                self.game_core.get_state(),
                game_core::game::resources::GameState::NotStarted
            )
        {
            return Err(PlayerGameActorError::new(
                "encounter_compendium_reset_requires_title",
                "Encounter compendium reset is only available before a run starts",
            ));
        }
        let persistence = self
            .encounter_compendium_persistence
            .clone()
            .ok_or_else(|| {
                PlayerGameActorError::new(
                    "encounter_compendium_persistence_unavailable",
                    "Encounter compendium persistence is unavailable",
                )
            })?;
        let mut persist_outcome = EncounterCompendiumPersistOutcome::Unchanged;
        let mut recovered_from_backup = false;
        let profile_clean;
        {
            let mut persistence = persistence.lock().map_err(|_| {
                PlayerGameActorError::new(
                    "encounter_compendium_lock_poisoned",
                    "Encounter compendium persistence lock is poisoned",
                )
            })?;
            if reset {
                persistence.reset().map_err(|error| {
                    PlayerGameActorError::new(
                        "encounter_compendium_reset_failed",
                        error.to_string(),
                    )
                })?;
                self.game_core.reset_encounter_compendium_profile();
            } else if matches!(
                persistence.state(),
                EncounterCompendiumPersistenceState::Fatal { .. }
            ) {
                persistence.reload().map_err(|error| {
                    PlayerGameActorError::new(
                        "encounter_compendium_retry_failed",
                        error.to_string(),
                    )
                })?;
                recovered_from_backup = persistence.take_recovery_notification();
            } else {
                persist_outcome = persistence.retry();
            }
            if let Ok(profile) = persistence.profile() {
                self.game_core
                    .merge_encounter_compendium_profile(profile)
                    .map_err(PlayerGameActorError::from)?;
            }
            profile_clean = matches!(
                persistence.state(),
                EncounterCompendiumPersistenceState::Clean
            );
        }
        let mut side_messages = Vec::new();
        if recovered_from_backup {
            side_messages.push(PlayerGameServerMessage::Notification {
                notification_type: "encounter_compendium_recovered_from_backup".to_string(),
                payload: serde_json::json!({}),
            });
        }
        if persist_outcome == EncounterCompendiumPersistOutcome::BecameDirty {
            side_messages.push(PlayerGameServerMessage::Notification {
                notification_type: "encounter_compendium_save_failed".to_string(),
                payload: serde_json::json!({ "unsaved_progress_may_be_lost": true }),
            });
        }
        Ok(CommandExecutionResult {
            response: PlayerGameServerMessage::CommandResult {
                request_id,
                ok: true,
                result_type: if reset {
                    "EncounterCompendiumReset"
                } else {
                    "EncounterCompendiumRetry"
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
        let snapshot = self
            .game_core
            .get_run_snapshot_dto()
            .map_err(PlayerGameActorError::from)?;
        let compressed_event_log = self
            .game_core
            .get_combat_result_event_log_attachment()
            .map(|attachment| compress_combat_result_event_log_attachment(&attachment))
            .transpose()?;

        let mut value =
            serde_json::to_value(PlayerStateSnapshotDto::new(snapshot, compressed_event_log))
                .map_err(|error| {
                    PlayerGameActorError::new("serialization_failed", error.to_string())
                })?;
        if matches!(
            self.game_core.get_state(),
            game_core::game::resources::GameState::NotStarted
        ) {
            if let Some(store) = self.active_run_store.as_ref() {
                let status = match store.lock() {
                    Ok(store) => match store.load() {
                        Ok(ActiveRunLoad::Missing) => serde_json::json!({
                            "available": false,
                            "recovered_from_backup": false,
                            "reset_required": false,
                        }),
                        Ok(ActiveRunLoad::Loaded {
                            recovered_from_backup,
                            ..
                        }) => {
                            if let Some(capabilities) = value["action_capabilities"].as_array_mut()
                            {
                                capabilities.push(
                                    serde_json::to_value(ActionCapabilityDto::enabled(
                                        ActionKind::ContinueRun,
                                        ActionScope::Recovery,
                                    ))
                                    .map_err(|error| {
                                        PlayerGameActorError::new(
                                            "serialization_failed",
                                            error.to_string(),
                                        )
                                    })?,
                                );
                            }
                            serde_json::json!({
                                "available": true,
                                "recovered_from_backup": recovered_from_backup,
                                "reset_required": false,
                            })
                        }
                        Err(error) => serde_json::json!({
                            "available": false,
                            "recovered_from_backup": false,
                            "reset_required": true,
                            "system_issue": error.to_string(),
                        }),
                    },
                    Err(_) => serde_json::json!({
                        "available": false,
                        "recovered_from_backup": false,
                        "reset_required": true,
                        "system_issue": "active-run persistence lock is poisoned",
                    }),
                };
                value["active_run_continue"] = status;
                if value["active_run_continue"]["reset_required"] == serde_json::json!(true) {
                    if let Some(capabilities) = value["action_capabilities"].as_array_mut() {
                        capabilities.push(
                            serde_json::to_value(ActionCapabilityDto::enabled(
                                ActionKind::ResetActiveRun,
                                ActionScope::Recovery,
                            ))
                            .map_err(|error| {
                                PlayerGameActorError::new("serialization_failed", error.to_string())
                            })?,
                        );
                    }
                }
            }
        }
        if let Some(persistence) = self.encounter_compendium_persistence.as_ref() {
            let persistence = persistence.lock().map_err(|_| {
                PlayerGameActorError::new(
                    "encounter_compendium_lock_poisoned",
                    "Encounter compendium persistence lock is poisoned",
                )
            })?;
            let (state, dirty, recovered_from_backup, reset_required, system_issue) =
                match persistence.state() {
                    EncounterCompendiumPersistenceState::Clean => {
                        ("clean", false, false, false, None)
                    }
                    EncounterCompendiumPersistenceState::RecoveredFromBackup => {
                        ("recovered", true, true, false, None)
                    }
                    EncounterCompendiumPersistenceState::Dirty { issue } => {
                        ("dirty", true, false, false, Some(issue.clone()))
                    }
                    EncounterCompendiumPersistenceState::Fatal { issue } => {
                        ("error", false, false, true, Some(issue.clone()))
                    }
                };
            value["encounter_compendium_persistence"] = serde_json::json!({
                "state": state,
                "dirty": dirty,
                "recovered_from_backup": recovered_from_backup,
                "reset_required": reset_required,
                "system_issue": system_issue,
            });
            if reset_required
                && matches!(
                    self.game_core.get_state(),
                    game_core::game::resources::GameState::NotStarted
                )
            {
                // Fatal profile recovery is a startup gate. Do not leave gameplay,
                // active-run recovery, or unrelated management capabilities enabled
                // while the preserved profile copies still require a user decision.
                value["action_capabilities"] = serde_json::json!([]);
            }
            if matches!(
                self.game_core.get_state(),
                game_core::game::resources::GameState::NotStarted
            ) {
                if let Some(capabilities) = value["action_capabilities"].as_array_mut() {
                    if dirty || reset_required {
                        capabilities.push(
                            serde_json::to_value(ActionCapabilityDto::enabled(
                                ActionKind::RetryEncounterCompendium,
                                ActionScope::Recovery,
                            ))
                            .map_err(|error| {
                                PlayerGameActorError::new("serialization_failed", error.to_string())
                            })?,
                        );
                    }
                    capabilities.push(
                        serde_json::to_value(ActionCapabilityDto::enabled(
                            ActionKind::ResetEncounterCompendium,
                            ActionScope::Recovery,
                        ))
                        .map_err(|error| {
                            PlayerGameActorError::new("serialization_failed", error.to_string())
                        })?,
                    );
                }
            }
        }
        Ok(value)
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
        error: GameError,
        request_id: String,
        battle_event_after_seq: u64,
    ) -> Result<CommandExecutionResult, PlayerGameActorError> {
        debug_assert!(matches!(
            error,
            GameError::MissingBattleStateAuthority { .. }
                | GameError::StaleBattleStateAuthority { .. }
        ));
        let mapped = PlayerGameActorError::from(error);
        let recovery = self
            .game_core
            .active_battle_reconnect_projection(battle_event_after_seq)
            .map_err(PlayerGameActorError::from)?;
        let BehaviorResult::BattleAdvanced { battle_update, .. } = recovery else {
            return Err(PlayerGameActorError::new(
                "invalid_battle_resync_projection",
                "Battle authority recovery did not produce a battle update",
            ));
        };
        Ok(CommandExecutionResult {
            response: mapped.to_server_message(Some(request_id)),
            side_messages: vec![PlayerGameServerMessage::battle_resync(battle_update)],
            state_snapshot: None,
        })
    }

    fn record_battle_message_delivery(&mut self, message: &PlayerGameServerMessage) {
        let delivered = match message {
            PlayerGameServerMessage::BattleUpdate {
                battle_uuid,
                events_delta,
                ..
            } => Some((*battle_uuid, events_delta.to_seq)),
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

    fn ensure_live_battle_tick(&mut self, ctx: &mut <Self as actix::Actor>::Context) {
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
        if self.socket.is_none() || !self.has_active_live_battle() {
            self.stop_live_battle_tick(ctx);
            return;
        }

        let delta_ms = self.live_battle_tick_interval.as_millis();
        let delta_ms = u64::try_from(delta_ms).unwrap_or(u64::MAX).max(1);
        let battle_event_after_seq = self.battle_event_after_seq();
        let result = match self
            .game_core
            .advance_active_battle_for_server_tick_after_seq(delta_ms, battle_event_after_seq)
        {
            Ok(result) => result,
            Err(error) => {
                warn!(
                    "Live battle tick failed for player {}: {:?}",
                    self.player_id, error
                );
                self.push_to_active_socket(
                    PlayerGameActorError::from(error).to_server_message(None),
                );
                self.stop_live_battle_tick(ctx);
                return;
            }
        };
        let Some(result) = result else {
            return;
        };

        let finished = matches!(
            result,
            game_core::game::behavior::BehaviorResult::BattleAdvanced { finished: true, .. }
        );

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

#[cfg(test)]
mod tests {
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
    use crate::persistence::{
        active_run_store::ActiveRunStore,
        encounter_compendium_store::{EncounterCompendiumPersistence, EncounterCompendiumStore},
    };
    use actix::{Actor, Context};
    use actix_web::rt::time;
    use game_core::game::{
        behavior::{BehaviorResult, GameStateContextDto, PlayerBehavior},
        combat_preview::CombatNodeType,
        data::{
            employee_data::EmployeeDefinitionDatabase,
            equipment_data::{EquipmentMetadata, EquipmentType},
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
        world::{ActiveRunCheckpointExport, GameCore},
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

    fn starter_test_loadout() -> EmployeeInitialLoadout {
        EmployeeInitialLoadout {
            equipment_ids: vec!["standard_armor".to_string()],
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
            starter_eligible,
            initial_loadout: if starter_eligible {
                starter_test_loadout()
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

    fn starter_test_skill_fragment() -> SkillFragmentMetadata {
        SkillFragmentMetadata {
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
        GameDataBuilder::empty()
            .with_equipment(vec![starter_test_equipment()])
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
                    GameMode::Standard,
                    standard_vn_id,
                ),
                test_run_start_visual_novel_trigger(
                    "run_start_endless_hq_briefing",
                    GameMode::Endless,
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
        let definitions = (0..5)
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
        let base =
            GameDataBase::load_live_embedded().expect("embedded live gameplay RON must be valid");
        let mut pve_encounters = base.pve_data.encounters.clone();
        if let Some(survive_timer_ms) = survive_timer_ms {
            for encounter in &mut pve_encounters {
                if encounter.node_type == Some(CombatNodeType::Defense) {
                    encounter.survive_timer_ms = Some(survive_timer_ms);
                }
            }
        }

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
            .with_boss_omen_data(base.boss_omen_data.clone())
            .with_run_policy_data(base.run_policy.clone())
            .with_skill_data(base.skill_data.clone())
            .with_buff_data(base.buff_data.clone())
            .with_skill_fragment_data(base.skill_fragment_data.clone())
            .build_arc()
    }

    fn start_new_game_with_default_starters(
        core: &mut GameCore,
        player_id: Uuid,
    ) -> BehaviorResult {
        let start = core
            .execute(
                player_id,
                PlayerBehavior::StartNewGame {
                    game_mode: GameMode::Standard,
                    replace_active_run: false,
                },
            )
            .expect("start new game should open pre-run briefing");
        let BehaviorResult::RunStartFlowStarted {
            visual_novel: Some(visual_novel),
            ..
        } = start
        else {
            panic!("start new game should return HQ briefing visual novel");
        };
        let advanced = core
            .execute(
                player_id,
                PlayerBehavior::AdvanceVisualNovel {
                    session_id: visual_novel.session_id,
                    current_scene_id: VisualNovelSceneId::new(visual_novel.current_scene_id),
                },
            )
            .expect("HQ briefing advance should open starter selection");
        assert!(
            matches!(
                advanced,
                BehaviorResult::VisualNovelState { visual_novel: None }
            ),
            "single-scene test HQ briefing should complete"
        );
        let snapshot = core
            .get_run_snapshot_dto()
            .expect("starter selection snapshot should build");
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
        core.execute(
            player_id,
            PlayerBehavior::SelectStarterEmployees {
                employee_definition_ids,
            },
        )
        .expect("default starter employee selection should start the run")
    }

    fn game_core_waiting_on_live_defense_node(player_id: Uuid) -> GameCore {
        game_core_waiting_on_live_defense_node_with_data(player_id, defense_game_data())
    }

    fn game_core_waiting_on_live_survival_defense_node(player_id: Uuid) -> GameCore {
        game_core_waiting_on_live_defense_node_with_data(
            player_id,
            defense_game_data_with_survive_timer(Some(1)),
        )
    }

    fn game_core_waiting_on_live_defense_node_with_data(
        player_id: Uuid,
        game_data: Arc<GameDataBase>,
    ) -> GameCore {
        for seed in 0..1_000 {
            let mut core = GameCore::new(game_data.clone(), seed);
            let BehaviorResult::StarterEmployeesSelected { mut map, .. } =
                start_new_game_with_default_starters(&mut core, player_id)
            else {
                panic!("starter selection should enter the run map");
            };

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
                        .execute(player_id, PlayerBehavior::SelectMapNode { node_id })
                        .expect("available node preview should succeed");
                    let is_live_defense = matches!(
                        &preview,
                        BehaviorResult::NodePreview {
                            combat_preview: Some(combat_preview),
                            ..
                        } if combat_preview.node_type == CombatNodeType::Defense
                    );
                    if is_live_defense {
                        return core;
                    }

                    core.execute(player_id, PlayerBehavior::CancelSelectedNode)
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
                    .execute(
                        player_id,
                        PlayerBehavior::SelectMapNode {
                            node_id: start_node_id,
                        },
                    )
                    .expect("start node preview should succeed");
                assert!(matches!(
                    preview,
                    BehaviorResult::NodePreview {
                        category: MapNodeCategory::Start,
                        ..
                    }
                ));
                let entered = core
                    .execute(player_id, PlayerBehavior::ConfirmEnterNode)
                    .expect("start node confirm should succeed");
                assert!(matches!(
                    entered,
                    BehaviorResult::NodeEntered {
                        category: MapNodeCategory::Start,
                        ..
                    }
                ));
                let completed = core
                    .execute(player_id, PlayerBehavior::CompleteNode)
                    .expect("start node complete should succeed");
                let BehaviorResult::NodeCompleted { map: next_map, .. } = completed else {
                    panic!("start node completion should return updated map");
                };
                map = next_map;
            }
        }

        panic!("expected at least one generated run seed to start near a live Defense node");
    }

    #[test]
    fn player_state_snapshot_preserves_core_snapshot_root_fields() {
        let core = GameCore::new(empty_game_data(), 7);
        let core_snapshot = core
            .get_run_snapshot_dto()
            .expect("core snapshot should build");
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

    #[actix_web::test]
    async fn execute_behavior_returns_updated_state_snapshot() {
        let load_balance = LoadBalanceActor::new().start();
        let actor = PlayerGameActor::new(
            Uuid::new_v4(),
            GameCore::new(empty_game_data(), 7),
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
                behavior: PlayerBehavior::StartNewGame {
                    game_mode: GameMode::Standard,
                    replace_active_run: false,
                },
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
        let store = EncounterCompendiumStore::open(directory.path()).unwrap();
        store
            .publish(
                &game_core::game::encounter_compendium::EncounterCompendiumProfileState::default(),
            )
            .unwrap();
        std::fs::write(
            directory.path().join("encounter_compendium.json"),
            b"corrupt",
        )
        .unwrap();
        let persistence = Arc::new(Mutex::new(EncounterCompendiumPersistence::bootstrap(store)));

        let load_balance = LoadBalanceActor::new().start();
        let actor = PlayerGameActor::new(
            Uuid::new_v4(),
            GameCore::new(empty_game_data(), 7),
            load_balance,
        )
        .with_encounter_compendium_persistence(persistence)
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
            attach.state_snapshot["encounter_compendium_persistence"]["state"],
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
            std::collections::BTreeSet::from([
                "ResetEncounterCompendium",
                "RetryEncounterCompendium",
            ])
        );
    }

    #[actix_web::test]
    async fn clean_title_exposes_confirmable_full_compendium_reset_without_retry() {
        let directory = tempfile::tempdir().unwrap();
        let persistence = Arc::new(Mutex::new(EncounterCompendiumPersistence::bootstrap(
            EncounterCompendiumStore::open(directory.path()).unwrap(),
        )));
        let load_balance = LoadBalanceActor::new().start();
        let actor = PlayerGameActor::new(
            Uuid::new_v4(),
            GameCore::new(empty_game_data(), 7),
            load_balance,
        )
        .with_encounter_compendium_persistence(persistence)
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
            .any(|capability| capability["action"] == "ResetEncounterCompendium"));
        assert!(actions
            .iter()
            .all(|capability| capability["action"] != "RetryEncounterCompendium"));
    }

    #[actix_web::test]
    async fn attach_session_merges_durable_compendium_before_building_snapshot() {
        let game_data =
            GameDataBase::load_live_embedded().expect("embedded live gameplay RON must be valid");
        let abnormality_id = game_data
            .abnormality_data
            .items
            .first()
            .expect("live catalog must contain an abnormality")
            .id
            .clone();
        let profile = game_core::game::encounter_compendium::EncounterCompendiumProfileState {
            discovered_abnormality_ids: std::collections::BTreeSet::from([abnormality_id.clone()]),
            ..Default::default()
        };
        let directory = tempfile::tempdir().unwrap();
        let store = EncounterCompendiumStore::open(directory.path()).unwrap();
        store.publish(&profile).unwrap();
        let persistence = Arc::new(Mutex::new(EncounterCompendiumPersistence::bootstrap(store)));

        let load_balance = LoadBalanceActor::new().start();
        let actor = PlayerGameActor::new(Uuid::new_v4(), GameCore::new(game_data, 7), load_balance)
            .with_encounter_compendium_persistence(persistence)
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
            .expect("durable discovered abnormality must be projected on attach");
        assert_eq!(projected["discovered"], true);
    }

    #[actix_web::test]
    async fn active_run_write_failure_does_not_rollback_or_skip_compendium_publish() {
        let active_directory = tempfile::tempdir().unwrap();
        let active_store = Arc::new(Mutex::new(
            ActiveRunStore::open(active_directory.path()).unwrap(),
        ));
        std::fs::create_dir(active_directory.path().join("active_run.invalidated")).unwrap();

        let profile_directory = tempfile::tempdir().unwrap();
        let profile_persistence = Arc::new(Mutex::new(EncounterCompendiumPersistence::bootstrap(
            EncounterCompendiumStore::open(profile_directory.path()).unwrap(),
        )));
        let mut core = GameCore::new(empty_game_data(), 7);
        core.merge_encounter_compendium_profile(
            game_core::game::encounter_compendium::EncounterCompendiumProfileState {
                discovered_abnormality_ids: std::collections::BTreeSet::from([
                    "dormant_before_active_failure".to_string(),
                ]),
                ..Default::default()
            },
        )
        .unwrap();

        let load_balance = LoadBalanceActor::new().start();
        let actor = PlayerGameActor::new(Uuid::new_v4(), core, load_balance)
            .with_active_run_store(active_store)
            .with_encounter_compendium_persistence(profile_persistence.clone())
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

        let error = actor
            .send(ExecutePlayerBehavior {
                session_id,
                request_id: "independent-profile-publish".to_string(),
                behavior: PlayerBehavior::StartNewGame {
                    game_mode: GameMode::Standard,
                    replace_active_run: true,
                },
                battle_authority: None,
                battle_side_message: BattleSideMessageKind::BattleUpdate,
            })
            .await
            .unwrap()
            .unwrap_err();

        assert_eq!(error.code, "active_run_persistence_failed");
        let persisted = profile_persistence.lock().unwrap().profile().unwrap();
        assert!(persisted
            .discovered_abnormality_ids
            .contains("dormant_before_active_failure"));
        assert!(profile_directory
            .path()
            .join("encounter_compendium.json")
            .is_file());
    }

    #[actix_web::test]
    async fn compendium_write_failure_does_not_rollback_active_run_commit() {
        let active_directory = tempfile::tempdir().unwrap();
        let active_store = Arc::new(Mutex::new(
            ActiveRunStore::open(active_directory.path()).unwrap(),
        ));
        active_store
            .lock()
            .unwrap()
            .publish(&ActiveRunCheckpointExport {
                core_payload_json: b"existing-active-run".to_vec(),
                loads_used: 0,
                run_peak_certified_power: 100,
            })
            .unwrap();

        let profile_directory = tempfile::tempdir().unwrap();
        let profile_persistence = Arc::new(Mutex::new(EncounterCompendiumPersistence::bootstrap(
            EncounterCompendiumStore::open(profile_directory.path()).unwrap(),
        )));
        std::fs::create_dir(profile_directory.path().join("encounter_compendium.tmp")).unwrap();
        let mut core = GameCore::new(empty_game_data(), 7);
        core.merge_encounter_compendium_profile(
            game_core::game::encounter_compendium::EncounterCompendiumProfileState {
                discovered_abnormality_ids: std::collections::BTreeSet::from([
                    "dirty_but_committed_in_memory".to_string(),
                ]),
                ..Default::default()
            },
        )
        .unwrap();

        let load_balance = LoadBalanceActor::new().start();
        let actor = PlayerGameActor::new(Uuid::new_v4(), core, load_balance)
            .with_active_run_store(active_store.clone())
            .with_encounter_compendium_persistence(profile_persistence.clone())
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
                behavior: PlayerBehavior::StartNewGame {
                    game_mode: GameMode::Standard,
                    replace_active_run: true,
                },
                battle_authority: None,
                battle_side_message: BattleSideMessageKind::BattleUpdate,
            })
            .await
            .unwrap()
            .expect("profile failure must not fail the gameplay command");

        assert_eq!(
            active_store.lock().unwrap().load().unwrap(),
            ActiveRunLoad::Missing
        );
        assert!(result.side_messages.iter().any(|message| matches!(
            message,
            PlayerGameServerMessage::Notification { notification_type, payload }
                if notification_type == "encounter_compendium_save_failed"
                    && payload["unsaved_progress_may_be_lost"] == true
        )));
        let persistence = profile_persistence.lock().unwrap();
        assert!(matches!(
            persistence.state(),
            EncounterCompendiumPersistenceState::Dirty { .. }
        ));
        assert!(persistence
            .profile()
            .unwrap()
            .discovered_abnormality_ids
            .contains("dirty_but_committed_in_memory"));
    }

    #[actix_web::test]
    async fn backup_recovery_notification_is_emitted_once_across_reconnects() {
        let directory = tempfile::tempdir().unwrap();
        let store = EncounterCompendiumStore::open(directory.path()).unwrap();
        let first = game_core::game::encounter_compendium::EncounterCompendiumProfileState {
            discovered_abnormality_ids: std::collections::BTreeSet::from([
                "first_generation".to_string()
            ]),
            ..Default::default()
        };
        let second = game_core::game::encounter_compendium::EncounterCompendiumProfileState {
            discovered_abnormality_ids: std::collections::BTreeSet::from([
                "first_generation".to_string(),
                "second_generation".to_string(),
            ]),
            ..Default::default()
        };
        store.publish(&first).unwrap();
        store.publish(&second).unwrap();
        std::fs::write(
            directory.path().join("encounter_compendium.json"),
            b"corrupt",
        )
        .unwrap();
        let persistence = Arc::new(Mutex::new(EncounterCompendiumPersistence::bootstrap(store)));

        let load_balance = LoadBalanceActor::new().start();
        let actor = PlayerGameActor::new(
            Uuid::new_v4(),
            GameCore::new(empty_game_data(), 7),
            load_balance,
        )
        .with_encounter_compendium_persistence(persistence)
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
                        if notification_type == "encounter_compendium_recovered_from_backup"
                ))
                .count(),
            1
        );
        assert!(second_attach.side_messages.iter().all(|message| !matches!(
            message,
            PlayerGameServerMessage::Notification { notification_type, .. }
                if notification_type == "encounter_compendium_recovered_from_backup"
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
                core_payload_json: b"opaque-core-payload".to_vec(),
                loads_used: 2,
                run_peak_certified_power: 100,
            })
            .unwrap();
        let load_balance = LoadBalanceActor::new().start();
        let actor = PlayerGameActor::new(
            Uuid::new_v4(),
            GameCore::new(empty_game_data(), 7),
            load_balance,
        )
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
                behavior: PlayerBehavior::StartNewGame {
                    game_mode: GameMode::Standard,
                    replace_active_run: false,
                },
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
                behavior: PlayerBehavior::StartNewGame {
                    game_mode: GameMode::Standard,
                    replace_active_run: true,
                },
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
                core_payload_json: b"opaque-core-payload".to_vec(),
                loads_used: 0,
                run_peak_certified_power: 100,
            })
            .unwrap();
        std::fs::write(directory.path().join("active_run.json"), b"corrupt").unwrap();
        let load_balance = LoadBalanceActor::new().start();
        let actor = PlayerGameActor::new(
            Uuid::new_v4(),
            GameCore::new(empty_game_data(), 7),
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
                behavior: PlayerBehavior::ResetActiveRun,
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
            GameCore::new(empty_game_data(), 9),
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
            GameCore::new(empty_game_data(), 13),
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
                behavior: PlayerBehavior::ConfirmEnterNode,
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
                PlayerGameServerMessage::BattleSetupSnapshot {
                    battle_uuid: setup_battle_uuid,
                    setup_version: 1,
                    routes,
                    deployment_zones,
                    ..
                },
                PlayerGameServerMessage::BattleUpdate {
                    battle_uuid,
                    battle_state_revision,
                    action_capabilities,
                    server_battle_time_ms: 0,
                    events_delta,
                    checkpoint,
                    visual_novel,
                }
            ] if !battle_uuid.is_nil()
                && setup_battle_uuid == battle_uuid
                && !routes.is_empty()
                && !deployment_zones.is_empty()
                && events_delta.after_seq == 0
                && *battle_state_revision == 1
                && action_capabilities.iter().any(|capability| {
                    capability.action == game_core::game::behavior::ActionKind::PauseBattle
                        && capability.enabled
                })
                && events_delta.to_seq == checkpoint.at_seq
                && checkpoint.battle_time_ms == 0
                && visual_novel.is_none()
        ));
        assert!(
            result.state_snapshot.is_none(),
            "battle start is represented by battle_setup_snapshot plus battle_update, not full state_snapshot"
        );
        let (started_battle_uuid, started_revision, started_to_seq) = result
            .side_messages
            .iter()
            .find_map(|message| match message {
                PlayerGameServerMessage::BattleUpdate {
                    battle_uuid,
                    battle_state_revision,
                    events_delta,
                    ..
                } => Some((*battle_uuid, *battle_state_revision, events_delta.to_seq)),
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
                    PlayerGameServerMessage::BattleSetupSnapshot { battle_uuid, .. },
                    PlayerGameServerMessage::BattleUpdate {
                        battle_uuid: update_battle_uuid,
                        battle_state_revision,
                        events_delta,
                        ..
                    }
                ] if request_id == "enter-live-defense"
                    && payload["battle_state_revision"] == 1
                    && battle_uuid == update_battle_uuid
                    && *battle_state_revision == 1
                    && events_delta.to_seq == started_to_seq
            ));
            messages.clear();
        }
        actor.do_send(EnsureLiveBattleTick);

        let resync = actor
            .send(ExecutePlayerBehavior {
                session_id,
                request_id: "resync-live-defense".to_string(),
                behavior: PlayerBehavior::RequestBattleState { since_seq: Some(0) },
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
            PlayerGameServerMessage::BattleUpdate {
                battle_uuid,
                events_delta,
                checkpoint,
                ..
            } => Some((battle_uuid, events_delta, checkpoint)),
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
    async fn stale_battle_authority_emits_error_then_current_resync_without_mutation() {
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
                behavior: PlayerBehavior::ConfirmEnterNode,
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
                PlayerGameServerMessage::BattleUpdate {
                    battle_uuid,
                    battle_state_revision,
                    ..
                } => Some((*battle_uuid, *battle_state_revision)),
                _ => None,
            })
            .expect("battle update authority");
        time::sleep(Duration::from_millis(20)).await;
        messages.lock().expect("messages mutex").clear();

        let rejected = actor
            .send(ExecutePlayerBehavior {
                session_id,
                request_id: "stale-pause".to_string(),
                behavior: PlayerBehavior::PauseBattle,
                battle_authority: Some(game_core::game::behavior::BattleStateAuthorityDto {
                    battle_uuid,
                    battle_state_revision: revision - 1,
                }),
                battle_side_message: BattleSideMessageKind::BattleUpdate,
            })
            .await
            .expect("stale request should complete as an ordered rejection batch")
            .expect("stale rejection should remain transportable");

        assert!(matches!(
            rejected.response,
            PlayerGameServerMessage::Error { ref request_id, ref code, .. }
                if request_id.as_deref() == Some("stale-pause")
                    && code == "stale_battle_state_authority"
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
            ] if request_id.as_deref() == Some("stale-pause")
                && code == "stale_battle_state_authority"
                && *actual_uuid == battle_uuid
                && update.battle_state_revision == revision
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
                behavior: PlayerBehavior::ConfirmEnterNode,
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
                PlayerGameServerMessage::BattleUpdate {
                    battle_uuid,
                    battle_state_revision,
                    ..
                } => Some((*battle_uuid, *battle_state_revision)),
                _ => None,
            })
            .expect("battle update authority");
        time::sleep(Duration::from_millis(20)).await;
        messages.lock().expect("messages mutex").clear();

        let rejected = actor
            .send(ExecutePlayerBehavior {
                session_id,
                request_id: "missing-pause".to_string(),
                behavior: PlayerBehavior::PauseBattle,
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
                behavior: PlayerBehavior::ConfirmEnterNode,
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
                            && state["selected_event"]["compressed_event_log"].is_object()
                )
            })
            .expect("finished live tick should push combat_result state_snapshot");
        let final_update_index = messages
            .iter()
            .position(|message| matches!(message, PlayerGameServerMessage::BattleUpdate { .. }))
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
                behavior: PlayerBehavior::ConfirmEnterNode,
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
                behavior: PlayerBehavior::RequestBattleState {
                    since_seq: Some(u64::MAX),
                },
                battle_authority: None,
                battle_side_message: BattleSideMessageKind::BattleResync,
            })
            .await
            .expect("future resync request should complete")
            .expect_err("future since_seq should be rejected");

        assert_eq!(err.code, "invalid_battle_resync_seq");
    }

    #[actix_web::test]
    async fn attaching_to_active_battle_preserves_revision_and_returns_recovery_batch() {
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
                behavior: PlayerBehavior::ConfirmEnterNode,
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
            1
        );
        assert!(matches!(
            attach.side_messages.as_slice(),
            [
                PlayerGameServerMessage::BattleSetupSnapshot {
                    battle_uuid: setup_battle_uuid,
                    ..
                },
                PlayerGameServerMessage::BattleUpdate {
                    battle_uuid,
                    battle_state_revision: 1,
                    events_delta,
                    ..
                }
            ] if setup_battle_uuid == battle_uuid
                && events_delta.after_seq == 0
                && events_delta.to_seq > 0
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
                behavior: PlayerBehavior::ConfirmEnterNode,
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
                PlayerGameServerMessage::BattleUpdate {
                    battle_uuid,
                    battle_state_revision,
                    ..
                } => Some((*battle_uuid, *battle_state_revision)),
                _ => None,
            })
            .expect("enter should expose battle authority");
        actor.do_send(EnsureLiveBattleTick);

        let paused = actor
            .send(ExecutePlayerBehavior {
                session_id,
                request_id: "pause-live-defense".to_string(),
                behavior: PlayerBehavior::PauseBattle,
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
            PlayerGameServerMessage::BattleUpdate { checkpoint, .. }
                if checkpoint.playback.paused
        )));
        assert!(
            paused.state_snapshot.is_none(),
            "battle playback commands must not be followed by full state_snapshot"
        );
        assert!(!paused
            .side_messages
            .iter()
            .any(|message| matches!(message, PlayerGameServerMessage::BattleSetupSnapshot { .. })));

        let (paused_revision, paused_to_seq) = paused
            .side_messages
            .iter()
            .find_map(|message| match message {
                PlayerGameServerMessage::BattleUpdate {
                    battle_state_revision,
                    events_delta,
                    checkpoint,
                    ..
                } if checkpoint.playback.paused => {
                    Some((*battle_state_revision, events_delta.to_seq))
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
                        PlayerGameServerMessage::BattleUpdate {
                            battle_state_revision,
                            checkpoint,
                            ..
                        } if *battle_state_revision == paused_revision && checkpoint.playback.paused
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
                .any(|message| matches!(message, PlayerGameServerMessage::BattleUpdate { .. })));
        }

        actor
            .send(ExecutePlayerBehavior {
                session_id,
                request_id: "resume-live-defense".to_string(),
                behavior: PlayerBehavior::ResumeBattle,
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
            PlayerGameServerMessage::BattleUpdate {
                battle_state_revision,
                events_delta,
                checkpoint,
                ..
            } if *battle_state_revision > paused_revision
                && events_delta.after_seq >= paused_to_seq
                && events_delta.to_seq == checkpoint.at_seq
        )));
    }
}
