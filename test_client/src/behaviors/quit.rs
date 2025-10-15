use super::{ErrorCode, PlayerBehavior};
use crate::{
    player_actor::{
        message::{InternalClose, InternalSendText},
        PlayerContext,
    },
    protocols::ClientMessage,
    BehaviorOutcome,
};
use async_trait::async_trait;
use tracing::info;

/// 큐가 잡히기 전 의도적으로 매칭을 취소(연결 종료)하는 플레이어
#[derive(Debug, Clone)]
pub struct QuitBeforeMatch;

#[async_trait]
impl PlayerBehavior for QuitBeforeMatch {
    async fn on_connected(&self, player_context: &PlayerContext) -> BehaviorOutcome {
        info!(
            "[{}] QuitBeforeMatch connected, sending Enqueue",
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

    /// Enqueued 직후 짧은 지연 후 연결 종료(큐 잡히기 전까지 취소 가능 규칙 반영)
    /// TODO: 만약 Enqueue 직후 Match 가 성사되는 케이스 처리 해야함.
    async fn on_enqueued(&self, player_context: &PlayerContext) -> BehaviorOutcome {
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
        player_context.addr.do_send(InternalClose);
        BehaviorOutcome::IntendError
    }

    /// 오류는 테스트 실패로 간주하지 않고 계속 진행(서버가 재시도 중일 수 있음)
    async fn on_error(
        &self,
        _player_context: &PlayerContext,
        _error_code: ErrorCode,
        _error_msg: &str,
    ) -> BehaviorOutcome {
        BehaviorOutcome::Continue
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
    async fn on_connected(&self, player_context: &PlayerContext) -> BehaviorOutcome {
        info!(
            "[{}] QuitAfterEnqueue connected, sending Enqueue",
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

    async fn on_enqueued(&self, player_context: &PlayerContext) -> BehaviorOutcome {
        info!(
            "[{}] Enqueued, now requesting dequeue...",
            player_context.player_id
        );

        // 짧은 지연 후 Dequeue 메시지 전송
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        let game_mode = crate::default_game_mode();
        let dequeue_msg = ClientMessage::Dequeue {
            player_id: player_context.player_id,
            game_mode,
        };

        player_context
            .addr
            .do_send(InternalSendText(dequeue_msg.to_string()));

        BehaviorOutcome::Continue
    }

    async fn on_dequeued(&self, player_context: &PlayerContext) -> BehaviorOutcome {
        info!(
            "[{}] Successfully dequeued after intentional quit!",
            player_context.player_id
        );
        BehaviorOutcome::IntendError
    }

    async fn on_error(
        &self,
        _player_context: &PlayerContext,
        _error_code: ErrorCode,
        _error_msg: &str,
    ) -> BehaviorOutcome {
        BehaviorOutcome::Continue
    }

    fn clone_trait(&self) -> Box<dyn PlayerBehavior> {
        Box::new(self.clone())
    }
}
