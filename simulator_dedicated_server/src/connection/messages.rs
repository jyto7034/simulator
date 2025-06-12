use std::io::Error;

use actix::{fut::wrap_future, prelude::*};
use tracing::{info, warn};

use simulator_core::{
    exception::{GameError, GameplayError, SystemError},
    game::msg::GameEvent,
};

use super::{connection::ConnectionActor, ServerMessage};

use actix::Message;

#[derive(Message)]
#[rtype(result = "()")]
pub struct StopActorOnError {
    pub error: GameError,
}

#[derive(Message)]
#[rtype(result = "Result<(), GameError>")]
pub struct CancelHeartbeat;

impl Handler<CancelHeartbeat> for ConnectionActor {
    type Result = Result<(), GameError>;

    fn handle(&mut self, _msg: CancelHeartbeat, ctx: &mut Context<Self>) -> Self::Result {
        info!("Cancelling heartbeat for player: {:?}", self.player_type);
        if let Some(handle) = self.heartbeat_handle {
            if ctx.cancel_future(handle) {
                info!(
                    "Heartbeat cancelled successfully for player: {:?}",
                    self.player_id
                );
                self.heartbeat_handle = None;
                return Ok(());
            } else {
                warn!(
                    "Failed to cancel heartbeat for player: {:?}, handle may not be valid.",
                    self.player_id
                );
                return Err(GameError::System(SystemError::Internal(
                    "Failed to cancel heartbeat, handle may not be valid.".to_string(),
                )));
            }
        }
        return Err(GameError::System(SystemError::Internal(
            "No heartbeat handle to cancel.".to_string(),
        )));
    }
}

impl Handler<StopActorOnError> for ConnectionActor {
    type Result = ();

    fn handle(&mut self, msg: StopActorOnError, ctx: &mut Context<Self>) -> Self::Result {
        info!(
            "Stopping ConnectionActor for player: {:?} due to error: {:?}",
            self.player_id, msg.error
        );
        // HeartBeat 등 리소스는 stopping 에서 처리함.
        ctx.stop();
    }
}

impl Handler<GameEvent> for ConnectionActor {
    type Result = ();
    fn handle(&mut self, msg: GameEvent, ctx: &mut Context<Self>) {
        match msg {
            GameEvent::SendMulliganDealCards { cards } => {
                info!("Received SendMulliganDealCards event, sending directly to client for player: {:?}", self.player_id);
                let data_to_send = ServerMessage::MulliganDealCards {
                    // ServerMessage는 클라이언트와 약속된 포맷
                    player: self.player_type.to_string(), // 또는 self.player_id.to_string()
                    cards: cards,                         // Uuid 리스트
                }
                .to_json(); // JSON 문자열로 변환하는 헬퍼

                let mut session_clone = self.ws_session.clone();
                let player_id_log = self.player_id;
                let actor_addr = ctx.address(); // 에러 시 중지를 위해

                let send_future = async move {
                    if let Err(e) = session_clone.text(data_to_send).await {
                        warn!(
                            "Failed to send Mulligan deal cards directly for player {}: {:?}",
                            player_id_log, e
                        );
                        actor_addr.do_send(StopActorOnError {
                            error: GameError::System(SystemError::Io(Error::new(
                                std::io::ErrorKind::ConnectionRefused,
                                format!("Failed to send Mulligan deal cards: {}", e),
                            ))),
                        });
                    } else {
                        info!(
                            "Successfully sent Mulligan deal cards directly to player: {}",
                            player_id_log
                        );
                    }
                };
                ctx.spawn(wrap_future::<_, Self>(send_future));
            }
            GameEvent::GameStopped => {
                info!(
                    "Game stopped event received, stopping ConnectionActor for player: {:?}",
                    self.player_id
                );
                ctx.stop(); // 게임이 중지되면 액터를 중지
            }
            GameEvent::SyncState { snapshot } => todo!(),
            GameEvent::StateUpdate(_) => todo!(),
        }
    }
}
