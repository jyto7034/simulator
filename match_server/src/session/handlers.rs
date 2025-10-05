use crate::{
    protocol::{ClientMessage, ErrorCode, ServerMessage},
    session::{
        helper::{send_err, SessionState},
        Session,
    },
    Stop,
};
use actix::{ActorContext, Handler, StreamHandler};
use actix_web_actors::ws::{self, Message, ProtocolError};
use tracing::{info, warn};

impl Handler<Stop> for Session {
    type Result = ();

    fn handle(&mut self, msg: Stop, ctx: &mut Self::Context) -> Self::Result {
        info!(
            "Stop message received in Session actor. Stopping actor. {:?}",
            msg.reason
        );
        ctx.stop();
    }
}

// SubScription, Matchmaker ( 하위 액터 ) 는 Session 을 거쳐서 Client 와 통신함.
impl Handler<ServerMessage> for Session {
    type Result = ();

    fn handle(&mut self, msg: ServerMessage, ctx: &mut Self::Context) -> Self::Result {
        match &msg {
            ServerMessage::EnQueued { pod_id: _ } => {
                // 하위 액터에서 Client 에게 메시지를 보낸다는 것은, Queue 에 성공적으로 등록되었음을 의미함.
                self.transition_to(SessionState::InQueue, ctx);
                if let Ok(json) = serde_json::to_string(&msg) {
                    ctx.text(json);
                } else {
                    warn!("Failed to serialize ServerMessage::EnQueued");
                }
            }
            ServerMessage::DeQueued => {
                self.transition_to(SessionState::Dequeued, ctx);
                if let Ok(json) = serde_json::to_string(&msg) {
                    ctx.text(json);
                } else {
                    warn!("Failed to serialize ServerMessage::DeQueued");
                }
            }
            ServerMessage::MatchFound {
                session_id: _,
                server_address: _,
            } => {
                self.transition_to(SessionState::Completed, ctx);
                if let Ok(json) = serde_json::to_string(&msg) {
                    ctx.text(json);
                } else {
                    warn!("Failed to serialize ServerMessage::MatchFound");
                }
                ctx.close(Some(ws::CloseCode::Normal.into()));
                ctx.stop();
            }
            ServerMessage::Error {
                code: _,
                message: _,
            } => {
                self.transition_to(SessionState::Error, ctx);
                if let Ok(json) = serde_json::to_string(&msg) {
                    ctx.text(json);
                } else {
                    warn!("Failed to serialize ServerMessage::Error");
                }
                ctx.stop();
            }
        }
    }
}

impl StreamHandler<Result<Message, ProtocolError>> for Session {
    fn handle(&mut self, msg: Result<Message, ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Ping(msg)) => {
                ctx.pong(&msg);
            }
            Ok(ws::Message::Pong(_)) => {}
            Ok(ws::Message::Text(text)) => match serde_json::from_str::<ClientMessage>(&text) {
                Ok(ClientMessage::Enqueue {
                    player_id,
                    game_mode,
                    metadata,
                }) => {
                    self.handle_enqueue(ctx, player_id, game_mode, metadata);
                }
                Ok(ClientMessage::Dequeue {
                    player_id,
                    game_mode,
                }) => {
                    self.handle_dequeue(ctx, player_id, game_mode);
                }
                Err(e) => {
                    warn!("Failed to parse client message: {}", e);
                    send_err(
                        ctx,
                        ErrorCode::InvalidMessageFormat,
                        "Invalid message format",
                    );
                }
            },
            Ok(ws::Message::Close(reason)) => {
                ctx.close(reason);
                ctx.stop();
            }
            _ => ctx.stop(),
        }
    }
}
