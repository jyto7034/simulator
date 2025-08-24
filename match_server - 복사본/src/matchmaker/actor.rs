use crate::{blacklist::BlacklistManager, provider::DedicatedServerProvider, pubsub::SubscriptionManager};
use actix::{Actor, Addr, AsyncContext, Context, Handler};
use redis::aio::ConnectionManager;
use serde_json::json;
use std::time::Duration;

use tracing::info;

use super::messages::{CheckStaleLoadingSessions, GetDebugInfo, SetLoadingSessionManager, TryMatch};

#[derive(Clone)]

pub struct Matchmaker {
    pub(super) redis: ConnectionManager,
    pub(super) http_client: reqwest::Client,

    pub(super) settings: crate::env::MatchmakingSettings,
    pub(super) provider_addr: Addr<DedicatedServerProvider>,
    pub(super) sub_manager_addr: Addr<SubscriptionManager>,
    pub(super) blacklist_manager_addr: Addr<BlacklistManager>,
    pub(super) loading_session_manager_addr: Option<Addr<crate::loading_session::LoadingSessionManager>>,
    pub(super) metrics: std::sync::Arc<crate::metrics::MetricsCtx>,
}

impl Matchmaker {
    pub fn new(
        redis: ConnectionManager,
        settings: crate::env::MatchmakingSettings,
        provider_addr: Addr<DedicatedServerProvider>,
        sub_manager_addr: Addr<SubscriptionManager>,
        blacklist_manager_addr: Addr<BlacklistManager>,
        metrics: std::sync::Arc<crate::metrics::MetricsCtx>,
    ) -> Self {
        Self {
            redis,
            http_client: reqwest::Client::new(),
            settings,
            provider_addr,
            sub_manager_addr,
            blacklist_manager_addr,
            loading_session_manager_addr: None, // Will be set later
            metrics,
        }
    }

    pub fn set_loading_session_manager(&mut self, addr: Addr<crate::loading_session::LoadingSessionManager>) {
        self.loading_session_manager_addr = Some(addr);
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
        // 오래된 로딩 세션 정리 타이머 (TTL과 동일 주기로 레이스가 나는 것을 방지하기 위해 더 자주 실행)
        let timeout_secs = self.settings.loading_session_timeout_seconds;
        let mut cleanup_secs = timeout_secs / 4;
        if cleanup_secs < 5 {
            cleanup_secs = 5;
        }
        ctx.run_interval(Duration::from_secs(cleanup_secs), |_act, ctx| {
            ctx.address().do_send(CheckStaleLoadingSessions);
        });
    }
}

impl Handler<SetLoadingSessionManager> for Matchmaker {
    type Result = ();

    fn handle(&mut self, msg: SetLoadingSessionManager, _ctx: &mut Self::Context) -> Self::Result {
        self.loading_session_manager_addr = Some(msg.addr);
        info!("Loading session manager reference set in Matchmaker");
    }
}

impl Handler<GetDebugInfo> for Matchmaker {
    type Result = String;

    fn handle(&mut self, _msg: GetDebugInfo, _ctx: &mut Context<Self>) -> Self::Result {
        let debug_info = json!({
            "matchmaker_status": "active",
            "settings": {
                "tick_interval_seconds": self.settings.tick_interval_seconds,
                "loading_session_timeout_seconds": self.settings.loading_session_timeout_seconds,
                "game_modes": self.settings.game_modes.iter().map(|mode| {
                    json!({
                        "id": mode.id,
                        "required_players": mode.required_players,
                        "use_mmr_matching": mode.use_mmr_matching
                    })
                }).collect::<Vec<_>>()
            },
            "internal_state": {
                "redis_connected": true,
                "http_client_ready": true,
                "provider_addr_available": true
            }
        });
        debug_info.to_string()
    }
}
