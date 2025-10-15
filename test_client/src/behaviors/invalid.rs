use async_trait::async_trait;

use crate::{
    player_actor::{message::InternalSendText, PlayerContext},
    protocols::ClientMessage,
    BehaviorOutcome,
};

use super::PlayerBehavior;

// ============================================================
// Invalid Enqueue Behaviors
// ============================================================

/// 존재하지 않는 타입의 메시지 전송
#[derive(Debug, Clone)]
pub struct InvalidEnqueueUnknownType;

#[async_trait]
impl PlayerBehavior for InvalidEnqueueUnknownType {
    async fn on_connected(&self, ctx: &PlayerContext) -> BehaviorOutcome {
        // 먼저 정상 Enqueue를 보내야 on_enqueued()가 호출됨
        let metadata = serde_json::json!({
            "test_session_id": ctx.test_session_id
        })
        .to_string();

        let msg = ClientMessage::Enqueue {
            player_id: ctx.player_id,
            game_mode: crate::default_game_mode(),
            metadata,
        };
        ctx.addr.do_send(InternalSendText(msg.to_string()));
        BehaviorOutcome::Continue
    }

    async fn on_enqueued(&self, ctx: &PlayerContext) -> BehaviorOutcome {
        ctx.addr
            .do_send(InternalSendText("{\"type\":\"bad_type\"}".to_string()));
        BehaviorOutcome::IntendError
    }

    fn clone_trait(&self) -> Box<dyn PlayerBehavior> {
        Box::new(self.clone())
    }
}

/// 필수 필드 누락된 메시지 전송
#[derive(Debug, Clone)]
pub struct InvalidEnqueueMissingField;

#[async_trait]
impl PlayerBehavior for InvalidEnqueueMissingField {
    async fn on_connected(&self, ctx: &PlayerContext) -> BehaviorOutcome {
        let metadata = serde_json::json!({
            "test_session_id": ctx.test_session_id
        })
        .to_string();

        let msg = ClientMessage::Enqueue {
            player_id: ctx.player_id,
            game_mode: crate::default_game_mode(),
            metadata,
        };
        ctx.addr.do_send(InternalSendText(msg.to_string()));
        BehaviorOutcome::Continue
    }

    async fn on_enqueued(&self, ctx: &PlayerContext) -> BehaviorOutcome {
        // player_id 누락
        ctx.addr
            .do_send(InternalSendText("{\"type\":\"enqueue\"}".to_string()));
        BehaviorOutcome::IntendError
    }

    fn clone_trait(&self) -> Box<dyn PlayerBehavior> {
        Box::new(self.clone())
    }
}

/// EnQueued 이후 중복 Enqueue 시도
#[derive(Debug, Clone)]
pub struct InvalidEnqueueDuplicate;

#[async_trait]
impl PlayerBehavior for InvalidEnqueueDuplicate {
    async fn on_connected(&self, ctx: &PlayerContext) -> BehaviorOutcome {
        let metadata = serde_json::json!({
            "test_session_id": ctx.test_session_id
        })
        .to_string();

        let msg = ClientMessage::Enqueue {
            player_id: ctx.player_id,
            game_mode: crate::default_game_mode(),
            metadata,
        };
        ctx.addr.do_send(InternalSendText(msg.to_string()));
        BehaviorOutcome::Continue
    }

    async fn on_enqueued(&self, ctx: &PlayerContext) -> BehaviorOutcome {
        // 중복 Enqueue 시도
        let metadata = serde_json::json!({
            "test_session_id": ctx.test_session_id
        })
        .to_string();

        let msg = ClientMessage::Enqueue {
            player_id: ctx.player_id,
            game_mode: crate::default_game_mode(),
            metadata,
        };
        ctx.addr.do_send(InternalSendText(msg.to_string()));
        BehaviorOutcome::IntendError
    }

    fn clone_trait(&self) -> Box<dyn PlayerBehavior> {
        Box::new(self.clone())
    }
}

// ============================================================
// Invalid Dequeue Behaviors
// ============================================================

/// Dequeue 시 존재하지 않는 타입 전송
#[derive(Debug, Clone)]
pub struct InvalidDequeueUnknownType;

#[async_trait]
impl PlayerBehavior for InvalidDequeueUnknownType {
    async fn on_connected(&self, ctx: &PlayerContext) -> BehaviorOutcome {
        let metadata = serde_json::json!({
            "test_session_id": ctx.test_session_id
        })
        .to_string();

        let msg = ClientMessage::Enqueue {
            player_id: ctx.player_id,
            game_mode: crate::default_game_mode(),
            metadata,
        };
        ctx.addr.do_send(InternalSendText(msg.to_string()));
        BehaviorOutcome::Continue
    }

    async fn on_enqueued(&self, ctx: &PlayerContext) -> BehaviorOutcome {
        // Enqueued 후 잘못된 타입의 dequeue 메시지
        ctx.addr
            .do_send(InternalSendText("{\"type\":\"bad_dequeue\"}".to_string()));
        BehaviorOutcome::IntendError
    }

    fn clone_trait(&self) -> Box<dyn PlayerBehavior> {
        Box::new(self.clone())
    }
}

/// Dequeue 시 필수 필드 누락
#[derive(Debug, Clone)]
pub struct InvalidDequeueMissingField;

#[async_trait]
impl PlayerBehavior for InvalidDequeueMissingField {
    async fn on_connected(&self, ctx: &PlayerContext) -> BehaviorOutcome {
        let metadata = serde_json::json!({
            "test_session_id": ctx.test_session_id
        })
        .to_string();

        let msg = ClientMessage::Enqueue {
            player_id: ctx.player_id,
            game_mode: crate::default_game_mode(),
            metadata,
        };
        ctx.addr.do_send(InternalSendText(msg.to_string()));
        BehaviorOutcome::Continue
    }

    async fn on_enqueued(&self, ctx: &PlayerContext) -> BehaviorOutcome {
        // player_id 누락된 dequeue
        ctx.addr
            .do_send(InternalSendText("{\"type\":\"dequeue\"}".to_string()));
        BehaviorOutcome::IntendError
    }

    fn clone_trait(&self) -> Box<dyn PlayerBehavior> {
        Box::new(self.clone())
    }
}

/// Dequeued 이후 중복 Dequeue 시도
#[derive(Debug, Clone)]
pub struct InvalidDequeueDuplicate;

#[async_trait]
impl PlayerBehavior for InvalidDequeueDuplicate {
    async fn on_connected(&self, ctx: &PlayerContext) -> BehaviorOutcome {
        let metadata = serde_json::json!({
            "test_session_id": ctx.test_session_id
        })
        .to_string();

        let msg = ClientMessage::Enqueue {
            player_id: ctx.player_id,
            game_mode: crate::default_game_mode(),
            metadata,
        };
        ctx.addr.do_send(InternalSendText(msg.to_string()));
        BehaviorOutcome::Continue
    }

    async fn on_enqueued(&self, ctx: &PlayerContext) -> BehaviorOutcome {
        // 첫 번째 정상 dequeue
        let msg = ClientMessage::Dequeue {
            player_id: ctx.player_id,
            game_mode: crate::default_game_mode(),
        };
        ctx.addr.do_send(InternalSendText(msg.to_string()));
        BehaviorOutcome::Continue
    }

    async fn on_dequeued(&self, ctx: &PlayerContext) -> BehaviorOutcome {
        // Dequeued 후 중복 dequeue 시도
        let msg = ClientMessage::Dequeue {
            player_id: ctx.player_id,
            game_mode: crate::default_game_mode(),
        };
        ctx.addr.do_send(InternalSendText(msg.to_string()));
        BehaviorOutcome::IntendError
    }

    fn clone_trait(&self) -> Box<dyn PlayerBehavior> {
        Box::new(self.clone())
    }
}

/// 잘못된 player_id로 Dequeue 시도
#[derive(Debug, Clone)]
pub struct InvalidDequeueWrongPlayerId;

#[async_trait]
impl PlayerBehavior for InvalidDequeueWrongPlayerId {
    async fn on_connected(&self, ctx: &PlayerContext) -> BehaviorOutcome {
        let metadata = serde_json::json!({
            "test_session_id": ctx.test_session_id
        })
        .to_string();

        let msg = ClientMessage::Enqueue {
            player_id: ctx.player_id,
            game_mode: crate::default_game_mode(),
            metadata,
        };
        ctx.addr.do_send(InternalSendText(msg.to_string()));
        BehaviorOutcome::Continue
    }

    async fn on_enqueued(&self, ctx: &PlayerContext) -> BehaviorOutcome {
        // 다른 player_id로 Dequeue 시도
        let wrong_id = uuid::Uuid::new_v4();
        let msg = ClientMessage::Dequeue {
            player_id: wrong_id,
            game_mode: crate::default_game_mode(),
        };
        ctx.addr.do_send(InternalSendText(msg.to_string()));
        BehaviorOutcome::IntendError
    }

    fn clone_trait(&self) -> Box<dyn PlayerBehavior> {
        Box::new(self.clone())
    }
}
