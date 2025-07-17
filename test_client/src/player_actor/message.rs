use actix::Message;

#[derive(Message)]
#[rtype(result = "()")]
pub struct SendMessage(String);

#[derive(Message)]
#[rtype(result = "uuid::Uuid")]
pub struct GetPlayerId;
