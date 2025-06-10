pub mod connection;
pub mod error_message;
pub mod gameplay;
pub mod lifecycle;
pub mod mulligan;
pub mod zones;

use actix::Message;
use uuid::Uuid;

#[derive(Message)]
#[rtype(result = "()")]
pub enum GameEvent {
    GameStopped,
    SendMulliganDealCards { cards: Vec<Uuid> },
}
