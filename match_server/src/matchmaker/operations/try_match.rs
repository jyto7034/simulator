use redis::{aio::ConnectionManager, AsyncCommands, ErrorKind, RedisError, RedisResult, Script};

use crate::{
    matchmaker::{scripts, MatchmakerDeps},
    protocol::BattleRequest,
};

async fn invoke_try_match_script(
    redis: &mut ConnectionManager,
    queue_key: String,
    batch_size: usize,
) -> RedisResult<Vec<String>> {
    Script::new(scripts::try_match_pop_script())
        .key(queue_key)
        .arg(batch_size)
        .invoke_async(redis)
        .await
}

pub async fn pop_candidates(
    queue_suffix: &str,
    batch_size: usize,
    deps: &MatchmakerDeps,
) -> RedisResult<Vec<PlayerCandidate>> {
    if batch_size == 0 {
        return Ok(Vec::new());
    }

    let mut redis = deps.redis.clone();
    let hash_tag = format!("{{{}}}", queue_suffix);
    let queue_key = format!("queue:{}", hash_tag);

    let raw: Vec<String> = invoke_try_match_script(&mut redis, queue_key, batch_size).await?;

    if raw.len() % 3 != 0 {
        return Err(RedisError::from((
            ErrorKind::TypeError,
            "unexpected response length in try_match pop script - expected triplets",
        )));
    }

    let mut candidates = Vec::with_capacity(raw.len() / 3);
    let mut iter = raw.chunks_exact(3);
    while let Some(chunk) = iter.next() {
        let player_id = chunk[0].clone();
        let score = chunk[1].parse::<i64>().map_err(|_| {
            RedisError::from((
                ErrorKind::TypeError,
                "invalid score returned from try_match pop script",
            ))
        })?;
        let metadata_json = chunk[2].clone();

        // metadata 파싱
        let metadata = serde_json::from_str::<serde_json::Value>(&metadata_json).map_err(|e| {
            RedisError::from((
                ErrorKind::TypeError,
                "failed to parse metadata JSON",
                e.to_string(),
            ))
        })?;

        // metadata에서 pod_id 추출
        let pod_id = metadata
            .get("pod_id")
            .and_then(|p| p.as_str())
            .map(String::from)
            .ok_or_else(|| {
                RedisError::from((ErrorKind::TypeError, "pod_id not found in metadata"))
            })?;
        // TODO: pod_id 가 없을 경우, 오염된 플레이어로 간주하고 로그 처리 해당 match 는 실패로 처리.

        candidates.push(PlayerCandidate {
            player_id,
            score,
            pod_id,
            metadata,
        });
    }

    Ok(candidates)
}

/// Battle request를 Redis에 publish하는 헬퍼 함수
pub async fn publish_battle_request(
    redis: &mut ConnectionManager,
    channel: &str,
    request: &BattleRequest,
) -> Result<usize, String> {
    let json = serde_json::to_string(request)
        .map_err(|e| format!("Failed to serialize BattleRequest: {}", e))?;

    // TODO: subscriber_count 를 활용하여 Game Server 생존 여부 확인, 오류 전파 등 구현해야함.
    let subscriber_count = redis
        .publish(channel, json)
        .await
        .map_err(|e| format!("Failed to publish battle request: {}", e))?;

    Ok(subscriber_count)
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PlayerCandidate {
    pub player_id: String,
    pub score: i64,
    pub pod_id: String,
    pub metadata: serde_json::Value,
}
