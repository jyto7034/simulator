use actix::{Actor, ActorContext, AsyncContext, Handler, Message, StreamHandler, WrapFuture};
use actix_web::{get, web, HttpRequest, HttpResponse, Result};
use actix_web_actors::ws;
use chrono::{DateTime, Utc};
use futures::stream::StreamExt;
use redis::aio::ConnectionManager;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use tracing::{error, info, warn};
use uuid::Uuid;

use crate::{protocol::ServerMessage as WsServerMessage, AppState};

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
                // Subscribe to state events as well (best-effort)
                if let Err(e) = pubsub.psubscribe("events:*").await {
                    error!("Failed to subscribe to state events: {}", e);
                    // Continue with notifications subscription only
                }

                let mut stream = pubsub.on_message();

                while let Some(msg) = stream.next().await {
                    let channel: String = msg.get_channel_name().to_string();
                    let payload: String = msg.get_payload().unwrap_or_default();

                    if channel.starts_with("notifications:") {
                        // Extract player_id from channel name
                        let player_id = channel
                            .strip_prefix("notifications:")
                            .and_then(|id_str| Uuid::parse_str(id_str).ok());

                        // Apply player filter
                        if let Some(filter_player_id) = filter.player_id {
                            if player_id != Some(filter_player_id) {
                                continue;
                            }
                        }

                        // Parse message payload into ServerMessage and map to event type
                        let (event_type, data) = match serde_json::from_str::<WsServerMessage>(&payload) {
                            Ok(WsServerMessage::EnQueued) => ("enqueued".to_string(), serde_json::json!({})),
                            Ok(WsServerMessage::StartLoading { loading_session_id }) => (
                                "start_loading".to_string(),
                                serde_json::json!({ "loading_session_id": loading_session_id }),
                            ),
                            Ok(WsServerMessage::MatchFound { session_id, server_address }) => (
                                "match_found".to_string(),
                                serde_json::json!({ "session_id": session_id, "server_address": server_address }),
                            ),
                            Ok(WsServerMessage::Error { message }) => (
                                "error".to_string(),
                                serde_json::json!({ "message": message }),
                            ),
                            Err(_) => ("unknown".to_string(), serde_json::json!({ "raw": payload })),
                        };

                        // Apply event_type filter, if set
                        if let Some(ref filter_event_type) = filter.event_type {
                            if &event_type != filter_event_type {
                                continue;
                            }
                        }

                        let event_message = EventStreamMessage {
                            event_type,
                            player_id,
                            timestamp: Utc::now(),
                            data,
                        };

                        addr.do_send(ForwardEvent(event_message));
                        continue;
                    }

                    if channel.starts_with("events:") {
                        // Raw state events: event_type from payload.type, data is full payload
                        let parsed: Result<JsonValue, _> = serde_json::from_str(&payload);
                        let data = parsed.unwrap_or(JsonValue::Null);
                        let event_type = data
                            .get("type")
                            .and_then(|v| v.as_str())
                            .unwrap_or("unknown")
                            .to_string();

                        // Apply event_type filter, if set
                        if let Some(ref filter_event_type) = filter.event_type {
                            if &event_type != filter_event_type {
                                continue;
                            }
                        }

                        let event_message = EventStreamMessage {
                            event_type,
                            player_id: None,
                            timestamp: Utc::now(),
                            data,
                        };

                        addr.do_send(ForwardEvent(event_message));
                        continue;
                    }
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
