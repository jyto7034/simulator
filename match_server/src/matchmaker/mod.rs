use actix::{Actor, Addr};
use redis::aio::ConnectionManager;

use crate::{
    blacklist::BlacklistManager, env::MatchmakingSettings, metrics::MetricsCtx,
    provider::DedicatedServerProvider, subscript::SubScriptionManager,
};

pub mod handlers;
pub mod handlers_inner;
pub mod messages;

pub struct Matchmaker {
    pub redis: ConnectionManager,
    pub http_client: reqwest::Client,

    pub settings: MatchmakingSettings,
    pub provider_addr: Addr<DedicatedServerProvider>,
    pub sub_manager_addr: Addr<SubScriptionManager>,
    pub blacklist_manager_addr: Addr<BlacklistManager>,
    pub metrics: std::sync::Arc<MetricsCtx>,
}

impl Actor for Matchmaker {
    type Context = actix::Context<Self>;
}
