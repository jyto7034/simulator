use crate::provider::DedicatedServerProvider;
use actix::{Actor, Addr, AsyncContext, Context, Handler};
use redis::aio::ConnectionManager;
use serde_json::json;
use std::time::Duration;
use std::sync::Arc;
use std::sync::RwLock;

use tracing::info;

use super::messages::{CheckStaleLoadingSessions, GetDebugInfo, TryMatch};

#[derive(Clone)]

pub struct Matchmaker {
    pub(super) redis: ConnectionManager,
    pub(super) http_client: reqwest::Client,
    pub(super) current_run_id: Arc<RwLock<Option<String>>>,

    pub(super) settings: crate::env::MatchmakingSettings,
    pub(super) provider_addr: Addr<DedicatedServerProvider>,
}

impl Matchmaker {
    pub fn new(
        redis: ConnectionManager,
        settings: crate::env::MatchmakingSettings,
        provider_addr: Addr<DedicatedServerProvider>,
        current_run_id: Arc<RwLock<Option<String>>>,
    ) -> Self {
        Self {
            redis,
            http_client: reqwest::Client::new(),
            settings,
            provider_addr,
            current_run_id,
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
            Duration::from_secs(self.settings.loading_session_timeout_seconds),
            |_act, ctx| {
                ctx.address().do_send(CheckStaleLoadingSessions);
            },
        );
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
