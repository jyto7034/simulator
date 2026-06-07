# Rapier Backend Quality Refactor Goal

이 goal은 `Movement Backend Policy Refactor` 이후 남은 Rapier backend의 구조적 부채를 정리하는 작업이다.

이전 goal은 gameplay source of truth를 Rapier에서 core runtime policy로 옮기는 데 집중했다. 이 goal은 그 전제를 유지한 채, Rapier backend가 장기적으로 안전한 ground static obstacle correction helper로 남을 수 있도록 내부 책임, 테스트 표면, 진단 가능성, Direct backend와의 의미 동등성을 정리한다.

## Goal Mode Working Method

이 문서는 `C:\Users\blast\Desktop\using-codex-goals-effectively-ko.md`의 goal 운용 원칙을 따른다.

작업자는 goal 시작 시 아래 작업 기억장치를 만들고 계속 갱신한다.

```text
docs/goals/rapier_backend_quality_refactor/PLAN.md
docs/goals/rapier_backend_quality_refactor/EXPERIMENTS.md
docs/goals/rapier_backend_quality_refactor/EXPERIMENT_NOTES.md
```

## Engineering Principles

- 코드 수정은 장기적인 방향으로 한다.
- Rapier/backend physics는 blocking, targetability, deploy occupancy, route progress, airborne terrain immunity의 source of truth가 아니다.
- Rapier는 ground movement static obstacle correction helper로만 남긴다.
- Direct backend와 Rapier backend가 서로 다른 gameplay semantics를 만들지 않게 한다.
- Rapier를 유지할 이유가 약해졌다고 판단되면 바로 제거하지 않고, 근거와 blast radius를 기록한 뒤 사용자와 먼저 의논한다.
- 문서를 무조건 신뢰하지 않고 `rapier_backend.rs`, `engine.rs`, `steering.rs`, `planner.rs`, `blocking.rs`, 관련 tests의 실제 흐름을 먼저 읽는다.
- 레거시 helper/test가 새 정책을 흐리면 과감히 제거한다. 단, 제거 이유와 대체 테스트를 기록한다.
- 사용자와 의논하여 정해야 할 정책이 발견되면 goal을 종료한다.

## Goal Execution Contract

- source of truth 우선순위는 실제 runtime code, live RON/data, Unity-facing snapshot/command 계약, 최신 정책 문서 순서로 본다.
- 문서와 코드가 충돌하면 코드를 먼저 읽고, 최신 정책과 코드의 차이를 `EXPERIMENT_NOTES.md`에 기록한 뒤 수정 방향을 정한다.
- compatibility layer, fallback path, dual behavior는 기본적으로 만들지 않는다. 정말 필요하면 이유, 제거 예정 조건, 테스트 범위를 `PLAN.md`에 기록하고 사용자 확인을 받는다.
- 테스트는 내부 구현 모양보다 user-visible movement behavior, route semantics, backend 동등성, 디버깅 가능한 stop/correction 결과를 고정한다.
- 새 정책과 충돌하는 테스트는 기대값만 바꾸지 않는다. 그 테스트가 무엇을 보호하던 것인지 확인한 뒤 삭제하거나 최신 정책 테스트로 교체한다.
- ignored test로 레거시를 보존하지 않는다.
- goal 범위를 벗어난 개선안은 바로 구현하지 말고 `EXPERIMENT_NOTES.md`에 후속 후보로 기록한다. 현재 goal 완료에 필수이거나 blast radius를 줄이는 경우에만 적용한다.
- Unity-facing DTO shape, live RON schema, 저장 데이터 migration, UX 의미 변화, 밸런스 기준, 기존 콘텐츠 삭제/대체처럼 사용자 결정이 필요한 정책이 발견되면 임의로 결정하지 않고 goal을 종료하고 질문 목록을 보고한다.
- 작은 변경 단위마다 빠른 focused test 또는 `cargo check`를 먼저 돌리고, 마지막에 넓은 테스트를 돌린다.
- 테스트 실패는 `EXPERIMENTS.md`에 실패 원인, 수정 내용, 재검증 결과를 기록한다.
- 완료 시 변경 요약, 제거한 레거시, 새로 고정한 계약, 갱신한 테스트, 남은 위험, 실행한 검증 명령을 보고한다.

## Canonical Unity Docs

Unity-facing 계약/구현 문서의 canonical 위치는 `F:\unity projects\ark\docs`다. Linux/WSL 경로로는 `/mnt/f/unity projects/ark/docs`다.

이 goal은 원칙적으로 core movement backend 내부 작업이다. 다만 movement stop/correction diagnostics가 Unity-facing snapshot 또는 timeline event로 노출되어야 한다고 판단되면 아래 외부 canonical 문서를 기준으로 확인하고 사용자와 먼저 의논한다.

- `F:\unity projects\ark\docs\unity_core_contract.md`
- `F:\unity projects\ark\docs\unity_client_implementation_goal.md`

이 저장소 안의 같은 이름 문서는 stale copy일 수 있다.

## Source Of Truth

우선 읽을 파일:

- `src/game/battle/core/movement/rapier_backend.rs`: Rapier continuous movement backend.
- `src/game/battle/core/movement/engine.rs`: backend trait, movement input/output, Direct backend.
- `src/game/battle/core/movement/steering.rs`: target approach, steering, obstacle response.
- `src/game/battle/core/movement/planner.rs`: route/attack goal 생성.
- `src/game/battle/core/movement/blocking.rs`: block_state와 route progress.
- `src/game/battle/core/movement/types.rs`: unit body, goals, world vectors.
- `src/game/battle/core/spatial.rs`: spatial indexing and range query assumptions.
- `src/game/battle/core/build.rs`: runtime movement policy 전달.
- `src/game/battle/core/types.rs`: `RuntimeUnit` movement-related fields.
- `docs/movement_backend_policy_refactor_goal.md`.
- `docs/unit_overlap_blocking_goal.md`.
- `docs/airborne_enemy_mobility_goal.md`.
- `docs/core_policy_decisions_2026_06.md`.

## Current Baseline

`Movement Backend Policy Refactor` 이후의 기준은 다음과 같다.

- `MovementUnitInput`에는 `MovementTerrainPolicy`가 있다.
- `Ground`는 static obstacle/void projection correction을 받는다.
- `Airborne`은 static obstacle/void projection correction을 받지 않고 board clamp만 받는다.
- runtime board bounds source of truth는 core clamp다.
- Rapier wall colliders는 runtime tick에서 board bounds source of truth로 쓰지 않는다.
- Rapier unit colliders는 movement/blocking source of truth가 아니다.
- block state, targetability, deploy occupancy, route progress는 core runtime policy가 결정한다.

## Objective

완료 후 다음 상태가 되어야 한다.

1. Rapier backend의 public/internal 책임이 ground static obstacle correction으로 명확히 제한된다.
2. Direct backend와 Rapier backend의 policy-visible behavior가 focused tests로 고정된다.
3. unit collider, board wall collider, static obstacle collider lifecycle이 디버깅 가능하고 누수 없이 관리된다.
4. movement correction 결과가 `Moved`, `Blocked`, `Adjusted`, `NoGoal`, `Dead`, `ActionLocked` 등으로 해석 가능한 형태를 유지하거나 개선한다.
5. stale helper, 우회 local avoidance, 중복 clamp/wall logic, 테스트만 만족시키는 legacy branch가 제거되거나 명시적으로 test-only로 격리된다.
6. 장애물에 막히는 route authoring 오류가 Rapier의 암묵 우회로 숨겨지지 않는다.
7. 공중 이동, 유닛 겹침, block_state 기반 저지 정책을 깨지 않는다.

## In Scope

- `rapier_backend.rs` 책임 분해와 이름 정리.
- Rapier collider lifecycle 정리: static obstacle, unit collider bookkeeping, board wall helper.
- Direct/Rapier backend 동등성 테스트 추가 또는 정리.
- ground static obstacle correction tests 보강.
- unit collider exclusion tests 보강.
- route 밖 local avoidance 또는 stale obstacle workaround 제거.
- board clamp와 Rapier wall collider의 역할 문서화/코드 정리.
- movement stop/correction reason이 디버깅 가능한지 확인하고 필요하면 개선.
- 불필요한 legacy tests 제거 또는 최신 정책 테스트로 교체.

## Out Of Scope

- 실제 `Airborne` enemy data/schema/combat behavior 구현. 이는 `airborne_enemy_mobility_goal.md`에서 처리한다.
- weapon `AirFirst`/`air_capable` 구현. 이는 `weapon_archetype_targeting_goal.md`에서 처리한다.
- pathfinding/route authoring tool 구현.
- Unity visual fan-out/offset 구현.
- Rapier 라이브러리 제거. 제거가 더 낫다고 판단되면 이유를 기록하고 사용자와 먼저 의논한다.
- gameplay policy 변경. backend 정리 중 정책 변경이 필요하면 goal을 종료하고 사용자와 의논한다.

## Policy Details

### Rapier Responsibility

Rapier may do:

- ground static obstacle correction.
- deterministic stop/slide behavior against authored static obstacles, if tests explicitly lock that behavior.
- low-level geometric query support for movement backend tests.

Rapier must not decide:

- which unit blocks which enemy.
- whether a unit can be targeted.
- whether a unit is airborne.
- whether an enemy reached protected objective.
- whether units can overlap.
- whether deploy placement is occupied.
- whether route progress advances.

### Board Bounds

- Runtime board bounds source of truth remains core clamp.
- Rapier wall collider helpers may exist only if they are test helpers or clearly documented low-level utilities.
- Direct and Rapier backend must not apply conflicting board rules.

### Static Obstacles

- Ground units respect authored static obstacles.
- Airborne units ignore static obstacles.
- If a ground route is blocked by authored obstacles, that should surface as blocked/adjusted movement, not as an untracked detour.

### Unit Colliders

- Unit colliders do not produce movement depenetration.
- Unit colliders do not implement block capacity.
- Unit colliders may only support backend bookkeeping if there is still a concrete reason after refactor.

## Implementation Plan

1. Read current Rapier/Direct movement flow and record actual responsibilities in `EXPERIMENT_NOTES.md`.
2. Add or update characterization tests for current desired semantics before refactoring.
3. Split Rapier backend internals into clear units: collider sync, static obstacle query, correction application, board clamp handoff.
4. Remove stale local avoidance/legacy fallback branches that can change route semantics without typed policy.
5. Make unit collider exclusion explicit in helper names and tests.
6. Keep runtime board bounds as core clamp and remove any duplicate runtime wall sync path if still present.
7. Ensure movement output exposes enough reason/correction data to debug blocked routes.
8. Run focused movement/blocking/airborne checks.
9. Record final decisions, removed legacy, and remaining risk in goal/master notes.

## Test Requirements

- Ground movement is corrected or stopped by static obstacles according to the selected ground obstacle policy.
- Airborne movement ignores static obstacles and void/walkability correction.
- Airborne movement still respects board clamp.
- Unit colliders are ignored for movement correction.
- Overlapping units are not depenetrated by Rapier.
- Blocked enemy behavior is controlled by block_state, not Rapier.
- Direct and Rapier backend do not diverge on policy-visible movement behavior.
- Route-blocked ground movement is visible through stop/correction output and tests.
- No stale local avoidance silently routes units around authored DefenseRoute obstacles.

검증 후보:

```text
cargo test -p game_core movement -- --nocapture
cargo test -p game_core blocking -- --nocapture
cargo test -p game_core airborne -- --nocapture
cargo test -p game_core fixed_defense -- --nocapture
cargo check -p game_core
```

## Completion Conditions

- Rapier backend responsibility is explicitly limited to ground static obstacle correction helper behavior.
- Direct/Rapier backend policy-visible semantics are locked by tests.
- Unit collider exclusion remains explicit and tested.
- Runtime board bounds remain core clamp's responsibility.
- Stale local avoidance/legacy fallback behavior that hides route errors is removed or explicitly justified.
- Movement diagnostics are sufficient to debug route authoring and obstacle issues.
- Goal work memory is updated with plan, experiments, findings, removed legacy, and remaining risk.
- 사용자와 의논하여 정해야 할 정책이 있을 경우 goal을 종료한다.

## Stop Conditions

- Rapier 제거가 더 적절해 보이지만 external impact가 커서 사용자 결정이 필요하다.
- static obstacle response should be `Stop`인지 `Slide`인지 gameplay 의미를 코드만으로 결정하기 어렵다.
- movement diagnostics를 Unity-facing DTO로 노출해야 할지 정책 결정이 필요하다.
- backend 정리 중 live route/data를 대량 수정해야 하는 상황이 발견된다.
- 사용자와 의논하여 정해야 할 정책이 있을 경우 goal을 종료한다.
