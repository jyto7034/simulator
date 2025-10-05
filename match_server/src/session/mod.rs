use crate::matchmaker::messages::{Dequeue, Enqueue};
use crate::protocol::ErrorCode;
use crate::session::helper::{classify_violation, send_err, SessionState, TransitionViolation};
use crate::subscript::messages::{Deregister, Register};
use crate::{matchmaker::MatchmakerAddr, subscript::SubScriptionManager, AppState, GameMode};
use crate::{Stop, StopReason};
use actix::dev::ContextFutureSpawner;
use actix::ActorContext;
use actix::{Actor, Addr, WrapFuture};
use actix::{AsyncContext, Running};
use actix_web::web;
use actix_web_actors::ws::{self};
use std::cell::OnceCell;
use std::time::Instant;
use std::{net::IpAddr, time::Duration};
use tracing::{info, warn};
use uuid::Uuid;

pub mod handlers;
pub mod helper;

type Ctx = ws::WebsocketContext<Session>;

pub struct Session {
    state: SessionState,
    matchmaker_addr: OnceCell<MatchmakerAddr>,
    subscript_addr: Addr<SubScriptionManager>,
    app_state: web::Data<AppState>,
    player_id: Uuid,
    game_mode: GameMode,
    heartbeat_interval: Duration,
    heartbeat_timeout: Duration,
    last_heartbeat: Instant,
    cleanup_started: bool,
    client_ip: IpAddr,
}

impl Session {
    pub fn new(
        subscript_addr: Addr<SubScriptionManager>,
        heartbeat_interval: Duration,
        heartbeat_timeout: Duration,
        app_state: web::Data<AppState>,
        client_ip: IpAddr,
    ) -> Self {
        Self {
            state: SessionState::Idle,
            matchmaker_addr: OnceCell::new(),
            subscript_addr,
            app_state,
            player_id: Uuid::new_v4(),
            game_mode: GameMode::None,
            heartbeat_interval,
            heartbeat_timeout,
            last_heartbeat: Instant::now(),
            cleanup_started: false,
            client_ip,
        }
    }

    fn transition_to(&mut self, new_state: SessionState, ctx: &mut ws::WebsocketContext<Self>) {
        // 유효한 상태 전환인지 확인. 만약 유효하지 않다면 Error 상태로 전환 후 error 메시지 전송.
        if !self.state.can_transition_to(new_state) {
            let violation = classify_violation(self.state, new_state);

            // Metrics: 상태 전환 위반
            metrics::STATE_VIOLATIONS_TOTAL.inc();

            match violation {
                TransitionViolation::Minor => {
                    warn!("Minor state violation, ignoring transition");
                    return; // 현재 상태 유지
                }

                TransitionViolation::Major => {
                    self.state = SessionState::Error;
                    send_err(ctx, ErrorCode::InternalError, "Invalid state transition");
                    return;
                }

                TransitionViolation::Critical => {
                    send_err(ctx, ErrorCode::InternalError, "Critical protocol violation");
                    // 메시지 전송 후 약간 대기 후 우아하게 종료
                    ctx.run_later(Duration::from_millis(100), |_act, ctx| {
                        ctx.close(Some(ws::CloseCode::Protocol.into()));
                        ctx.stop();
                    });
                    return;
                }
            }
        }
        let old_state = self.state;
        self.state = new_state;

        info!(
            "Player {:?} transitioned: {} -> {}",
            self.player_id,
            old_state.description(),
            new_state.description()
        );
    }

    fn hb(&self, ctx: &mut ws::WebsocketContext<Self>) {
        ctx.run_interval(self.heartbeat_interval, |act, ctx| {
            if act.last_heartbeat.elapsed() > act.heartbeat_timeout {
                info!("Websocket Client heartbeat failed, disconnecting!");
                ctx.stop();
                return;
            }
            ctx.ping(b"");
        });
    }
}

impl Actor for Session {
    type Context = Ctx;

    fn started(&mut self, ctx: &mut Self::Context) {
        info!("Session started for player {:?}", self.player_id);

        // Metrics: WebSocket 연결 증가
        metrics::ACTIVE_WS_CONNECTIONS.inc();

        self.transition_to(SessionState::Idle, ctx);

        self.hb(ctx);
    }

    fn stopping(&mut self, ctx: &mut Self::Context) -> Running {
        info!("Session stopping for player {:#?}", self.player_id);

        // Metrics: WebSocket 연결 감소
        metrics::ACTIVE_WS_CONNECTIONS.dec();

        if self.cleanup_started {
            return Running::Stop;
        }

        self.cleanup_started = true;

        ctx.run_later(Duration::from_secs(10), |_act, ctx| {
            warn!("Cleanup wachdog triggered - forcing shutdown");
            ctx.stop();
        });

        let matchmaker_addr = self.matchmaker_addr.get().cloned();
        let subscription_addr = self.subscript_addr.clone();
        let ctx_clone = ctx.address();
        let player_id = self.player_id.clone();
        let game_mode = self.game_mode.clone();

        async move {
            let res_match = if let Some(addr) = matchmaker_addr {
                addr.dequeue(Dequeue {
                    player_id,
                    game_mode,
                })
                .await
            } else {
                Ok(())
            };
            let res_sub = subscription_addr.send(Deregister { player_id }).await;

            match (res_match, res_sub) {
                (Ok(()), Ok(())) => {
                    info!("Successfully sent message to matchmaker and subscription manager.")
                }
                (Ok(()), Err(e)) => {
                    warn!("Failed to send Dequeue to matchmaker manager. : {:?}", e)
                }
                (Err(e), Ok(())) => warn!(
                    "Failed to send Deregister to subscription manager. : {:?}",
                    e
                ),
                (Err(e), Err(_e)) => warn!(
                    "Failed to send message to matchmaker and subscription manager. {:?}, {:?}",
                    e, _e
                ),
            }

            ctx_clone.do_send(Stop {
                reason: StopReason::GracefulShutdown,
            });
        }
        .into_actor(self)
        .wait(ctx);

        Running::Continue
    }
}

impl Session {
    fn handle_enqueue(
        &mut self,
        ctx: &mut Ctx,
        player_id: Uuid,
        game_mode: GameMode,
        metadata: String,
    ) {
        // Rate limiting check
        if !self.app_state.rate_limiter.check(&self.client_ip) {
            warn!("Rate limit exceeded for IP: {}", self.client_ip);
            send_err(
                ctx,
                ErrorCode::RateLimitExceeded,
                "Too many requests. Please slow down.",
            );
            return;
        }

        // Session 객체 상태가 Idle 에서만 Enqueue 허용
        if self.state != SessionState::Idle && self.state != SessionState::Error {
            warn!(
                "Player {:?} sent Enqueue request in invalid state: {:?}. Ignoring.",
                self.player_id, self.state
            );
            return;
        }

        let matchmaker = match self.resolve_matchmaker(game_mode) {
            Ok(handle) => handle,
            Err(code) => {
                send_err(ctx, code, "Unsupported game mode");
                return;
            }
        };

        self.transition_to(SessionState::Enqueuing, ctx);

        self.player_id = player_id;
        self.game_mode = game_mode;
        matchmaker.do_send_enqueue(Enqueue {
            player_id: self.player_id,
            game_mode: self.game_mode,
            metadata,
        });

        self.subscript_addr.do_send(Register {
            player_id,
            addr: ctx.address(),
        });
    }

    fn handle_dequeue(&mut self, ctx: &mut Ctx, player_id: Uuid, game_mode: GameMode) {
        // InQueue 상태에서만 Dequeue 허용
        if self.state != SessionState::InQueue {
            warn!(
                "Player {:?} sent Dequeue request in invalid state: {:?}. Ignoring.",
                self.player_id, self.state
            );
            return;
        }

        // player_id 검증
        if self.player_id != player_id {
            send_err(ctx, ErrorCode::WrongSessionId, "Player ID mismatch");
            return;
        }

        let matchmaker = match self.resolve_matchmaker(game_mode) {
            Ok(handle) => handle,
            Err(code) => {
                send_err(ctx, code, "Unsupported game mode");
                return;
            }
        };

        self.transition_to(SessionState::Dequeuing, ctx);

        matchmaker.do_send_dequeue(Dequeue {
            player_id: self.player_id,
            game_mode: self.game_mode,
        });
    }
}

impl Session {
    fn resolve_matchmaker(&mut self, game_mode: GameMode) -> Result<MatchmakerAddr, ErrorCode> {
        if let Some(existing) = self.matchmaker_addr.get() {
            return Ok(existing.clone());
        }

        let handle = self
            .app_state
            .matchmakers
            .get(&game_mode)
            .cloned()
            .ok_or(ErrorCode::InvalidGameMode)?;

        let _ = self.matchmaker_addr.set(handle.clone());

        Ok(handle)
    }
}
