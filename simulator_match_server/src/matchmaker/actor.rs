use actix::{Actor, Context, Handler};

use crate::matchmaker::message::{JoinQueue, LeaveQueue, MatchmakingError, Tick};

pub struct MatchMakingActor {}

impl MatchMakingActor {
    pub fn new() -> Self {
        todo!()
    }
}

impl Actor for MatchMakingActor {
    type Context = Context<Self>;
}

impl Handler<JoinQueue> for MatchMakingActor {
    type Result = Result<(), MatchmakingError>;
    fn handle(&mut self, msg: JoinQueue, ctx: &mut Self::Context) -> Self::Result {
        todo!()
    }
}

impl Handler<LeaveQueue> for MatchMakingActor {
    type Result = Result<(), MatchmakingError>;

    fn handle(&mut self, msg: LeaveQueue, ctx: &mut Self::Context) -> Self::Result {
        todo!()
    }
}

impl Handler<Tick> for MatchMakingActor {
    type Result = Result<(), MatchmakingError>;

    fn handle(&mut self, msg: Tick, ctx: &mut Self::Context) -> Self::Result {
        todo!()
    }
}
