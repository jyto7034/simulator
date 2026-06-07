# Weapon Archetype Targeting Goal

이 goal은 직원 고정 직업 대신 장착 무기가 현재 전투 역할, 피해 타입, 사거리, 타겟팅 프로필을 결정하도록 장비/전투 구조를 확장하는 작업이다.

이 goal은 데이터 schema, battle profile, targeting runtime에 걸치는 큰 goal이므로 독립적으로 수행한다.

`AirFirst`와 `air_capable`은 `docs/airborne_enemy_mobility_goal.md`의 `mobility_kind: Airborne` 정책을 source of truth로 사용한다. 공중 적 mobility/피격/저지 정책을 이 goal 안에서 새로 정의하지 않는다.

## Goal Mode Working Method

이 문서는 `C:\Users\blast\Desktop\using-codex-goals-effectively-ko.md`의 goal 운용 원칙을 따른다.

작업자는 goal 시작 시 아래 작업 기억장치를 만들고 계속 갱신한다.

```text
docs/goals/weapon_archetype_targeting/PLAN.md
docs/goals/weapon_archetype_targeting/EXPERIMENTS.md
docs/goals/weapon_archetype_targeting/EXPERIMENT_NOTES.md
```

## Engineering Principles

- 코드 수정은 장기적인 방향으로 한다.
- 장비 이름을 직업처럼 하드코딩하지 않는다. archetype/profile을 data-driven source of truth로 둔다.
- 처음 구조 판단이 어려우면 schema trial을 작게 수행하고 기록한다.
- 레거시 permanent targeting/stats 구조가 새 정책과 충돌하면 과감히 정리한다.
- 문서를 무조건 신뢰하지 않고 실제 equipment/combat profile/build/targeting path를 읽는다.
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

- `src/game/data/equipment_data.rs`: equipment metadata.
- `src/game/battle/types.rs`: `CombatProfile`, stats, targeting.
- `src/game/employee.rs`: equipment 적용과 combat profile 계산.
- `src/game/battle/core/build.rs`: runtime unit 생성.
- `src/game/battle/core/targeting.rs`: attack target selection.
- `src/game/battle/core/commands.rs`: basic attack damage type.
- `docs/airborne_enemy_mobility_goal.md`: `AirFirst`/`air_capable`의 선행 mobility 정책.
- `src/game/battle/tile_range.rs`: DefenseRoute range pattern.
- `src/game/world/snapshot.rs`: equipment/employee snapshot.
- `src/game/behavior.rs`: Unity-facing DTO.
- `docs/core_policy_decisions_2026_06.md`.
- `docs/skill_target_contract.md`.

필요하면 `../game_resources/data/equipment/**`와 live RON을 확인한다.

## Objective

완료 후 다음이 가능해야 한다.

1. 직원의 현재 전투 역할은 장착 무기가 결정한다.
2. weapon item은 `range_role`, `weapon_archetype`, `damage_type`, `targeting_profile`, `air_capable`, `defense_tile_range`를 제공한다.
3. 기본 공격 damage type이 무기에서 나온다.
4. 원거리 기본 공격은 무기 `targeting_profile`을 사용한다.
5. 근거리 기본 공격은 저지 중인 적을 우선하고, 없으면 무기 profile을 사용한다.
6. `DefaultForward`는 route progress 기준이다.
7. `AirFirst`와 공중 단일 타겟 가능 여부는 `mobility_kind: Airborne`과 `air_capable`을 사용한다.

## In Scope

- weapon combat profile data schema 추가.
- 초기 7종 archetype:
  - `Sword`
  - `Spear`
  - `Shield`
  - `Bow`
  - `Gun`
  - `GrenadeLauncher`
  - `Staff`
- targeting profile enum:
  - `DefaultForward`
  - `AirFirst`
  - `LowDefenseFirst`
  - `LowMagicResistFirst`
  - `SplashClusterFirst`
- equipment validation.
- employee combat profile 계산 변경.
- battle runtime basic attack targeting 변경.
- Unity snapshot/contract 갱신.

## Out Of Scope

- skill fragment compatibility. 이는 `skill_fragment_compatibility_goal.md`에서 처리한다.
- airborne enemy mobility/저지/광역 피격 정책. 이는 `airborne_enemy_mobility_goal.md`에서 처리한다.
- 신규 장비 대량 컨텐츠.
- Unity loadout UI 구현.
- AD/AP balance validation.

## Implementation Plan

1. 현재 `EquipmentMetadata`와 `CombatProfile`의 관계를 읽는다.
2. weapon-only 필드를 equipment metadata에 직접 넣을지 nested `WeaponProfile`로 둘지 비교한다.
3. armor/accessory에는 weapon-only 필드를 허용하지 않도록 validation한다.
4. live equipment data를 최소 대표 무기 7종으로 갱신한다.
5. employee effective combat profile이 weapon profile을 반영하게 한다.
6. basic attack damage type을 hard-coded Physical에서 weapon damage type으로 바꾼다.
7. `targeting_profile` 기반 target selection을 구현한다.
8. route progress 기준 `DefaultForward`를 구현한다.
9. Unity snapshot에 current weapon combat profile을 노출한다.
10. 레거시 targeting tests를 최신 정책으로 교체한다.

## Test Requirements

- weapon 없는 직원은 명확한 fallback 또는 validation failure를 가진다.
- Sword는 melee/default forward profile을 제공한다.
- Bow는 `AirFirst`로 공중 enemy를 우선한다.
- Gun은 `LowDefenseFirst`를 적용할 수 있다.
- Staff는 `LowMagicResistFirst`를 적용할 수 있다.
- GrenadeLauncher는 `SplashClusterFirst`를 적용할 수 있다.
- basic attack damage type이 weapon damage type을 따른다.
- 근거리 유닛은 blocked enemy를 targeting profile보다 우선한다.
- Unity snapshot에 current weapon profile이 포함된다.

검증 후보:

```text
cargo test -p game_core equipment -- --nocapture
cargo test -p game_core targeting -- --nocapture
cargo test -p game_core battle -- --nocapture
cargo test -p game_core ron_loading -- --nocapture
cargo check -p game_core
cargo check -p game_server
```

## Completion Conditions

- weapon archetype/profile schema가 구현된다.
- basic attack과 targeting runtime이 weapon profile을 따른다.
- live/minimal RON이 새 schema로 통과한다.
- Unity-facing contract가 구현과 일치한다.
- 사용자와 의논하여 정해야 할 정책이 있을 경우 goal을 종료한다.

## Stop Conditions

- weapon 없는 직원 fallback 정책을 코드 근거만으로 정하기 어렵다.
- `defense_tile_range`를 weapon에 둘지 skill/basic attack profile에 둘지 정책 결정이 필요하다.
- 기존 equipment/stat modifier 구조와 새 weapon profile이 크게 충돌한다.
- 사용자와 의논하여 정해야 할 정책이 있을 경우 goal을 종료한다.
