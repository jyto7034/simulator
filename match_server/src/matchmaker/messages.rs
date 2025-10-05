use actix::Message;
use uuid::Uuid;

use crate::{env::MatchModeSettings, GameMode};

#[derive(Message)]
#[rtype(result = "()")]
pub struct Enqueue {
    pub player_id: Uuid,
    pub game_mode: GameMode,
    pub metadata: String,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct Dequeue {
    pub player_id: Uuid,
    pub game_mode: GameMode,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct TryMatch {
    pub match_mode_settings: MatchModeSettings,
}
