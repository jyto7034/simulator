use actix::{Actor, Context};
use redis::aio::ConnectionManager;

use crate::env::Settings;

pub struct DedicatedServerProvider {
    redis: ConnectionManager,
    settings: Settings,
}

impl DedicatedServerProvider {
    pub fn new(redis: ConnectionManager, settings: Settings) -> Self {
        Self { redis, settings }
    }
}

impl Actor for DedicatedServerProvider {
    type Context = Context<Self>;
}
