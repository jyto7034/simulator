use std::{
    ops::{Deref, DerefMut},
    time::Duration,
};

use actix::{Actor, Addr, AsyncContext};
use redis::aio::ConnectionManager;
use tokio_util::sync::CancellationToken;
use tracing::info;

pub mod handlers;

use crate::{
    env::{MatchModeSettings, MatchmakingSettings},
    game::load_balance_actor::LoadBalanceActor,
    matchmaking::matchmaker::{common::MatchmakerInner, messages::TryMatch},
    matchmaking::subscript::SubScriptionManager,
    shared::{circuit_breaker::CircuitBreaker, metrics::MetricsCtx},
};

pub struct RankedMatchmaker {
    inner: MatchmakerInner,
}

impl Deref for RankedMatchmaker {
    type Target = MatchmakerInner;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for RankedMatchmaker {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl RankedMatchmaker {
    pub fn new(
        redis: ConnectionManager,
        settings: MatchmakingSettings,
        mode_settings: MatchModeSettings,
        sub_manager_addr: Addr<SubScriptionManager>,
        load_balance_addr: Addr<LoadBalanceActor>,
        metrics: std::sync::Arc<MetricsCtx>,
        shutdown_token: CancellationToken,
        redis_circuit: std::sync::Arc<CircuitBreaker>,
    ) -> Self {
        Self {
            inner: MatchmakerInner::new(
                redis,
                settings,
                mode_settings,
                sub_manager_addr,
                load_balance_addr,
                metrics,
                shutdown_token,
                redis_circuit,
            ),
        }
    }
}

impl Actor for RankedMatchmaker {
    type Context = actix::Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        info!(
            "RankedMatchmaker actor started for mode {:?}",
            self.mode_settings.game_mode
        );
        let interval = self.settings.try_match_tick_interval_seconds;
        let mode_settings = self.mode_settings.clone();
        let addr = ctx.address();
        let shutdown_token = self.shutdown_token.clone();

        // Use tokio::spawn instead of ctx.run_interval for more reliable timing
        tokio::spawn(async move {
            let mut interval_timer = tokio::time::interval(Duration::from_secs(interval));
            // First tick completes immediately, so we skip it to start after `interval` seconds
            interval_timer.tick().await;

            loop {
                interval_timer.tick().await;

                if shutdown_token.is_cancelled() {
                    break;
                }

                addr.do_send(TryMatch {
                    match_mode_settings: mode_settings.clone(),
                });
            }
        });
    }

    fn stopping(&mut self, _ctx: &mut Self::Context) -> actix::Running {
        info!(
            "RankedMatchmaker for mode {:?} stopping, cancelling futures",
            self.mode_settings.game_mode
        );

        // 모든 실행 중인 future에게 종료 신호
        self.shutdown_token.cancel();

        actix::Running::Stop
    }
}
