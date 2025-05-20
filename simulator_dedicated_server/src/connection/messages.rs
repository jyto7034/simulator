use actix::{fut::wrap_future, prelude::*};
use tracing::{info, warn};

use simulator_core::{exception::GameError, game::message::GameEvent};

use super::{connection::ConnectionActor, ServerMessage};

use actix::Message;

#[derive(Message)]
#[rtype(result = "()")]
pub struct StopActorOnError {
    pub error_message: GameError,
}

impl Handler<StopActorOnError> for ConnectionActor {
    type Result = ();

    fn handle(&mut self, msg: StopActorOnError, ctx: &mut Context<Self>) -> Self::Result {
        warn!(
            "ConnectionActor for player {:?} (session_id: {}): {}",
            self.player_type, self.player_id, msg.error_message
        );
        ctx.stop(); // 여기서 컨텍스트를 통해 액터 중지
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
                            error_message: GameError::CardNotFound,
                        }); // 에러 시 자신에게 중지 요청
                    } else {
                        info!(
                            "Successfully sent Mulligan deal cards directly to player: {}",
                            player_id_log
                        );
                    }
                };
                ctx.spawn(wrap_future::<_, Self>(send_future));
            }
        }
    }
}
