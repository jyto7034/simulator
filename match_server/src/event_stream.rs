use actix::{Actor, ActorContext, AsyncContext, StreamHandler};
use actix_web_actors::ws;
use chrono::Utc;
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::sync::mpsc;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// WebSocket 메시지로 전송할 이벤트 형식
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct EventStreamMessage {
    pub event_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub player_id: Option<Uuid>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub data: serde_json::Value,
}

/// 필터 옵션
#[derive(Debug, Clone)]
pub struct StreamFilters {
    pub run_id: Option<String>,
    pub session_id: Option<String>,
    pub kind: Option<String>,
    pub game_mode: Option<String>,
    pub event_type: Option<String>,
}

impl StreamFilters {
    /// URL 쿼리 파라미터로부터 필터 생성
    pub fn from_query(query: &str) -> Self {
        let mut params = HashMap::new();

        for pair in query.split('&') {
            if let Some((key, value)) = pair.split_once('=') {
                params.insert(key.to_string(), value.to_string());
            }
        }

        Self {
            run_id: params.get("run_id").cloned(),
            session_id: params.get("session_id").cloned(),
            kind: params.get("kind").cloned(),
            game_mode: params.get("game_mode").cloned(),
            event_type: params.get("event_type").cloned(),
        }
    }

    /// 이벤트가 필터를 통과하는지 확인
    fn matches(&self, event: &EventStreamMessage) -> bool {
        // event_type 필터
        if let Some(ref filter_type) = self.event_type {
            if event.event_type != *filter_type {
                return false;
            }
        }

        // game_mode 필터 (data 필드에서 확인)
        if let Some(ref filter_mode) = self.game_mode {
            if let Some(mode) = event.data.get("game_mode").and_then(|v| v.as_str()) {
                if mode != filter_mode {
                    return false;
                }
            }
        }

        true
    }
}

/// WebSocket 세션 Actor
pub struct EventStreamSession {
    /// 필터 설정
    filters: StreamFilters,
    /// 내부 메시지 수신용 채널
    event_rx: Option<mpsc::UnboundedReceiver<EventStreamMessage>>,
}

impl EventStreamSession {
    pub fn new(redis: redis::aio::ConnectionManager, filters: StreamFilters) -> Self {
        let (tx, rx) = mpsc::unbounded_channel();

        // Pub/Sub 구독을 별도 tokio 태스크에서 실행
        if let Some(ref session_id) = filters.session_id {
            let channel = format!("events:test:{}", session_id);
            let filters_clone = filters.clone();

            // ConnectionManager에서 원본 Client를 추출할 수 없으므로
            // 새 Redis 연결을 생성합니다
            tokio::spawn(async move {
                // Redis URL을 환경 변수에서 가져옴
                let redis_url = std::env::var("REDIS_URL")
                    .unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());

                match redis::Client::open(redis_url.as_str()) {
                    Ok(client) => {
                        if let Err(e) = Self::subscribe_pubsub(client, channel, filters_clone, tx).await {
                            error!("Pub/Sub subscription failed: {}", e);
                        }
                    }
                    Err(e) => {
                        error!("Failed to create Redis client for Pub/Sub: {}", e);
                    }
                }
            });
        } else {
            warn!("No session_id filter provided, Pub/Sub will not subscribe");
        }

        Self {
            filters,
            event_rx: Some(rx),
        }
    }

    /// Redis Pub/Sub 구독 (별도 태스크에서 실행)
    async fn subscribe_pubsub(
        client: redis::Client,
        channel: String,
        filters: StreamFilters,
        tx: mpsc::UnboundedSender<EventStreamMessage>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        info!("Starting Pub/Sub subscription on channel: {}", channel);

        // Client에서 비동기 연결 생성 후 PubSub으로 전환
        let conn = client.get_async_connection().await?;
        let mut pubsub = conn.into_pubsub();
        pubsub.subscribe(&channel).await?;

        info!("Successfully subscribed to channel: {}", channel);

        let mut stream = pubsub.on_message();

        // 메시지 수신 루프
        while let Some(msg) = stream.next().await {
            let payload: String = match msg.get_payload() {
                Ok(p) => p,
                Err(e) => {
                    warn!("Failed to get payload: {}", e);
                    continue;
                }
            };

            // JSON 파싱
            let raw_data: serde_json::Value = match serde_json::from_str(&payload) {
                Ok(d) => d,
                Err(e) => {
                    warn!("Failed to parse JSON payload: {}", e);
                    continue;
                }
            };

            // EventStreamMessage 구성
            if let Some(event) = Self::parse_pubsub_message(&raw_data) {
                if filters.matches(&event) {
                    if tx.send(event).is_err() {
                        info!("Event receiver dropped, stopping Pub/Sub");
                        break;
                    }
                }
            }
        }

        info!("Pub/Sub subscription ended for channel: {}", channel);
        Ok(())
    }

    /// Pub/Sub 메시지를 EventStreamMessage로 파싱
    fn parse_pubsub_message(data: &serde_json::Value) -> Option<EventStreamMessage> {
        let event_type = data.get("type")?.as_str()?.to_string();

        let player_id = data
            .get("player_id")
            .and_then(|v| v.as_str())
            .and_then(|s| Uuid::parse_str(s).ok());

        let timestamp = data
            .get("timestamp")
            .and_then(|v| v.as_str())
            .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(Utc::now);

        Some(EventStreamMessage {
            event_type,
            player_id,
            timestamp,
            data: data.clone(),
        })
    }
}

impl Actor for EventStreamSession {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        info!("Event stream WebSocket session started");

        // 채널에서 이벤트 수신하여 WebSocket으로 전송
        if let Some(mut rx) = self.event_rx.take() {
            ctx.add_stream(async_stream::stream! {
                while let Some(event) = rx.recv().await {
                    yield event;
                }
            });
        }
    }

    fn stopped(&mut self, _ctx: &mut Self::Context) {
        info!("Event stream WebSocket session stopped");
    }
}

/// 내부 이벤트를 WebSocket으로 전송하는 Stream Handler
impl StreamHandler<EventStreamMessage> for EventStreamSession {
    fn handle(&mut self, event: EventStreamMessage, ctx: &mut Self::Context) {
        match serde_json::to_string(&event) {
            Ok(json) => {
                ctx.text(json);
            }
            Err(e) => {
                error!("Failed to serialize event: {}", e);
            }
        }
    }
}

/// WebSocket 메시지 핸들러
impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for EventStreamSession {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Ping(msg)) => ctx.pong(&msg),
            Ok(ws::Message::Pong(_)) => {}
            Ok(ws::Message::Text(_)) => {
                // 클라이언트로부터의 텍스트 메시지는 무시 (읽기 전용 스트림)
            }
            Ok(ws::Message::Close(reason)) => {
                debug!("WebSocket close: {:?}", reason);
                ctx.stop();
            }
            Err(e) => {
                error!("WebSocket error: {}", e);
                ctx.stop();
            }
            _ => {}
        }
    }
}
