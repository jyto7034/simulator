use super::PlayerBehavior;
use crate::behaviors::ClientMessage;
use crate::player_actor::message::InternalSendText;
use crate::{player_actor::PlayerContext, BehaviorOutcome, BehaviorResult};
use async_trait::async_trait;

/// InvalidMessages: 고의로 잘못된/순서가 어긋난 메시지를 전송해 서버의 강건성을 검증
/// - 모드 별로 다른 invalid 시나리오를 구성할 수 있도록 단순 variant 제공
#[derive(Debug, Clone)]
pub enum InvalidMode {
    /// 존재하지 않는 타입
    UnknownType,
    /// 필수 필드 누락
    MissingField,
    /// EnQueued 이후 중복 Enqueue 시도
    DuplicateEnqueue,
    /// 잘못된 player_id로 Dequeue 시도
    WrongPlayerId,
    /// 존재하지 않는 game_mode
    InvalidGameMode,
    /// 비정상적으로 큰 metadata (1MB)
    LargeMetadata,
    /// 잘못된 JSON 구조
    MalformedJson,
    /// Idle 상태에서 Dequeue 시도 (state machine 위반)
    IdleToDequeue,
}

#[derive(Debug, Clone)]
pub struct InvalidMessages {
    pub mode: InvalidMode,
}

#[async_trait]
impl PlayerBehavior for InvalidMessages {
    async fn on_connected(&self, ctx: &PlayerContext) -> BehaviorResult {
        match self.mode {
            InvalidMode::IdleToDequeue => {
                // Idle 상태에서 즉시 Dequeue 시도 (아직 Enqueue도 안했는데)
                let msg = ClientMessage::Dequeue {
                    player_id: ctx.player_id,
                    game_mode: crate::default_game_mode(),
                };
                ctx.addr.do_send(InternalSendText(msg.to_string()));
            }
            InvalidMode::InvalidGameMode => {
                // 존재하지 않는 game_mode로 Enqueue
                ctx.addr.do_send(InternalSendText(
                    format!(
                        r#"{{"type":"enqueue","player_id":"{}","game_mode":"NonExistentMode","metadata":"{{}}"}}"#,
                        ctx.player_id
                    )
                ));
            }
            InvalidMode::LargeMetadata => {
                // 1MB 크기의 metadata로 Enqueue
                let large_data = "x".repeat(1_000_000);
                let msg = ClientMessage::Enqueue {
                    player_id: ctx.player_id,
                    game_mode: crate::default_game_mode(),
                    metadata: format!(r#"{{"data":"{}"}}"#, large_data),
                };
                ctx.addr.do_send(InternalSendText(msg.to_string()));
            }
            InvalidMode::MalformedJson => {
                // 잘못된 JSON 구조 (닫히지 않은 중괄호)
                ctx.addr.do_send(InternalSendText(
                    r#"{"type":"enqueue","player_id":"#.to_string()
                ));
            }
            _ => {
                // 다른 모드들은 on_enqueued에서 처리
            }
        }
        Ok(BehaviorOutcome::Continue)
    }

    async fn on_enqueued(&self, ctx: &PlayerContext) -> BehaviorResult {
        match self.mode {
            InvalidMode::UnknownType => {
                ctx.addr
                    .do_send(InternalSendText("{\"type\":\"bad_type\"}".to_string()));
            }
            InvalidMode::MissingField => {
                // player_id 누락
                ctx.addr
                    .do_send(InternalSendText("{\"type\":\"enqueue\"}".to_string()));
            }
            InvalidMode::DuplicateEnqueue => {
                // 중복 Enqueue 시도
                let msg = ClientMessage::Enqueue {
                    player_id: ctx.player_id,
                    game_mode: crate::default_game_mode(),
                    metadata: "{}".to_string(),
                };
                ctx.addr.do_send(InternalSendText(msg.to_string()));
            }
            InvalidMode::WrongPlayerId => {
                // 다른 player_id로 Dequeue 시도
                let wrong_id = uuid::Uuid::new_v4();
                let msg = ClientMessage::Dequeue {
                    player_id: wrong_id,
                    game_mode: crate::default_game_mode(),
                };
                ctx.addr.do_send(InternalSendText(msg.to_string()));
            }
            _ => {
                // IdleToDequeue, InvalidGameMode, LargeMetadata, MalformedJson은
                // on_connected에서 이미 처리됨
            }
        }
        Ok(BehaviorOutcome::Continue)
    }

    fn clone_trait(&self) -> Box<dyn PlayerBehavior> {
        Box::new(self.clone())
    }
}
