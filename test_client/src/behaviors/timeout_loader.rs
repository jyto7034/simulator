use super::PlayerBehavior;
use crate::{player_actor::PlayerContext, BehaviorOutcome, BehaviorResult};
use async_trait::async_trait;
use uuid::Uuid;

/// TimeoutLoader: 로딩 시작 후 아무 것도 하지 않아 서버 타임아웃/실패 경로를 유발
#[derive(Debug, Clone)]
pub struct TimeoutLoader;

#[async_trait]
impl PlayerBehavior for TimeoutLoader {
    async fn on_loading_start(
        &self,
        _player_context: &PlayerContext,
        _loading_session_id: Uuid,
    ) -> BehaviorResult {
        // do nothing
        Ok(BehaviorOutcome::Continue)
    }

    fn clone_trait(&self) -> Box<dyn PlayerBehavior> {
        Box::new(self.clone())
    }
}
