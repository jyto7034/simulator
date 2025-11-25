use actix::{Actor, Context};

pub mod handlers;

pub struct PlayerGameActor {}

impl Actor for PlayerGameActor {
    type Context = Context<Self>;
}
