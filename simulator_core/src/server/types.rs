use actix::Addr;
use uuid::Uuid;

use crate::{card::cards::Cards, game::GameActor};

pub struct ServerState {
    pub game: Addr<GameActor>,
    pub player1_id: Uuid,
    pub player2_id: Uuid,
}

pub trait ValidationPayload {
    fn validate(&self, cards: &Cards) -> Option<()>;
}
