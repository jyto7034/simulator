use super::PlayerBehavior;
use crate::{player_actor::PlayerContext, BehaviorResponse, TestFailure};
use async_trait::async_trait;
use tracing::warn;

/// 매칭 성공 무시 - match_found를 받아도 로딩 단계로 가지 않음
#[derive(Debug, Clone)]
pub struct IgnoreMatchFound;

#[async_trait]
impl PlayerBehavior for IgnoreMatchFound {
    async fn on_match_found(&self, player_context: &PlayerContext) -> BehaviorResponse {
        warn!(
            "[{}] Ignoring match found - staying in queue",
            player_context.player_id
        );
        BehaviorResponse(
            Err(TestFailure::Behavior(
                "Intentionally ignoring match found".to_string(),
            )),
            None,
        ) // 로딩 단계로 가지 않고 종료
    }

    fn clone_trait(&self) -> Box<dyn PlayerBehavior> {
        Box::new(self.clone())
    }
}
