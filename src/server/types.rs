use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

use crate::{card::types::PlayerType, enums::UUID, game::Game};

pub struct ServerState {
    pub game: Mutex<Game>,
    pub player_cookie: Mutex<GameKey>,
    pub opponent_cookie: Mutex<GameKey>,
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

pub struct GameKey {
    value: String,
}

impl GameKey {
    pub fn new(value: String) -> Self {
        Self { value }
    }
}
#[derive(Debug, Deserialize)]
pub enum ServerGameStep{
    Mulligan,
}