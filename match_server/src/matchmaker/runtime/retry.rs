use redis::{aio::ConnectionManager, AsyncCommands};
use tracing::warn;

#[derive(Debug, Clone)]
pub enum RetryDecision {
    RetryRemaining {
        attempt: u32,
        max: u32,
        _retry_key: String,
    },
    FinalExhausted {
        max: u32,
        retry_key: String,
    },
}

/// Increment group-based retry counter and decide whether to retry or exhaust.
/// - Key pattern: retry:alloc:<game_mode>:<sorted_player_ids>
/// - TTL strategy: refresh TTL on each increment (aligns with existing behavior)
pub async fn incr_and_decide(
    redis: &mut ConnectionManager,
    game_mode: &str,
    players_unsorted: &[String],
    max_retries: u32,
    ttl_seconds: usize,
) -> RetryDecision {
    let mut players = players_unsorted.to_vec();
    players.sort();
    let group_id = players.join(",");
    let retry_key = format!("retry:alloc:{}:{}", game_mode, group_id);

    // INCR with TTL refresh
    let count: i64 = match redis.incr::<_, _, i64>(&retry_key, 1).await {
        Ok(c) => c,
        Err(e) => {
            warn!(
                "Failed to increment retry counter for key {}: {} — proceeding as first attempt",
                retry_key, e
            );
            1
        }
    };
    let _ = redis.expire::<_, bool>(&retry_key, ttl_seconds).await;

    if (count as u32) >= max_retries {
        RetryDecision::FinalExhausted {
            max: max_retries,
            retry_key,
        }
    } else {
        RetryDecision::RetryRemaining {
            attempt: count as u32,
            max: max_retries,
            _retry_key: retry_key,
        }
    }
}
