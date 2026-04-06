use actix::Addr;
use redis::{aio::ConnectionManager, AsyncCommands};
use std::sync::Arc;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::{
    game::{
        load_balance_actor::{messages::RouteToGamePlayer, LoadBalanceActor},
        player_game_actor::state::legacy_server_message_to_unity,
    },
    matchmaking::{
        matchmaker::{operations::try_match::PlayerCandidate, MatchmakerDeps},
        subscript::{messages::ForwardServerMessage, SubScriptionManager},
    },
    shared::{metrics::MetricsCtx, protocol::ServerMessage, redis_events},
    RETRY_CONFIG,
};

use metrics::{MESSAGES_ROUTED_CROSS_POD_TOTAL, MESSAGES_ROUTED_SAME_POD_TOTAL};

/// 메시지 라우팅에 필요한 의존성
pub struct MessageRoutingDeps {
    /// 레거시 경로 (test_client)
    pub subscription_addr: Addr<SubScriptionManager>,

    /// 신규 경로 (Unity client)
    pub load_balance_addr: Option<Addr<LoadBalanceActor>>,

    /// Redis 연결
    pub redis: ConnectionManager,

    /// 메트릭
    pub metrics: Arc<MetricsCtx>,
}

impl From<&MatchmakerDeps> for MessageRoutingDeps {
    fn from(deps: &MatchmakerDeps) -> Self {
        Self {
            subscription_addr: deps.subscription_addr.clone(),
            load_balance_addr: Some(deps.load_balance_addr.clone()),
            redis: deps.redis.clone(),
            metrics: deps.metrics.clone(),
        }
    }
}

/// 플레이어에게 메시지 전달 (Pod 구분 자동)
///
/// # Arguments
/// * `player` - 대상 플레이어 정보 (pod_id 포함)
/// * `message` - 전달할 서버 메시지
/// * `deps` - 라우팅 의존성
///
pub async fn send_message_to_player(
    player: &PlayerCandidate,
    message: ServerMessage,
    deps: &MessageRoutingDeps,
) {
    let player_uuid = match player.player_uuid() {
        Ok(uuid) => uuid,
        Err(_) => {
            error!("Invalid player_id format: {}", player.player_id);
            return;
        }
    };

    // Pod 구분 처리
    if player.is_same_pod() {
        info!("Routing to same-pod player {}", player.player_id);
        route_to_same_pod(player_uuid, &player.player_id, &message, deps).await;
    } else {
        info!(
            "Routing to cross-pod player {} (pod: {})",
            player.player_id, player.pod_id
        );
        route_to_cross_pod(player, &message, deps).await;
    }

    // 레거시 경로 (test_client 호환)
    send_direct_message_legacy(&deps.subscription_addr, player_uuid, &message).await;

    // 테스트 이벤트 발행 (metadata에 test_session_id 있을 때만)
    if let Ok(metadata_str) = serde_json::to_string(&player.metadata) {
        redis_events::try_publish_test_event(
            &mut deps.redis.clone(),
            &metadata_str,
            &message.to_string(),
            PlayerCandidate::current_pod_id(),
            vec![("player_id", player.player_id.clone())],
        )
        .await;
    }
}

/// Same-pod 플레이어에게 메시지 전달 (player_id 기반, 레거시 호환)
///
/// # Note
/// - enqueue/dequeue 등 이미 연결된 플레이어용
/// - 항상 same-pod로 처리됨
pub async fn send_message_to_player_by_id(
    player_id: Uuid,
    message: ServerMessage,
    deps: &MessageRoutingDeps,
) {
    info!("⚡ Routing to same-pod player {} (by ID)", player_id);

    // Same-pod 라우팅
    route_to_same_pod(player_id, &player_id.to_string(), &message, deps).await;

    // 레거시 경로 (test_client 호환)
    send_direct_message_legacy(&deps.subscription_addr, player_id, &message).await;
}

/// Same-pod 플레이어에게 직접 메시지 전달 (Actor 메시지)
async fn route_to_same_pod(
    player_uuid: Uuid,
    player_id: &str,
    message: &ServerMessage,
    deps: &MessageRoutingDeps,
) {
    if let Some(lb_addr) = &deps.load_balance_addr {
        lb_addr.do_send(RouteToGamePlayer {
            player_id: player_uuid,
            message: legacy_server_message_to_unity(message.clone()),
        });

        // 메트릭
        MESSAGES_ROUTED_SAME_POD_TOTAL.inc();

        debug!("✅ Message sent to same-pod player {}", player_id);
    } else {
        warn!(
            "LoadBalanceActor not available for same-pod player {}",
            player_id
        );
    }
}

/// Cross-pod 플레이어에게 Redis Pub/Sub로 메시지 전달
async fn route_to_cross_pod(
    player: &PlayerCandidate,
    message: &ServerMessage,
    deps: &MessageRoutingDeps,
) {
    let channel = format!("pod:{}:game_message", player.pod_id);

    // 메시지 payload 구성
    let payload = serde_json::json!({
        "player_id": player.player_id,
        "message": message,
        "timestamp": chrono::Utc::now().to_rfc3339(),
    });

    let payload_str = match serde_json::to_string(&payload) {
        Ok(s) => s,
        Err(e) => {
            error!("Failed to serialize cross-pod message: {}", e);
            return;
        }
    };

    // Redis PUBLISH (재시도 없음, 1회만)
    match redis::cmd("PUBLISH")
        .arg(&channel)
        .arg(&payload_str)
        .query_async::<_, i64>(&mut deps.redis.clone())
        .await
    {
        Ok(subscriber_count) => {
            if subscriber_count == 0 {
                warn!(
                    "No subscribers for channel {} (player {} may be offline)",
                    channel, player.player_id
                );
            } else {
                info!(
                    "📤 Published to {} ({} subscribers)",
                    channel, subscriber_count
                );
            }

            // 메트릭
            MESSAGES_ROUTED_CROSS_POD_TOTAL.inc();
        }
        Err(e) => {
            error!("Failed to publish to {}: {}", channel, e);
        }
    }
}

/// 레거시 경로: SubScriptionManager를 통한 메시지 전달 (test_client용)
///
/// # Note
/// - test_client 호환성을 위해 유지
/// - Unity client는 이 경로를 사용하지 않음
async fn send_direct_message_legacy(
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
                        "Legacy message delivery attempt failed for player {}: {:?}",
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
            "Legacy message delivery permanently failed for player {}: {:?}",
            player_id, e
        );
    }
}

/// ❌ DEPRECATED: notification:{player_id} 채널 발행
///
/// 이 함수는 더 이상 사용되지 않습니다.
/// - 아무도 구독하지 않는 채널입니다
/// - 향후 버전에서 제거 예정
/// - 대신 `route_to_cross_pod()` 사용
///
/// # Deprecation Timeline
/// - 2025-10-22: Deprecated 마킹
/// - 2025-11-22: 제거 예정 (1개월 후)
#[deprecated(
    since = "0.2.0",
    note = "Use route_to_cross_pod() instead. This channel has no subscribers."
)]
#[allow(dead_code)]
async fn publish_to_redis_deprecated(
    redis: &mut ConnectionManager,
    player_id: Uuid,
    message: &ServerMessage,
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
