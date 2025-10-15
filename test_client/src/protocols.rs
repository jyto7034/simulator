use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Clone)]
#[serde(tag = "type")]
pub enum ClientMessage {
    #[serde(rename = "enqueue")]
    Enqueue {
        player_id: Uuid,
        game_mode: String,
        metadata: String,
    },
    #[serde(rename = "dequeue")]
    Dequeue { player_id: Uuid, game_mode: String },
}

impl ClientMessage {
    pub fn to_string(&self) -> String {
        serde_json::to_string(self).unwrap()
    }
}

#[derive(Deserialize, Debug, PartialEq, Clone)]
#[serde(tag = "type")]
pub enum ServerMessage {
    #[serde(rename = "enqueued")]
    EnQueued { pod_id: String },
    #[serde(rename = "dequeued")]
    DeQueued,
    #[serde(rename = "match_found")]
    MatchFound,
    #[serde(rename = "error")]
    Error { code: ErrorCode, message: String },
}

#[derive(Deserialize, Debug, PartialEq, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub enum ErrorCode {
    InvalidGameMode,
    AlreadyInQueue,
    InternalError,
    NotInQueue,
    InvalidMessageFormat,
    WrongSessionId,
    TemporaryAllocationError,
    DedicatedServerTimeout,
    DedicatedServerErrorResponse,
    MaxRetriesExceeded,
    MatchmakingTimeout,
    PlayerTemporarilyBlocked,
    RateLimitExceeded,
}
