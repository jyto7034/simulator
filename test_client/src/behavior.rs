use crate::{player_actor::PlayerContext, TestFailure, TestResult};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tracing::{error, info, warn};
use uuid::Uuid;

// --- 메시지 정의 ---
#[derive(Serialize)]
#[serde(tag = "type")]
pub enum ClientMessage {
    #[serde(rename = "enqueue")]
    Enqueue { player_id: Uuid, game_mode: String },
    #[serde(rename = "loading_complete")]
    LoadingComplete { loading_session_id: Uuid },
}

impl ClientMessage {
    pub fn to_string(&self) -> String {
        serde_json::to_string(self).unwrap()
    }
}

#[derive(Deserialize, Debug, PartialEq)]
pub enum ServerMessage {
    /// 대기열에 성공적으로 등록되었음을 알립니다.
    #[serde(rename = "enqueued")]
    EnQueued,

    /// 클라이언트에게 에셋 로딩을 시작하라고 지시합니다.
    #[serde(rename = "start_loading")]
    StartLoading { loading_session_id: Uuid },

    /// 최종적으로 매칭이 성사되었고, 게임 서버 접속 정보를 전달합니다.
    #[serde(rename = "match_found")]
    MatchFound {
        session_id: Uuid, // dedicated_server의 게임 세션 ID
        server_address: String,
    },

    /// 에러가 발생했음을 알립니다.
    #[serde(rename = "error")]
    Error { message: String },
}

/// 플레이어 행동을 정의하는 trait
/// 매칭 서버와의 상호작용에서 발생하는 모든 이벤트에 대한 반응을 정의
#[async_trait]
pub trait PlayerBehavior: Send + Sync {
    /// 0. 매칭 실패 시 (모든 단계에서 발생 가능)
    fn on_error(&self, player_context: &PlayerContext, error_msg: &str) -> TestResult {
        error!(
            "[{}] Error occurred: {}",
            player_context.player_id, error_msg
        );
        TestResult::Fail(TestFailure::System(error_msg.to_string()))
    }

    /// 1. 큐 진입 확인 - 서버로부터 EnQueued 응답을 받았을 때
    async fn on_enqueued(&self, player_context: &PlayerContext) -> TestResult {
        info!(
            "[{}] Successfully enqueued - confirmed by server",
            player_context.player_id
        );
        TestResult::Continue // 기본적으로 계속 진행
    }

    /// 2. 로딩 시작 - 상대방 발견 시, 리소스 로딩 시작
    async fn on_loading_start(
        &self,
        player_context: &PlayerContext,
        loading_session_id: Uuid,
    ) -> TestResult {
        info!(
            "[{}] Loading started - session: {}",
            player_context.player_id, loading_session_id
        );
        TestResult::Continue // 기본적으로 계속 진행
    }

    /// 3. 로딩 완료 - 모든 플레이어의 로딩이 완료된 후
    fn on_loading_complete(&self, player_context: &PlayerContext) -> TestResult {
        info!(
            "[{}] Loading complete - ready to start game",
            player_context.player_id
        );
        TestResult::Stop // 기본적으로 테스트 완료 후 종료
    }

    /// 4. 매칭 성공 - 모든 플레이어가 로딩을 완료하고 전용 서버까지 할당 받았을 때.
    fn on_match_found(&self, player_context: &PlayerContext) -> TestResult {
        info!(
            "[{}] Match found - opponent discovered!",
            player_context.player_id
        );
        TestResult::Continue // 기본적으로 로딩 단계로 진행
    }
}

// --- 구체적인 행동 구현들 ---

/// 정상적인 플레이어 - 모든 단계를 순서대로 완주
#[derive(Clone, Debug)]
pub struct NormalPlayer;

#[async_trait]
impl PlayerBehavior for NormalPlayer {
    /// 서버로부터 EnQueued 확인 응답을 받았을 때
    async fn on_enqueued(&self, player_context: &PlayerContext) -> TestResult {
        info!(
            "[{}] Normal player successfully enqueued",
            player_context.player_id
        );
        TestResult::Continue
    }

    fn on_match_found(&self, player_context: &PlayerContext) -> TestResult {
        info!(
            "[{}] Normal player excited about match!",
            player_context.player_id
        );
        TestResult::Continue
    }

    async fn on_loading_start(
        &self,
        player_context: &PlayerContext,
        loading_session_id: Uuid,
    ) -> TestResult {
        info!(
            "[{}] Normal player starting to load assets",
            player_context.player_id
        );

        // 정상적으로 loading_complete 메시지 전송
        let msg = ClientMessage::LoadingComplete { loading_session_id };

        // ws_sink
        //     .send(Message::Text(serde_json::to_string(&msg)?))
        //     .await?;

        info!(
            "[{}] Normal player sent loading_complete",
            player_context.player_id
        );
        TestResult::Continue
    }

    fn on_loading_complete(&self, player_context: &PlayerContext) -> TestResult {
        info!(
            "[{}] Normal player successfully completed the flow!",
            player_context.player_id
        );
        TestResult::Stop // 성공적으로 완료했으므로 종료
    }
}

/// 매칭 중 나가는 플레이어 - 큐에서 기다리다가 포기
#[derive(Clone, Debug)]
pub struct QuitDuringMatch;

#[async_trait]
impl PlayerBehavior for QuitDuringMatch {
    async fn on_enqueued(&self, player_context: &PlayerContext) -> TestResult {
        warn!(
            "[{}] Impatient player - quitting after enqueue confirmed!",
            player_context.player_id
        );

        TestResult::Fail(TestFailure::Behavior(
            "Intentionally quit after enqueue".to_string(),
        ))
    }
}

/// 로딩 중 연결 끊는 플레이어 - 로딩 시작되자마자 나가기
#[derive(Clone, Debug)]
pub struct QuitDuringLoading;

#[async_trait]
impl PlayerBehavior for QuitDuringLoading {
    async fn on_loading_start(
        &self,
        player_context: &PlayerContext,
        _loading_session_id: Uuid,
    ) -> TestResult {
        warn!(
            "[{}] Quitting during loading start!",
            player_context.player_id
        );
        TestResult::Fail(TestFailure::Behavior(
            "Intentionally quit during loading".to_string(),
        ))
    }
}

/// 느린 로더 - 로딩에 오랜 시간이 걸리는 플레이어
#[derive(Clone, Debug)]
pub struct SlowLoader {
    pub delay_seconds: u64,
}

#[async_trait]
impl PlayerBehavior for SlowLoader {
    async fn on_loading_start(
        &self,
        player_context: &PlayerContext,
        loading_session_id: Uuid,
    ) -> TestResult {
        warn!(
            "[{}] Slow loader - waiting {} seconds",
            player_context.player_id, self.delay_seconds
        );

        tokio::time::sleep(tokio::time::Duration::from_secs(self.delay_seconds)).await;

        let msg = ClientMessage::LoadingComplete { loading_session_id };
        info!(
            "[{}] Slow loader finally sent loading_complete",
            player_context.player_id
        );
        TestResult::Continue
    }
}

/// 매칭 성공 무시 - match_found를 받아도 로딩 단계로 가지 않음
#[derive(Clone, Debug)]
pub struct IgnoreMatchFound;

#[async_trait]
impl PlayerBehavior for IgnoreMatchFound {
    fn on_match_found(&self, player_context: &PlayerContext) -> TestResult {
        warn!(
            "[{}] Ignoring match found - staying in queue",
            player_context.player_id
        );
        TestResult::Fail(TestFailure::Behavior(
            "Intentionally ignoring match found".to_string(),
        )) // 로딩 단계로 가지 않고 종료
    }
}

// --- 연결 문제 행동들 ---
pub struct UnstableConnection; // 간헐적 연결 끊김
pub struct SlowConnection; // 네트워크 지연
#[derive(Clone, Debug)]
pub struct SuddenDisconnect; // 갑작스런 종료
pub struct HeartbeatFailure; // 120초 하트비트 실패

#[async_trait]
impl PlayerBehavior for SuddenDisconnect {
    async fn on_enqueued(&self, player_context: &PlayerContext) -> TestResult {
        warn!(
            "[{}] Sudden Disconnect - quitting after enqueue confirmed!",
            player_context.player_id
        );

        TestResult::Fail(TestFailure::Behavior(
            "Intentionally disconnected after enqueue".to_string(),
        ))
    }
}

// --- 로딩 단계 문제 행동들 ---
#[derive(Clone, Debug)]
pub struct LoadingFailure; // 로딩 중 실패 보고

#[async_trait]
impl PlayerBehavior for LoadingFailure {
    async fn on_loading_start(
        &self,
        player_context: &PlayerContext,
        _loading_session_id: Uuid,
    ) -> TestResult {
        error!("[{}] Reporting loading failure!", player_context.player_id);
        TestResult::Fail(TestFailure::Behavior(
            "Intentionally failing loading".to_string(),
        ))
    }
}

#[derive(Clone, Debug)]
pub struct LoadingIgnorer; // 로딩 메시지 무시

#[async_trait]
impl PlayerBehavior for LoadingIgnorer {
    async fn on_loading_start(
        &self,
        player_context: &PlayerContext,
        _loading_session_id: Uuid,
    ) -> TestResult {
        warn!(
            "[{}] Ignoring loading message and doing nothing.",
            player_context.player_id
        );
        // 아무것도 하지 않고 계속 진행하여 서버 타임아웃을 유발
        TestResult::Continue
    }
}

pub struct PartialLoader; // 일부만 로딩하고 멈춤

// --- 프로토콜 위반 행동들 ---
pub struct InvalidMessageSender; // 잘못된 JSON 전송
pub struct WrongStateSender; // 잘못된 상태에서 메시지 전송
pub struct DuplicateEnqueuer; // 중복 큐 참가 시도
pub struct InvalidGameMode; // 존재하지 않는 게임 모드

// --- 악의적/스트레스 테스트 행동들 ---
pub struct Spammer; // 메시지 스팸
pub struct ConnectionFlooder; // 연결 폭탄
pub struct MalformedSender; // 의도적 잘못된 데이터
pub struct ResourceExhauster; // 리소스 고갈 유도

// --- 에지 케이스 행동들 ---
pub struct RaceConditionTester; // 동시성 문제 유발
pub struct StateTransitionAbuser; // 상태 전환 악용
pub struct TimingAttacker; // 타이밍 기반 공격
pub struct CleanupEscaper; // 정리 과정 회피

// --- Behavior Enum Wrapper ---
#[derive(Clone, Debug)]
pub enum BehaviorType {
    Normal(NormalPlayer),
    QuitDuringMatch(QuitDuringMatch),
    QuitDuringLoading(QuitDuringLoading),
    SlowLoader(SlowLoader),
    IgnoreMatchFound(IgnoreMatchFound),
    SuddenDisconnect(SuddenDisconnect),
    LoadingFailure(LoadingFailure),
    LoadingIgnorer(LoadingIgnorer),
}

#[async_trait]
impl PlayerBehavior for BehaviorType {
    fn on_error(&self, player_context: &PlayerContext, error_msg: &str) -> TestResult {
        match self {
            BehaviorType::Normal(b) => b.on_error(player_context, error_msg),
            BehaviorType::QuitDuringMatch(b) => b.on_error(player_context, error_msg),
            BehaviorType::QuitDuringLoading(b) => b.on_error(player_context, error_msg),
            BehaviorType::SlowLoader(b) => b.on_error(player_context, error_msg),
            BehaviorType::IgnoreMatchFound(b) => b.on_error(player_context, error_msg),
            BehaviorType::SuddenDisconnect(b) => b.on_error(player_context, error_msg),
            BehaviorType::LoadingFailure(b) => b.on_error(player_context, error_msg),
            BehaviorType::LoadingIgnorer(b) => b.on_error(player_context, error_msg),
        }
    }

    async fn on_enqueued(&self, player_context: &PlayerContext) -> TestResult {
        match self {
            BehaviorType::Normal(b) => b.on_enqueued(player_context).await,
            BehaviorType::QuitDuringMatch(b) => b.on_enqueued(player_context).await,
            BehaviorType::QuitDuringLoading(b) => b.on_enqueued(player_context).await,
            BehaviorType::SlowLoader(b) => b.on_enqueued(player_context).await,
            BehaviorType::IgnoreMatchFound(b) => b.on_enqueued(player_context).await,
            BehaviorType::SuddenDisconnect(b) => b.on_enqueued(player_context).await,
            BehaviorType::LoadingFailure(b) => b.on_enqueued(player_context).await,
            BehaviorType::LoadingIgnorer(b) => b.on_enqueued(player_context).await,
        }
    }

    fn on_match_found(&self, player_context: &PlayerContext) -> TestResult {
        match self {
            BehaviorType::Normal(b) => b.on_match_found(player_context),
            BehaviorType::QuitDuringMatch(b) => b.on_match_found(player_context),
            BehaviorType::QuitDuringLoading(b) => b.on_match_found(player_context),
            BehaviorType::SlowLoader(b) => b.on_match_found(player_context),
            BehaviorType::IgnoreMatchFound(b) => b.on_match_found(player_context),
            BehaviorType::SuddenDisconnect(b) => b.on_match_found(player_context),
            BehaviorType::LoadingFailure(b) => b.on_match_found(player_context),
            BehaviorType::LoadingIgnorer(b) => b.on_match_found(player_context),
        }
    }

    async fn on_loading_start(
        &self,
        player_context: &PlayerContext,
        loading_session_id: Uuid,
    ) -> TestResult {
        match self {
            BehaviorType::Normal(b) => b.on_loading_start(player_context, loading_session_id).await,
            BehaviorType::QuitDuringMatch(b) => {
                b.on_loading_start(player_context, loading_session_id).await
            }
            BehaviorType::QuitDuringLoading(b) => {
                b.on_loading_start(player_context, loading_session_id).await
            }
            BehaviorType::SlowLoader(b) => {
                b.on_loading_start(player_context, loading_session_id).await
            }
            BehaviorType::IgnoreMatchFound(b) => {
                b.on_loading_start(player_context, loading_session_id).await
            }
            BehaviorType::SuddenDisconnect(b) => {
                b.on_loading_start(player_context, loading_session_id).await
            }
            BehaviorType::LoadingFailure(b) => {
                b.on_loading_start(player_context, loading_session_id).await
            }
            BehaviorType::LoadingIgnorer(b) => {
                b.on_loading_start(player_context, loading_session_id).await
            }
        }
    }

    fn on_loading_complete(&self, player_context: &PlayerContext) -> TestResult {
        match self {
            BehaviorType::Normal(b) => b.on_loading_complete(player_context),
            BehaviorType::QuitDuringMatch(b) => b.on_loading_complete(player_context),
            BehaviorType::QuitDuringLoading(b) => b.on_loading_complete(player_context),
            BehaviorType::SlowLoader(b) => b.on_loading_complete(player_context),
            BehaviorType::IgnoreMatchFound(b) => b.on_loading_complete(player_context),
            BehaviorType::SuddenDisconnect(b) => b.on_loading_complete(player_context),
            BehaviorType::LoadingFailure(b) => b.on_loading_complete(player_context),
            BehaviorType::LoadingIgnorer(b) => b.on_loading_complete(player_context),
        }
    }
}
