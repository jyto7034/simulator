use super::PlayerBehavior;
use crate::{player_actor::PlayerContext, BehaviorResponse, TestFailure};
use async_trait::async_trait;
use tracing::warn;

#[derive(Debug, Clone)]
pub struct NetworkDisconnect; // 네트워크 오류로 인한 연결 끊김

#[async_trait]
impl PlayerBehavior for NetworkDisconnect {
    async fn on_enqueued(&self, player_context: &PlayerContext) -> BehaviorResponse {
        warn!(
            "[{}] Network Disconnect - simulating network failure after enqueue!",
            player_context.player_id
        );

        BehaviorResponse(
            Err(TestFailure::Behavior(
                "Simulating network failure after enqueue".to_string(),
            )),
            None,
        )
    }

    fn clone_trait(&self) -> Box<dyn PlayerBehavior> {
        Box::new(self.clone())
    }
}
