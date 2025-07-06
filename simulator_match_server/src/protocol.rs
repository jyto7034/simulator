use actix::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// --- Client to Server Messages ---

#[derive(Deserialize, Message)]
#[rtype(result = "()")]
#[serde(tag = "type")]
pub enum ClientMessage {
    #[serde(rename = "enqueue")]
    Enqueue {
        // JWT should be sent in an auth step, but for now, we'll pass ID directly
        player_id: Uuid,
        game_mode: String,
    },
}

// --- Server to Client Messages ---

#[derive(Serialize, Deserialize, Message, Clone)]
#[rtype(result = "()")]
#[serde(tag = "type")]
pub enum ServerMessage {
    #[serde(rename = "match_found")]
    MatchFound {
        session_id: Uuid,
        server_address: String,
    },
    #[serde(rename = "error")]
    Error {
        message: String,
    },
    #[serde(rename = "queued")]
    Queued,
}
