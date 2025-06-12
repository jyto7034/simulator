// ===================================================================
// 1. 클라이언트에게 전송될 동기화 페이로드 정의
// ===================================================================

use actix::{Message, Recipient};

use crate::{card::types::PlayerKind, game::msg::GameEvent, sync::types::StateChange};
// ===================================================================
// 2. SyncActor와 상호작용하기 위한 메시지 정의
// ===================================================================

/// GameActor가 SyncActor에게 상태 변경 목록을 알리기 위해 사용하는 메시지
#[derive(Message)]
#[rtype(result = "()")]
pub struct NotifyChanges(pub Vec<StateChange>);

/// GameActor가 SyncActor에게 주기적인 해시 계산 및 전송을 요청하는 메시지
#[derive(Message)]
#[rtype(result = "()")]
pub struct RequestStateHashSync;

/// ConnectionActor가 연결될 때 SyncActor에 자신을 등록하는 메시지
#[derive(Message)]
#[rtype(result = "()")]
pub struct RegisterConnectionToSync {
    pub player: PlayerKind,
    pub recipient: Recipient<GameEvent>,
}

/// ConnectionActor가 연결을 해제할 때 SyncActor에서 등록을 해제하는 메시지
#[derive(Message)]
#[rtype(result = "()")]
pub struct UnregisterConnectionFromSync {
    pub player: PlayerKind,
}
