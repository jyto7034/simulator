use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

use crate::{card::types::PlayerType, enums::UUID, game::Game, test::create_server_state};

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

#[derive(Serialize, Deserialize)]
pub struct SelectedCard {
    pub uuids: Vec<UUID>,
}

#[derive(Serialize, Deserialize)]
pub struct MulliganCards {
    pub uuids: Vec<UUID>,
}

/// end point 접근 제어를 위한 struct
/// FromRequest 를 구현함.
pub struct Player {
    pub player_type: PlayerType,
}

pub struct SessionKey(pub String);

#[derive(Debug, Deserialize)]
pub enum ServerGameStep {
    Mulligan,
}
