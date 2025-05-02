use crate::{card::types::PlayerKind, game::GameActor, player::PlayerActor};

use actix::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{
    connection::ConnectionActor,
    types::{GameStateSnapshot, PlayerInputRequest},
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
#[rtype(result = "()")]
pub struct HandleUserAction {
    pub player_id: Uuid,
    pub action: UserAction,
}

impl Handler<HandleUserAction> for GameActor {
    type Result = ();

    fn handle(&mut self, msg: HandleUserAction, ctx: &mut Self::Context) -> Self::Result {
        match msg.action {
            UserAction::PlayCard { card_id, target_id } => {}
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
                self.restore_card(player_type, &card_id)
                    .expect("Failed to restore card");

                let new_cards = self
                    .get_new_mulligan_cards(player_type, card_id.len())
                    .expect("Failed to get new mulligan cards");
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
