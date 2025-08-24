use crate::{
    blacklist::messages::{CheckPlayerBlock, RecordViolation, BlockCheckResult},
    blacklist::ViolationType,
    matchmaker::messages::{CancelLoadingSession, DequeuePlayer, EnqueuePlayer},
    protocol::{ClientMessage, ServerMessage},
    pubsub::{Deregister, Register},
    Matchmaker, SubscriptionManager,
};
use std::net::IpAddr;
use actix::{
    fut, Actor, ActorContext, ActorFutureExt, Addr, AsyncContext, Handler, Running, StreamHandler, WrapFuture,
};
use actix_web_actors::ws;
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

use tracing::{debug, info, warn};
use uuid::Uuid;

/// Represents the state of the matchmaking session.
#[derive(Clone, Debug, PartialEq)]
enum SessionState {
    Idle,          // Initial state, no activity.
    Enqueuing,     // Enqueue request received, processing.
    InQueue,       // Successfully enqueued, waiting for a match.
    InLoading,     // Match found, loading assets.
    Completed,     // Completed (match found), normal end.
    Disconnecting, // Session is gracefully shutting down.
    Error,         // Error state, requires cleanup.
}

impl SessionState {
    /// Check if transition from current state to new state is valid
    fn can_transition_to(&self, new_state: &SessionState) -> bool { 
        use SessionState::*;
        match (self, new_state) {
            // From Idle
            (Idle, Enqueuing) => true,
            (Idle, Disconnecting) => true,
            
            // From Enqueuing
            (Enqueuing, InQueue) => true,
            (Enqueuing, Error) => true,
            (Enqueuing, Disconnecting) => true,
            
            // From InQueue
            (InQueue, InLoading) => true,
            (InQueue, Error) => true,
            (InQueue, Disconnecting) => true,
            
            // From InLoading
            (InLoading, Completed) => true,
            (InLoading, Error) => true,
            (InLoading, Disconnecting) => true,
            
            // From Completed
            (Completed, Disconnecting) => true,
            
            // From Error (allow recovery)
            (Error, Enqueuing) => true,    // Allow re-enrollment after error
            (Error, Disconnecting) => true,
            
            // From Disconnecting (terminal state)
            (Disconnecting, _) => false,
            
            // All other transitions are invalid
            _ => false,
        }
    }
    
    /// Get human-readable description of the state
    fn description(&self) -> &'static str {
        match self {
            SessionState::Idle => "Waiting for player input",
            SessionState::Enqueuing => "Processing enqueue request",
            SessionState::InQueue => "Waiting for match",
            SessionState::InLoading => "Loading match assets",
            SessionState::Completed => "Match completed successfully",
            SessionState::Disconnecting => "Cleaning up connection",
            SessionState::Error => "Error occurred, cleaning up",
        }
    }
    
    /// Check if state requires cleanup on disconnect
    fn requires_cleanup(&self) -> bool {
        matches!(self, SessionState::InQueue | SessionState::InLoading | SessionState::Enqueuing)
    }
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
    client_ip: Option<IpAddr>,
}

impl MatchmakingSession {
    /// Safely transition to a new state with validation
    fn transition_to(&mut self, new_state: SessionState, ctx: &mut ws::WebsocketContext<Self>) {
        if !self.state.can_transition_to(&new_state) {
            // Handle specific duplicate message cases gracefully
            match (&self.state, &new_state) {
                (SessionState::InLoading, SessionState::InLoading) => {
                    warn!(
                        "Duplicate StartLoading message ignored for player {:?}",
                        self.player_id
                    );
                    return; // Simply ignore duplicate, don't treat as error
                }
                _ => {
                    warn!(
                        "Invalid state transition attempted: {:?} -> {:?} for player {:?}",
                        self.state, new_state, self.player_id
                    );
                    self.app_state.metrics.inc_state_violation();
                    
                    // Force transition to Error state for other invalid transitions
                    if self.state != SessionState::Error && new_state != SessionState::Disconnecting {
                        self.state = SessionState::Error;
                        send_err(ctx, crate::protocol::ErrorCode::InternalError, "Invalid state transition");
                        return;
                    }
                }
            }
        }
        
        let old_state = self.state.clone();
        self.state = new_state.clone();
        
        info!(
            "Player {:?} transitioned: {} -> {}",
            self.player_id,
            old_state.description(),
            new_state.description()
        );
    }
}

impl MatchmakingSession {
    pub fn new(
        matchmaker_addr: Addr<Matchmaker>,
        sub_manager_addr: Addr<SubscriptionManager>,
        heartbeat_interval: Duration,
        client_timeout: Duration,
        app_state: actix_web::web::Data<crate::AppState>,
        client_ip: Option<IpAddr>,
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
            client_ip,
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
        self.transition_to(SessionState::Disconnecting, ctx);

        if let Some(player_id) = self.player_id {
            info!(
                "Player {:?} disconnected. Starting graceful shutdown...",
                self.player_id
            );

            let sub_manager_addr_inner = self.sub_manager_addr.clone();
            let matchmaker_addr_inner = self.matchmaker_addr.clone();
            let loading_session_id = self.loading_session_id;
            let game_mode = self.game_mode.clone();

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

            // Immediately deregister from session manager to prevent further messages
            sub_manager_addr_inner.do_send(Deregister { player_id });

            let cleanup_future = async move {
                // Only perform cleanup if the previous state requires it
                if prev_state.requires_cleanup() {
                    match (loading_session_id, game_mode.clone()) {
                        (Some(loading_session_id), _) => {
                            let cancel_fut = matchmaker_addr_inner.send(CancelLoadingSession {
                                player_id,
                                loading_session_id,
                            });
                            _ = cancel_fut.await;
                        }
                        (None, Some(game_mode)) => {
                            let dequeue_fut = matchmaker_addr_inner.send(DequeuePlayer {
                                player_id,
                                game_mode,
                            });
                            _ = dequeue_fut.await;
                        }
                        _ => {
                            // No further cleanup needed for anonymous sessions
                        }
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
            Running::Stop
        }
    }
}

impl Handler<ServerMessage> for MatchmakingSession {
    type Result = ();

    fn handle(&mut self, msg: ServerMessage, ctx: &mut Self::Context) {
        match msg {
            ServerMessage::EnQueued => {
                self.transition_to(SessionState::InQueue, ctx);
            }
            ServerMessage::StartLoading { loading_session_id } => {
                // Check if this is a duplicate StartLoading message
                if self.state == SessionState::InLoading {
                    if let Some(existing_session_id) = self.loading_session_id {
                        if existing_session_id != loading_session_id {
                            warn!(
                                "Received StartLoading with different session ID for player {:?}: existing={}, new={}",
                                self.player_id, existing_session_id, loading_session_id
                            );
                            self.app_state.metrics.inc_state_violation();
                            return;
                        } else {
                            // Same session ID - just ignore the duplicate
                            debug!(
                                "Ignoring duplicate StartLoading for player {:?} with session {}",
                                self.player_id, loading_session_id
                            );
                            return;
                        }
                    }
                }
                
                self.transition_to(SessionState::InLoading, ctx);
                self.loading_session_id = Some(loading_session_id);
            }
            ServerMessage::MatchFound { .. } => {
                // Validate current state before transitioning
                if self.state != SessionState::InLoading {
                    warn!(
                        "Received MatchFound in unexpected state {:?} for player {:?}",
                        self.state, self.player_id
                    );
                    self.app_state.metrics.inc_state_violation();
                    return;
                }
                
                self.loading_session_id = None;
                self.transition_to(SessionState::Completed, ctx);
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
        // Allow recovery from Error state
        if self.state != SessionState::Idle && self.state != SessionState::Error {
            // Abnormal: DuplicateEnqueue attempt while not Idle or Error
            self.app_state.metrics.abnormal_duplicate_enqueue();
            // Record duplicate enqueue violation
            self.app_state.blacklist_manager_addr.do_send(RecordViolation {
                player_id,
                violation_type: ViolationType::Duplicated,
                ip_addr: None, // IP tracking can be added later if needed
            });
            warn!(
                "Player {:?} sent Enqueue request in invalid state: {:?}. Ignoring.",
                self.player_id.or(Some(player_id)),
                self.state
            );
            return;
        }

        // Log recovery from error state
        if self.state == SessionState::Error {
            info!(
                "Player {:?} attempting to recover from Error state by re-enrolling",
                self.player_id.or(Some(player_id))
            );
        }

        // Check blacklist before processing enqueue
        let blacklist_manager = self.app_state.blacklist_manager_addr.clone();
        let client_ip = self.client_ip;
        let future = async move {
            blacklist_manager.send(CheckPlayerBlock {
                player_id,
                ip_addr: client_ip,
            }).await
        };

        let ctx_addr = ctx.address();
        let game_mode_clone = game_mode.clone();
        
        ctx.spawn(
            future
                .into_actor(self)
                .map(move |result, act, ctx| {
                    match result {
                        Ok(Ok(BlockCheckResult::Allowed)) => {
                            // Player is not blocked, proceed with enqueue
                            act.proceed_with_enqueue(ctx, player_id, game_mode_clone, ctx_addr);
                        }
                        Ok(Ok(BlockCheckResult::Blocked { remaining_seconds, reason })) => {
                            // Player is blocked
                            warn!("Player {} is blocked for {} more seconds", player_id, remaining_seconds);
                            let minutes = (remaining_seconds + 59) / 60; // Round up to minutes
                            let message = format!("{} 잠시 후 다시 시도해주세요. (약 {}분 후)", reason, minutes);
                            
                            let error_msg = ServerMessage::Error {
                                code: Some(crate::protocol::ErrorCode::PlayerTemporarilyBlocked),
                                message,
                            };
                            
                            if let Ok(text) = serde_json::to_string(&error_msg) {
                                ctx.text(text);
                            }
                            ctx.close(Some(ws::CloseReason::from(ws::CloseCode::Policy)));
                        }
                        Ok(Err(e)) => {
                            // Error checking blacklist - allow connection but log
                            warn!("Failed to check blacklist for player {}: {}. Allowing connection.", player_id, e);
                            act.proceed_with_enqueue(ctx, player_id, game_mode_clone, ctx_addr);
                        }
                        Err(e) => {
                            // Actor communication error - allow connection but log
                            warn!("Blacklist actor communication failed for player {}: {}. Allowing connection.", player_id, e);
                            act.proceed_with_enqueue(ctx, player_id, game_mode_clone, ctx_addr);
                        }
                    }
                }),
        );
    }

    fn proceed_with_enqueue(&mut self, ctx: &mut Ctx, player_id: Uuid, game_mode: String, ctx_addr: Addr<MatchmakingSession>) {
        info!(
            "Player {} requests queue for {}. Updating session state.",
            player_id, game_mode
        );
        self.transition_to(SessionState::Enqueuing, ctx);
        self.player_id = Some(player_id);
        self.game_mode = Some(game_mode.clone());
        self.sub_manager_addr.do_send(Register {
            player_id,
            addr: ctx_addr,
        });

        self.matchmaker_addr.do_send(EnqueuePlayer {
            player_id,
            game_mode,
        });
    }

    fn handle_loading_complete(&mut self, ctx: &mut Ctx, loading_session_id: Uuid) {
        if self.state != SessionState::InLoading {
            // Abnormal: MissingField/WrongSequence, ignore gracefully
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
                // Abnormal: WrongSessionId
                self.app_state.metrics.abnormal_wrong_session_id();
                if let Some(player_id) = self.player_id {
                    self.app_state.blacklist_manager_addr.do_send(RecordViolation {
                        player_id,
                        violation_type: ViolationType::WrongSessionId,
                        ip_addr: None,
                    });
                }
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

            // Send to new LoadingSessionManager
            self.app_state.loading_session_manager_addr.do_send(
                crate::loading_session::PlayerLoadingComplete {
                    player_id,
                    session_id: loading_session_id,
                }
            );
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
                Err(e) => {
                    // Classify parse errors as MissingField or UnknownType if possible
                    let msg = e.to_string();
                    if msg.contains("unknown variant") || msg.contains("unknown field") {
                        self.app_state.metrics.abnormal_unknown_type();
                        if let Some(player_id) = self.player_id {
                            self.app_state.blacklist_manager_addr.do_send(RecordViolation {
                                player_id,
                                violation_type: ViolationType::UnknownType,
                                ip_addr: None,
                            });
                        }
                    } else if msg.contains("missing field") {
                        self.app_state.metrics.abnormal_missing_field();
                        if let Some(player_id) = self.player_id {
                            self.app_state.blacklist_manager_addr.do_send(RecordViolation {
                                player_id,
                                violation_type: ViolationType::MissingField,
                                ip_addr: None,
                            });
                        }
                    }
                    warn!("Failed to parse client message: {}", e);
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
