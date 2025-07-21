use crate::{
    player_actor::PlayerContext, BehaviorOutcome, BehaviorResponse, TestFailure,
};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tracing::{error, info};
use uuid::Uuid;

pub mod disconnect;
pub mod failure;
pub mod ignore;
pub mod normal;
pub mod quit;
pub mod slow;

// --- 메시지 정의 ---
#[derive(Serialize, Clone)]
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

#[derive(Deserialize, Debug, PartialEq, Clone)]
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
    async fn on_error(&self, player_context: &PlayerContext, error_msg: &str) -> BehaviorResponse {
        error!(
            "[{}] Error occurred: {}",
            player_context.player_id, error_msg
        );
        BehaviorResponse(Err(TestFailure::System(error_msg.to_string())), None)
    }

    /// 1. 큐 진입 확인 - 서버로부터 EnQueued 응답을 받았을 때
    async fn on_enqueued(&self, player_context: &PlayerContext) -> BehaviorResponse {
        info!(
            "[{}] Successfully enqueued - confirmed by server",
            player_context.player_id
        );
        BehaviorResponse(Ok(BehaviorOutcome::Continue), None) // 기본적으로 계속 진행
    }

    /// 2. 로딩 시작 - 상대방 발견 시, 리소스 로딩 시작
    async fn on_loading_start(
        &self,
        player_context: &PlayerContext,
        loading_session_id: Uuid,
    ) -> BehaviorResponse {
        info!(
            "[{}] Loading started - session: {}",
            player_context.player_id, loading_session_id
        );
        BehaviorResponse(Ok(BehaviorOutcome::Continue), None) // 기본적으로 계속 진행
    }

    /// 3. 로딩 완료 - 모든 플레이어의 로딩이 완료된 후
    async fn on_loading_complete(&self, player_context: &PlayerContext) -> BehaviorResponse {
        info!(
            "[{}] Loading complete - ready to start game",
            player_context.player_id
        );
        BehaviorResponse(Ok(BehaviorOutcome::Stop), None) // 기본적으로 테스트 완료 후 종료
    }

    /// 4. 매칭 성공 - 모든 플레이어가 로딩을 완료하고 전용 서버까지 할당 받았을 때.
    async fn on_match_found(&self, player_context: &PlayerContext) -> BehaviorResponse {
        info!(
            "[{}] Match found - opponent discovered!",
            player_context.player_id
        );
        BehaviorResponse(Ok(BehaviorOutcome::Continue), None) // 기본적으로 로딩 단계로 진행
    }

    fn clone_trait(&self) -> Box<dyn PlayerBehavior>;
}

// --- 미구현 행동들 ---
pub struct UnstableConnection; // 간헐적 연결 끊김
pub struct SlowConnection; // 네트워크 지연
pub struct HeartbeatFailure; // 120초 하트비트 실패
pub struct PartialLoader; // 일부만 로딩하고 멈춤
pub struct InvalidMessageSender; // 잘못된 JSON 전송
pub struct WrongStateSender; // 잘못된 상태에서 메시지 전송
pub struct DuplicateEnqueuer; // 중복 큐 참가 시도
pub struct InvalidGameMode; // 존재하지 않는 게임 모드
pub struct Spammer; // 메시지 스팸
pub struct ConnectionFlooder; // 연결 폭탄
pub struct MalformedSender; // 의도적 잘못된 데이터
pub struct ResourceExhauster; // 리소스 고갈 유도
pub struct RaceConditionTester; // 동시성 문제 유발
pub struct StateTransitionAbuser; // 상태 전환 악용
pub struct TimingAttacker; // 타이밍 기반 공격
pub struct CleanupEscaper; // 정리 과정 회피

// --- Behavior Enum Wrapper ---
#[derive(Debug, Clone)]
pub enum BehaviorType {
    Normal(normal::NormalPlayer),
    QuitDuringMatch(quit::QuitDuringMatch),
    QuitDuringLoading(quit::QuitDuringLoading),
    SlowLoader(slow::SlowLoader),
    IgnoreMatchFound(ignore::IgnoreMatchFound),
    SuddenDisconnect(disconnect::SuddenDisconnect),
    LoadingFailure(failure::LoadingFailure),
    LoadingIgnorer(failure::LoadingIgnorer),
}

#[async_trait]
impl PlayerBehavior for BehaviorType {
    async fn on_error(&self, player_context: &PlayerContext, error_msg: &str) -> BehaviorResponse {
        match self {
            BehaviorType::Normal(b) => b.on_error(player_context, error_msg).await,
            BehaviorType::QuitDuringMatch(b) => b.on_error(player_context, error_msg).await,
            BehaviorType::QuitDuringLoading(b) => b.on_error(player_context, error_msg).await,
            BehaviorType::SlowLoader(b) => b.on_error(player_context, error_msg).await,
            BehaviorType::IgnoreMatchFound(b) => b.on_error(player_context, error_msg).await,
            BehaviorType::SuddenDisconnect(b) => b.on_error(player_context, error_msg).await,
            BehaviorType::LoadingFailure(b) => b.on_error(player_context, error_msg).await,
            BehaviorType::LoadingIgnorer(b) => b.on_error(player_context, error_msg).await,
        }
    }

    async fn on_enqueued(&self, player_context: &PlayerContext) -> BehaviorResponse {
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

    async fn on_match_found(&self, player_context: &PlayerContext) -> BehaviorResponse {
        match self {
            BehaviorType::Normal(b) => b.on_match_found(player_context).await,
            BehaviorType::QuitDuringMatch(b) => b.on_match_found(player_context).await,
            BehaviorType::QuitDuringLoading(b) => b.on_match_found(player_context).await,
            BehaviorType::SlowLoader(b) => b.on_match_found(player_context).await,
            BehaviorType::IgnoreMatchFound(b) => b.on_match_found(player_context).await,
            BehaviorType::SuddenDisconnect(b) => b.on_match_found(player_context).await,
            BehaviorType::LoadingFailure(b) => b.on_match_found(player_context).await,
            BehaviorType::LoadingIgnorer(b) => b.on_match_found(player_context).await,
        }
    }

    async fn on_loading_start(
        &self,
        player_context: &PlayerContext,
        loading_session_id: Uuid,
    ) -> BehaviorResponse {
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

    async fn on_loading_complete(&self, player_context: &PlayerContext) -> BehaviorResponse {
        match self {
            BehaviorType::Normal(b) => b.on_loading_complete(player_context).await,
            BehaviorType::QuitDuringMatch(b) => b.on_loading_complete(player_context).await,
            BehaviorType::QuitDuringLoading(b) => b.on_loading_complete(player_context).await,
            BehaviorType::SlowLoader(b) => b.on_loading_complete(player_context).await,
            BehaviorType::IgnoreMatchFound(b) => b.on_loading_complete(player_context).await,
            BehaviorType::SuddenDisconnect(b) => b.on_loading_complete(player_context).await,
            BehaviorType::LoadingFailure(b) => b.on_loading_complete(player_context).await,
            BehaviorType::LoadingIgnorer(b) => b.on_loading_complete(player_context).await,
        }
    }

    fn clone_trait(&self) -> Box<dyn PlayerBehavior> {
        match self {
            BehaviorType::Normal(b) => b.clone_trait(),
            BehaviorType::QuitDuringMatch(b) => b.clone_trait(),
            BehaviorType::QuitDuringLoading(b) => b.clone_trait(),
            BehaviorType::SlowLoader(b) => b.clone_trait(),
            BehaviorType::IgnoreMatchFound(b) => b.clone_trait(),
            BehaviorType::SuddenDisconnect(b) => b.clone_trait(),
            BehaviorType::LoadingFailure(b) => b.clone_trait(),
            BehaviorType::LoadingIgnorer(b) => b.clone_trait(),
        }
    }
}
