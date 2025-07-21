use super::PlayerBehavior;
use crate::behaviors::ClientMessage;
use crate::{player_actor::PlayerContext, BehaviorOutcome, BehaviorResponse};
use async_trait::async_trait;
use tracing::{info, warn};
use uuid::Uuid;

/// 느린 로더 - 로딩에 오랜 시간이 걸리는 플레이어
#[derive(Debug, Clone)]
pub struct SlowLoader {
    pub delay_seconds: u64,
}

#[async_trait]
impl PlayerBehavior for SlowLoader {
    async fn on_loading_start(
        &self,
        player_context: &PlayerContext,
        loading_session_id: Uuid,
    ) -> BehaviorResponse {
        warn!(
            "[{}] Slow loader - waiting {} seconds",
            player_context.player_id, self.delay_seconds
        );

        tokio::time::sleep(tokio::time::Duration::from_secs(self.delay_seconds)).await;

        let _msg = ClientMessage::LoadingComplete { loading_session_id };
        info!(
            "[{}] Slow loader finally sent loading_complete",
            player_context.player_id
        );
        BehaviorResponse(Ok(BehaviorOutcome::Continue), None)
    }

    fn clone_trait(&self) -> Box<dyn PlayerBehavior> {
        Box::new(self.clone())
    }
}
