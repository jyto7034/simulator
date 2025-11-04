use std::{collections::HashMap, sync::Arc};

use actix::{Actor, Addr, Context};
use tracing::info;
use uuid::Uuid;

use crate::{game::player_game_actor::PlayerGameActor, shared::metrics::MetricsCtx};

pub mod handlers;
pub mod messages;

pub struct LoadBalanceActor {
    players: HashMap<Uuid, Addr<PlayerGameActor>>,
    metrics: Arc<MetricsCtx>,
}

impl LoadBalanceActor {
    pub fn new(metrics: Arc<MetricsCtx>) -> Self {
        Self {
            players: HashMap::new(),
            metrics,
        }
    }
}

impl Actor for LoadBalanceActor {
    type Context = Context<Self>;

    fn started(&mut self, _ctx: &mut Self::Context) {
        info!("LoadBalanceActor started");
    }
}
