# Movement Backend Policy Refactor Goal

이 goal은 Rapier backend를 포함한 continuous movement backend의 책임을 재정의하고, 지상/공중 이동 정책을 유닛별로 명시할 수 있도록 movement 입력/해결 구조를 정리하는 작업이다.

이 goal은 `Airborne Enemy Mobility` 구현 전에 완료되어야 한다. 공중 적은 지형지물과 static obstacle을 무시해야 하므로, 현재처럼 모든 유닛이 같은 Rapier obstacle correction을 타는 구조를 먼저 분리한다.

## Goal Mode Working Method

이 문서는 `C:\Users\blast\Desktop\using-codex-goals-effectively-ko.md`의 goal 운용 원칙을 따른다.

작업자는 goal 시작 시 아래 작업 기억장치를 만들고 계속 갱신한다.

```text
docs/goals/movement_backend_policy_refactor/PLAN.md
docs/goals/movement_backend_policy_refactor/EXPERIMENTS.md
docs/goals/movement_backend_policy_refactor/EXPERIMENT_NOTES.md
```

## Engineering Principles

- 코드 수정은 장기적인 방향으로 한다.
- Rapier/backend physics is not the source of truth for blocking, targetability, or airborne terrain immunity. It is only a ground movement collision helper for terrain/static obstacle correction.
- 이동 정책은 backend 내부 암묵 동작이 아니라 `MovementUnitInput` 또는 그에 준하는 typed policy로 표현한다.
- 유닛 간 겹침 허용과 block_state 기반 저지 정책을 Rapier collision으로 되돌리지 않는다.
- 공중 이동 정책을 Rapier 예외 패치로 숨기지 않는다. 지상/공중 movement policy를 명시적으로 분기한다.
- 문서를 무조건 신뢰하지 않고 `rapier_backend.rs`, `engine.rs`, `steering.rs`, `planner.rs`, `blocking.rs`의 실제 흐름을 먼저 읽는다.
- 사용자와 의논하여 정해야 할 정책이 발견되면 goal을 종료한다.

## Goal Execution Contract

- source of truth 우선순위는 실제 runtime code, live RON/data, Unity-facing snapshot/command 계약, 최신 정책 문서 순서로 본다.
- 문서와 코드가 충돌하면 코드를 먼저 읽고, 최신 정책과 코드의 차이를 `EXPERIMENT_NOTES.md`에 기록한 뒤 수정 방향을 정한다.
- compatibility layer, fallback path, dual schema는 기본적으로 만들지 않는다. 정말 필요하면 이유, 제거 예정 조건, 테스트 범위를 `PLAN.md`에 기록하고 사용자 확인을 받는다.
- 테스트는 내부 구현 모양보다 사용자-visible behavior, Unity-facing DTO, data validation, live RON loading, 실제 gameplay flow를 고정한다.
- 새 정책과 충돌하는 테스트는 기대값만 바꾸지 않는다. 그 테스트가 무엇을 보호하던 것인지 확인한 뒤 삭제하거나 최신 정책 테스트로 교체한다.
- ignored test로 레거시를 보존하지 않는다.
- goal 범위를 벗어난 개선안은 바로 구현하지 말고 `EXPERIMENT_NOTES.md`에 후속 후보로 기록한다. 현재 goal 완료에 필수이거나 blast radius를 줄이는 경우에만 적용한다.
- Unity-facing DTO shape, live RON schema, 저장 데이터 migration, UX 의미 변화, 밸런스 기준, 기존 콘텐츠 삭제/대체, 실패/보상/소비 시점 변화처럼 사용자 결정이 필요한 정책이 발견되면 임의로 결정하지 않고 goal을 종료하고 질문 목록을 보고한다.
- 작은 변경 단위마다 빠른 focused test 또는 `cargo check`를 먼저 돌리고, 마지막에 넓은 테스트를 돌린다.
- 테스트 실패는 `EXPERIMENTS.md`에 실패 원인, 수정 내용, 재검증 결과를 기록한다.
- 완료 시 변경 요약, 제거한 레거시, 새로 고정한 계약, 갱신한 테스트, 남은 위험, 실행한 검증 명령을 보고한다.

## Canonical Unity Docs

Unity-facing 계약/구현 문서의 canonical 위치는 `F:\unity projects\ark\docs`다. Linux/WSL 경로로는 `/mnt/f/unity projects/ark/docs`다.

아래 파일은 외부 canonical 문서를 기준으로 확인하고 갱신한다.

- `F:\unity projects\ark\docs\unity_core_contract.md`
- `F:\unity projects\ark\docs\unity_client_implementation_goal.md`

이 저장소 안의 같은 이름 문서는 stale copy일 수 있다.

## Source Of Truth

우선 읽을 파일:

- `src/game/battle/core/movement/rapier_backend.rs`: Rapier continuous movement backend.
- `src/game/battle/core/movement/engine.rs`: `MovementUnitInput`, `MovementTickInput`, `DirectContinuousMovement`, backend wrapper.
- `src/game/battle/core/movement/steering.rs`: target approach, steering, avoidance.
- `src/game/battle/core/movement/planner.rs`: route/attack goal 생성.
- `src/game/battle/core/movement/blocking.rs`: block_state와 route progress.
- `src/game/battle/core/movement/types.rs`: unit body, goals, world vectors.
- `src/game/battle/core/build.rs`: runtime unit 생성과 movement profile 전달.
- `src/game/battle/core/types.rs`: `RuntimeUnit` movement-related fields.
- `docs/unit_overlap_blocking_goal.md`.
- `docs/airborne_enemy_mobility_goal.md`.
- `docs/core_policy_decisions_2026_06.md`.

## Current Findings

현재 Rapier backend는 아래 성질을 가진다.

- unit collider는 생성하지만 corrected translation query에서 unit colliders를 제외한다.
- 유닛 간 물리 충돌은 이미 길막/저지 source of truth가 아니다.
- static obstacle과 board bounds는 모든 유닛에 동일하게 적용된다.
- `MovementUnitInput`에는 지상/공중/지형 충돌 정책을 표현하는 필드가 없다.
- Rapier backend와 Direct backend가 dead/can_move/reached_goal/goal_target/obstacle 처리 루프를 각각 가진다.
- `corrected_translation_with_local_avoidance`는 obstacle에 막혔을 때 route 밖 우회 후보를 시도한다.

## Objective

완료 후 다음이 가능해야 한다.

1. movement input은 유닛별 movement/collision policy를 명시한다.
2. `Ground` 유닛은 terrain/static obstacle correction을 받는다.
3. `Airborne` 유닛은 terrain/static obstacle/walkable correction을 받지 않고, route polyline movement와 board bounds만 따른다.
4. 유닛 간 collision은 여전히 movement/blocking source of truth가 아니다.
5. Rapier는 지상 이동의 static obstacle correction helper로 제한된다.
6. blocking, targetability, airborne terrain immunity는 Rapier가 아니라 core runtime policy가 결정한다.
7. Direct backend와 Rapier backend가 서로 다른 gameplay semantics를 만들지 않는다.
8. local obstacle avoidance가 route source of truth를 깨지 않거나, 명시 정책으로 비활성화/제한된다.
9. blocked/collision-adjusted/no-goal 상태가 디버깅 가능한 movement output 또는 test behavior로 드러난다.

## In Scope

- `MovementUnitInput`에 movement policy 추가.
- `MovementCollisionPolicy`, `TerrainCollisionPolicy`, `MovementLayer`, 또는 동등한 typed enum 도입.
- Rapier backend에서 unit collider 제외 정책을 명시 test로 유지.
- Rapier static obstacle correction을 `Ground` policy에만 적용.
- `Airborne` policy는 static obstacle correction을 건너뛰고 board clamp만 적용.
- Direct/Rapier backend의 공통 movement flow 정리 또는 의미 동등성 테스트 추가.
- local avoidance의 역할 재검토. 필요하면 default path에서 제거하거나 obstacle response policy로 제한.
- board bounds source of truth 정리. clamp와 Rapier wall collider 중 어느 것이 authoritative인지 명확히 한다.
- movement backend 관련 docs/goal notes 갱신.

## Out Of Scope

- 실제 `Airborne` enemy combat behavior 전체 구현. 이는 `airborne_enemy_mobility_goal.md`에서 처리한다.
- weapon `AirFirst`/`air_capable` 구현. 이는 `weapon_archetype_targeting_goal.md`에서 처리한다.
- pathfinding/route authoring tool 구현.
- Unity fan-out/visual offset 구현.
- Rapier 라이브러리 제거. 제거가 더 낫다고 판단되면 이유를 기록하고 사용자와 먼저 의논한다.

## Policy Details

### Backend Responsibility

Rapier/backend physics is not the source of truth for:

- blocking.
- targetability.
- airborne terrain immunity.
- deploy occupancy.
- route progress.
- damage/hit decisions.

Rapier may be used only for:

- ground movement static obstacle correction.
- deterministic slide/stop against terrain obstacles, if that behavior is explicitly desired.
- board-bound helper only if clamp source of truth is documented.

### Movement Policy

초기 권장 형태:

```text
MovementTerrainPolicy
- Ground
- Airborne

Ground
- static_obstacles 적용.
- board bounds 적용.
- unit colliders 무시.

Airborne
- static_obstacles 무시.
- walkable/void tile 영향 없음.
- board bounds 적용.
- unit colliders 무시.
```

이 enum 이름은 실제 코드 구조에 맞게 조정할 수 있다. 핵심은 policy가 backend 내부 암묵 분기가 아니라 typed data로 흐르는 것이다.

### Local Avoidance

- 유닛 간 local avoidance는 기본 정책에서 사용하지 않는다.
- static obstacle local avoidance는 authored route를 벗어나는 우회가 될 수 있으므로 기본 전투 정책에서는 신중히 제한한다.
- 장애물에 막힌 지상 route는 데이터 오류 또는 route authoring 문제로 드러나는 편이 좋다.
- local avoidance를 유지해야 한다면 `ObstacleResponse::Slide` 같은 명시 정책과 focused tests를 둔다.

### Board Bounds

- board bounds는 core board clamp를 source of truth로 두는 방향을 우선 검토한다.
- Rapier wall collider와 clamp를 동시에 쓰면 의미가 중복되므로, 둘 중 하나의 역할을 축소하거나 명시한다.
- 공중 유닛도 board 밖으로 나가지 않는다.

## Implementation Plan

1. 현재 Rapier/Direct backend의 movement semantics 차이를 focused tests로 확인한다.
2. `MovementUnitInput`에 terrain/collision policy를 추가한다.
3. `BattleCore::build_continuous_movement_input_with_goals`에서 runtime unit policy를 채운다.
4. Ground policy는 기존 static obstacle correction을 유지한다.
5. Airborne policy는 static obstacle correction을 건너뛰고 route/board clamp만 적용한다.
6. Rapier `corrected_translation_for`에서 unit collider exclusion을 명시 helper/test로 유지한다.
7. local avoidance를 default path에서 제거하거나, 명시 obstacle response policy 아래로 이동한다.
8. board wall collider와 board clamp 중 source of truth를 정리한다.
9. Direct/Rapier가 같은 policy 입력에 대해 user-visible movement semantics를 맞추는 tests를 추가한다.
10. movement backend docs/goal 기록을 갱신한다.

## Test Requirements

- Ground unit is blocked or adjusted by static obstacle according to the chosen ground obstacle policy.
- Airborne unit ignores static obstacle and void/walkability obstacle projection.
- Airborne unit still stays inside board bounds.
- Unit colliders are ignored for movement correction.
- Overlapping units are not depenetrated by Rapier.
- Blocked enemy behavior is controlled by block_state, not Rapier.
- Direct and Rapier backends do not diverge on policy-visible behavior for Ground/Airborne movement.
- Local avoidance does not silently route units around authored DefenseRoute obstacles unless an explicit policy allows it.
- Movement output/tests expose collision-adjusted or blocked movement clearly enough to debug route authoring issues.

검증 후보:

```text
cargo test -p game_core movement -- --nocapture
cargo test -p game_core blocking -- --nocapture
cargo test -p game_core airborne -- --nocapture
cargo check -p game_core
```

## Completion Conditions

- movement/collision policy가 typed input/runtime path에 구현된다.
- Rapier backend 역할이 ground static obstacle correction helper로 제한된다.
- Airborne policy가 static obstacle/walkable/void correction을 우회할 수 있는 기반이 마련된다.
- 유닛 겹침/저지/source-of-truth 정책과 충돌하는 Rapier behavior가 남아 있지 않다.
- Direct/Rapier backend의 user-visible movement semantics가 tests로 고정된다.
- 사용자와 의논하여 정해야 할 정책이 있을 경우 goal을 종료한다.

## Stop Conditions

- Rapier 제거가 더 적절해 보이지만 외부 영향이 커서 사용자 결정이 필요하다.
- board bounds source of truth를 clamp로 할지 Rapier wall로 할지 코드 근거만으로 결정하기 어렵다.
- local avoidance를 유지할지 제거할지 gameplay 의미가 불명확하다.
- Airborne movement policy가 movement backend refactor 범위를 넘어 combat targeting/failure condition 정책을 다시 요구한다.
- 사용자와 의논하여 정해야 할 정책이 있을 경우 goal을 종료한다.
