use super::PlayerBehavior;
use crate::{player_actor::PlayerContext, BehaviorResponse, TestFailure};
use async_trait::async_trait;
use tracing::warn;
use uuid::Uuid;

/// 매칭 중 나가는 플레이어 - 큐에서 기다리다가 포기
#[derive(Debug, Clone)]
pub struct QuitDuringMatch;

#[async_trait]
impl PlayerBehavior for QuitDuringMatch {
    async fn on_enqueued(&self, player_context: &PlayerContext) -> BehaviorResponse {
        warn!(
            "[{}] Impatient player - quitting after enqueue confirmed!",
            player_context.player_id
        );

        BehaviorResponse(
            Err(TestFailure::Behavior(
                "Intentionally quit after enqueue".to_string(),
            )),
            None,
        )
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
    ) -> BehaviorResponse {
        warn!(
            "[{}] Quitting during loading start!",
            player_context.player_id
        );
        BehaviorResponse(
            Err(TestFailure::Behavior(
                "Intentionally quit during loading".to_string(),
            )),
            None,
        )
    }

    fn clone_trait(&self) -> Box<dyn PlayerBehavior> {
        Box::new(self.clone())
    }
}
