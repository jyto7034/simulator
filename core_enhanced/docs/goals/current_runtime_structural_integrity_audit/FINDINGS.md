# Findings

## 목적

감사 중 발견한 결함 후보, 확정 finding과 반증된 후보를 발견 즉시 기록한다.

이 문서는 마지막 취합 때 새로 작성하는 보고서가 아니다. 조사 중 지속적으로 갱신되는 evidence ledger다.

## 기록 원칙

- 결함 가능성을 처음 식별한 시점에 `Hypothesis`로 등록한다.
- 확정될 때까지 기록을 미루지 않는다.
- 코드 위치, 기준선과 관찰 사실을 먼저 기록하고 해석을 분리한다.
- 증거가 추가되면 확신도와 affected path를 갱신한다.
- 반증된 항목은 삭제하지 않고 `Rejected Candidates`로 이동한다.
- 기존 finding을 분리하거나 통합하면 원래 ID와 이유를 보존한다.
- 심각도는 코드 크기나 스타일이 아니라 영향, 도달 가능성, 범위, 복구성과 탐지 가능성으로 정한다.
- 과거 Goal이나 과거 감사의 finding ID를 재사용하지 않는다.
- 정책 차이는 구조 결함으로 등록하지 않는다.

## ID 규칙

- 확정 전 후보와 확정 finding 모두 `RUNTIME-AUDIT-###` 형식을 사용한다.
- ID는 한 번 부여하면 재사용하지 않는다.
- 반증되거나 통합된 ID도 history에서 유지한다.

## Finding 형식

```markdown
## RUNTIME-AUDIT-###. <제목>

- 최초 기록 시각:
- 마지막 갱신 시각:
- 기준선:
- criterion ID:
- 상태: Candidate | Active | Superseded | Rejected
- 심각도: S0 Blocker | S1 High | S2 Medium | S3 Low | 미판정
- 확신도: Hypothesis | Strong inference | Confirmed
- 영향받는 불변조건:
- 코드 위치:
- production 호출 경로:
- 최초 raw observation:
- 재현 명령 또는 test:
- 재현 입력:
- 현재 동작:
- 구조적으로 기대되는 동작:
- 상태 변경 전후 증거:
- 영향 범위:
- 근본 원인:
- 반대 증거 또는 불확실성:
- 중복·연관 finding:
- 권장 수정 방향:
- 필요한 회귀 테스트:
- 범위 밖 후속 후보 여부:
- 연결된 experiment:
```

동적 재현이 없는 구조 finding은 다음 정적 증거를 대신 기록한다.

- 관련 타입과 field
- 값을 쓰는 production 경로
- 값을 읽는 authority 경로
- invalid 또는 inconsistent state가 가능한 조건
- compiler, visibility 또는 validation이 이를 막지 못한다는 근거

## Active Findings

## RUNTIME-AUDIT-001. Test-only currency setter가 production public API로 노출됨

- 최초 기록 시각: 2026-08-21 23:13 KST
- 마지막 갱신 시각: 2026-08-21 23:58 KST
- 기준선: `B-001`
- criterion ID: C01, C02, C06, C09
- 상태: Active
- 심각도: S3 Low
- 확신도: Strong inference
- 영향받는 불변조건: authoritative resource mutation은 typed command/capability/commit/result 경계를 거친다.
- 코드 위치: `src/game/world.rs`의 `GameCore::set_enkephalin`; `src/game/resources/economy.rs`의
  `Enkephalin.amount`; `src/game/world/snapshot.rs`와 `src/game/world/shop.rs`의 authority readers
- production 호출 경로: external Rust caller → public `GameCore::set_enkephalin` → private
  `GameCoreState.enkephalin.amount`; 이후 snapshot/shop command가 변경 값을 authority로 읽음
- 최초 raw observation: method 주석은 “테스트 헬퍼”지만 `#[cfg(test)]`가 없고 public이다. Repository 내
  caller는 world test 5곳뿐이며 production domain caller는 없다.
- 재현 명령 또는 test: E-006의 static visibility/caller/read-path search
- 재현 입력: 임의 생성된 `GameCore`와 임의 `u32`
- 현재 동작: command ID, action gate, state precondition, event/result/provenance 없이 currency를 덮어쓸 수 있다.
- 구조적으로 기대되는 동작: test-only mutation은 test configuration/visibility에 격리되고 production
  authority 변경은 typed command 또는 명시적 admin boundary를 거친다.
- 상태 변경 전후 증거: `amount_before != amount`이면 호출 직후 `get_enkephalin()`과 run snapshot/eligibility가
  새 값을 반환하도록 writer/reader가 정적으로 직접 연결된다. 이 단순 assignment를 재확인하는 동적 probe는
  새 증거를 제공하지 않아 생략했다.
- 영향 범위: Rust API를 직접 소비하는 adapter/server가 이 method를 호출할 수 있는 boundary bypass
- 근본 원인: test fixture 편의를 production `impl GameCore`에 무조건 공개
- 반대 증거 또는 불확실성: repository production caller는 없고 current server의 명시적 admin mutation은 별도
  `AdminSetEnkephalin` 경로를 사용한다. 따라서 현재 server 동작 결함이 아니라 production crate authority
  encapsulation leak으로 S3를 부여했다; public visibility는 compiler가 차단하지 않는다.
- 중복·연관 finding: 없음
- 권장 수정 방향: 후속 remediation에서 `#[cfg(test)]` 또는 감사된 typed admin command 경계로 격리
- 필요한 회귀 테스트: production build에서 helper 비가시성 compile contract; admin 경로를 유지한다면
  authorization/typed result/projection test
- 범위 밖 후속 후보 여부: remediation은 현재 Goal 비범위
- 연결된 experiment: E-006

## RUNTIME-AUDIT-003. BattleCore authority fields와 harness event injection이 public mutable surface임

- 최초 기록 시각: 2026-08-21 23:20 KST
- 마지막 갱신 시각: 2026-08-21 23:58 KST
- 기준선: `B-001`
- criterion ID: C01, C02, C04, C06, C09
- 상태: Active
- 심각도: S2 Medium
- 확신도: Strong inference
- 영향받는 불변조건: battle unit/index/event/counter/recording state는 BattleCore command/event loop만 mutate한다.
- 코드 위치: `src/game/battle/core/mod.rs`의 `BattleCore` public fields와 `enqueue_event`;
  `src/game/battle/core/types.rs`의 public `RuntimeUnit` fields
- production 호출 경로: external Rust caller → public `BattleCore::new_from_scenario` → direct field mutation 또는
  public `enqueue_event` → public run/step/apply command
- 최초 raw observation: `units`, `battlefield`, `game_data`, `event_log`, event/cast/projectile/area/damage counters,
  seed와 recording stacks가 public이다. `RuntimeUnit`의 lifecycle/identity/stats/body/action/cooldown fields 대부분도
  public이다. `enqueue_event` 주석은 integration test setup harness를 주 용도로 밝히지만 production-visible이다.
- 재현 명령 또는 test: E-008/E-017 battle public method/field, reader/order and world integration inspection
- 현재 동작: caller가 lifecycle/HP/position/counter/event ordering을 leaf command와 cleanup/revision/event log 없이
  바꿀 타입 권한을 가진다.
- 구조적으로 기대되는 동작: authority mutation은 validated command/event API로 제한되고 debug/fixture injection은
  test boundary로 격리된다.
- 상태 변경 전후 증거: caller can assign `RuntimeUnit.stats.current_health/lifecycle/body` or public counters/log/seed
  independently; subsequent public queries/event processing read these values without a visibility gate or automatic
  reconciliation. `enqueue_event` directly changes heap contents without scenario validation.
- 영향 범위: standalone BattleCore consumer와 crate 내부 future adapter; active world battle은 private GameCore state로
  외부에서 직접 얻을 수 없다는 제한이 있음.
- 근본 원인: query/fixture 편의를 authority struct의 public mutability로 제공
- 반대 증거 또는 불확실성: current GameCore keeps its active `BattleCore` private and non-test world callers observed
  here read public fields rather than mutate them. This limits active-server reachability, so severity is S2 rather than
  S1; standalone `BattleCore` remains an exported production API and compiler cannot enforce its documented harness intent.
- 중복·연관 finding: RUNTIME-AUDIT-001과 public test convenience root cause 후보
- 권장 수정 방향: 후속 remediation에서 fields private, typed query/command 제공, harness는 cfg(test) 또는 fixture API
- 필요한 회귀 테스트: 외부 compile-fail visibility 및 command cleanup/event/counter invariants
- 범위 밖 후속 후보 여부: remediation은 현재 Goal 비범위
- 연결된 experiment: E-008, E-017, E-020

## RUNTIME-AUDIT-004. BattleEvent Ord의 finite-float 전제가 타입/validation으로 보장되지 않음

- 최초 기록 시각: 2026-08-21 23:20 KST
- 마지막 갱신 시각: 2026-08-21 23:58 KST
- 기준선: `B-001`
- criterion ID: C02, C04, C07, C10
- 상태: Active
- 심각도: S2 Medium
- 확신도: Strong inference
- 영향받는 불변조건: `BinaryHeap<BattleEvent>`의 `Eq`/`Ord` 계약과 same-time total ordering이 모든 reachable
  event payload에 대해 성립한다.
- 코드 위치: `src/game/battle/enums.rs`의 manual `Eq`/`Ord`; `src/game/battle/core/movement/types.rs`의
  public float-bearing `WorldVec2`
- production 호출 경로: external Rust caller → public `WorldVec2::{new, fields}` + public float-bearing
  `BattleEvent::{SkillProjectileImpact,SkillAreaTick}` → public `BattleCore::enqueue_event` → `BinaryHeap<BattleEvent>`;
  internal valid-data path is a positive counterexample
- 최초 raw observation: `BattleEvent`는 float payload 때문에 derived `PartialEq`만 있고, finite simulation을
  주석 전제로 manual `Eq`를 구현한다. `WorldVec2::new`와 fields는 public이며 finite check가 없다. Ord fallback은
  full Debug string을 사용한다.
- 재현 명령 또는 test: E-008/E-017 static Eq/Ord/type/queue path and passing finite-event permutation tests
- 현재 동작: two otherwise identical NaN-bearing events have derived `PartialEq == false` while manual `Eq` is asserted;
  identity comparison and identical Debug fallback can make `cmp == Equal`. Even self-equality violates the asserted Eq
  contract. No constructor/enqueue finite check blocks this public input.
- 구조적으로 기대되는 동작: non-finite value가 event construction 이전 validated type/boundary에서 불가능하거나
  comparator가 total float ordering을 사용한다.
- 상태 변경 전후 증거: enqueue adds the invalid event to the authoritative heap; heap comparison then observes a value
  for which `Eq` reflexivity and `cmp == Equal`/equality consistency do not hold. This is a static language-contract proof.
- 영향 범위: event heap ordering, determinism, potential missed/equal ordering
- 근본 원인: invalid-float exclusion을 주석에 의존할 가능성
- 반대 증거 또는 불확실성: official live inputs use validated integer/fixed-point data and internal tile/world
  conversions are finite; E-002's 720-permutation and movement probes pass. Current GameCore does not expose event injection.
  Finding scope is therefore the exported BattleCore boundary, not evidence of observed live-server nondeterminism.
- 중복·연관 finding: RUNTIME-AUDIT-003 public event injection이 reachability를 넓힘
- 권장 수정 방향: 도달성 확정 뒤 finite newtype/validation 또는 total order
- 필요한 회귀 테스트: non-finite input rejection 및 Ord law/property test
- 범위 밖 후속 후보 여부: remediation은 현재 Goal 비범위
- 연결된 experiment: E-008, E-017

## RUNTIME-AUDIT-005. Playback command가 future cursor 오류 전에 authority를 commit함

- 최초 기록 시각: 2026-08-21 23:25 KST
- 마지막 갱신 시각: 2026-08-22 00:03 KST
- 기준선: `B-001`
- criterion ID: C03, C06, C10
- 상태: Active
- 심각도: S1 High
- 확신도: Strong inference
- 영향받는 불변조건: `GameCore::execute*`가 `Err`를 반환하면 playback/focus/remainder/revision authority는
  command 전과 의미상 동일하다.
- 코드 위치: `src/game/world/combat.rs`의 `handle_pause_battle`, `handle_resume_battle`,
  `handle_set_battle_speed`, `handle_begin_skill_targeting`, `handle_cancel_skill_targeting`,
  `handle_activate_skill`; `src/game/world/state.rs`의 `commit_battle_projection`과
  `battle_update_dto_after`
- production 호출 경로: external caller → `execute_with_*battle_cursor` → battle dispatch → playback handler →
  mutate playback/focus/remainder → `commit_battle_projection` → `battle_update_dto_after` → cursor validation error
- 최초 raw observation: 세 handler는 projection commit 뒤 DTO builder를 호출하며, builder가 `after_seq >
  latest_event_log_seq`이면 `InvalidBattleResyncSeq`를 반환한다. 이 경로는 tactical staged transaction과 달리
  rollback wrapper 안에 있지 않다.
- 재현 명령 또는 test: E-009 complete static call/mutation/error path. E-002 suite에는 advance/tactical
  future-cursor atomicity tests가 있으나 affected six-command regression은 없음.
- 재현 입력: active live battle, 실제 latest event seq보다 큰 `battle_event_after_seq`, 유효한 pause/resume/speed
- 현재 동작: command result는 error이나 pause/resume/speed는 playback/focus/remainder/revision, targeting은
  focus/playback/remainder/revision, activation은 BattleCore cast/event/cooldown/resource/focus/playback/revision까지
  성공 path와 동일하게 변경될 수 있다.
- 구조적으로 기대되는 동작: cursor/revision/preconditions 모두 mutation 전 검증하거나 staged clone을 성공 DTO
  construction 후 commit한다.
- 상태 변경 전후 증거: before의 `latest_event_log_seq < after_seq`; handler가 live `active`를 mutate하고
  revision `checked_add`를 assign한 뒤 DTO builder가 cursor를 비교해 `Err`를 반환한다. 어떤 rollback wrapper도
  이 six-command dispatch를 감싸지 않는다. 따라서 after는 before와 다르다.
- 영향 범위: live battle transport playback commands; retry가 `AlreadyActive` 또는 달라진 state를 관측할 수 있음
- 근본 원인: domain mutation commit과 fallible transport projection이 같은 handler에서 mutation 뒤 결합
- 반대 증거 또는 불확실성: inspected adjacent actor normally supplies its internally tracked last-delivered seq rather
  than accepting a mutation cursor directly from the client, reducing ordinary-server incidence. The public transport-context
  API still accepts arbitrary cursor values and explicitly models future cursor as recoverable `GameError`; reconnect/state
  drift or another adapter can reach it. Advance/tactical staging is the correct counterexample and is not applied here.
- 중복·연관 finding: advance/tactical path는 staged/prevalidated counterexample
- 권장 수정 방향: 별도 remediation에서 cursor preflight 또는 staged playback transaction
- 필요한 회귀 테스트: 세 command 각각 future cursor `Err` 전후 complete active-session equality와 retry behavior
- 범위 밖 후속 후보 여부: remediation은 현재 Goal 비범위
- 연결된 experiment: E-009

## RUNTIME-AUDIT-006. StartNewGame가 fallible trigger activation 전에 world roots를 변경함

- 최초 기록 시각: 2026-08-21 23:30 KST
- 마지막 갱신 시각: 2026-08-22 00:03 KST
- 기준선: `B-001`
- criterion ID: C03, C07, C09, C11
- 상태: Active
- 심각도: S2 Medium
- 확신도: Strong inference
- 영향받는 불변조건: failed root command는 player/run/session/resource/VN/pending-mode authority를 보존한다.
- 코드 위치: `src/game/world.rs`의 `handle_start_new_game`; `src/game/world/tests/snapshots_and_start.rs`의
  `missing_run_start_visual_novel_trigger_fails_start_new_game`
- production 호출 경로: `GameCore::execute` → map/run dispatch → `handle_start_new_game` → initialize player,
  clear roots, set pending mode → `evaluate_visual_novel_triggers`/missing-trigger check → `Err`
- 최초 raw observation: trigger activation과 missing-trigger validation은 multiple live field assignments 뒤에 있다.
  Existing baseline test constructs data without RunStart trigger and confirms `InvalidStaticData` but does not compare state.
- 재현 명령 또는 test: E-010 static mutation/error path; E-002 ran existing
  `missing_run_start_visual_novel_trigger_fails_start_new_game` error reproduction
- 재현 입력: `GameCore` whose data has no matching RunStart VN trigger; valid StartNewGame behavior
- 현재 동작: error is returned after player/pending mode and cleared roots have been committed.
- 구조적으로 기대되는 동작: static prerequisite preflight before mutation or complete non-battle transaction rollback
- 상태 변경 전후 증거: before default has no player/pending mode; handler assigns player and pending mode and clears
  listed roots before missing-trigger `Err`. Existing test confirms this error is reachable through public builder/new;
  no wrapper restores before.
- 영향 범위: startup/retry on invalid or incompletely validated injected GameData
- 근본 원인: root setup handler performs destructive initialization before fallible data-driven activation and is not
  wrapped by `with_state_transaction`.
- 반대 증거 또는 불확실성: official embedded live loader validates current live bundle, reducing live-server
  reachability. Public builder/new and GameCore signature do not encode or enforce that exclusive path.
- 중복·연관 finding: RUNTIME-AUDIT-005 (inconsistent transaction adoption)
- 권장 수정 방향: remediation에서 complete preflight/staged state 또는 transaction wrapper; validated-data type
- 필요한 회귀 테스트: missing trigger error before/after full non-battle authority equality and retry
- 범위 밖 후속 후보 여부: remediation은 현재 Goal 비범위
- 연결된 experiment: E-010

## RUNTIME-AUDIT-007. Public GameData construction이 invalid input을 panic으로 처리함

- 최초 기록 시각: 2026-08-21 23:32 KST
- 마지막 갱신 시각: 2026-08-21 23:34 KST
- 기준선: `B-001`
- criterion ID: C07, C10, C13
- 상태: Active
- 심각도: S2 Medium
- 확신도: Confirmed
- 영향받는 불변조건: untrusted/authoring data errors are rejected at the publication boundary with contextual typed errors;
  published `GameDataBase` is validated.
- 코드 위치: `src/game/data/mod.rs`의 public `GameDataBuilder::{empty,with_*,build,build_arc}`와 public
  `GameDataBase::new`; `build_unique_index`/`insert_item_index`
- production 호출 경로: external Rust caller → public builder/parts → `GameDataBase::new` → `validate_indexes`/
  cross-reference validation → `panic!`/`expect`
- 최초 raw observation: live loader has a typed parse→validated→publication route, but a second public constructor route
  validates with panic/expect and returns bare `GameDataBase`, not `Result` or validated newtype.
- 재현 명령 또는 test: E-010 complete public constructor/panic path; E-002 baseline includes duplicate-index panic
  expectation tests in the same module
- 재현 입력: duplicate ID/UUID or invalid boss/shop/starter/reference contract in public parts
- 현재 동작: recoverable data authoring/injection error can unwind/terminate rather than return contextual `LiveRonError`.
- 구조적으로 기대되는 동작: one publication boundary returning typed source/object/field error; fixture-only unchecked path
  explicitly non-production and scoped.
- 상태 변경 전후 증거: no GameCore state exists yet; failure is process/control-flow boundary rather than partial commit.
- 영향 범위: any direct user of public builder/new; official live loader is a counterexample with typed errors.
- 근본 원인: validated and fixture construction share the same published type without visibility/type-state separation.
- 반대 증거 또는 불확실성: current adjacent server startup does use the official typed loader and no inspected
  production server caller uses the builder, reducing current live incidence. Public builder/new and bare GameCore input
  remain production-visible; E-002 dynamically executes seven `#[should_panic]` invalid-construction contracts.
- 중복·연관 finding: RUNTIME-AUDIT-006, likely common unvalidated/public fixture boundary root cause
- 권장 수정 방향: remediation에서 fixture builder cfg/test or explicit `try_build`, private unchecked constructor,
  validated catalog type
- 필요한 회귀 테스트: malformed public input returns typed error without panic; live and fixture publication parity
- 범위 밖 후속 후보 여부: remediation은 현재 Goal 비범위
- 연결된 experiment: E-010

## RUNTIME-AUDIT-008. Result/error wire schema가 Rust/domain representation과 직접 결합됨

- 최초 기록 시각: 2026-08-21 23:47 KST
- 마지막 갱신 시각: 2026-08-21 23:50 KST
- 기준선: `B-001`
- criterion ID: C05, C06, C13
- 상태: Active
- 심각도: S2 Medium
- 확신도: Strong inference
- 영향받는 불변조건: 외부 result/error contract는 internal domain layout과 Rust enum spelling changes로부터
  분리되고 명시적으로 versioned/stable해야 한다.
- 코드 위치: `src/game/behavior.rs`의 `PlayerBehavior`, `BehaviorResult`, `GameError`와 다수 DTO
- production 호출 경로: external transport → strict serde command → `GameCore::execute*` → central exhaustive
  `command_result_projection` → explicit server envelope/`Value`; error는 `GameError::code()` adapter를 거침
- 최초 raw observation: input enum은 strict tagged snake-case contract인 반면 result/error enum은 direct serde default
  representation이고, result fields가 목적 DTO와 internal/domain structs를 혼합한다. `GameError::code()`라는 별도
  stable identity mapping도 존재한다.
- 재현 명령 또는 test: E-015 static type/derive inspection
- 재현 입력: serialized behavior result/error or a domain-field/variant rename
- 현재 동작: outer result/error envelope는 중앙화되어 있지만 payload에 재사용된 domain representation 변경은 wire
  schema와 core/server compile fan-out을 직접 바꿀 수 있고 envelope 자체의 protocol version이 없다.
- 구조적으로 기대되는 동작: stable/versioned external result/error envelope와 internal-to-DTO mapper, 또는 current
  direct representation을 명시적으로 동결하는 compatibility contract/tests
- 상태 변경 전후 증거: authority mutation finding이 아니라 external protocol/change-amplification finding
- 영향 범위: active command-result payload/state snapshot/live battle DTO protocol; outer result/error identity는 adapter로
  보호됨
- 근본 원인: command execution, internal domain result, transport projection ownership이 대형 behavior module에 혼재
- 반대 증거 또는 불확실성: server는 실제로 central adapter/explicit error code를 사용하고 E-002는 47 result types의
  현재 projection을 모두 characterization한다. 따라서 whole-enum direct-wire 가설은 반증되었고 finding은 내부
  domain payload와 unversioned envelope 범위로 축소됐다. 오래된 Unity 소비 여부는 명시적 비범위다.
- 중복·연관 finding: RUNTIME-AUDIT-005의 fallible projection coupling과 same boundary ownership signal
- 권장 수정 방향: remediation에서 explicit result/error DTO envelope와 mapper, compatibility snapshot tests
- 필요한 회귀 테스트: representative result/error serialized schema snapshots and backward-compatibility fixtures
- 범위 밖 후속 후보 여부: Unity consumer는 비범위
- 연결된 experiment: E-015, E-016

## RUNTIME-AUDIT-009. Live battle delta가 매 tick 전체 append-only history를 다시 순회함

- 최초 기록 시각: 2026-08-21 23:54 KST
- 마지막 갱신 시각: 2026-08-22 00:01 KST
- 기준선: `B-001`
- criterion ID: C10, C11, C12, C13
- 상태: Active
- 심각도: S2 Medium
- 확신도: Confirmed
- 영향받는 불변조건: live delta extraction cost is proportional to the requested delta and retained battle authority has
  an explicit, deterministic growth/work bound.
- 코드 위치: `src/game/battle/event_log.rs::entries_after_seq`, `src/game/battle/recording.rs::record_event_log`,
  `src/game/world/state.rs::battle_update_dto_after`, `src/game/battle/core/sim.rs` execution loops; adjacent direct tick
  reference in `player_game_actor/handlers.rs`
- production 호출 경로: server fixed live tick → `advance_active_battle_for_server_tick_after_seq(last_delivered)` →
  battle advance → `battle_update_dto_after` → `entries_after_seq` full scan + tail clone
- 최초 raw observation: log is append-only with no event/byte cap, and delta lookup filters the whole Vec from entry zero
  rather than indexing by monotonic seq. Complete log is also serialized/compressed at result.
- 재현 명령 또는 test: E-018 complete static call/cost path; E-021 bounded actual-method scaling probe
- 재현 입력: any long-running battle with N retained log events and repeated live ticks requesting a small recent tail
- 현재 동작: each request is O(N + tail clone), yielding O(N^2) cumulative scan work as history grows; retained memory and
  result serialization are unbounded by a runtime contract.
- 측정 증거: with a constant two-entry tail, 1K/10K/100K retained entries cost 4.99/47.87/489.66 microseconds per call
  in E-021 (roughly 10× per 10× history). One 100K full clone copied at least 24MB of inline entry storage and took 13.76ms
  in the bounded synthetic fixture.
- 구조적으로 기대되는 동작: explicit event/time/byte/work budget and indexed/segmented/ring extraction whose recent-tail
  cost is O(log N + delta) or O(delta), with a defined replay/result retention policy.
- 상태 변경 전후 증거: after every recorded event, `entries.len()` grows by one and never shrinks during battle; next tick
  iterates all prior entries even when `last_seen_seq` is the previous tail.
- 영향 범위: every active live battle tick and combat-result log attachment; practical magnitude depends on battle length
  and event density.
- 근본 원인: one append-only collection simultaneously serves live delta transport, result history, debug trace and tests
  without separate storage/index/retention responsibilities.
- 반대 증거 또는 불확실성: server tracks last delivered seq and uses fixed tick duration; ordinary live encounters may be
  short. Boss runtime has a local microstep limit. None of these bounds retained history or full-scan cost.
- 중복·연관 finding: RUNTIME-AUDIT-008 (transport/domain responsibility concentration)
- 권장 수정 방향: separate authoritative cursor index from retained replay archive; define caps/work budgets and typed
  limit errors; avoid full-history re-scan per tick.
- 필요한 회귀 테스트: constant-tail extraction scaling test, max event/byte/work limit, deterministic limit behavior,
  full result/archive policy test
- 범위 밖 후속 후보 여부: production load testing/remediation은 별도 Goal
- 연결된 experiment: E-018, E-021; GAP-004

## Rejected Candidates

## RUNTIME-AUDIT-002. Shop sale currency 정산의 unchecked `u32` addition

- 최초 기록 시각: 2026-08-21 23:15 KST
- 마지막 갱신 시각: 2026-08-21 23:17 KST
- 기준선: `B-001`
- criterion ID: C02, C03, C10
- 상태: Rejected
- 심각도: 해당 없음
- 확신도: Strong inference (반증)
- 최초 raw observation: sale commit이 `amount + total_quote`를 unchecked로 계산한다.
- 반증: sole production path의 바로 앞 `sale_evaluation`이 동일 current amount와 반환할 동일 total quote를
  `checked_add`하고, 그 사이 authoritative currency mutation이 없다. Overflow 조건은 commit 도달 전에 typed
  disabled reason으로 거부된다.
- 남는 signal: check와 commit의 산술 표현이 중복되어 drift 가능성은 있으나 현재 결함/영향은 증명되지 않음.
- 연결된 experiment: E-006, E-007

## Root-Cause Consolidation Log

### RC-01. Fixture/debug construction boundary가 production type/visibility로 분리되지 않음

- linked findings: RUNTIME-AUDIT-001, 003, 004, 006, 007
- common mechanism: tests/fixtures and production callers share public mutation constructors or the same bare published
  type. Comments state intended use, but cfg/visibility/validated type-state does not enforce it.
- distinct consequences retained: currency bypass(001), battle authority bypass(003), invalid Eq/Ord input(004), failed
  start partial mutation(006), and panic publication(007) are not duplicate symptoms and remain separate findings.
- compiler coverage: none for intended caller class; all APIs type-check in production builds.

### RC-02. Transaction/preflight/projection ordering is adopted per handler rather than enforced at the root

- linked findings: RUNTIME-AUDIT-005, 006; RUNTIME-AUDIT-008 is a secondary boundary contributor.
- common mechanism: fallible validation/projection can remain after direct authority assignment. Shop, node completion,
  battle advance and tactical clone/commit paths prove reusable safe patterns exist, but root execution does not enforce one
  transaction protocol for all mutating commands.
- priority: RUNTIME-AUDIT-005 is the sole S1 and first remediation target; RUNTIME-AUDIT-006 is narrower and depends on
  alternate invalid data reachability.

### RC-03. Transport projection and retained history reuse domain-owned representations for multiple responsibilities

- linked findings: RUNTIME-AUDIT-008, 009; RUNTIME-AUDIT-005 shares projection placement but remains primarily RC-02.
- common mechanism: `behavior.rs` centralizes command/domain/protocol shapes while `BattleEventLog` serves live delta,
  result archive, debug export and tests. Centralization gives exhaustive matches but couples change and resource costs.
- distinct consequences retained: wire evolution/change amplification(008) and measured per-tick/history growth(009).

### Consolidation result

- 8 Active findings remain; none was superseded solely for sharing a root cause.
- 1 Rejected candidate remains as historical evidence (RUNTIME-AUDIT-002).
- no Candidate/Hypothesis remains at audit close.
