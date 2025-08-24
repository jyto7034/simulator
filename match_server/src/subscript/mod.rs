use actix::{Actor, Context};

pub mod handlers;
pub mod messages;

pub struct SubScriptionManager {}

impl Actor for SubScriptionManager {
    type Context = Context<Self>;
}
