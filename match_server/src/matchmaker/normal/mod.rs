use std::{
    ops::{Deref, DerefMut},
    time::Duration,
};

use actix::{Actor, Addr, AsyncContext};
use redis::aio::ConnectionManager;
use tokio_util::sync::CancellationToken;
use tracing::{info, warn};

pub mod handlers;

use crate::{
    env::{MatchModeSettings, MatchmakingSettings},
    matchmaker::{common::MatchmakerInner, messages::TryMatch},
    metrics::MetricsCtx,
    subscript::SubScriptionManager,
};

pub struct NormalMatchmaker {
    inner: MatchmakerInner,
}

impl Deref for NormalMatchmaker {
    type Target = MatchmakerInner;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for NormalMatchmaker {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl NormalMatchmaker {
    pub fn new(
        redis: ConnectionManager,
        settings: MatchmakingSettings,
        mode_settings: MatchModeSettings,
        sub_manager_addr: Addr<SubScriptionManager>,
        metrics: std::sync::Arc<MetricsCtx>,
        shutdown_token: CancellationToken,
    ) -> Self {
        Self {
            inner: MatchmakerInner::new(redis, settings, mode_settings, sub_manager_addr, metrics, shutdown_token),
        }
    }
}

impl Actor for NormalMatchmaker {
    type Context = actix::Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        info!(
            "NormalMatchmaker actor started for mode {:?}",
            self.mode_settings.game_mode
        );
        let interval = self.settings.try_match_tick_interval_seconds;
        let mode_settings = self.mode_settings.clone();

        ctx.run_interval(Duration::from_secs(interval), move |_actor, ctx| {
            ctx.notify(TryMatch {
                match_mode_settings: mode_settings.clone(),
            });
        });
    }

    fn stopping(&mut self, _ctx: &mut Self::Context) -> actix::Running {
        info!(
            "NormalMatchmaker for mode {:?} stopping, cancelling futures",
            self.mode_settings.game_mode
        );

        // 모든 실행 중인 future에게 종료 신호
        self.shutdown_token.cancel();

        // 즉시 종료 (System shutdown 중에는 run_later() 호출 불가)
        info!("Stopping immediately");
        actix::Running::Stop
    }
}
