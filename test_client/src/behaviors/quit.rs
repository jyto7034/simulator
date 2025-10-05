use super::{ErrorCode, PlayerBehavior};
use crate::{player_actor::PlayerContext, BehaviorOutcome, BehaviorResult};
use async_trait::async_trait;
use tracing::info;

/// 큐가 잡히기 전 의도적으로 매칭을 취소(연결 종료)하는 플레이어
#[derive(Debug, Clone)]
pub struct QuitBeforeMatch;

#[async_trait]
impl PlayerBehavior for QuitBeforeMatch {
    /// Enqueued 직후 짧은 지연 후 연결 종료(큐 잡히기 전까지 취소 가능 규칙 반영)
    async fn on_enqueued(&self, player_context: &PlayerContext) -> BehaviorResult {
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
        player_context
            .addr
            .do_send(crate::player_actor::message::InternalClose);
        Ok(BehaviorOutcome::Stop)
    }

    /// 오류는 테스트 실패로 간주하지 않고 계속 진행(서버가 재시도 중일 수 있음)
    async fn on_error(
        &self,
        _player_context: &PlayerContext,
        _error_code: ErrorCode,
        _error_msg: &str,
    ) -> BehaviorResult {
        Ok(BehaviorOutcome::Continue)
    }

    fn clone_trait(&self) -> Box<dyn PlayerBehavior> {
        Box::new(self.clone())
    }
}

/// Enqueue 성공 후 즉시 Dequeue 요청하는 플레이어
#[derive(Debug, Clone)]
pub struct QuitAfterEnqueue;

#[async_trait]
impl PlayerBehavior for QuitAfterEnqueue {
    async fn on_enqueued(&self, player_context: &PlayerContext) -> BehaviorResult {
        info!(
            "[{}] Enqueued, now requesting dequeue...",
            player_context.player_id
        );

        // 짧은 지연 후 Dequeue 메시지 전송
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        let game_mode = crate::default_game_mode();
        let dequeue_msg = crate::behaviors::ClientMessage::Dequeue {
            player_id: player_context.player_id,
            game_mode,
        };

        player_context
            .addr
            .do_send(crate::player_actor::message::InternalSendText(
                dequeue_msg.to_string(),
            ));

        Ok(BehaviorOutcome::Continue)
    }

    async fn on_dequeued(&self, player_context: &PlayerContext) -> BehaviorResult {
        info!(
            "[{}] Successfully dequeued after intentional quit!",
            player_context.player_id
        );
        Ok(BehaviorOutcome::Stop)
    }

    async fn on_error(
        &self,
        _player_context: &PlayerContext,
        _error_code: ErrorCode,
        _error_msg: &str,
    ) -> BehaviorResult {
        Ok(BehaviorOutcome::Continue)
    }

    fn clone_trait(&self) -> Box<dyn PlayerBehavior> {
        Box::new(self.clone())
    }
}
