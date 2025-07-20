use std::sync::Arc;

use actix::{
    fut, ActorContext, ActorFutureExt, AsyncContext, Handler, MessageResult, StreamHandler,
    WrapFuture,
};
use futures_util::{FutureExt, SinkExt};
use tokio_tungstenite::tungstenite::Message;
use tracing::{error, info, warn};

use crate::{
    behavior::{ClientMessage, ServerMessage},
    player_actor::{
        message::{ConnectionEstablished, GetPlayerId, InternalSendText, SendMessage, SetState},
        PlayerActor, PlayerContext, PlayerState,
    },
    TestFailure,
};

impl Handler<GetPlayerId> for PlayerActor {
    type Result = MessageResult<GetPlayerId>;

    fn handle(&mut self, _msg: GetPlayerId, _ctx: &mut Self::Context) -> Self::Result {
        MessageResult(self.player_id)
    }
}

impl Handler<SendMessage> for PlayerActor {
    type Result = ();

    fn handle(&mut self, _msg: SendMessage, _ctx: &mut Self::Context) {
        info!("SendMessage handler called for player {}", self.player_id);
    }
}

impl StreamHandler<Result<Message, tokio_tungstenite::tungstenite::Error>> for PlayerActor {
    fn handle(
        &mut self,
        item: Result<Message, tokio_tungstenite::tungstenite::Error>,
        ctx: &mut Self::Context,
    ) {
        // --- 1. 메시지 파싱 (동기 작업) ---
        let msg = match item {
            Ok(Message::Text(text)) => {
                match serde_json::from_str::<ServerMessage>(&text) {
                    Ok(server_msg) => server_msg,
                    Err(e) => {
                        warn!("[{}] Failed to parse server message: {}", self.player_id, e);
                        return; // 파싱 실패 시 아무것도 안 하고 종료
                    }
                }
            }
            Ok(Message::Close(reason)) => {
                info!(
                    "[{}] Server closed connection: {:?}",
                    self.player_id, reason
                );
                ctx.stop(); // 서버가 연결을 닫으면 액터도 중지
                return;
            }
            Err(e) => {
                error!("[{}] WebSocket connection error: {}", self.player_id, e);
                ctx.stop(); // 프로토콜 에러 발생 시 액터 중지
                return;
            }
            _ => return, // Ping, Pong, Binary 등 다른 메시지는 무시
        };

        info!("[{}] Received message: {:?}", self.player_id, msg);

        let behavior = Arc::clone(&self.behavior);
        let current_state = self.state;
        let self_addr = ctx.address();

        let async_logic = async move {
            let player_id = self_addr.send(GetPlayerId).await.unwrap();
            let player_context = PlayerContext {
                player_id,
                addr: self_addr.clone(),
            };

            match current_state {
                // PlayerState 가 Idle 인 경우 매칭 시작하기 전 상태임.
                // 서버에 EnQueued 메시지를 보내고, 상태를 Enqueued 로 변경합니다.
                PlayerState::Idle => {
                    if let ServerMessage::EnQueued = msg {
                        match behavior.on_queued(&player_context).await {
                            Ok(_) => {
                                info!("[{}] Match started successfully", player_id);
                                return Ok(PlayerState::Enqueued);
                            }
                            Err(_) => {
                                error!("[{}] Failed to start match", player_id);
                                return Err(anyhow::anyhow!("Failed to start match"));
                            }
                        };
                    }
                }
                PlayerState::Enqueued => {
                    if let ServerMessage::MatchFound {
                        session_id,
                        server_address,
                    } = msg
                    {
                        match behavior.on_match_found(&player_context) {
                            true => todo!(),
                            false => todo!(),
                        }
                    }
                }
                PlayerState::Loading => {}
                _ => {}
            }

            Ok(current_state)
        };

        let future_wrapper = fut::wrap_future::<_, Self>(async_logic).map(
            |result: Result<PlayerState, _>, actor, ctx| match result {
                Ok(new_state) => {
                    info!(
                        "[{}] Transitioning from {:?} to {:?}",
                        actor.player_id, actor.state, new_state
                    );
                    actor.state = new_state;
                }
                Err(e) => {
                    error!(
                        "[{}] Behavior action failed: {}. Stopping actor.",
                        actor.player_id, e
                    );
                    ctx.stop();
                }
            },
        );

        ctx.wait(future_wrapper);
    }
}

impl Handler<SetState> for PlayerActor {
    type Result = ();

    fn handle(&mut self, msg: SetState, _ctx: &mut Self::Context) -> Self::Result {
        self.state = msg.0;
    }
}

impl Handler<InternalSendText> for PlayerActor {
    type Result = ();

    // 해당 Handler 는 clone 이 불가능한 sink 를 async closure 내부로 옮겨서 사용하는 방법에 대해 구현함.
    // async closure 내부에는 self 사용이 불가능하기 때문에, take() 를 통해 소유권을 future 내부로 가져온 뒤, ctx.wait() 을 통해 future 가 완료되기 까지 대기함.
    // 모든 future 작업이 완료되면 sink 의 소유권을 다시 Actor 에게 돌려줌.
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
            // TODO: Backoff ?
            error!("Cannot send message: WebScoket sink is not available or already in use.");
        }
    }
}

impl Handler<ConnectionEstablished> for PlayerActor {
    type Result = ();

    fn handle(&mut self, msg: ConnectionEstablished, ctx: &mut Self::Context) -> Self::Result {
        info!("Player [{}] connection established.", self.player_id);
        self.sink = Some(msg.sink);

        ctx.add_stream(msg.stream);

        let msg = ClientMessage::Enqueue {
            player_id: self.player_id,
            game_mode: "Normal_1v1".to_string(),
        };

        ctx.address().do_send(InternalSendText(msg.to_string()));
    }
}
