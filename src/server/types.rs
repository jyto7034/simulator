use tokio::sync::Mutex;

use crate::{
    card::cards::Cards,
    game::{phase::Phase, Game},
    test::create_server_state,
};

use super::session::PlayerSessionManager;

pub struct ServerState {
    pub game: Mutex<Game>,
    pub player_cookie: SessionKey,
    pub opponent_cookie: SessionKey,
    pub session_manager: PlayerSessionManager,
}

impl ServerState {
    pub async fn reset(&self) {
        let state = create_server_state();
        let mut new_game = {
            let lock = state.game.lock().await;
            lock.clone()
        };
        new_game.get_phase_state_mut().set_phase(Phase::Mulligan);
        let mut current_game = self.game.lock().await;
        *current_game = new_game;
    }
}

pub trait ValidationPayload {
    fn validate(&self, cards: &Cards) -> Option<()>;
}

pub struct SessionKey(pub String);
