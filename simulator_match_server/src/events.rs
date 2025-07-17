use actix::{Actor, ActorContext, AsyncContext, Handler, Message, StreamHandler, WrapFuture};
use actix_web::{get, web, HttpRequest, HttpResponse, Result};
use actix_web_actors::ws;
use chrono::{DateTime, Utc};
use futures::stream::StreamExt;
use redis::aio::ConnectionManager;
use serde::{Deserialize, Serialize};
use tracing::{error, info, warn};
use uuid::Uuid;

use crate::AppState;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct EventStreamMessage {
    pub event_type: String,
    pub player_id: Option<Uuid>,
    pub timestamp: DateTime<Utc>,
    pub data: serde_json::Value,
}

#[derive(Deserialize, Debug, Clone)]
pub struct EventStreamQuery {
    pub player_id: Option<Uuid>,
    pub event_type: Option<String>,
}

// EventStream WebSocket Actor
pub struct EventStreamSession {
    pub filter: EventStreamQuery,
    pub redis_conn: ConnectionManager,
}

impl EventStreamSession {
    pub fn new(filter: EventStreamQuery, redis_conn: ConnectionManager) -> Self {
        Self { filter, redis_conn }
    }
}

impl Actor for EventStreamSession {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        info!("EventStreamSession started with filter: {:?}", self.filter);

        // Start Redis subscription
        let addr = ctx.address();
        let mut _redis_conn = self.redis_conn.clone();
        let filter = self.filter.clone();

        ctx.spawn(
            async move {
                // Create a new connection for pubsub
                let client = redis::Client::open("redis://127.0.0.1:6379").unwrap();
                let mut pubsub = client.get_async_connection().await.unwrap().into_pubsub();

                // Subscribe to all notification channels
                if let Err(e) = pubsub.psubscribe("notifications:*").await {
                    error!("Failed to subscribe to Redis: {}", e);
                    return;
                }

                let mut stream = pubsub.on_message();

                while let Some(msg) = stream.next().await {
                    let channel: String = msg.get_channel_name().to_string();
                    let payload: String = msg.get_payload().unwrap_or_default();

                    // Extract player_id from channel name
                    let player_id = channel
                        .strip_prefix("notifications:")
                        .and_then(|id_str| Uuid::parse_str(id_str).ok());

                    // Apply filter
                    if let Some(filter_player_id) = filter.player_id {
                        if player_id != Some(filter_player_id) {
                            continue;
                        }
                    }

                    // Parse message payload
                    let event_data: serde_json::Value = serde_json::from_str(&payload)
                        .unwrap_or_else(|_| serde_json::json!({"raw": payload}));

                    let event_message = EventStreamMessage {
                        event_type: "server_message".to_string(),
                        player_id,
                        timestamp: Utc::now(),
                        data: event_data,
                    };

                    addr.do_send(ForwardEvent(event_message));
                }
            }
            .into_actor(self),
        );
    }
}

#[derive(Message)]
#[rtype(result = "()")]
struct ForwardEvent(EventStreamMessage);

impl Handler<ForwardEvent> for EventStreamSession {
    type Result = ();

    fn handle(&mut self, msg: ForwardEvent, ctx: &mut Self::Context) -> Self::Result {
        let event_json = serde_json::to_string(&msg.0).unwrap_or_else(|e| {
            error!("Failed to serialize event: {}", e);
            "{}".to_string()
        });

        ctx.text(event_json);
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for EventStreamSession {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Ping(msg)) => ctx.pong(&msg),
            Ok(ws::Message::Pong(_)) => {}
            Ok(ws::Message::Text(_text)) => {
                // Echo back for heartbeat
                ctx.text("pong");
            }
            Ok(ws::Message::Binary(_)) => {
                warn!("Received binary message in event stream");
            }
            Ok(ws::Message::Close(_)) => {
                info!("EventStreamSession closing");
                ctx.stop();
            }
            Ok(ws::Message::Continuation(_)) => {
                // Handle continuation frames
            }
            Ok(ws::Message::Nop) => {
                // Handle nop frames
            }
            Err(e) => {
                error!("EventStreamSession error: {}", e);
                ctx.stop();
            }
        }
    }
}

#[get("/events/stream")]
pub async fn event_stream_ws(
    req: HttpRequest,
    stream: web::Payload,
    query: web::Query<EventStreamQuery>,
    state: web::Data<AppState>,
) -> Result<HttpResponse> {
    let session = EventStreamSession::new(query.into_inner(), state.redis_conn_manager.clone());

    ws::start(session, &req, stream)
}
