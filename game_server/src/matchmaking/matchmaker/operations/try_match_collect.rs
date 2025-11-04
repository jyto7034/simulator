use backoff::backoff::Backoff;
use tokio::time::sleep;
use tokio_util::sync::CancellationToken;
use tracing::{error, warn};
use uuid::Uuid;

use crate::{
    matchmaking::matchmaker::{
        operations::{
            notify::{self, MessageRoutingDeps},
            try_match::pop_candidates,
        },
        MatchmakerDeps,
    },
    shared::protocol::{ErrorCode, ServerMessage},
    RETRY_CONFIG,
};

use super::try_match::PlayerCandidate;

/// Candidates 수집 (백오프 및 poisoned 처리 포함)
pub async fn collect_candidates_with_retry(
    queue_suffix: &str,
    required_count: usize,
    deps: &MatchmakerDeps,
    shutdown_token: &CancellationToken,
) -> Result<Vec<PlayerCandidate>, String> {
    let mut backoff = RETRY_CONFIG
        .read()
        .await
        .as_ref()
        .expect("Retry config not initialized")
        .clone();

    loop {
        // 종료 신호 체크
        if shutdown_token.is_cancelled() {
            return Err("Shutdown requested".to_string());
        }

        match pop_candidates(queue_suffix, required_count, deps).await {
            Ok((candidates, poisoned_ids)) => {
                // Circuit breaker 성공 기록
                deps.redis_circuit.record_success();

                // Poisoned candidates 알림
                if !poisoned_ids.is_empty() {
                    notify_poisoned_candidates(poisoned_ids, deps).await;
                }

                return Ok(candidates);
            }
            Err(err) => {
                // Circuit breaker 실패 기록
                deps.redis_circuit.record_failure();

                if let Some(delay) = backoff.next_backoff() {
                    warn!(
                        "Failed to pop candidates from queue {}: {} (retrying in {:?})",
                        queue_suffix, err, delay
                    );

                    // 종료 신호와 함께 대기
                    tokio::select! {
                        _ = sleep(delay) => continue,
                        _ = shutdown_token.cancelled() => {
                            return Err("Shutdown during backoff".to_string());
                        }
                    }
                } else {
                    error!(
                        "Failed to pop candidates after all retries from queue {}: {}",
                        queue_suffix, err
                    );
                    return Err(format!("Max retries exceeded: {}", err));
                }
            }
        }
    }
}

/// Poisoned candidates에게 알림
async fn notify_poisoned_candidates(poisoned_ids: Vec<String>, deps: &MatchmakerDeps) {
    let routing_deps = MessageRoutingDeps::from(deps);

    for player_id_str in poisoned_ids {
        error!(
            "Notifying poisoned candidate {} that they were dequeued",
            player_id_str
        );

        let player_uuid = match Uuid::parse_str(&player_id_str) {
            Ok(uuid) => uuid,
            Err(_) => {
                error!(
                    "Failed to parse poisoned player_id as UUID: {}",
                    player_id_str
                );
                continue;
            }
        };

        // Dequeued 메시지
        notify::send_message_to_player_by_id(player_uuid, ServerMessage::DeQueued, &routing_deps)
            .await;

        // Error 메시지
        notify::send_message_to_player_by_id(
            player_uuid,
            ServerMessage::Error {
                code: ErrorCode::InvalidMetadata,
                message: "Invalid player metadata - removed from queue".to_string(),
            },
            &routing_deps,
        )
        .await;
    }
}
