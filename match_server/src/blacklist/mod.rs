use actix::{Actor, Context};

pub struct BlacklistManager {}

impl Actor for BlacklistManager {
    type Context = Context<Self>;
}
