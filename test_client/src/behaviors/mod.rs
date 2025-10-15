use crate::{player_actor::PlayerContext, protocols::ErrorCode, BehaviorOutcome};
use async_trait::async_trait;

pub mod invalid;
pub mod normal;
pub mod quit;

#[async_trait]
pub trait PlayerBehavior: Send + Sync {
    async fn try_connect(&self, _player: &PlayerContext) -> BehaviorOutcome {
        BehaviorOutcome::Continue
    }

    // 연결 직후 훅(WS 연결/스트림 준비 완료 뒤 호출). 기본은 no-op
    async fn on_connected(&self, _player: &PlayerContext) -> BehaviorOutcome {
        BehaviorOutcome::Continue
    }

    // 에러 수신 시 - 기본 동작은 Error 반환
    async fn on_error(
        &self,
        _player: &PlayerContext,
        code: ErrorCode,
        msg: &str,
    ) -> BehaviorOutcome {
        BehaviorOutcome::Error(format!("server_error: {:?} - {}", code, msg))
    }

    // 대기열 진입 확인
    async fn on_enqueued(&self, _player: &PlayerContext) -> BehaviorOutcome {
        BehaviorOutcome::Continue
    }

    // 대기열 탈출 확인
    async fn on_dequeued(&self, _player: &PlayerContext) -> BehaviorOutcome {
        BehaviorOutcome::Continue
    }

    // 매치 성사 (최종 단계 - 이후 Game Server로 이동)
    async fn on_match_found(&self, _player: &PlayerContext) -> BehaviorOutcome {
        BehaviorOutcome::Complete
    }

    fn clone_trait(&self) -> Box<dyn PlayerBehavior>;
}

// --- Behavior Enum (flattened) ---
#[derive(Debug, Clone)]
pub enum BehaviorType {
    Normal,
    QuitBeforeMatch,
    QuitAfterEnqueue, // Enqueue 성공 후 즉시 Dequeue 또는 종료

    // Invalid Enqueue behaviors
    InvalidEnqueueUnknownType,
    InvalidEnqueueMissingField,
    InvalidEnqueueDuplicate,

    // Invalid Dequeue behaviors
    InvalidDequeueUnknownType,
    InvalidDequeueMissingField,
    InvalidDequeueDuplicate,
    InvalidDequeueWrongPlayerId,
    // Deprecated (Loading phase removed)
    // SlowLoader, SpikyLoader, TimeoutLoader
}

#[async_trait]
impl PlayerBehavior for BehaviorType {
    async fn on_connected(&self, p: &PlayerContext) -> BehaviorOutcome {
        match self {
            BehaviorType::Normal => self::normal::NormalPlayer.on_connected(p).await,
            BehaviorType::QuitBeforeMatch => self::quit::QuitBeforeMatch.on_connected(p).await,
            BehaviorType::QuitAfterEnqueue => self::quit::QuitAfterEnqueue.on_connected(p).await,
            BehaviorType::InvalidEnqueueUnknownType => {
                self::invalid::InvalidEnqueueUnknownType
                    .on_connected(p)
                    .await
            }
            BehaviorType::InvalidEnqueueMissingField => {
                self::invalid::InvalidEnqueueMissingField
                    .on_connected(p)
                    .await
            }
            BehaviorType::InvalidEnqueueDuplicate => {
                self::invalid::InvalidEnqueueDuplicate.on_connected(p).await
            }
            BehaviorType::InvalidDequeueUnknownType => {
                self::invalid::InvalidDequeueUnknownType
                    .on_connected(p)
                    .await
            }
            BehaviorType::InvalidDequeueMissingField => {
                self::invalid::InvalidDequeueMissingField
                    .on_connected(p)
                    .await
            }
            BehaviorType::InvalidDequeueDuplicate => {
                self::invalid::InvalidDequeueDuplicate.on_connected(p).await
            }
            BehaviorType::InvalidDequeueWrongPlayerId => {
                self::invalid::InvalidDequeueWrongPlayerId
                    .on_connected(p)
                    .await
            }
        }
    }

    async fn on_error(&self, p: &PlayerContext, code: ErrorCode, m: &str) -> BehaviorOutcome {
        match self {
            BehaviorType::Normal => self::normal::NormalPlayer.on_error(p, code, m).await,
            BehaviorType::QuitBeforeMatch => self::quit::QuitBeforeMatch.on_error(p, code, m).await,
            BehaviorType::QuitAfterEnqueue => {
                self::quit::QuitAfterEnqueue.on_error(p, code, m).await
            }
            BehaviorType::InvalidEnqueueUnknownType => {
                self::invalid::InvalidEnqueueUnknownType
                    .on_error(p, code, m)
                    .await
            }
            BehaviorType::InvalidEnqueueMissingField => {
                self::invalid::InvalidEnqueueMissingField
                    .on_error(p, code, m)
                    .await
            }
            BehaviorType::InvalidEnqueueDuplicate => {
                self::invalid::InvalidEnqueueDuplicate
                    .on_error(p, code, m)
                    .await
            }
            BehaviorType::InvalidDequeueUnknownType => {
                self::invalid::InvalidDequeueUnknownType
                    .on_error(p, code, m)
                    .await
            }
            BehaviorType::InvalidDequeueMissingField => {
                self::invalid::InvalidDequeueMissingField
                    .on_error(p, code, m)
                    .await
            }
            BehaviorType::InvalidDequeueDuplicate => {
                self::invalid::InvalidDequeueDuplicate
                    .on_error(p, code, m)
                    .await
            }
            BehaviorType::InvalidDequeueWrongPlayerId => {
                self::invalid::InvalidDequeueWrongPlayerId
                    .on_error(p, code, m)
                    .await
            }
        }
    }

    async fn on_enqueued(&self, p: &PlayerContext) -> BehaviorOutcome {
        match self {
            BehaviorType::Normal => self::normal::NormalPlayer.on_enqueued(p).await,
            BehaviorType::QuitBeforeMatch => self::quit::QuitBeforeMatch.on_enqueued(p).await,
            BehaviorType::QuitAfterEnqueue => self::quit::QuitAfterEnqueue.on_enqueued(p).await,
            BehaviorType::InvalidEnqueueUnknownType => {
                self::invalid::InvalidEnqueueUnknownType
                    .on_enqueued(p)
                    .await
            }
            BehaviorType::InvalidEnqueueMissingField => {
                self::invalid::InvalidEnqueueMissingField
                    .on_enqueued(p)
                    .await
            }
            BehaviorType::InvalidEnqueueDuplicate => {
                self::invalid::InvalidEnqueueDuplicate.on_enqueued(p).await
            }
            BehaviorType::InvalidDequeueUnknownType => {
                self::invalid::InvalidDequeueUnknownType
                    .on_enqueued(p)
                    .await
            }
            BehaviorType::InvalidDequeueMissingField => {
                self::invalid::InvalidDequeueMissingField
                    .on_enqueued(p)
                    .await
            }
            BehaviorType::InvalidDequeueDuplicate => {
                self::invalid::InvalidDequeueDuplicate.on_enqueued(p).await
            }
            BehaviorType::InvalidDequeueWrongPlayerId => {
                self::invalid::InvalidDequeueWrongPlayerId
                    .on_enqueued(p)
                    .await
            }
        }
    }

    async fn on_dequeued(&self, p: &PlayerContext) -> BehaviorOutcome {
        match self {
            BehaviorType::Normal => self::normal::NormalPlayer.on_dequeued(p).await,
            BehaviorType::QuitBeforeMatch => self::quit::QuitBeforeMatch.on_dequeued(p).await,
            BehaviorType::QuitAfterEnqueue => self::quit::QuitAfterEnqueue.on_dequeued(p).await,
            BehaviorType::InvalidEnqueueUnknownType => {
                self::invalid::InvalidEnqueueUnknownType
                    .on_dequeued(p)
                    .await
            }
            BehaviorType::InvalidEnqueueMissingField => {
                self::invalid::InvalidEnqueueMissingField
                    .on_dequeued(p)
                    .await
            }
            BehaviorType::InvalidEnqueueDuplicate => {
                self::invalid::InvalidEnqueueDuplicate.on_dequeued(p).await
            }
            BehaviorType::InvalidDequeueUnknownType => {
                self::invalid::InvalidDequeueUnknownType
                    .on_dequeued(p)
                    .await
            }
            BehaviorType::InvalidDequeueMissingField => {
                self::invalid::InvalidDequeueMissingField
                    .on_dequeued(p)
                    .await
            }
            BehaviorType::InvalidDequeueDuplicate => {
                self::invalid::InvalidDequeueDuplicate.on_dequeued(p).await
            }
            BehaviorType::InvalidDequeueWrongPlayerId => {
                self::invalid::InvalidDequeueWrongPlayerId
                    .on_dequeued(p)
                    .await
            }
        }
    }

    async fn on_match_found(&self, p: &PlayerContext) -> BehaviorOutcome {
        match self {
            BehaviorType::Normal => self::normal::NormalPlayer.on_match_found(p).await,
            BehaviorType::QuitBeforeMatch => self::quit::QuitBeforeMatch.on_match_found(p).await,
            BehaviorType::QuitAfterEnqueue => self::quit::QuitAfterEnqueue.on_match_found(p).await,
            BehaviorType::InvalidEnqueueUnknownType => {
                self::invalid::InvalidEnqueueUnknownType
                    .on_match_found(p)
                    .await
            }
            BehaviorType::InvalidEnqueueMissingField => {
                self::invalid::InvalidEnqueueMissingField
                    .on_match_found(p)
                    .await
            }
            BehaviorType::InvalidEnqueueDuplicate => {
                self::invalid::InvalidEnqueueDuplicate
                    .on_match_found(p)
                    .await
            }
            BehaviorType::InvalidDequeueUnknownType => {
                self::invalid::InvalidDequeueUnknownType
                    .on_match_found(p)
                    .await
            }
            BehaviorType::InvalidDequeueMissingField => {
                self::invalid::InvalidDequeueMissingField
                    .on_match_found(p)
                    .await
            }
            BehaviorType::InvalidDequeueDuplicate => {
                self::invalid::InvalidDequeueDuplicate
                    .on_match_found(p)
                    .await
            }
            BehaviorType::InvalidDequeueWrongPlayerId => {
                self::invalid::InvalidDequeueWrongPlayerId
                    .on_match_found(p)
                    .await
            }
        }
    }

    fn clone_trait(&self) -> Box<dyn PlayerBehavior> {
        Box::new(self.clone())
    }
}
