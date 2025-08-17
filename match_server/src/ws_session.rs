// metrics helper removed
use crate::{
    matchmaker::messages::{CancelLoadingSession, DequeuePlayer, EnqueuePlayer},
    protocol::{ClientMessage, ServerMessage},
    pubsub::{Deregister, Register},
    Matchmaker, SubscriptionManager,
};
use actix::{
    fut, Actor, ActorContext, ActorFutureExt, Addr, AsyncContext, Handler, Running, StreamHandler,
};
use actix_web_actors::ws;
// metrics removed
use std::time::{Duration, Instant};
type Ctx = ws::WebsocketContext<MatchmakingSession>;

fn send_err(ctx: &mut Ctx, code: crate::protocol::ErrorCode, message: &str) {
    if let Ok(text) = serde_json::to_string(&crate::protocol::ServerMessage::Error {
        code: Some(code),
        message: message.to_string(),
    }) {
        ctx.text(text);
    }
}

use tracing::{info, warn};
use uuid::Uuid;
// test_behavior 경로 제거됨

/// Represents the state of the matchmaking session.
#[derive(Clone, Debug, PartialEq)]
enum SessionState {
    Idle,          // Initial state, no activity.
    Enqueuing,     // Enqueue request received, processing.
    InQueue,       // Successfully enqueued, waiting for a match.
    InLoading,     // Match found, loading assets.
    Completed,     // Completed (match found), normal end.
    Disconnecting, // Session is gracefully shutting down.
}

pub struct MatchmakingSession {
    player_id: Option<Uuid>,
    game_mode: Option<String>,
    loading_session_id: Option<Uuid>,
    state: SessionState,
    hb: Instant,
    matchmaker_addr: Addr<Matchmaker>,
    sub_manager_addr: Addr<SubscriptionManager>,
    heartbeat_interval: Duration,
    client_timeout: Duration,
    app_state: actix_web::web::Data<crate::AppState>,
}

impl MatchmakingSession {
    pub fn new(
        matchmaker_addr: Addr<Matchmaker>,
        sub_manager_addr: Addr<SubscriptionManager>,
        heartbeat_interval: Duration,
        client_timeout: Duration,
        app_state: actix_web::web::Data<crate::AppState>,
    ) -> Self {
        Self {
            player_id: None,
            game_mode: None,
            loading_session_id: None,
            state: SessionState::Idle,
            hb: Instant::now(),
            matchmaker_addr,
            sub_manager_addr,
            app_state,
            heartbeat_interval,
            client_timeout,
        }
    }

    fn hb(&self, ctx: &mut ws::WebsocketContext<Self>) {
        ctx.run_interval(self.heartbeat_interval, |act, ctx| {
            if Instant::now().duration_since(act.hb) > act.client_timeout {
                info!("Websocket Client heartbeat failed, disconnecting!");
                // metrics removed

                ctx.stop();
                return;
            }
            ctx.ping(b"");
        });
    }

    fn current_run_id(&self) -> Option<String> {
        if let Ok(guard) = self.app_state.current_run_id.read() {
            guard.clone()
        } else {
            None
        }
    }

    // test_behavior 핸들러 제거됨
}

impl Actor for MatchmakingSession {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        info!("MatchmakingSession started.");
        self.hb(ctx);
    }

    fn stopping(&mut self, ctx: &mut Self::Context) -> Running {
        if self.state == SessionState::Disconnecting {
            return Running::Stop;
        }
        let prev_state = self.state.clone();
        self.state = SessionState::Disconnecting;

        if let Some(player_id) = self.player_id {
            info!(
                "Player {:?} disconnected. Starting graceful shutdown...",
                self.player_id
            );

            let sub_manager_addr_inner = self.sub_manager_addr.clone();
            let matchmaker_addr_inner = self.matchmaker_addr.clone();
            let loading_session_id = self.loading_session_id;
            let game_mode = self.game_mode.clone();

            let gm_label = game_mode.clone().unwrap_or_else(|| "unknown".to_string());
            // Classify using the state before entering Disconnecting
            match prev_state {
                SessionState::Completed => { /* normal end, do not count as quit */ }
                SessionState::InLoading => {
                    // metrics removed
                }
                SessionState::InQueue | SessionState::Enqueuing => {
                    // metrics removed
                }
                _ => {
                    // metrics removed
                }
            }

            let cleanup_future = async move {
                let deregister_fut = sub_manager_addr_inner.send(Deregister { player_id });

                match (loading_session_id, game_mode.clone()) {
                    (Some(loading_session_id), _) => {
                        let cancel_fut = matchmaker_addr_inner.send(CancelLoadingSession {
                            player_id,
                            loading_session_id,
                        });
                        _ = tokio::join!(deregister_fut, cancel_fut);
                    }
                    (None, Some(game_mode)) => {
                        let dequeue_fut = matchmaker_addr_inner.send(DequeuePlayer {
                            player_id,
                            game_mode,
                        });
                        _ = tokio::join!(deregister_fut, dequeue_fut);
                    }
                    _ => {
                        _ = deregister_fut.await;
                    }
                }
            };

            let graceful_shutdown =
                fut::wrap_future::<_, Self>(cleanup_future).then(|_result, _actor, ctx| {
                    info!(
                        "Cleanup finished for player {:?}. Stopping actor now.",
                        _actor.player_id
                    );
                    ctx.stop();
                    fut::ready(())
                });

            ctx.wait(graceful_shutdown);
            Running::Continue
        } else {
            info!("Anonymous session is stopping.");
            // metrics removed
            Running::Stop
        }
    }
}

impl Handler<ServerMessage> for MatchmakingSession {
    type Result = ();

    fn handle(&mut self, msg: ServerMessage, ctx: &mut Self::Context) {
        match msg {
            ServerMessage::EnQueued => {
                self.state = SessionState::InQueue;
            }
            ServerMessage::StartLoading { loading_session_id } => {
                self.state = SessionState::InLoading;
                self.loading_session_id = Some(loading_session_id);
            }
            ServerMessage::MatchFound { .. } => {
                // Completed flow: clear loading session and mark as completed
                self.loading_session_id = None;
                self.state = SessionState::Completed;
            }
            _ => {}
        }

        match serde_json::to_string(&msg) {
            Ok(text) => ctx.text(text),
            Err(e) => warn!("Failed to serialize ServerMessage for client: {}", e),
        }
    }
}
impl MatchmakingSession {
    fn handle_enqueue(&mut self, ctx: &mut Ctx, player_id: Uuid, game_mode: String) {
        if self.state != SessionState::Idle {
            warn!(
                "Player {:?} sent Enqueue request in non-idle state: {:?}. Ignoring.",
                self.player_id.or(Some(player_id)),
                self.state
            );
            return;
        }
        info!(
            "Player {} requests queue for {}. Updating session state.",
            player_id, game_mode
        );
        self.state = SessionState::Enqueuing;
        self.player_id = Some(player_id);
        self.game_mode = Some(game_mode.clone());
        self.sub_manager_addr.do_send(Register {
            player_id,
            addr: ctx.address(),
        });
        self.matchmaker_addr.do_send(EnqueuePlayer {
            player_id,
            game_mode,
        });
    }

    fn handle_loading_complete(&mut self, ctx: &mut Ctx, loading_session_id: Uuid) {
        if self.state != SessionState::InLoading {
            warn!(
                "Received LoadingComplete from player {:?} not in loading state. Ignoring.",
                self.player_id
            );
            return;
        }
        if let Some(player_id) = self.player_id {
            info!(
                "Player {} finished loading for session {}",
                player_id, loading_session_id
            );
            if Some(loading_session_id) != self.loading_session_id {
                send_err(
                    ctx,
                    crate::protocol::ErrorCode::WrongSessionId,
                    "Wrong session id",
                );
                return;
            }

            self.matchmaker_addr
                .do_send(crate::matchmaker::messages::HandleLoadingComplete {
                    player_id,
                    loading_session_id,
                });
        } else {
            warn!("Received LoadingComplete from a session with no player_id.");
        }
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for MatchmakingSession {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Ping(msg)) => {
                self.hb = Instant::now();
                ctx.pong(&msg);
            }
            Ok(ws::Message::Pong(_)) => {
                self.hb = Instant::now();
            }
            Ok(ws::Message::Text(text)) => match serde_json::from_str::<ClientMessage>(&text) {
                Ok(ClientMessage::Enqueue {
                    player_id,
                    game_mode,
                }) => {
                    self.handle_enqueue(ctx, player_id, game_mode);
                }
                Ok(ClientMessage::LoadingComplete { loading_session_id }) => {
                    self.handle_loading_complete(ctx, loading_session_id);
                }
                // test_behavior 메시지는 제거됨
                Err(e) => {
                    warn!("Failed to parse client message: {}", e);
                    // metrics removed
                    send_err(
                        ctx,
                        crate::protocol::ErrorCode::InvalidMessageFormat,
                        "Invalid message format",
                    );
                }
            },
            Ok(ws::Message::Close(reason)) => {
                ctx.close(reason);
                ctx.stop();
            }
            _ => ctx.stop(),
        }
    }
}
