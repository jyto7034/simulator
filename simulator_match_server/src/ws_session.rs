use crate::{
    matchmaker::actor::{CancelLoadingSession, DequeuePlayer, EnqueuePlayer},
    protocol::{ClientMessage, ServerMessage},
    pubsub::{Deregister, Register},
    Matchmaker, SubscriptionManager,
};
use actix::{
    Actor, ActorContext, Addr, AsyncContext, Handler, Recipient, Running, StreamHandler,
};
use actix_web_actors::ws;
use std::time::{Duration, Instant};
use tracing::{info, warn};
use uuid::Uuid;

const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);
const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);

pub struct MatchmakingSession {
    player_id: Option<Uuid>,
    game_mode: Option<String>,
    loading_session_id: Option<Uuid>,
    hb: Instant,
    matchmaker_addr: Addr<Matchmaker>,
    sub_manager: Recipient<Register>,
}

impl MatchmakingSession {
    pub fn new(
        matchmaker_addr: Addr<Matchmaker>,
        sub_manager: Recipient<Register>,
    ) -> Self {
        Self {
            player_id: None,
            game_mode: None,
            loading_session_id: None,
            hb: Instant::now(),
            matchmaker_addr,
            sub_manager,
        }
    }

    fn hb(&self, ctx: &mut ws::WebsocketContext<Self>) {
        ctx.run_interval(HEARTBEAT_INTERVAL, |act, ctx| {
            if Instant::now().duration_since(act.hb) > CLIENT_TIMEOUT {
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

    fn stopping(&mut self, _: &mut Self::Context) -> Running {
        if let Some(player_id) = self.player_id {
            // Unregister from the subscription manager
            let deregister_msg = Deregister { player_id };
            if let Err(e) = self.sub_manager.do_send(deregister_msg) {
                warn!("Failed to send Deregister message: {}", e);
            }

            // Send appropriate cleanup messages to the matchmaker
            match (self.loading_session_id, self.game_mode.clone()) {
                (Some(loading_session_id), _) => {
                    self.matchmaker_addr.do_send(CancelLoadingSession {
                        player_id,
                        loading_session_id,
                    });
                }
                (None, Some(game_mode)) => {
                    self.matchmaker_addr.do_send(DequeuePlayer {
                        player_id,
                        game_mode,
                    });
                }
                _ => {}
            }
        }
        info!("MatchmakingSession for player {:?} is stopping.", self.player_id);
        Running::Stop
    }
}

// This handler now receives messages forwarded from the SubscriptionManager
impl Handler<ServerMessage> for MatchmakingSession {
    type Result = ();

    fn handle(&mut self, msg: ServerMessage, ctx: &mut Self::Context) {
        if let ServerMessage::StartLoading { loading_session_id } = &msg {
            self.loading_session_id = Some(*loading_session_id);
        }
        ctx.text(serde_json::to_string(&msg).unwrap());
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
            Ok(ws::Message::Text(text)) => {
                match serde_json::from_str::<ClientMessage>(&text) {
                    Ok(ClientMessage::Enqueue { player_id, game_mode }) => {
                        if self.player_id.is_some() {
                            warn!("Player {} tried to enqueue more than once.", player_id);
                            return;
                        }

                        info!("Player {} requests queue for {}.", player_id, game_mode);
                        self.player_id = Some(player_id);
                        self.game_mode = Some(game_mode.clone());

                        // Register with the SubscriptionManager
                        let register_msg = Register {
                            player_id,
                            addr: ctx.address().recipient(),
                        };
                        if let Err(e) = self.sub_manager.do_send(register_msg) {
                             error!("Failed to send Register message: {}", e);
                             ctx.stop();
                             return;
                        }

                        // Send enqueue message to the matchmaker
                        self.matchmaker_addr.do_send(EnqueuePlayer {
                            player_id,
                            game_mode,
                        });
                    }
                    Ok(ClientMessage::LoadingComplete { loading_session_id }) => {
                        if let Some(player_id) = self.player_id {
                            info!("Player {} finished loading for session {}", player_id, loading_session_id);
                            // This message is now named HandleLoadingComplete in matchmaker
                            self.matchmaker_addr.do_send(crate::matchmaker::actor::HandleLoadingComplete {
                                player_id,
                                loading_session_id,
                            });
                        } else {
                            warn!("Received LoadingComplete from a session with no player_id.");
                        }
                    }
                    Err(e) => {
                        warn!("Failed to parse client message: {}", e);
                        ctx.text(serde_json::to_string(&ServerMessage::Error { message: "Invalid message format".to_string() }).unwrap());
                    }
                }
            }
            Ok(ws::Message::Close(reason)) => {
                ctx.close(reason);
                ctx.stop();
            }
            _ => ctx.stop(),
        }
    }
}