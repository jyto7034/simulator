use super::PlayerBehavior;
use crate::behaviors::ClientMessage;
use crate::{player_actor::PlayerContext, BehaviorOutcome, BehaviorResponse};
use async_trait::async_trait;
use tracing::info;
use uuid::Uuid;

/// 정상적인 플레이어 - 모든 단계를 순서대로 완주
#[derive(Debug, Clone)]
pub struct NormalPlayer;

#[async_trait]
impl PlayerBehavior for NormalPlayer {
    /// 서버로부터 EnQueued 확인 응답을 받았을 때
    async fn on_enqueued(&self, player_context: &PlayerContext) -> BehaviorResponse {
        info!(
            "[{}] Normal player successfully enqueued",
            player_context.player_id
        );
        BehaviorResponse(Ok(BehaviorOutcome::Continue), None)
    }

    async fn on_match_found(&self, player_context: &PlayerContext) -> BehaviorResponse {
        info!(
            "[{}] Normal player excited about match!",
            player_context.player_id
        );
        BehaviorResponse(Ok(BehaviorOutcome::Continue), None)
    }

    async fn on_loading_start(
        &self,
        player_context: &PlayerContext,
        loading_session_id: Uuid,
    ) -> BehaviorResponse {
        info!(
            "[{}] Normal player starting to load assets",
            player_context.player_id
        );

        // 정상적으로 loading_complete 메시지 전송
        let _msg = ClientMessage::LoadingComplete { loading_session_id };

        // ws_sink
        //     .send(Message::Text(serde_json::to_string(&msg)?))
        //     .await?;

        info!(
            "[{}] Normal player sent loading_complete",
            player_context.player_id
        );
        BehaviorResponse(Ok(BehaviorOutcome::Continue), None)
    }

    async fn on_loading_complete(&self, player_context: &PlayerContext) -> BehaviorResponse {
        info!(
            "[{}] Normal player successfully completed the flow!",
            player_context.player_id
        );
        BehaviorResponse(Ok(BehaviorOutcome::Stop), None) // 성공적으로 완료했으므로 종료
    }

    fn clone_trait(&self) -> Box<dyn PlayerBehavior> {
        Box::new(self.clone())
    }
}
