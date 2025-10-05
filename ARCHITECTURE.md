# 온라인 1vs1 카드 게임 아키텍처 설계

## 개요

- **게임 서버**: Rust (Actix Actor 기반)
- **클라이언트**: Unity
- **게임 연산**: 보안을 위해 Game Server에서 모두 처리
- **클라이언트 역할**: 연산 결과 시각화만 담당

---

## 서비스 구조

### 독립적 서비스 (단일 인스턴스)

- **Redis Server**: 메시지 발행/관리, 큐 관리
- **Auth Server**: 플레이어 인증 및 고유 키 발급

### 비독립적 서비스 (Pod 단위, 복수 존재)

- **Game Server**: 대규모 플레이어 관리 (별도 프로세스)
- **Match Server**: 매치메이킹 처리 (별도 프로세스, Game Server와 1:1 쌍)

**프로세스 간 통신:** Redis Pub/Sub 사용 (Actix Actor 메시지는 같은 프로세스만 가능)

---

## 시스템 아키텍처 도식도

```
                    ┌─────────────────────────────────────────┐
                    │         Redis Cluster                   │
                    │                                         │
                    │  [Data Storage]                         │
                    │  ├─ queue:{mode} (Sorted Set)          │
                    │  └─ metadata:{player_id} (Hash)        │
                    │                                         │
                    │  [Pub/Sub Channels]                    │
                    │  ├─ battle:request                     │
                    │  ├─ pod:pod-a:battle_result            │
                    │  ├─ pod:pod-b:battle_result            │
                    │  └─ pod:pod-c:battle_result            │
                    └──────────┬──────────────┬───────────────┘
                               │              │
                ┌──────────────┴──┐    ┌──────┴──────────────┐
                │   Subscribe     │    │    Subscribe        │
                │   Publish       │    │    Publish          │
                ▼                 ▼    ▼                     ▼
┌───────────────────────────────────────────────────────────────┐
│ Pod A                                                         │
│ ┌─────────────────────┐     ┌───────────────────────────┐   │
│ │  Match Server       │     │   Game Server             │   │
│ │  (프로세스 1)        │     │   (프로세스 2)             │   │
│ │                     │     │                           │   │
│ │  ┌──────────────┐   │     │  ┌──────────────────┐    │   │
│ │  │NormalMaker   │   │     │  │LoadBalanceActor  │    │   │
│ │  │              │───┼─────┼─▶│                  │    │   │
│ │  │TryMatch:     │   │     │  │ HashMap<         │    │   │
│ │  │ - pop queue  │   │     │  │  player_id,      │    │   │
│ │  │ - match 2~4  │   │     │  │  PlayerGameActor>│    │   │
│ │  │ - publish    │   │     │  └──────────────────┘    │   │
│ │  │   battle:req │   │     │                           │   │
│ │  └──────────────┘   │     │  ┌──────────────────┐    │   │
│ │                     │     │  │ BattleActor      │    │   │
│ │  ┌──────────────┐   │     │  │  - calculate     │    │   │
│ │  │Enqueue       │   │     │  │  - publish       │    │   │
│ │  │Dequeue       │   │     │  │    pod:*:result  │    │   │
│ │  └──────────────┘   │     │  └──────────────────┘    │   │
│ │                     │     │                           │   │
│ │  WebSocket Sessions │     │  ┌──────────────────┐    │   │
│ │  - Enqueue/Dequeue  │     │  │PlayerGameActor   │◀───┼───┤
│ │    requests         │     │  │ - WebSocket      │    │   │
│ └─────────────────────┘     │  │ - Game logic     │    │   │
│                             │  │ - Battle result  │    │   │
│                             │  └──────────────────┘    │   │
│                             │          ▲               │   │
│                             └──────────┼───────────────┘   │
│                                        │                   │
│                              Redis Pub/Sub:                │
│                              pod:pod-a:battle_result       │
└────────────────────────────────────────┼───────────────────┘
                                         │
                                    WebSocket
                                         │
                                         ▼
                              ┌──────────────────┐
                              │  Player 1        │
                              │  (Unity Client)  │
                              └──────────────────┘


┌───────────────────────────────────────────────────────────────┐
│ Pod B                                                         │
│ ┌─────────────────────┐     ┌───────────────────────────┐   │
│ │  Match Server       │     │   Game Server             │   │
│ │  (프로세스 1)        │     │   (프로세스 2)             │   │
│ │                     │     │                           │   │
│ │  ┌──────────────┐   │     │  ┌──────────────────┐    │   │
│ │  │NormalMaker   │   │     │  │LoadBalanceActor  │    │   │
│ │  │RankedMaker   │   │     │  │                  │    │   │
│ │  │PartyMaker    │   │     │  └──────────────────┘    │   │
│ │  └──────────────┘   │     │                           │   │
│ │                     │     │  ┌──────────────────┐    │   │
│ │  WebSocket Sessions │     │  │PlayerGameActor   │◀───┼───┤
│ └─────────────────────┘     │  │ - WebSocket      │    │   │
│                             │  │ - Game logic     │    │   │
│                             │  └──────────────────┘    │   │
│                             │          ▲               │   │
│                             └──────────┼───────────────┘   │
│                                        │                   │
│                              Redis Pub/Sub:                │
│                              pod:pod-b:battle_result       │
└────────────────────────────────────────┼───────────────────┘
                                         │
                                    WebSocket
                                         │
                                         ▼
                              ┌──────────────────┐
                              │  Player 2        │
                              │  (Unity Client)  │
                              └──────────────────┘
```

### 통신 흐름

**1. 매칭 요청 (Enqueue)**
```
Player → Match Server (WebSocket)
       → Redis (ZADD queue:{mode}, HSET metadata:{player_id})
```

**2. 매칭 성사 (TryMatch)**
```
Match Server → Redis (ZPOPMIN queue:{mode})
             → Redis (Publish battle:request)
             → All Game Servers receive
```

**3. 전투 처리**
```
Game Server (player1.pod_id 일치)
         → BattleActor 생성
         → 전투 계산
         → Redis (Publish pod:{pod_id}:battle_result) × 2
```

**4. 결과 전달**
```
Game Server (자기 Pod 채널 구독)
         → LoadBalanceActor
         → PlayerGameActor
         → Player (WebSocket)
```

---

## Redis 데이터 구조

### 데이터 저장

```
Redis Cluster
├── queue:{mode}              (Sorted Set, score=enqueue_timestamp)
│   ├── normal               → 일반 큐
│   ├── ranked               → 랭크 큐
│   └── party                → 파티 큐
│
└── metadata:{player_id}     (String, JSON)
    → {"deck_build": {...}, "artifacts": {...}, "items": [...], "pod_id": "pod-a"}
    → BattleActor에 필요한 전투 스냅샷 (JSON 문자열로 저장)
```

### Pub/Sub 채널

```
Redis Pub/Sub Channels
├── battle:request              → 전투 요청 (모든 Game Server 구독)
├── pod:{pod_id}:battle_result  → Pod별 전투 결과 (해당 Pod만 구독)
│   ├── pod:pod-a:battle_result
│   ├── pod:pod-b:battle_result
│   └── pod:pod-c:battle_result
└── (미래 확장용 채널들)
```

**핵심 원칙:**

- `queue:{mode}`는 FIFO 보장 (score 기반 정렬)
- `metadata`는 정렬 불필요 (player_id로 O(1) 접근)
- **연결 상태는 SubScriptionManager가 단일 진실 원천으로 관리**
- **WebSocket 종료 시 queue + metadata 자동 삭제**

---

## Game Server 상세

### 시작 시 초기화

```rust
async fn start_game_server() {
    let our_pod_id = env::var("POD_ID").unwrap();
    let redis = ConnectionManager::new(...).await;

    // 1. battle:request 구독 (전투 생성용)
    spawn(subscribe_battle_requests(redis.clone(), our_pod_id));

    // 2. pod:{our_pod_id}:battle_result 구독 (결과 수신용)
    spawn(subscribe_battle_results(redis.clone(), our_pod_id));

    // 3. LoadBalanceActor, 기타 Actor 시작
    let load_balancer = LoadBalanceActor::start();

    // 4. WebSocket 서버 시작
    HttpServer::new(...).bind(...).run().await;
}
```

### PlayerGameActor

- 플레이어별 게임 진행 담당
- 플레이어와 직접 WebSocket 통신
- 상태 격리로 동시성 문제 자연스럽게 해결
- 전투 결과 수신 시 클라이언트에 전달

### LoadBalanceActor

- PlayerGameActor 추적/관리
- 내부적으로 `HashMap<player_id, Addr<PlayerGameActor>>` 보유
- 재접속 시 기존 Actor 찾기 지원
- **전투 결과 라우팅에 사용** (player_id → PlayerGameActor)

### BattleActor

- 두 플레이어의 전투 결과 계산
- metadata의 덱/아티팩트/아이템 기반 시뮬레이션
- **전투 완료 시:**
  - 각 플레이어의 `pod_id`로 Redis Pub/Sub 발행
  - `redis.publish("pod:{pod_id}:battle_result", result)`
  - 같은 Pod / 다른 Pod 구분 없이 동일한 방식

---

## Match Server 상세

### 구조

```rust
MatchServer
├── NormalMatchmaker    (일반 매칭)
├── RankedMatchmaker    (랭크 매칭, MMR 기반)
└── PartyMatchmaker     (파티 매칭)
```

### 핵심 메시지

#### Enqueue

```rust
pub struct Enqueue {
    pub player_id: Uuid,
    pub game_mode: GameMode,
    pub metadata: String,  // JSON: deck, artifacts, items, etc.
}
```

**동작:**

1. Auth Server로 player_id 검증
2. Lua 스크립트로 원자적 처리:
   - `ZADD queue:{mode} {timestamp} {player_id}`
   - `SET metadata:{player_id} "{\"pod_id\": \"pod-a\", \"deck\": {...}, \"artifacts\": {...}}"` (JSON 문자열)
3. 플레이어에게 `EnQueued` 응답

#### Dequeue

```rust
pub struct Dequeue {
    pub player_id: Uuid,
    pub game_mode: GameMode,
}
```

**동작:**

1. Lua 스크립트로 원자적 제거:
   - `ZREM queue:{mode} {player_id}`
   - `DEL metadata:{player_id}`
2. 플레이어에게 `DeQueued` 응답

#### TryMatch (주기적 실행)

```rust
pub struct TryMatch {
    pub match_mode_settings: MatchModeSettings,
}
```

**동작:**

1. `ZPOPMIN queue:{mode} {batch_size}` (FIFO 보장, metadata 포함)
2. 2~4명씩 매칭 (Redis에 있으면 = 연결되어 있음 보장)
3. 매칭 결과:
   - **성공**: Game Server로 전달 (BattleActor 생성)
   - **실패**: 남은 플레이어 재enqueue
4. 결과를 PlayerGameActor에 전달

**보장 메커니즘:**
- WebSocket 종료 시 `stopping()`에서 queue + metadata 자동 삭제
- TryMatch가 pop한 플레이어 = 100% 연결되어 있음
- 별도 연결 확인 불필요

### 실제 구현 상세

#### Session State Machine (session/mod.rs, session/helper.rs)

```
┌──────┐  Enqueue   ┌───────────┐  EnQueued   ┌─────────┐
│ Idle │ ────────> │ Enqueuing │ ─────────> │ InQueue │
└──────┘           └───────────┘            └─────────┘
                                                   │
                                    ┌──────────────┼──────────────┐
                                    │              │              │
                                Dequeue       MatchFound       Error
                                    │              │              │
                                    ▼              ▼              ▼
                              ┌──────────┐  ┌───────────┐  ┌───────┐
                              │ Dequeued │  │ Completed │  │ Error │
                              └──────────┘  └───────────┘  └───────┘
```

**상태 전환 규칙 (session/helper.rs:15-60):**
- **Idle**: 최초 연결, Enqueue 가능
- **Enqueuing**: Enqueue 처리 중, SubScriptionManager 등록 중
- **InQueue**: 큐에 등록됨, Dequeue 가능
- **Dequeued**: 큐에서 제거됨
- **Completed**: 매칭 성공 (현재는 Game Server에서 처리)
- **Error**: 오류 발생, 연결 종료

**위반 처리 (session/helper.rs:classify_violation):**
- **Minor**: 무시 (예: InQueue → InQueue 중복 요청)
- **Major**: Error 상태 전환 + 에러 메시지
- **Critical**: 즉시 WebSocket 종료

#### Rate Limiter (lib.rs:181-236)

- **Token Bucket 알고리즘**: 10 tokens/sec per IP
- **자동 정리**: 10분 비활성 IP는 제거
- **적용 위치**: Session::handle_enqueue (session/mod.rs:201)

#### Graceful Shutdown

**main.rs:170-183 (Ctrl+C 핸들러):**
1. `shutdown_token.cancel()` → 모든 Actor에 종료 신호
2. `System::current().stop()` → Actix 시스템 종료

**Matchmaker Actor (normal/mod.rs:71-91, rank/mod.rs 유사):**
1. `stopping()` 호출 → `shutdown_token.cancel()`
2. 실행 중인 TryMatch Future 체크 (ctx.waiting())
3. 25초 타임아웃 후 강제 종료

**Session Actor (session/mod.rs:129-189):**
1. `stopping()` 호출
2. Cleanup watchdog 시작 (10초 타임아웃)
3. Matchmaker에 Dequeue 전송 (if matchmaker_addr exists)
4. SubScriptionManager에 Deregister 전송
5. 정리 완료 후 자신에게 Stop 메시지

**TryMatch Future (normal/handlers.rs:104-164):**
- `shutdown_token.is_cancelled()` 주기적 체크
- 종료 시 pop한 candidates 재enqueue
- Backoff 중에도 `tokio::select!`로 즉시 종료 가능

---

## 플레이어 연결 흐름

### 1. 초기 접속

```
Player (with Auth key)
  → Game Server (검증)
  → LoadBalanceActor 조회
  ├─ 신규: PlayerGameActor 생성 + 등록
  └─ 재접속: 기존 PlayerGameActor 찾기
  → WebSocket 수립
```

### 2. PvP 진입

```
PlayerGameActor
  → Enqueue 메시지 발행
  → Match Server (같은 Pod)
  → Redis queue + metadata 저장
  → 플레이어에게 EnQueued 응답
```

### 3. 매칭 성사 및 전투

**Step 1: TryMatch (Match Server)**
```
TryMatch (주기 실행)
  → Redis queue pop (FIFO, metadata 포함)
  → 2~4명 매칭 (pop한 플레이어 = 연결됨 보장)
  → redis.publish("battle:request", {
      player1: {id, pod_id, deck, ...},
      player2: {id, pod_id, deck, ...}
    })
  → 남은 플레이어 재enqueue
```

**Step 2: BattleActor 생성 (Game Server)**
```
Game Server (battle:request 구독 중)
  → 메시지 수신
  → player1.pod_id == 우리 Pod?
    ├─ Yes → BattleActor 생성
    └─ No  → 무시 (다른 Pod가 처리)
  → BattleActor가 전투 계산
```

**Step 3: 결과 전달 (BattleActor → PlayerGameActor)**
```
BattleActor (전투 완료)
  → Player1 결과:
      redis.publish("pod:{player1.pod_id}:battle_result", {
        player_id: player1.id,
        battle_data: {...}
      })
  → Player2 결과:
      redis.publish("pod:{player2.pod_id}:battle_result", {
        player_id: player2.id,
        battle_data: {...}
      })

각 Pod의 Game Server (자기 채널 구독 중)
  → "pod:{our_pod}:battle_result" 수신
  → LoadBalanceActor로 player_id 찾기
  → PlayerGameActor에 전달
  → WebSocket으로 클라이언트에 전송
```

### 4. WebSocket 종료 (정상/비정상)

```
WsSession::finished() 트리거 (Actix 자동 호출)
  → queue에서 자동 Dequeue
  → metadata 삭제
  → SubScriptionManager에서 자동 제거

→ 이후 TryMatch는 해당 플레이어를 절대 pop하지 않음 (보장)
```

---

## Lua 스크립트 원자성 보장

### ENQUEUE_PLAYER.lua

```lua
-- KEYS[1] = queue:{mode} (Sorted Set)
-- ARGV[1] = player_id
-- ARGV[2] = timestamp (score)
-- ARGV[3] = metadata JSON string

local queue_key = KEYS[1]
local player_id = ARGV[1]
local timestamp = tonumber(ARGV[2])
local metadata_json = ARGV[3]

-- 유효성 검사
if timestamp == nil or metadata_json == nil or metadata_json == "" then
    local size = redis.call('ZCARD', queue_key)
    return {0, size}
end

-- 이미 큐에 있는지 확인
local exists = redis.call('ZSCORE', queue_key, player_id)
if exists then
    local size = redis.call('ZCARD', queue_key)
    return {0, size}
end

-- queue에 추가 (Sorted Set)
redis.call('ZADD', queue_key, timestamp, player_id)

-- metadata 저장 (JSON 문자열 그대로 저장)
local metadata_key = 'metadata:' .. player_id
redis.call('SET', metadata_key, metadata_json)

-- 현재 큐 크기 반환
local size = redis.call('ZCARD', queue_key)
return {1, size}
```

### DEQUEUE_PLAYER.lua

```lua
-- KEYS[1] = queue:{mode} (Sorted Set)
-- ARGV[1] = player_id

local queue_key = KEYS[1]
local player_id = ARGV[1]

-- queue에서 제거
local removed = redis.call('ZREM', queue_key, player_id)

-- metadata 삭제
if removed == 1 then
    local metadata_key = 'metadata:' .. player_id
    redis.call('DEL', metadata_key)
end

-- 현재 큐 크기 반환
local size = redis.call('ZCARD', queue_key)
return {removed, size}
```

### TRY_MATCH_POP.lua

```lua
-- KEYS[1] = queue:{mode} (Sorted Set)
-- ARGV[1] = batch_size (integer)

local queue_key = KEYS[1]
local batch_size = tonumber(ARGV[1])

-- 유효성 검사
if batch_size == nil or batch_size <= 0 then
    return {}
end

-- ZPOPMIN으로 원자적으로 pop (FIFO 보장)
local popped = redis.call('ZPOPMIN', queue_key, batch_size)

if #popped == 0 then
    return {}
end

local result = {}

-- popped format: [player_id, score, player_id, score, ...]
for idx = 1, #popped, 2 do
    local player_id = popped[idx]
    local score = popped[idx + 1]

    -- metadata 가져오기 (JSON 문자열 그대로)
    local metadata_key = 'metadata:' .. player_id
    local metadata_json = redis.call('GET', metadata_key)

    -- metadata가 없으면 빈 객체
    if not metadata_json then
        metadata_json = "{}"
    end

    -- 결과에 추가: [player_id, score, metadata_json, ...]
    table.insert(result, player_id)
    table.insert(result, score)
    table.insert(result, metadata_json)

    -- metadata 삭제 (이미 pop했으므로)
    redis.call('DEL', metadata_key)
end

return result
```

---

## 동시성 처리

### 여러 Pod의 TryMatch가 동시에 큐 접근

```
Pod A TryMatch: ZPOPMIN queue:normal 10
Pod B TryMatch: ZPOPMIN queue:normal 10 (동시)
→ Redis Sorted Set의 ZPOPMIN은 원자적이므로 중복 없음 ✅
```

---

## 전투 결과 데이터 구조

### Event Timeline 방식

**서버에서 전투를 완전히 시뮬레이션하고 이벤트 타임라인 생성**

#### BattleResult 구조

```rust
#[derive(Serialize, Deserialize)]
pub struct BattleResult {
    pub winner: PlayerId,
    pub duration: f32,              // 전투 총 시간 (초)
    pub timeline: Vec<BattleEvent>, // 시간 순서대로 정렬된 이벤트
    pub rewards: Rewards,
}

#[derive(Serialize, Deserialize)]
pub struct BattleEvent {
    pub timestamp: f32,           // 전투 시작 후 경과 시간 (초)
    pub actor_id: String,         // 행동하는 환상체 ID
    pub action: ActionType,       // Attack, Skill, Death, Spawn
    pub target_id: String,        // 대상 환상체 ID

    // 결과 데이터
    pub damage: Option<u32>,
    pub heal: Option<u32>,
    pub effects: Vec<Effect>,     // 버프/디버프

    // 상태 변화 (UI 동기화용)
    pub actor_hp: u32,
    pub target_hp: u32,
}

#[derive(Serialize, Deserialize)]
pub enum ActionType {
    Attack,
    Skill { skill_id: String },
    Death,
    Spawn,
}

#[derive(Serialize, Deserialize)]
pub struct Effect {
    pub effect_type: String,  // "Burn", "Freeze", "Buff", etc.
    pub duration: f32,
    pub value: i32,
}
```

#### JSON 예시

```json
{
  "winner": "player1",
  "duration": 45.5,
  "timeline": [
    {
      "timestamp": 0.0,
      "actor_id": "p1_entity_1",
      "action": "Attack",
      "target_id": "p2_entity_1",
      "damage": 50,
      "heal": null,
      "effects": [],
      "actor_hp": 300,
      "target_hp": 250
    },
    {
      "timestamp": 1.5,
      "actor_id": "p2_entity_2",
      "action": {"Skill": {"skill_id": "fireball"}},
      "target_id": "p1_entity_1",
      "damage": 80,
      "heal": null,
      "effects": [
        {"effect_type": "Burn", "duration": 3.0, "value": 10}
      ],
      "actor_hp": 200,
      "target_hp": 220
    },
    {
      "timestamp": 3.0,
      "actor_id": "p1_entity_1",
      "action": "Death",
      "target_id": "p1_entity_1",
      "damage": null,
      "heal": null,
      "effects": [],
      "actor_hp": 0,
      "target_hp": 0
    }
  ],
  "rewards": {
    "gold": 100,
    "exp": 50
  }
}
```

#### BattleActor 시뮬레이션 로직

```rust
impl BattleActor {
    fn simulate_battle(&self) -> BattleResult {
        let mut timeline = Vec::new();
        let mut time = 0.0;
        let dt = 0.1; // 100ms 단위 시뮬레이션

        let mut entities = self.initialize_entities_from_metadata();

        while !self.is_battle_over(&entities) && time < 120.0 {
            // 각 엔티티의 쿨다운 체크
            for entity in &mut entities {
                entity.cooldown -= dt;

                if entity.cooldown <= 0.0 {
                    // 행동 실행
                    let target = self.select_target(&entity, &entities);
                    let action = entity.next_action();

                    let result = self.execute_action(entity, target, action);

                    // 이벤트 기록
                    timeline.push(BattleEvent {
                        timestamp: time,
                        actor_id: entity.id.clone(),
                        action,
                        target_id: target.id.clone(),
                        damage: result.damage,
                        heal: result.heal,
                        effects: result.effects,
                        actor_hp: entity.hp,
                        target_hp: target.hp,
                    });

                    // 쿨다운 리셋
                    entity.cooldown = entity.action_cooldown;

                    // 사망 처리
                    if target.hp == 0 {
                        timeline.push(BattleEvent {
                            timestamp: time,
                            actor_id: target.id.clone(),
                            action: ActionType::Death,
                            target_id: target.id.clone(),
                            damage: None,
                            heal: None,
                            effects: vec![],
                            actor_hp: 0,
                            target_hp: 0,
                        });
                    }
                }
            }

            time += dt;
        }

        BattleResult {
            winner: self.determine_winner(&entities),
            duration: time,
            timeline,
            rewards: self.calculate_rewards(&entities),
        }
    }
}
```

#### Unity 재생 로직

```csharp
public class BattleReplayController : MonoBehaviour
{
    private BattleResult result;
    private float elapsedTime = 0f;
    private int currentEventIndex = 0;

    void Update()
    {
        if (isReplaying)
        {
            elapsedTime += Time.deltaTime;

            // 다음 이벤트 처리
            while (currentEventIndex < result.timeline.Count)
            {
                var evt = result.timeline[currentEventIndex];

                if (evt.timestamp <= elapsedTime)
                {
                    PlayEvent(evt);
                    currentEventIndex++;
                }
                else
                {
                    break; // 아직 시간 안 됨
                }
            }

            if (currentEventIndex >= result.timeline.Count)
            {
                OnBattleReplayFinished();
            }
        }
    }

    void PlayEvent(BattleEvent evt)
    {
        var actor = GetEntity(evt.actor_id);
        var target = GetEntity(evt.target_id);

        switch (evt.action)
        {
            case "Attack":
                actor.PlayAttackAnimation();
                if (evt.damage.HasValue)
                {
                    target.TakeDamage(evt.damage.Value);
                }
                break;

            case "Skill":
                actor.PlaySkillAnimation(evt.action.skill_id);
                if (evt.damage.HasValue)
                {
                    target.TakeDamage(evt.damage.Value);
                }
                ApplyEffects(target, evt.effects);
                break;

            case "Death":
                actor.PlayDeathAnimation();
                break;
        }

        // HP UI 업데이트
        actor.UpdateHPBar(evt.actor_hp);
        target.UpdateHPBar(evt.target_hp);
    }
}
```

### Timeline 방식의 장점

1. **정확성**: 서버 연산 그대로 재생 (클라이언트-서버 불일치 없음)
2. **유연성**: 배속 재생, 일시정지, 되감기 가능
3. **디버깅**: 전투 로그를 그대로 볼 수 있음
4. **확장성**: 새로운 액션 타입 추가 쉬움
5. **보안**: 클라이언트에서 연산 불가 (치팅 방지)

### 데이터 크기 예측

```
전투 시간: 60초
엔티티 수: 6개 (각 플레이어 3개)
평균 쿨다운: 2초
총 이벤트: (60 ÷ 2) × 6 = 180개 이벤트

원본 JSON 크기: ~45KB
```

---

## 전투 데이터 압축

### 압축 방법 비교

| 방법 | 크기 | 압축률 | 구현 난이도 | 디버깅 | 추천 |
|------|------|--------|------------|--------|------|
| 원본 JSON | 50KB | - | 쉬움 | 쉬움 | - |
| **gzip** | **10KB** | **80%** | **쉬움** | **쉬움** | ✅ |
| MessagePack | 6KB | 88% | 중간 | 어려움 | - |
| JSON 최적화 | 20KB | 60% | 중간 | 중간 | - |
| gzip + 최적화 | 5KB | 90% | 중간 | 중간 | - |

### 방법 1: gzip 압축 (추천) ✅

**서버 (Rust)**

```rust
use flate2::Compression;
use flate2::write::GzEncoder;
use flate2::read::GzDecoder;
use std::io::{Write, Read};

// 압축 헬퍼 함수
pub fn compress_json<T: Serialize>(data: &T) -> Result<String, String> {
    let json = serde_json::to_string(data)
        .map_err(|e| format!("Serialize error: {}", e))?;

    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(json.as_bytes())
        .map_err(|e| format!("Compression error: {}", e))?;

    let compressed = encoder.finish()
        .map_err(|e| format!("Finish error: {}", e))?;

    // base64 인코딩 (JSON에 바이너리 넣기 위해)
    Ok(base64::encode(&compressed))
}

// BattleActor에서 사용
impl BattleActor {
    async fn send_battle_result(
        &self,
        redis: &mut ConnectionManager,
        pod_id: &str,
        player_id: Uuid,
        result: BattleResult
    ) -> Result<(), String> {
        // gzip 압축
        let compressed_base64 = compress_json(&result)?;

        // Redis로 발행
        redis.publish(
            format!("pod:{}:battle_result", pod_id),
            serde_json::to_string(&BattleResultMessage {
                player_id: player_id.to_string(),
                battle_data_compressed: compressed_base64,
            }).unwrap()
        ).await
        .map_err(|e| format!("Redis publish error: {}", e))?;

        Ok(())
    }
}
```

**Unity (C#)**

```csharp
using System;
using System.IO;
using System.IO.Compression;
using System.Text;
using UnityEngine;

public class BattleResultHandler
{
    public BattleResult DecompressBattleData(string base64Compressed)
    {
        try
        {
            // base64 디코딩
            byte[] compressed = Convert.FromBase64String(base64Compressed);

            // gzip 압축 해제
            using (var input = new MemoryStream(compressed))
            using (var gzip = new GZipStream(input, CompressionMode.Decompress))
            using (var output = new MemoryStream())
            {
                gzip.CopyTo(output);
                string json = Encoding.UTF8.GetString(output.ToArray());

                // JSON 파싱
                return JsonUtility.FromJson<BattleResult>(json);
            }
        }
        catch (Exception e)
        {
            Debug.LogError($"Failed to decompress battle data: {e}");
            return null;
        }
    }
}
```

**압축 효과:** 50KB → 8~12KB (70~80% 감소)

### 방법 2: MessagePack (바이너리 포맷)

```toml
# Cargo.toml
[dependencies]
rmp-serde = "1.1"
```

```rust
use rmp_serde;

pub fn compress_messagepack<T: Serialize>(data: &T) -> Result<String, String> {
    let msgpack_bytes = rmp_serde::to_vec(data)
        .map_err(|e| format!("MessagePack error: {}", e))?;

    Ok(base64::encode(&msgpack_bytes))
}
```

```csharp
// Unity - MessagePack for C#
using MessagePack;

public BattleResult DecompressMessagePack(string base64)
{
    byte[] bytes = Convert.FromBase64String(base64);
    return MessagePackSerializer.Deserialize<BattleResult>(bytes);
}
```

**압축 효과:** 50KB → 6~8KB (85% 감소)

**장점:**
- JSON보다 작고 빠름
- 타입 안전성

**단점:**
- 사람이 읽을 수 없음 (디버깅 어려움)
- Unity에 MessagePack 라이브러리 필요

### 방법 3: JSON 구조 최적화

```json
// Before (장황)
{
  "timeline": [
    {"timestamp": 0.0, "actor_id": "p1_e1", "action": "Attack", ...}
  ]
}

// After (배열로 최적화)
{
  "t": [
    [0.0, "p1_e1", 0, "p2_e1", 50, 300, 250],
    [1.5, "p2_e2", 0, "p1_e1", 45, 200, 255]
  ],
  "legend": ["timestamp", "actor_id", "action", "target_id", "damage", "actor_hp", "target_hp"]
}
```

**압축 효과:** 50KB → 20KB (60% 감소)

---

## 게임 플레이 흐름

### Day 구조

```
Day N 시작
  → 이벤트 선택 (랜덤 3개 중 1개)
  → 이벤트 선택
  → PvE 전투
  → 이벤트 선택
  → PvP 매칭 + 자동 전투
Day N+1 시작
  ...
```

### 이벤트 종류

- 상점 입장
- 골드 획득
- 환상체 획득
- 퀘스트 획득
- 기타 등등

### 레벨업 시스템

- 특정 행동/시간 경과로 경험치 획득
- 레벨업 시 전략적 선택 가능

---

## 크로스 Pod 매칭 처리

### 시나리오: Pod A의 Player1 + Pod B의 Player2

**1. 매칭 성사 (Pod A Match Server)**
```rust
// TryMatch handler
let candidates = pop_candidates(...).await?;
// [Player1@pod-a, Player2@pod-b]

redis.publish("battle:request", BattleRequest {
    player1: { id: p1, pod_id: "pod-a", deck: {...} },
    player2: { id: p2, pod_id: "pod-b", deck: {...} }
}).await;
```

**2. 전투 처리 결정 (모든 Game Server)**
```rust
// battle:request 구독 중
pubsub.subscribe("battle:request").await;

while let msg = pubsub.on_message().next().await {
    let request: BattleRequest = parse(msg);

    // player1의 Pod가 전투 처리
    if request.player1.pod_id == our_pod_id {
        spawn_battle_actor(request); // Pod A만 실행
    }
}
```

**3. 전투 계산 (Pod A BattleActor)**
```rust
impl BattleActor {
    async fn finish_battle(&self) {
        let result1 = calculate(&self.player1);
        let result2 = calculate(&self.player2);

        // metadata의 pod_id로 라우팅
        redis.publish("pod:pod-a:battle_result", {
            player_id: p1,
            battle_data: result1
        }).await;

        redis.publish("pod:pod-b:battle_result", {
            player_id: p2,
            battle_data: result2
        }).await;
    }
}
```

**4. 결과 수신 (각 Pod Game Server)**
```rust
// Pod A Game Server
pubsub.subscribe("pod:pod-a:battle_result").await;
// → Player1 결과 수신 → PlayerGameActor 전달

// Pod B Game Server
pubsub.subscribe("pod:pod-b:battle_result").await;
// → Player2 결과 수신 → PlayerGameActor 전달
```

**핵심:**
- metadata의 `pod_id`로 결과 라우팅
- 각 Pod는 정적으로 자기 채널만 구독
- 동적 구독 불필요 (효율적)

---

## 연결 상태 관리

### 단일 진실 원천 (Single Source of Truth)

**SubScriptionManager가 유일한 연결 상태 관리자:**
- WebSocket 연결/해제 자동 추적
- Redis에 중복 저장하지 않음
- 타임스탬프 비교 불필요

### 연결 수명 주기

**입장 시:**
```rust
// 1. SubScriptionManager에 WebSocket 등록 (진실 원천)
sub_manager.register(player_id, ws_addr);

// 2. Redis에 metadata만 저장 (연결 정보 X)
SET metadata:{player_id} "{\"pod_id\": \"pod-a\", \"deck\": {...}, \"artifacts\": {...}}"
```

**WebSocket 종료 시 (자동):**
```rust
impl StreamHandler for WsSession {
    fn finished(&mut self, ctx: &mut Self::Context) {
        // 1. SubScriptionManager에서 자동 제거
        sub_manager.unregister(player_id);

        // 2. queue에 있으면 제거
        if in_queue {
            dequeue(player_id);
        }

        // 3. metadata 삭제
        DEL metadata:{player_id}
    }
}
```

**TryMatch (단순화):**
```rust
// Redis에서 candidates pop
let candidates = pop_candidates(...).await?;

// candidates에 있으면 = 연결되어 있음 (finished()가 자동 삭제 보장)
// 별도 검증 불필요!

// 매칭
let matched_pairs = match_players(&candidates, required_players);

// 게임 서버로 전달
for pair in matched_pairs {
    send_to_game_server(pair).await;
}

// 남은 플레이어 재enqueue
for leftover in leftovers {
    re_enqueue(leftover).await;
}
```

### 효율성 비교

| 방식 | Redis 쓰기 | 정확성 | 복잡도 |
|------|-----------|--------|--------|
| Timestamp | 2초마다 (500 ops/sec @ 1000명) | 시간차 존재 | 높음 |
| **자동 정리 (현재)** | **입장/퇴장만 (~1 ops/sec)** | **실시간** | **매우 낮음** |

### 보장 메커니즘

**Redis에 있으면 = 연결되어 있음:**
- Actix의 `StreamHandler::finished()` 자동 호출
- WebSocket 종료 즉시 queue + metadata 삭제
- TryMatch는 연결된 플레이어만 pop (100% 보장)

**엣지 케이스 처리:**
- 매칭 후 전달 전 끊김 → 전달 실패 에러 핸들링
- Actor 크래시 → Actix drop 시 자동 정리
- 네트워크 장애 → TCP timeout으로 `finished()` 트리거

---

## 확장성 고려사항

### 현재 설계 (Bridge 제거)

- 각 Match Server가 Redis와 직접 통신
- 초기 단계에서 단순성 우선
- ZPOPMIN의 원자성으로 동시성 해결

### 미래 확장 (운영 후 도입)

- **Matchmaker Bridge**: 중앙 집중 매칭
- **리더 선출**: Redis lock + heartbeat
- **장애 복구**: Standby Bridge 자동 승격

---

## 메트릭 & 관찰성

### 수집 항목

- 큐 대기 시간 (enqueue → match)
- 매칭 성공률
- Pod별 부하 (active sessions)
- 연결 끊김 감지율 (TryMatch 시점)
- 크로스 Pod 매칭 비율

### 구현

- `metrics/src/lib.rs`에 Prometheus 메트릭
- Redis pub/sub로 이벤트 발행 (외부 모니터링)

---

## 구현 우선순위

### Phase 1 (완료) ✅

1. ✅ Enqueue/Dequeue operations (Lua Scripts 포함)
2. ✅ NormalMatchmaker TryMatch 구현
   - pop_candidates로 플레이어 가져오기
   - 2명 매칭 로직
   - redis.publish("battle:request") 발행
   - 남은 플레이어 재enqueue
3. ✅ RankedMatchmaker (MMR 기반)
4. ✅ WebSocket Session 관리
   - Session State Machine 구현
   - Heartbeat (30s ping, 120s timeout)
   - Graceful shutdown (Dequeue 자동 호출)
5. ✅ SubScriptionManager (플레이어 세션 추적)
6. ✅ Rate Limiter (10 req/sec per IP)
7. ✅ Prometheus Metrics (/metrics endpoint)
8. ✅ CancellationToken 기반 Graceful Shutdown

### Phase 2 (현재) ⚠️

1. ⚠️ Game Server 구현 (별도 프로젝트: `game_server/`)
   - battle:request 구독 → BattleActor 생성
   - BattleActor 전투 로직 (Event Timeline)
   - pod:{pod_id}:battle_result 구독 → PlayerGameActor 전달
   - LoadBalanceActor로 player_id → PlayerGameActor 찾기
2. ⚠️ 통합 테스트 (Match Server + Game Server)
3. ⚠️ 부하 테스트 (1000 동시 접속)

### Phase 3 (계획) ❌

1. ❌ PartyMatchmaker 구현
2. ❌ Battle Timeline gzip 압축
3. ❌ 고급 메트릭 및 알람 (Grafana, Alertmanager)
4. ❌ Bridge 패턴 (운영 데이터 기반 판단)
5. ❌ Redis Timeout 보호 (P0 안전장치 - SAFETY_IMPROVEMENTS.md 참고)
6. ❌ Max In-Flight Limit (P0 안전장치)

---

## 파일 구조 (현재 코드베이스)

```
match_server/
├── src/
│   ├── main.rs                    ✅ HTTP/WebSocket 서버 진입점
│   ├── lib.rs                     ✅ AppState, RateLimiter, 공통 로직
│   ├── env.rs                     ✅ 설정 로드 (TOML)
│   ├── metrics.rs                 ✅ Prometheus 메트릭
│   ├── protocol.rs                ✅ 메시지 프로토콜 정의
│   │
│   ├── session/
│   │   ├── mod.rs                 ✅ Session Actor (WebSocket)
│   │   ├── handlers.rs            ✅ 메시지 핸들러
│   │   └── helper.rs              ✅ State Machine 로직
│   │
│   ├── subscript/
│   │   ├── mod.rs                 ✅ SubScriptionManager Actor
│   │   ├── handlers.rs            ✅ Register/Deregister 핸들러
│   │   └── messages.rs            ✅ 메시지 정의
│   │
│   └── matchmaker/
│       ├── mod.rs                 ✅ Matchmaker 팩토리, MatchmakerAddr enum
│       ├── common.rs              ✅ MatchmakerInner (공통 데이터)
│       ├── messages.rs            ✅ Enqueue, Dequeue, TryMatch 메시지
│       ├── scripts.rs             ✅ Lua 스크립트 로더
│       │
│       ├── operations/
│       │   ├── mod.rs             ✅ 모듈 export
│       │   ├── enqueue.rs         ✅ Enqueue 로직 + re-enqueue
│       │   ├── dequeue.rs         ✅ Dequeue 로직
│       │   ├── notify.rs          ✅ 플레이어 알림 헬퍼
│       │   └── try_match.rs       ✅ pop_candidates, publish_battle_request
│       │
│       ├── normal/
│       │   ├── mod.rs             ✅ NormalMatchmaker Actor
│       │   └── handlers.rs        ✅ Enqueue, Dequeue, TryMatch 핸들러 (완료)
│       │
│       ├── rank/
│       │   ├── mod.rs             ✅ RankedMatchmaker Actor
│       │   └── handlers.rs        ✅ MMR 기반 매칭 (완료)
│       │
│       └── patry/
│           └── mod.rs             ❌ 미구현 (빈 모듈)
│
├── scripts/
│   ├── ENQUEUE_PLAYER.lua         ✅ 원자적 Enqueue + metadata 저장
│   ├── DEQUEUE_PLAYER.lua         ✅ 원자적 Dequeue + metadata 삭제
│   └── TRY_MATCH_POP.lua          ✅ ZPOPMIN + metadata 조회 + 삭제
│
└── config/
    ├── development.toml           ✅ 개발 환경 설정
    └── production.toml            ✅ 운영 환경 설정

game_server/                       ⚠️ 별도 프로젝트 (구현 중)
└── src/
    ├── main.rs                    ⚠️ Game Server 진입점
    ├── battle_actor/              ⚠️ 전투 로직 (TODO)
    ├── player_game_actor/         ⚠️ 플레이어 게임 Actor (TODO)
    └── load_balance_actor/        ⚠️ 플레이어 라우팅 (TODO)
```

### 구현 상태 요약

**✅ Match Server (완료)**
- WebSocket 세션 관리 (heartbeat, graceful shutdown)
- Session State Machine (Idle → Enqueuing → InQueue → Dequeued/Completed/Error)
- SubScriptionManager (플레이어 세션 추적)
- NormalMatchmaker, RankedMatchmaker (2명 매칭, 재enqueue)
- Redis Lua Scripts (원자성 보장)
- Rate Limiter (10 req/sec per IP)
- Prometheus Metrics (/metrics endpoint)
- Graceful Shutdown (CancellationToken)

**⚠️ Game Server (구현 중)**
- Redis Pub/Sub 구독 시스템 (TODO)
- BattleActor (전투 계산) (TODO)
- LoadBalanceActor (플레이어 라우팅) (TODO)
- PlayerGameActor (WebSocket 통신) (TODO)

**❌ 미구현**
- PartyMatchmaker (파티 매칭)
- Battle Timeline 생성 (BattleActor 내부)
- 크로스 Pod 전투 결과 라우팅

---

## 다음 단계

### Match Server ✅ (완료)

Match Server는 핵심 기능이 모두 구현되었습니다:
- ✅ WebSocket 세션 관리 및 State Machine
- ✅ Enqueue/Dequeue 원자적 처리 (Lua Scripts)
- ✅ NormalMatchmaker, RankedMatchmaker TryMatch 완료
- ✅ Graceful shutdown 및 재enqueue 로직
- ✅ Rate Limiting 및 Metrics

**남은 작업:**
- ❌ PartyMatchmaker 구현 (match_server/src/matchmaker/patry/)
- ⚠️ 운영 환경 모니터링 및 알람 설정

### Game Server ⚠️ (현재 작업 중)

Game Server는 별도 디렉토리(`game_server/`)에서 구현 중입니다:

1. **Redis Pub/Sub 시스템 구축**
   - `battle:request` 채널 구독 → BattleActor 생성
   - `pod:{pod_id}:battle_result` 채널 구독 → 결과 라우팅
   - 시작 시 정적 구독 설정 (동적 구독 불필요)

2. **BattleActor 구현** (game_server/src/battle_actor/)
   - metadata 기반 전투 시뮬레이션
   - Event Timeline 생성 (타임스탬프, 액션, 데미지, 효과)
   - gzip 압축 + base64 인코딩
   - 결과를 각 플레이어의 pod_id 채널로 발행

3. **LoadBalanceActor 구현** (game_server/src/load_balance_actor/)
   - `HashMap<player_id, Addr<PlayerGameActor>>` 관리
   - 전투 결과 수신 → player_id로 PlayerGameActor 찾기
   - 재접속 시 기존 Actor 조회

4. **PlayerGameActor 구현** (game_server/src/player_game_actor/)
   - 플레이어별 WebSocket 세션 관리
   - 전투 결과를 Unity 클라이언트로 전송
   - 게임 진행 상태 관리

5. **통합 테스트**
   - Match Server + Game Server 연동 검증
   - 크로스 Pod 매칭 시나리오 테스트
   - 부하 테스트 (동시 접속 1000명)

### Unity 클라이언트

6. **BattleResult 압축 해제** (Unity C#)
   - base64 디코딩 → gzip 압축 해제 → JSON 파싱
   - BattleReplayController 구현 (타임라인 재생)

7. **WebSocket 통신**
   - Match Server: 매칭 요청/응답
   - Game Server: 전투 결과 수신 및 재생
