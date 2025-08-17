use super::PlayerBehavior;
use crate::behaviors::ClientMessage;
use crate::player_actor::message::InternalSendText;
use crate::{player_actor::PlayerContext, BehaviorOutcome, BehaviorResult};
use async_trait::async_trait;
use uuid::Uuid;

/// SpikyLoader: 로딩 단계에서 가변 지연을 주어 스파이크를 유발
/// - min_delay_ms..=max_delay_ms 범위에서 per-player 결정적 지연을 적용한다(상위에서 주입 필요)
#[derive(Debug, Clone)]
pub struct SpikyLoader {
    pub delay_ms: u64,
}

#[async_trait]
impl PlayerBehavior for SpikyLoader {
    async fn on_loading_start(
        &self,
        player_context: &PlayerContext,
        loading_session_id: Uuid,
    ) -> BehaviorResult {
        tokio::time::sleep(tokio::time::Duration::from_millis(self.delay_ms)).await;
        let msg = ClientMessage::LoadingComplete { loading_session_id };
        player_context.addr.do_send(InternalSendText(msg.to_string()));
        Ok(BehaviorOutcome::Continue)
    }

    fn clone_trait(&self) -> Box<dyn PlayerBehavior> {
        Box::new(self.clone())
    }
}

