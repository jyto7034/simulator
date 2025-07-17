use actix::{fut, Actor, ActorContext, ActorFutureExt, Addr, AsyncContext, Context, StreamHandler};
use actix_web_actors::ws;
use std::sync::Arc;
use tracing::{error, info, warn};
use uuid::Uuid;

use crate::{
    behavior::{PlayerBehavior, ServerMessage},
    player_actor::message::GetPlayerId,
};

pub mod handler;
pub mod message;

#[derive(Debug, Clone, PartialEq, Copy)]
pub enum PlayerState {
    Idle,
    Enqueued,
    Loading,
    Disconnected,
}

#[derive(Clone)]
pub struct PlayerContext {
    pub player_id: Uuid,
    pub addr: Addr<PlayerActor>,
}

// TODO: Test Player 를 Actor 로 모델링해야함.
// 플레이어
pub struct PlayerActor {
    pub state: PlayerState,
    pub behavior: Arc<dyn PlayerBehavior>,
    pub player_id: Uuid,
}

impl Actor for PlayerActor {
    type Context = Context<Self>;

    fn started(&mut self, _ctx: &mut Self::Context) {
        info!("PlayerActor started with state");
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for PlayerActor {
    fn handle(&mut self, item: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        // --- 1. 메시지 파싱 (동기 작업) ---
        let msg = match item {
            Ok(ws::Message::Text(text)) => {
                match serde_json::from_str::<ServerMessage>(&text) {
                    Ok(server_msg) => server_msg,
                    Err(e) => {
                        warn!("[{}] Failed to parse server message: {}", self.player_id, e);
                        return; // 파싱 실패 시 아무것도 안 하고 종료
                    }
                }
            }
            Ok(ws::Message::Close(reason)) => {
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
            let mut player_context = PlayerContext {
                player_id,
                addr: self_addr,
            };

            match current_state {
                PlayerState::Enqueued => {
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
                PlayerState::Loading => {
                    if let ServerMessage::StartLoading { loading_session_id } = msg {
                        let loading_id = loading_session_id;
                        match behavior
                            .on_loading_start(&mut player_context, loading_id)
                            .await
                        {
                            Ok(_) => {
                                info!(
                                    "[{}] Loading started with session ID: {}",
                                    player_id, loading_id
                                );
                                return Ok(PlayerState::Loading);
                            }
                            Err(_) => {
                                error!("[{}] Failed to start loading", player_id);
                                return Err(anyhow::anyhow!("Failed to start loading"));
                            }
                        }
                    }
                }
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

// match msg {
//     Ok(ws::Message::Text(text)) => {
//         match serde_json::from_str::<ServerMessage>(&text.to_string()) {
//             Ok(msg) => match msg {
//                 ServerMessage::EnQueued => {
//                     info!("Player is queued");
//                     self.state = PlayerState::Matching;
//                     if behavior.on_match_found(player_ctx.player_id) {
//                         info!("Match found for player: {}", player_ctx.player_id);
//                         self.state = PlayerState::Loading;
//                     } else {
//                         info!("No match found for player: {}", player_ctx.player_id);
//                     }
//                 }
//                 ServerMessage::StartLoading { loading_session_id } => {
//                     info!(
//                         "Player started loading with session ID: {}",
//                         loading_session_id
//                     );
//                     self.state = PlayerState::Loading;
//                 }
//                 ServerMessage::MatchFound {
//                     session_id,
//                     server_address,
//                 } => {
//                     info!(
//                         "Match found! Session ID: {}, Server Address: {}",
//                         session_id, server_address
//                     );
//                     self.state = PlayerState::Playing;
//                 }
//                 ServerMessage::Error { message } => {
//                     info!("Error received: {}", message);
//                     self.state = PlayerState::Disconnected;
//                 }
//             },
//             Err(_) => todo!(),
//         }
//     }
//     _ => {}
// }
