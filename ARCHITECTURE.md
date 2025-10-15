# 온라인 1vs1 카드 게임 아키텍처 설계 (목표)

> 📌 **현재 구현 상태**: [ARCHITECTURE_CURRENT.md](./ARCHITECTURE_CURRENT.md) 참고
>
> 이 문서는 **목표 아키텍처**를 설명합니다.

---

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

- **Game Server**: 플레이어 게임 진행 관리 (별도 프로세스)
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
                    │  └─ metadata:{player_id} (String/JSON) │
                    │                                         │
                    │  [Pub/Sub Channels]                    │
                    │  ├─ match:enqueue:request              │
                    │  ├─ match:dequeue:request              │
                    │  ├─ pod:{pod_id}:match_result          │
                    │  ├─ battle:request                     │
                    │  └─ pod:{pod_id}:battle_result         │
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
│ │  │              │◀──┼─────┼──│                  │    │   │
│ │  │TryMatch:     │   │     │  │ HashMap<         │    │   │
│ │  │ - pop queue  │   │     │  │  player_id,      │    │   │
│ │  │ - match 2~4  │   │     │  │  PlayerGameActor>│    │   │
│ │  │ - publish    │   │     │  └──────────────────┘    │   │
│ │  │   battle:req │   │     │           ▲              │   │
│ │  └──────────────┘   │     │           │              │   │
│ │         ▲           │     │  ┌────────┴─────────┐    │   │
│ │         │           │     │  │ PlayerGameActor  │    │   │
│ │  Redis Pub/Sub:    │     │  │  - WebSocket     │◀───┼───┤
│ │  - match:enqueue   │     │  │  - 로비, PvE      │    │   │
│ │    :request        │     │  │  - Enqueue 대리   │    │   │
│ │                    │     │  └──────────────────┘    │   │
│ │                    │     │                          │   │
│ │                    │     │  ┌──────────────────┐    │   │
│ │                    │     │  │ BattleActor      │    │   │
│ │                    │     │  │  - calculate     │    │   │
│ │                    │     │  │  - publish       │    │   │
│ │                    │     │  │    pod:*:result  │    │   │
│ │                    │     │  └──────────────────┘    │   │
│ └────────────────────┘     └───────────────────────────┘   │
│                                        ▲                   │
│                              Redis Pub/Sub:                │
│                              pod:pod-a:match_result        │
│                              pod:pod-a:battle_result       │
└────────────────────────────────────────┼───────────────────┘
                                         │
                                    WebSocket (유일한 연결)
                                         │
                                         ▼
                              ┌──────────────────┐
                              │  Player 1        │
                              │  (Unity Client)  │
                              └──────────────────┘
```

### 핵심 설계 원칙

1. **단일 WebSocket 연결**: 플레이어는 Game Server에만 연결
2. **서버 간 통신**: Redis Pub/Sub 사용
3. **Game Server = Authoritative**: 모든 플레이어 상태 소유
4. **Match Server = 내부 서비스**: 클라이언트 직접 접근 불가

---

## 통신 흐름

### 1. 플레이어 로비 입장

```
Player (Unity)
  │ Auth Token
  ▼
Auth Server
  │ 검증 성공
  ▼
Game Server
  │ PlayerGameActor 생성 또는 재접속
  ▼
WebSocket 수립 (유일한 연결)
```

### 2. PvP 매칭 요청

```
Player
  │ "PvP 시작" 버튼 클릭
  ▼
Game Server (PlayerGameActor)
  │ 1. 플레이어 준비도 검증 (덱, 레벨, 아이템)
  │ 2. metadata 생성 (서버에서, 조작 불가)
  │ 3. Redis Pub/Sub 발행
  ▼
Redis: "match:enqueue:request"
  {
    player_id: "uuid",
    game_mode: "Ranked",
    metadata: {...},  // Game Server가 생성
    pod_id: "pod-a"
  }
  ▼
Match Server (구독 중)
  │ Matchmaker Actor
  │ Redis Lua Script
  ▼
Redis
  ├─ ZADD queue:ranked {timestamp} {player_id}
  └─ SET metadata:{player_id} {json}
  ▼
Match Server
  │ Redis Pub/Sub 발행
  ▼
Redis: "pod:pod-a:match_result"
  {
    player_id: "uuid",
    result: "EnQueued"
  }
  ▼
Game Server (구독 중)
  │ LoadBalanceActor
  │ player_id로 PlayerGameActor 찾기
  ▼
PlayerGameActor
  │ WebSocket
  ▼
Player
  └─ "매칭 대기 중..." UI
```

### 3. 매칭 성사 (TryMatch)

```
Match Server (5초마다)
  │ TryMatch handler
  ▼
pop_candidates()
  │ Redis Lua Script (ZPOPMIN)
  ▼
[player1@pod-a, player2@pod-b]
  │
  ├─> Redis: "battle:request"
  │    {
  │      player1: {id, pod_id: "pod-a", deck, ...},
  │      player2: {id, pod_id: "pod-b", deck, ...}
  │    }
  │    ▼
  │    Game Server (player1.pod_id == "pod-a")
  │    └─> BattleActor 생성
  │
  └─> Redis: "pod:pod-a:match_result", "pod:pod-b:match_result"
       {
         player_id: "uuid",
         result: "MatchFound",
         opponent_id: "uuid2"
       }
       ▼
       각 Game Server
       └─> PlayerGameActor
            └─> Player (WebSocket)
```

### 4. 전투 처리 및 결과 전달

```
BattleActor (Pod A)
  │ 전투 시뮬레이션
  │ Event Timeline 생성
  ▼
Redis Pub/Sub 발행
  ├─> "pod:pod-a:battle_result"
  │    {player_id: p1, battle_data: {...}}
  │
  └─> "pod:pod-b:battle_result"
       {player_id: p2, battle_data: {...}}
  ▼
각 Pod의 Game Server
  │ LoadBalanceActor
  │ player_id로 PlayerGameActor 찾기
  ▼
PlayerGameActor
  │ WebSocket
  ▼
Player
  └─ 전투 재생
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
    → BattleActor에 필요한 전투 스냅샷 (Game Server가 생성)
```

### Pub/Sub 채널

```
Redis Pub/Sub Channels

[Match Server 구독]
├── match:enqueue:request              → Game Server가 발행
├── match:dequeue:request              → Game Server가 발행
└── (Match Server가 받는 요청)

[Match Server 발행]
├── pod:{pod_id}:match_result          → Game Server가 구독
├── battle:request                     → 모든 Game Server 구독
└── (Match Server가 보내는 응답)

[Game Server 구독]
├── pod:{pod_id}:match_result          → 자기 Pod만
├── pod:{pod_id}:battle_result         → 자기 Pod만
├── battle:request                     → 모든 Pod
└── (Game Server가 받는 메시지)

[Game Server 발행]
├── match:enqueue:request              → Match Server가 구독
├── match:dequeue:request              → Match Server가 구독
├── pod:{pod_id}:battle_result         → 타겟 Pod (크로스 Pod 전투)
└── (Game Server가 보내는 요청)
```

**핵심 원칙:**

- 플레이어는 **Game Server에만 연결**
- Match Server는 **내부 서비스** (Redis Pub/Sub로만 통신)
- **WebSocket 종료 시** Game Server가 자동으로 Dequeue 요청 발행
- **연결 상태는 Game Server가 단일 진실 원천으로 관리**

---

## Game Server 상세

### 시작 시 초기화

```rust
async fn start_game_server() {
    let our_pod_id = env::var("POD_ID").unwrap();
    let redis = ConnectionManager::new(...).await;

    // 1. match:result 구독 (매칭 결과 수신용)
    spawn(subscribe_match_results(redis.clone(), our_pod_id.clone()));

    // 2. battle:request 구독 (전투 생성용)
    spawn(subscribe_battle_requests(redis.clone(), our_pod_id.clone()));

    // 3. pod:{our_pod_id}:battle_result 구독 (결과 수신용)
    spawn(subscribe_battle_results(redis.clone(), our_pod_id.clone()));

    // 4. LoadBalanceActor 시작
    let load_balancer = LoadBalanceActor::start();

    // 5. WebSocket 서버 시작
    HttpServer::new(...).bind(...).run().await;
}
```

### PlayerGameActor

```rust
impl PlayerGameActor {
    /// PvP 매칭 진입
    async fn enter_pvp_queue(&self, game_mode: GameMode) -> Result<()> {
        // 1. 플레이어 준비도 검증
        if !self.is_ready_for_pvp() {
            return Err("Not ready: incomplete deck");
        }

        // 2. metadata 생성 (서버에서, 조작 불가)
        let metadata = self.build_pvp_metadata();

        // 3. Match Server에 대리 요청
        self.redis.publish(
            "match:enqueue:request",
            serde_json::to_string(&EnqueueRequest {
                player_id: self.player_id,
                game_mode,
                metadata,
                pod_id: self.pod_id.clone(),
            }).unwrap()
        ).await?;

        Ok(())
    }

    /// 매칭 결과 수신
    async fn on_match_result(&self, result: MatchResult) {
        match result.result_type {
            MatchResultType::EnQueued => {
                self.send_to_player(ServerMessage::EnQueued).await;
            }
            MatchResultType::MatchFound { opponent_id } => {
                self.send_to_player(ServerMessage::MatchFound {
                    opponent_id,
                }).await;
            }
            MatchResultType::Error { code, message } => {
                self.send_to_player(ServerMessage::Error {
                    code,
                    message,
                }).await;
            }
        }
    }

    /// WebSocket 종료 시 자동 호출
    async fn on_disconnect(&self) {
        // Dequeue 요청
        self.redis.publish(
            "match:dequeue:request",
            serde_json::to_string(&DequeueRequest {
                player_id: self.player_id,
                game_mode: self.game_mode,
            }).unwrap()
        ).await;
    }
}
```

### LoadBalanceActor

- PlayerGameActor 추적/관리
- 내부적으로 `HashMap<player_id, Addr<PlayerGameActor>>` 보유
- 재접속 시 기존 Actor 찾기 지원
- **매칭/전투 결과 라우팅에 사용** (player_id → PlayerGameActor)

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

### Redis Pub/Sub 구독 핸들러

```rust
// match_server/src/main.rs
async fn main() {
    let redis = ConnectionManager::new(...).await;
    let matchmakers = spawn_matchmakers(...);

    // "match:enqueue:request" 채널 구독
    spawn(subscribe_enqueue_requests(
        redis.clone(),
        matchmakers.clone()
    ));

    // "match:dequeue:request" 채널 구독
    spawn(subscribe_dequeue_requests(
        redis.clone(),
        matchmakers.clone()
    ));

    // HTTP 서버 시작 (메트릭, health check만)
    HttpServer::new(...)
        .bind("0.0.0.0:8080")
        .run()
        .await;
}

async fn subscribe_enqueue_requests(
    redis: ConnectionManager,
    matchmakers: HashMap<GameMode, MatchmakerAddr>
) {
    let mut pubsub = redis.into_pubsub();
    pubsub.subscribe("match:enqueue:request").await.unwrap();

    while let Some(msg) = pubsub.on_message().next().await {
        let payload: String = msg.get_payload().unwrap();
        let req: EnqueueRequest = serde_json::from_str(&payload).unwrap();

        if let Some(matchmaker) = matchmakers.get(&req.game_mode) {
            matchmaker.send(Enqueue {
                player_id: req.player_id,
                game_mode: req.game_mode,
                metadata: req.metadata,
            }).await;
        }
    }
}
```

### TryMatch (주기적 실행)

```rust
// Matchmaker Actor (5초마다)
impl Handler<TryMatch> for RankedMatchmaker {
    fn handle(&mut self, msg: TryMatch, ctx: &mut Self::Context) {
        let deps = self.deps.clone();

        async move {
            // 1. Redis에서 플레이어 pop
            let (candidates, poisoned) = pop_candidates(
                "ranked",
                4,  // batch_size
                &deps
            ).await?;

            // 2. 2명씩 매칭
            for chunk in candidates.chunks(2) {
                match chunk {
                    [player1, player2] => {
                        // 3. battle:request 발행
                        publish_battle_request(
                            &mut redis,
                            "battle:request",
                            &BattleRequest {
                                player1: player1.clone(),
                                player2: player2.clone(),
                            }
                        ).await?;

                        // 4. 각 플레이어에게 MatchFound 통知
                        publish_match_result(
                            &mut redis,
                            &player1.pod_id,
                            MatchResult {
                                player_id: player1.player_id,
                                result_type: MatchResultType::MatchFound {
                                    opponent_id: player2.player_id,
                                },
                            }
                        ).await;

                        publish_match_result(
                            &mut redis,
                            &player2.pod_id,
                            MatchResult {
                                player_id: player2.player_id,
                                result_type: MatchResultType::MatchFound {
                                    opponent_id: player1.player_id,
                                },
                            }
                        ).await;
                    }
                    [single] => {
                        // 홀수 남은 플레이어 재enqueue
                        re_enqueue_candidates(...).await;
                    }
                    _ => unreachable!(),
                }
            }
        }
        .into_actor(self)
        .spawn(ctx);
    }
}
```

---

## 메시지 프로토콜

### Enqueue Request (Game Server → Match Server)

```rust
#[derive(Serialize, Deserialize)]
pub struct EnqueueRequest {
    pub player_id: Uuid,
    pub game_mode: GameMode,
    pub metadata: String,  // JSON, Game Server가 생성
    pub pod_id: String,
}

// Redis Pub/Sub
// Channel: "match:enqueue:request"
```

### Match Result (Match Server → Game Server)

```rust
#[derive(Serialize, Deserialize)]
pub struct MatchResult {
    pub player_id: Uuid,
    pub result_type: MatchResultType,
}

#[derive(Serialize, Deserialize)]
pub enum MatchResultType {
    EnQueued,
    MatchFound { opponent_id: Uuid },
    Dequeued,
    Error { code: ErrorCode, message: String },
}

// Redis Pub/Sub
// Channel: "pod:{pod_id}:match_result"
```

### Battle Request (Match Server → Game Server)

```rust
#[derive(Serialize, Deserialize)]
pub struct BattleRequest {
    pub player1: PlayerCandidate,
    pub player2: PlayerCandidate,
}

#[derive(Serialize, Deserialize)]
pub struct PlayerCandidate {
    pub player_id: String,
    pub score: i64,
    pub pod_id: String,
    pub metadata: serde_json::Value,
}

// Redis Pub/Sub
// Channel: "battle:request"
```

---

## 보안 개선

### 현재 구현 (취약)

```rust
// ❌ 클라이언트가 metadata 직접 전송
ClientMessage::Enqueue {
    player_id: Uuid,
    metadata: String,  // 조작 가능!
}
```

### 목표 구현 (안전)

```rust
// ✅ Game Server가 metadata 생성
impl PlayerGameActor {
    fn build_pvp_metadata(&self) -> String {
        // 서버에서 검증된 데이터만 사용
        serde_json::to_string(&PvpMetadata {
            deck: self.deck.clone(),         // 서버 검증됨
            level: self.level,               // 서버 검증됨
            artifacts: self.artifacts.clone(), // 서버 검증됨
            items: self.items.clone(),       // 서버 검증됨
            pod_id: self.pod_id.clone(),
        }).unwrap()
    }
}
```

---

## Game Server 장애 처리

### "Game Server 죽음 = 플레이어 연결 끊김" ✅

```
Game Server 죽음
  │
  ├─> 모든 PlayerGameActor 종료
  │    └─> WebSocket 연결 끊김
  │         └─> 플레이어는 요청 불가
  │
  ├─> Match Server: subscriber_count == 0 감지
  │    (battle:request 채널에 구독자 없음)
  │
  └─> Match Server 조치:
       ├─ 연속 5번 실패 확인 (30초)
       ├─ Redis 큐의 모든 플레이어 조회 (ZSCAN)
       ├─ 각 플레이어 Dequeue (Redis에서만 제거)
       ├─ Maintenance Mode 진입
       │   ├─ is_maintenance = true
       │   └─ redis.set("maintenance:flag", "1", EX 300)
       │
       └─ K8s Health Check:
            ├─ /health/game-server → 500
            ├─ /ready → 503
            └─ K8s가 Pod 재시작
```

**전제 성립:**
- 플레이어는 Game Server에만 연결
- Game Server 죽음 = 모든 WebSocket 끊김
- 큐의 플레이어도 사실상 오프라인
- 안전하게 Redis에서만 정리 가능

---

## 크로스 Pod 매칭 처리

### 시나리오: Pod A의 Player1 + Pod B의 Player2

**1. 매칭 성사 (Pod A Match Server)**
```rust
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

## 구현 우선순위

### Phase 1 (완료) ✅

1. ✅ Enqueue/Dequeue operations (Lua Scripts 포함)
2. ✅ NormalMatchmaker TryMatch 구현
3. ✅ RankedMatchmaker (MMR 기반)
4. ✅ WebSocket Session 관리 (레거시, 향후 제거)
5. ✅ SubScriptionManager (레거시, 향후 역할 축소)
6. ✅ Rate Limiter (10 req/sec per IP)
7. ✅ Prometheus Metrics (/metrics endpoint)
8. ✅ CancellationToken 기반 Graceful Shutdown

### Phase 2 (현재) ⚠️

1. ⚠️ **Match Server Redis Pub/Sub 구독**
   - `match:enqueue:request` 핸들러
   - `match:dequeue:request` 핸들러
   - 결과를 `pod:{pod_id}:match_result`로 발행

2. ⚠️ **Game Server 구현** (별도 프로젝트: `game_server/`)
   - PlayerGameActor WebSocket 관리
   - `enter_pvp_queue()` → Match Server 대리 요청
   - `pod:{pod_id}:match_result` 구독
   - battle:request 구독 → BattleActor 생성
   - BattleActor 전투 로직 (Event Timeline)
   - pod:{pod_id}:battle_result 구독 → PlayerGameActor 전달
   - LoadBalanceActor로 player_id → PlayerGameActor 찾기

3. ⚠️ **통합 테스트** (Match Server + Game Server)

### Phase 3 (계획) ❌

1. ❌ **Unity 클라이언트 수정**
   - Match Server WebSocket 제거
   - Game Server WebSocket만 사용
   - PvP 버튼 클릭 → Game Server로 요청

2. ❌ **Match Server WebSocket 엔드포인트 제거**
   - `/ws/` 제거
   - Session Actor 제거
   - SubScriptionManager 역할 축소 (또는 제거)

3. ❌ PartyMatchmaker 구현
4. ❌ Battle Timeline gzip 압축
5. ❌ 고급 메트릭 및 알람 (Grafana, Alertmanager)

---

## 파일 구조 (목표)

```
match_server/
├── src/
│   ├── main.rs                    ✅ Redis Pub/Sub 구독 (신규)
│   ├── lib.rs                     ✅ AppState, 공통 로직
│   ├── env.rs                     ✅ 설정 로드 (TOML)
│   ├── metrics.rs                 ✅ Prometheus 메트릭
│   ├── protocol.rs                ⚠️ 메시지 프로토콜 (수정 필요)
│   │
│   ├── pubsub/                    ❌ 신규 모듈
│   │   ├── mod.rs                 ❌ Redis Pub/Sub 핸들러
│   │   ├── enqueue_handler.rs    ❌ match:enqueue:request
│   │   └── dequeue_handler.rs    ❌ match:dequeue:request
│   │
│   └── matchmaker/
│       ├── mod.rs                 ✅ Matchmaker 팩토리
│       ├── common.rs              ✅ MatchmakerInner
│       ├── messages.rs            ✅ Enqueue, Dequeue, TryMatch 메시지
│       ├── scripts.rs             ✅ Lua 스크립트
│       │
│       ├── operations/
│       │   ├── mod.rs             ✅ 모듈 export
│       │   ├── enqueue.rs         ✅ Enqueue 로직
│       │   ├── dequeue.rs         ✅ Dequeue 로직
│       │   ├── notify.rs          ⚠️ Redis Pub/Sub 발행으로 변경
│       │   └── try_match.rs       ✅ pop_candidates, publish_battle_request
│       │
│       ├── normal/
│       │   ├── mod.rs             ✅ NormalMatchmaker Actor
│       │   └── handlers.rs        ✅ 핸들러 (완료)
│       │
│       ├── rank/
│       │   ├── mod.rs             ✅ RankedMatchmaker Actor
│       │   └── handlers.rs        ✅ MMR 기반 매칭 (완료)
│       │
│       └── patry/
│           └── mod.rs             ❌ 미구현
│
└── config/
    ├── development.toml           ✅ 개발 환경 설정
    └── production.toml            ✅ 운영 환경 설정

game_server/                       ⚠️ 별도 프로젝트 (구현 중)
└── src/
    ├── main.rs                    ⚠️ Game Server 진입점
    ├── player_game_actor/         ⚠️ 플레이어 게임 Actor
    ├── load_balance_actor/        ⚠️ 플레이어 라우팅
    └── battle_actor/              ⚠️ 전투 로직
```

---

## 다음 단계

### Match Server

1. **Redis Pub/Sub 구독 핸들러 구현**
   ```rust
   // match_server/src/pubsub/mod.rs (신규)
   async fn subscribe_enqueue_requests(...)
   async fn subscribe_dequeue_requests(...)
   ```

2. **notify.rs 수정**
   - 현재: SubScriptionManager → Session Actor → WebSocket
   - 목표: Redis Pub/Sub 발행 (`pod:{pod_id}:match_result`)

3. **WebSocket 엔드포인트 제거 준비**
   - main.rs의 `/ws/` 라우트 deprecate
   - Session Actor 제거 일정

### Game Server

4. **PlayerGameActor 구현**
   ```rust
   impl PlayerGameActor {
       async fn enter_pvp_queue(...);
       async fn on_match_result(...);
       async fn on_disconnect(...);
   }
   ```

5. **Redis Pub/Sub 구독**
   ```rust
   subscribe_match_results(redis, pod_id);
   subscribe_battle_requests(redis, pod_id);
   subscribe_battle_results(redis, pod_id);
   ```

6. **LoadBalanceActor 구현**
   ```rust
   pub struct LoadBalanceActor {
       players: HashMap<Uuid, Addr<PlayerGameActor>>,
   }
   ```

### Unity 클라이언트

7. **Match Server WebSocket 제거**
   - Game Server WebSocket만 사용
   - PvP 버튼 → Game Server로 요청

---

## 참고

- **현재 구현**: [ARCHITECTURE_CURRENT.md](./ARCHITECTURE_CURRENT.md)
- **보안 개선**: Game Server가 metadata 생성 (조작 불가)
- **단일 연결**: 클라이언트 코드 단순화
- **명확한 책임**: Game Server = 플레이어 상태 소유자
