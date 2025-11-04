use actix::Addr;
use backoff::{backoff::Backoff, ExponentialBackoff};
use futures::StreamExt;
use redis::Client as RedisClient;
use std::sync::Arc;
use tokio_util::sync::CancellationToken;
use tracing::{error, info, warn};
use uuid::Uuid;

use crate::{
    game::load_balance_actor::{messages::RouteToPlayer, LoadBalanceActor},
    shared::{circuit_breaker::CircuitBreaker, protocol::ServerMessage},
};

pub async fn spawn_redis_subscribers(
    redis_client: RedisClient,
    pod_id: String,
    load_balance_addr: Addr<LoadBalanceActor>,
    shutdown_token: CancellationToken,
    circuit_breaker: Arc<CircuitBreaker>,
) {
    // match_result 구독 (매칭 결과 + 배틀 결과 포함)
    let redis_client_clone = redis_client.clone();
    let pod_id_clone = pod_id.clone();
    let load_balance_addr_clone = load_balance_addr.clone();
    let shutdown_token_clone = shutdown_token.clone();
    let circuit_breaker_clone = circuit_breaker.clone();
    tokio::spawn(async move {
        subscribe_match_result_channel(
            redis_client_clone,
            pod_id_clone,
            load_balance_addr_clone,
            shutdown_token_clone,
            circuit_breaker_clone,
        )
        .await;
    });

    // game_message 구독 (크로스 Pod 메시지 수신)
    let redis_client_clone = redis_client.clone();
    let pod_id_clone = pod_id.clone();
    let load_balance_addr_clone = load_balance_addr.clone();
    let shutdown_token_clone = shutdown_token.clone();
    let circuit_breaker_clone = circuit_breaker.clone();
    tokio::spawn(async move {
        subscribe_game_message_channel(
            redis_client_clone,
            pod_id_clone,
            load_balance_addr_clone,
            shutdown_token_clone,
            circuit_breaker_clone,
        )
        .await;
    });

    info!("Redis Pub/Sub subscribers started for pod: {}", pod_id);
    info!("  - match_result channel");
    info!("  - pod:{}:game_message channel", pod_id);
}

/// match_result 채널 구독
async fn subscribe_match_result_channel(
    redis_client: RedisClient,
    pod_id: String,
    load_balance_addr: Addr<LoadBalanceActor>,
    shutdown_token: CancellationToken,
    circuit_breaker: Arc<CircuitBreaker>,
) {
    let channel = "match_result";
    let channel_name = format!("pod:{}:match_result", pod_id);
    let mut backoff = ExponentialBackoff::default();

    loop {
        if shutdown_token.is_cancelled() {
            info!("[{}] Shutting down subscriber", pod_id);
            break;
        }

        // Circuit Breaker 체크 (shutdown과 함께)
        if circuit_breaker.is_open() {
            warn!(
                "[{}] Circuit breaker is open for {} subscriber, waiting...",
                pod_id, channel_name
            );
            tokio::select! {
                _ = tokio::time::sleep(tokio::time::Duration::from_secs(5)) => {},
                _ = shutdown_token.cancelled() => {
                    info!("[{}] Shutdown during circuit breaker cooldown for {}", pod_id, channel_name);
                    return;
                }
            }
            continue;
        }

        match redis_client.get_async_connection().await {
            Ok(conn) => {
                let mut pubsub = conn.into_pubsub();

                match pubsub.subscribe(&channel).await {
                    Ok(_) => {
                        circuit_breaker.record_success();
                        backoff.reset();
                        info!("Subscribed to Redis channel: {}", channel);

                        let mut message_stream = pubsub.on_message();

                        // 메시지 수신 루프
                        loop {
                            tokio::select! {
                                _ = shutdown_token.cancelled() => {
                                    info!("Shutting down {} subscriber", channel_name);
                                    return;
                                }

                                msg_result = message_stream.next() => {
                                    if let Some(msg) = msg_result {
                                        match msg.get_payload::<String>() {
                                            Ok(payload) => {
                                                handle_match_result_message(
                                                    payload,
                                                    &load_balance_addr,
                                                ).await;
                                            }
                                            Err(e) => {
                                                error!("Failed to get payload from {}: {}", channel_name, e);
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    Err(e) => {
                        circuit_breaker.record_failure();
                        error!("Failed to subscribe to {}: {}", channel, e);

                        let delay = backoff
                            .next_backoff()
                            .unwrap_or(tokio::time::Duration::from_secs(60));
                        warn!("Retrying subscription to {} in {:?}", channel_name, delay);

                        tokio::select! {
                            _ = tokio::time::sleep(delay) => {},
                            _ = shutdown_token.cancelled() => {
                                info!("Shutdown during backoff for {}", channel_name);
                                return;
                            }
                        }
                    }
                }
            }
            Err(e) => {
                circuit_breaker.record_failure();
                error!("Failed to connect to Redis for {}: {}", channel_name, e);

                let delay = backoff
                    .next_backoff()
                    .unwrap_or(tokio::time::Duration::from_secs(60));
                warn!("Retrying connection for {} in {:?}", channel_name, delay);

                tokio::select! {
                    _ = tokio::time::sleep(delay) => {},
                    _ = shutdown_token.cancelled() => {
                        info!("Shutdown during backoff for {}", channel_name);
                        return;
                    }
                }
            }
        }
    }
}

/// game_message 채널 구독 (크로스 Pod 메시지 수신)
async fn subscribe_game_message_channel(
    redis_client: RedisClient,
    pod_id: String,
    load_balance_addr: Addr<LoadBalanceActor>,
    shutdown_token: CancellationToken,
    circuit_breaker: Arc<CircuitBreaker>,
) {
    let channel = format!("pod:{}:game_message", pod_id);
    let channel_name = channel.clone();
    let mut backoff = ExponentialBackoff::default();

    loop {
        if shutdown_token.is_cancelled() {
            info!("[{}] Shutting down game_message subscriber", pod_id);
            break;
        }

        // Circuit Breaker 체크 (shutdown과 함께)
        if circuit_breaker.is_open() {
            warn!(
                "[{}] Circuit breaker is open for {} subscriber, waiting...",
                pod_id, channel_name
            );
            tokio::select! {
                _ = tokio::time::sleep(tokio::time::Duration::from_secs(5)) => {},
                _ = shutdown_token.cancelled() => {
                    info!("[{}] Shutdown during circuit breaker cooldown for {}", pod_id, channel_name);
                    return;
                }
            }
            continue;
        }

        match redis_client.get_async_connection().await {
            Ok(conn) => {
                let mut pubsub = conn.into_pubsub();

                match pubsub.subscribe(&channel).await {
                    Ok(_) => {
                        circuit_breaker.record_success();
                        backoff.reset();
                        info!("Subscribed to Redis channel: {}", channel);

                        let mut message_stream = pubsub.on_message();

                        // 메시지 수신 루프
                        loop {
                            tokio::select! {
                                _ = shutdown_token.cancelled() => {
                                    info!("Shutting down {} subscriber", channel_name);
                                    return;
                                }

                                msg_result = message_stream.next() => {
                                    if let Some(msg) = msg_result {
                                        match msg.get_payload::<String>() {
                                            Ok(payload) => {
                                                handle_game_message(
                                                    payload,
                                                    &load_balance_addr,
                                                ).await;
                                            }
                                            Err(e) => {
                                                error!("Failed to get payload from {}: {}", channel_name, e);
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    Err(e) => {
                        circuit_breaker.record_failure();
                        error!("Failed to subscribe to {}: {}", channel, e);

                        let delay = backoff
                            .next_backoff()
                            .unwrap_or(tokio::time::Duration::from_secs(60));
                        warn!("Retrying subscription to {} in {:?}", channel_name, delay);

                        tokio::select! {
                            _ = tokio::time::sleep(delay) => {},
                            _ = shutdown_token.cancelled() => {
                                info!("Shutdown during backoff for {}", channel_name);
                                return;
                            }
                        }
                    }
                }
            }
            Err(e) => {
                circuit_breaker.record_failure();
                error!("Failed to connect to Redis for {}: {}", channel_name, e);

                let delay = backoff
                    .next_backoff()
                    .unwrap_or(tokio::time::Duration::from_secs(60));
                warn!("Retrying connection for {} in {:?}", channel_name, delay);

                tokio::select! {
                    _ = tokio::time::sleep(delay) => {},
                    _ = shutdown_token.cancelled() => {
                        info!("Shutdown during backoff for {}", channel_name);
                        return;
                    }
                }
            }
        }
    }
}

/// 크로스 Pod 게임 메시지 처리
async fn handle_game_message(payload: String, load_balance_addr: &Addr<LoadBalanceActor>) {
    // JSON 파싱
    #[derive(serde::Deserialize)]
    struct GameMessagePayload {
        player_id: String,
        message: ServerMessage,
    }

    let parsed: GameMessagePayload = match serde_json::from_str(&payload) {
        Ok(p) => p,
        Err(e) => {
            error!("Failed to parse GameMessage payload: {}", e);
            return;
        }
    };

    let player_uuid = match Uuid::parse_str(&parsed.player_id) {
        Ok(uuid) => uuid,
        Err(e) => {
            error!("Invalid player_id in GameMessage: {}", e);
            return;
        }
    };

    info!("Cross-pod message received for player {}", player_uuid);

    // LoadBalanceActor를 통해 PlayerGameActor에 라우팅
    load_balance_addr.do_send(RouteToPlayer {
        player_id: player_uuid,
        message: parsed.message,
    });
}

/// 매칭 결과 메시지 처리 (배틀 결과 포함)
async fn handle_match_result_message(payload: String, load_balance_addr: &Addr<LoadBalanceActor>) {
    // JSON 파싱
    let result: MatchResult = match serde_json::from_str(&payload) {
        Ok(r) => r,
        Err(e) => {
            error!("Failed to parse MatchResult: {}", e);
            return;
        }
    };

    info!("Match result received for player {}", result.player_id);

    // LoadBalanceActor를 통해 PlayerGameActor에 라우팅
    // TODO: MatchResult에서 battle 정보를 추출해서 ServerMessage에 포함
    load_balance_addr.do_send(RouteToPlayer {
        player_id: result.player_id,
        message: ServerMessage::MatchFound {
            winner_id: "TODO".to_string(),
            opponent_id: "TODO".to_string(),
            battle_data: None,
        },
    });
}

// 메시지 타입 정의
// TODO: battle_result 정보를 포함하도록 확장 필요
#[derive(serde::Deserialize)]
struct MatchResult {
    player_id: Uuid,
}
