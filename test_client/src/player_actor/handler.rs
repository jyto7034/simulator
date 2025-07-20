use actix::{
    fut, ActorContext, ActorFutureExt, AsyncContext, Handler, MessageResult, StreamHandler,
};
use futures_util::SinkExt;
use tokio_tungstenite::tungstenite::Message;
use tracing::{error, info, warn};

use crate::{
    behaviors::{ClientMessage, ServerMessage},
    player_actor::{
        message::{
            BehaviorFinished, ConnectionEstablished, GetPlayerId, InternalSendText, SendMessage,
            SetState,
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
        // --- 이 부분은 그대로 유지 ---
        let msg = match item {
            Ok(Message::Text(text)) => match serde_json::from_str::<ServerMessage>(&text) {
                Ok(server_msg) => server_msg,
                Err(e) => {
                    warn!("[{}] Failed to parse server message: {}", self.player_id, e);
                    return;
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

        // --- 비동기 작업 준비 (소유권을 이전할 데이터들) ---
        let behavior = self.behavior.clone_trait(); // Arc<T> 복사는 저렴합니다.
        let player_context = PlayerContext {
            player_id: self.player_id,
            addr: ctx.address(), // Addr<T>는 Clone 가능하며 'static 입니다.
        };

        // `msg`도 소유권을 이전해야 하므로 클론합니다.
        let fut_msg = msg.clone();

        // --- 핵심 변경: ctx.wait 대신 actix::spawn 사용 ---
        // 이 비동기 블록은 'static 생명주기를 가지므로, self를 참조할 수 없습니다.
        actix::spawn(async move {
            // PlayerBehavior의 비동기 함수를 호출합니다.
            let result = match fut_msg {
                ServerMessage::EnQueued => behavior.on_enqueued(&player_context).await,
                ServerMessage::StartLoading { loading_session_id } => {
                    behavior
                        .on_loading_start(&player_context, loading_session_id)
                        .await
                }
                ServerMessage::MatchFound { .. } => behavior.on_match_found(&player_context).await,
                ServerMessage::Error { message } => {
                    behavior.on_error(&player_context, &message).await
                }
            };

            // 작업이 끝나면, 그 결과와 원본 메시지를 담아 액터에게 다시 보냅니다.
            player_context.addr.do_send(BehaviorFinished {
                result,
                original_message: msg,
            });
        });
    }
}
impl Handler<BehaviorFinished> for PlayerActor {
    type Result = ();

    fn handle(&mut self, msg: BehaviorFinished, ctx: &mut Self::Context) {
        // 이 핸들러는 동기적이므로, `self`와 `ctx`에 아무런 문제 없이 접근 가능합니다.
        let result = msg.result;
        let original_message = msg.original_message;

        match result {
            Ok(BehaviorOutcome::Continue) => {
                info!("[{}] Continuing with flow", self.player_id);
                // 원본 메시지를 기반으로 상태를 변경합니다.
                match original_message {
                    ServerMessage::EnQueued => self.state = PlayerState::Enqueued,
                    ServerMessage::StartLoading { .. } => self.state = PlayerState::Loading,
                    _ => {}
                }
            }
            Ok(BehaviorOutcome::Stop) => {
                info!(
                    "[{}] Player completed flow, stopping actor.",
                    self.player_id
                );
                ctx.stop();
            }
            Ok(BehaviorOutcome::Retry) => {
                warn!("[{}] Retry requested by behavior", self.player_id);
                // 여기에 재시도 로직을 구현할 수 있습니다.
            }
            Err(test_failure) => {
                error!("[{}] Test failed: {:?}", self.player_id, test_failure);
                ctx.stop();
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

        // 연결이 성공하면 큐에 등록 요청
        let enqueue_msg = ClientMessage::Enqueue {
            player_id: self.player_id,
            game_mode: "normal".to_string(), // TODO: scenario ID 사용
        };

        ctx.address()
            .do_send(InternalSendText(enqueue_msg.to_string()));
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

impl Handler<SetState> for PlayerActor {
    type Result = ();

    fn handle(&mut self, msg: SetState, _ctx: &mut Self::Context) {
        self.state = msg.0;
        info!("[{}] State changed to: {:?}", self.player_id, self.state);
    }
}
