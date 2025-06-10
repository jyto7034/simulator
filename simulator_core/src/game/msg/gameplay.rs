use actix::{Handler, Message};
use uuid::Uuid;

use crate::{
    card::types::PlayerKind,
    exception::{GameError, StateError},
    game::{phase::Phase, GameActor},
};

#[derive(Message)]
#[rtype(result = "Result<(), GameError>")]
pub struct RequestPlayCard {
    pub player_type: PlayerKind,
    pub card_id: Uuid,
}

#[derive(Message)]
#[rtype(result = "Result<(), GameError>")]
pub struct SubmitInput {
    pub player_type: PlayerKind,
    pub request_id: Uuid,
}

#[derive(Message)]
#[rtype(result = "Result<(), GameError>")]
pub struct RequestInput {
    pub player_type: PlayerKind,
    pub request_id: Uuid,
}

#[derive(Message)]
#[rtype(result = "Result<(), GameError>")]
pub struct IsCorrectPhase {
    pub phase: Phase,
}

pub struct ChoiceCardRequestPayload {
    pub player: String,
    pub choice_type: String,
    pub source_card_id: Uuid,
    pub min_selections: usize,
    pub max_selections: usize,
    pub destination: String,
    pub is_open: bool,
    pub is_hidden_from_opponent: bool,
}

impl Handler<RequestPlayCard> for GameActor {
    type Result = Result<(), GameError>;

    fn handle(&mut self, msg: RequestPlayCard, _: &mut Self::Context) -> Self::Result {
        Ok(())
    }
}

impl Handler<SubmitInput> for GameActor {
    type Result = Result<(), GameError>;

    fn handle(&mut self, msg: SubmitInput, _: &mut Self::Context) -> Self::Result {
        Ok(())
    }
}

impl Handler<RequestInput> for GameActor {
    type Result = Result<(), GameError>;

    fn handle(&mut self, msg: RequestInput, _: &mut Self::Context) -> Self::Result {
        Ok(())
    }
}

impl Handler<IsCorrectPhase> for GameActor {
    type Result = Result<(), GameError>;

    fn handle(&mut self, msg: IsCorrectPhase, _: &mut Self::Context) -> Self::Result {
        if self.turn.current_phase == msg.phase {
            Ok(())
        } else {
            Err(GameError::State(StateError::InvalidActionForPhase { current_phase: format!("{:?}", self.turn.current_phase), action: format!("{:?}", msg.phase) }))
        }
    }
}
