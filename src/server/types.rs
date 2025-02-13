use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

use crate::{card::types::PlayerType, enums::UUID, game::Game};

pub struct ServerState {
    pub game: Mutex<Game>,
    pub player_cookie: SessionKey,
    pub opponent_cookie: SessionKey,
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
