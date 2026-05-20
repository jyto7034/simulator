use actix::{ActorContext, AsyncContext, Handler};
use serde_json::Value;

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
        let mut snapshot = self
            .game_core
            .get_run_snapshot_json()
            .map_err(PlayerGameActorError::from)?;

        if let Some((winner, timeline)) = self.game_core.get_active_combat_replay() {
            if let Some(root) = snapshot.as_object_mut() {
                if let Some(Value::Object(selected_event)) = root.get_mut("selected_event") {
                    selected_event.insert(
                        "compressed_timeline".to_string(),
                        serde_json::to_value(compress_timeline_payload(winner, &timeline)?)
                            .map_err(|error| {
                                PlayerGameActorError::new("serialization_failed", error.to_string())
                            })?,
                    );
                }
            }
        }

        Ok(snapshot)
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
        data::{employee_data::StarterEmployeeCandidateDatabase, GameDataBase, GameDataBuilder},
        employee::{EmployeeGrade, StarterEmployeeCandidate},
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
        let candidates = (0..5)
            .map(|index| StarterEmployeeCandidate {
                id: format!("candidate_{index}"),
                name: format!("Candidate {index}"),
                grade: EmployeeGrade::Junior,
                role: "Test Role".to_string(),
                background: "Test Background".to_string(),
            })
            .collect();

        GameDataBuilder::empty()
            .with_starter_employee_candidates(StarterEmployeeCandidateDatabase::new(candidates))
            .build_arc()
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

        assert_eq!(
            result.state_snapshot["game_state"],
            "selecting_starter_employees"
        );
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
