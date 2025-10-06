use actix::{Actor, ActorContext, ActorFutureExt, AsyncContext, Handler, Message, StreamHandler};
use actix_web_actors::ws;
use chrono::Utc;
use redis::streams::{StreamReadOptions, StreamReadReply};
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;
use tracing::{debug, error, info};
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
    /// Redis connection
    redis: redis::aio::ConnectionManager,
    /// 필터 설정
    filters: StreamFilters,
    /// 마지막으로 읽은 스트림 ID (각 스트림별)
    last_ids: HashMap<String, String>,
}

impl EventStreamSession {
    pub fn new(redis: redis::aio::ConnectionManager, filters: StreamFilters) -> Self {
        Self {
            redis,
            filters,
            last_ids: HashMap::new(),
        }
    }

    /// Redis Stream에서 이벤트 읽기 (업데이트된 last_ids도 반환)
    async fn read_events(
        &self,
    ) -> Result<(Vec<EventStreamMessage>, HashMap<String, String>), String> {
        let mut events = Vec::new();
        let mut updated_last_ids = self.last_ids.clone();

        // 구독할 스트림 키 결정
        let stream_keys = self.get_stream_keys();

        if stream_keys.is_empty() {
            // 필터가 없으면 모든 이벤트 스트림 읽기 (비효율적이므로 기본 패턴 사용)
            // 현재는 test session 기반만 지원
            return Ok((events, updated_last_ids));
        }

        // 각 스트림에서 이벤트 읽기
        let mut redis = self.redis.clone();
        for stream_key in stream_keys {
            let last_id = updated_last_ids
                .get(&stream_key)
                .map(|s| s.as_str())
                .unwrap_or("0");

            // XREAD COUNT 100 BLOCK 1000 STREAMS {stream_key} {last_id}
            let opts = StreamReadOptions::default()
                .count(100)
                .block(1000); // 1초 블록

            let result: Result<StreamReadReply, redis::RedisError> = redis
                .xread_options(&[&stream_key], &[last_id], &opts)
                .await;

            match result {
                Ok(reply) => {
                    for stream_key_data in &reply.keys {
                        for stream_id_data in &stream_key_data.ids {
                            // 스트림 ID 업데이트
                            updated_last_ids
                                .insert(stream_key.clone(), stream_id_data.id.clone());

                            // 이벤트 파싱
                            if let Some(event) = self.parse_stream_entry(&stream_id_data.map) {
                                // 필터 적용
                                if self.filters.matches(&event) {
                                    events.push(event);
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    if !e.is_timeout() {
                        error!("Failed to read from Redis stream {}: {}", stream_key, e);
                    }
                    // Timeout은 정상 동작 (새 이벤트 없음)
                }
            }
        }

        Ok((events, updated_last_ids))
    }

    /// 구독할 스트림 키 목록 반환
    fn get_stream_keys(&self) -> Vec<String> {
        let mut keys = Vec::new();

        // session_id 필터가 있으면 해당 스트림만 구독
        if let Some(ref session_id) = self.filters.session_id {
            keys.push(format!("events:test:{}", session_id));
        }

        keys
    }

    /// Redis Stream 엔트리를 EventStreamMessage로 파싱
    fn parse_stream_entry(
        &self,
        map: &HashMap<String, redis::Value>,
    ) -> Option<EventStreamMessage> {
        // "type" 필드 추출
        let event_type = map
            .get("type")
            .and_then(|v| match v {
                redis::Value::Data(bytes) => String::from_utf8(bytes.clone()).ok(),
                _ => None,
            })?
            .to_string();

        // "player_id" 필드 추출 (선택적)
        let player_id = map
            .get("player_id")
            .and_then(|v| match v {
                redis::Value::Data(bytes) => String::from_utf8(bytes.clone()).ok(),
                _ => None,
            })
            .and_then(|s| Uuid::parse_str(&s).ok());

        // 나머지 필드들을 data로 변환
        let mut data = serde_json::Map::new();
        for (key, value) in map {
            if key == "type" {
                continue; // 이미 event_type으로 사용
            }

            let json_value = match value {
                redis::Value::Data(bytes) => {
                    if let Ok(s) = String::from_utf8(bytes.clone()) {
                        serde_json::Value::String(s)
                    } else {
                        continue;
                    }
                }
                redis::Value::Int(i) => serde_json::Value::Number((*i).into()),
                _ => continue,
            };

            data.insert(key.clone(), json_value);
        }

        Some(EventStreamMessage {
            event_type,
            player_id,
            timestamp: Utc::now(),
            data: serde_json::Value::Object(data),
        })
    }
}

impl Actor for EventStreamSession {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        info!("Event stream WebSocket session started");

        // 주기적으로 Redis Stream 폴링
        ctx.run_interval(Duration::from_millis(100), |act, ctx| {
            let redis = act.redis.clone();
            let filters = act.filters.clone();
            let last_ids = act.last_ids.clone();

            let fut = async move {
                let temp_session = EventStreamSession {
                    redis,
                    filters,
                    last_ids,
                };
                temp_session.read_events().await
            };

            ctx.spawn(
                actix::fut::wrap_future::<_, EventStreamSession>(fut).map(
                    |result, act, ctx| match result {
                        Ok((events, updated_last_ids)) => {
                            // last_ids 업데이트
                            act.last_ids = updated_last_ids;

                            // 이벤트 전송
                            for event in events {
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
                        Err(e) => {
                            error!("Failed to read events: {}", e);
                        }
                    },
                ),
            );
        });
    }

    fn stopped(&mut self, _ctx: &mut Self::Context) {
        info!("Event stream WebSocket session stopped");
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

/// Heartbeat 메시지
#[derive(Message)]
#[rtype(result = "()")]
struct Heartbeat;

impl Handler<Heartbeat> for EventStreamSession {
    type Result = ();

    fn handle(&mut self, _msg: Heartbeat, ctx: &mut Self::Context) {
        ctx.ping(b"");
    }
}
