use crate::{player_actor::PlayerContext, BehaviorOutcome, BehaviorResult, TestFailure};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub mod invalid;
pub mod normal;
pub mod quit;
pub mod slow;
pub mod spiky_loader;
pub mod timeout_loader;

// --- 메시지 정의 (서버 프로토콜과 1:1 매핑) ---
#[derive(Serialize, Clone)]
#[serde(tag = "type")]
pub enum ClientMessage {
    #[serde(rename = "enqueue")]
    Enqueue { player_id: Uuid, game_mode: String },
    #[serde(rename = "loading_complete")]
    LoadingComplete { loading_session_id: Uuid },
    // 주의: 현재 프로토콜에 "cancel"은 없습니다. (큐 잡히기 전만 클라이언트가 연결을 끊어 취소 가능)
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
    EnQueued,
    #[serde(rename = "start_loading")]
    StartLoading { loading_session_id: Uuid },
    #[serde(rename = "match_found")]
    MatchFound {
        session_id: Uuid,
        server_address: String,
    },
    // error code 넣어야함.
    #[serde(rename = "error")]
    Error { message: String },
}

// --- Behavior 설계 원칙 ---
// - 매칭에는 거절/수락 개념이 없음.
// - 큐가 잡히기 전까지만 취소(=연결 종료) 가능.
// - 큐가 잡히면(StartLoading/MatchFound) 즉시 게임 진입.
// - 따라서 Behavior는 다음 네 가지 축으로 단순화:
//   1) 정상 흐름(Normal)
//   2) 느린 로딩(SlowLoader)
//   3) 큐 잡히기 전 임의 시점 종료(QuitBeforeMatch)
//   4) 매치 성사 무시(Timeout 유도) -> 필요시 유지(여기서는 제외 가능)

#[async_trait]
pub trait PlayerBehavior: Send + Sync {
    // 연결 직후 훅(WS 연결/스트림 준비 완료 뒤 호출). 기본은 no-op
    async fn on_connected(&self, _player: &PlayerContext) -> BehaviorResult {
        Ok(BehaviorOutcome::Continue)
    }

    // 0) 에러 수신 시
    async fn on_error(&self, _player: &PlayerContext, _msg: &str) -> BehaviorResult {
        Err(TestFailure::System("server_error".into()))
    }

    // 1) 대기열 진입 확인
    async fn on_enqueued(&self, _player: &PlayerContext) -> BehaviorResult {
        Ok(BehaviorOutcome::Continue)
    }

    // 2) 로딩 시작(=매치가 성사되어 게임 진입을 준비)
    async fn on_loading_start(
        &self,
        _player: &PlayerContext,
        _loading_session_id: Uuid,
    ) -> BehaviorResult {
        Ok(BehaviorOutcome::Continue)
    }

    // 3) 매치 최종 확정(일부 서버에서는 MatchFound가 먼저/혹은 StartLoading 이후 올 수 있음)
    async fn on_match_found(&self, _player: &PlayerContext) -> BehaviorResult {
        Ok(BehaviorOutcome::Continue)
    }

    // 4) 로딩 완료 후 종료
    async fn on_loading_complete(&self, _player: &PlayerContext) -> BehaviorResult {
        Ok(BehaviorOutcome::Stop)
    }

    fn clone_trait(&self) -> Box<dyn PlayerBehavior>;
}

// --- Behavior Enum (flattened) ---
#[derive(Debug, Clone)]
pub enum BehaviorType {
    Normal,
    SlowLoader { delay_seconds: u64 },
    SpikyLoader { delay_ms: u64 },
    TimeoutLoader,
    QuitBeforeMatch,
    QuitDuringLoading,
    Invalid { mode: invalid::InvalidMode },
}

#[async_trait]
impl PlayerBehavior for BehaviorType {
    async fn on_connected(&self, p: &PlayerContext) -> BehaviorResult {
        match self {
            BehaviorType::Normal => self::normal::NormalPlayer.on_connected(p).await,
            BehaviorType::SlowLoader { delay_seconds } => {
                self::slow::SlowLoader {
                    delay_seconds: *delay_seconds,
                }
                .on_connected(p)
                .await
            }
            BehaviorType::SpikyLoader { delay_ms } => {
                self::spiky_loader::SpikyLoader {
                    delay_ms: *delay_ms,
                }
                .on_connected(p)
                .await
            }
            BehaviorType::TimeoutLoader => {
                self::timeout_loader::TimeoutLoader.on_connected(p).await
            }
            BehaviorType::QuitBeforeMatch => self::quit::QuitBeforeMatch.on_connected(p).await,
            BehaviorType::QuitDuringLoading => self::quit::QuitDuringLoading.on_connected(p).await,
            BehaviorType::Invalid { mode } => {
                self::invalid::InvalidMessages { mode: mode.clone() }
                    .on_connected(p)
                    .await
            }
        }
    }

    async fn on_error(&self, p: &PlayerContext, m: &str) -> BehaviorResult {
        match self {
            BehaviorType::Normal => self::normal::NormalPlayer.on_error(p, m).await,
            BehaviorType::SlowLoader { delay_seconds } => {
                self::slow::SlowLoader {
                    delay_seconds: *delay_seconds,
                }
                .on_error(p, m)
                .await
            }
            BehaviorType::SpikyLoader { delay_ms } => {
                self::spiky_loader::SpikyLoader {
                    delay_ms: *delay_ms,
                }
                .on_error(p, m)
                .await
            }
            BehaviorType::TimeoutLoader => self::timeout_loader::TimeoutLoader.on_error(p, m).await,
            BehaviorType::QuitBeforeMatch => self::quit::QuitBeforeMatch.on_error(p, m).await,
            BehaviorType::QuitDuringLoading => self::quit::QuitDuringLoading.on_error(p, m).await,
            BehaviorType::Invalid { mode } => {
                self::invalid::InvalidMessages { mode: mode.clone() }
                    .on_error(p, m)
                    .await
            }
        }
    }

    async fn on_enqueued(&self, p: &PlayerContext) -> BehaviorResult {
        match self {
            BehaviorType::Normal => self::normal::NormalPlayer.on_enqueued(p).await,
            BehaviorType::SlowLoader { delay_seconds } => {
                self::slow::SlowLoader {
                    delay_seconds: *delay_seconds,
                }
                .on_enqueued(p)
                .await
            }
            BehaviorType::SpikyLoader { delay_ms } => {
                self::spiky_loader::SpikyLoader {
                    delay_ms: *delay_ms,
                }
                .on_enqueued(p)
                .await
            }
            BehaviorType::TimeoutLoader => self::timeout_loader::TimeoutLoader.on_enqueued(p).await,
            BehaviorType::QuitBeforeMatch => self::quit::QuitBeforeMatch.on_enqueued(p).await,
            BehaviorType::QuitDuringLoading => self::quit::QuitDuringLoading.on_enqueued(p).await,
            BehaviorType::Invalid { mode } => {
                self::invalid::InvalidMessages { mode: mode.clone() }
                    .on_enqueued(p)
                    .await
            }
        }
    }

    async fn on_match_found(&self, p: &PlayerContext) -> BehaviorResult {
        match self {
            BehaviorType::Normal => self::normal::NormalPlayer.on_match_found(p).await,
            BehaviorType::SlowLoader { delay_seconds } => {
                self::slow::SlowLoader {
                    delay_seconds: *delay_seconds,
                }
                .on_match_found(p)
                .await
            }
            BehaviorType::SpikyLoader { delay_ms } => {
                self::spiky_loader::SpikyLoader {
                    delay_ms: *delay_ms,
                }
                .on_match_found(p)
                .await
            }
            BehaviorType::TimeoutLoader => {
                self::timeout_loader::TimeoutLoader.on_match_found(p).await
            }
            BehaviorType::QuitBeforeMatch => self::quit::QuitBeforeMatch.on_match_found(p).await,
            BehaviorType::QuitDuringLoading => {
                self::quit::QuitDuringLoading.on_match_found(p).await
            }
            BehaviorType::Invalid { mode } => {
                self::invalid::InvalidMessages { mode: mode.clone() }
                    .on_match_found(p)
                    .await
            }
        }
    }

    async fn on_loading_start(&self, p: &PlayerContext, id: Uuid) -> BehaviorResult {
        match self {
            BehaviorType::Normal => self::normal::NormalPlayer.on_loading_start(p, id).await,
            BehaviorType::SlowLoader { delay_seconds } => {
                self::slow::SlowLoader {
                    delay_seconds: *delay_seconds,
                }
                .on_loading_start(p, id)
                .await
            }
            BehaviorType::SpikyLoader { delay_ms } => {
                self::spiky_loader::SpikyLoader {
                    delay_ms: *delay_ms,
                }
                .on_loading_start(p, id)
                .await
            }
            BehaviorType::TimeoutLoader => {
                self::timeout_loader::TimeoutLoader
                    .on_loading_start(p, id)
                    .await
            }
            BehaviorType::QuitBeforeMatch => {
                self::quit::QuitBeforeMatch.on_loading_start(p, id).await
            }
            BehaviorType::QuitDuringLoading => {
                self::quit::QuitDuringLoading.on_loading_start(p, id).await
            }
            BehaviorType::Invalid { mode } => {
                self::invalid::InvalidMessages { mode: mode.clone() }
                    .on_loading_start(p, id)
                    .await
            }
        }
    }

    async fn on_loading_complete(&self, p: &PlayerContext) -> BehaviorResult {
        match self {
            BehaviorType::Normal => self::normal::NormalPlayer.on_loading_complete(p).await,
            BehaviorType::SlowLoader { delay_seconds } => {
                self::slow::SlowLoader {
                    delay_seconds: *delay_seconds,
                }
                .on_loading_complete(p)
                .await
            }
            BehaviorType::SpikyLoader { delay_ms } => {
                self::spiky_loader::SpikyLoader {
                    delay_ms: *delay_ms,
                }
                .on_loading_complete(p)
                .await
            }
            BehaviorType::TimeoutLoader => {
                self::timeout_loader::TimeoutLoader
                    .on_loading_complete(p)
                    .await
            }
            BehaviorType::QuitBeforeMatch => {
                self::quit::QuitBeforeMatch.on_loading_complete(p).await
            }
            BehaviorType::QuitDuringLoading => {
                self::quit::QuitDuringLoading.on_loading_complete(p).await
            }
            BehaviorType::Invalid { mode } => {
                self::invalid::InvalidMessages { mode: mode.clone() }
                    .on_loading_complete(p)
                    .await
            }
        }
    }

    fn clone_trait(&self) -> Box<dyn PlayerBehavior> {
        Box::new(self.clone())
    }
}
