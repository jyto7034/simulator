# Unit Overlap Blocking Goal

이 goal은 DefenseRoute 전투에서 아군 배치 겹침은 금지하되, 적/전투 중 이동 유닛의 좌표 겹침을 허용하고, 저지를 물리 충돌이 아닌 명시 논리 상태로 정리하는 작업이다.

이 goal은 이동/물리/저지/타겟팅에 걸치는 큰 goal이므로 독립적으로 수행한다.

## Goal Mode Working Method

이 문서는 `C:\Users\blast\Desktop\using-codex-goals-effectively-ko.md`의 goal 운용 원칙을 따른다.

작업자는 goal 시작 시 아래 작업 기억장치를 만들고 계속 갱신한다.

```text
docs/goals/unit_overlap_blocking/PLAN.md
docs/goals/unit_overlap_blocking/EXPERIMENTS.md
docs/goals/unit_overlap_blocking/EXPERIMENT_NOTES.md
```

## Engineering Principles

- 코드 수정은 장기적인 방향으로 한다.
- 물리 충돌 기반 길막 레거시는 compatibility layer로 보존하지 않는다.
- 처음 구조 판단이 어려우면 작은 전투 fixture로 trial and error를 수행한다.
- 문서를 무조건 신뢰하지 않고 Rapier/movement/blocking code를 꼼꼼히 읽는다.
- 과거 collision/spacing 정책을 고정하는 테스트는 삭제 또는 최신 정책 테스트로 교체한다.
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

- `src/game/battle/core/movement/blocking.rs`: blocking 매칭.
- `src/game/battle/core/movement/rapier_backend.rs`: 물리 충돌.
- `src/game/battle/core/movement/engine.rs`: movement tick.
- `src/game/battle/core/movement/planner.rs`: movement/route planning tests.
- `src/game/battle/core/movement/types.rs`: movement state.
- `src/game/battle/core/build.rs`: runtime unit 생성과 blocking stats.
- `src/game/battle/core/targeting.rs`: 기본 공격 target selection.
- `src/game/battle/core/commands.rs`: attack resolve.
- `src/game/battle/types.rs`: `CombatProfile`, block fields.
- `src/game/world/combat.rs`: deploy position validation.
- `src/game/battle/validation/**`: timeline validation.
- `docs/core_policy_decisions_2026_06.md`.

## Objective

완료 후 다음이 가능해야 한다.

1. 아군 배치 좌표 겹침은 `position_occupied`로 거부된다.
2. 적끼리 좌표 겹침이 가능하다.
3. 적과 아군 좌표 겹침이 가능하다.
4. 저지 중인 유닛끼리도 좌표상 겹칠 수 있다.
5. 유닛 간 물리 충돌은 길막/저지 source of truth가 아니다.
6. 저지는 `block_state` 같은 명시 상태로 결정된다.
7. 동시 저지 우선순위는 route progress 큰 적 우선이다.
8. 저지 용량 초과 적은 통과한다.

## In Scope

- unit-unit physics collision 제거 또는 무력화.
- obstacle/terrain collision 보존.
- block matching algorithm 갱신.
- route progress 기반 enemy 우선순위.
- spawn order, unit id tie-break.
- 가장 가까운 blocker 선택.
- blocked enemy attacks blocker.
- blocker prioritizes blocked enemies.
- Unity fan-out은 표시 전용임을 문서/contract와 맞춤.
- tests 대량 교체.

## Out Of Scope

- Unity fan-out 구현.
- weapon archetype targeting profile 전체 도입.
- boss/special movement 기믹 추가.
- route branching 새 설계.

## Implementation Plan

1. 현재 Rapier collision group과 unit collider 생성 경로를 확인한다.
2. 적/아군 unit collision이 route 진행을 막는지 작은 fixture로 확인한다.
3. obstacle/terrain collision과 unit-unit collision을 분리할 수 있는 최소 지점을 찾는다.
4. block state가 movement tick 전 deterministic하게 갱신되는지 확인한다.
5. enemy candidate 정렬을 route progress desc, spawn order, unit id로 바꾼다.
6. blocker candidate는 거리, deterministic id로 고른다.
7. capacity 초과 enemy가 blocked 대기열 때문에 멈추지 않고 route 진행하는지 확인한다.
8. targeting에서 blocker/blocked enemy 우선순위를 검증한다.
9. 레거시 collision spacing tests를 최신 정책으로 교체한다.

## Test Requirements

- 외길에서 4 enemies가 block capacity 2 ally에게 접근하면 2명은 blocked, 2명은 통과한다.
- blocked enemies와 blocker는 좌표상 겹칠 수 있다.
- enemy끼리 겹쳐도 route 진행이 막히지 않는다.
- route progress가 큰 enemy가 먼저 blocked 된다.
- 같은 progress면 spawn order, 그다음 unit id로 결정된다.
- 두 blocker 후보가 있으면 가장 가까운 blocker가 잡는다.
- deploy occupied ally tile은 여전히 거부된다.
- blocked enemy attacks blocker.
- blocker attacks blocked enemy first.

검증 후보:

```text
cargo test -p game_core blocking -- --nocapture
cargo test -p game_core movement -- --nocapture
cargo test -p game_core live_defense -- --nocapture
cargo test -p game_core -- --nocapture
cargo check -p game_core
```

## Completion Conditions

- unit overlap/blocking 정책이 runtime과 tests에 반영된다.
- 물리 충돌은 유닛 길막 source of truth가 아니다.
- 레거시 collision/spacing tests가 최신 정책으로 교체된다.
- 문서와 core behavior가 일치한다.
- 사용자와 의논하여 정해야 할 정책이 있을 경우 goal을 종료한다.

## Stop Conditions

- Rapier unit collision 제거가 projectile/hit detection까지 크게 깨뜨린다.
- route progress 계산 기준이 현재 route data에서 명확하지 않다.
- airborne/unblockable/flying 정책이 추가로 필요하다.
- 사용자와 의논하여 정해야 할 정책이 있을 경우 goal을 종료한다.
