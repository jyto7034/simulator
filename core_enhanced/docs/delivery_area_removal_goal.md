# DeliveryDef::Area Removal Goal

이 goal은 `DefenseRoute` 단일 전투 정책에 맞지 않는 레거시 geometric AoE 경로를 완전히 제거하기 위한 리팩토링 계획서다.

핵심 목적은 새 스킬 시스템을 만드는 것이 아니라, 현재 공식 스킬 범위 source of truth를 `SkillStepDef.defense_tile_range`와 `DeliveryDef::TileArea`로 단일화하는 것이다. `DeliveryDef::Area`, `SkillAreaShapeDef`, `Circle/Line/Box/Rectangle/Cone` 기반 피격 판정은 레거시로 간주하고 compatibility layer 없이 제거한다.

## Goal Mode Working Method

이 문서는 `C:\Users\blast\Desktop\using-codex-goals-effectively-ko.md`의 goal 운용 원칙을 따른다.

작업자는 goal 시작 시 아래 작업 기억장치를 만들고 계속 갱신한다.

```text
docs/goals/delivery_area_removal/PLAN.md
docs/goals/delivery_area_removal/EXPERIMENTS.md
docs/goals/delivery_area_removal/EXPERIMENT_NOTES.md
```

각 파일의 역할:

- `PLAN.md`: 구현 순서, 현재 판단, 남은 체크리스트, 완료 조건을 기록한다.
- `EXPERIMENTS.md`: 시도한 접근, 실패/성공 결과, 테스트 실패 원인과 해결을 기록한다.
- `EXPERIMENT_NOTES.md`: 작업 중 발견한 의심, 정책 질문, 기술 부채 후보를 시간순으로 기록한다.

이 파일들은 최종 정책 문서가 아니라 goal 실행 중의 작업 기억장치다. goal 완료 후 유지해야 할 내용만 `skill_target_contract.md`, `unity_core_contract.md`, `game_rulebook.md`, `refactor_preparation_plan.md`로 옮긴다.

## Engineering Principles

- 코드 수정은 장기적인 방향으로 한다. 임시방편, 최소 수정, 특정 테스트만 맞추는 패치는 피한다.
- 처음에는 어떤 구조가 장기적인 방향인지 확실하지 않을 수 있다. 작은 단위로 trial and error를 수행하고, 실패한 접근과 이유를 `EXPERIMENTS.md`에 남긴다.
- 레거시는 과감하게 제거한다. `DeliveryDef::Area`를 validation으로만 막거나 adapter로 감싸서 보존하지 않는다.
- 문서를 무조건 신뢰하지 않는다. 실제 코드와 live RON/API를 읽으면서 더 나은 개선안이 있으면 근거를 기록하고 적용한다.
- 문서에 적힌 내용보다 코드에서 더 단순하고 안전한 개선안이 보이면, 왜 더 나은지 기록한 뒤 적용한다. 단, 정책 판단이 필요한 경우 사용자에게 질문하고 goal을 종료한다.
- 새 추상화는 실제 반복이 확인된 뒤 만든다. 이번 goal의 기본 방향은 추가 추상화가 아니라 레거시 삭제와 source of truth 축소다.
- 테스트는 내부 구조가 아니라 공식 DefenseRoute 스킬 사용 흐름, 데이터 로딩 실패, Unity-facing 계약을 고정한다.
- 정책 변경으로 실패하는 테스트가 geometric AoE, 구형 스킬 타겟팅, TFT식 범위, legacy/generated RON 호환 같은 레거시 동작을 고정하고 있다면 업데이트보다 삭제를 우선한다. 레거시 테스트를 ignored 처리해 보존하지 않는다.

## Source Of Truth

우선 읽을 파일:

- `src/game/ability.rs`: `DeliveryDef`, `SkillAreaDeliveryDef`, `SkillAreaShapeDef`, `SkillTileAreaDeliveryDef`.
- `src/game/data/skill_data.rs`: skill validation.
- `src/game/data/mod.rs`: cross-database DefenseRoute skill validation.
- `src/game/battle/tile_range.rs`: `TileRangePattern` 공식 범위 계약.
- `src/game/battle/core/sim.rs`: skill step execution, `DeliveryDef::Area`/`TileArea` 분기.
- `src/game/battle/core/skill_runtime/area.rs`: geometric area runtime과 tile area runtime.
- `src/game/battle/core/targeting.rs`: basic attack/skill range 판정.
- `src/game/battle/core/commands.rs`: delivery classification과 skill command 처리.
- `src/game/battle/core/types.rs`: `AreaRuntime`.
- `src/game/battle/core/spatial.rs`: geometric area query backend.
- `src/game/battle/timeline.rs`: `TimelineSkillAreaShape`.
- `src/game/world/snapshot.rs`: `skill_catalog` delivery 노출.
- `src/game/world/combat.rs`: deploy-time skill contract validation.
- `tests/ron_loading.rs`, `tests/skill_refactor_validation.rs`, `tests/skill_test/**`: 스킬 계약 테스트.
- `../game_resources/data/skills/base.ron`: 공식 live skill source of truth.
- `../game_resources/data/skills/legacy_base.generated.ron`: 제거 또는 보존 여부를 결정해야 하는 legacy generated RON.
- `docs/skill_target_contract.md`: 공식 스킬 범위 계약.
- `docs/unity_core_contract.md`: Unity-facing timeline/snapshot 계약.
- `docs/game_rulebook.md`: 전투/스킬 정책.
- `docs/refactor_preparation_plan.md`: 리팩토링 판단 기준.

필요하면 `../game_server`의 `/game` request/result mapping도 확인한다. server mapping 변경이 필요하면 core 계약과 함께 갱신한다.

## Current Findings

현재 확인된 상태:

- 공식 live skill RON인 `../game_resources/data/skills/base.ron`은 이미 `TileArea`와 `defense_tile_range`를 사용한다.
- `../game_resources/data/skills/legacy_base.generated.ron`에는 `delivery: Area(...)`와 `Circle/Line/Box/Rectangle/Cone` 기반 데이터가 남아 있다.
- `DeliveryDef::Area`는 공식 DefenseRoute 경로에서는 validation으로 차단되지만, Rust schema와 runtime에는 아직 남아 있다.
- `SkillAreaShapeDef`, `SkillAreaDeliveryDef`, `AreaQueryShape`, `TimelineSkillAreaShape::Circle/Line/Box/Rectangle/Cone`이 레거시 geometric AoE를 유지하고 있다.
- core 단위 테스트 일부는 geometric area runtime 자체를 검증한다. 공식 게임 흐름을 고정하는 테스트가 아니라면 삭제 대상이다.
- `skill_catalog`는 `DeliveryDef::Area`를 `"legacy_area"`로 노출할 수 있는 분기를 갖고 있다. 공식 계약에서는 이 분기가 없어야 한다.

## Objective

완료 후 다음이 가능해야 한다.

1. 공식 스킬 delivery schema에서 `DeliveryDef::Area`가 사라진다.
2. geometric AoE shape인 `Circle`, `Line`, `Box`, `Rectangle`, `Cone`이 공식 skill/timeline/snapshot 계약에 남지 않는다.
3. DefenseRoute 스킬 범위는 `defense_tile_range`와 `TileArea`만으로 표현된다.
4. Unity는 timeline/snapshot에서 geometric area를 받을 가능성을 고려하지 않아도 된다.
5. 레거시 geometric area 테스트는 ignored 처리 없이 삭제되거나 tile range focused test로 교체된다.
6. live RON과 code schema의 source of truth가 일치한다.

## In Scope

- `DeliveryDef::Area` variant 제거.
- `SkillAreaDeliveryDef` 제거.
- `SkillAreaShapeDef` 제거.
- `DeliveryDef::Area` execution branch 제거.
- geometric area runtime 함수 제거:
  - `resolve_area_anchor_position`
  - `resolve_area_geometry`
  - `resolve_instant_area_targets`
  - `collect_area_targets_at`
  - `record_skill_area_declared`
  - `register_persistent_area`
  - 단, 같은 이름의 helper가 `TileArea`에도 필요하면 tile 전용 이름으로 정리한다.
- `AreaRuntime.shape` 제거.
- `AreaQueryShape`와 geometric area query 제거 또는 skill runtime에서 분리.
- `TimelineSkillAreaShape`를 `TilePattern` 중심으로 축소.
- `skill_catalog`에서 `"legacy_area"` delivery 제거.
- `skill_data` validation에서 `DeliveryDef::Area` 분기 제거.
- `world/combat.rs`, `targeting.rs`, `commands.rs`의 `Area | TileArea` 동시 처리 분기 제거.
- `legacy_base.generated.ron` 제거 또는 game_resources의 명확한 archive 정책에 맞게 공식 검색/검증 경로에서 제외.
- geometric area를 고정하는 테스트 삭제.
- 필요한 경우 tile area focused test 추가.
- `skill_target_contract.md`, `unity_core_contract.md`, `game_rulebook.md` 갱신.

## Out Of Scope

- 신규 스킬 컨텐츠 추가.
- 스킬 수치 밸런스 조정.
- 투사체 런타임 재작성.
- buff/effect scripting runtime 확장.
- 이동 시스템 리팩토링.
- Unity 클라이언트 UI 구현.
- 조건부 BattleScenario 이벤트 구현.
- `DefenseRoute` 외 전투 모드 부활.

## Implementation Plan

1. 현 사용처 감사

- `DeliveryDef::Area`, `SkillAreaDeliveryDef`, `SkillAreaShapeDef`, `AreaQueryShape`, `TimelineSkillAreaShape` geometric variants의 모든 사용처를 `rg`로 수집한다.
- 각 사용처를 live path, test-only path, dead/legacy path로 분류한다.
- live path에서 실제 geometric area가 필요하다는 근거가 발견되면 goal을 종료하고 사용자에게 보고한다.

2. live RON 계약 확인

- `../game_resources/data/skills/base.ron`에 `delivery: Area` 또는 geometric shape가 없는지 확인한다.
- `legacy_base.generated.ron`이 코드, 테스트, 문서, 외부 loader에서 참조되는지 확인한다.
- 참조가 없으면 제거한다. 보존 필요성이 발견되면 goal을 종료하고 사용자에게 질문한다.

3. schema 제거

- `DeliveryDef::Area` variant를 제거한다.
- `SkillAreaDeliveryDef`와 `SkillAreaShapeDef`를 제거한다.
- 남는 delivery는 `Instant`, `Projectile`, `TileArea`로 단순화한다.
- serde/RON schema가 변경되므로 live RON과 tests를 함께 갱신한다.

4. runtime 제거

- `BattleCore`의 geometric area execution branch를 제거한다.
- persistent area runtime은 tile area만 지원하게 정리한다.
- `AreaRuntime`은 tile area에 필요한 필드만 남긴다.
- geometric area helper와 spatial query 의존을 제거한다.
- 삭제 후 이름이 애매한 함수는 `tile_area` 의미가 드러나도록 rename한다.

5. timeline/snapshot 계약 정리

- `TimelineSkillAreaShape`를 `TilePattern { affected_tiles }` 중심으로 축소한다.
- Unity contract에서 `circle`, `line`, `box`, `rectangle`, `cone` legacy/debug 설명을 제거한다.
- `skill_catalog` delivery 문자열에서 `legacy_area`를 제거한다.
- snapshot/timeline focused test로 geometric area가 더 이상 노출되지 않음을 검증한다.

6. tests 정리

- geometric area 자체를 검증하는 테스트는 공식 게임 흐름과 맞지 않으므로 삭제한다.
- 기존 테스트가 geometric shape semantics를 고정한다면 ignored 처리하지 않는다.
- 필요한 경우 같은 의도를 `TileArea + defense_tile_range` focused test로 대체한다.
- spatial backend에서 projectile/collision에 필요한 연속좌표 테스트는 보존하되, geometric skill area 전용 테스트는 제거한다.

7. 문서 갱신

- `skill_target_contract.md`에서 geometric area를 legacy/debug 가능성으로 남긴 문장을 제거한다.
- `unity_core_contract.md`에서 `TimelineSkillAreaShape` geometric variants를 제거한다.
- `game_rulebook.md`에 공식 스킬 범위는 tile range라고 정리한다.
- `refactor_preparation_plan.md`에 별도 지시가 필요하면 현재 기준만 간단히 반영한다.

## Test Requirements

focused test를 먼저 작성/갱신한다.

- live skill RON 로딩이 `DeliveryDef::Area` 없이 통과한다.
- `SkillDatabase` 또는 RON deserialize 단계에서 `Area(...)`를 더 이상 공식 schema로 받지 않는다.
- `SkillAreaDeclared` timeline event는 `tile_pattern`만 노출한다.
- `skill_catalog.skills[*].steps[*].delivery`에는 `instant`, `projectile`, `tile_area`만 등장한다.
- DefenseRoute `TileArea` 스킬은 `defense_tile_range`에 포함된 타일의 유닛만 피격한다.
- 삭제한 geometric area 테스트는 ignored 처리로 남기지 않는다.
- `rg -n "DeliveryDef::Area|SkillAreaDeliveryDef|SkillAreaShapeDef|legacy_area|TimelineSkillAreaShape::Circle|TimelineSkillAreaShape::Line|TimelineSkillAreaShape::Box|TimelineSkillAreaShape::Rectangle|TimelineSkillAreaShape::Cone" src tests` 결과가 없어야 한다. 단, 문서나 goal 기록은 제외한다.
- `rg -n "delivery:\\s*Area|shape:\\s*(Circle|Line|Box|Rectangle|Cone)" ../game_resources/data/skills` 결과가 없어야 한다.

검증 순서:

```text
cargo test -p game_core ron_loading -- --nocapture
cargo test -p game_core skill_refactor_validation -- --nocapture
cargo test -p game_core skill_test_suite -- --nocapture
cargo test -p game_core -- --nocapture
cargo check -p game_core
cargo check -p game_server
```

server mapping을 변경했다면 다음도 수행한다.

```text
cargo test -p game_server -- --nocapture
```

## Stop Conditions

다음 경우 임의 확정하지 말고 goal을 종료하고 질문 목록을 보고한다.

- live path에서 `DeliveryDef::Area`가 실제로 필요한 공식 기능이 발견된다.
- `legacy_base.generated.ron`이 외부 도구, Unity 작업, 데이터 생성 파이프라인에서 아직 필요하다는 근거가 발견된다.
- geometric shape가 skill이 아닌 다른 공식 gameplay 판정에 재사용되고 있어 제거 범위가 스킬 시스템을 넘어선다.
- Unity contract에서 `circle/line/cone` 등 geometric shape를 아직 소비하는 최신 클라이언트 코드가 발견된다.
- `TileArea`만으로 표현할 수 없는 공식 스킬 정책이 발견된다.
- 사용자와 의논하여 정해야 할 정책이 있을 경우 goal을 종료한다.

## Completion Criteria

1. `DeliveryDef::Area`가 코드에서 제거된다.
2. `SkillAreaDeliveryDef`와 `SkillAreaShapeDef`가 코드에서 제거된다.
3. 공식 skill/timeline/snapshot contract에는 `TileArea`와 `tile_pattern`만 남는다.
4. `skill_catalog`에서 `legacy_area`가 사라진다.
5. live skill RON과 tests에 `delivery: Area` 또는 geometric shape가 남지 않는다.
6. geometric area runtime과 테스트가 ignored 처리 없이 삭제된다.
7. DefenseRoute tile area focused tests가 최신 정책을 고정한다.
8. 필요한 문서가 최신 정책으로 갱신된다.
9. focused tests와 `cargo test -p game_core -- --nocapture`, `cargo check -p game_core`, `cargo check -p game_server`가 통과한다.
10. game_server mapping을 변경했다면 `cargo test -p game_server -- --nocapture`가 통과한다.
11. 사용자와 의논하여 정해야 할 정책이 있을 경우 goal을 종료한다.

## Goal Command

```text
/goal docs/delivery_area_removal_goal.md를 기준으로, DefenseRoute 공식 스킬 범위 정책과 충돌하는 레거시 geometric AoE 경로를 완전히 제거하라. 먼저 src/game/ability.rs, src/game/data/skill_data.rs, src/game/data/mod.rs, src/game/battle/tile_range.rs, src/game/battle/core/sim.rs, src/game/battle/core/skill_runtime/area.rs, src/game/battle/core/targeting.rs, src/game/battle/core/commands.rs, src/game/battle/core/types.rs, src/game/battle/core/spatial.rs, src/game/battle/timeline.rs, src/game/world/snapshot.rs, src/game/world/combat.rs, tests/ron_loading.rs, tests/skill_refactor_validation.rs, tests/skill_test/**, ../game_resources/data/skills/base.ron, ../game_resources/data/skills/legacy_base.generated.ron, docs/skill_target_contract.md, docs/unity_core_contract.md, docs/game_rulebook.md, docs/refactor_preparation_plan.md를 꼼꼼히 읽고 DeliveryDef::Area, SkillAreaDeliveryDef, SkillAreaShapeDef, TimelineSkillAreaShape의 geometric variants, AreaQueryShape 사용처를 live path/test-only/dead legacy로 분류하라. 공식 live path에 geometric Area가 필요하다는 근거가 없으면 DeliveryDef::Area variant, SkillAreaDeliveryDef, SkillAreaShapeDef, geometric area runtime, geometric timeline shape, skill_catalog legacy_area 분기, legacy generated Area RON, geometric area 테스트를 compatibility layer 없이 제거하라. 공식 DefenseRoute 스킬 범위는 SkillStepDef.defense_tile_range와 DeliveryDef::TileArea만 source of truth로 유지하고, Unity-facing timeline/snapshot 계약은 tile_pattern/affected_tiles만 노출하게 하라. 코드 수정은 장기적인 방향으로 하고 임시방편, 최소 수정, 특정 테스트만 맞추는 패치를 피하라. 처음 구조 판단이 어려우면 작은 trial and error를 수행하고 docs/goals/delivery_area_removal/PLAN.md, EXPERIMENTS.md, EXPERIMENT_NOTES.md에 계획, 실험, 생각을 기록하라. 레거시는 과감하게 제거하고, 정책 변경으로 실패하는 geometric AoE/구형 스킬 타겟팅/legacy compatibility 테스트는 ignored 처리하지 말고 삭제하거나 최신 tile range focused test로 교체하라. 문서를 무조건 신뢰하지 말고 실제 코드와 live RON/API를 읽으면서 더 나은 개선안이 보이면 근거를 기록하고 적용하라. 단, live path에서 DeliveryDef::Area가 실제로 필요한 공식 기능, legacy_base.generated.ron 외부 의존, geometric shape를 소비하는 최신 Unity 계약, TileArea만으로 표현할 수 없는 공식 스킬 정책처럼 사용자와 의논해야 할 정책이 발견되면 goal을 종료하고 질문 목록을 보고하라. 검증은 cargo test -p game_core ron_loading -- --nocapture, cargo test -p game_core skill_refactor_validation -- --nocapture, cargo test -p game_core skill_test_suite -- --nocapture, cargo test -p game_core -- --nocapture, cargo check -p game_core, cargo check -p game_server 순서로 수행하고, game_server mapping을 변경했다면 cargo test -p game_server -- --nocapture도 수행하라. 완료 조건은 1) DeliveryDef::Area 제거, 2) SkillAreaDeliveryDef/SkillAreaShapeDef 제거, 3) 공식 skill/timeline/snapshot contract가 TileArea/tile_pattern만 사용, 4) skill_catalog legacy_area 제거, 5) live skill RON/tests에서 delivery: Area/geometric shape 제거, 6) geometric area runtime/tests가 ignored 없이 삭제, 7) DefenseRoute tile area focused tests 갱신, 8) 필요한 문서 갱신, 9) 검증 명령 통과, 10) 사용자와 의논하여 정해야 할 정책이 있을 경우 goal을 종료함이다.
```
