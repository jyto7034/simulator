use actix::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{matchmaker::operations::try_match::PlayerCandidate, GameMode};

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

    /// 최종적으로 매칭이 성사되었고, 게임 서버 접속 정보를 전달합니다.
    #[serde(rename = "match_found")]
    MatchFound {
        session_id: Uuid, // dedicated_server의 게임 세션 ID
        server_address: String,
    },

    /// 에러가 발생했음을 알립니다.
    #[serde(rename = "error")]
    Error { code: ErrorCode, message: String },
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
}

// --- Battle Request Messages ---

/// Match Server가 Game Server로 전송하는 전투 요청
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct BattleRequest {
    pub player1: PlayerCandidate,
    pub player2: PlayerCandidate,
}
