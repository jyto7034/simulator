use crate::{player_actor::PlayerContext, BehaviorOutcome, BehaviorResult, TestFailure};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub mod invalid;
pub mod normal;
pub mod quit;

// --- 메시지 정의 (서버 프로토콜과 1:1 매핑) ---
#[derive(Serialize, Clone)]
#[serde(tag = "type")]
pub enum ClientMessage {
    #[serde(rename = "enqueue")]
    Enqueue {
        player_id: Uuid,
        game_mode: String, // "Normal" or "Ranked"
        metadata: String,  // JSON: {"player_id": "...", "test_session_id": "...", ...}
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
    MatchFound {
        session_id: Uuid,
        server_address: String,
    },
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

// --- Behavior 설계 원칙 ---
// - 매칭에는 거절/수락 개념이 없음.
// - 큐가 잡히기 전까지만 취소(=Dequeue 또는 연결 종료) 가능.
// - 매칭 성사(MatchFound) 후 Game Server로 이동하여 전투 시작.
// - 따라서 Behavior는 다음과 같이 단순화:
//   1) 정상 흐름(Normal): Enqueue → EnQueued → MatchFound → Stop
//   2) 큐 탈출(Dequeue): Enqueue → EnQueued → Dequeue → DeQueued
//   3) 큐 잡히기 전 종료(QuitBeforeMatch): 연결 종료
//   4) 프로토콜 위반(Invalid): 잘못된 메시지 전송

#[async_trait]
pub trait PlayerBehavior: Send + Sync {
    // 연결 직후 훅(WS 연결/스트림 준비 완료 뒤 호출). 기본은 no-op
    async fn on_connected(&self, _player: &PlayerContext) -> BehaviorResult {
        Ok(BehaviorOutcome::Continue)
    }

    // 에러 수신 시
    async fn on_error(
        &self,
        _player: &PlayerContext,
        _code: ErrorCode,
        _msg: &str,
    ) -> BehaviorResult {
        Err(TestFailure::System(format!(
            "server_error: {:?} - {}",
            _code, _msg
        )))
    }

    // 대기열 진입 확인
    async fn on_enqueued(&self, _player: &PlayerContext) -> BehaviorResult {
        Ok(BehaviorOutcome::Continue)
    }

    // 대기열 탈출 확인
    async fn on_dequeued(&self, _player: &PlayerContext) -> BehaviorResult {
        Ok(BehaviorOutcome::Continue)
    }

    // 매치 성사 (최종 단계 - 이후 Game Server로 이동)
    async fn on_match_found(&self, _player: &PlayerContext) -> BehaviorResult {
        Ok(BehaviorOutcome::Stop)
    }

    fn clone_trait(&self) -> Box<dyn PlayerBehavior>;
}

// --- Behavior Enum (flattened) ---
#[derive(Debug, Clone)]
pub enum BehaviorType {
    Normal,
    QuitBeforeMatch,
    QuitAfterEnqueue, // Enqueue 성공 후 즉시 Dequeue 또는 종료
    Invalid { mode: invalid::InvalidMode },
}

impl BehaviorType {
    /// Returns true if this behavior is expected to fail (receive an error from server)
    pub fn is_expected_to_fail(&self) -> bool {
        matches!(self, BehaviorType::Invalid { .. })
    }
}

#[async_trait]
impl PlayerBehavior for BehaviorType {
    async fn on_connected(&self, p: &PlayerContext) -> BehaviorResult {
        match self {
            BehaviorType::Normal => self::normal::NormalPlayer.on_connected(p).await,
            BehaviorType::QuitBeforeMatch => self::quit::QuitBeforeMatch.on_connected(p).await,
            BehaviorType::QuitAfterEnqueue => self::quit::QuitAfterEnqueue.on_connected(p).await,
            BehaviorType::Invalid { mode } => {
                self::invalid::InvalidMessages { mode: mode.clone() }
                    .on_connected(p)
                    .await
            }
        }
    }

    async fn on_error(&self, p: &PlayerContext, code: ErrorCode, m: &str) -> BehaviorResult {
        match self {
            BehaviorType::Normal => self::normal::NormalPlayer.on_error(p, code, m).await,
            BehaviorType::QuitBeforeMatch => self::quit::QuitBeforeMatch.on_error(p, code, m).await,
            BehaviorType::QuitAfterEnqueue => {
                self::quit::QuitAfterEnqueue.on_error(p, code, m).await
            }
            BehaviorType::Invalid { mode } => {
                self::invalid::InvalidMessages { mode: mode.clone() }
                    .on_error(p, code, m)
                    .await
            }
        }
    }

    async fn on_enqueued(&self, p: &PlayerContext) -> BehaviorResult {
        match self {
            BehaviorType::Normal => self::normal::NormalPlayer.on_enqueued(p).await,
            BehaviorType::QuitBeforeMatch => self::quit::QuitBeforeMatch.on_enqueued(p).await,
            BehaviorType::QuitAfterEnqueue => self::quit::QuitAfterEnqueue.on_enqueued(p).await,
            BehaviorType::Invalid { mode } => {
                self::invalid::InvalidMessages { mode: mode.clone() }
                    .on_enqueued(p)
                    .await
            }
        }
    }

    async fn on_dequeued(&self, p: &PlayerContext) -> BehaviorResult {
        match self {
            BehaviorType::Normal => self::normal::NormalPlayer.on_dequeued(p).await,
            BehaviorType::QuitBeforeMatch => self::quit::QuitBeforeMatch.on_dequeued(p).await,
            BehaviorType::QuitAfterEnqueue => self::quit::QuitAfterEnqueue.on_dequeued(p).await,
            BehaviorType::Invalid { mode } => {
                self::invalid::InvalidMessages { mode: mode.clone() }
                    .on_dequeued(p)
                    .await
            }
        }
    }

    async fn on_match_found(&self, p: &PlayerContext) -> BehaviorResult {
        match self {
            BehaviorType::Normal => self::normal::NormalPlayer.on_match_found(p).await,
            BehaviorType::QuitBeforeMatch => self::quit::QuitBeforeMatch.on_match_found(p).await,
            BehaviorType::QuitAfterEnqueue => self::quit::QuitAfterEnqueue.on_match_found(p).await,
            BehaviorType::Invalid { mode } => {
                self::invalid::InvalidMessages { mode: mode.clone() }
                    .on_match_found(p)
                    .await
            }
        }
    }

    fn clone_trait(&self) -> Box<dyn PlayerBehavior> {
        Box::new(self.clone())
    }
}
