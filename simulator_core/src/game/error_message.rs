use actix::{Handler, Message};
use tracing::error;

use crate::exception::GameError;

use super::GameActor;

#[derive(Message)]
#[rtype(result = "Result<(), GameError>")]
pub struct PingSentFailure {
    pub error_message: String,
}

impl Handler<PingSentFailure> for GameActor {
    type Result = Result<(), GameError>;

    fn handle(&mut self, msg: PingSentFailure, _: &mut Self::Context) -> Self::Result {
        error!("Ping sent failure: {}", msg.error_message);
        Err(GameError::PingSentFailure)
    }
}
