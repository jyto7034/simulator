use actix::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::GameMode;

// --- Client to Server Messages ---

#[derive(Deserialize, Message)]
#[rtype(result = "()")]
#[serde(tag = "type")]
pub enum ClientMessage {
    /// 플레이어가 매칭 대기열에 들어가기를 요청합니다.
    #[serde(rename = "enqueue")]
    Enqueue {
        player_id: Uuid,
        game_mode: GameMode,
        metadata: String,
    },

    /// 플레이어가 매칭 대기열에서 나가기를 요청합니다.
    #[serde(rename = "dequeue")]
    Dequeue {
        player_id: Uuid,
        game_mode: GameMode,
    },
}

// --- Server to Client Messages ---
#[derive(Serialize, Deserialize, Message, Clone)]
#[rtype(result = "()")]
#[serde(tag = "type")]
pub enum ServerMessage {
    /// 대기열에 성공적으로 등록되었음을 알립니다.
    #[serde(rename = "enqueued")]
    EnQueued {
        pod_id: String, // 플레이어가 연결된 Match Server pod ID
    },

    /// 대기열에서 성공적으로 제거되었음을 알립니다.
    #[serde(rename = "dequeued")]
    DeQueued,

    /// 최종적으로 매칭이 성사되었고, 배틀 결과를 함께 전달합니다.
    #[serde(rename = "match_found")]
    MatchFound {
        winner_id: String,
        opponent_id: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        battle_data: Option<serde_json::Value>,
    },

    /// 에러가 발생했음을 알립니다.
    #[serde(rename = "error")]
    Error { code: ErrorCode, message: String },
}

impl ServerMessage {
    pub fn to_string(&self) -> String {
        match &self {
            ServerMessage::EnQueued { .. } => "player.enqueued".to_string(),
            ServerMessage::DeQueued => "player.dequeued".to_string(),
            ServerMessage::MatchFound { .. } => "player.match_found".to_string(),
            ServerMessage::Error { .. } => "player.error".to_string(),
        }
    }
}

#[derive(Serialize, Deserialize, Message, Clone, Debug)]
#[rtype(result = "()")]
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
    InvalidMetadata,
}
