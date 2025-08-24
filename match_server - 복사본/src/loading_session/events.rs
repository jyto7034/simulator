use actix::prelude::*;
use std::time::Instant;
use uuid::Uuid;

/// Loading session 관련 이벤트들
#[derive(Message, Clone, Debug)]
#[rtype(result = "()")]
pub enum LoadingEvent {
    /// 플레이어가 로딩을 시작했을 때
    PlayerStartedLoading {
        player_id: Uuid,
        session_id: Uuid,
        started_at: Instant,
    },
    /// 플레이어가 로딩을 완료했을 때  
    PlayerCompletedLoading {
        player_id: Uuid,
        session_id: Uuid,
        completed_at: Instant,
    },
    /// 플레이어가 로딩 timeout됐을 때
    PlayerTimeout {
        player_id: Uuid,
        session_id: Uuid,
        timeout_at: Instant,
    },
    /// 세션 전체가 완료됐을 때
    SessionCompleted {
        session_id: Uuid,
        completed_at: Instant,
        players: Vec<Uuid>,
    },
    /// 세션이 취소됐을 때
    SessionCanceled {
        session_id: Uuid,
        canceled_at: Instant,
        reason: String,
    },
}

/// Loading session 생명주기 관리를 위한 메시지들
#[derive(Message)]
#[rtype(result = "Result<(), anyhow::Error>")]
pub struct CreateLoadingSession {
    pub session_id: Uuid,
    pub players: Vec<Uuid>,
    pub game_mode: String,
    pub timeout_seconds: u64,
}

#[derive(Message)]
#[rtype(result = "Result<(), anyhow::Error>")]
pub struct PlayerLoadingComplete {
    pub player_id: Uuid,
    pub session_id: Uuid,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct CheckPlayerTimeout {
    pub player_id: Uuid,
    pub session_id: Uuid,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct CleanupSession {
    pub session_id: Uuid,
}