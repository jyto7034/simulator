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

use crate::{protocol::ServerMessage as WsServerMessage, AppState, invariants};
use redis::AsyncCommands;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EventKind {
    // Client notification derived events
    Enqueued,
    StartLoading,
    MatchFound,
    Error,

    // Server state events
    QueueSizeChanged,
    LoadingSessionCreated,
    PlayerReady,
    LoadingSessionCompleted,
    LoadingSessionTimeout,
    PlayersRequeued,
    DedicatedSessionCreated,
    DedicatedSessionFailed,
    LoadingSessionCanceled,

    StateViolation,

    Unknown,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct EventStreamMessage {
    // New: typed kind for V2 consumers
    pub kind: EventKind,
    // Backward compatible string field for V1 consumers
    pub event_type: String,
    pub player_id: Option<Uuid>,
    pub timestamp: DateTime<Utc>,
    pub data: serde_json::Value,
    // Optional run_id tagging for test runs
    #[serde(skip_serializing_if = "Option::is_none")]
    pub run_id: Option<String>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct EventStreamQuery {
    pub player_id: Option<Uuid>,
    pub event_type: Option<String>, // V1 compat
    pub kind: Option<EventKind>,    // V2 filter
    pub game_mode: Option<String>,  // state events
    pub session_id: Option<String>, // state events or MatchFound
    pub run_id: Option<String>,     // optional tagging only
}

// EventStream WebSocket Actor
pub struct EventStreamSession {
    pub filter: EventStreamQuery,
    pub redis_conn: ConnectionManager,
    pub redis_url: String,
    pub notification_channel_pattern: String,
    pub state_event_channel_pattern: String,
    pub enable_state_events: bool,
}

impl EventStreamSession {
    pub fn new(
        filter: EventStreamQuery,
        redis_conn: ConnectionManager,
        redis_url: String,
        notification_channel_pattern: String,
        state_event_channel_pattern: String,
        enable_state_events: bool,
    ) -> Self {
        Self {
            filter,
            redis_conn,
            redis_url,
            notification_channel_pattern,
            state_event_channel_pattern,
            enable_state_events,
        }
    }
}

impl Actor for EventStreamSession {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        info!("EventStreamSession started with filter: {:?}", self.filter);

        // Start Redis subscription
        let addr = ctx.address();
        let filter = self.filter.clone();
        let redis_url = self.redis_url.clone();
        let notif_pattern = self.notification_channel_pattern.clone();
        let state_pattern = self.state_event_channel_pattern.clone();
        let enable_state_events = self.enable_state_events;

        ctx.spawn(
            async move {
                // Create a new connection for pubsub
                let client = match redis::Client::open(redis_url) {
                    Ok(c) => c,
                    Err(e) => {
                        error!("Failed to create Redis client for event stream: {}", e);
                        return;
                    }
                };
                let mut pubsub = match client.get_async_connection().await {
                    Ok(conn) => conn.into_pubsub(),
                    Err(e) => {
                        error!("Failed to open Redis connection for event stream: {}", e);
                        return;
                    }
                };

                // Subscribe to all notification channels 
                if let Err(e) = pubsub.psubscribe(&notif_pattern).await {
                    error!("Failed to subscribe to Redis: {}", e);
                    return;
                }
                // Subscribe to state events as well (best-effort)
                if enable_state_events {
                    if let Err(e) = pubsub.psubscribe(&state_pattern).await {
                        error!("Failed to subscribe to state events: {}", e);
                        // Continue with notifications subscription only
                    }
                }

                let mut stream = pubsub.on_message();

                while let Some(msg) = stream.next().await {
                    let channel: String = msg.get_channel_name().to_string();
                    let payload: String = match msg.get_payload() {
                        Ok(p) => p,
                        Err(e) => {
                            warn!("Failed to get Redis pubsub payload: {}", e);
                            continue;
                        }
                    };

                    if channel.starts_with(notif_pattern.trim_end_matches('*')) {
                        // Extract player_id from channel name
                        let player_id = channel
                            .strip_prefix(notif_pattern.trim_end_matches('*'))
                            .and_then(|id_str| Uuid::parse_str(id_str).ok());

                        // Apply player filter
                        if let Some(filter_player_id) = filter.player_id {
                            if player_id != Some(filter_player_id) {
                                continue;
                            }
                        }

                        // Parse message payload into ServerMessage and map to typed event
                        let (kind, event_type, data) = match serde_json::from_str::<WsServerMessage>(&payload) {
                            Ok(WsServerMessage::EnQueued) => (EventKind::Enqueued, "enqueued".to_string(), serde_json::json!({})),
                            Ok(WsServerMessage::StartLoading { loading_session_id }) => (
                                EventKind::StartLoading,
                                "start_loading".to_string(),
                                serde_json::json!({ "loading_session_id": loading_session_id }),
                            ),
                            Ok(WsServerMessage::MatchFound { session_id, server_address }) => (
                                EventKind::MatchFound,
                                "match_found".to_string(),
                                serde_json::json!({ "session_id": session_id, "server_address": server_address }),
                            ),
                            Ok(WsServerMessage::Error { .. }) => (
                                EventKind::Error,
                                "error".to_string(),
                                serde_json::from_str(&payload).unwrap_or_else(|_| serde_json::json!({"raw": payload})),
                            ),
                            Err(_) => (EventKind::Unknown, "unknown".to_string(), serde_json::json!({ "raw": payload })),
                        };

                        // Apply V2 kind filter
                        if let Some(ref k) = filter.kind { if &kind != k { continue; } }
                        // Apply V1 event_type filter
                        if let Some(ref filter_event_type) = filter.event_type { if &event_type != filter_event_type { continue; } }
                        // Apply optional game_mode/session_id filters when available in data
                        if let Some(ref gm) = filter.game_mode {
                            match data.get("game_mode").and_then(|v| v.as_str()) {
                                Some(found) if found == gm => {},
                                _ => { continue; }
                            }
                        }
                        if let Some(ref sid) = filter.session_id {
                            let matches_sid = data.get("session_id").and_then(|v| v.as_str()).map(|s| s == sid).unwrap_or(false)
                                || data.get("loading_session_id").and_then(|v| v.as_str()).map(|s| s == sid).unwrap_or(false);
                            if !matches_sid { continue; }
                        }

                        let event_message = EventStreamMessage {
                            kind,
                            event_type,
                            player_id,
                            timestamp: Utc::now(),
                            data,
                            run_id: filter.run_id.clone(),
                        };

                        addr.do_send(ForwardEvent(event_message));
                        continue;
                    }

                    if enable_state_events && channel.starts_with(state_pattern.trim_end_matches('*')) {
                        // Raw state events: event_type from payload.type, data is full payload
                        let parsed: Result<JsonValue, _> = serde_json::from_str(&payload);
                        let data = parsed.unwrap_or(JsonValue::Null);
                        let event_type_str = data.get("type").and_then(|v| v.as_str()).unwrap_or("unknown").to_string();
                        let kind = match event_type_str.as_str() {
                            "queue_size_changed" => EventKind::QueueSizeChanged,
                            "loading_session_created" => EventKind::LoadingSessionCreated,
                            "player_ready" => EventKind::PlayerReady,
                            "loading_session_completed" => EventKind::LoadingSessionCompleted,
                            "loading_session_timeout" => EventKind::LoadingSessionTimeout,
                            "players_requeued" => EventKind::PlayersRequeued,
                            "dedicated_session_created" => EventKind::DedicatedSessionCreated,
                            "dedicated_session_failed" => EventKind::DedicatedSessionFailed,
                            "loading_session_canceled" => EventKind::LoadingSessionCanceled,
                            "state_violation" => EventKind::StateViolation,
                            _ => EventKind::Unknown,
                        };

                        // Apply filters
                        if let Some(ref k) = filter.kind { if &kind != k { continue; } }
                        if let Some(ref filter_event_type) = filter.event_type { if &event_type_str != filter_event_type { continue; } }
                        if let Some(ref gm) = filter.game_mode {
                            match data.get("game_mode").and_then(|v| v.as_str()) {
                                Some(found) if found == gm => {},
                                _ => { continue; }
                            }
                        }
                        if let Some(ref sid) = filter.session_id {
                            let matches_sid = data.get("session_id").and_then(|v| v.as_str()).map(|s| s == sid).unwrap_or(false);
                            if !matches_sid { continue; }
                        }

                        let event_message = EventStreamMessage { kind, event_type: event_type_str, player_id: None, timestamp: Utc::now(), data, run_id: filter.run_id.clone() };

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
        let event = msg.0.clone();
        // Invariant checks for StartLoading events: ensure player ∈ loading session and status is 'loading'
        if event.kind == EventKind::StartLoading {
            if let (Some(player_id), Some(session_id)) = (
                event.player_id,
                event
                    .data
                    .get("loading_session_id")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string()),
            ) {
                let mut conn = self.redis_conn.clone();
                actix::spawn(async move {
                    let loading_key = format!("loading:{}", session_id);
                    // status check
                    if let Ok(Some(status)) =
                        conn.hget::<_, _, Option<String>>(&loading_key, "status").await
                    {
                        if status.as_str() != "loading" {
                            invariants::emit_violation_kv(
                                &mut conn,
                                "start_loading_wrong_status",
                                &[
                                    ("status", status),
                                    ("session_id", session_id.clone()),
                                    ("player_id", player_id.to_string()),
                                ],
                            )
                            .await;
                        }
                    }
                    // membership check
                    match conn
                        .hexists::<_, _, bool>(&loading_key, player_id.to_string())
                        .await
                    {
                        Ok(false) => {
                            invariants::emit_violation_kv(
                                &mut conn,
                                "start_loading_non_member",
                                &[
                                    ("session_id", session_id.clone()),
                                    ("player_id", player_id.to_string()),
                                ],
                            )
                            .await;
                        }
                        _ => {}
                    }
                });
            }
        }

        let event_json = serde_json::to_string(&event).unwrap_or_else(|e| {
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
    let session = EventStreamSession::new(
        query.into_inner(),
        state.redis_conn_manager.clone(),
        state.settings.redis.url.clone(),
        state.settings.redis.notification_channel_pattern.clone(),
        state.settings.redis.state_event_channel_pattern.clone(),
        state.settings.redis.enable_state_events,
    );

    ws::start(session, &req, stream)
}
