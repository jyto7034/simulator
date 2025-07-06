use crate::{
    matchmaker::actor::{DequeuePlayer, EnqueuePlayer},
    protocol::{ClientMessage, ServerMessage},
    Matchmaker,
};
use actix::{
    fut, Actor, ActorContext, Addr, AsyncContext, Handler, Running, StreamHandler,
};
use actix_web_actors::ws;
use futures_util::stream::StreamExt;
use redis::Client as RedisClient;
use std::time::{Duration, Instant};
use tracing::{error, info, warn};
use uuid::Uuid;

const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);
const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);

pub struct MatchmakingSession {
    player_id: Option<Uuid>,
    game_mode: Option<String>, // Store the game mode for dequeueing
    hb: Instant,
    matchmaker_addr: Addr<Matchmaker>,
    redis_client: RedisClient,
}

impl MatchmakingSession {
    pub fn new(matchmaker_addr: Addr<Matchmaker>, redis_client: RedisClient) -> Self {
        Self {
            player_id: None,
            game_mode: None,
            hb: Instant::now(),
            matchmaker_addr,
            redis_client,
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
        if let (Some(player_id), Some(game_mode)) = (self.player_id, self.game_mode.clone()) {
            info!("Player {} disconnected, sending dequeue request for game mode {}", player_id, game_mode);
            self.matchmaker_addr.do_send(DequeuePlayer {
                player_id,
                game_mode,
            });
        }
        info!("MatchmakingSession for player {:?} is stopping.", self.player_id);
        Running::Stop
    }
}

impl Handler<ServerMessage> for MatchmakingSession {
    type Result = ();

    fn handle(&mut self, msg: ServerMessage, ctx: &mut Self::Context) {
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
                        
                        info!("Player {} requests queue for {}. Subscribing before enqueuing.", player_id, game_mode);
                        self.player_id = Some(player_id);
                        self.game_mode = Some(game_mode.clone());
                        
                        let redis_client = self.redis_client.clone();
                        let addr = ctx.address().clone();
                        let matchmaker_addr = self.matchmaker_addr.clone();

                        let future = async move {
                            let mut conn = match redis_client.get_async_connection().await {
                                Ok(c) => c,
                                Err(e) => {
                                    error!("Failed to get redis connection: {}", e);
                                    addr.do_send(ServerMessage::Error { message: "Internal server error".into() });
                                    return;
                                }
                            };
                            let mut pubsub = conn.into_pubsub();
                            let channel = format!("notifications:{}", player_id);
                            
                            if let Err(e) = pubsub.subscribe(&channel).await {
                                error!("Failed to subscribe to channel {}: {}", channel, e);
                                addr.do_send(ServerMessage::Error { message: "Internal server error".into() });
                                return;
                            }

                            // Now that we are subscribed, we can safely enqueue
                            matchmaker_addr.do_send(EnqueuePlayer {
                                player_id,
                                game_mode,
                            });

                            // Start listening for messages
                            let mut stream = pubsub.on_message();
                            while let Some(msg) = stream.next().await {
                                let payload: String = match msg.get_payload() {
                                    Ok(p) => p,
                                    Err(_) => continue,
                                };
                                if let Ok(server_msg) = serde_json::from_str::<ServerMessage>(&payload) {
                                    addr.do_send(server_msg);
                                }
                            }
                        };
                        ctx.spawn(fut::wrap_future(future));
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