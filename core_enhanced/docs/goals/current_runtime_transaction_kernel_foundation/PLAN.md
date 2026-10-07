# Plan — Current Runtime Transaction Kernel Foundation

- 상태: `inactive-premature-draft`
- 작성일: 2026-08-22
- 실행 시작: 미정
- 실행 종료: 미정
- 상위 결정: `REMEDIATION_BLUEPRINT.md`
- 완료 기준: `ACCEPTANCE_GATES.md`

## 1. Objective

현재 runtime의 외부 동작과 보존 자산을 유지하면서, command mutation을 root에서 분류·강제하는 transaction foundation을 만들고 감사에서 확인된 일곱 명령의 partial commit 결함을 제거한다.

대상 명령은 live battle command 여섯 개와 `StartNewGame`이다.

## 2. Problem statement

현재 `ActionDescriptor`는 route, scope, battle authority requirement를 나타내지만 mutation의 transaction boundary를 강제하지 않는다. 그 결과 일부 handler는 live authority를 먼저 변경한 뒤 future cursor 또는 trigger/projection 생성 단계에서 실패할 수 있다.

기존 staged tactical/economy 경로는 유효한 seam이지만 convention에 의존한다. 현재 Goal은 convention을 재사용하되, 대상 mutation이 공통 root policy를 우회하지 못하게 만든다.

## 3. Success definition

- `ActionKind::ALL` 전부에 현재 authority와 target transaction classification이 기록되어 있다.
- transaction classification이 documentation-only metadata가 아니라 execute root에서 강제된다.
- 대상 일곱 명령에서 거부된 실행은 권위 상태를 변경하지 않는다.
- 대상 명령 성공 시 기존 response, revision, event/projection, deterministic behavior가 유지된다.
- 실패 후 같은 요청을 재실행할 수 있다.
- old/new dual-write나 silent fallback이 없다.
- 기존 test/performance baseline보다 악화되지 않는다.

## 4. Scope

### In scope

- transaction inventory와 behavior oracle
- descriptor에 transaction semantics 연결
- root transaction runner 또는 동등한 enforcement port
- pause/resume/speed/focus/activation command migration
- `StartNewGame` full-session transaction migration
- 관련 conformance/regression/performance 검증
- Goal artifact 갱신

### Out of scope

- 새 gameplay feature
- 전체 RunPhase/type-state rewrite
- active battle history/archive redesign
- public BattleCore API 전체 축소
- `ValidatedGameData`
- protocol DTO/error versioning
- crate/module 대규모 재배치
- server/Unity migration
- 무관한 formatting/lint/dead-code cleanup

## 5. Fixed design constraints

1. `PlayerBehavior`와 외부 관찰 결과는 compatibility surface로 취급한다.
2. 모든 fallible decide/validate/project 작업은 authority publish 전이어야 한다.
3. publish는 infallible이거나, 실패가 가능한 경우 authority를 전혀 노출하지 않는 교체 연산이어야 한다.
4. monotonic telemetry가 실패 시 변할 수 있다면 항목과 이유를 명시한다. 암묵적 예외는 없다.
5. transaction wrapper가 모든 명령에 전체 `GameCore` clone을 강제해서는 안 된다. scope별 최소 authority를 사용한다.
6. 첫 구현은 추상화의 일반성보다 대상 명령의 올바른 단일 경로를 우선한다.
7. existing staged tactical/economy helper는 positive control로 사용하되 무조건 하나의 generic trait로 합치지 않는다.
8. trait 도입은 최소 두 개의 실제 구현이 동일한 불변식을 공유하고 root enforcement에 이득이 있을 때만 한다. 이름뿐인 `AtomicTransaction` trait는 완료 조건이 아니다.
9. 새 dependency는 사용하지 않는다.

## 6. Required artifacts

준비 단계에서 존재해야 하는 문서:

- `GOAL_PROMPT.md`
- `PLAN.md`
- `REMEDIATION_BLUEPRINT.md`
- `COMMAND_TRANSACTION_INVENTORY.md`
- `ACCEPTANCE_GATES.md`
- `MIGRATION_LEDGER.md`

실행 시 생성할 문서:

- `BASELINE.md`
- `SOURCE_LOG.md`
- 필요할 경우 `EXPERIMENT_NOTES.md`
- `COMPLETION_AUDIT.md`

## 7. Execution phases

### Phase 0 — Activate and baseline

목적: 감사 시점의 가정이 아니라 현재 checkout을 기준으로 작업한다.

작업:

- Goal 상태를 active로 전환
- commit/dirty worktree/toolchain/features 기록
- 현재 `ActionKind::ALL`, descriptor, execute route 재확인
- full test/fmt/clippy baseline 1회 기록
- 관련 command test와 performance fixture 식별

Exit criteria:

- `BASELINE.md`가 재현 가능한 명령과 결과를 포함함
- 사용자 변경과 Goal 변경의 경계가 명확함
- 감사 finding이 여전히 재현되거나 현재 코드상 존재함을 확인함

### Phase 1 — Freeze oracle and complete inventory

목적: 내부 수정 전에 성공/실패 관찰 동작을 고정한다.

작업:

- `COMMAND_TRANSACTION_INVENTORY.md`를 1 action 1 row 수준으로 확정
- 각 action의 current route, authority root, current mutation mechanism, target transaction class 기록
- 대상 일곱 명령의 기존 성공 behavior를 characterisation test로 고정
- future cursor 및 RunStart trigger failure를 재현하는 regression test 작성
- equality에서 제외되는 monotonic field가 있다면 명시

Exit criteria:

- inventory가 `ActionKind::ALL`과 exact match
- 대상 defect를 구현 변경 전 테스트가 재현함
- success oracle과 rejection equality comparator가 준비됨

Commit unit: `oracle and transaction inventory`

### Phase 2 — Root transaction classification and runner

목적: transaction을 handler convention이 아니라 root route의 의무로 만든다.

작업:

- descriptor 또는 동등한 exhaustive metadata에 transaction class 연결
- `ReadOnly`, battle mutation, full-session mutation 등 필요한 scope 표현
- execute root에서 metadata에 맞는 runner/port를 선택하도록 구현
- unclassified gameplay mutation이 compile/test gate를 통과하지 못하도록 함
- query와 host-owned route가 mutation runner에 잘못 들어가지 않도록 검증
- 기존 staged tactical path가 semantics를 유지하도록 adapter 또는 direct integration

Exit criteria:

- 모든 `ActionKind` descriptor가 exhaustive
- 대상 handler를 direct call하여 root guard를 우회하는 production path가 없음
- metadata만 추가하고 실질 dispatch는 그대로인 상태가 아님
- narrow descriptor/route tests 통과

Commit unit: `descriptor and transaction kernel foundation`

### Phase 3 — Live battle command migration

대상:

- `PauseBattle`
- `ResumeBattle`
- `SetBattleSpeed`
- `BeginSkillTargeting`
- `CancelSkillTargeting`
- `ActivateSkill`

작업:

- 각 명령을 staged active battle authority에서 실행
- cursor-dependent update/projection을 staged state에서 생성
- 성공이 확정된 뒤 live session을 한 번 publish
- failure equality, retry, success parity 검증
- revision/event/focus/activation state의 정확한 delta 검증

Exit criteria:

- 여섯 명령 모두 future cursor 실패가 live authority를 변경하지 않음
- 같은 initial state/input의 성공 결과가 oracle과 동일함
- 실패 직후 valid retry가 성공함
- 기존 tactical staged command 회귀 없음

Commit unit: `live battle command transaction migration`

### Phase 4 — StartNewGame migration

목적: multi-root 초기화와 RunStart trigger를 하나의 full-session transition으로 만든다.

작업:

- StartNewGame이 변경하는 root와 trigger/projection dependency를 전수 기록
- 새 session state를 staged construction
- RunStart trigger 및 response 생성 가능성을 publish 전에 검증
- 성공 시 session roots를 단일 전환으로 publish
- invalid/missing trigger의 failure equality와 retry 검증
- save/checkpoint/new-game success behavior 회귀 검증

Exit criteria:

- trigger/projection 실패 전에 live session mutation이 없음
- 성공 결과가 기존 oracle과 동일함
- failure 후 동일 또는 corrected request 재시도 가능
- partial new-game state를 관찰할 경로가 없음

Commit unit: `StartNewGame full-session transaction migration`

### Phase 5 — Conformance and non-regression

목적: 개별 fix가 공통 불변식과 전체 runtime을 깨지 않았음을 증명한다.

작업:

- 대상 일곱 명령 table-driven conformance 실행
- descriptor inventory exhaustiveness 실행
- battle determinism/checkpoint/save 관련 회귀 실행
- full test/fmt/clippy final 실행
- same-runner performance 비교
- code search로 old/new dual-write, direct bypass, TODO fallback 확인

Exit criteria:

- `ACCEPTANCE_GATES.md`의 G0–G9 판정 완료
- 필수 gate 전부 PASS
- baseline failure 외 새 실패 없음
- performance budget 충족

Commit unit: `transaction conformance and final audit`

### Phase 6 — Close

작업:

- `MIGRATION_LEDGER.md` 실제 상태와 일치
- `COMPLETION_AUDIT.md` 작성
- 남은 finding을 후속 Goal 후보로 기록하되 자동 시작하지 않음
- PLAN 상태를 completed로 전환
- 활성 Goal이 있다면 실제 완료 후에만 complete 처리

## 8. Verification strategy

### Narrow verification

- descriptor/inventory exhaustiveness tests
- command별 handler/route tests
- rejection authority equality
- retry after rejection
- success state delta and response parity

### Broad verification

- full library/workspace test according to current baseline command
- `cargo fmt --all -- --check`
- baseline과 동일 feature 조건의 strict clippy
- determinism/checkpoint/save 관련 suites
- 관련 performance fixture

### Equality policy

단순 response equality로 원자성을 판정하지 않는다. 대상 authority root의 before/after snapshot을 비교한다. 비교에서 제외되는 값은 monotonic telemetry처럼 정책적으로 승인된 항목만 허용하며, comparator와 문서에 필드명을 적는다.

## 9. Performance budget

- 같은 build profile과 fixture에서 대상 success path의 median 또는 p50이 baseline보다 10% 넘게 악화되면 조사한다.
- p95가 baseline보다 15% 넘게 악화되면 필수 gate 실패로 본다.
- 측정 noise가 큰 경우 충분한 반복과 양방향 순서로 다시 측정한다.
- 전체 `GameCore` clone을 모든 gameplay action에 추가하는 설계는 수치와 무관하게 거부한다.
- 대상 command가 cold/control-plane path라 절대 시간이 미미해도, clone 크기와 allocation 변화는 기록한다.

사용자가 다른 명시적 budget을 승인하면 그 값을 기록하고 이 절을 갱신한다.

## 10. Commit and rollback units

각 unit은 이전 unit이 green인 상태에서만 시작한다. commit이 승인되지 않은 실행에서는 동일 경계를 commit-ready checkpoint로 유지한다.

rollback은 old/new dual path를 runtime에 남기는 방식으로 하지 않는다. 문제가 생기면 해당 unit의 변경 전체를 되돌릴 수 있어야 하며, 다음 unit은 시작하지 않는다.

## 11. Completion prohibition

다음 상태에서는 완료로 표시하지 않는다.

- trait/type은 생겼지만 대상 handler가 우회 가능함
- regression test가 response만 보고 authority equality를 검증하지 않음
- 여섯 command 중 일부만 migration됨
- `StartNewGame` 실패 경로가 staged가 아님
- inventory에 unclassified 또는 누락 action이 남음
- broad test를 실행하지 않았거나 결과가 기록되지 않음
- performance comparison이 없음
- ledger와 실제 코드가 다름
