use super::PlayerBehavior;
use crate::{player_actor::PlayerContext, BehaviorOutcome, BehaviorResult, TestFailure};
use async_trait::async_trait;
use tracing::{error, warn};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct LoadingFailure; // 로딩 중 실패 보고

#[async_trait]
impl PlayerBehavior for LoadingFailure {
    async fn on_loading_start(
        &self,
        player_context: &PlayerContext,
        _loading_session_id: Uuid,
    ) -> BehaviorResult {
        error!("[{}] Reporting loading failure!", player_context.player_id);
        Err(TestFailure::Behavior(
            "Intentionally failing loading".to_string(),
        ))
    }

    fn clone_trait(&self) -> Box<dyn PlayerBehavior> {
        Box::new(self.clone())
    }
}

#[derive(Debug, Clone)]
pub struct LoadingIgnorer; // 로딩 메시지 무시

#[async_trait]
impl PlayerBehavior for LoadingIgnorer {
    async fn on_loading_start(
        &self,
        player_context: &PlayerContext,
        _loading_session_id: Uuid,
    ) -> BehaviorResult {
        warn!(
            "[{}] Ignoring loading message and doing nothing.",
            player_context.player_id
        );
        // 아무것도 하지 않고 계속 진행하여 서버 타임아웃을 유발
        Ok(BehaviorOutcome::Continue)
    }

    fn clone_trait(&self) -> Box<dyn PlayerBehavior> {
        Box::new(self.clone())
    }
}
