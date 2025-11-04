# Game Server 아키텍처 현황 문서

> **작성일**: 2025-10-23
> **버전**: 1.1
> **목적**: 마이그레이션 작업 현황 및 현재 구현 상태 통합 정리

---

## 📋 목차

1. [개요](#개요)
2. [게임 설계 개념](#게임-설계-개념)
3. [서비스 구조](#서비스-구조)
4. [현재 구현 상태](#현재-구현-상태)
5. [통신 흐름](#통신-흐름)
6. [완료된 작업](#완료된-작업)
7. [미완료 작업](#미완료-작업)
8. [TODO: 매칭 시스템 개선](#todo-매칭-시스템-개선-ghost-시스템)
9. [다음 단계](#다음-단계)

---

## 개요

### 프로젝트 개요

**온라인 1vs1 카드 게임**
- **게임 서버**: Rust (Actix Actor 모델)
- **클라이언트**: Unity
- **게임 연산**: 보안을 위해 Game Server에서 전부 처리
- **클라이언트 역할**: 연산 결과 시각화만 담당

### 게임 장르

Day 기반 턴제 로그라이크 카드 게임
- 이벤트 선택 → PvE 전투 → PvP 매칭 → 다음 Day
- 상점, 골드, 환상체, 퀘스트 등 이벤트
- 경험치 획득 → 레벨업 시스템

---

## 게임 설계 개념

### 게임 진행 흐름

```
Day 1 시작
  ├─ 이벤트 선택 (상점, 골드, 환상체, 퀘스트 등 중 3개 랜덤)
  ├─ 이벤트 선택
  ├─ PvE 전투
  ├─ 이벤트 선택
  └─ PvP 매칭 → 자동 전투
      ↓
Day 2 시작
  ├─ 이벤트 선택
  ├─ 이벤트 선택
  ├─ PvE 전투
  ├─ 이벤트 선택
  └─ PvP 매칭 → 자동 전투
      ↓
반복...
```

### 이벤트 종류

- **상점 입장** - 아이템/카드 구매
- **골드 획득** - 재화 획득
- **환상체 획득** - 특수 능력
- **퀘스트** - 미션 수행
- **기타** - 향후 확장

### 레벨업 시스템

- 특정 행동 또는 시간 경과로 경험치 획득
- 경험치 일정량 누적 → 레벨업
- 전략적 투자 가능 (스탯, 스킬 등)

---

## 서비스 구조

### 서비스 분류

#### 독립적 서비스 (단일 인스턴스)
- **Redis Server** - 메시지 발행/관리, 큐 관리
- **Auth Server** - 플레이어 인증, 고유 키 발급

#### 비독립적 서비스 (Pod 단위, 복수 인스턴스)
- **Game Server** - 플레이어 게임 진행 관리
- **Match Server** - PvP 매칭 처리 (현재 Game Server에 통합됨)

---

## 현재 구현 상태

### 프로젝트 구조

```
game_server/  (Match Server 통합 완료)
├── src/
│   ├── main.rs                    ✅ 서버 진입점
│   ├── lib.rs                     ✅ AppState, 공통 모듈
│   │
│   ├── game/                      [신규 - Unity Client용]
│   │   ├── battle_actor/          ✅ 전투 시뮬레이션 (순수 함수)
│   │   ├── load_balance_actor/    ✅ PlayerGameActor 라우팅
│   │   ├── match_coordinator/     ✅ 매칭 요청 조정 (사용 안 됨)
│   │   ├── player_game_actor/     ⚠️ stub (빈 구조체)
│   │   └── pubsub.rs             ✅ Redis 구독 (match_result, game_message)
│   │
│   ├── matchmaking/              [레거시 - test_client용]
│   │   ├── session/              ✅ WebSocket 세션 관리
│   │   ├── subscript/            ✅ Session 라우팅
│   │   └── matchmaker/           ✅ 매칭 로직
│   │       ├── normal/           ✅ 일반 매칭
│   │       ├── rank/             ✅ 랭크 매칭
│   │       └── operations/       ✅ Enqueue, Dequeue, TryMatch
│   │           ├── try_match.rs              ✅ Candidates 수집
│   │           ├── try_match_collect.rs      ✅ 재시도 로직
│   │           ├── try_match_process.rs      ✅ 매칭 처리 + Battle 실행
│   │           ├── enqueue.rs                ✅ Redis 큐 추가
│   │           ├── dequeue.rs                ✅ Redis 큐 제거
│   │           └── notify.rs                 ✅ Same/Cross-pod 라우팅
│   │
│   └── shared/                   [공유 인프라]
│       ├── protocol.rs           ✅ 메시지 정의
│       ├── metrics.rs            ✅ Prometheus 메트릭
│       ├── circuit_breaker.rs    ✅ Redis 장애 격리
│       ├── event_stream.rs       ✅ 이벤트 스트리밍
│       └── redis_events.rs       ✅ 테스트 이벤트 발행
│
└── config/
    ├── development.toml          ✅ 개발 환경 설정
    └── production.toml           ✅ 운영 환경 설정
```

### 액터 구조

```
┌─────────────────────────────────────────────────────────┐
│ Game Server (Actix Actor System)                       │
├─────────────────────────────────────────────────────────┤
│                                                         │
│  [레거시 경로 - test_client]                             │
│  ┌─────────────────────────────────────────────┐       │
│  │ /ws/ → Session Actor                        │       │
│  │          ↓                                  │       │
│  │    SubScriptionManager                      │       │
│  │          ↓                                  │       │
│  │    Matchmaker (Normal/Ranked)               │       │
│  └─────────────────────────────────────────────┘       │
│                                                         │
│  [신규 경로 - Unity Client] ⚠️ 미완성                   │
│  ┌─────────────────────────────────────────────┐       │
│  │ /game → PlayerGameActor (stub)              │       │
│  │          ↓                                  │       │
│  │    MatchCoordinator (구현됨, 사용 안 됨)      │       │
│  │          ↓                                  │       │
│  │    Matchmaker                               │       │
│  └─────────────────────────────────────────────┘       │
│                                                         │
│  [공유 인프라]                                           │
│  ┌─────────────────────────────────────────────┐       │
│  │ LoadBalanceActor                            │       │
│  │   └─ HashMap<Uuid, Addr<PlayerGameActor>>  │       │
│  │                                             │       │
│  │ Matchmaker (Normal/Ranked)                  │       │
│  │   ├─ TryMatch (주기적 실행)                  │       │
│  │   ├─ Enqueue/Dequeue                        │       │
│  │   └─ Battle 실행 + 결과 라우팅               │       │
│  │                                             │       │
│  │ Redis Subscribers                           │       │
│  │   ├─ match_result 채널                      │       │
│  │   └─ pod:{pod_id}:game_message 채널        │       │
│  └─────────────────────────────────────────────┘       │
└─────────────────────────────────────────────────────────┘
```

---

## 통신 흐름

### 1. 플레이어 접속 (현재: test_client만)

```
test_client
  │ Auth Token
  ▼
Game Server (/ws/)
  │ 토큰 검증
  ▼
Session Actor 생성
  │
  ├─ 신규 플레이어: 새로운 Session 생성
  │   └─ SubScriptionManager 등록
  │
  └─ 기존 플레이어: (재접속 로직 없음)
      └─ 새로운 Session 생성
```

**⚠️ 목표 (Unity Client - 미구현):**
```
Unity Client
  │ Auth Token
  ▼
Game Server (/game)
  │ 토큰 검증
  ▼
LoadBalanceActor 조회
  │
  ├─ 신규 플레이어: PlayerGameActor 생성
  │   └─ LoadBalanceActor 등록
  │
  └─ 기존 플레이어: 기존 PlayerGameActor 찾기
      └─ WebSocket 재수립
```

### 2. PvP 매칭 흐름 (현재 구현)

```
┌─────────────────────────────────────────────────────────┐
│ Phase 1: Enqueue (플레이어 큐 등록)                       │
└─────────────────────────────────────────────────────────┘

test_client
  │ {"type": "enqueue", "game_mode": "Normal", "metadata": "..."}
  ▼
Session Actor
  │ handle_enqueue()
  ▼
Matchmaker (Normal/Ranked)
  │ Lua Script: ENQUEUE_PLAYER.lua
  ▼
Redis
  ├─ ZADD queue:{mode} {timestamp} {player_id}
  └─ SET metadata:{player_id} {json}
  ▼
Session Actor
  │ ServerMessage::EnQueued
  ▼
test_client (대기 중...)

┌─────────────────────────────────────────────────────────┐
│ Phase 2: TryMatch (주기적 매칭 시도 - 5초마다)            │
└─────────────────────────────────────────────────────────┘

Matchmaker (TryMatch)
  │ Lua Script: TRY_MATCH_POP.lua
  ▼
Redis
  │ ZPOPMIN queue:{mode} {batch_size}
  │ GET metadata:{player_id} ...
  │ DEL metadata:{player_id} ...
  ▼
Matchmaker
  │ candidates = [player1@pod-a, player2@pod-b]
  ▼
process_match_pair()
  │
  ├─ execute_battle(player1, player2)
  │   └─ BattleResult {winner_id, battle_data}
  │
  └─ notify_match_found_with_result()
      │
      ├─ player1: Same-pod?
      │   ├─ YES → LoadBalanceActor.do_send() ⚡ 0.1ms
      │   │        └─ PlayerGameActor (미구현)
      │   │        └─ SubScriptionManager (레거시)
      │   │             └─ Session Actor → test_client
      │   │
      │   └─ NO  → Redis PUBLISH("pod:{pod_id}:game_message") 🌐 5-10ms
      │            └─ 대상 Pod Game Server 구독
      │                 └─ LoadBalanceActor.do_send()
      │
      └─ player2: (동일한 로직)

┌─────────────────────────────────────────────────────────┐
│ Phase 3: Battle 결과 수신                                │
└─────────────────────────────────────────────────────────┘

test_client
  │ ServerMessage::MatchFound {winner_id, opponent_id, battle_data}
  ▼
게임 결과 표시
  └─ WebSocket 종료 (MatchFound 수신 시 자동 종료)
```

### 3. Battle 실행 (즉시 실행 방식)

```
Matchmaker (매칭 성사)
  │ [player1@pod-a, player2@pod-b]
  ▼
battle_actor::execute_battle()  ← 순수 함수 (Actor 아님)
  │
  ├─ simulate_battle()
  │   └─ 승자 결정 (현재: player1 항상 승리 - stub)
  │
  └─ BattleResult {
        winner_id: "player1_id",
        battle_data: {...}
      }
  ▼
결과 라우팅 (player1, player2 각각)
  │
  ├─ Same-pod: LoadBalanceActor → Actor 메시지 (0.1ms)
  └─ Cross-pod: Redis Pub/Sub (5-10ms)
```

**핵심 특징:**
- ✅ Actor가 아닌 **순수 함수** 사용
- ✅ Matchmaker가 **즉시 실행** (Redis 홉 없음)
- ✅ 동기적 결과 대기
- ✅ BATTLE_ACTOR_REFACTORING_PLAN.md 설계 완료

---

## 완료된 작업

### ✅ Match Server → Game Server 통합

**상태:** 완료 (2025-10-22)

- Match Server 코드를 game_server 프로젝트로 통합
- 단일 프로세스로 동작 (별도 Match Server 프로세스 불필요)
- Pod당 하나의 game_server 실행

### ✅ TryMatch 리팩토링

**상태:** 완료 (2025-10-22)

**변경사항:**
- TryMatch 핸들러: 353 lines → **80 lines (78% 감소)**
- 함수 분리: `try_match_collect.rs`, `try_match_process.rs`
- 가독성 개선, 테스트 가능성 향상

**파일:**
- `operations/try_match_collect.rs` (~100 lines)
  - `collect_candidates_with_retry()` - Candidates 수집, 재시도
  - `notify_poisoned_candidates()` - 오염된 후보 알림

- `operations/try_match_process.rs` (~150 lines)
  - `process_match_pair()` - 매칭 처리 + Battle 실행
  - `notify_match_found_with_result()` - 결과 전달

### ✅ Battle 즉시 실행 방식

**상태:** 완료 (2025-10-22)

**변경사항:**
- Redis `battle:request` 채널 제거
- 항상 로컬에서 Battle 실행 (순수 함수)
- Same-pod/Cross-pod 구분은 결과 라우팅에만 적용

**장점:**
- Redis 홉 1개 제거 (50% 지연 감소)
- Cross-pod 지연: 15-20ms → **5-10ms**
- 코드 간소화: 300 lines → **150 lines**

### ✅ Same-pod/Cross-pod 라우팅

**상태:** 완료 (2025-10-22)

**구현:**
```rust
// notify.rs
pub async fn send_message_to_player(player: &PlayerCandidate, ...) {
    if player.is_same_pod() {
        // Same-pod: Actor 메시지 (0.1ms)
        LoadBalanceActor.do_send(RouteToPlayer { ... });
    } else {
        // Cross-pod: Redis Pub/Sub (5-10ms)
        Redis PUBLISH("pod:{pod_id}:game_message", ...);
    }
}
```

**메트릭:**
- `MESSAGES_ROUTED_SAME_POD_TOTAL`
- `MESSAGES_ROUTED_CROSS_POD_TOTAL`
- `MATCHES_SAME_POD_TOTAL`
- `MATCHES_CROSS_POD_TOTAL`

### ✅ 신규 Actor 구현

**LoadBalanceActor:**
```rust
pub struct LoadBalanceActor {
    players: HashMap<Uuid, Addr<PlayerGameActor>>,
    metrics: Arc<MetricsCtx>,
}
```
- ✅ player_id → PlayerGameActor 매핑
- ✅ 메시지 라우팅 (`RouteToPlayer`)
- ⚠️ PlayerGameActor가 stub이라 실제 사용 안 됨

**MatchCoordinator:**
```rust
pub struct MatchCoordinator {
    matchmakers: HashMap<GameMode, MatchmakerAddr>,
    load_balance_addr: Addr<LoadBalanceActor>,
    redis: ConnectionManager,
}
```
- ✅ GameMode별 Matchmaker 라우팅
- ✅ 서버에서 metadata 생성 (보안)
- ⚠️ 호출하는 코드 없음 (Unity Client 대기)

### ✅ Redis Pub/Sub 구독

**game/pubsub.rs:**
```rust
spawn_redis_subscribers(...)
  ├─ subscribe_match_result_channel()      // "match_result" 구독
  └─ subscribe_game_message_channel()      // "pod:{pod_id}:game_message" 구독
```

- ✅ Circuit Breaker 적용
- ✅ Exponential Backoff 재시도
- ✅ Graceful Shutdown 지원
- ✅ LoadBalanceActor로 메시지 라우팅

### ✅ 메트릭 수집

**구현된 메트릭:**
```rust
// Matchmaking
MATCHES_CREATED_TOTAL
MATCHES_SAME_POD_TOTAL
MATCHES_CROSS_POD_TOTAL
MATCHED_PLAYERS_TOTAL_BY_MODE

// Routing
MESSAGES_ROUTED_SAME_POD_TOTAL
MESSAGES_ROUTED_CROSS_POD_TOTAL

// Redis
POISONED_CANDIDATES_TOTAL
GAME_SERVER_AVAILABLE
GAME_SERVER_UNAVAILABLE_TOTAL

// TryMatch
TRY_MATCH_SKIPPED_TOTAL
```

---

## 미완료 작업

### ❌ PlayerGameActor 구현

**현재 상태:**
```rust
pub struct PlayerGameActor {}  // 빈 구조체
```

**필요한 구현:**
```rust
pub struct PlayerGameActor {
    player_id: Uuid,
    state: PlayerState,  // Lobby, InQueue, InBattle, ...

    // 게임 상태
    day: u32,
    level: u32,
    gold: u32,
    deck: DeckBuild,
    items: Vec<Item>,
    artifacts: Vec<Artifact>,

    // Actor 주소
    match_coordinator_addr: Addr<MatchCoordinator>,
    load_balance_addr: Addr<LoadBalanceActor>,

    // 인프라
    redis: ConnectionManager,
    metrics: Arc<MetricsCtx>,
}

pub enum PlayerState {
    Lobby,           // 로비
    EventSelection,  // 이벤트 선택 중
    InShop,          // 상점
    InPvE,           // PvE 전투
    InQueue,         // PvP 큐 대기
    InPvP,           // PvP 전투
}
```

**필요한 핸들러:**
- Day 진행 관리
- 이벤트 선택 (상점, 골드, 환상체, 퀘스트)
- PvE 전투
- PvP 매칭 요청 (MatchCoordinator 호출)
- 레벨업 시스템
- 아이템/카드 관리

### ❌ Unity Client WebSocket 엔드포인트

**현재:** 없음

**필요한 구현:**
```rust
// main.rs
#[get("/game")]
async fn player_game_ws_route(
    req: HttpRequest,
    stream: web::Payload,
    state: web::Data<AppState>,
) -> Result<HttpResponse, Error> {
    // 1. Auth Token 검증
    let auth_token = extract_auth_token(&req)?;
    let player_id = verify_token_with_auth_server(&auth_token).await?;

    // 2. 기존 PlayerGameActor 찾기 (재접속)
    let player_actor = state
        .load_balance_addr
        .send(FindPlayer { player_id })
        .await?;

    let player_actor = match player_actor {
        Some(actor) => {
            info!("재접속: player {}", player_id);
            actor
        }
        None => {
            info!("신규 접속: player {}", player_id);
            // 3. PlayerGameActor 생성
            let actor = PlayerGameActor::new(
                player_id,
                state.match_coordinator_addr.clone(),
                state.load_balance_addr.clone(),
                state.redis.clone(),
                state.metrics.clone(),
            ).start();

            // 4. LoadBalanceActor 등록
            state.load_balance_addr.do_send(RegisterPlayer {
                player_id,
                addr: actor.clone(),
            });

            actor
        }
    };

    // 5. WebSocket 시작
    ws::start(player_actor, &req, stream)
}
```

### ❌ PlayerGameActor ↔ MatchCoordinator 연동

**현재:** MatchCoordinator 사용 안 됨

**필요한 구현:**
```rust
impl PlayerGameActor {
    /// PvP 매칭 진입
    async fn enter_pvp_queue(&self, game_mode: GameMode) -> Result<()> {
        // 1. 준비도 검증
        if !self.is_ready_for_pvp() {
            return Err("덱이 준비되지 않음");
        }

        // 2. 상태 변경
        self.state = PlayerState::InQueue;

        // 3. MatchCoordinator에 Enqueue 요청
        self.match_coordinator_addr
            .send(EnqueuePlayer {
                player_id: self.player_id,
                game_mode,
            })
            .await?;

        Ok(())
    }

    /// 매칭 결과 수신 (LoadBalanceActor에서 라우팅됨)
    fn handle_match_found(&mut self, msg: ServerMessage::MatchFound) {
        self.state = PlayerState::InPvP;

        // 클라이언트에 전달
        self.send_to_client(msg);
    }
}
```

### ❌ 게임 진행 로직

**필요한 구현:**
- Day 시작/종료
- 이벤트 선택 (상점, 골드, 환상체, 퀘스트)
- PvE 전투 시스템
- 레벨업 시스템
- 덱 빌딩
- 아이템/환상체 관리

### ❌ Auth Server 연동

**현재:** 없음

**필요한 구현:**
```rust
async fn verify_token_with_auth_server(token: &str) -> Result<Uuid, Error> {
    // Auth Server API 호출
    let client = reqwest::Client::new();
    let response = client
        .post("http://auth-server/verify")
        .json(&json!({"token": token}))
        .send()
        .await?;

    if response.status().is_success() {
        let data: AuthResponse = response.json().await?;
        Ok(data.player_id)
    } else {
        Err(Error::Unauthorized)
    }
}
```

### ❌ 실제 Battle 로직

**현재:**
```rust
async fn simulate_battle(...) -> String {
    // TODO: 실제 battle 로직 구현
    player1.player_id.clone()  // 임시로 player1 승리
}
```

**필요한 구현:**
- 카드 덱 기반 전투 시뮬레이션
- 턴제 전투 로직
- 스킬/아이템/환상체 효과 적용
- 전투 타임라인 생성 (클라이언트 재생용)

---

## TODO: 매칭 시스템 개선 (Ghost 시스템)

### ❌ 실시간 매칭 → Ghost 스냅샷 기반 매칭 (우선순위: 높음)

**현재 방식 (문제점):**
- 두 명의 플레이어가 **동시에 큐에 진입**해야 매칭 가능
- PvP 단계에 도달한 플레이어가 즉시 큐에 진입 → 상대를 기다림
- 매칭 대기 시간 발생
- 동시 접속자가 적으면 매칭이 안됨

**개선 방안 (The Bazaar Ghost 시스템):**
```rust
// Hour 5 (PvP) 도달 시:
// 1. 플레이어 스냅샷을 Redis에 저장
ZADD player_snapshots:day_{day}:mmr_{mmr_range} {timestamp} {snapshot_json}

// 2. 스냅샷 풀에서 즉시 매칭
let ghost = ZRANDMEMBER player_snapshots:day_{day}:mmr_{mmr_range} 1

// 3. Ghost와 전투 수행 (비동기)
execute_battle(player, ghost)

// 4. 결과 저장
PUBLISH player:{player_id}:battle_result {result}
```

**장점:**
- ✅ 매칭 대기 시간 **거의 0**
- ✅ 동시 접속자 무관 (과거 스냅샷 활용)
- ✅ 비동기 플레이 가능 (새벽/낮 상관없이 매칭)
- ✅ 서버 부하 분산 (큐 관리 단순화)

**구현 사항:**

1. **스냅샷 저장 시점:**
   - Hour 5 (PvP) 도달 시 플레이어 덱 스냅샷 Redis 저장
   - Key: `player_snapshots:day_{day}:mmr_{mmr_range}`
   - Value: `{player_id, deck, level, items, artifacts, ...}`
   - Score: timestamp
   - TTL: 24-48시간

2. **매칭 로직:**
   - 큐 진입 대신 스냅샷 풀에서 랜덤 선택
   - 같은 Day, 비슷한 MMR 필터링
   - 최근 스냅샷 우선 (24시간 이내)

3. **Dequeue 시점:**
   - ❌ 매칭 성사 시 (현재 방식)
   - ✅ 게임 완전 종료 시 (승리/패배/포기)
   - ✅ 중도 포기 시 (타임아웃, 연결 끊김)
   - ✅ Run 종료 시

   ```rust
   // 게임 종료 시 스냅샷 제거
   async fn on_game_end(player_id: Uuid, day: u32, mmr: u32) {
       let key = format!("player_snapshots:day_{}:mmr_{}", day, mmr_range(mmr));
       redis.zrem(key, player_id).await;
   }
   ```

4. **보상 처리:**
   - 단방향 처리 (Ghost는 보상 없음)
   - 또는 양방향 처리 (Ghost 주인에게 "방어 성공/실패" 알림)

5. **매칭 알고리즘:**
   ```rust
   async fn find_ghost_opponent(day: u32, mmr: u32) -> Option<PlayerSnapshot> {
       let key = format!("player_snapshots:day_{}:mmr_{}", day, mmr_range(mmr));

       // 최근 24시간 내 스냅샷만
       let now = unix_timestamp();
       let yesterday = now - 86400;

       redis.zrangebyscore(key, yesterday, now, RAND, 1).await
   }
   ```

**예상 시간:** 3-5일

**전제 조건:**
- PlayerGameActor 구현 (Day 진행 관리)
- Redis 스냅샷 저장/조회 로직

---

## 다음 단계

### Phase 1: PlayerGameActor 기본 구현 (우선순위: 높음)

**목표:** Unity Client 연결 가능하게 만들기

**작업 목록:**
1. PlayerGameActor 구조체 완성
   - 플레이어 상태 (day, level, gold, deck, etc.)
   - WebSocket 핸들러

2. /game 엔드포인트 구현
   - Auth Token 검증
   - PlayerGameActor 생성/재접속
   - LoadBalanceActor 등록

3. MatchCoordinator 연동
   - enter_pvp_queue() 구현
   - 매칭 결과 수신

**예상 시간:** 3-5일

### Phase 2: 게임 진행 로직 (우선순위: 중)

**목표:** Day 기반 게임 진행 구현

**작업 목록:**
1. Day 시스템
   - Day 시작/종료
   - 이벤트 선택 (3개 랜덤)

2. 이벤트 구현
   - 상점 (아이템/카드 구매)
   - 골드 획득
   - 환상체 획득
   - 퀘스트

3. PvE 전투 시스템
   - NPC 전투 로직

**예상 시간:** 1-2주

### Phase 3: 레벨업 및 진행 시스템 (우선순위: 중)

**목표:** 전략적 깊이 추가

**작업 목록:**
1. 경험치 시스템
2. 레벨업 보상
3. 스탯 투자
4. 덱 빌딩 시스템

**예상 시간:** 1주

### Phase 4: Auth Server 연동 (우선순위: 낮음)

**목표:** 실제 인증 시스템 연동

**작업 목록:**
1. Auth Server API 정의
2. Token 검증 로직
3. 플레이어 DB 연동

**예상 시간:** 3-5일

### Phase 5: Battle 로직 구현 (우선순위: 낮음)

**목표:** 실제 카드 전투 시스템

**작업 목록:**
1. 카드 덱 기반 전투
2. 턴제 시뮬레이션
3. 스킬/아이템 효과
4. 전투 타임라인 생성

**예상 시간:** 2-3주

### Phase 6: 레거시 제거 (우선순위: 낮음)

**목표:** test_client 경로 제거

**전제 조건:**
- Unity Client 안정 동작
- 충분한 검증 기간 (최소 1개월)

**작업 목록:**
1. /ws/ 엔드포인트 제거
2. Session Actor 제거
3. SubScriptionManager 제거 또는 역할 축소

**예상 시간:** 2-3일

---

## 기술 스택

### Backend (Game Server)
- **언어**: Rust
- **프레임워크**: Actix (Actor 모델)
- **웹 서버**: Actix-web
- **WebSocket**: actix-web-actors
- **비동기 런타임**: Tokio
- **데이터베이스**: Redis (큐, 메시지)
- **직렬화**: serde, serde_json
- **메트릭**: Prometheus
- **로깅**: tracing, tracing-subscriber

### Frontend (Client)
- **엔진**: Unity
- **언어**: C#
- **WebSocket**: (Unity WebSocket 라이브러리)

### Infrastructure
- **컨테이너**: Kubernetes (Pod 단위 배포)
- **메시지 브로커**: Redis Pub/Sub
- **모니터링**: Prometheus + Grafana
- **인증**: Auth Server (별도 서비스)

---

## Redis 데이터 구조

### 큐 관리
```
queue:{mode}              (Sorted Set, score=timestamp)
├── normal                → 일반 큐
├── ranked                → 랭크 큐
└── party                 → 파티 큐

metadata:{player_id}      (String, JSON)
└── {"pod_id": "...", "deck": {...}, "level": 10, ...}
```

### Pub/Sub 채널
```
[Match Server → Game Server]
├── match_result                     → 매칭 결과 (Deprecated, 사용 안 됨)
└── pod:{pod_id}:match_result       → Pod별 매칭 결과 (사용 안 됨)

[Game Server ↔ Game Server]
├── pod:{pod_id}:game_message       → Cross-pod 메시지 라우팅 ✅
└── events:test:{session_id}        → 테스트 이벤트 스트리밍
```

---

## 설정 파일

### development.toml
```toml
[server]
bind_address = "0.0.0.0"
port = 8080
log_level = "info"

[matchmaking]
try_match_tick_interval_seconds = 5
heartbeat_interval_seconds = 30
heartbeat_timeout = 120
redis_operation_timeout_seconds = 10
skip_game_server_check = true  # 개발 전용

[[matchmaking.game_modes]]
game_mode = "Normal"
required_players = 2
use_mmr_matching = false

[[matchmaking.game_modes]]
game_mode = "Ranked"
required_players = 2
use_mmr_matching = true
```

---

## 메트릭 모니터링

### Prometheus 엔드포인트
```
GET /metrics
Authorization: Bearer {token}  (optional)
```

### 주요 메트릭
```
# Matchmaking
matches_created_total
matches_same_pod_total
matches_cross_pod_total
matched_players_total_by_mode{game_mode}

# Routing
messages_routed_same_pod_total
messages_routed_cross_pod_total

# Redis
poisoned_candidates_total
game_server_available
game_server_unavailable_total

# Performance
try_match_skipped_total
```

---

## 보안 고려사항

### 완료된 보안 강화
1. ✅ **Same-pod/Cross-pod 구분** - 불필요한 Redis 홉 제거
2. ✅ **Circuit Breaker** - Redis 장애 격리
3. ✅ **Rate Limiting** - 구조 준비 (현재 비활성화)

### 미완료 보안 강화
1. ❌ **서버에서 metadata 생성** - 현재 클라이언트가 전송 (레거시)
2. ❌ **Auth Token 검증** - Auth Server 연동 필요
3. ❌ **플레이어 상태 검증** - PlayerGameActor 구현 필요
4. ❌ **Rate Limiting 활성화** - 필요 시 활성화

---

## 알려진 이슈

### 1. PlayerGameActor stub
- **상태**: 빈 구조체만 존재
- **영향**: Unity Client 연결 불가
- **우선순위**: 높음

### 2. Battle 로직 stub
- **상태**: player1 항상 승리
- **영향**: 실제 게임 진행 불가
- **우선순위**: 중

### 3. /game 엔드포인트 없음
- **상태**: 라우트 미등록
- **영향**: Unity Client 연결 불가
- **우선순위**: 높음

### 4. 레거시 이중 메시지 전송
- **상태**: Same-pod도 레거시 경로 실행
- **영향**: 약간의 오버헤드
- **우선순위**: 낮음 (Unity 전환 후 제거 예정)

---

## 성능 벤치마크

### 매칭 지연 시간

| 시나리오 | Before | After | 개선율 |
|---------|--------|-------|--------|
| Same-pod 매칭 | 0.1ms | 0.1ms | - |
| Cross-pod 매칭 | 15-20ms | **5-10ms** | **50%** |

### 코드 복잡도

| 항목 | Before | After | 개선율 |
|------|--------|-------|--------|
| TryMatch 핸들러 | 353 lines | **80 lines** | **78%** |
| Battle 처리 | 300 lines | **150 lines** | **50%** |

---

## 참고 문서

### 기존 문서 (통합됨)
- ~~ARCHITECTURE_CURRENT.md~~ → 이 문서로 통합
- ~~ARCHITECTURE.md~~ → 이 문서로 통합
- ~~MIGRATION_PLAN.md~~ → 이 문서로 통합
- ~~TRYMATCH_REFACTORING_PLAN.md~~ → 완료 (이 문서에 기록)
- ~~BATTLE_ACTOR_REFACTORING_PLAN.md~~ → 완료 (이 문서에 기록)

### 유지할 문서
- `AGENTS.md` - 에이전트 관련
- `GIT_COMMIT_CONVENTION.md` - 커밋 컨벤션

---

## 변경 이력

| 날짜 | 버전 | 변경 사항 |
|------|------|-----------|
| 2025-10-23 | 1.0 | 초안 작성 (마이그레이션 문서 통합) |
| 2025-10-26 | 1.1 | TODO 추가: Ghost 스냅샷 기반 매칭 시스템 (Dequeue 시점 변경) |

---

**작성자**: Development Team
**최종 수정**: 2025-10-26
