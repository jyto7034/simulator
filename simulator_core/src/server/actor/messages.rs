use actix::prelude::*;
use serde::{Deserialize, Serialize};
use tracing::info;
use uuid::Uuid;

use crate::exception::GameError;

use super::{
    connection::ConnectionActor,
    types::{GameStateSnapshot, PlayerInputRequest},
    ServerMessage,
};

#[derive(Message, Serialize, Deserialize, Debug, Clone)]
#[rtype(result = "()")]
pub enum GameEvent {
    GameStateUpdate(GameStateSnapshot),
    RequestPlayerInput(PlayerInputRequest),
    GameOver { winner: Uuid },
}

#[derive(Message, Deserialize, Debug, Clone)]
#[rtype(result = "Result<(), GameError>")]
pub struct SendMulliganDealCards {
    pub cards: Vec<Uuid>,
}
impl Handler<SendMulliganDealCards> for ConnectionActor {
    type Result = ResponseFuture<Result<(), GameError>>;

    fn handle(&mut self, msg: SendMulliganDealCards, _: &mut Self::Context) -> Self::Result {
        info!("Sending Mulligan deal cards to player: {:?}", msg.cards);
        let mut session = self.ws_session.clone();
        let player_type = self.player_type.clone();
        Box::pin(async move {
            match session
                .text(
                    ServerMessage::MulliganDealCards {
                        player: player_type.to_string(),
                        cards: msg.cards.clone(),
                    }
                    .to_json(),
                )
                .await
            {
                Ok(_) => Ok(()),
                Err(e) => Err(GameError::InternalServerError),
            }
        })
    }
}
