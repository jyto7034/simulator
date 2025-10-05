use super::{ErrorCode, PlayerBehavior};
use crate::{player_actor::PlayerContext, BehaviorOutcome};
use crate::{BehaviorResult, TestFailure};
use async_trait::async_trait;
use tracing::{info, warn};

/// 정상적인 플레이어 - 모든 단계를 순서대로 완주
#[derive(Debug, Clone)]
pub struct NormalPlayer;

#[async_trait]
impl PlayerBehavior for NormalPlayer {
    /// 매칭 실패 시 처리 - Normal 플레이어는 에러를 받으면 로그를 남기고 테스트 실패로 처리
    async fn on_error(
        &self,
        player_context: &PlayerContext,
        error_code: ErrorCode,
        error_msg: &str,
    ) -> BehaviorResult {
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
                return Ok(BehaviorOutcome::Continue);
            }
            _ => {}
        }

        // 다른 에러는 테스트 실패로 처리
        Err(TestFailure::Behavior(format!(
            "Normal player should not receive errors during matchmaking: {:?} - {}",
            error_code, error_msg
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

    /// 대기열에서 성공적으로 제거됨 (테스트 시나리오에서만 사용)
    async fn on_dequeued(&self, player_context: &PlayerContext) -> BehaviorResult {
        info!(
            "[{}] Normal player successfully dequeued",
            player_context.player_id
        );
        Ok(BehaviorOutcome::Stop) // Dequeue 성공 시 테스트 종료
    }

    /// 매치 성사 - 이제 Game Server로 이동
    async fn on_match_found(&self, player_context: &PlayerContext) -> BehaviorResult {
        info!(
            "[{}] Normal player match found! Moving to game server...",
            player_context.player_id
        );
        // Match Server 테스트는 여기서 종료
        Ok(BehaviorOutcome::Stop)
    }

    fn clone_trait(&self) -> Box<dyn PlayerBehavior> {
        Box::new(self.clone())
    }
}
