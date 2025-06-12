pub mod connection;
pub mod error_message;
pub mod gameplay;
pub mod helper;
pub mod lifecycle;
pub mod mulligan;
pub mod system;
pub mod zones;

use actix::Message;
use uuid::Uuid;

use crate::sync::{snapshots::GameStateSnapshot, types::StateUpdatePayload};

#[derive(Message, Clone)]
#[rtype(result = "()")]
pub enum GameEvent {
    GameStopped,
    SendMulliganDealCards { cards: Vec<Uuid> },
    SyncState { snapshot: GameStateSnapshot },
    StateUpdate(StateUpdatePayload), // 신규: 델타 업데이트를 위한 variant
}
