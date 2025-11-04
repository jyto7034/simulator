use super::{ErrorCode, PlayerBehavior};
use crate::{
    player_actor::{message::InternalSendText, PlayerContext},
    protocols::ClientMessage,
    BehaviorOutcome,
};
use async_trait::async_trait;
use tracing::{info, warn};

/// 정상적인 플레이어 - 모든 단계를 순서대로 완주
#[derive(Debug, Clone)]
pub struct NormalPlayer;

#[async_trait]
impl PlayerBehavior for NormalPlayer {
    /// 연결 성공 시 자동으로 Enqueue 메시지 전송
    async fn on_connected(&self, player_context: &PlayerContext) -> BehaviorOutcome {
        info!(
            "[{}] Normal player connected, sending Enqueue",
            player_context.player_id
        );

        let metadata = serde_json::json!({
            "test_session_id": player_context.test_session_id
        })
        .to_string();

        let msg = ClientMessage::Enqueue {
            player_id: player_context.player_id,
            game_mode: crate::default_game_mode(),
            metadata,
        };

        player_context
            .addr
            .do_send(InternalSendText(msg.to_string()));
        BehaviorOutcome::Continue
    }

    /// 매칭 실패 시 처리 - Normal 플레이어는 에러를 받으면 로그를 남기고 처리
    async fn on_error(
        &self,
        player_context: &PlayerContext,
        error_code: ErrorCode,
        error_msg: &str,
    ) -> BehaviorOutcome {
        warn!(
            "[{}] Normal player received error: {:?} - {}",
            player_context.player_id, error_code, error_msg
        );

        // Rate limit나 일시적 에러는 계속 진행 가능
        match error_code {
            ErrorCode::RateLimitExceeded
            | ErrorCode::TemporaryAllocationError
            | ErrorCode::MatchmakingTimeout => {
                info!(
                    "[{}] Temporary error occurred, continuing",
                    player_context.player_id
                );
                return BehaviorOutcome::Continue;
            }
            _ => {}
        }

        // 다른 에러는 에러로 처리
        BehaviorOutcome::Error(format!(
            "Normal player should not receive errors during matchmaking: {:?} - {}",
            error_code, error_msg
        ))
    }

    /// 서버로부터 EnQueued 확인 응답을 받았을 때
    async fn on_enqueued(&self, player_context: &PlayerContext) -> BehaviorOutcome {
        info!(
            "[{}] Normal player successfully enqueued",
            player_context.player_id
        );
        BehaviorOutcome::Continue
    }

    /// 대기열에서 성공적으로 제거됨 (테스트 시나리오에서만 사용)
    async fn on_dequeued(&self, player_context: &PlayerContext) -> BehaviorOutcome {
        info!(
            "[{}] Normal player successfully dequeued",
            player_context.player_id
        );
        BehaviorOutcome::Complete // Dequeue 성공 시 테스트 종료
    }

    /// 매치 성사 - 이제 Game Server로 이동
    async fn on_match_found(&self, player_context: &PlayerContext) -> BehaviorOutcome {
        info!(
            "[{}] Normal player match found! Moving to game server...",
            player_context.player_id
        );
        // Match Server 테스트는 여기서 종료
        BehaviorOutcome::Complete
    }

    fn clone_trait(&self) -> Box<dyn PlayerBehavior> {
        Box::new(self.clone())
    }
}
