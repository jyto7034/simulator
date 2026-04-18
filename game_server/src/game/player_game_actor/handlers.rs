use actix::{ActorContext, AsyncContext, Handler};
use serde_json::{json, Value};

use tracing::info;

use crate::{
    game::player_game_actor::{
        messages::{
            AttachSession, CommandExecutionResult, DetachSession, ExecutePlayerBehavior,
            ForceDisconnect, PushServerMessage, QuitPlayerActor,
        },
        state::{
            behavior_result_to_command_result, compress_timeline_payload,
            legacy_server_message_to_unity, PlayerGameActorError,
        },
        PlayerGameActor,
    },
    shared::protocol::ServerMessage,
};

impl Handler<AttachSession> for PlayerGameActor {
    type Result = Result<Value, PlayerGameActorError>;

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

        self.active_session_id = Some(msg.session_id);
        self.socket = Some(msg.socket);
        self.session_control = Some(msg.control);
        self.build_state_snapshot()
    }
}

impl Handler<DetachSession> for PlayerGameActor {
    type Result = ();

    fn handle(&mut self, msg: DetachSession, ctx: &mut Self::Context) -> Self::Result {
        if self.active_session_id == Some(msg.session_id) {
            self.active_session_id = None;
            self.socket = None;
            self.session_control = None;

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
        let result = self
            .game_core
            .execute(self.player_id, msg.behavior)
            .map_err(PlayerGameActorError::from)?;
        let response = behavior_result_to_command_result(msg.request_id, result)?;
        let state_snapshot = self.build_state_snapshot()?;

        Ok(CommandExecutionResult {
            response,
            state_snapshot,
        })
    }
}

impl Handler<PushServerMessage> for PlayerGameActor {
    type Result = ();

    fn handle(&mut self, msg: PushServerMessage, _ctx: &mut Self::Context) -> Self::Result {
        self.push_to_active_socket(msg.message);
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
        self.active_session_id = None;
        self.socket = None;
        self.session_control = None;
        // stopped() 에서 LoadBalance Deregister 가 호출됨 → 다음 접속 시 새 Actor 생성.
        ctx.stop();
    }
}

impl Handler<ServerMessage> for PlayerGameActor {
    type Result = ();

    fn handle(&mut self, msg: ServerMessage, _ctx: &mut Self::Context) -> Self::Result {
        self.push_to_active_socket(legacy_server_message_to_unity(msg));
    }
}

impl PlayerGameActor {
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
        let (ordeal, phase) = self
            .game_core
            .get_progression()
            .map_err(PlayerGameActorError::from)?;
        let qliphoth = self
            .game_core
            .get_qliphoth()
            .map_err(PlayerGameActorError::from)?;
        let mut selected_event = self
            .game_core
            .get_selected_event_snapshot_json()
            .map_err(PlayerGameActorError::from)?;

        if let Some((winner, timeline)) = self.game_core.get_active_suppression_replay() {
            if let Some(Value::Object(ref mut object)) = selected_event {
                object.insert(
                    "compressed_timeline".to_string(),
                    serde_json::to_value(compress_timeline_payload(winner, &timeline)?).map_err(
                        |error| {
                            PlayerGameActorError::new("serialization_failed", error.to_string())
                        },
                    )?,
                );
            }
        }

        Ok(json!({
            "game_state": self.game_core.game_state_name(),
            "enkephalin": self.game_core.get_enkephalin(),
            "progression": {
                "ordeal": ordeal,
                "phase": phase,
            },
            "qliphoth": {
                "amount": qliphoth.amount(),
                "level": self.game_core.qliphoth_level_name(qliphoth.level()),
            },
            "allowed_actions": self.game_core.get_allowed_actions(),
            "current_phase_events": self.game_core.get_current_phase_events(),
            "inventory": self
                .game_core
                .get_inventory_snapshot_json()
                .map_err(PlayerGameActorError::from)?,
            "field": self
                .game_core
                .get_field_snapshot_json()
                .map_err(PlayerGameActorError::from)?,
            "selected_event": selected_event,
        }))
    }

    fn push_to_active_socket(&self, message: super::messages::PlayerGameServerMessage) {
        if let Some(socket) = &self.socket {
            socket.do_send(message);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        game::{
            load_balance_actor::{messages::Register, LoadBalanceActor},
            player_game_actor::{
                messages::{
                    AttachSession, DetachSession, ForceDisconnect, PlayerGameServerMessage,
                },
                PlayerGameActor,
            },
        },
        shared::metrics::MetricsCtx,
    };
    use actix::{Actor, Context};
    use actix_web::rt::time;
    use game_core::game::{
        behavior::PlayerBehavior,
        data::{
            abnormality_data::AbnormalityDatabase,
            artifact_data::ArtifactDatabase,
            bonus_data::BonusDatabase,
            equipment_data::EquipmentDatabase,
            event_pools::{EventPhasePool, EventPoolConfig},
            pve_data::PveEncounterDatabase,
            random_event_data::RandomEventDatabase,
            shop_data::ShopDatabase,
            skill_data::SkillDatabase,
            GameDataBase, GameDataBaseParts,
        },
        world::GameCore,
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

    fn empty_game_data() -> Arc<GameDataBase> {
        let empty = EventPhasePool {
            shops: vec![],
            bonuses: vec![],
            random_events: vec![],
        };

        Arc::new(GameDataBase::new(GameDataBaseParts {
            abnormality_data: Arc::new(AbnormalityDatabase::new(vec![])),
            artifact_data: Arc::new(ArtifactDatabase::new(vec![])),
            equipment_data: Arc::new(EquipmentDatabase::new(vec![])),
            shop_data: Arc::new(ShopDatabase::new(vec![])),
            bonus_data: Arc::new(BonusDatabase::new(vec![])),
            random_event_data: Arc::new(RandomEventDatabase::new(vec![])),
            pve_data: Arc::new(PveEncounterDatabase::new(vec![])),
            skill_data: Arc::new(SkillDatabase::new(vec![])),
            event_pools: EventPoolConfig {
                dawn: empty.clone(),
                noon: empty.clone(),
                dusk: empty.clone(),
                midnight: empty.clone(),
                white: empty,
            },
        }))
    }

    #[actix_web::test]
    async fn execute_behavior_returns_updated_state_snapshot() {
        let load_balance = LoadBalanceActor::new(Arc::new(MetricsCtx::new())).start();
        let actor = PlayerGameActor::new(
            Uuid::new_v4(),
            GameCore::new(empty_game_data(), 7),
            load_balance,
        )
        .start();

        let (probe, _, _) = spawn_probe();
        let session_id = Uuid::new_v4();

        let snapshot = actor
            .send(AttachSession {
                session_id,
                socket: probe.clone().recipient(),
                control: probe.recipient(),
            })
            .await
            .expect("attach request should complete")
            .expect("attach should succeed");

        assert_eq!(snapshot["game_state"], "not_started");

        let result = actor
            .send(ExecutePlayerBehavior {
                session_id,
                request_id: "req-1".to_string(),
                behavior: PlayerBehavior::StartNewGame,
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
                assert_eq!(result_type, "StartNewGame");
            }
            other => panic!("expected command_result, got {other:?}"),
        }

        assert_eq!(result.state_snapshot["game_state"], "waiting_phase_request");
    }

    #[actix_web::test]
    async fn attach_replaces_previous_session_immediately() {
        let load_balance = LoadBalanceActor::new(Arc::new(MetricsCtx::new())).start();
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
        let load_balance = LoadBalanceActor::new(Arc::new(MetricsCtx::new())).start();
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
}
