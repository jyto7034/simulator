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
use std::time::{Duration, Instant};
use tracing::{info, warn};
use uuid::Uuid;

/// Represents the state of the matchmaking session.
#[derive(Clone, Debug, PartialEq)]
enum SessionState {
    Idle,          // Initial state, no activity.
    Enqueuing,     // Enqueue request received, processing.
    InQueue,       // Successfully enqueued, waiting for a match.
    InLoading,     // Match found, loading assets.
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
}

impl MatchmakingSession {
    pub fn new(
        matchmaker_addr: Addr<Matchmaker>,
        sub_manager_addr: Addr<SubscriptionManager>,
        heartbeat_interval: Duration,
        client_timeout: Duration,
    ) -> Self {
        Self {
            player_id: None,
            game_mode: None,
            loading_session_id: None,
            state: SessionState::Idle,
            hb: Instant::now(),
            matchmaker_addr,
            sub_manager_addr,
            heartbeat_interval,
            client_timeout,
        }
    }

    fn hb(&self, ctx: &mut ws::WebsocketContext<Self>) {
        ctx.run_interval(self.heartbeat_interval, |act, ctx| {
            if Instant::now().duration_since(act.hb) > act.client_timeout {
                info!("Websocket Client heartbeat failed, disconnecting!");
                ctx.stop();
                return;
            }
            ctx.ping(b"");
        });
    }
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
            _ => {}
        }

        match serde_json::to_string(&msg) {
            Ok(text) => ctx.text(text),
            Err(e) => warn!("Failed to serialize ServerMessage for client: {}", e),
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
                Ok(ClientMessage::LoadingComplete { loading_session_id }) => {
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
                        self.matchmaker_addr.do_send(
                            crate::matchmaker::messages::HandleLoadingComplete {
                                player_id,
                                loading_session_id,
                            },
                        );
                    } else {
                        warn!("Received LoadingComplete from a session with no player_id.");
                    }
                }
                Err(e) => {
                    warn!("Failed to parse client message: {}", e);
                    match serde_json::to_string(&ServerMessage::Error {
                        message: "Invalid message format".to_string(),
                    }) {
                        Ok(text) => ctx.text(text),
                        Err(e) => warn!("Failed to serialize error message for client: {}", e),
                    }
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
