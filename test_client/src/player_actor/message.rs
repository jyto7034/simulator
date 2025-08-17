use actix::Message;

use crate::{
    behaviors::ServerMessage, player_actor::PlayerState, BehaviorResult, WsSink, WsStream,
};

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

#[derive(Message)]
#[rtype(result = "()")]
pub struct InternalClose;

#[derive(Message)]
#[rtype(result = "()")]
pub struct BehaviorFinished {
    pub response: BehaviorResult,
    pub original_message: ServerMessage,
}


#[derive(Message)]
#[rtype(result = "()")]
pub struct TriggerEnqueueNow;
