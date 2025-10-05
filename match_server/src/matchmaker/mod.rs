use actix::{Actor, Addr, MailboxError};
use redis::aio::ConnectionManager;
use std::collections::HashMap;
use tokio_util::sync::CancellationToken;

use crate::{
    env::MatchmakingSettings,
    matchmaker::{
        messages::{Dequeue, Enqueue},
        normal::NormalMatchmaker,
        rank::RankedMatchmaker,
    },
    metrics::MetricsCtx,
    subscript::SubScriptionManager,
    GameMode,
};

pub mod common;
pub mod messages;
pub mod normal;
pub mod operations;
pub mod patry;
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
    pub metrics: std::sync::Arc<MetricsCtx>,
    pub shutdown_token: CancellationToken,
}

impl From<&common::MatchmakerInner> for MatchmakerDeps {
    fn from(source: &common::MatchmakerInner) -> Self {
        Self {
            redis: source.redis.clone(),
            settings: source.settings.clone(),
            subscription_addr: source.sub_manager_addr.clone(),
            metrics: source.metrics.clone(),
            shutdown_token: source.shutdown_token.clone(),
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
        GameMode::None => {
            Err("Unsupported game mode: None".to_string())
        }
        GameMode::Normal => Ok(MatchmakerAddr::Normal(
            NormalMatchmaker::new(
                deps.redis.clone(),
                deps.settings.clone(),
                mode_settings,
                deps.subscription_addr.clone(),
                deps.metrics.clone(),
                deps.shutdown_token.clone(),
            )
            .start(),
        )),
        GameMode::Ranked => Ok(MatchmakerAddr::Ranked(
            RankedMatchmaker::new(
                deps.redis.clone(),
                deps.settings.clone(),
                mode_settings,
                deps.subscription_addr.clone(),
                deps.metrics.clone(),
                deps.shutdown_token.clone(),
            )
            .start(),
        )),
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
