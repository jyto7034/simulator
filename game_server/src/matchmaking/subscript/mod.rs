use std::collections::HashMap;

use actix::{Actor, Addr, Context};
use uuid::Uuid;

use crate::matchmaking::session::Session;

pub mod handlers;
pub mod messages;

pub struct SubScriptionManager {
    pub sessions: HashMap<Uuid, Addr<Session>>,
}

impl Actor for SubScriptionManager {
    type Context = Context<Self>;
}

impl SubScriptionManager {
    pub fn new() -> Self {
        Self {
            sessions: HashMap::new(),
        }
    }
}
