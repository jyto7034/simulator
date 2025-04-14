use actix::Addr;

use crate::{card::cards::Cards, game::GameActor};

use super::session::PlayerSessionManager;

pub struct ServerState {
    pub game: Addr<GameActor>,
    pub player_cookie: SessionKey,
    pub opponent_cookie: SessionKey,
    pub session_manager: PlayerSessionManager,
}

impl ServerState {
    pub async fn reset(&self) {
        todo!()
    }

    pub fn new() -> Self {
        Self {
            game: todo!(),
            player_cookie: todo!(),
            opponent_cookie: todo!(),
            session_manager: todo!(),
        }
    }
}

pub trait ValidationPayload {
    fn validate(&self, cards: &Cards) -> Option<()>;
}

pub struct SessionKey {
    key: String,
}

impl SessionKey {
    pub fn new(key: String) -> Self {
        Self { key }
    }

    pub fn get(&self) -> &str {
        &self.key
    }
}
