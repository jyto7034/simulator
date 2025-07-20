use super::PlayerBehavior;
use crate::{player_actor::PlayerContext, TestFailure, TestResult};
use async_trait::async_trait;
use tracing::warn;

#[derive(Debug, Clone)]
pub struct SuddenDisconnect; // 갑작스런 종료

#[async_trait]
impl PlayerBehavior for SuddenDisconnect {
    async fn on_enqueued(&self, player_context: &PlayerContext) -> TestResult {
        warn!(
            "[{}] Sudden Disconnect - quitting after enqueue confirmed!",
            player_context.player_id
        );

        Err(TestFailure::Behavior(
            "Intentionally disconnected after enqueue".to_string(),
        ))
    }

    fn clone_trait(&self) -> Box<dyn PlayerBehavior> {
        Box::new(self.clone())
    }
}
