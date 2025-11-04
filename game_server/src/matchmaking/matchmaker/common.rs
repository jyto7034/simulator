use std::sync::{atomic::AtomicBool, Arc};

use actix::Addr;
use redis::aio::ConnectionManager;
use tokio_util::sync::CancellationToken;

use crate::{
    env::{MatchModeSettings, MatchmakingSettings},
    game::load_balance_actor::LoadBalanceActor,
    matchmaking::subscript::SubScriptionManager,
    shared::{circuit_breaker::CircuitBreaker, metrics::MetricsCtx},
    GameMode,
};

pub struct MatchmakerInner {
    pub redis: ConnectionManager,
    pub settings: MatchmakingSettings,
    pub mode_settings: MatchModeSettings,
    pub sub_manager_addr: Addr<SubScriptionManager>,
    pub load_balance_addr: Addr<LoadBalanceActor>,
    pub metrics: Arc<MetricsCtx>,
    pub shutdown_token: CancellationToken,
    pub is_matching: Arc<AtomicBool>,
    pub redis_circuit: Arc<CircuitBreaker>,
}

impl MatchmakerInner {
    pub fn new(
        redis: ConnectionManager,
        settings: MatchmakingSettings,
        mode_settings: MatchModeSettings,
        sub_manager_addr: Addr<SubScriptionManager>,
        load_balance_addr: Addr<LoadBalanceActor>,
        metrics: Arc<MetricsCtx>,
        shutdown_token: CancellationToken,
        redis_circuit: Arc<CircuitBreaker>,
    ) -> Self {
        Self {
            redis,
            settings,
            mode_settings,
            sub_manager_addr,
            load_balance_addr,
            metrics,
            shutdown_token,
            is_matching: Arc::new(AtomicBool::new(false)),
            redis_circuit,
        }
    }

    pub fn queue_suffix(&self, mode: GameMode) -> &'static str {
        match mode {
            GameMode::Ranked => "ranked",
            GameMode::Normal => "normal",
            GameMode::None => "none",
        }
    }
}
