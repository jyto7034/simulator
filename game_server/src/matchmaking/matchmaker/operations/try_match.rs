use redis::{aio::ConnectionManager, Script};
use std::sync::OnceLock;
use tracing::{error, info, warn};
use uuid::Uuid;

use crate::matchmaking::matchmaker::{operations::with_redis_timeout, scripts, MatchmakerDeps};

async fn invoke_try_match_script(
    redis: &mut ConnectionManager,
    queue_key: String,
    batch_size: usize,
    timeout_secs: u64,
) -> Result<Vec<String>, String> {
    info!(
        "invoke_try_match_script: Calling Lua script with queue_key={}, batch_size={}",
        queue_key, batch_size
    );

    info!(
        "invoke_try_match_script: Before invoke_async with timeout={}s",
        timeout_secs
    );
    let start = std::time::Instant::now();

    let result = with_redis_timeout("pop_candidates_script", timeout_secs, async {
        Script::new(scripts::try_match_pop_script())
            .key(queue_key)
            .arg(batch_size)
            .invoke_async(redis)
            .await
    })
    .await;

    let elapsed = start.elapsed();
    info!(
        "invoke_try_match_script: After invoke_async, elapsed: {:?}",
        elapsed
    );

    info!("invoke_try_match_script: Lua script completed");
    result
}

pub async fn pop_candidates(
    queue_suffix: &str,
    batch_size: usize,
    deps: &MatchmakerDeps,
) -> Result<(Vec<PlayerCandidate>, Vec<String>), String> {
    info!(
        "pop_candidates: Starting for queue {} with batch_size {}",
        queue_suffix, batch_size
    );

    if batch_size == 0 {
        info!("pop_candidates: batch_size is 0, returning empty");
        return Ok((Vec::new(), Vec::new()));
    }

    let mut redis = deps.redis.clone();
    let hash_tag = format!("{{{}}}", queue_suffix);
    let queue_key = format!("queue:{}", hash_tag);
    let timeout_secs = deps.settings.redis_operation_timeout_seconds;

    let raw: Vec<String> =
        invoke_try_match_script(&mut redis, queue_key, batch_size, timeout_secs).await?;

    if raw.len() % 3 != 0 {
        info!(
            "pop_candidates: ERROR - raw.len() % 3 != 0, got {} strings",
            raw.len()
        );
        return Err(format!(
            "unexpected response length in try_match pop script - expected triplets, got {} items",
            raw.len()
        ));
    }

    info!("pop_candidates: Parsing {} candidates", raw.len() / 3);
    let mut candidates = Vec::new();
    let mut poisoned_player_ids = Vec::new();

    for chunk in raw.chunks_exact(3) {
        let player_id = chunk[0].clone();

        // score 파싱 실패 시 스킵
        let score = match chunk[1].parse::<i64>() {
            Ok(s) => s,
            Err(e) => {
                error!("Poisoned candidate {}: invalid score - {}", player_id, e);
                poisoned_player_ids.push(player_id);
                continue;
            }
        };

        let metadata_json = chunk[2].clone();

        // metadata JSON 파싱 실패 시 스킵
        let metadata = match serde_json::from_str::<serde_json::Value>(&metadata_json) {
            Ok(m) => m,
            Err(e) => {
                error!("Poisoned candidate {}: invalid JSON - {}", player_id, e);
                poisoned_player_ids.push(player_id);
                continue;
            }
        };

        // pod_id 없으면 오염된 플레이어에 추가
        let pod_id = match metadata.get("pod_id").and_then(|p| p.as_str()) {
            Some(p) => p.to_string(),
            None => {
                error!(
                    "Poisoned candidate {}: pod_id not found in metadata",
                    player_id
                );
                poisoned_player_ids.push(player_id);
                continue;
            }
        };

        candidates.push(PlayerCandidate {
            player_id,
            score,
            pod_id,
            metadata,
        });
    }

    if !poisoned_player_ids.is_empty() {
        warn!(
            "pop_candidates: Skipped {} poisoned candidate(s) from queue {}, {} valid candidates remain",
            poisoned_player_ids.len(),
            queue_suffix,
            candidates.len()
        );
        // Record poisoned candidates metric
        metrics::POISONED_CANDIDATES_TOTAL.inc_by(poisoned_player_ids.len() as u64);
    }

    info!(
        "pop_candidates: Successfully parsed {} candidates",
        candidates.len()
    );
    Ok((candidates, poisoned_player_ids))
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PlayerCandidate {
    pub player_id: String,
    pub score: i64,
    pub pod_id: String,
    pub metadata: serde_json::Value,
}

impl PlayerCandidate {
    /// 현재 Pod와 같은 Pod에 있는지 확인
    pub fn is_same_pod(&self) -> bool {
        self.pod_id == Self::current_pod_id()
    }

    /// 현재 Pod ID 조회 (캐싱, public)
    pub fn current_pod_id() -> &'static str {
        static POD_ID: OnceLock<String> = OnceLock::new();

        POD_ID.get_or_init(|| {
            std::env::var("POD_ID").unwrap_or_else(|_| {
                warn!("POD_ID not set, using 'unknown'");
                "unknown".to_string()
            })
        })
    }

    /// player_id를 UUID로 파싱
    pub fn player_uuid(&self) -> Result<Uuid, uuid::Error> {
        Uuid::parse_str(&self.player_id)
    }
}
