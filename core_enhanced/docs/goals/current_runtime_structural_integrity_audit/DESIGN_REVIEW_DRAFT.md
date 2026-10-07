# Runtime Architecture Design Review — Draft

- 문서 상태: `DRAFT`
- 작성일: 2026-08-22 KST
- 대상: `core_enhanced` Rust game core
- 감사 기준선: `B-001`, HEAD `f318414fb0156a67e04c5479000c2a2683fc0c09`
- 근거 문서: `COMPLETION_AUDIT.md`, `FINDINGS.md`, `EVIDENCE_MATRIX.md`, `STATE_AUTHORITY_MAP.md`,
  `TRANSACTION_BOUNDARY_CATALOG.md`, `GAP_REGISTER.md`
- 목적: 확인된 결함을 다시 열거하는 것이 아니라, 현 설계를 보존·교체·재작성 관점에서 평가하고 후속 설계
  방향을 임시로 기록한다.
- 비고: 이 문서는 remediation 승인, 최종 ADR 또는 외부 protocol 변경 승인이 아니다.

## 1. 결론

`core_enhanced` 전체의 big-bang rewrite는 권고하지 않는다.

대신 아래 두 계층은 신규 설계로 단계적으로 교체하는 통제된 부분 재작성을 권고한다.

1. `world`의 상태 표현, command execution, transaction orchestration
2. `behavior.rs`에 집중된 외부 protocol, DTO, result, error projection 경계

아래 자산은 원칙적으로 보존한다.

- `BattleCore`의 전투 규칙과 event-driven simulation
- seeded RNG, deterministic identity와 event ordering 구조
- 공식 live-data parse/validate/compile/publication 경로
- checkpoint/restore와 adjacent durable persistence의 기존 기반
- 현재 gameplay behavior를 포착한 1,131개 baseline test와 characterization contract

따라서 권고안은 게임 규칙 전체를 다시 구현하는 전면 재작성이 아니라, 게임 규칙을 둘러싼 application shell을
교체하는 architectural migration이다.

## 2. 현재 설계 평가

| 영역 | 현재 평가 | 권고 |
| --- | --- | --- |
| domain/gameplay 규칙 | 상세하고 test evidence가 강함 | 보존 |
| battle simulation | typed command/event 기반이 있으나 authority가 과도하게 공개됨 | 알고리즘 보존, 경계 캡슐화 |
| root state 표현 | phase와 병렬 optional authority가 잘못된 조합을 표현할 수 있음 | 재설계 |
| command atomicity | staged pattern은 있으나 handler별 선택 사항임 | root-enforced transaction kernel 도입 |
| action routing | descriptor와 exhaustive match가 강함 | transaction metadata의 강제 지점으로 활용 |
| official data publication | strict parse/validate/compile/private validated path가 강함 | 보존하고 published type 강화 |
| alternate data construction | fixture와 production type/visibility가 혼합됨 | validated type-state와 fixture 격리 |
| checkpoint/persistence | strict V4 validation, rollback, adjacent atomic store 기반이 강함 | 보존하고 compatibility proof 보강 |
| external protocol | domain serde 및 대형 `behavior.rs`에 결합됨 | versioned contract 계층으로 분리 |
| retained event history | live delta, archive, debug, test 책임이 한 collection에 집중됨 | storage와 projection 책임 분리 |
| module/API 경계 | 29개 top-level module이 모두 public이고 global types가 넓게 공유됨 | facade 축소 후 물리적 crate 분리 검토 |

## 3. 핵심 설계 문제

### 3.1 Phase와 authority가 병렬 상태로 표현됨

현재 `GameCoreState`는 `GameState`와 다음 authority를 병렬 field로 보유한다.

- `run`
- `node_session`
- `active_node_content`
- `active_battle`
- `pending_game_mode`
- `visual_novel`
- `run_checkpoint`
- run-scoped roster/inventory/economy/UUID state

이 구조는 타입 수준에서 다음과 같은 잘못된 조합을 배제하지 못한다.

```text
GameState::InBattle + active_battle = None
GameState::ViewingMap + active_battle = Some(...)
GameState::InShop + node_session = None
GameState::NotStarted + run = Some(...)
```

현재 runtime action gate와 handler precondition이 이 조합을 방어하지만, `transition_to` 자체는 transition graph나
관련 authority를 검증하지 않고 `game_state`만 대입한다. 이 때문에 각 handler가 전이·authority·projection 순서를
개별적으로 맞춰야 하며 누락 가능성이 구조적으로 남는다.

### 3.2 Transaction protocol이 root invariant가 아님

현재 다음 안전한 패턴은 이미 존재한다.

- non-battle `with_state_transaction`
- staged battle advance
- staged tactical world mutation
- staged shop/economy grant

그러나 모든 mutation command가 반드시 이 패턴을 통과하도록 root에서 강제하지 않는다. RUNTIME-AUDIT-005와
RUNTIME-AUDIT-006은 transaction machinery의 부재가 아니라 일관되지 않은 채택으로 발생했다.

### 3.3 Mutation success와 transport cursor/projection이 결합됨

일부 battle handler는 authority mutation과 revision commit 후 `battle_event_after_seq`를 검증하고 DTO를 만든다.
따라서 presentation cursor error가 이미 성공한 domain mutation을 `Err`로 보이게 할 수 있다.

Mutation concurrency authority와 presentation synchronization은 별도 책임이어야 한다.

- mutation: expected aggregate/battle revision
- response: committed revision과 이 command가 만든 event sequence range
- reconnect/delta: 별도 query 또는 resync protocol

### 3.4 Domain, application, wire contract가 한 module에 집중됨

`behavior.rs`는 command, action descriptor, capability DTO, snapshot DTO, battle DTO, behavior result, error,
projection contract와 관련 test를 함께 소유한다. Exhaustive compiler coverage라는 장점이 있으나 다음 변경을
동시에 증폭시킨다.

- domain field 변경
- Rust enum/variant 변경
- server payload 변경
- Unity/client contract 변경
- error/result characterization 변경

### 3.5 Battle retained history가 여러 책임을 공유함

현재 `BattleEventLog`는 다음 역할을 동시에 수행한다.

- live delta transport
- combat result archive
- debug trace/export
- test observation

이 때문에 append-only full history와 매-tick delta 조회의 비용/보존 정책이 서로 묶인다.

## 4. 목표 상태 모델

권장 개념 구조는 다음과 같다.

```rust
struct GameRuntime {
    catalog: Arc<ValidatedGameData>,
    profile: PlayerProfileAggregate,
    session: SessionState,
}

enum SessionState {
    Idle,
    Starting(StartingRunState),
    Running(RunAggregate),
    Finished(FinishedRunState),
}

struct RunAggregate {
    progression: RunProgression,
    economy: RunEconomy,
    roster: EmployeeRoster,
    inventory: RunInventory,
    phase: RunPhase,
    overlay: Option<VisualNovelSession>,
}

enum RunPhase {
    Map(MapPhase),
    Node(NodePhase),
    Shop(ShopPhase),
    Reward(RewardPhase),
    Battle(BattleSession),
    CombatResult(CombatResultPhase),
}
```

설계 원칙:

- phase-specific authority는 해당 enum variant 안에 둔다.
- `player_profile`처럼 transaction rollback 대상이 아닌 monotonic aggregate는 run state 밖에 둔다.
- cross-cutting VN은 명시적 overlay policy로 모델링한다.
- static data는 `ValidatedGameData`만 runtime constructor에 전달할 수 있게 한다.
- raw/fixture authoring data와 published runtime data를 같은 bare type으로 취급하지 않는다.

## 5. 목표 Command Transaction Kernel

권장 실행 순서:

```text
validate/preflight
    -> stage/decide
    -> finalize/project
    -> commit
    -> publish/push
```

필수 불변조건:

1. command가 `Err`를 반환하면 authoritative state는 command 전과 의미상 동일하다.
2. authoritative commit 후에는 command 성공 여부를 뒤집는 fallible domain/projection 작업이 없다.
3. commit은 staged authority의 infallible swap이다.
4. post-commit I/O failure는 retry/outbox 또는 명시적 host rollback policy로 처리한다.

기본 interface 예시:

```rust
fn decide(
    &self,
    command: Command,
    context: CommandContext,
) -> Result<PreparedTransition, CommandRejection>;

fn commit(&mut self, prepared: PreparedTransition) -> CommitReceipt;

struct PreparedTransition {
    next_state: StagedState,
    result: DomainResult,
    events: Vec<DomainEvent>,
}
```

`commit`은 `Result`를 반환하지 않는다. 모든 checked arithmetic, validation, cursor policy, typed result 구성과
내부 invariant 검사는 `decide/finalize`에서 완료한다.

### 5.1 기존 ActionDescriptor의 확장

현재 exhaustive action descriptor를 유지하면서 다음 metadata를 추가하는 방향을 권고한다.

```rust
enum TransactionScope {
    ReadOnly,
    Profile,
    Run,
    ActiveBattle,
    FullSession,
}

struct ActionDescriptor {
    route: ActionRoute,
    scope: ActionScope,
    transaction: TransactionScope,
    authority: AuthorityPolicy,
}
```

Root dispatcher는 descriptor에 따라 transaction을 자동 선택해야 한다. Handler가 transaction helper를 선택적으로
호출하게 두지 않는다.

### 5.2 Trait만으로는 충분하지 않음

Trait은 transaction 구현을 표준화할 수 있지만, handler가 live `GameCoreState`에 접근할 수 있으면 강제력이 없다.
다음 조합이 필요하다.

- root-enforced dispatcher
- private authoritative fields
- scope별 staged transaction view
- `Prepared`/`Projected` typestate
- projected state에서만 호출 가능한 infallible commit

## 6. Battle 설계 방향

### 보존할 부분

- deterministic event queue
- seeded RNG와 identity namespace
- typed `BattleLiveCommand`
- step/run/finalize semantics
- combat rules와 현재 characterization tests

### 교체할 경계

- `BattleCore`와 `RuntimeUnit` authority field private화
- test event injection의 fixture/cfg 격리
- battle leaf의 transport-owned `GameError` 의존 제거
- validated command/query API 제공
- finite coordinate/event type 또는 enqueue validation

### Event/history 역할 분리

```text
BattleScheduler
  simulation 내부 event heap

BattleDelta
  이번 command/step에서 발생한 typed presentation events

ReconnectBuffer
  명시적 event/byte limit을 가진 ring 또는 segmented buffer

BattleArchive
  combat result/replay/debug용 선택적 전체 기록 sink
```

Live command/step은 이미 알고 있는 신규 `BattleDelta`를 직접 반환해야 한다. 최근 두 event를 얻기 위해 전체
history를 다시 순회하지 않는다.

## 7. Protocol과 Error 설계 방향

권장 logical module:

```text
domain/
  Command, DomainResult, DomainError, value objects

application/
  GameRuntime, CommandKernel, Transaction, CommitReceipt

contract/v1/
  CommandDtoV1, ResultDtoV1, ErrorDtoV1, SnapshotDtoV1

projection/
  domain/application result -> versioned contract mapper
```

Error도 최소한 다음 책임으로 분리한다.

- `CommandRejection`: 정상적인 gameplay 거부
- `InvariantViolation`: 내부 programming error
- `DataValidationError`: authoring/publication 오류
- `ProtocolError`: schema/version/deserialization 오류
- `InfrastructureError`: persistence/network/trace 오류

외부 error code/message는 versioned mapper가 소유한다. Battle/domain leaf는 wire error type을 import하지 않는다.

## 8. 전면 재작성 대안 비교

| 대안 | 장점 | 주요 위험 | 판정 |
| --- | --- | --- | --- |
| 결함만 국소 patch | 단기 변경량 최소 | transaction 누락과 responsibility concentration 재발 | 불충분 |
| 전체 big-bang rewrite | 목표 구조를 처음부터 적용 가능 | 숨은 gameplay rule 유실, determinism/persistence/protocol parity 증명 곤란, 장기 이중 개발 | 비권고 |
| application/protocol 계층의 단계적 재작성 | 기존 engine/data/test를 oracle로 유지하면서 root cause 제거 | migration discipline과 compatibility harness 필요 | 권고 |

전체 rewrite가 현재 정당화되지 않는 근거:

- 단일 `GameCore` authority owner가 존재한다.
- declared player/admin command routing이 exhaustive하다.
- 안전한 transaction/staging 패턴이 이미 여러 vertical path에서 동작한다.
- official data loader와 checkpoint validation이 강하다.
- battle determinism과 규칙에 상세한 test 자산이 있다.
- active findings는 세 root cause로 수렴하며 gameplay 알고리즘 전체의 부정확성을 증명하지 않는다.

## 9. 권장 이행 순서

### Phase 0. Contract freeze와 consumer inventory

- server와 Unity의 실제 command/result/error/snapshot 소비 schema를 목록화한다.
- canonical command sequence와 ordered result/event/final-state fixture를 만든다.
- current persistence continuation behavior를 동등성 oracle로 고정한다.
- 새로운 구조와 기존 구조의 dual-write는 금지하고 authority owner를 항상 하나로 유지한다.

### Phase 1. 현재 state 위에 transaction kernel 설치

- `ActionDescriptor`에 transaction/authority metadata 추가
- query와 mutation 실행 경로 분리
- RUNTIME-AUDIT-005의 여섯 battle command를 공통 staged transaction으로 이전
- RUNTIME-AUDIT-006 `StartNewGame`을 full-session transaction 또는 preflight plan으로 이전
- 모든 affected failure에 exact before/after equality와 retry test 추가

### Phase 2. State aggregate 재구성

- profile/monotonic authority와 run-transaction authority 분리
- `GameState + parallel Option`을 `SessionState/RunPhase`로 단계적 치환
- phase payload와 authority를 같은 enum variant로 이동
- existing public `GameCore` facade는 compatibility adapter로 유지

### Phase 3. Data publication type-state 강화

- runtime constructor가 `Arc<ValidatedGameData>`만 허용
- public `GameDataBuilder`는 `try_build` typed error를 반환
- unchecked fixture builder는 `cfg(test)` 또는 별도 fixture module로 격리

### Phase 4. Versioned protocol 추출

- domain command/result/error와 wire DTO 분리
- `contract/v1`을 current compatibility contract로 고정
- 필요 시 `contract/v2`를 병행하고 server/Unity consumer를 명시적으로 migration
- typed schema snapshot/golden fixture 추가

### Phase 5. Battle boundary와 history 개선

- BattleCore authority private화
- `BattleDelta` 직접 반환
- reconnect retention/work/byte budget 정의
- archive와 live transport buffer 분리
- constant-tail scaling guard 추가

### Phase 6. 물리적 crate 분리 검토

Logical dependency가 안정된 후에만 다음 workspace split을 검토한다.

```text
game_domain
game_battle
game_application
game_contract
game_data
```

구조 정리 전에 crate부터 나누는 것은 import/visibility churn만 키울 수 있으므로 권고하지 않는다.

## 10. Migration Acceptance Gates

후속 architectural migration의 최소 종료 조건 후보:

- 기존 broad test suite 전부 통과
- strict fmt/clippy gate 통과 또는 승인된 lint baseline
- 모든 mutating action이 transaction scope를 선언하고 root가 이를 강제
- 모든 typed rejection에서 authoritative state equality 또는 명시적 monotonic exception 증명
- future battle cursor가 mutation success를 뒤집지 않음
- canonical original-vs-restored multi-domain continuation equivalence 통과
- current v1 wire fixture compatibility 또는 승인된 version migration
- event delta cost가 retained total history가 아니라 returned delta에 비례
- production runtime constructor에서 unvalidated static data가 표현 불가능
- exported battle authority를 외부 caller가 직접 변경할 수 없음

## 11. 전면 재작성 재검토 조건

다음 중 하나가 추가로 입증될 경우에만 전체 rewrite를 다시 검토한다.

- 제품 요구사항이 현재 gameplay/data model의 대부분을 폐기한다.
- server/Unity 소비자 조사 결과 backward compatibility가 불필요하다.
- canonical parity harness를 만들 수 없거나 기존 semantics 자체를 신뢰할 수 없다.
- 실제 workload에서 현재 battle model 자체가 목표 budget을 충족할 수 없고 국소 알고리즘 교체로 해결되지 않는다.
- checkpoint continuation이 광범위하게 비결정적이어서 기존 state model을 migration source로 사용할 수 없다.

현재 감사 증거에는 위 조건이 성립한다는 근거가 없다.

## 12. 임시 최종 권고

```text
전체 codebase big-bang rewrite: 하지 않음
world/application shell: 통제된 단계적 재작성
behavior/protocol boundary: versioned 계층으로 병행 재작성
battle algorithms: 보존하고 authority/API 캡슐화
data validation/persistence/determinism: 보존하고 type/test 강화
```

가장 먼저 착수할 설계 작업은 `ActionDescriptor + TransactionScope + root-enforced staged execution`의 작은
prototype이다. 이 prototype이 여섯 S1 battle command와 `StartNewGame`을 동일한 protocol로 안전하게 수용하는지
증명한 뒤 state aggregate 재구성을 진행한다.
