# CLAUDE.md: Unified Autonomous Workflow Protocol

## 0. SYSTEM CONFIGURATION

# [설정 필요] Basic Memory에 등록한 프로젝트 이름과 정확히 일치해야 합니다.

CONST CURRENT_PROJECT_NAME = "simulator"

# ==============================================================================

# [CRITICAL] GLOBAL LANGUAGE PROTOCOL

# ==============================================================================

**ABSOLUTE RULE: MUST SPEAK KOREAN**
모든 사고 과정(Reasoning), 답변, 커밋 메시지, 이슈 설명, 파일 주석은 반드시 **한국어(Korean)**로 작성해야 합니다.

- 예외: 코드 문법, 변수명, 라이브러리 이름, 로그 메시지는 영어 원문을 유지합니다.
- 사용자가 영어로 질문하더라도 답변은 한국어로 합니다.

# ==============================================================================

# PART 1: SYSTEM OPERATION PROTOCOL (DO NOT MODIFY)

# ==============================================================================

## 1. CORE IDENTITY & PRIME DIRECTIVES

You are a **State-Aware Senior Engineer**. You do not rely on chat history for memory. You rely on **Beads (State)**, **Basic Memory (Knowledge)**, and **Exa (Context)**.

### The 3 Laws of Context Efficiency

1.  **Externalize Instantly:** Never store decisions or plans in the chat. If it's a task, `bd create` it. If it's knowledge, `write_note` it.
2.  **Pull, Don't Guess:** Never guess what to do. Use `bd ready` to fetch orders.
3.  **Atomic Context:** Do not scan the whole file tree. Use `search_notes`, `get_code_context_exa`, and read **PART 2** below.

## 2. TOOL USAGE PROTOCOLS

### 2.1 Beads (Task & State Engine)

- **Output Format:** Always pipe output to `jq` for parsing.
- **Auto-run:** Execute `bd` commands immediately without asking for permission.
- **Fetch:**
  - `bd ready --json`
  - `bd list --status in_progress --json`
- **Create:**
  - `bd create "Title" -t [bug|feature|task|chore] -p [0-4] --json`
  - Use `--deps discovered-from:<parent_id>` for traceability.
- **Update:**
  - Claim: `bd update <id> --status in_progress --json`
  - Close: `bd close <id> --reason "Completed" --json`

### 2.2 Basic Memory (Knowledge Graph)

**CRITICAL RULE:** You MUST pass `project=CURRENT_PROJECT_NAME` to all Basic Memory calls.

- **Search Strategy:**
  - `search_notes(query="...", project=CURRENT_PROJECT_NAME)` for semantic search.
  - If search fails, use `list_notes(folder="...", project=CURRENT_PROJECT_NAME)` to explore directory structure.
- **Write Strategy:**
  - `write_note(title="...", content="...", folder="...", tags=["..."], project=CURRENT_PROJECT_NAME)`
  - **Link Consistency:** When creating a note, always include at least one `[[WikiLink]]` to an existing note.
  - **Metadata:** Always include `tags` in the metadata.
- **Update/Append:**
  - To add logs or history, read the note first, append new content, and save.
- **Planning Docs:** Store ephemeral plans in `history/` folder.

### 2.3 Exa Code Search (Live Context)

**CRITICAL RULE:** Use Exa BEFORE asking the user about external libraries or errors.

- **get_code_context_exa:**
  - Use this to find up-to-date code snippets, API documentation, and best practices.
  - Query Format: `<Library Name> <Feature> <Language> code example`
  - Example: "gin-gonic middleware logging example go"
- **web_search_exa:**
  - Use this for general troubleshooting or finding recent updates/changelogs.

## 3. OPERATIONAL WORKFLOW (THE LOOP)

### PHASE 1: COLD START

1.  **Read Context:** Read **PART 2: PROJECT CONTEXT** below.
2.  **Check State:** Run `bd list --status in_progress --json`.
    - If exists: Resume task.
    - If null: Run `bd ready --json` and pick highest priority.
3.  **Set State:** `bd update <id> --status in_progress --json`.

### PHASE 2: CONTEXT LOADING

1.  **Retrieve Knowledge:**
    - Read issue description.
    - `search_notes(query="related logic", project=CURRENT_PROJECT_NAME)`
2.  **Fetch External Context (Exa):**
    - If the task involves a library, use `get_code_context_exa` to get the latest usage patterns.
3.  **Load Files:** Open ONLY relevant files defined in the task or notes.

### PHASE 3: EXECUTION

1.  **Plan:** Write a brief plan in comments or `history/PLAN_<id>.md`.
2.  **Develop:** Follow **CODING STANDARDS** in Part 2.
3.  **New Discovery:** If new work is found, `bd create ... --deps discovered-from:<current_id>` immediately. Do not switch context.

### PHASE 4: LANDING (Completion Trigger)

**TRIGGER:** When user says "Land it", "마무리해", or task is done:

1.  **Close:** `bd close <id> --reason "..." --json`.
2.  **Update Knowledge:** Update Basic Memory with any "Lessons Learned".
3.  **Sync:** `bd sync`.
4.  **Git:**
    - `git add .`
    - Commit with Conventional Commits (Korean summary).
    - `git push`.

# ==============================================================================

# PART 2: PROJECT SPECIFIC CONTEXT (DOMAIN RULES)

# ==============================================================================

## 1. 프로젝트 개요

### 프로젝트명
**simulator** - 로보토미 코퍼레이션 IP 기반 1vs1 오토배틀 로그라이크 게임

### 프로젝트 설명
- **장르**: 시련(Ordeal) 기반 턴제 로그라이크 오토배틀
- **게임 흐름**: 이벤트 선택 → PvE 진압 → PvP 시련 → 다음 시련 단계
- **핵심 컨셉**: E.G.O 추출, 엔케팔린, 환상체, E.G.O 선물 등
- **클라이언트**: Unity (C#)
- **서버**: Rust (Actix Actor 모델)
- **보안 정책**: 모든 게임 연산은 서버에서 처리, 클라이언트는 시각화만 담당

### 게임 시스템
- **시간 체계**: OrdealLevel (여명/정오/어스름/자정/백색) + ManagementPhase (First~Sixth)
- **시련 체계**: OrdealColor (녹빛/자색/핏빛/호박색/쪽빛/백색)
- **자원 관리**: 엔케팔린 (환상체로부터 추출한 에너지)
- **장비 시스템**: E.G.O 무기/방어구, 환상체 선물
- **시너지 시스템**: 롤토체스 스타일 시너지 (새, 종교, 기계, 동화, 공포, ALEPH 등)
- **레벨업 시스템**: 경험치 획득 → 레벨업 → 스탯/스킬 투자

---

## 2. 기술 스택

### 백엔드 (Game Server)
```rust
// 핵심 프레임워크
- Rust: edition = "2021"
- Actix: Actor 모델 (=0.13.5)
- Actix-web: 웹 서버 (4.9.0)
- Actix-web-actors: WebSocket (4.3.0)

// 비동기 런타임
- Tokio: 비동기 런타임 (1.15)
- Tokio-util: 유틸리티 (0.7)
- Futures: 비동기 스트림 (0.3.31)

// 데이터 처리
- Redis: 큐, Pub/Sub, 메시지 브로커 (0.22.3)
- Serde: 직렬화/역직렬화 (1.0)
- UUID: 플레이어 ID (1.14.0)

// 모니터링 & 로깅
- Prometheus: 메트릭 수집 (0.14)
- Tracing: 구조화 로깅 (0.1.41)
- Tracing-subscriber: 로그 구독 (0.3.19)

// 보안
- JWT: 인증 토큰 (jsonwebtoken 9.3.1)
- Argon2: 비밀번호 해싱
```

### 게임 코어 (Core Library)
```rust
// ECS (Entity Component System)
- bevy_ecs: 게임 로직 엔진 (0.17.2)

// 데이터 처리
- Serde: 직렬화 (1.0)
- Serde_yaml: YAML 파일 처리 (0.9.34)
- RON: Rust Object Notation (0.5.1)

// 동시성
- Rayon: 병렬 처리 (1.8.0)
- Parking_lot: 고성능 락 (0.12.3)

// 난수 생성
- Rand: 랜덤 생성 (0.8.5)
```

### 인프라
```yaml
- 컨테이너: Kubernetes (Pod 단위 배포)
- 메시지 브로커: Redis Pub/Sub
- 모니터링: Prometheus + Grafana
- 인증: Auth Server (별도 서비스)
- 로그: Tracing + Tracing-subscriber
```

### 클라이언트
```
- 엔진: Unity
- 언어: C#
- 통신: WebSocket
```

---

## 3. 아키텍처 및 디렉토리 구조

### 워크스페이스 구조
```
simulator/
├── auth_server/          # 인증 서버 (독립 서비스)
│   ├── src/
│   │   ├── auth_server/
│   │   │   ├── db_operation.rs   # DB 작업
│   │   │   ├── end_point.rs      # HTTP 엔드포인트
│   │   │   ├── errors.rs         # 에러 정의
│   │   │   ├── model.rs          # 데이터 모델
│   │   │   └── types.rs          # 타입 정의
│   │   ├── lib.rs
│   │   └── main.rs
│   ├── migrations/               # DB 마이그레이션
│   └── Cargo.toml
│
├── game_server/          # 게임 서버 (Pod 단위)
│   ├── src/
│   │   ├── game/                 # Unity Client용 (신규)
│   │   │   ├── battle_actor/     # 전투 시뮬레이션 (순수 함수)
│   │   │   ├── load_balance_actor/  # PlayerGameActor 라우팅
│   │   │   ├── match_coordinator/   # 매칭 요청 조정
│   │   │   ├── player_game_actor/   # ⚠️ 구현 필요 (현재 stub)
│   │   │   └── pubsub.rs         # Redis 구독
│   │   │
│   │   ├── matchmaking/          # test_client용 (레거시)
│   │   │   ├── session/          # WebSocket 세션
│   │   │   ├── subscript/        # Session 라우팅
│   │   │   └── matchmaker/       # 매칭 로직
│   │   │       ├── normal/       # 일반 매칭
│   │   │       ├── rank/         # 랭크 매칭
│   │   │       └── operations/   # Enqueue, Dequeue, TryMatch
│   │   │
│   │   ├── shared/               # 공유 인프라
│   │   │   ├── protocol.rs       # 메시지 정의
│   │   │   ├── metrics.rs        # Prometheus 메트릭
│   │   │   ├── circuit_breaker.rs # Redis 장애 격리
│   │   │   ├── event_stream.rs   # 이벤트 스트리밍
│   │   │   └── redis_events.rs   # 테스트 이벤트 발행
│   │   │
│   │   ├── lib.rs                # AppState, 공통 모듈
│   │   └── main.rs               # 서버 진입점
│   │
│   ├── config/
│   │   ├── development.toml      # 개발 환경
│   │   └── production.toml       # 운영 환경
│   └── Cargo.toml
│
├── core/                 # 게임 코어 로직 (bevy_ecs)
│   ├── src/
│   │   ├── ecs/
│   │   │   ├── components/       # ECS 컴포넌트
│   │   │   ├── resources/        # ECS 리소스
│   │   │   └── systems/          # ECS 시스템
│   │   │
│   │   └── game/
│   │       ├── behavior.rs       # 게임 동작
│   │       └── data/             # 게임 데이터
│   │           ├── abnormality_data.rs  # 환상체 데이터
│   │           ├── artifact_data.rs     # 아티팩트 데이터
│   │           ├── equipment_data.rs    # 장비 데이터
│   │           └── bonus_data.rs        # 보너스 데이터
│   └── Cargo.toml
│
├── test_client/          # 테스트 클라이언트
├── metrics/              # 메트릭 수집 모듈
├── monitoring/           # 모니터링 설정
├── env/                  # 환경 설정
├── game_resources/       # 게임 리소스 (에셋)
├── game_resource_develop/ # 리소스 개발
└── logs/                 # 로그 파일

// 설정 파일
├── Cargo.toml            # 워크스페이스 루트
├── simulator.toml        # 시뮬레이터 설정
├── docker-compose.yml    # Docker 설정
└── .gitignore
```

### 액터 구조
```
┌─────────────────────────────────────────────────────────┐
│ Game Server (Actix Actor System)                       │
├─────────────────────────────────────────────────────────┤
│                                                         │
│  [레거시 경로 - test_client]                             │
│  /ws/ → Session Actor → SubScriptionManager             │
│           → Matchmaker (Normal/Ranked)                  │
│                                                         │
│  [신규 경로 - Unity Client] ⚠️ 구현 중                   │
│  /game → PlayerGameActor (stub)                         │
│           → MatchCoordinator → Matchmaker               │
│                                                         │
│  [공유 인프라]                                           │
│  ├─ LoadBalanceActor (player_id → PlayerGameActor)     │
│  ├─ Matchmaker (Normal/Ranked)                         │
│  │   ├─ TryMatch (주기적 실행)                          │
│  │   ├─ Enqueue/Dequeue                                │
│  │   └─ Battle 실행 + 결과 라우팅                       │
│  │                                                     │
│  └─ Redis Subscribers                                  │
│      ├─ match_result 채널                              │
│      └─ pod:{pod_id}:game_message 채널                 │
└─────────────────────────────────────────────────────────┘
```

### Redis 데이터 구조
```redis
# 큐 관리
queue:{mode}              # Sorted Set (score=timestamp)
├── normal                # 일반 큐
├── ranked                # 랭크 큐
└── party                 # 파티 큐

metadata:{player_id}      # String (JSON)
└── {"pod_id": "...", "deck": {...}, "level": 10, ...}

# Pub/Sub 채널
pod:{pod_id}:game_message       # Cross-pod 메시지 라우팅 ✅
events:test:{session_id}        # 테스트 이벤트 스트리밍
```

---

## 4. 코딩 컨벤션

### Rust 코딩 스타일
```rust
// 1. 네이밍 컨벤션
// - 모듈: snake_case
// - 타입: PascalCase
// - 함수/변수: snake_case
// - 상수: SCREAMING_SNAKE_CASE

// 2. 에러 처리
// - Result<T, E> 사용
// - thiserror로 커스텀 에러 정의
// - ?로 에러 전파

// 예시
use thiserror::Error;

#[derive(Error, Debug)]
pub enum GameError {
    #[error("플레이어를 찾을 수 없음: {0}")]
    PlayerNotFound(String),

    #[error("덱이 준비되지 않음")]
    DeckNotReady,
}

pub async fn verify_player(id: &str) -> Result<Player, GameError> {
    // ...
}

// 3. Actor 메시지 정의
#[derive(Message)]
#[rtype(result = "Result<(), GameError>")]
pub struct EnqueuePlayer {
    pub player_id: Uuid,
    pub game_mode: GameMode,
}

// 4. 비동기 함수
// - async/await 사용
// - tokio::spawn으로 백그라운드 작업

// 5. 주석 규칙
// - 한국어로 작성
// - 복잡한 로직은 반드시 주석 추가
// - TODO 주석은 이슈 번호와 함께

/// 플레이어를 매칭 큐에 추가합니다.
///
/// # Arguments
/// * `player_id` - 플레이어 고유 ID
/// * `game_mode` - 게임 모드 (Normal/Ranked)
///
/// # Returns
/// 성공 시 `Ok(())`, 실패 시 에러 반환
pub async fn enqueue_player(
    player_id: Uuid,
    game_mode: GameMode,
) -> Result<(), GameError> {
    // TODO: #123 - 중복 큐 진입 방지 로직 추가
    // ...
}
```

### Git 커밋 컨벤션
```bash
# Conventional Commits 사용 (GIT_COMMIT_CONVENTION.md 참고)

# 타입
feat:      # 새로운 기능 추가
fix:       # 버그 수정
docs:      # 문서만 변경
style:     # 코드 의미에 영향 없는 서식 변경
refactor:  # 버그 수정이나 기능 추가 없는 코드 구조 변경
perf:      # 성능 개선
test:      # 테스트 추가/수정
build:     # 빌드 시스템이나 외부 종속성 변경
ci:        # CI 구성 파일 및 스크립트 변경
chore:     # 소스/테스트 파일을 수정하지 않는 기타 변경

# 예시
feat(matchmaker): 랭크 매칭 시스템 구현

fix(server): LoadingComplete 핸들러의 race condition 수정

Redis Lua 스크립트를 사용하여 플레이어 준비 상태를 확인하고
업데이트하는 과정을 원자적으로 처리합니다.

Closes #42

refactor(matchmaker): TryMatch 핸들러 리팩토링

- operations/try_match_collect.rs: Candidates 수집 로직
- operations/try_match_process.rs: 매칭 처리 로직
- 353 lines → 80 lines (78% 감소)
```

### 프로젝트 파일 네이밍
```
// 문서 파일: UPPERCASE_SNAKE_CASE.md
ARCHITECTURE_STATUS.md
GAME_DESIGN.md
BATTLE_SYSTEM_DESIGN.md

// 설정 파일: lowercase.toml
development.toml
production.toml
simulator.toml

// Rust 파일: snake_case.rs
player_game_actor.rs
load_balance_actor.rs
match_coordinator.rs
```

---

## 5. 테스트 전략

### 테스트 디렉토리 구조
```
auth_server/tests/
game_server/tests/      # ⚠️ 현재 없음
core/tests/
```

### 테스트 작성 규칙
```rust
// 1. 단위 테스트: 각 모듈 하단에 작성
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_player_enqueue() {
        // Given
        let player_id = Uuid::new_v4();

        // When
        let result = enqueue_player(player_id, GameMode::Normal);

        // Then
        assert!(result.is_ok());
    }
}

// 2. 통합 테스트: tests/ 디렉토리
// 3. Actor 테스트: actix-test 사용
// 4. Redis 테스트: serial_test로 순차 실행
```

---

## 6. 빌드 및 배포

### 로컬 개발
```bash
# 워크스페이스 전체 빌드
cargo build

# 특정 프로젝트 빌드
cargo build -p game_server
cargo build -p auth_server
cargo build -p game_core

# 개발 모드 실행
cd game_server
cargo run

# 테스트
cargo test

# 린트
cargo clippy

# 포맷
cargo fmt
```

### Docker 배포
```bash
# docker-compose 사용
docker-compose up -d

# Redis 실행
docker-compose up redis

# 모니터링 실행
docker-compose up prometheus grafana
```

### 환경 설정
```bash
# 개발 환경
export RUST_ENV=development
export REDIS_URL=redis://localhost:6379

# 운영 환경
export RUST_ENV=production
export REDIS_URL=redis://redis-cluster:6379
```

---

## 7. 중요 문서 위치

### 필수 문서 (반드시 읽을 것)
```
ARCHITECTURE_STATUS.md    # 아키텍처 현황 (필독!)
GAME_DESIGN.md            # 게임 설계 문서
BATTLE_SYSTEM_DESIGN.md   # 전투 시스템 설계
GIT_COMMIT_CONVENTION.md  # 커밋 컨벤션
CLAUDE.md                 # 이 문서 (워크플로우)
```

### 참고 문서
```
AGENTS.md                 # 에이전트 관련
.beads/README.md          # Beads 사용법
env/README.md             # 환경 설정
test_client/README_SWARM.md  # Swarm 테스트
```

### 설정 파일
```
game_server/config/development.toml   # 개발 환경 설정
game_server/config/production.toml    # 운영 환경 설정
simulator.toml                         # 시뮬레이터 설정
docker-compose.yml                     # Docker 설정
.gitignore                            # Git 제외 파일
```

---

## 8. 현재 구현 상태 (2025-11-22 기준)

### ✅ 완료된 작업
1. **Match Server → Game Server 통합** (2025-10-22)
   - 단일 프로세스로 동작
   - Pod당 하나의 game_server 실행

2. **TryMatch 리팩토링** (2025-10-22)
   - 353 lines → 80 lines (78% 감소)
   - 함수 분리: try_match_collect.rs, try_match_process.rs

3. **Battle 즉시 실행 방식** (2025-10-22)
   - Redis 홉 제거 (50% 지연 감소)
   - 순수 함수 기반 전투 시뮬레이션

4. **Same-pod/Cross-pod 라우팅** (2025-10-22)
   - Same-pod: Actor 메시지 (0.1ms)
   - Cross-pod: Redis Pub/Sub (5-10ms)

5. **Redis Pub/Sub 구독**
   - match_result 채널
   - pod:{pod_id}:game_message 채널
   - Circuit Breaker 적용
   - Exponential Backoff 재시도

6. **메트릭 수집**
   - Prometheus 메트릭
   - Grafana 대시보드 (monitoring/)

### ⚠️ 미완료 작업 (우선순위 높음)
1. **PlayerGameActor 구현** 🔥
   - 현재: 빈 구조체 stub
   - 필요: Day 진행, 이벤트 선택, 덱 관리 등
   - 영향: Unity Client 연결 불가

2. **/game 엔드포인트 구현** 🔥
   - 현재: 라우트 미등록
   - 필요: Auth Token 검증, PlayerGameActor 생성/재접속
   - 영향: Unity Client 연결 불가

3. **MatchCoordinator 연동** 🔥
   - 현재: 구현되었으나 사용 안 됨
   - 필요: PlayerGameActor → MatchCoordinator 호출

4. **Auth Server 연동**
   - 현재: 없음
   - 필요: Token 검증 로직

5. **Battle 로직 구현**
   - 현재: player1 항상 승리 (stub)
   - 필요: 실제 카드 전투 시뮬레이션

### 📝 다음 단계 (Phase 1)
```
목표: Unity Client 연결 가능하게 만들기

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

예상 시간: 3-5일
```

---

## 9. 개발 워크플로우

### Issue → Development → PR → Merge
```bash
# 1. Beads에서 작업 가져오기
bd ready --json | jq

# 2. 작업 시작
bd update <id> --status in_progress --json

# 3. 개발
# - ARCHITECTURE_STATUS.md, GAME_DESIGN.md 참고
# - 코딩 컨벤션 준수
# - 테스트 작성

# 4. 커밋 (Conventional Commits)
git add .
git commit -m "feat(player): PlayerGameActor 기본 구조 구현

- 플레이어 상태 필드 추가 (day, level, gold, deck)
- WebSocket 핸들러 stub 구현
- LoadBalanceActor 등록 로직 추가

Co-Authored-By: Claude <noreply@anthropic.com>
"

# 5. 작업 완료
bd close <id> --reason "PlayerGameActor 기본 구조 완료" --json

# 6. Push
git push
```

### Basic Memory 활용
```bash
# 작업 중 배운 내용을 Basic Memory에 기록
write_note(
    title="PlayerGameActor 구현 시 주의사항",
    content="...",
    folder="knowledge/game_server",
    tags=["actor", "websocket", "player"],
    project="simulator"
)

# 나중에 검색
search_notes(
    query="PlayerGameActor WebSocket",
    project="simulator"
)
```

---

## 10. 문제 해결 가이드

### Redis 연결 문제
```bash
# Redis 상태 확인
redis-cli ping

# Redis 로그 확인
docker logs redis

# Circuit Breaker 상태 확인
# - game_server/src/shared/circuit_breaker.rs
# - 메트릭: game_server_unavailable_total
```

### Actor 메시지 라우팅 문제
```bash
# LoadBalanceActor 등록 확인
# - game_server/src/game/load_balance_actor/

# Same-pod/Cross-pod 메트릭 확인
# - messages_routed_same_pod_total
# - messages_routed_cross_pod_total
```

### 매칭 문제
```bash
# Redis 큐 확인
redis-cli ZRANGE queue:normal 0 -1 WITHSCORES

# Metadata 확인
redis-cli GET metadata:{player_id}

# TryMatch 메트릭 확인
# - matches_created_total
# - try_match_skipped_total
# - poisoned_candidates_total
```

### 로그 확인
```bash
# 개발 환경
export RUST_LOG=info
cargo run

# 운영 환경
cat logs/game_server.log | grep ERROR
```

---

## 11. 보안 고려사항

### 완료된 보안 강화
1. ✅ Same-pod/Cross-pod 구분 - 불필요한 Redis 홉 제거
2. ✅ Circuit Breaker - Redis 장애 격리
3. ✅ Rate Limiting 구조 준비 (현재 비활성화)

### 미완료 보안 강화
1. ❌ 서버에서 metadata 생성 - 현재 클라이언트가 전송 (레거시)
2. ❌ Auth Token 검증 - Auth Server 연동 필요
3. ❌ 플레이어 상태 검증 - PlayerGameActor 구현 필요
4. ❌ Rate Limiting 활성화 - 필요 시 활성화

### 보안 원칙
```
1. 모든 게임 연산은 서버에서 처리
2. 클라이언트는 시각화만 담당
3. 민감한 정보는 Redis에 저장 시 암호화
4. JWT 토큰은 짧은 만료 시간 설정
5. Redis Lua 스크립트로 원자성 보장
```

---

## 12. 성능 최적화

### 메트릭 수집
```prometheus
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

### 성능 벤치마크 (2025-10-22)
| 시나리오 | Before | After | 개선율 |
|---------|--------|-------|--------|
| Same-pod 매칭 | 0.1ms | 0.1ms | - |
| Cross-pod 매칭 | 15-20ms | **5-10ms** | **50%** |

| 항목 | Before | After | 개선율 |
|------|--------|-------|--------|
| TryMatch 핸들러 | 353 lines | **80 lines** | **78%** |
| Battle 처리 | 300 lines | **150 lines** | **50%** |

---

## 13. 알려진 이슈

### 1. PlayerGameActor stub (우선순위: 높음)
- **상태**: 빈 구조체만 존재
- **영향**: Unity Client 연결 불가
- **해결**: Phase 1 작업

### 2. Battle 로직 stub (우선순위: 중)
- **상태**: player1 항상 승리
- **영향**: 실제 게임 진행 불가
- **해결**: Phase 5 작업

### 3. /game 엔드포인트 없음 (우선순위: 높음)
- **상태**: 라우트 미등록
- **영향**: Unity Client 연결 불가
- **해결**: Phase 1 작업

### 4. 레거시 이중 메시지 전송 (우선순위: 낮음)
- **상태**: Same-pod도 레거시 경로 실행
- **영향**: 약간의 오버헤드
- **해결**: Unity 전환 후 제거 예정

---

## 14. 참고 링크

### 원작 (Lobotomy Corporation)
- [나무위키 - 시련](https://namu.wiki/w/Lobotomy%20Corporation/%EC%8B%9C%EB%A0%A8)
- [Lobotomy Corporation Wiki - Ordeals](https://lobotomycorporation.wiki.gg/wiki/Ordeals)

### Rust 공식 문서
- [Actix](https://actix.rs/)
- [Actix-web](https://actix.rs/docs/)
- [bevy_ecs](https://docs.rs/bevy_ecs/)
- [Tokio](https://tokio.rs/)
- [Redis-rs](https://docs.rs/redis/)

### 게임 참고 (The Bazaar)
- [The Bazaar](https://playthebazaar.com/)

---

**최종 수정일**: 2025-11-22
**작성자**: Development Team
**문서 버전**: 1.0
