use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

use crate::{card::types::PlayerType, enums::UUID, game::Game};

pub struct ServerState{
    pub game: Arc<Mutex<Game>>,
}

impl ServerState {
    pub async fn get_game(&self) -> impl std::ops::Deref<Target = Game> + '_ {
        self.game.lock().await
    }
    pub async fn get_game_mut(&self) -> impl std::ops::DerefMut<Target = Game> + '_ {
        self.game.lock().await
    }
}
#[derive(Serialize, Deserialize)]
pub struct SelectedCard{
    pub uuids: Vec<UUID>
}

#[derive(Serialize, Deserialize)]
pub struct MulliganCards{
    pub uuids: Vec<UUID>
}

/// end point 접근 제어를 위한 struct
/// FromRequest 를 구현함.
pub struct Player{
    pub player_type: PlayerType
}