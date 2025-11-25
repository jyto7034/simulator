use actix::{Actor, Addr, Arbiter, MailboxError};
use redis::aio::ConnectionManager;
use std::collections::HashMap;
use tokio_util::sync::CancellationToken;

use crate::{
    env::MatchmakingSettings,
    game::load_balance_actor::LoadBalanceActor,
    matchmaking::matchmaker::{
        messages::{Dequeue, Enqueue},
        normal::NormalMatchmaker,
        rank::RankedMatchmaker,
    },
    matchmaking::subscript::SubScriptionManager,
    shared::{circuit_breaker::CircuitBreaker, metrics::MetricsCtx},
    GameMode,
};

pub mod common;
pub mod messages;
pub mod normal;
pub mod operations;
pub mod rank;
pub mod scripts;

#[derive(Clone)]
pub enum MatchmakerAddr {
    Normal(Addr<NormalMatchmaker>),
    Ranked(Addr<RankedMatchmaker>),
}

impl MatchmakerAddr {
    pub fn do_send_enqueue(&self, msg: Enqueue) {
        match self {
            Self::Normal(addr) => addr.do_send(msg),
            Self::Ranked(addr) => addr.do_send(msg),
        }
    }

    pub fn do_send_dequeue(&self, msg: Dequeue) {
        match self {
            Self::Normal(addr) => addr.do_send(msg),
            Self::Ranked(addr) => addr.do_send(msg),
        }
    }

    pub async fn dequeue(&self, msg: Dequeue) -> Result<(), MailboxError> {
        match self {
            Self::Normal(addr) => addr.send(msg).await,
            Self::Ranked(addr) => addr.send(msg).await,
        }
    }
}

#[derive(Clone)]
pub struct MatchmakerDeps {
    pub redis: ConnectionManager,
    pub settings: MatchmakingSettings,
    pub subscription_addr: Addr<SubScriptionManager>,
    pub load_balance_addr: Addr<LoadBalanceActor>,
    pub metrics: std::sync::Arc<MetricsCtx>,
    pub shutdown_token: CancellationToken,
    pub redis_circuit: std::sync::Arc<CircuitBreaker>,
}

impl From<&common::MatchmakerInner> for MatchmakerDeps {
    fn from(source: &common::MatchmakerInner) -> Self {
        Self {
            redis: source.redis.clone(),
            settings: source.settings.clone(),
            subscription_addr: source.sub_manager_addr.clone(),
            load_balance_addr: source.load_balance_addr.clone(),
            metrics: source.metrics.clone(),
            shutdown_token: source.shutdown_token.clone(),
            redis_circuit: source.redis_circuit.clone(),
        }
    }
}

pub fn spawn_matchmaker_for_mode(
    game_mode: GameMode,
    deps: &MatchmakerDeps,
) -> Result<MatchmakerAddr, String> {
    // settings에서 해당 game_mode의 MatchModeSettings 찾기
    let mode_settings = deps
        .settings
        .game_modes
        .iter()
        .find(|m| m.game_mode == game_mode)
        .cloned()
        .ok_or_else(|| format!("MatchModeSettings not found for mode {:?}", game_mode))?;

    match game_mode {
        GameMode::None => Err("Unsupported game mode: None".to_string()),
        GameMode::Normal => {
            let redis = deps.redis.clone();
            let settings = deps.settings.clone();
            let subscription_addr = deps.subscription_addr.clone();
            let load_balance_addr = deps.load_balance_addr.clone();
            let metrics = deps.metrics.clone();
            let shutdown_token = deps.shutdown_token.clone();
            let redis_circuit = deps.redis_circuit.clone();

            let arbiter = Arbiter::new();
            let addr = NormalMatchmaker::start_in_arbiter(&arbiter.handle(), move |_ctx| {
                NormalMatchmaker::new(
                    redis,
                    settings,
                    mode_settings,
                    subscription_addr,
                    load_balance_addr,
                    metrics,
                    shutdown_token,
                    redis_circuit,
                )
            });
            Ok(MatchmakerAddr::Normal(addr))
        }
        GameMode::Ranked => {
            let redis = deps.redis.clone();
            let settings = deps.settings.clone();
            let subscription_addr = deps.subscription_addr.clone();
            let load_balance_addr = deps.load_balance_addr.clone();
            let metrics = deps.metrics.clone();
            let shutdown_token = deps.shutdown_token.clone();
            let redis_circuit = deps.redis_circuit.clone();

            let arbiter = Arbiter::new();
            let addr = RankedMatchmaker::start_in_arbiter(&arbiter.handle(), move |_ctx| {
                RankedMatchmaker::new(
                    redis,
                    settings,
                    mode_settings,
                    subscription_addr,
                    load_balance_addr,
                    metrics,
                    shutdown_token,
                    redis_circuit,
                )
            });
            Ok(MatchmakerAddr::Ranked(addr))
        }
    }
}

pub fn spawn_matchmakers<I>(
    deps: &MatchmakerDeps,
    modes: I,
) -> Result<HashMap<GameMode, MatchmakerAddr>, String>
where
    I: IntoIterator<Item = GameMode>,
{
    let mut map = HashMap::new();
    for mode in modes {
        let handle = spawn_matchmaker_for_mode(mode, deps)?;
        map.insert(mode, handle);
    }
    Ok(map)
}
