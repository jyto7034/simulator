// messages.rs
use actix::Message;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// 플레이어 정보 (매칭 요청 시 전달)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerInfo {
    pub player_id: Uuid,
    pub mmr: i32,
    pub game_mode: String,
}

// 매칭 요청 메시지
#[derive(Message, Debug)]
#[rtype(result = "Result<(), MatchmakingError>")] // 성공 또는 에러 반환
pub struct JoinQueue {
    // pub player: PlayerInfo,
}

// 매칭 취소 메시지
#[derive(Message, Debug)]
#[rtype(result = "Result<(), MatchmakingError>")]
pub struct LeaveQueue {
    pub player_id: Uuid,
    pub game_mode: String,
}

// 매칭 로직을 주기적으로 실행하기 위한 내부 메시지
// 분산락의 도입으로 Tick 이 실패
#[derive(Message, Debug)]
#[rtype(result = "Result<(), MatchmakingError>")]
pub struct Tick;

// 매칭 성공 시 다른 Actor (예: GameSessionManagerActor)에게 보낼 메시지
#[derive(Message, Debug, Clone)]
#[rtype(result = "()")]
pub struct MatchFound {
    pub game_id: Uuid,
    // pub players: Vec<PlayerInfo>,
}

// 에러 타입
#[derive(Debug, thiserror::Error)]
pub enum MatchmakingError {
    #[error("Redis error: {0}")]
    RedisError(#[from] redis::RedisError),
    #[error("Player not found in queue")]
    PlayerNotFound,
    #[error("Internal server error: {0}")]
    InternalError(String),
    #[error("Failed to acquire lock")]
    LockError,
}
