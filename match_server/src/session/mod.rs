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
    metadata: Option<String>, // Store metadata for test event publishing
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
            metadata: None,
        }
    }

    fn transition_to(
        &mut self,
        new_state: SessionState,
        ctx: &mut ws::WebsocketContext<Self>,
    ) -> bool {
        // 유효한 상태 전환인지 확인. 만약 유효하지 않다면 Error 상태로 전환 후 error 메시지 전송.
        if !self.state.can_transition_to(new_state) {
            let violation = classify_violation(self.state, new_state);

            // Metrics: 상태 전환 위반
            metrics::STATE_VIOLATIONS_TOTAL.inc();

            match violation {
                TransitionViolation::Minor => {
                    warn!("Minor state violation, ignoring transition");
                    return false; // 현재 상태 유지
                }

                TransitionViolation::Major => {
                    self.state = SessionState::Error;
                    self.send_error(ctx, ErrorCode::InternalError, "Invalid state transition");
                    return false;
                }

                TransitionViolation::Critical => {
                    self.send_error(ctx, ErrorCode::InternalError, "Critical protocol violation");
                    // 메시지 전송 후 약간 대기 후 우아하게 종료
                    ctx.run_later(Duration::from_millis(100), |_act, ctx| {
                        ctx.close(Some(ws::CloseCode::Protocol.into()));
                        ctx.stop();
                    });
                    return false;
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
        true
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

        // Session is already initialized to Idle state in constructor
        // No need to transition_to(Idle) here

        self.hb(ctx);
    }

    fn stopping(&mut self, ctx: &mut Self::Context) -> Running {
        info!("Session stopping for player {:#?}", self.player_id);

        if self.cleanup_started {
            // Metrics: WebSocket 연결 감소
            metrics::ACTIVE_WS_CONNECTIONS.dec();
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
        let current_state = self.state;

        async move {
            // Only dequeue if player is still in queue
            // Skip if already dequeued or never enqueued
            let res_match = if let Some(addr) = matchmaker_addr {
                if current_state == SessionState::InQueue
                    || current_state == SessionState::Dequeuing
                {
                    addr.dequeue(Dequeue {
                        player_id,
                        game_mode,
                    })
                    .await
                } else {
                    Ok(()) // Already dequeued or never enqueued, skip cleanup dequeue
                }
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
        .spawn(ctx);

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
            self.send_error(
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
                self.send_error(ctx, code, "Unsupported game mode");
                return;
            }
        };

        if !self.transition_to(SessionState::Enqueuing, ctx) {
            return; // State transition failed, stop processing
        }

        self.player_id = player_id;
        self.game_mode = game_mode;
        self.metadata = Some(metadata.clone()); // Store metadata for error event publishing
        let player_id = self.player_id;
        let game_mode = self.game_mode;
        let subscript_addr = self.subscript_addr.clone();
        let ctx_addr = ctx.address();

        async move {
            // TODO: Retry 로직
            if let Err(err) = subscript_addr
                .send(Register {
                    player_id,
                    addr: ctx_addr,
                })
                .await
            {
                warn!("Failed to register player {}: {:?}", player_id, err);
                return;
            }

            matchmaker.do_send_enqueue(Enqueue {
                player_id: player_id,
                game_mode: game_mode,
                metadata,
            });
        }
        .into_actor(self)
        .spawn(ctx);
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
            self.send_error(ctx, ErrorCode::WrongSessionId, "Player ID mismatch");
            return;
        }

        let matchmaker = match self.resolve_matchmaker(game_mode) {
            Ok(handle) => handle,
            Err(code) => {
                self.send_error(ctx, code, "Unsupported game mode");
                return;
            }
        };

        if !self.transition_to(SessionState::Dequeuing, ctx) {
            return; // State transition failed, stop processing
        }

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

    /// Send error to client via WebSocket and publish to Redis event stream for tests
    fn send_error(&self, ctx: &mut Ctx, code: ErrorCode, message: &str) {
        // Publish to Redis event stream first (before moving code)
        if let Some(ref metadata) = self.metadata {
            let mut redis = self.app_state.redis.clone();
            let metadata = metadata.clone();
            let player_id = self.player_id;
            let pod_id = self.app_state.current_run_id.clone();
            let error_code_str = format!("{:?}", code); // Use Debug trait to get error code name
            let error_message = message.to_string();

            actix::spawn(async move {
                crate::redis_events::try_publish_test_event(
                    &mut redis,
                    &metadata,
                    "player.error",
                    pod_id.to_string().as_str(),
                    vec![
                        ("player_id", player_id.to_string()),
                        ("code", error_code_str),
                        ("message", error_message),
                    ],
                )
                .await;
            });
        }

        // Send via WebSocket (code is moved here)
        helper::send_err(ctx, code, message);
    }
}
