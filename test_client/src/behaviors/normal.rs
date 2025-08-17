use super::PlayerBehavior;
use crate::behaviors::ClientMessage;
use crate::player_actor::message::InternalSendText;
use crate::{player_actor::PlayerContext, BehaviorOutcome};
use crate::{BehaviorResult, TestFailure};
use async_trait::async_trait;
use tracing::{info, warn};
use uuid::Uuid;

/// 정상적인 플레이어 - 모든 단계를 순서대로 완주
#[derive(Debug, Clone)]
pub struct NormalPlayer;

#[async_trait]
impl PlayerBehavior for NormalPlayer {
    /// 매칭 실패 시 처리 - Normal 플레이어는 에러를 받으면 로그를 남기고 테스트 실패로 처리
    async fn on_error(&self, player_context: &PlayerContext, error_msg: &str) -> BehaviorResult {
        warn!(
            "[{}] Normal player received error: {}",
            player_context.player_id, error_msg
        );

        // Normal 플레이어가 에러를 받는 것은 예상치 못한 상황이므로 테스트 실패
        Err(TestFailure::MatchmakingError(format!(
            "Normal player should not receive errors during matchmaking: {}",
            error_msg
        )))
    }

    /// 서버로부터 EnQueued 확인 응답을 받았을 때
    async fn on_enqueued(&self, player_context: &PlayerContext) -> BehaviorResult {
        info!(
            "[{}] Normal player successfully enqueued",
            player_context.player_id
        );
        Ok(BehaviorOutcome::Continue)
    }

    async fn on_match_found(&self, player_context: &PlayerContext) -> BehaviorResult {
        info!(
            "[{}] Normal player excited about match!",
            player_context.player_id
        );
        Ok(BehaviorOutcome::Continue)
    }

    async fn on_loading_start(
        &self,
        player_context: &PlayerContext,
        loading_session_id: Uuid,
    ) -> BehaviorResult {
        info!(
            "[{}] Normal player starting to load assets",
            player_context.player_id
        );

        // loading_complete 메시지 전송
        let msg = ClientMessage::LoadingComplete { loading_session_id };
        player_context
            .addr
            .do_send(InternalSendText(msg.to_string()));

        info!(
            "[{}] Normal player sent loading_complete",
            player_context.player_id
        );

        Ok(BehaviorOutcome::Continue)
    }

    async fn on_loading_complete(&self, player_context: &PlayerContext) -> BehaviorResult {
        info!(
            "[{}] Normal player successfully completed the flow!",
            player_context.player_id
        );
        Ok(BehaviorOutcome::Stop) // 성공적으로 완료했으므로 종료
    }

    fn clone_trait(&self) -> Box<dyn PlayerBehavior> {
        Box::new(self.clone())
    }
}
