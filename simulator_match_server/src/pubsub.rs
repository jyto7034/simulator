use actix::{
    Actor, Addr, AsyncContext, Context, ContextFutureSpawner, Handler, Message, WrapFuture,
};
use futures_util::stream::StreamExt;
use redis::Client as RedisClient;
use std::collections::HashMap;
use std::time::Duration;
use tracing::{error, info, warn};
use uuid::Uuid;

use crate::protocol::ServerMessage;
use crate::ws_session::MatchmakingSession;

const RECONNECT_DELAY: Duration = Duration::from_secs(5);

// --- Messages for this module ---
#[derive(Message)]
#[rtype(result = "()")]
struct Connect;

// --- SubscriptionManager Actor ---

/// Manages the mapping between player_id and their WebSocket session actor address.
pub struct SubscriptionManager {
    sessions: HashMap<Uuid, Addr<MatchmakingSession>>,
}

impl SubscriptionManager {
    pub fn new() -> Self {
        Self {
            sessions: HashMap::new(),
        }
    }
}

impl Actor for SubscriptionManager {
    type Context = Context<Self>;
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct Register {
    pub player_id: Uuid,
    pub addr: Addr<MatchmakingSession>,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct Deregister {
    pub player_id: Uuid,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct ForwardMessage {
    pub player_id: Uuid,
    pub message: ServerMessage,
}

impl Handler<Register> for SubscriptionManager {
    type Result = ();
    fn handle(&mut self, msg: Register, _ctx: &mut Context<Self>) -> Self::Result {
        info!("Player {} registered for notifications.", msg.player_id);
        self.sessions.insert(msg.player_id, msg.addr);
    }
}

impl Handler<Deregister> for SubscriptionManager {
    type Result = ();
    fn handle(&mut self, msg: Deregister, _ctx: &mut Context<Self>) -> Self::Result {
        info!("Player {} deregistered.", msg.player_id);
        self.sessions.remove(&msg.player_id);
    }
}

impl Handler<ForwardMessage> for SubscriptionManager {
    type Result = ();
    fn handle(&mut self, msg: ForwardMessage, _ctx: &mut Context<Self>) -> Self::Result {
        if let Some(recipient_addr) = self.sessions.get(&msg.player_id) {
            recipient_addr.do_send(msg.message);
        } else {
            warn!(
                "Could not find session for player {} to forward message.",
                msg.player_id
            );
        }
    }
}

// --- RedisSubscriber Actor ---

pub struct RedisSubscriber {
    redis_client: RedisClient,
    manager_addr: Addr<SubscriptionManager>,
}

impl RedisSubscriber {
    pub fn new(redis_client: RedisClient, manager_addr: Addr<SubscriptionManager>) -> Self {
        Self {
            redis_client,
            manager_addr,
        }
    }

    fn connect_and_subscribe(&self, ctx: &mut Context<Self>) {
        info!("Attempting to connect and subscribe to Redis...");
        let client = self.redis_client.clone();
        let manager = self.manager_addr.clone();
        let self_addr = ctx.address();

        async move {
            let conn = match client.get_async_connection().await {
                Ok(c) => {
                    info!("Successfully connected to Redis.");
                    c
                }
                Err(e) => {
                    error!("RedisSubscriber failed to get connection: {}", e);
                    self_addr.do_send(Connect); // Trigger reconnect
                    return;
                }
            };
            let mut pubsub = conn.into_pubsub();
            if let Err(e) = pubsub.psubscribe("notifications:*").await {
                error!("RedisSubscriber failed to psubscribe: {}", e);
                self_addr.do_send(Connect); // Trigger reconnect
                return;
            }
            info!("Successfully subscribed to 'notifications:*'");

            let mut stream = pubsub.on_message();
            while let Some(msg) = stream.next().await {
                let channel: String = msg.get_channel_name().to_string();
                let payload: String = match msg.get_payload() {
                    Ok(p) => p,
                    Err(_) => continue,
                };

                if let Some(player_id_str) = channel.strip_prefix("notifications:") {
                    if let Ok(player_id) = Uuid::parse_str(player_id_str) {
                        if let Ok(server_msg) = serde_json::from_str::<ServerMessage>(&payload) {
                            manager.do_send(ForwardMessage {
                                player_id,
                                message: server_msg,
                            });
                        }
                    }
                }
            }
            warn!("Redis Pub/Sub stream ended. Attempting to reconnect...");
            self_addr.do_send(Connect); // Trigger reconnect
        }
        .into_actor(self)
        .wait(ctx);
    }
}

impl Actor for RedisSubscriber {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Context<Self>) {
        info!("RedisSubscriber actor started.");
        self.connect_and_subscribe(ctx);
    }
}

impl Handler<Connect> for RedisSubscriber {
    type Result = ();

    fn handle(&mut self, _msg: Connect, ctx: &mut Context<Self>) -> Self::Result {
        info!("Reconnect message received. Waiting for a delay...");
        ctx.run_later(RECONNECT_DELAY, |act, ctx| {
            act.connect_and_subscribe(ctx);
        });
    }
}
