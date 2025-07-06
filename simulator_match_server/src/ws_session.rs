use crate::{
    matchmaker::actor::{CancelLoadingSession, DequeuePlayer, EnqueuePlayer, HandleLoadingComplete},
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
    game_mode: Option<String>,
    loading_session_id: Option<Uuid>, // 현재 참여 중인 로딩 세션 ID
    hb: Instant,
    matchmaker_addr: Addr<Matchmaker>,
    redis_client: RedisClient,
}

impl MatchmakingSession {
    pub fn new(matchmaker_addr: Addr<Matchmaker>, redis_client: RedisClient) -> Self {
        Self {
            player_id: None,
            game_mode: None,
            loading_session_id: None,
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
        // 세션이 종료될 때, 상태에 따라 적절한 정리 메시지를 보냅니다.
        match (self.player_id, self.loading_session_id, self.game_mode.clone()) {
            // 로딩 중에 연결이 끊긴 경우
            (Some(player_id), Some(loading_session_id), _) => {
                info!("Player {} disconnected during loading session {}, sending cancel request.", player_id, loading_session_id);
                self.matchmaker_addr.do_send(CancelLoadingSession {
                    player_id,
                    loading_session_id,
                });
            },
            // 단순 대기열에만 있다가 연결이 끊긴 경우
            (Some(player_id), None, Some(game_mode)) => {
                info!("Player {} disconnected from queue, sending dequeue request for game mode {}", player_id, game_mode);
                self.matchmaker_addr.do_send(DequeuePlayer {
                    player_id,
                    game_mode,
                });
            },
            _ => {}
        }
        info!("MatchmakingSession for player {:?} is stopping.", self.player_id);
        Running::Stop
    }
}

impl Handler<ServerMessage> for MatchmakingSession {
    type Result = ();

    fn handle(&mut self, msg: ServerMessage, ctx: &mut Self::Context) {
        // 서버로부터 메시지를 받으면, 로딩 세션 ID를 상태에 저장합니다.
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
                        
                        info!("Player {} requests queue for {}. Subscribing before enqueuing.", player_id, game_mode);
                        self.player_id = Some(player_id);
                        self.game_mode = Some(game_mode.clone());
                        
                        let redis_client = self.redis_client.clone();
                        let addr = ctx.address().clone();
                        let matchmaker_addr = self.matchmaker_addr.clone();

                        let future = async move {
                            let conn = match redis_client.get_async_connection().await {
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

                            matchmaker_addr.do_send(EnqueuePlayer {
                                player_id,
                                game_mode,
                            });

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
                    Ok(ClientMessage::LoadingComplete { loading_session_id }) => {
                        if let Some(player_id) = self.player_id {
                            info!("Player {} finished loading for session {}", player_id, loading_session_id);
                            self.matchmaker_addr.do_send(HandleLoadingComplete {
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
