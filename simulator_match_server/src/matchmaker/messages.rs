use crate::env::GameModeSettings;
use actix::Message;
use uuid::Uuid;

#[derive(Message)]
#[rtype(result = "()")]
pub struct EnqueuePlayer {
    pub player_id: Uuid,
    pub game_mode: String,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct DequeuePlayer {
    pub player_id: Uuid,
    pub game_mode: String,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct HandleLoadingComplete {
    pub player_id: Uuid,
    pub loading_session_id: Uuid,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct CancelLoadingSession {
    pub player_id: Uuid,
    pub loading_session_id: Uuid,
}

#[derive(Message, Clone)]
#[rtype(result = "()")]
pub(super) struct TryMatch {
    pub(super) game_mode: GameModeSettings,
}

/// 오래된 로딩 세션을 정리하기 위한 내부 메시지입니다.
#[derive(Message)]
#[rtype(result = "()")]
pub(super) struct CheckStaleLoadingSessions;
