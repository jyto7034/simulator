use actix::Addr;
use redis::{aio::ConnectionManager, AsyncCommands};
use tracing::warn;
use uuid::Uuid;

use crate::{
    protocol::ServerMessage,
    subscript::{messages::ForwardServerMessage, SubScriptionManager},
    RETRY_CONFIG,
};

/// 플레이어에게 직접 전달 + Redis 발행으로 메시지를 보냅니다.
/// Redis 발행은 외부 관측성을 위함입니다.
pub async fn send_message_to_player(
    subscription_addr: Addr<SubScriptionManager>,
    redis: &mut ConnectionManager,
    player_id: Uuid,
    message: ServerMessage,
) {
    // 직접 전달
    send_direct_message(&subscription_addr, player_id, &message).await;

    // Redis로 발행 (관측성용)
    publish_to_redis_with_retry(redis, player_id, message).await;
}

/// 플레이어에게 직접 메시지 전달 (WebSocket)
async fn send_direct_message(
    subscription_addr: &Addr<SubScriptionManager>,
    player_id: Uuid,
    message: &ServerMessage,
) {
    let backoff = RETRY_CONFIG
        .read()
        .await
        .as_ref()
        .expect("Retry config not initialized")
        .clone();

    let subscription_addr = subscription_addr.clone();
    let message = message.clone();

    let result = backoff::future::retry(backoff, move || {
        let subscription_addr = subscription_addr.clone();
        let message = message.clone();

        async move {
            subscription_addr
                .send(ForwardServerMessage { player_id, message })
                .await
                .map_err(|e| {
                    warn!(
                        "Direct message attempt failed for player {}: {:?}",
                        player_id, e
                    );
                    backoff::Error::Transient {
                        err: "Transient",
                        retry_after: None,
                    }
                })
        }
    })
    .await;

    if let Err(e) = result {
        warn!(
            "Direct message delivery permanently failed for player {}: {:?}",
            player_id, e
        );
    }
}

/// Redis로 메시지 발행 (재시도 포함)
async fn publish_to_redis_with_retry(
    redis: &mut ConnectionManager,
    player_id: Uuid,
    message: ServerMessage,
) {
    let backoff = RETRY_CONFIG
        .read()
        .await
        .as_ref()
        .expect("Retry config not initialized")
        .clone();

    let redis_conn = redis.clone();
    let result = backoff::future::retry(backoff, move || {
        let mut redis_conn = redis_conn.clone();
        let message = message.clone();

        async move {
            let channel = format!("notification:{}", player_id);
            let payload = serde_json::to_string(&message).map_err(|e| {
                warn!(
                    "Failed to serialize message for player {}: {}",
                    player_id, e
                );
                backoff::Error::Permanent("Permanent")
            })?;

            redis_conn
                .publish::<_, _, ()>(&channel, &payload)
                .await
                .map_err(|e| {
                    warn!("Failed to publish to Redis for player {}: {}", player_id, e);
                    backoff::Error::Transient {
                        err: "Transient",
                        retry_after: None,
                    }
                })
        }
    })
    .await;

    if let Err(e) = result {
        warn!(
            "Redis publish permanently failed for player {}: {:?}",
            player_id, e
        );
    }
}
