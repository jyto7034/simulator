use super::PlayerBehavior;
use crate::{player_actor::PlayerContext, TestFailure, BehaviorResult, BehaviorOutcome};
use async_trait::async_trait;
use tracing::warn;
use uuid::Uuid;

/// 큐가 잡히기 전 의도적으로 매칭을 취소(연결 종료)하는 플레이어
#[derive(Debug, Clone)]
pub struct QuitBeforeMatch;

#[async_trait]
impl PlayerBehavior for QuitBeforeMatch {
    /// Enqueued 직후 짧은 지연 후 연결 종료(큐 잡히기 전까지 취소 가능 규칙 반영)
    async fn on_enqueued(&self, player_context: &PlayerContext) -> BehaviorResult {
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
        player_context.addr.do_send(crate::player_actor::message::InternalClose);
        Ok(BehaviorOutcome::Stop)
    }

    /// 오류는 테스트 실패로 간주하지 않고 계속 진행(서버가 재시도 중일 수 있음)
    async fn on_error(&self, _player_context: &PlayerContext, _error_msg: &str) -> BehaviorResult {
        Ok(BehaviorOutcome::Continue)
    }

    fn clone_trait(&self) -> Box<dyn PlayerBehavior> {
        Box::new(self.clone())
    }
}

/// 로딩 중 연결 끊는 플레이어 - 로딩 시작되자마자 나가기
#[derive(Debug, Clone)]
pub struct QuitDuringLoading;

#[async_trait]
impl PlayerBehavior for QuitDuringLoading {
    async fn on_loading_start(
        &self,
        player_context: &PlayerContext,
        _loading_session_id: Uuid,
    ) -> BehaviorResult {
        warn!(
            "[{}] Quitting during loading start!",
            player_context.player_id
        );
        Err(TestFailure::Behavior(
            "Intentionally quit during loading".to_string(),
        ))
    }

    fn clone_trait(&self) -> Box<dyn PlayerBehavior> {
        Box::new(self.clone())
    }
}
