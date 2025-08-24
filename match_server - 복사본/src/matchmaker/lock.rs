use redis::aio::ConnectionManager;
use redis::{RedisResult, Script};
use uuid::Uuid;

// Improved lock acquisition script with timestamp validation
const ACQUIRE_SCRIPT: &str = r#"
    local key = KEYS[1]
    local value = ARGV[1]
    local ttl_ms = tonumber(ARGV[2])
    
    -- Check if key exists
    local existing = redis.call("GET", key)
    if existing == false then
        -- Key doesn't exist, acquire lock
        redis.call("SET", key, value, "PX", ttl_ms)
        return {"OK", value}
    else
        -- Key exists, check if it's expired (fallback safety)
        local ttl = redis.call("PTTL", key)
        if ttl == -1 then
            -- Key exists but has no TTL, force release and re-acquire
            redis.call("DEL", key)
            redis.call("SET", key, value, "PX", ttl_ms)
            return {"OK", value}
        elseif ttl == -2 then
            -- Key was deleted between GET and PTTL, try again
            redis.call("SET", key, value, "PX", ttl_ms)
            return {"OK", value}
        else
            -- Lock is held by another process
            return {"BUSY", ttl}
        end
    end
"#;

const RELEASE_SCRIPT: &str = r#"
    local key = KEYS[1]
    local value = ARGV[1]
    
    if redis.call("GET", key) == value then
        return redis.call("DEL", key)
    else
        return 0
    end
"#;


/// Redis를 이용한 분산락 구조체.
/// Drop 트레이트를 구현하지 않았으므로, 사용 후 반드시 `release`를 명시적으로 호출해야 합니다.
#[derive(Debug, Clone)]
pub struct DistributedLock {
    key: String,
    value: String,
}

#[derive(Debug, PartialEq)]
pub enum LockResult {
    Acquired,
    Busy { remaining_ttl_ms: i64 },
    Error(String),
}

impl DistributedLock {
    /// 분산락을 획득합니다.
    ///
    /// # Arguments
    /// * `redis` - Redis 커넥션 매니저
    /// * `key` - 락을 걸 대상 키
    /// * `duration_ms` - 락의 만료 시간 (밀리초)
    ///
    /// # Returns
    /// * `Ok((lock, LockResult::Acquired))` - 락 획득 성공
    /// * `Ok((None, LockResult::Busy))` - 다른 프로세스가 락을 이미 소유하고 있음
    /// * `Err(e)` - Redis 오류 발생
    pub async fn acquire(
        redis: &mut ConnectionManager,
        key: &str,
        duration_ms: usize,
    ) -> RedisResult<(Option<Self>, LockResult)> {
        let value = Uuid::new_v4().to_string();

        let script = Script::new(ACQUIRE_SCRIPT);
        let result: Vec<String> = script
            .key(key)
            .arg(&value)
            .arg(duration_ms)
            .invoke_async(redis)
            .await?;

        if result.len() >= 2 {
            match result[0].as_str() {
                "OK" => {
                    let lock = Self {
                        key: key.to_string(),
                        value: result[1].clone(),
                    };
                    Ok((Some(lock), LockResult::Acquired))
                }
                "BUSY" => {
                    let remaining_ttl = result[1].parse::<i64>().unwrap_or(0);
                    Ok((
                        None,
                        LockResult::Busy {
                            remaining_ttl_ms: remaining_ttl,
                        },
                    ))
                }
                _ => Ok((
                    None,
                    LockResult::Error("Unknown script response".to_string()),
                )),
            }
        } else {
            Ok((
                None,
                LockResult::Error("Invalid script response".to_string()),
            ))
        }
    }

    /// 획득했던 분산락을 해제합니다.
    ///
    /// # Returns
    /// * `Ok(true)` - 락이 성공적으로 해제됨
    /// * `Ok(false)` - 락이 이미 해제되었거나 다른 프로세스가 소유함
    pub async fn release(&self, redis: &mut ConnectionManager) -> RedisResult<bool> {
        let script = Script::new(RELEASE_SCRIPT);
        let result: i32 = script
            .key(&self.key)
            .arg(&self.value)
            .invoke_async(redis)
            .await?;
        Ok(result == 1)
    }

}
