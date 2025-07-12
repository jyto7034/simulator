use redis::aio::ConnectionManager;
// `cmd`를 추가하고, `Commands`와 `Script`를 임포트합니다. `SetOptions`는 사용하지 않으므로 삭제합니다.
use redis::{cmd, RedisResult, Script};
use uuid::Uuid;

const RELEASE_SCRIPT: &str = r#"
    if redis.call("get", KEYS[1]) == ARGV[1] then
        return redis.call("del", KEYS[1])
    else
        return 0
    end
"#;

/// Redis를 이용한 분산락 구조체.
/// Drop 트레이트를 구현하지 않았으므로, 사용 후 반드시 `release`를 명시적으로 호출해야 합니다.
pub struct DistributedLock {
    key: String,
    value: String,
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
    /// * `Ok(Some(lock))` - 락 획득 성공
    /// * `Ok(None)` - 다른 프로세스가 락을 이미 소유하고 있음
    /// * `Err(e)` - Redis 오류 발생
    pub async fn acquire(
        redis: &mut ConnectionManager,
        key: &str,
        duration_ms: usize,
    ) -> RedisResult<Option<Self>> {
        let value = Uuid::new_v4().to_string();

        // `set_options` 대신 `cmd`를 사용하여 'SET key value NX PX ms' 명령을 직접 구성합니다.
        let result: Option<String> = cmd("SET")
            .arg(key)
            .arg(&value)
            .arg("NX") // Set only if the key does not already exist.
            .arg("PX") // Set the specified expire time, in milliseconds.
            .arg(duration_ms)
            .query_async(redis) // ConnectionManager에서 비동기적으로 실행
            .await?;

        // 'SET NX'는 성공 시 "OK"를 반환하고, 키가 이미 존재하면 nil을 반환합니다.
        // `redis-rs`는 "OK"를 `Some("OK".to_string())`으로, nil을 `None`으로 변환합니다.
        if result.is_some() {
            Ok(Some(Self {
                key: key.to_string(),
                value,
            }))
        } else {
            Ok(None)
        }
    }

    /// 획득했던 분산락을 해제합니다.
    pub async fn release(&self, redis: &mut ConnectionManager) -> RedisResult<()> {
        let script = Script::new(RELEASE_SCRIPT);
        script
            .key(&self.key)
            .arg(&self.value)
            .invoke_async::<_, ()>(redis)
            .await?;
        Ok(())
    }
}
