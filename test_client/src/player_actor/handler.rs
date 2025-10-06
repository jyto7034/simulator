use actix::{
    fut, ActorContext, ActorFutureExt, AsyncContext, Handler, MessageResult, StreamHandler,
    WrapFuture,
};
use futures_util::SinkExt;
use tokio_tungstenite::tungstenite::Message;
use tracing::{error, info, warn};

use crate::{
    behaviors::{ClientMessage, ServerMessage},
    player_actor::{
        message::{
            BehaviorFinished, ConnectionEstablished, GetPlayerId, InternalClose, InternalSendText,
            SetState, TriggerEnqueueNow,
        },
        PlayerActor, PlayerContext, PlayerState,
    },
    BehaviorOutcome,
};

impl Handler<GetPlayerId> for PlayerActor {
    type Result = MessageResult<GetPlayerId>;

    fn handle(&mut self, _msg: GetPlayerId, _ctx: &mut Self::Context) -> Self::Result {
        MessageResult(self.player_id)
    }
}

impl StreamHandler<Result<Message, tokio_tungstenite::tungstenite::Error>> for PlayerActor {
    fn handle(
        &mut self,
        item: Result<Message, tokio_tungstenite::tungstenite::Error>,
        ctx: &mut Self::Context,
    ) {
        let msg = match item {
            Ok(Message::Text(text)) => match serde_json::from_str::<ServerMessage>(&text) {
                Ok(server_msg) => server_msg,
                Err(e) => {
                    error!("[{}] Failed to parse server message: {}", self.player_id, e);
                    error!("[{}] Raw message: {}", self.player_id, text);

                    // 파싱 에러 시 테스트 실패하도록 panic
                    panic!(
                        "Player {} failed to parse server message: {}. Raw message: {}",
                        self.player_id, e, text
                    );
                }
            },
            Ok(Message::Close(reason)) => {
                info!(
                    "[{}] Server closed connection: {:?}",
                    self.player_id, reason
                );
                ctx.stop();
                return;
            }
            Err(e) => {
                error!("[{}] WebSocket connection error: {}", self.player_id, e);
                ctx.stop();
                return;
            }
            _ => return,
        };

        info!("[{}] Received message: {:?}", self.player_id, msg);

        let behavior = self.behavior.clone_trait();
        let player_context = PlayerContext {
            player_id: self.player_id,
            pod_id: "".to_string(),
            addr: ctx.address(),
        };

        let fut_msg = msg.clone();

        actix::spawn(async move {
            let response = match &fut_msg {
                ServerMessage::EnQueued { pod_id } => {
                    // Update player_context with pod_id
                    let mut ctx = player_context.clone();
                    ctx.pod_id = pod_id.clone();
                    behavior.on_enqueued(&ctx).await
                }
                ServerMessage::DeQueued => behavior.on_dequeued(&player_context).await,
                ServerMessage::MatchFound { .. } => behavior.on_match_found(&player_context).await,
                ServerMessage::Error { code, message } => {
                    behavior.on_error(&player_context, *code, message).await
                }
            };

            player_context.addr.do_send(BehaviorFinished {
                response,
                original_message: msg,
            });
        });
    }
}

impl Handler<BehaviorFinished> for PlayerActor {
    type Result = ();

    fn handle(&mut self, msg: BehaviorFinished, ctx: &mut Self::Context) {
        let result = msg.response;
        let original_message = msg.original_message;

        match result {
            Ok(BehaviorOutcome::Continue) => {
                info!("[{}] Continuing with flow", self.player_id);
                match &original_message {
                    ServerMessage::EnQueued { pod_id } => {
                        info!("[{}] Stored pod_id: {}", self.player_id, pod_id);
                        self.state = PlayerState::Enqueued;
                    }
                    ServerMessage::DeQueued => self.state = PlayerState::Idle,
                    ServerMessage::MatchFound { .. } => self.state = PlayerState::Matched,
                    _ => {}
                }
            }
            Ok(BehaviorOutcome::Stop) => {
                info!(
                    "[{}] Player completed flow, stopping actor.",
                    self.player_id
                );
                // Notify Observer that this player finished, so overall scenario can close
                self.observer
                    .do_send(crate::observer_actor::message::PlayerFinishedFromActor {
                        player_id: self.player_id,
                        result: Ok(BehaviorOutcome::Stop),
                    });
                ctx.stop();
            }
            Ok(BehaviorOutcome::Retry) => {
                warn!("[{}] Retry requested by behavior", self.player_id);
            }
            Err(test_failure) => {
                match test_failure {
                    // 의도한 실패
                    crate::TestFailure::Behavior(_) => {
                        warn!("[{}] Behavior failure: {:?}", self.player_id, test_failure);
                    }
                    _ => {
                        error!("[{}] Test failed: {:?}", self.player_id, test_failure);
                        // Notify Observer to allow scenario to conclude even on error branches
                        self.observer.do_send(
                            crate::observer_actor::message::PlayerFinishedFromActor {
                                player_id: self.player_id,
                                result: Ok(BehaviorOutcome::Stop),
                            },
                        );
                        ctx.stop();
                    }
                }
            }
        }
    }
}

impl Handler<ConnectionEstablished> for PlayerActor {
    type Result = ();

    fn handle(&mut self, msg: ConnectionEstablished, ctx: &mut Self::Context) {
        info!("[{}] Connection established", self.player_id);

        self.sink = Some(msg.sink);
        self.stream = Some(msg.stream);

        if let Some(stream) = self.stream.take() {
            ctx.add_stream(stream);
        }
        // 연결 직후 behavior 훅 호출(자동 Enqueue 사용 시에도 no-op로 계속)
        {
            let behavior = self.behavior.clone_trait();
            let ctx_addr = ctx.address();
            let player_id = self.player_id;
            actix::spawn(async move {
                let _ = behavior
                    .on_connected(&crate::player_actor::PlayerContext {
                        player_id,
                        pod_id: "".to_string(),
                        addr: ctx_addr,
                    })
                    .await;
            });
        }

        // 자동 Enqueue 설정일 때만 전송
        if self.auto_enqueue {
            let enqueue_msg = ClientMessage::Enqueue {
                player_id: self.player_id,
                game_mode: crate::default_game_mode(),
                metadata: format!(
                    r#"{{"player_id":"{}","test_session_id":"{}"}}"#,
                    self.player_id, self.test_session_id
                ),
            };
            ctx.address()
                .do_send(InternalSendText(enqueue_msg.to_string()));
        }
    }
}

impl Handler<TriggerEnqueueNow> for PlayerActor {
    type Result = ();
    fn handle(&mut self, _msg: TriggerEnqueueNow, ctx: &mut Self::Context) {
        if let Some(_) = self.sink {
            // sink가 있어야만 가능
            let enqueue_msg = ClientMessage::Enqueue {
                player_id: self.player_id,
                game_mode: crate::default_game_mode(),
                metadata: format!(
                    r#"{{"player_id":"{}","test_session_id":"{}"}}"#,
                    self.player_id, self.test_session_id
                ),
            };
            ctx.address()
                .do_send(InternalSendText(enqueue_msg.to_string()));
        }
    }
}

impl Handler<InternalSendText> for PlayerActor {
    type Result = ();

    fn handle(&mut self, msg: InternalSendText, ctx: &mut Self::Context) {
        if let Some(mut sink) = self.sink.take() {
            let send_future = async move {
                let result = sink.send(Message::Text(msg.0)).await;
                (result, sink)
            };

            let actor_future =
                fut::wrap_future::<_, Self>(send_future).map(|(result, sink), actor, ctx| {
                    if let Err(e) = result {
                        error!("WebSocket sink failed: {}. Connection will be closed.", e);
                        ctx.stop();
                    } else {
                        info!("Message sent Successfully.");
                        actor.sink = Some(sink);
                    }
                });
            ctx.wait(actor_future);
        } else {
            error!("Cannot send message: WebSocket sink is not available or already in use.");
        }
    }
}

impl actix::Handler<InternalClose> for PlayerActor {
    type Result = ();
    fn handle(&mut self, _msg: InternalClose, ctx: &mut Self::Context) {
        if let Some(mut sink) = self.sink.take() {
            let fut = async move {
                let _ = sink.send(Message::Close(None)).await;
                sink
            };
            let fut = fut.into_actor(self).map(|_sink, _, ctx| {
                ctx.stop();
            });
            ctx.spawn(fut);
        } else {
            ctx.stop();
        }
    }
}

impl Handler<SetState> for PlayerActor {
    type Result = ();

    fn handle(&mut self, msg: SetState, _ctx: &mut Self::Context) {
        self.state = msg.0;
        info!("[{}] State changed to: {:?}", self.player_id, self.state);
    }
}
