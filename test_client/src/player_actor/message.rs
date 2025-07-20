use actix::Message;

use crate::{player_actor::PlayerState, TestFailure, WsSink, WsStream};

#[derive(Message)]
#[rtype(result = "()")]
pub struct SendMessage(pub String);

#[derive(Message)]
#[rtype(result = "uuid::Uuid")]
pub struct GetPlayerId;

#[derive(Message)]
#[rtype(result = "()")]

pub struct SetState(pub PlayerState);

#[derive(Message)]
#[rtype(result = "()")]

pub struct ConnectionEstablished {
    pub sink: WsSink,
    pub stream: WsStream,
}

#[derive(Message)]
#[rtype(result = "()")]

pub struct InternalSendText(pub String);
