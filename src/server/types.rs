use tokio::sync::Mutex;

use crate::{card::cards::Cards, game::Game, test::create_server_state};

pub struct ServerState {
    pub game: Mutex<Game>,
    pub player_cookie: SessionKey,
    pub opponent_cookie: SessionKey,
}

impl ServerState {
    pub async fn reset(&self) {
        let state = create_server_state();
        let new_game = {
            let lock = state.game.lock().await;
            lock.clone()
        };
        let mut current_game = self.game.lock().await;
        *current_game = new_game;
    }
}

pub trait ValidationPayload {
    fn validate(&self, cards: &Cards) -> Option<()>;
}   

pub struct SessionKey(pub String);
