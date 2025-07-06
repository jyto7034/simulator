use actix::{Actor, Context, Handler, Message, ResponseFuture};
use redis::{aio::ConnectionManager, AsyncCommands};
use serde::Deserialize;
use tracing::{info, warn};

// --- Actor Definition ---

/// 사용 가능한 Dedicated Server를 찾아 제공하는 책임을 가진 액터입니다.
pub struct DedicatedServerProvider {
    redis: ConnectionManager,
}

impl DedicatedServerProvider {
    pub fn new(redis: ConnectionManager) -> Self {
        Self { redis }
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

// --- Message Handler ---

impl Handler<FindAvailableServer> for DedicatedServerProvider {
    type Result = ResponseFuture<Result<ServerInfo, anyhow::Error>>;

    /// `FindAvailableServer` 메시지를 처리합니다.
    fn handle(&mut self, _msg: FindAvailableServer, _ctx: &mut Context<Self>) -> Self::Result {
        let mut redis = self.redis.clone();

        Box::pin(async move {
            info!("Finding an available dedicated server from Redis...");

            // 1. 모든 서버 키를 가져옵니다.
            let server_keys: Vec<String> = redis.keys("dedicated_server:*").await?;
            if server_keys.is_empty() {
                warn!("No dedicated server instances found in Redis.");
                return Err(anyhow::anyhow!("No dedicated server instances registered."));
            }

            // 2. 각 서버의 상태를 확인하여 "idle"인 서버를 찾습니다.
            for key in server_keys {
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

                // 3. "idle" 상태인 서버를 찾으면 즉시 반환합니다.
                if server_info.status == "idle" {
                    info!("Found idle server: {:?}", server_info);
                    return Ok(server_info);
                }
            }

            // 4. 모든 서버를 확인했지만 "idle" 상태인 서버가 없는 경우
            warn!("All dedicated servers are currently busy.");
            Err(anyhow::anyhow!("All dedicated servers are busy."))
        })
    }
}
