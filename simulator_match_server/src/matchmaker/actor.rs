use crate::provider::DedicatedServerProvider;
use actix::{Actor, Addr, AsyncContext, Context};
use redis::aio::ConnectionManager;
use std::time::Duration;
use tracing::info;

use super::messages::{CheckStaleLoadingSessions, TryMatch};

pub(super) const LOADING_SESSION_TIMEOUT_SECONDS: u64 = 60;

// --- Actor Definition ---
pub struct Matchmaker {
    pub(super) redis: ConnectionManager,
    pub(super) http_client: reqwest::Client,
    pub(super) settings: crate::env::MatchmakingSettings,
    pub(super) provider_addr: Addr<DedicatedServerProvider>,
}

impl Matchmaker {
    pub fn new(
        redis: ConnectionManager,
        settings: crate::env::MatchmakingSettings,
        provider_addr: Addr<DedicatedServerProvider>,
    ) -> Self {
        Self {
            redis,
            http_client: reqwest::Client::new(),
            settings,
            provider_addr,
        }
    }
}

impl Actor for Matchmaker {
    type Context = Context<Self>;
    fn started(&mut self, ctx: &mut Self::Context) {
        info!("Matchmaker actor started.");
        // 매칭 시도 타이머
        ctx.run_interval(
            Duration::from_secs(self.settings.tick_interval_seconds),
            |act, ctx| {
                for mode_settings in act.settings.game_modes.clone() {
                    ctx.address().do_send(TryMatch {
                        game_mode: mode_settings,
                    });
                }
            },
        );
        // 오래된 로딩 세션 정리 타이머
        ctx.run_interval(
            Duration::from_secs(LOADING_SESSION_TIMEOUT_SECONDS),
            |_act, ctx| {
                ctx.address().do_send(CheckStaleLoadingSessions);
            },
        );
    }
}
