# Test Client Observer Verification Design

## 개요

Test Client의 ObserverActor가 각 PlayerBehavior의 행동이 서버에 올바르게 반영되었는지 실시간으로 검증하는 설계 문서입니다. 특히 normal.rs의 loading_complete 메시지 전송 버그와 같은 문제를 조기에 탐지하기 위한 구조입니다.

## 문제 상황

### 발견된 버그: normal.rs
```rust
// normal.rs의 on_loading_start에서
let _msg = ClientMessage::LoadingComplete { loading_session_id };

// 🚨 이 부분이 주석 처리되어 실제로 메시지가 전송되지 않음
// ws_sink
//     .send(Message::Text(serde_json::to_string(&msg)?))
//     .await?;

info!("[{}] Normal player sent loading_complete", player_context.player_id);
// ❌ 로그는 출력되지만 실제로는 메시지가 전송되지 않음
```

### 결과
- 클라이언트는 "sent loading_complete" 로그 출력 ✅
- 하지만 실제로는 서버에 메시지 전송하지 않음 ❌
- Redis에서 플레이어 상태가 "loading"으로 그대로 남아있음 ❌
- 26초 후 타임아웃으로 매칭 실패 ❌

## 설계 아키텍처

### 1. 전체 흐름도

```
[ServerMessage] 
       ↓
[PlayerActor.StreamHandler] 
       ↓
[behavior.on_xxx()] → [BehaviorResponse(TestResult, Option<ExpectEvent>)]
       ↓
[PlayerActor.BehaviorFinished Handler]
       ↓ (ExpectEvent가 있으면)
[ObserverActor.ExpectEvent Handler] 
       ↓
[Matcher Closure 실행] → [Redis/서버 상태 검증]
       ↓ (검증 실패 시)
[panic!] → [테스트 즉시 실패]
```

### 2. 핵심 컴포넌트

#### PlayerActor StreamHandler
```rust
// player_actor/handler.rs:86-103
let response = match fut_msg {
    ServerMessage::StartLoading { loading_session_id } => {
        behavior.on_loading_start(&player_context, loading_session_id).await
    }
    // ... 다른 메시지들
};

player_context.addr.do_send(BehaviorFinished {
    response,           // BehaviorResponse(TestResult, Option<ExpectEvent>)
    original_message: msg,
});
```

#### BehaviorFinished Handler
```rust
// player_actor/handler.rs:106-144
fn handle(&mut self, msg: BehaviorFinished, ctx: &mut Self::Context) {
    let response: BehaviorResponse = msg.response;
    
    // 🎯 ExpectEvent가 있으면 ObserverActor에게 검증 요청
    if let Some(expected_event) = response.1 {
        self.observer.do_send(expected_event);
    }
    
    // TestResult 처리
    match response.0 {
        Ok(BehaviorOutcome::Continue) => { /* 계속 진행 */ }
        Ok(BehaviorOutcome::Stop) => { ctx.stop(); }
        Err(test_failure) => { /* 테스트 실패 처리 */ }
    }
}
```

#### ExpectEvent 구조
```rust
// observer_actor/message.rs:23-28
pub struct ExpectEvent {
    pub event_type: String,
    pub player_id: Option<Uuid>,
    pub data_matcher: Box<dyn Fn(&serde_json::Value) -> bool + Send + Sync>,
    pub timeout: Duration,
}
```

## Redis 상태 추적 메커니즘

### Match Server의 Loading Session 구조

**Redis 키:** `loading:{loading_session_id}`
**데이터 구조:** Hash
```
loading:306cbaae-7fa6-4829-8390-e7aaf5be9564
├── game_mode: "Normal_1v1"
├── created_at: "1642953600"
├── status: "loading"
├── player1_uuid: "loading"  ← 초기 상태
└── player2_uuid: "ready"    ← loading_complete 후 상태
```

### 상태 변화 과정
1. **매칭 성공** → `loading:{session_id}` 생성, 모든 플레이어 상태 = "loading"
2. **클라이언트가 loading_complete 전송** → 해당 플레이어 상태 = "ready"
3. **모든 플레이어가 ready** → 키 삭제 후 dedicated server 생성
4. **타임아웃 (60초)** → 키 삭제 후 플레이어들 재큐잉

## Behavior Method 수정 패턴

### 1. Normal Player (정상 케이스)
```rust
// behaviors/normal.rs
async fn on_loading_start(
    &self,
    player_context: &PlayerContext,
    loading_session_id: Uuid,
) -> BehaviorResponse {
    info!("[{}] Normal player starting to load assets", player_context.player_id);
    
    // 실제 loading_complete 메시지 전송
    let msg = ClientMessage::LoadingComplete { loading_session_id };
    // TODO: 실제 WebSocket 전송 코드 주석 해제 필요
    // ws_sink.send(Message::Text(serde_json::to_string(&msg)?)).await?;
    
    info!("[{}] Normal player sent loading_complete", player_context.player_id);
    
    // 🎯 Redis 상태 검증을 위한 ExpectEvent
    let redis_verification = ExpectEvent::new(
        "redis_verification".to_string(),
        Some(player_context.player_id),
        Box::new(move |_data| {
            // Redis에서 loading:{session_id}의 플레이어 상태 확인
            // "ready" 상태여야 정상, "loading" 상태면 메시지가 실제로 전송되지 않았음
            match check_redis_loading_status(loading_session_id, player_context.player_id) {
                RedisPlayerStatus::Ready => true,  // ✅ 정상
                RedisPlayerStatus::Loading => {
                    panic!("❌ LOADING_COMPLETE NOT SENT! Player {} claims to have sent loading_complete but Redis still shows 'loading' status", player_context.player_id);
                }
                RedisPlayerStatus::NotFound => {
                    panic!("❌ LOADING SESSION NOT FOUND! Session {} does not exist in Redis", loading_session_id);
                }
            }
        }),
        Duration::from_secs(3)  // 3초 후 검증
    );
    
    BehaviorResponse(Ok(BehaviorOutcome::Continue), Some(redis_verification))
}
```

### 2. Quit During Loading (의도적 종료)
```rust
// behaviors/quit.rs
async fn on_loading_start(
    &self,
    player_context: &PlayerContext,
    loading_session_id: Uuid,
) -> BehaviorResponse {
    warn!("[{}] Quitting during loading start!", player_context.player_id);
    
    // 🎯 플레이어가 큐에서 제거되었는지 검증
    let quit_verification = ExpectEvent::new(
        "player_removed_verification".to_string(),
        Some(player_context.player_id),
        Box::new(move |_data| {
            // Redis에서 플레이어가 loading session에서 제거되었는지 확인
            verify_player_removed_from_loading_session(loading_session_id, player_context.player_id)
        }),
        Duration::from_secs(2)
    );
    
    BehaviorResponse(
        Err(TestFailure::Behavior("Intentionally quit during loading".to_string())),
        Some(quit_verification)
    )
}
```

### 3. Slow Loader (지연 로딩)
```rust
// behaviors/slow.rs
async fn on_loading_start(
    &self,
    player_context: &PlayerContext,
    loading_session_id: Uuid,
) -> BehaviorResponse {
    warn!("[{}] Slow loader - waiting {} seconds", player_context.player_id, self.delay_seconds);
    
    // 의도적 지연
    tokio::time::sleep(tokio::time::Duration::from_secs(self.delay_seconds)).await;
    
    // 지연 후 loading_complete 전송
    let msg = ClientMessage::LoadingComplete { loading_session_id };
    // ws_sink.send(...).await;
    
    // 🎯 지연된 시점에서 Redis 상태 확인
    let delayed_verification = ExpectEvent::new(
        "delayed_loading_verification".to_string(),
        Some(player_context.player_id),
        Box::new(move |_data| {
            // 지연 로딩 후에도 정상적으로 ready 상태가 되었는지 확인
            verify_delayed_loading_completion(loading_session_id, player_context.player_id, self.delay_seconds)
        }),
        Duration::from_secs(2)
    );
    
    BehaviorResponse(Ok(BehaviorOutcome::Continue), Some(delayed_verification))
}
```

## ObserverActor 수정 사항

### 1. Redis 클라이언트 추가
```rust
// observer_actor/mod.rs
pub struct ObserverActor {
    pub match_server_url: String,
    pub expected_sequence: Vec<ExpectEvent>,
    pub received_events: Vec<EventStreamMessage>,
    pub current_step: usize,
    pub test_name: String,
    pub scenario_runner_addr: Addr<ScenarioRunnerActor>,
    pub redis_client: Option<redis::aio::ConnectionManager>, // 🆕 Redis 클라이언트 추가
}
```

### 2. Redis 검증 헬퍼 함수들
```rust
// observer_actor/redis_verification.rs (새 파일)
use redis::AsyncCommands;
use uuid::Uuid;

#[derive(Debug, PartialEq)]
pub enum RedisPlayerStatus {
    Loading,
    Ready,
    NotFound,
}

pub async fn check_redis_loading_status(
    redis_client: &mut redis::aio::ConnectionManager,
    loading_session_id: Uuid,
    player_id: Uuid,
) -> RedisPlayerStatus {
    let loading_key = format!("loading:{}", loading_session_id);
    
    match redis_client.hget::<_, _, Option<String>>(&loading_key, player_id.to_string()).await {
        Ok(Some(status)) => match status.as_str() {
            "loading" => RedisPlayerStatus::Loading,
            "ready" => RedisPlayerStatus::Ready,
            _ => RedisPlayerStatus::NotFound,
        },
        Ok(None) => RedisPlayerStatus::NotFound,
        Err(_) => RedisPlayerStatus::NotFound,
    }
}

pub async fn verify_player_removed_from_loading_session(
    redis_client: &mut redis::aio::ConnectionManager,
    loading_session_id: Uuid,
    player_id: Uuid,
) -> bool {
    let loading_key = format!("loading:{}", loading_session_id);
    
    // 세션 자체가 삭제되었거나, 플레이어가 세션에서 제거되었으면 성공
    match redis_client.hexists::<_, _, bool>(&loading_key, player_id.to_string()).await {
        Ok(false) => true,  // 플레이어가 세션에서 제거됨
        Ok(true) => false,  // 여전히 세션에 남아있음 (문제)
        Err(_) => true,     // 세션 자체가 없음 (정상)
    }
}
```

## 검증 시점과 타이밍

### 1. 검증 타이밍 설계
- **즉시 검증**: 행동 완료 직후 (1-2초 내)
- **지연 검증**: 의도적 지연 행동의 경우 (SlowLoader 등)
- **타임아웃**: 각 검증마다 적절한 타임아웃 설정

### 2. 검증 실패 처리
```rust
// ExpectEvent의 matcher closure에서
Box::new(move |_data| {
    match verify_condition() {
        Ok(true) => {
            info!("✅ Verification passed: {}", description);
            true
        }
        Ok(false) => {
            panic!("❌ VERIFICATION FAILED: {}", error_description);
        }
        Err(e) => {
            panic!("❌ VERIFICATION ERROR: {}", e);
        }
    }
})
```

## 이점 및 효과

### 1. 조기 버그 탐지
- 코드에서 의도한 행동과 실제 서버 상태 불일치를 즉시 발견
- normal.rs의 주석 처리된 WebSocket 전송 같은 버그를 3초 내에 탐지

### 2. 테스트 신뢰성 향상
- 로그 출력만으로 성공을 판단하지 않고 실제 서버 상태 확인
- 각 behavior의 의도된 동작이 올바르게 수행되었는지 보장

### 3. 디버깅 효율성
- 문제 발생 시 정확한 원인과 위치를 즉시 파악 가능
- Redis 상태와 클라이언트 행동의 불일치를 명확하게 표시

### 4. 확장 가능성
- 새로운 behavior 추가 시 동일한 패턴으로 검증 로직 구현
- 다양한 서버 상태 (큐, 매칭, 게임 세션 등) 검증으로 확장 가능

## 구현 우선순위

1. **Phase 1**: ObserverActor에 Redis 클라이언트 추가
2. **Phase 2**: normal.rs의 ExpectEvent 구현 및 테스트
3. **Phase 3**: 다른 behavior들의 ExpectEvent 구현
4. **Phase 4**: Redis 검증 헬퍼 함수들 완성
5. **Phase 5**: 전체 시스템 통합 테스트

## 주의사항

1. **Redis 연결 관리**: 적절한 connection pooling 및 에러 처리
2. **타이밍 이슈**: 서버 처리 시간을 고려한 적절한 검증 지연
3. **동시성**: 여러 플레이어의 동시 검증에서 발생할 수 있는 race condition
4. **테스트 격리**: 각 테스트 시나리오 간 Redis 상태 간섭 방지