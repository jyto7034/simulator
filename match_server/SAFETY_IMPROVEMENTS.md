# Match Server 안전 장치 개선 사항

현재 `CancellationToken`으로 좀비 Future 방지는 구현되었으나, 추가로 필요한 안전 장치들을 정리합니다.

---

## 우선순위 요약

| 우선순위 | 안전장치 | 구현 난이도 | 영향도 | 상태 |
|---------|---------|------------|--------|------|
| **P0** | **Redis Timeout 보호** | 쉬움 | 치명적 (무한 대기) | ❌ 미구현 |
| **P0** | **Max In-Flight Limit** | 쉬움 | 높음 (과부하 방지) | ❌ 미구현 |
| **P1** | **Poison Message 처리** | 중간 | 중간 (가용성) | ⚠️ TODO 있음 |
| **P1** | **Game Server Monitor** | 쉬움 | 중간 (운영 가시성) | ⚠️ TODO 있음 |
| **P2** | **Circuit Breaker** | 중간 | 낮음 (장애 시간 단축) | ❌ 미구현 |
| ✅ | **Re-enqueue on Failure** | - | - | ✅ 구현됨 |

---

## P0: Redis Timeout 보호 (치명적)

### 문제

Redis 작업이 무한 대기할 수 있습니다.

```rust
// 현재 코드 (rank/handlers.rs:123)
match pop_candidates(queue_suffix, required_players as usize * 2, &deps).await {
    Ok(candidates) => break candidates,
    Err(err) => { /* retry */ }
}

// try_match.rs:94
let subscriber_count = redis.publish(channel, json).await?;
```

**시나리오**: Redis 네트워크 장애 → `.await`가 영원히 리턴 안 함 → Actor 멈춤

### 해결책

#### 1. Settings에 timeout 설정 추가

```rust
// env.rs
#[derive(Debug, Deserialize, Clone)]
pub struct MatchmakingSettings {
    pub redis_operation_timeout_seconds: u64,  // 추가 (권장: 10)
    // ...
}
```

```toml
# config/production.toml
[matchmaking]
redis_operation_timeout_seconds = 10
```

#### 2. Timeout wrapper 함수 작성

```rust
// matchmaker/operations/mod.rs
use tokio::time::{timeout, Duration};

pub async fn with_timeout<F, T>(
    operation_name: &str,
    timeout_secs: u64,
    future: F,
) -> Result<T, String>
where
    F: Future<Output = Result<T, String>>,
{
    match timeout(Duration::from_secs(timeout_secs), future).await {
        Ok(Ok(result)) => Ok(result),
        Ok(Err(err)) => Err(err),
        Err(_) => {
            error!("{} timeout after {}s", operation_name, timeout_secs);
            Err(format!("{} timeout", operation_name))
        }
    }
}
```

#### 3. 모든 Redis 작업에 적용

```rust
// rank/handlers.rs:123
let timeout_secs = deps.settings.redis_operation_timeout_seconds;

match with_timeout(
    "pop_candidates",
    timeout_secs,
    async {
        pop_candidates(queue_suffix, required_players * 2, &deps)
            .await
            .map_err(|e| e.to_string())
    }
).await {
    Ok(candidates) => break candidates,
    Err(err) => {
        // timeout 또는 Redis 에러
        if let Some(delay) = backoff.next_backoff() { /* retry */ }
    }
}

// publish도 동일하게
with_timeout(
    "publish_battle_request",
    timeout_secs,
    async {
        publish_battle_request(&mut redis, channel, &request).await
    }
).await?;
```

### 효과

- Redis 장애 시 10초 후 자동 복구 시도
- Actor가 무한 대기 상태에 빠지지 않음
- 명확한 에러 로그 (timeout vs network error)

---

## P0: Max In-Flight Limit (과부하 방지)

### 문제

동시에 너무 많은 TryMatch가 실행될 수 있습니다.

```rust
// rank/mod.rs:62
ctx.run_interval(Duration::from_secs(5), move |_actor, ctx| {
    ctx.notify(TryMatch { ... });  // 무조건 실행!
});
```

**시나리오**:
```
시간 0초:  TryMatch #1 시작 (Redis 느림, 20초 소요)
시간 5초:  TryMatch #2 시작 (interval 트리거)
시간 10초: TryMatch #3 시작
시간 15초: TryMatch #4 시작
→ 동시에 4개 실행 중! Redis/Game Server 폭주 😱
```

### 해결책

#### 1. MatchmakerInner에 플래그 추가

```rust
// matchmaker/common.rs
use std::sync::atomic::{AtomicBool, Ordering};

pub struct MatchmakerInner {
    pub is_try_match_running: Arc<AtomicBool>,
    // ...
}

impl MatchmakerInner {
    pub fn new(...) -> Self {
        Self {
            is_try_match_running: Arc::new(AtomicBool::new(false)),
            // ...
        }
    }
}
```

#### 2. Handler에서 중복 실행 방지

```rust
// rank/handlers.rs:92
impl Handler<TryMatch> for RankedMatchmaker {
    type Result = ();

    fn handle(&mut self, msg: TryMatch, ctx: &mut Self::Context) -> Self::Result {
        // 이미 실행 중이면 스킵
        if self.is_try_match_running.swap(true, Ordering::Relaxed) {
            warn!("TryMatch already running, skipping this tick");
            return;
        }

        let deps: MatchmakerDeps = (&self.inner).into();
        let is_running = self.is_try_match_running.clone();
        let shutdown_token = self.shutdown_token.clone();
        // ...

        async move {
            // ... 기존 로직 ...

            // 완료 후 플래그 해제
            is_running.store(false, Ordering::Relaxed);
        }
        .into_actor(self)
        .wait(ctx);
    }
}
```

#### 3. 메트릭 추가 (선택)

```rust
// 스킵 횟수 기록
if self.is_try_match_running.swap(true, Ordering::Relaxed) {
    warn!("TryMatch already running, skipping this tick");
    deps.metrics.try_match_skipped.inc();
    return;
}
```

### 효과

- 동시 실행 = 최대 1개로 제한
- Redis/Game Server 부하 예측 가능
- 느린 작업이 쌓이지 않음

---

## P1: Poison Message 처리

### 문제

`pod_id` 없는 플레이어 1명 때문에 전체 매칭 실패합니다.

```rust
// try_match.rs:64-71
let pod_id = metadata
    .get("pod_id")
    .and_then(|p| p.as_str())
    .map(String::from)
    .ok_or_else(|| {
        RedisError::from((ErrorKind::TypeError, "pod_id not found in metadata"))
    })?;  // ❌ 여기서 에러 → 전체 candidates 버림!

// TODO: pod_id 가 없을 경우, 오염된 플레이어로 간주하고 로그 처리 해당 match 는 실패로 처리.
```

**시나리오**:
```
Queue: [Player1(정상), Player2(pod_id 없음), Player3(정상), Player4(정상)]
→ pop_candidates() 호출
→ Player2에서 에러 발생
→ 전체 Err 반환
→ Player1, 3, 4도 매칭 실패 😱
```

### 해결책

#### 1. PoisonedCandidate 타입 추가

```rust
// try_match.rs:102 아래
#[derive(Debug, Clone)]
pub struct PoisonedCandidate {
    pub player_id: String,
    pub reason: String,
}
```

#### 2. pop_candidates 반환 타입 변경

```rust
// try_match.rs:20
pub async fn pop_candidates(
    queue_suffix: &str,
    batch_size: usize,
    deps: &MatchmakerDeps,
) -> RedisResult<(Vec<PlayerCandidate>, Vec<PoisonedCandidate>)> {
    if batch_size == 0 {
        return Ok((Vec::new(), Vec::new()));
    }

    let mut redis = deps.redis.clone();
    let hash_tag = format!("{{{}}}", queue_suffix);
    let queue_key = format!("queue:{}", hash_tag);

    let raw: Vec<String> = invoke_try_match_script(&mut redis, queue_key, batch_size).await?;

    if raw.len() % 3 != 0 {
        return Err(RedisError::from((
            ErrorKind::TypeError,
            "unexpected response length",
        )));
    }

    let mut candidates = Vec::with_capacity(raw.len() / 3);
    let mut poisoned = Vec::new();

    for chunk in raw.chunks_exact(3) {
        let player_id = chunk[0].clone();
        let score = chunk[1].parse::<i64>().unwrap_or(0);
        let metadata_json = chunk[2].clone();

        // 파싱 시도
        match parse_candidate(&player_id, score, &metadata_json) {
            Ok(candidate) => candidates.push(candidate),
            Err(reason) => {
                warn!("Poisoned candidate {}: {}", player_id, reason);
                poisoned.push(PoisonedCandidate { player_id, reason });
                // ✅ 계속 진행! (전체 실패 X)
            }
        }
    }

    // 메트릭 기록
    if !poisoned.is_empty() {
        error!("Found {} poisoned candidates in queue {}", poisoned.len(), queue_suffix);
        deps.metrics.poisoned_candidates.inc_by(poisoned.len() as u64);
    }

    Ok((candidates, poisoned))
}

fn parse_candidate(
    player_id: &str,
    score: i64,
    metadata_json: &str,
) -> Result<PlayerCandidate, String> {
    let metadata = serde_json::from_str::<serde_json::Value>(metadata_json)
        .map_err(|e| format!("Invalid JSON: {}", e))?;

    let pod_id = metadata
        .get("pod_id")
        .and_then(|p| p.as_str())
        .ok_or_else(|| "pod_id not found".to_string())?;

    Ok(PlayerCandidate {
        player_id: player_id.to_string(),
        score,
        pod_id: pod_id.to_string(),
        metadata,
    })
}
```

#### 3. Handler 업데이트

```rust
// rank/handlers.rs:123
match pop_candidates(queue_suffix, required_players * 2, &deps).await {
    Ok((candidates, poisoned)) => {
        // Poisoned는 로깅만 (이미 큐에서 제거됨)
        for p in poisoned {
            error!("Dropped poisoned candidate {}: {}", p.player_id, p.reason);
        }
        break candidates;
    }
    Err(err) => { /* retry */ }
}
```

### 효과

- 오염된 플레이어 1명이 전체 매칭을 막지 못함
- 정상 플레이어는 계속 매칭됨
- 오염된 데이터 추적 가능 (메트릭)

---

## P1: Game Server 모니터링 & Alert

### 문제

Game Server가 죽어있어도 감지하지 못합니다.

```rust
// try_match.rs:93
// TODO: subscriber_count 를 활용하여 Game Server 생존 여부 확인, 오류 전파 등 구현해야함.

// rank/handlers.rs:182-185
if subscriber_count == 0 {
    // TODO: Game Server 가 구독중이지 않음.
    warn!("No Game Server is subscribed to battle:request channel");
    // ✅ re-enqueue는 이미 구현됨
}
```

**문제**: 로그만 찍고 끝. 10분간 Game Server 다운되어도 모름.

### 해결책

#### 1. 모니터링 구조체 추가

```rust
// matchmaker/common.rs
use std::sync::atomic::{AtomicU64, Ordering};

pub struct GameServerHealthMonitor {
    no_subscriber_count: AtomicU64,
    last_alert_timestamp: AtomicU64,
}

impl GameServerHealthMonitor {
    pub fn new() -> Self {
        Self {
            no_subscriber_count: AtomicU64::new(0),
            last_alert_timestamp: AtomicU64::new(0),
        }
    }

    pub fn record_no_subscriber(&self) {
        use chrono::Utc;

        let count = self.no_subscriber_count.fetch_add(1, Ordering::Relaxed) + 1;
        let now = Utc::now().timestamp() as u64;
        let last_alert = self.last_alert_timestamp.load(Ordering::Relaxed);

        // 연속 10번 실패 + 5분마다 알림
        if count >= 10 && now - last_alert > 300 {
            error!(
                "CRITICAL: Game Server unavailable for {} consecutive attempts! \
                 No subscribers on battle:request channel.",
                count
            );
            // TODO: Slack/PagerDuty/Email 알림
            self.last_alert_timestamp.store(now, Ordering::Relaxed);
        } else if count % 5 == 0 {
            warn!("Game Server unavailable count: {}", count);
        }
    }

    pub fn record_has_subscriber(&self, count: usize) {
        let previous = self.no_subscriber_count.swap(0, Ordering::Relaxed);

        if previous > 0 {
            info!(
                "Game Server recovered! {} subscriber(s) available. \
                 (Was down for {} attempts)",
                count, previous
            );
        }
    }
}

pub struct MatchmakerInner {
    pub game_server_monitor: Arc<GameServerHealthMonitor>,
    // ...
}
```

#### 2. Handler에서 사용

```rust
// rank/handlers.rs:181
match publish_battle_request(&mut redis, channel, &request).await {
    Ok(subscriber_count) => {
        if subscriber_count == 0 {
            deps.game_server_monitor.record_no_subscriber();
            warn!("No Game Server subscribed");
            re_enqueue_candidates(...).await;
        } else {
            deps.game_server_monitor.record_has_subscriber(subscriber_count);
            info!("Battle request sent to {} server(s)", subscriber_count);
        }
    }
    Err(err) => {
        error!("Failed to publish: {}", err);
        re_enqueue_candidates(...).await;
    }
}
```

#### 3. 메트릭 추가 (선택)

```rust
// metrics/src/lib.rs
pub struct MetricsCtx {
    pub game_server_unavailable_total: Counter,
    pub game_server_available: Gauge,
    // ...
}
```

### 효과

- Game Server 장애 즉시 감지 (10번 연속 실패)
- 5분마다 CRITICAL 알림 (중복 방지)
- 복구 시 자동 감지 및 로깅
- 운영팀이 즉시 대응 가능

---

## P2: Circuit Breaker Pattern

### 문제

Redis가 죽어도 계속 재시도합니다.

```rust
// rank/handlers.rs:116-148
let candidates = loop {
    match pop_candidates(...).await {
        Ok(candidates) => break candidates,
        Err(err) => {
            if let Some(delay) = backoff.next_backoff() {
                sleep(delay).await;
                continue;  // 무한 재시도 (Redis 복구될 때까지)
            }
        }
    }
};
```

**시나리오**: Redis 죽음 → 모든 TryMatch가 재시도 → CPU/로그 폭증

### 해결책

#### 1. Circuit Breaker 구현

```rust
// matchmaker/circuit_breaker.rs (신규)
use std::sync::atomic::{AtomicU64, Ordering};
use chrono::Utc;

pub struct CircuitBreaker {
    consecutive_failures: AtomicU64,
    threshold: u64,           // 예: 5번 연속 실패 시 차단
    open_until: AtomicU64,    // timestamp (열린 시각)
    cooldown_seconds: u64,    // 예: 60초
}

impl CircuitBreaker {
    pub fn new(threshold: u64, cooldown_seconds: u64) -> Self {
        Self {
            consecutive_failures: AtomicU64::new(0),
            threshold,
            open_until: AtomicU64::new(0),
            cooldown_seconds,
        }
    }

    pub fn check(&self) -> Result<(), String> {
        let now = Utc::now().timestamp() as u64;
        let open_until = self.open_until.load(Ordering::Relaxed);

        if open_until > now {
            let remaining = open_until - now;
            return Err(format!("Circuit open for {}s", remaining));
        }

        Ok(())
    }

    pub fn record_success(&self) {
        let previous = self.consecutive_failures.swap(0, Ordering::Relaxed);
        let was_open = self.open_until.swap(0, Ordering::Relaxed);

        if was_open > 0 {
            info!("Circuit breaker CLOSED (recovered after {} failures)", previous);
        }
    }

    pub fn record_failure(&self) {
        let failures = self.consecutive_failures.fetch_add(1, Ordering::Relaxed) + 1;

        if failures >= self.threshold {
            let now = Utc::now().timestamp() as u64;
            let open_until = now + self.cooldown_seconds;
            self.open_until.store(open_until, Ordering::Relaxed);

            error!(
                "Circuit breaker OPEN! {} consecutive failures. \
                 Blocking operations for {}s",
                failures, self.cooldown_seconds
            );
        }
    }
}
```

#### 2. MatchmakerInner에 추가

```rust
// matchmaker/common.rs
pub struct MatchmakerInner {
    pub redis_circuit: Arc<CircuitBreaker>,
    // ...
}

impl MatchmakerInner {
    pub fn new(...) -> Self {
        Self {
            redis_circuit: Arc::new(CircuitBreaker::new(5, 60)),
            // threshold=5, cooldown=60초
            // ...
        }
    }
}
```

#### 3. Handler에서 사용

```rust
// rank/handlers.rs:103
async move {
    if shutdown_token.is_cancelled() { return; }

    // Circuit breaker 체크
    if let Err(e) = deps.redis_circuit.check() {
        warn!("Redis circuit open, skipping TryMatch: {}", e);
        return;
    }

    let candidates = loop {
        match pop_candidates(...).await {
            Ok(candidates) => {
                deps.redis_circuit.record_success();
                break candidates;
            }
            Err(err) => {
                deps.redis_circuit.record_failure();

                if let Some(delay) = backoff.next_backoff() {
                    // ...
                } else {
                    return;
                }
            }
        }
    };
    // ...
}
```

### 효과

- 5번 연속 실패 시 60초간 자동 중단
- 불필요한 재시도 방지 (CPU/로그 절약)
- 60초 후 자동 복구 시도
- Redis 복구 시 즉시 정상화

---

## ✅ 이미 구현된 항목

### Re-enqueue on Failure

다음 경우들에서 자동 재enqueue가 구현되어 있습니다:

1. **Game Server 없음** (rank/handlers.rs:182-194)
```rust
if subscriber_count == 0 {
    re_enqueue_candidates(...).await;
}
```

2. **Publish 실패** (rank/handlers.rs:202-213)
```rust
Err(err) => {
    error!("Failed to publish: {}", err);
    re_enqueue_candidates(...).await;
}
```

3. **홀수 플레이어** (rank/handlers.rs:216-220)
```rust
[single] => {
    re_enqueue_candidates(...).await;
}
```

4. **Shutdown 중** (rank/handlers.rs:160-164)
```rust
if shutdown_token.is_cancelled() {
    re_enqueue_candidates(queue_suffix, settings.game_mode, chunk, &deps).await;
    continue;
}
```

---

## 구현 순서 권장

### Phase 1: 프로덕션 필수 (P0)
1. ✅ **Redis Timeout 보호** - 1~2시간
2. ✅ **Max In-Flight Limit** - 30분

### Phase 2: 운영 안정화 (P1)
3. ✅ **Poison Message 처리** - 2~3시간
4. ✅ **Game Server Monitor** - 1시간

### Phase 3: 고급 최적화 (P2)
5. **Circuit Breaker** - 2시간 (선택)

---

## 테스트 시나리오

### Timeout 테스트
```bash
# Redis 중단
docker stop redis

# 로그 확인: 10초 후 timeout 에러 발생하는지
tail -f logs/match_server.log | grep timeout
```

### In-Flight 테스트
```bash
# Redis에 sleep 추가 (느린 응답 시뮬레이션)
# 로그에서 "TryMatch already running, skipping" 확인
```

### Poison Message 테스트
```bash
# 수동으로 잘못된 metadata 추가
redis-cli ZADD "queue:{normal}" $(date +%s) "bad_player"
redis-cli SET "metadata:bad_player" '{"invalid": "no_pod_id"}'

# 로그 확인: poisoned candidate 로그, 다른 플레이어는 정상 매칭
```

### Game Server 모니터링 테스트
```bash
# Game Server 중단
# 10번 연속 매칭 시도 후 CRITICAL 로그 확인
```

---

## 참고 자료

- [tokio::select! 문서](https://docs.rs/tokio/latest/tokio/macro.select.html)
- [tokio::time::timeout 문서](https://docs.rs/tokio/latest/tokio/time/fn.timeout.html)
- [Circuit Breaker Pattern](https://learn.microsoft.com/en-us/azure/architecture/patterns/circuit-breaker)
- [Poison Message Pattern](https://www.enterpriseintegrationpatterns.com/patterns/messaging/InvalidMessageChannel.html)
