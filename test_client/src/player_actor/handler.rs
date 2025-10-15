use actix::{
    dev::ContextFutureSpawner, fut, ActorContext, ActorFutureExt, AsyncContext, Handler,
    StreamHandler, WrapFuture,
};
use futures_util::SinkExt;
use tokio_tungstenite::tungstenite::Message;
use tracing::{error, info};

use crate::{
    observer_actor::message::PlayerFinished,
    player_actor::{
        message::{
            BehaviorFinished, ConnectionEstablished, InternalClose, InternalSendText, SetState,
        },
        PlayerActor, PlayerContext, PlayerState,
    },
    protocols::ServerMessage,
    BehaviorOutcome,
};

/// Server 에게 msg 를 보내는 핸들러
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

impl Handler<InternalClose> for PlayerActor {
    type Result = ();

    fn handle(&mut self, _msg: InternalClose, ctx: &mut Self::Context) -> Self::Result {
        info!("[{}] Closing WebSocket connection", self.player_id);

        if let Some(mut sink) = self.sink.take() {
            let player_id = self.player_id;
            async move {
                let _ = sink.send(Message::Close(None)).await;
                sink
            }
            .into_actor(self)
            .map(move |sink, actor, ctx| {
                actor.sink = Some(sink);
                info!("[{}] WebSocket close sent, stopping actor", player_id);
                ctx.stop();
            })
            .spawn(ctx);
        } else {
            error!("[{}] Cannot close: sink not available", self.player_id);
            ctx.stop();
        }
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
                Ok(msg) => msg,
                Err(e) => {
                    error!("[{}] Failed to parse server message: {}", self.player_id, e);
                    error!("[{}] raw message: {}", self.player_id, text);

                    panic!(
                        "Player {} failed to parse server message: {}. Raw message: {}",
                        self.player_id, e, text,
                    );
                }
            },
            Ok(Message::Close(reason)) => {
                self.connection_closed = true;
                self.stream = None;
                info!(
                    "[{}] Server closed connection: {:?}",
                    self.player_id, reason
                );
                return;
            }
            Err(e) => {
                if self.connection_closed {
                    return;
                }
                error!("[{}] Websocket connection error: {}", self.player_id, e);
                ctx.stop();
                return;
            }
            _ => return,
        };

        info!("[{}] Received message: {:?}", self.player_id, msg);

        let behavior = self.behaviors.clone_trait();
        let mut player_ctx = PlayerContext {
            player_id: self.player_id,
            pod_id: None,
            addr: ctx.address(),
            test_session_id: self.test_session_id.clone(),
        };

        let msg_fut = msg.clone();

        async move {
            let response = match msg_fut {
                ServerMessage::EnQueued { pod_id } => {
                    player_ctx.pod_id = Some(pod_id);
                    behavior.on_enqueued(&player_ctx).await
                }
                ServerMessage::DeQueued => behavior.on_dequeued(&player_ctx).await,
                ServerMessage::MatchFound => behavior.on_match_found(&player_ctx).await,
                ServerMessage::Error { code, message } => {
                    behavior.on_error(&player_ctx, code, message.as_str()).await
                }
            };

            player_ctx.addr.do_send(BehaviorFinished {
                response,
                original_message: msg,
            });
        }
        .into_actor(self)
        .spawn(ctx);
    }
}

// 플레이어의 Behavior 한 단계가 종료되면 호출되는 핸들러
// BehaviorOutcome 에 따라 다음 행동을 결정함.
impl Handler<BehaviorFinished> for PlayerActor {
    type Result = ();

    fn handle(&mut self, msg: BehaviorFinished, ctx: &mut Self::Context) -> Self::Result {
        match msg.response {
            BehaviorOutcome::Continue => {
                self.update_state_from_message(&msg.original_message);
            }
            outcome @ BehaviorOutcome::Complete
            | outcome @ BehaviorOutcome::Error(_)
            | outcome @ BehaviorOutcome::IntendError => {
                self.finish_with_outcome(outcome, ctx);
            }
        }
    }
}

impl Handler<SetState> for PlayerActor {
    type Result = ();

    fn handle(&mut self, msg: SetState, _ctx: &mut Self::Context) -> Self::Result {
        info!("[{}] New state has been set!", self.player_id);
        self.state = msg.0;
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

        // 연결 직후 behavior 훅 호출
        let behavior = self.behaviors.clone_trait();
        let player_context = PlayerContext {
            player_id: self.player_id,
            pod_id: None,
            addr: ctx.address(),
            test_session_id: self.test_session_id.clone(),
        };

        async move { behavior.on_connected(&player_context).await }
            .into_actor(self)
            .map(|outcome, actor, ctx| match outcome {
                BehaviorOutcome::Continue => {
                    info!("[{}] on_connected completed, continuing", actor.player_id);
                }
                outcome @ BehaviorOutcome::Complete
                | outcome @ BehaviorOutcome::Error(_)
                | outcome @ BehaviorOutcome::IntendError => {
                    actor.finish_with_outcome(outcome, ctx);
                }
            })
            .spawn(ctx);
    }
}

impl PlayerActor {
    fn update_state_from_message(&mut self, message: &ServerMessage) {
        match message {
            ServerMessage::EnQueued { pod_id } => {
                info!("[{}] Stored pod_id: {}", self.player_id, pod_id);
                self.state = PlayerState::Enqueued;
            }
            ServerMessage::DeQueued => {
                info!("[{}] Dequeued", self.player_id);
                self.state = PlayerState::Idle;
            }
            ServerMessage::MatchFound => {
                info!("[{}] Match found", self.player_id);
                self.state = PlayerState::Matched;
            }
            ServerMessage::Error { code, message } => {
                error!(
                    "[{}] Server error: {:?} - {}",
                    self.player_id, code, message
                );
                self.state = PlayerState::Error(message.clone());
            }
        }
    }

    fn finish_with_outcome(&mut self, outcome: BehaviorOutcome, ctx: &mut actix::Context<Self>) {
        let log_msg = match &outcome {
            BehaviorOutcome::Complete => "completed successfully",
            BehaviorOutcome::Error(err) => {
                error!("[{}] Error: {}", self.player_id, err);
                "failed with error"
            }
            BehaviorOutcome::IntendError => "completed with intended error",
            BehaviorOutcome::Continue => unreachable!(),
        };

        info!("[{}] Player {}, stopping actor", self.player_id, log_msg);

        self.observer.do_send(PlayerFinished {
            player_id: self.player_id,
            result: outcome,
        });

        ctx.stop();
    }
}
