use actix::Message;

#[derive(Message)]
#[rtype(result = "()")]
pub struct ResetReconnectAttempts;

#[derive(Message)]
#[rtype(result = "()")]
pub struct RecordFailure;

#[derive(Message)]
#[rtype(result = "()")]
pub struct Connect;
