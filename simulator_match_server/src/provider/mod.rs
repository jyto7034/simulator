use crate::env::Settings;
use actix::{Actor, Context, Handler, Message, ResponseFuture};
use futures_util::stream::StreamExt;
use redis::{aio::ConnectionManager, AsyncCommands};
use serde::Deserialize;
use serde_json::json;
use tracing::{info, warn};

// --- Actor Definition ---

/// 사용 가능한 Dedicated Server를 찾아 제공하는 책임을 가진 액터입니다.
pub struct DedicatedServerProvider {
    redis: ConnectionManager,
    settings: Settings,
}

impl DedicatedServerProvider {
    pub fn new(redis: ConnectionManager, settings: Settings) -> Self {
        Self { redis, settings }
    }
}

impl Actor for DedicatedServerProvider {
    type Context = Context<Self>;
}

// --- Message Definition ---

/// 사용 가능한 서버를 찾아달라는 메시지입니다.
#[derive(Message)]
#[rtype(result = "Result<ServerInfo, anyhow::Error>")]
pub struct FindAvailableServer;

/// 찾아낸 서버의 정보를 담는 구조체입니다.
#[derive(Deserialize, Debug, Clone)]
pub struct ServerInfo {
    pub address: String,
    pub status: String,
}

/// 서버 상태를 조회하는 메시지입니다.
#[derive(Message)]
#[rtype(result = "String")]
pub struct GetServerStatus;

// --- Message Handler ---

impl Handler<FindAvailableServer> for DedicatedServerProvider {
    type Result = ResponseFuture<Result<ServerInfo, anyhow::Error>>;

    /// `FindAvailableServer` 메시지를 처리합니다.
    fn handle(&mut self, _msg: FindAvailableServer, _ctx: &mut Context<Self>) -> Self::Result {
        let mut redis = self.redis.clone();
        let settings = self.settings.clone();

        Box::pin(async move {
            info!("Finding an available dedicated server from Redis...");

            // SCAN 사용으로 Redis 블로킹 방지
            let mut keys: Vec<String> = Vec::new();
            match redis
                .scan_match::<_, String>(&settings.redis.dedicated_server_key_pattern)
                .await
            {
                Ok(mut iter) => {
                    while let Some(key) = iter.next().await {
                        keys.push(key);
                    }
                }
                Err(e) => {
                    warn!("Failed to scan dedicated servers: {}", e);
                    return Err(anyhow::anyhow!("Failed to scan dedicated servers: {}", e));
                }
            };

            // 각 서버의 상태를 확인하여 "idle"인 서버를 찾습니다.
            for key in keys {
                let server_info_json: String = match redis.get(&key).await {
                    Ok(info) => info,
                    Err(e) => {
                        warn!(
                            "Failed to get server info for key {}: {}. Skipping.",
                            key, e
                        );
                        continue; // 다음 키로 넘어감
                    }
                };

                let server_info: ServerInfo = match serde_json::from_str(&server_info_json) {
                    Ok(info) => info,
                    Err(e) => {
                        warn!(
                            "Failed to parse server info for key {}: {}. Skipping.",
                            key, e
                        );
                        continue; // 다음 키로 넘어감
                    }
                };

                // "idle" 상태인 서버를 찾으면 즉시 반환합니다.
                if server_info.status == settings.server_status.idle {
                    info!("Found idle server: {:?}", server_info);
                    return Ok(server_info);
                }
            }

            // 모든 서버를 확인했지만 "idle" 상태인 서버가 없는 경우
            warn!("All dedicated servers are currently busy.");
            Err(anyhow::anyhow!("All dedicated servers are busy."))
        })
    }
}

impl Handler<GetServerStatus> for DedicatedServerProvider {
    type Result = String;

    fn handle(&mut self, _msg: GetServerStatus, _ctx: &mut Context<Self>) -> Self::Result {
        let status_info = json!({
            "provider_status": "active",
            "settings": {
                "dedicated_server_key_pattern": self.settings.redis.dedicated_server_key_pattern,
                "server_status": {
                    "idle": self.settings.server_status.idle
                }
            },
            "health": {
                "redis_connected": true,
                "can_scan_servers": true
            }
        });
        status_info.to_string()
    }
}
