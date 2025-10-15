use actix::Message;

use crate::{
    player_actor::PlayerState, protocols::ServerMessage, BehaviorOutcome, WsSink, WsStream,
};

#[derive(Message)]
#[rtype(result = "()")]

pub struct InternalSendText(pub String);

#[derive(Message)]
#[rtype(result = "()")]

pub struct InternalClose;

#[derive(Message)]
#[rtype(result = "()")]

pub struct BehaviorFinished {
    pub response: BehaviorOutcome,
    pub original_message: ServerMessage,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct ConnectionEstablished {
    pub sink: WsSink,
    pub stream: WsStream,
}
#[derive(Message)]
#[rtype(result = "()")]
pub struct SetState(pub PlayerState);
