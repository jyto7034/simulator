use std::collections::HashMap;

use actix::{Actor, Addr, Context};
use tracing::info;
use uuid::Uuid;

use crate::game::player_game_actor::PlayerGameActor;

pub mod handlers;
pub mod messages;

pub struct LoadBalanceActor {
    players: HashMap<Uuid, Addr<PlayerGameActor>>,
}

impl LoadBalanceActor {
    pub fn new() -> Self {
        Self {
            players: HashMap::new(),
        }
    }
}

impl Actor for LoadBalanceActor {
    type Context = Context<Self>;

    fn started(&mut self, _ctx: &mut Self::Context) {
        info!("LoadBalanceActor started");
    }
}
