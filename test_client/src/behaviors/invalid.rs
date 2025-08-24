use super::PlayerBehavior;
use crate::behaviors::ClientMessage;
use crate::player_actor::message::InternalSendText;
use crate::{player_actor::PlayerContext, BehaviorOutcome, BehaviorResult};
use async_trait::async_trait;
use uuid::Uuid;

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
    /// 잘못된(다른) 로딩 세션 ID로 loading_complete 전송
    WrongSessionId,
}

#[derive(Debug, Clone)]
pub struct InvalidMessages {
    pub mode: InvalidMode,
}

#[async_trait]
impl PlayerBehavior for InvalidMessages {
    async fn on_enqueued(&self, ctx: &PlayerContext) -> BehaviorResult {
        match self.mode {
            InvalidMode::UnknownType => {
                ctx.addr
                    .do_send(InternalSendText("{\"type\":\"bad_type\"}".to_string()));
            }
            InvalidMode::MissingField => {
                ctx.addr
                    .do_send(InternalSendText("{\"type\":\"enqueue\"}".to_string()));
            }

            InvalidMode::DuplicateEnqueue => {
                let msg = ClientMessage::Enqueue {
                    player_id: ctx.player_id,
                    game_mode: crate::default_game_mode(),
                };
                ctx.addr.do_send(InternalSendText(msg.to_string()));
            }
            _ => {}
        }
        Ok(BehaviorOutcome::Continue)
    }

    async fn on_loading_start(&self, ctx: &PlayerContext, _id: Uuid) -> BehaviorResult {
        match self.mode {
            InvalidMode::WrongSessionId => {
                // 올바르지 않은(다른) 세션 ID로 완료 통지
                let wrong = Uuid::new_v4();
                let msg = ClientMessage::LoadingComplete {
                    loading_session_id: wrong,
                };
                ctx.addr.do_send(InternalSendText(msg.to_string()));
                return Ok(BehaviorOutcome::Stop);
            }
            InvalidMode::DuplicateEnqueue => {
                // 이후 플로우는 정상으로 진행하여 시나리오를 종료할 수 있게 한다
                let msg = ClientMessage::LoadingComplete {
                    loading_session_id: _id,
                };
                ctx.addr.do_send(InternalSendText(msg.to_string()));
                return Ok(BehaviorOutcome::Stop);
            }
            _ => {}
        }
        Ok(BehaviorOutcome::Continue)
    }

    fn clone_trait(&self) -> Box<dyn PlayerBehavior> {
        Box::new(self.clone())
    }
}
