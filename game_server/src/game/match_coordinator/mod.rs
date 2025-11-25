use actix::{Actor, Addr, Context};
use redis::aio::ConnectionManager;
use std::collections::HashMap;
use tracing::info;

use crate::game::load_balance_actor::LoadBalanceActor;
use crate::matchmaking::matchmaker::MatchmakerAddr;
use crate::GameMode;

pub mod handlers;
pub mod messages;

pub struct MatchCoordinator {
    matchmakers: HashMap<GameMode, MatchmakerAddr>,
    load_balance_addr: Addr<LoadBalanceActor>,
    redis: ConnectionManager,
}

impl MatchCoordinator {
    pub fn new(
        matchmakers: HashMap<GameMode, MatchmakerAddr>,
        load_balance_addr: Addr<LoadBalanceActor>,
        redis: ConnectionManager,
    ) -> Self {
        Self {
            matchmakers,
            load_balance_addr,
            redis,
        }
    }
}

impl Actor for MatchCoordinator {
    type Context = Context<Self>;

    fn started(&mut self, _ctx: &mut Self::Context) {
        info!("MatchCoordinator started");
    }
}
