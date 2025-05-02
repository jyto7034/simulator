use actix::Addr;
use uuid::Uuid;

use crate::{card::cards::Cards, game::GameActor};

pub struct ServerState {
    pub game: Addr<GameActor>,
    pub player1_id: Uuid,
    pub player2_id: Uuid,
}

impl ServerState {
    pub async fn reset(&self) {
        todo!()
    }

    pub fn new() -> Self {
        Self {
            game: todo!(),
            player1_id: todo!(),
            player2_id: todo!(),
        }
    }
}

pub trait ValidationPayload {
    fn validate(&self, cards: &Cards) -> Option<()>;
}
