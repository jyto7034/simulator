# Match Server - 현재 구현 상태 (2025-10-15)

> ⚠️ 이 문서는 현재 코드베이스의 실제 구현을 기록합니다.
> 향후 Game Server 중심 설계로 전환 예정입니다.

---

## 현재 아키텍처

### 플레이어 연결 구조

```
Player (Unity)
  │
  ├─> Game Server WebSocket
  │    └─> PlayerGameActor
  │         ├─ 로비, 상점, 친구, PvE
  │         └─ 게임 진행 상태 관리
  │
  └─> Match Server WebSocket (/ws/)  ← 현재 구현
       └─> Session Actor (플레이어별 1개)
            ├─ State Machine (Idle → InQueue → Completed)
            ├─ Enqueue/Dequeue 처리
            └─> Matchmaker Actor
```

**문제점:**
- 플레이어가 **두 개의 WebSocket 연결** 관리
- Match Server가 플레이어와 **직접 통신**
- 클라이언트가 metadata를 직접 전송 (조작 가능)

---

## 현재 통신 흐름

### 1. Enqueue 요청

```
Player (Unity)
  │ WebSocket
  ▼
Match Server (/ws/)
  │ ClientMessage::Enqueue
  ▼
Session Actor
  │ 1. Rate limiting 체크
  │ 2. State: Idle → Enqueuing
  │ 3. SubScriptionManager 등록
  │ 4. Matchmaker 호출
  ▼
Matchmaker Actor
  │ Redis Lua Script
  ▼
Redis
  ├─ ZADD queue:{mode} {timestamp} {player_id}
  └─ SET metadata:{player_id} {json}
  ▼
Matchmaker
  │ notify::send_message_to_player()
  ▼
SubScriptionManager
  │ sessions.get(player_id)
  ▼
Session Actor
  │ ServerMessage::EnQueued
  ▼
Player (Unity)
```

### 2. TryMatch (매칭 시도)

```
Matchmaker Actor (5초 간격)
  │ TryMatch 메시지
  ▼
pop_candidates()
  │ Redis Lua Script (ZPOPMIN)
  ▼
[player1, player2] 매칭
  │
  ├─> Redis Pub/Sub
  │    └─ publish("battle:request", {player1, player2})
  │         └─> Game Server 구독 중
  │              └─> BattleActor 생성
  │
  └─> notify::send_message_to_player()
       └─> SubScriptionManager
            └─> Session Actor
                 └─> Player (ServerMessage::MatchFound)
```

### 3. Session Actor 구조

```rust
// session/mod.rs:24-37
pub struct Session {
    state: SessionState,                      // State Machine
    matchmaker_addr: OnceCell<MatchmakerAddr>, // Lazy 초기화
    subscript_addr: Addr<SubScriptionManager>,
    app_state: web::Data<AppState>,
    player_id: Uuid,                          // Session 생성 시 랜덤 생성
    game_mode: GameMode,
    heartbeat_interval: Duration,             // 30초
    heartbeat_timeout: Duration,              // 120초
    last_heartbeat: Instant,
    cleanup_started: bool,
    client_ip: IpAddr,                        // Rate limiting용
    metadata: Option<String>,                 // 플레이어 metadata
}
```

**State Machine:**
```
Idle (초기)
  ↓ Enqueue
Enqueuing (처리 중)
  ↓ EnQueued
InQueue (대기 중)
  ↓ Dequeue or MatchFound
Dequeued / Completed
```

### 4. SubScriptionManager

```rust
// subscript/mod.rs:11-24
pub struct SubScriptionManager {
    pub sessions: HashMap<Uuid, Addr<Session>>,
}

// 역할:
// 1. player_id → Session Actor 매핑
// 2. Matchmaker가 플레이어에게 메시지 전달 시 사용
// 3. 연결 상태 추적 (단일 진실 원천)
```

### 5. 메시지 전달 (notify.rs)

```rust
pub async fn send_message_to_player(
    subscription_addr: Addr<SubScriptionManager>,
    redis: &mut ConnectionManager,
    player_id: Uuid,
    message: ServerMessage,
) {
    // 1. SubScriptionManager를 통해 Session Actor 찾기
    subscription_addr.send(ForwardServerMessage {
        player_id,
        message: message.clone(),
    }).await;

    // 2. Redis로도 발행 (관측성/디버깅용)
    redis.publish(
        format!("notification:{}", player_id),
        serde_json::to_string(&message).unwrap()
    ).await;
}
```

---

## 현재 구현의 문제점

### 1. 이중 WebSocket 연결

**문제:**
```
Player
  ├─> Game Server WebSocket (로비, PvE, 친구, ...)
  └─> Match Server WebSocket (매칭만)
```

**단점:**
- 클라이언트 복잡도 증가 (두 연결 관리)
- 재연결 로직 2배
- 네트워크 부하
- 연결 상태 불일치 가능성

### 2. 보안 취약점

```rust
// protocol.rs:12-27
#[serde(rename = "enqueue")]
Enqueue {
    player_id: Uuid,
    game_mode: GameMode,
    metadata: String,  // ⚠️ 클라이언트가 직접 전송
}
```

**문제:**
- 클라이언트가 `metadata` 조작 가능
  - 덱 구성 변조
  - 레벨/스탯 조작
  - 아이템 추가
- Match Server는 검증 없이 그대로 Redis에 저장

**현재 검증 부재:**
```rust
// session/mod.rs:209-277
fn handle_enqueue(..., metadata: String) {
    // ❌ metadata 검증 없음
    // ❌ 덱 유효성 체크 없음
    // ❌ 플레이어 레벨 확인 없음

    matchmaker.do_send_enqueue(Enqueue {
        metadata,  // 그대로 전달
    });
}
```

### 3. 책임 분리 불명확

**현재:**
- Match Server가 플레이어 세션 관리 (Session Actor)
- Game Server도 플레이어 세션 관리 (PlayerGameActor)
- **중복된 책임**

**혼란:**
- 플레이어 연결 상태는 어디서 관리?
- 재접속 시 어느 서버로?
- 플레이어 데이터는 어디가 진실 원천?

### 4. Game Server 죽음 처리 복잡

```
Game Server 죽음
  ├─ PlayerGameActor 전멸
  ├─ BUT Match Server Session은 살아있음
  └─ 플레이어는 여전히 Enqueue 가능 ⚠️
       (하지만 Game Server 없어서 전투 불가)
```

**문제:**
- Match Server 입장: 플레이어 정상 연결됨
- 실제: Game Server 죽어서 게임 진행 불가
- **불일치 상태**

---

## 목표 아키텍처 (향후 전환)

### 단일 WebSocket 설계

```
Player (Unity)
  │
  └─> Game Server WebSocket (유일한 연결)
       └─> PlayerGameActor
            ├─ 로비, 상점, 친구, PvE (직접 처리)
            │
            └─ PvP 진입 시:
                 └─> Match Server에 대리 요청
                      (Redis Pub/Sub 또는 HTTP)
```

### 새로운 통신 흐름

```
Player
  │ "PvP 시작" 버튼
  ▼
Game Server (PlayerGameActor)
  │ 1. 플레이어 상태 검증 (덱, 레벨, 준비도)
  │ 2. metadata 생성 (서버에서 직접, 조작 불가)
  │ 3. Match Server에 대리 요청
  ▼
Redis Pub/Sub: "match:enqueue:request"
  ▼
Match Server (구독 중)
  │ Matchmaker Actor
  │ Redis Queue 추가
  ▼
Redis Pub/Sub: "pod:{pod_id}:match_result"
  ▼
Game Server (구독 중)
  │ PlayerGameActor 찾기
  ▼
Player (Unity)
  └─ "매칭 대기 중..." UI 표시
```

### 장점

1. **단일 연결**: 클라이언트 코드 단순
2. **보안**: Game Server가 metadata 검증/생성
3. **명확한 책임**: Game Server = 플레이어 상태 소유자
4. **일관성**: "Game Server 죽음 = 플레이어 연결 끊김" (전제 성립)
5. **내부 서비스**: Match Server = 백엔드 전용 (클라이언트 직접 접근 X)

---

## 전환 계획

### Phase 1: 현재 상태 유지 + 분석
- [x] 현재 구현 정확히 문서화
- [ ] 보안 취약점 테스트 (metadata 조작)
- [ ] 이중 연결의 실제 문제 측정

### Phase 2: 병렬 구현
- [ ] Game Server → Match Server 통신 구현 (Redis Pub/Sub)
- [ ] Match Server 구독 핸들러 추가
- [ ] 기존 WebSocket 엔드포인트 유지 (호환성)

### Phase 3: 클라이언트 전환
- [ ] Unity 클라이언트 수정 (Match Server WebSocket 제거)
- [ ] Game Server WebSocket만 사용
- [ ] A/B 테스트

### Phase 4: 레거시 제거
- [ ] Match Server WebSocket 엔드포인트 제거 (`/ws/`)
- [ ] Session Actor 제거
- [ ] SubScriptionManager 역할 축소 또는 제거

---

## 현재 코드 참조

| 파일 | 역할 | 상태 |
|------|------|------|
| `main.rs:20-42` | `/ws/` WebSocket 엔드포인트 | ✅ 구현됨 (레거시) |
| `session/mod.rs` | Session Actor (플레이어별) | ✅ 구현됨 (레거시) |
| `session/helper.rs` | State Machine | ✅ 구현됨 |
| `subscript/mod.rs` | SubScriptionManager | ✅ 구현됨 |
| `matchmaker/operations/notify.rs` | 메시지 전달 | ✅ 구현됨 |
| `protocol.rs` | ClientMessage/ServerMessage | ✅ 구현됨 |

---

## 다음 단계

1. **보안 테스트**: metadata 조작 테스트
2. **Game Server 구현**: PlayerGameActor → Match Server 통신
3. **Match Server 구독**: Redis Pub/Sub 핸들러
4. **클라이언트 수정**: 단일 WebSocket 전환
5. **레거시 제거**: Session Actor 삭제

---

## 참고

- 현재 Match Server는 **독립형 서비스**처럼 동작
- 플레이어가 Match Server에 **직접 연결**
- 향후 **내부 백엔드 서비스**로 전환 예정
