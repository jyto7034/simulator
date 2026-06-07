# Airborne Enemy Mobility Goal

이 goal은 공중 적을 명일방주 드론처럼 지형지물의 영향을 받지 않는 별도 이동 계층으로 구현하고, 저지/피격/타겟팅/노드 경고 정책을 core source of truth로 정리하는 작업이다.

이 goal은 `Movement Backend Policy Refactor` 이후, `Weapon Archetype Targeting`의 `AirFirst`/`air_capable` 구현 전에 완료되어야 한다.

## Goal Mode Working Method

이 문서는 `C:\Users\blast\Desktop\using-codex-goals-effectively-ko.md`의 goal 운용 원칙을 따른다.

작업자는 goal 시작 시 아래 작업 기억장치를 만들고 계속 갱신한다.

```text
docs/goals/airborne_enemy_mobility/PLAN.md
docs/goals/airborne_enemy_mobility/EXPERIMENTS.md
docs/goals/airborne_enemy_mobility/EXPERIMENT_NOTES.md
```

## Engineering Principles

- 코드 수정은 장기적인 방향으로 한다.
- 공중 여부는 임시 `target_traits` 배열이 아니라 `mobility_kind`를 source of truth로 둔다.
- `Airborne`에서 파생되는 저지 불가, 지형 무시, 대공 필요, preview warning은 core가 계산한다.
- 데이터 작성자가 `Airborne`과 지상 전용 규칙을 동시에 설정할 수 있는 길은 validation으로 막는다.
- 공중 적을 지상 적 movement의 우연한 예외로 처리하지 않는다. route progress는 공유하되, terrain/obstacle/blocking 영향은 분리한다.
- 문서를 무조건 신뢰하지 않고 현재 movement/planner/blocking/targeting/skill area runtime을 읽는다.
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

- `src/game/data/abnormality_data.rs`: enemy metadata, movement/basic attack schema.
- `src/game/combat_enemy_spawns.rs`: enemy draft/spawn 생성.
- `src/game/battle/types.rs`: `UnitCombatProfile`, runtime mobility fields.
- `src/game/battle/core/build.rs`: runtime unit 생성.
- `src/game/battle/core/movement/planner.rs`: route movement planning.
- `src/game/battle/core/movement/blocking.rs`: block matching.
- `src/game/battle/core/movement/engine.rs`: continuous movement.
- `src/game/battle/core/targeting.rs`: basic attack target selection.
- `src/game/battle/core/skill_runtime/cast.rs`: single-target skill targeting.
- `src/game/battle/core/skill_runtime/area.rs`: area skill hit filtering.
- `src/game/combat_preview.rs`: threat warning generation.
- `src/game/world/snapshot.rs`: Unity-facing snapshot.
- `docs/movement_backend_policy_refactor_goal.md`: movement/collision policy와 Rapier backend 책임.
- `docs/core_policy_decisions_2026_06.md`.

필요하면 `/mnt/f/work/simulator/game_resources/data/abnormalities/**`, `/mnt/f/work/simulator/game_resources/data/skills/**`, `/mnt/f/work/simulator/game_resources/data/map/**` live RON을 확인한다.

## Objective

완료 후 다음이 가능해야 한다.

1. enemy는 `mobility_kind: Ground | Airborne`을 가진다.
2. `Airborne` enemy는 route polyline/waypoint를 가지지만 지형지물, 장애물, walkable tile 여부, 지상 충돌의 영향을 받지 않는다.
3. `Airborne` enemy는 route polyline segment를 따라 이동하되 route 끝을 넘어가지 않는다.
4. `Airborne` enemy는 route 끝에 도달하면 그 위치에서 대기하고, 공격 가능 시점마다 사거리 안 대상을 공격한다.
5. `Airborne` enemy는 저지되지 않는다.
6. `Airborne` enemy의 기본 공격 대상은 사거리 안 보호 목표 우선, 없으면 가장 가까운 살아있는 player combat unit이다.
7. `Airborne` enemy는 공격 사거리 안 대상 포착 시 공격 동안 잠깐 멈추고, 공격이 끝나면 다음 공격 가능 시점까지 이동한다.
8. 단일 기본 공격/단일 스킬은 `air_capable` 또는 동등한 명시 플래그가 있어야 `Airborne` enemy를 대상으로 삼을 수 있다.
9. 광역 스킬은 지상/공중 구분 없이 피격시킨다.
10. `CombatPreview`는 공중 적 등장 가능성을 `air_enemy_possible` warning으로 표현할 수 있다.
11. Unity는 공중 판정/저지/피격/타겟팅을 재계산하지 않고 core DTO를 표시한다.

## In Scope

- `MobilityKind` schema 도입.
- `AbnormalityMetadata` 또는 enemy combat profile에 `mobility_kind` 추가.
- `Airborne`에서 `blockable = false`를 runtime source of truth로 파생하거나, 잘못된 데이터 조합을 validation으로 거부.
- route progress 계산은 유지하되, 공중 이동은 terrain/obstacle/walkable 검사를 무시하게 분리.
- 공중 enemy 기본 공격 target selection 정책 구현.
- 단일 타겟팅의 air-capable 필터 기초 구현.
- 광역 스킬은 공중 포함 hit 유지/검증.
- `air_enemy_possible` preview warning 연결.
- Unity-facing snapshot/contract 갱신.
- focused tests 작성.

## Out Of Scope

- 아군 공중 유닛. 현재 추가 계획 없음.
- 무기 아키타입 전체 구현. 이는 `weapon_archetype_targeting_goal.md`에서 처리한다.
- 스킬 파편 호환성. 이는 `skill_fragment_compatibility_goal.md`에서 처리한다.
- 공중 전용 보스 기믹.
- 공중 전용 피해량/누수 피해 규칙. 공중 적은 route 끝에서 즉시 누수 피해를 주지 않고, 사거리 안 보호 목표를 공격한다.
- `Heavy`, `Shielded`, `Regenerating` 같은 별도 target trait 체계. 필요해질 때 독립 확장한다.

## Policy Details

### Mobility Kind

`mobility_kind`가 source of truth다.

```text
Ground
- 지형/장애물/walkable tile/저지 규칙을 받는다.
- blockable 설정 가능.

Airborne
- 지형지물/장애물/walkable tile 여부에 영향을 받지 않는다.
- 지상 충돌과 저지에 영향을 받지 않는다.
- 직선 비행으로 시작점에서 도착점까지 날아가지 않고, authored route waypoint/polyline은 따른다.
- route progress와 endpoint 도달 판정은 가진다.
- route 끝을 넘어가지 않는다.
- route 끝은 보호 목표를 기본 공격 사거리 안에 둘 수 있어야 한다.
- 지상 배치 타일 점유나 통행 점유에 영향을 주지 않는다.
- 대공 가능 단일 공격/스킬로만 단일 타겟팅 가능하다.
- 광역 스킬에는 피격된다.
```

초기 구현에서는 `target_traits: [Airborne]` 같은 중복 데이터는 만들지 않는다. 필요하면 runtime/snapshot에서 `mobility_kind == Airborne`으로 표시용 trait를 파생한다.

### Airborne Enemy Attack Loop

- 공중 적은 route waypoint/polyline을 따라 이동한다.
- 지형지물은 무시하지만 authored route waypoint/polyline 순서는 유지한다.
- 이동 중 공격 사거리 안에 대상이 들어오면 이동을 멈추고 기본 공격을 수행한다.
- 공격이 끝나면 다음 기본 공격 가능 시점까지 이동한다.
- 다음 공격 가능 시점에 사거리 안 대상이 있으면 다시 공격한다.
- 사거리 안 대상이 없으면 공격하지 않고 route를 따라 계속 이동한다.
- route 끝에 도달하면 그 위치에서 대기한다.
- route 끝에서 다음 공격 가능 시점마다 사거리 안 대상을 다시 찾는다.

### Airborne Enemy Target Priority

공중 적 기본 공격 대상:

1. 사거리 안 보호 목표.
2. 사거리 안 살아있는 player combat unit 중 가장 가까운 대상.
3. 거리 동률이면 deterministic unit id 순서.

보호 목표가 여러 개로 확장되면 거리 우선, 동률이면 deterministic id 순서로 처리한다.

이 보호 목표 우선순위는 공중 적 기본 공격 전용 정책이다. 지상 적 기본 공격/저지 타겟팅은 기존 지상 규칙을 따른다.

### Air Targeting

- 기본 공격과 단일 스킬은 공격/스킬 정의에 대공 가능 플래그가 있어야 공중 적을 후보로 삼는다.
- 대공 불가능한 단일 공격은 공중 적을 target candidate에서 제외한다.
- persisted/current target도 공격/스킬 해결 시점마다 대공 가능 여부를 다시 검증한다.
- 광역 스킬은 지상/공중 구분 없이 피격시킨다.
- 광역 스킬의 범위 판정은 공중 유닛의 2D projected world/tile position을 기준으로 한다.
- 기본 공격 splash/폭발이 별도 광역 delivery로 확장되기 전까지, 기본 공격은 단일 공격의 대공 규칙을 따른다.

### Data Validation

- `Airborne` enemy route endpoint는 보호 목표를 기본 공격 사거리 안에 둘 수 있어야 한다.
- `Airborne` enemy가 명시적으로 `blockable: true` 같은 지상 저지 전용 조합을 작성하면 validation에서 거부한다.
- 실제 공중 적이 등장 가능한 encounter/preview에는 `air_enemy_possible` warning 후보가 포함되어야 한다.

## Implementation Plan

1. 현재 enemy movement, route planning, blocking, basic attack target selection 흐름을 읽는다.
2. `MobilityKind` enum을 core data/runtime에 추가한다.
3. live/minimal enemy data가 기본 `Ground`로 deserialize되게 한다.
4. `Airborne` enemy는 block matching에서 후보가 되지 않게 한다.
5. `Airborne` movement는 terrain/obstacle/walkable check를 무시하되 authored route polyline을 따라가고 route endpoint를 넘지 않게 한다.
6. `Airborne` route endpoint가 보호 목표를 기본 공격 사거리 안에 둘 수 있는지 validation한다.
7. `Airborne` enemy target selection을 보호 목표 우선/가장 가까운 player unit으로 분리한다.
8. 단일 basic attack/skill target candidate에서 `Airborne`을 `air_capable`로 필터링하고, persisted/current target도 해결 시점마다 재검증한다.
9. area skill hit path가 projected 2D position 기준으로 공중을 포함하는지 확인하고 테스트로 고정한다.
10. `CombatPreview` warning generation에 `air_enemy_possible`을 연결한다.
11. Unity snapshot/contract에 `mobility_kind`와 공중 표시 규칙을 반영한다.
12. live RON에 최소 공중 적 fixture 또는 대표 데이터를 추가한다.

## Test Requirements

- `Airborne` enemy는 authored route polyline/waypoint를 따라 이동하고 route 끝을 넘지 않는다.
- `Airborne` enemy는 blocked state에 들어가지 않는다.
- `Airborne` enemy는 obstacle/walkable tile 제약을 무시하고 route segment를 진행한다.
- `Airborne` route endpoint가 보호 목표를 기본 공격 사거리 안에 둘 수 없으면 data validation이 실패한다.
- `Airborne` enemy는 사거리 안 보호 목표를 가장 가까운 player unit보다 우선 공격한다.
- 보호 목표가 사거리 밖이면 가장 가까운 player combat unit을 공격한다.
- 공격 중에는 이동이 잠깐 멈추고, 공격 후 다음 공격 가능 시점까지 이동한다.
- route 끝에 도달한 `Airborne` enemy는 대기하며 다음 공격 가능 시점마다 공격 대상을 찾는다.
- 대공 불가능한 기본 공격/단일 스킬은 `Airborne` enemy를 target candidate로 선택하지 않는다.
- 대공 가능 여부가 바뀐 persisted/current target은 공격/스킬 해결 시점에 invalid 처리된다.
- 대공 가능한 기본 공격/단일 스킬은 `Airborne` enemy를 target candidate로 선택할 수 있다.
- 광역 스킬은 projected 2D position이 범위 안인 `Airborne` enemy를 피격시킨다.
- 공중 적이 등장 가능한 preview는 `air_enemy_possible` warning을 포함한다.
- live RON loading이 `mobility_kind` schema를 통과한다.

검증 후보:

```text
cargo test -p game_core airborne -- --nocapture
cargo test -p game_core movement -- --nocapture
cargo test -p game_core blocking -- --nocapture
cargo test -p game_core targeting -- --nocapture
cargo test -p game_core combat_preview -- --nocapture
cargo test -p game_core ron_loading -- --nocapture
cargo check -p game_core
cargo check -p game_server
```

## Completion Conditions

- `mobility_kind` schema가 구현된다.
- `Airborne` movement/targeting/blocking/피격 정책이 runtime과 tests에 반영된다.
- `air_enemy_possible` warning이 core source of truth로 생성된다.
- Unity-facing contract가 구현과 일치한다.
- live/minimal RON이 새 schema로 통과한다.
- 사용자와 의논하여 정해야 할 정책이 있을 경우 goal을 종료한다.

## Stop Conditions

- 현재 movement engine이 terrain 무시 route movement와 지상 movement를 분리하기 어렵고, 새 movement layer 설계가 필요하다.
- single-target skill의 `air_capable` 필드 위치가 skill target schema와 크게 충돌한다.
- area skill hit path가 공중/지상 구분을 강하게 전제하고 있어 별도 skill policy 논의가 필요하다.
- 공중 적이 보호 목표를 공격하는 방식이 현재 DefenseRoute 실패/보호 목표 HP 구조와 충돌한다.
- 사용자와 의논하여 정해야 할 정책이 있을 경우 goal을 종료한다.
