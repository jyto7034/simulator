# Goal Prompt — Current Runtime Transaction Kernel Foundation

> 상태: `inactive-premature-draft`
>
> 이 문서는 두 감사의 통합이 끝나기 전에 잘못 선행 작성된 remediation 초안이다. 현재 다음 Goal이 아니며,
> `current_runtime_audit_consolidation` 결과와 사용자의 별도 승인 없이는 실행하지 않는다.

이 문서는 `PLAN.md`를 실제로 수행하기 위한 운영 명령이다. 이 문서를 읽는 것만으로 production 수정이나 Goal 실행이 시작되지는 않는다. 사용자가 이 Goal의 실행을 명시적으로 요청했을 때만 아래 절차를 시작한다.

## 1. Goal identity

- Goal name: `current_runtime_transaction_kernel_foundation`
- Goal directory: `docs/goals/current_runtime_transaction_kernel_foundation`
- 대상 저장소: `core_enhanced`
- 준비 상태: `inactive-premature-draft`
- 목적: 두 런타임 감사에서 확인된 transaction/application boundary 결함을 통합하고, 첫 remediation 단위를 원자적으로 구현·검증한다.

## 2. 운영 권위

1. `PLAN.md`가 수행 방식과 단계의 source of truth다.
2. `REMEDIATION_BLUEPRINT.md`가 설계 결정, 보존 범위, 후속 roadmap의 source of truth다.
3. `ACCEPTANCE_GATES.md`가 완료 조건의 source of truth다.
4. `COMMAND_TRANSACTION_INVENTORY.md`와 `MIGRATION_LEDGER.md`는 실행 중 사실에 맞게 갱신하는 living document다.
5. 두 기존 감사 디렉터리는 immutable evidence다. 사실 오류를 발견해도 원문을 조용히 고치지 말고 이 Goal의 기록에 정정한다.

이 Goal 디렉터리의 운영 문서는 서로 직접 의존하므로 실행 시작 시 전부 읽는다. 외부 contract, 정책, 감사 원문은 무조건 처음부터 끝까지 읽지 않는다. blueprint crosswalk와 현재 작업 단계가 지정하는 finding, 문서 구간, 코드 경로만 우선 읽고 부족할 때 범위를 확장한다.

## 3. 실행 명령

사용자가 Goal 실행을 지시하면 다음을 순서대로 수행하라.

1. Goal 관리 기능을 사용할 수 있으면 이 문서의 objective로 Goal을 생성한다. 이미 활성 Goal이 있으면 덮어쓰지 말고 상태를 확인한다.
2. `PLAN.md`의 상태를 `prepared`에서 `active`로 바꾸고 시작 시각을 기록한다.
3. 현재 commit, working tree, Rust toolchain, feature 조건을 `BASELINE.md`에 기록한다.
4. 사용자 변경과 감사 당시 이후 변경을 보존한다. dirty worktree를 정리하거나 되돌리지 않는다.
5. `COMMAND_TRANSACTION_INVENTORY.md`를 현재 `ActionKind::ALL` 및 descriptor 구현과 대조하여 전수 확정한다.
6. `PLAN.md`의 Phase 순서대로 한 remediation unit씩 구현한다.
7. 각 unit마다 oracle test를 먼저 확보하고, 구현하고, 좁은 검증을 실행하고, ledger를 갱신한다.
8. 넓은 검증은 baseline과 최종 gate에서 수행한다. 좁은 수정마다 전체 suite를 반복하지 않는다.
9. 모든 필수 gate가 통과한 뒤 `COMPLETION_AUDIT.md`를 작성하고 Goal을 완료 처리한다.

## 4. 읽기 전략

### 항상 먼저 읽을 것

- 이 디렉터리의 `PLAN.md`
- `REMEDIATION_BLUEPRINT.md`
- `COMMAND_TRANSACTION_INVENTORY.md`
- `ACCEPTANCE_GATES.md`
- `MIGRATION_LEDGER.md`

### 단계별로 읽을 것

- Structural audit의 `RUNTIME-AUDIT-005`, `RUNTIME-AUDIT-006`, `RC-02`
- Architecture review의 `ARCH-REVIEW-004`, `ARCH-REVIEW-005`, `RC-A`
- 현재 `src/game/behavior.rs`의 `ActionKind`, descriptor, route/scope
- 현재 `src/game/world.rs`의 execute/dispatch와 `StartNewGame`
- 현재 `src/game/world/combat.rs`의 대상 여섯 handler와 staged tactical helper
- 대상 동작을 고정하는 기존 테스트와 fixture

### 필요할 때만 읽을 것

- `game_rulebook.md`의 대상 command가 구현하는 정책 구간
- stale runtime/Unity contract의 compatibility 비교 구간
- 관련 없는 recent feature design note
- 후속 Goal 대상 finding 및 코드

읽은 사실은 byte size/hash 수집 자체를 목적으로 기록하지 않는다. 판단에 실제 사용한 경로, 구간, 역할, stale 여부, 결론만 `SOURCE_LOG.md`에 남긴다.

## 5. 구현 규율

### 5.1 고정 규율

- production behavior, wire shape, save compatibility를 의도 없이 바꾸지 않는다.
- 새 dependency를 추가하지 않는다. 필요하면 별도 사용자 승인을 받는다.
- 각 command에는 하나의 authority writer만 둔다.
- compatibility를 이유로 old/new path를 동시에 쓰지 않는다.
- fallible response/projection construction은 publish 전에 끝낸다.
- commit은 가능한 한 작은 infallible operation으로 만든다.
- 대상 범위 밖 구조 개선을 함께 하지 않는다.
- panic 제거, lint 청소, rename은 현재 transaction 수정에 직접 필요할 때만 한다.

### 5.2 허용되는 주요 production 경로

- `src/game/behavior.rs`
- `src/game/world.rs`
- `src/game/world/state.rs`
- `src/game/world/combat.rs`
- 위 경로의 직접 관련 test module 및 test support

다른 production 파일이 필요하면 수정 전에 ledger에 이유와 경계 영향을 기록한다. game server, Unity, canonical game data, rulebook은 이 Goal에서 수정하지 않는다.

### 5.3 실험 규율

가설 검증용 임시 코드는 `/tmp` 또는 명시된 Goal artifact로 제한한다. production에 임시 fallback, debug branch, 허위 abstraction을 남기지 않는다. 실험 결과가 설계를 바꾸면 `EXPERIMENT_NOTES.md`에 가설, 명령, 관찰, 결론을 기록한다.

## 6. 검증 규율

- baseline에서 full test, fmt check, strict clippy를 각각 한 번 실행해 기존 실패를 기록한다.
- 구현 중에는 가장 좁은 관련 테스트를 우선한다.
- 각 migration unit이 끝날 때 해당 command family의 conformance tests를 실행한다.
- 최종에는 `ACCEPTANCE_GATES.md`의 전체 검증을 실행한다.
- 기존 clippy 실패가 있으면 exact baseline과 비교하여 새 경고가 없는지 판정한다. 기존 실패를 Goal 외 범위에서 고치지 않는다.
- performance는 같은 runner, build profile, fixture, 반복 조건으로 전후 비교한다.

## 7. Git recovery protocol

이 Goal의 기술적 작업 단위는 다음 commit 경계로 설계한다.

1. oracle 및 command inventory
2. descriptor/transaction kernel foundation
3. live battle command migration
4. `StartNewGame` migration
5. conformance와 최종 문서

각 단위는 검증을 통과한 뒤에만 다음 단위로 넘어간다. 사용자가 commit 권한을 포함해 Goal 실행을 요청했다면 각 단위를 `add -> staged diff 검토 -> commit`한다. push는 별도의 명시적 요청이나 실행 지시에 push 권한이 포함된 경우에만 수행한다.

Goal 실행 요청만으로 기존 사용자 변경의 commit, reset, checkout, clean 권한을 얻은 것으로 간주하지 않는다.

## 8. Stop conditions

다음 중 하나면 추측으로 진행하지 말고 현재 단위를 안전한 상태에 두고 사용자에게 보고한다.

- 최신 rulebook과 보존해야 할 실제 behavior가 양립하지 않음
- save/wire compatibility 변경 없이는 대상 transaction을 만들 수 없음
- 대상 명령의 진짜 authority root가 감사 결과와 다름
- 단일 writer 전환에 game server/Unity 동시 변경이 필수임
- performance regression이 budget을 넘고 scope 안에서 해소되지 않음
- dirty worktree의 사용자 변경과 직접 충돌함
- 현재 Goal 밖의 data/protocol/battle-history 재설계가 필수 선행 조건임

멈출 때는 blocker, 증거, 가능한 선택지, 권장 선택을 기록한다. 단순히 구현이 어렵거나 오래 걸린다는 이유로 멈추지 않는다.

## 9. Completion command

완료 시 다음을 수행한다.

1. `ACCEPTANCE_GATES.md`를 실제 증거로 채운다.
2. `MIGRATION_LEDGER.md`의 모든 현재 Goal row를 `completed` 또는 승인된 `deferred`로 닫는다.
3. 변경 파일, test evidence, performance delta, 알려진 잔여 위험을 `COMPLETION_AUDIT.md`에 기록한다.
4. `PLAN.md`를 `completed`로 바꾼다.
5. Goal 관리 기능을 사용했다면 required work가 모두 끝난 경우에만 complete 처리한다.
6. 후속 Goal을 자동 시작하지 않는다.
