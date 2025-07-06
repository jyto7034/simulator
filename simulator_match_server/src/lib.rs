use crate::{matchmaker::Matchmaker, provider::DedicatedServerProvider};
use actix::Addr;
use redis::Client as RedisClient;

pub mod auth;
pub mod env;
pub mod matchmaker;
pub mod util;
pub mod ws_session;
pub mod protocol;
pub mod provider;

// 서버 전체에서 공유될 상태
#[derive(Clone)]
pub struct AppState {
    pub jwt_secret: String,
    pub redis_client: RedisClient,
    pub matchmaker_addr: Addr<Matchmaker>,
    pub provider_addr: Addr<DedicatedServerProvider>,
}

