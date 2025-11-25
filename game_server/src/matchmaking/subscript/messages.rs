use actix::{Addr, Message};
use uuid::Uuid;

use crate::{matchmaking::session::Session, shared::protocol::ServerMessage};

#[derive(Message)]
#[rtype(result = "()")]
pub struct Register {
    pub player_id: Uuid,
    pub addr: Addr<Session>,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct Deregister {
    pub player_id: Uuid,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct ForwardServerMessage {
    pub player_id: Uuid,
    pub message: ServerMessage,
}
