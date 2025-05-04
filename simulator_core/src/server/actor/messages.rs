use crate::{
    card::types::PlayerKind,
    exception::GameError,
    game::{message::RerollRequestMulliganCard, GameActor},
    player::PlayerActor,
};

use actix::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{
    connection::ConnectionActor,
    types::{GameStateSnapshot, PlayerInputRequest, PlayerInputResponse},
    UserAction,
};

#[derive(Message, Serialize, Deserialize, Debug, Clone)]
#[rtype(result = "()")]
pub enum GameEvent {
    GameStateUpdate(GameStateSnapshot),
    RequestPlayerInput(PlayerInputRequest),
    GameOver { winner: Uuid },
}

#[derive(Message, Deserialize, Debug, Clone)]
#[rtype(result = "Result<PlayerInputResponse, GameError>")]
pub struct HandleUserAction {
    pub player_id: Uuid,
    pub action: UserAction,
}

impl Handler<HandleUserAction> for GameActor {
    type Result = ResponseFuture<Result<PlayerInputResponse, GameError>>;

    fn handle(&mut self, msg: HandleUserAction, ctx: &mut Self::Context) -> Self::Result {
        match msg.action {
            UserAction::PlayCard { card_id, target_id } => {
                todo!()
            }
            UserAction::Attack {
                attacker_id,
                defender_id,
            } => todo!(),
            UserAction::EndTurn => todo!(),
            UserAction::SubmitInput {
                request_id,
                response_data,
            } => todo!(),
            UserAction::RerollRequestMulliganCard { card_id } => {
                let player_type = self.get_player_type_by_uuid(msg.player_id);
                let addr = ctx.address();
                Box::pin(async move {
                    let rerolled_cards = addr
                        .send(RerollRequestMulliganCard {
                            player_type,
                            cards: card_id.clone(),
                        })
                        .await??
                        .iter()
                        .map(|card| card.get_uuid())
                        .collect();

                    Ok(PlayerInputResponse::MulliganRerollAnswer(rerolled_cards))
                })
            }
            UserAction::CompleteMulligan => todo!(),
        }
    }
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct RegisterConnection {
    pub player_id: Uuid,
    pub addr: Addr<ConnectionActor>,
}

impl Handler<RegisterConnection> for GameActor {
    type Result = ();

    fn handle(&mut self, msg: RegisterConnection, _: &mut Self::Context) {
        let player_1_actor_addr = PlayerActor::create(|ctx| {
            let player_actor = PlayerActor::new(PlayerKind::Player1);

            player_actor
        });
    }
}
