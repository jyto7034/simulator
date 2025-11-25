use backoff::backoff::Backoff;
use chrono::Utc;
use redis::aio::ConnectionManager;
use redis::Script;
use serde_json::json;
use tokio::time::sleep;
use tracing::{error, info, warn};
use uuid::Uuid;

use crate::{
    matchmaking::matchmaker::{
        operations::{notify::{self, MessageRoutingDeps}, try_match::PlayerCandidate, with_redis_timeout},
        scripts, MatchmakerDeps,
    },
    shared::protocol::{ErrorCode, ServerMessage},
    shared::redis_events,
    GameMode, RETRY_CONFIG,
};

/// metadata JSON에 pod_id를 자동으로 추가합니다.
/// Client는 pod_id를 알 수 없으므로 Match Server가 자동으로 주입합니다.
fn add_pod_id_to_metadata(metadata: &str, pod_id: &str) -> String {
    match serde_json::from_str::<serde_json::Value>(metadata) {
        Ok(mut value) => {
            if let Some(obj) = value.as_object_mut() {
                obj.insert("pod_id".to_string(), json!(pod_id));
                serde_json::to_string(&value).unwrap_or_else(|_| metadata.to_string())
            } else {
                // JSON이지만 object가 아닌 경우 (array 등)
                format!(r#"{{"pod_id":"{}","original":{}}}"#, pod_id, metadata)
            }
        }
        Err(_) => {
            // JSON 파싱 실패 시 최소한의 metadata 생성
            warn!("Failed to parse metadata JSON, creating minimal metadata with pod_id");
            format!(r#"{{"pod_id":"{}"}}"#, pod_id)
        }
    }
}

async fn invoke_enqueue_script(
    redis: &mut ConnectionManager,
    queue_key: String,
    player_id: Uuid,
    timestamp: String,
    metadata: String,
    timeout_secs: u64,
) -> Result<Vec<i64>, String> {
    with_redis_timeout("enqueue_player_script", timeout_secs, async {
        Script::new(scripts::enqueue_player_script())
            .key(queue_key)
            .arg(player_id.to_string())
            .arg(timestamp)
            .arg(metadata)
            .invoke_async(redis)
            .await
    })
    .await
}

pub async fn enqueue(
    queue_suffix: &str,
    game_mode: GameMode,
    player_id: Uuid,
    metadata: String,
    deps: &MatchmakerDeps,
) {
    let routing_deps = MessageRoutingDeps::from(deps);
    let mut redis = deps.redis.clone();
    let settings = deps.settings.clone();

    let is_known_mode = settings
        .game_modes
        .iter()
        .any(|mode| mode.game_mode == game_mode);

    if !is_known_mode {
        warn!(
            "Player {} tried to enqueue for unsupported mode {:?}",
            player_id, game_mode
        );
        notify::send_message_to_player_by_id(
            player_id,
            ServerMessage::Error {
                code: ErrorCode::InvalidGameMode,
                message: "Unsupported game mode".to_string(),
            },
            &routing_deps,
        )
        .await;
        return;
    }

    // pod_id를 metadata에 자동 추가
    let pod_id = PlayerCandidate::current_pod_id();
    let metadata_with_pod = add_pod_id_to_metadata(&metadata, pod_id);

    let suffix = queue_suffix;
    let hash_tag = format!("{{{}}}", suffix);
    let queue_key = format!("queue:{}", hash_tag);

    let timestamp = Utc::now().timestamp().to_string();
    let timeout_secs = settings.redis_operation_timeout_seconds;

    let backoff = RETRY_CONFIG
        .read()
        .await
        .as_ref()
        .expect("Retry config not initialized")
        .clone();

    let mut backoff_state = backoff;
    let enqueue_result = loop {
        let mut redis_clone = redis.clone();

        match invoke_enqueue_script(
            &mut redis_clone,
            queue_key.clone(),
            player_id,
            timestamp.clone(),
            metadata_with_pod.clone(),
            timeout_secs,
        )
        .await
        {
            Ok(res) => break Ok(res),
            Err(err) => {
                if let Some(delay) = backoff_state.next_backoff() {
                    warn!(
                        "Temporary enqueue failure for player {}: {} (retrying in {:?})",
                        player_id, err, delay
                    );
                    sleep(delay).await;
                    continue;
                } else {
                    break Err(err);
                }
            }
        }
    };

    let result = match enqueue_result {
        Ok(res) => res,
        Err(err) => {
            error!(
                "Failed to enqueue player {} into {}: {}",
                player_id, queue_key, err
            );
            notify::send_message_to_player_by_id(
                player_id,
                ServerMessage::Error {
                    code: ErrorCode::InternalError,
                    message: "Failed to enqueue".to_string(),
                },
                &routing_deps,
            )
            .await;
            return;
        }
    };

    let added_flag = result.get(0).copied().unwrap_or_default();
    let current_size = result.get(1).copied().unwrap_or_default();

    let response = if added_flag == 1 {
        info!(
            "Player {} enqueued for {:?} on pod {}. queue size = {}",
            player_id, game_mode, pod_id, current_size
        );

        // Metrics: 신규 Enqueue 카운트
        metrics::PLAYERS_ENQUEUED_NEW_TOTAL.inc();
        metrics::ENQUEUED_TOTAL_BY_MODE
            .with_label_values(&[&format!("{:?}", game_mode)])
            .inc();

        // Publish test event
        redis_events::try_publish_test_event(
            &mut redis,
            &metadata_with_pod,
            "player.enqueued",
            &pod_id,
            vec![
                ("player_id", player_id.to_string()),
                ("queue_size", current_size.to_string()),
                ("game_mode", format!("{:?}", game_mode)),
            ],
        )
        .await;

        // Publish queue_size_changed event (global event for all watchers)
        redis_events::try_publish_test_event(
            &mut redis,
            &metadata_with_pod,
            "global.queue_size_changed",
            &pod_id,
            vec![
                ("size", current_size.to_string()),
                ("game_mode", format!("{:?}", game_mode)),
                ("reason", "enqueued".to_string()),
            ],
        )
        .await;

        ServerMessage::EnQueued { pod_id: pod_id.to_string() }
    } else {
        warn!("Player {} already in queue {:?}", player_id, game_mode);

        // Metrics: 중복 Enqueue 시도
        metrics::ABNORMAL_DUPLICATE_ENQUEUE_TOTAL.inc();

        ServerMessage::Error {
            code: ErrorCode::AlreadyInQueue,
            message: "Already in queue".to_string(),
        }
    };

    notify::send_message_to_player_by_id(player_id, response, &routing_deps).await;
}

pub async fn re_enqueue_candidates(
    queue_suffix: &str,
    game_mode: GameMode,
    candidates: &[PlayerCandidate],
    deps: &MatchmakerDeps,
) {
    // Metrics: Re-enqueue 카운트
    if !candidates.is_empty() {
        metrics::PLAYERS_REQUEUED_TOTAL.inc_by(candidates.len() as u64);
    }

    let suffix = queue_suffix;
    let hash_tag = format!("{{{}}}", suffix);
    let queue_key = format!("queue:{}", hash_tag);
    let pod_id = PlayerCandidate::current_pod_id();

    for candidate in candidates {
        let player_id = match Uuid::parse_str(&candidate.player_id) {
            Ok(id) => id,
            Err(e) => {
                error!("Invalid player_id during re-enqueue: {}", e);
                continue;
            }
        };

        let metadata_json = match serde_json::to_string(&candidate.metadata) {
            Ok(json) => json,
            Err(e) => {
                error!("Failed to serialize metadata: {}", e);
                continue;
            }
        };

        let timestamp = Utc::now().timestamp().to_string();
        let mut redis = deps.redis.clone();
        let timeout_secs = deps.settings.redis_operation_timeout_seconds;

        // Redis에 re-enqueue
        match invoke_enqueue_script(
            &mut redis,
            queue_key.clone(),
            player_id,
            timestamp.clone(),
            metadata_json.clone(),
            timeout_secs,
        )
        .await
        {
            Ok(result) => {
                let added_flag = result.get(0).copied().unwrap_or_default();
                let current_size = result.get(1).copied().unwrap_or_default();

                if added_flag == 1 {
                    info!(
                        "Player {} re-enqueued for {:?}. queue size = {}",
                        player_id, game_mode, current_size
                    );

                    // Publish PlayerReEnqueued event (테스트용)
                    redis_events::try_publish_test_event(
                        &mut redis,
                        &metadata_json,
                        "player.re_enqueued",
                        &pod_id,
                        vec![
                            ("player_id", player_id.to_string()),
                            ("queue_size", current_size.to_string()),
                            ("game_mode", format!("{:?}", game_mode)),
                            ("reason", "match_failed".to_string()),
                        ],
                    )
                    .await;

                    // Publish queue_size_changed event
                    redis_events::try_publish_test_event(
                        &mut redis,
                        &metadata_json,
                        "global.queue_size_changed",
                        &pod_id,
                        vec![
                            ("size", current_size.to_string()),
                            ("game_mode", format!("{:?}", game_mode)),
                            ("reason", "re_enqueued".to_string()),
                        ],
                    )
                    .await;

                    // Send EnQueued message to player via WebSocket
                    // notify::send_message_to_player(
                    //     deps.subscription_addr.clone(),
                    //     &mut redis,
                    //     player_id,
                    //     ServerMessage::EnQueued {
                    //         pod_id: pod_id.clone(),
                    //     },
                    // )
                    // .await;
                } else {
                    warn!(
                        "Player {} already in queue during re-enqueue {:?}",
                        player_id, game_mode
                    );
                }
            }
            Err(e) => {
                error!(
                    "Failed to re-enqueue player {} into {}: {}",
                    player_id, queue_key, e
                );
            }
        }
    }
}
