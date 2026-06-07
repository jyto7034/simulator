# Skill Fragment Compatibility Goal

이 goal은 스킬 파편 장착 가능 여부를 직원 고정 직업이 아니라 현재 장착 무기/전투 프로필 기준으로 검증하도록 바꾸는 작업이다.

선행 권장 goal: `docs/weapon_archetype_targeting_goal.md`.

## Goal Mode Working Method

이 문서는 `C:\Users\blast\Desktop\using-codex-goals-effectively-ko.md`의 goal 운용 원칙을 따른다.

작업자는 goal 시작 시 아래 작업 기억장치를 만들고 계속 갱신한다.

```text
docs/goals/skill_fragment_compatibility/PLAN.md
docs/goals/skill_fragment_compatibility/EXPERIMENTS.md
docs/goals/skill_fragment_compatibility/EXPERIMENT_NOTES.md
```

## Engineering Principles

- 코드 수정은 장기적인 방향으로 한다.
- compatibility를 Unity 쪽 추론에 맡기지 않는다.
- 레거시 “모든 파편 장착 가능” 정책이 새 설계와 충돌하면 과감히 교체한다.
- 문서를 무조건 신뢰하지 않고 실제 equip/loadout/data validation path를 읽는다.
- 테스트가 과거 자유 장착 정책을 고정하면 최신 정책으로 교체한다.
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

- `src/game/data/skill_fragment_data.rs`: fragment metadata/effect.
- `src/game/skill_fragment.rs`: inventory/loadout/equip policy.
- `src/game/employee.rs`: employee loadout and combat profile.
- `src/game/world/maintenance.rs`: equip/unequip command handling.
- `src/game/world/helpers.rs`: command validation.
- `src/game/world/snapshot.rs`: Unity-facing roster/loadout snapshot.
- `src/game/data/mod.rs`: cross-data validation.
- `src/game/data/equipment_data.rs`: weapon profile schema after prior goal.
- `docs/core_policy_decisions_2026_06.md`.
- `docs/skills/lobotomy_content_catalog.md`.

필요하면 `../game_resources/data/skill_fragments/base.ron`을 확인한다.

## Objective

완료 후 다음이 가능해야 한다.

1. fragment metadata가 장착 조건을 표현한다.
2. 조건은 current weapon combat profile을 기준으로 검증한다.
3. core가 최종 장착 가능 여부의 source of truth다.
4. Unity snapshot은 호환/비호환 상태와 실패 사유를 표시할 수 있다.
5. 유명/개성 강한 환상체 파편은 좁은 archetype 조건을 가질 수 있다.
6. 범용 파편은 넓은 조건을 가질 수 있다.

## In Scope

- fragment requirement schema 추가:
  - `range_role`
  - `weapon_archetype`
  - `damage_type`
  - `targeting_profile`
  - `air_capable`
  - `block_capacity_min`
  - `required_capability_tags`
  - `incompatible_tags`
- equip validation.
- snapshot compatibility DTO.
- command failure reason 정리.
- live RON 최소 갱신.
- tests 갱신.

## Out Of Scope

- reward pool smart weighting 전체 구현.
- 신규 환상체 파편 대량 작성.
- Unity drag/drop UI 구현.
- weapon archetype runtime 도입.

## Implementation Plan

1. 현재 `SkillFragmentMetadata` schema와 equip path를 읽는다.
2. requirement를 optional nested struct로 둘지, repeated enum condition으로 둘지 비교한다.
3. current employee weapon combat profile을 equip validation에 전달하는 경로를 찾는다.
4. command validation과 data validation을 분리한다.
5. Unity snapshot에 `is_compatible`, `compatibility_reason`, `missing_requirements` 같은 표시 정보를 둘지 검토한다.
6. 모르는 requirement가 생겼을 때 fail-closed로 처리한다.
7. starter/basic fragment의 호환 조건을 넓게 설정한다.
8. 몇 개 대표 live fragment에 좁은 조건을 추가한다.
9. 기존 자유 장착 테스트를 최신 정책 테스트로 교체한다.

## Test Requirements

- compatible weapon이면 fragment equip 성공.
- incompatible weapon이면 equip 실패와 명확한 reason 반환.
- no weapon 또는 invalid weapon 상태에서 active fragment equip 정책이 명확하다.
- broad/generic fragment는 여러 archetype에서 equip 가능.
- archetype-locked fragment는 지정 archetype에서만 equip 가능.
- Unity snapshot에 compatibility 표시 정보가 있다.
- data validation이 존재하지 않는 archetype/profile requirement를 잡는다.

검증 후보:

```text
cargo test -p game_core skill_fragment -- --nocapture
cargo test -p game_core equipment -- --nocapture
cargo test -p game_core employee -- --nocapture
cargo test -p game_core ron_loading -- --nocapture
cargo check -p game_core
cargo check -p game_server
```

## Completion Conditions

- fragment compatibility schema와 validation이 구현된다.
- Unity-facing snapshot/contract가 compatibility를 표시할 수 있다.
- 기존 자유 장착 정책 테스트가 최신 정책으로 교체된다.
- live RON이 새 schema validation을 통과한다.
- 사용자와 의논하여 정해야 할 정책이 있을 경우 goal을 종료한다.

## Stop Conditions

- no weapon 상태에서 fragment equip 허용 여부를 정책적으로 정해야 한다.
- reward pool에서 incompatible fragment 반복 지급을 동시에 해결해야 해서 범위가 커진다.
- compatibility failure DTO shape가 Unity 정책 결정을 요구한다.
- 사용자와 의논하여 정해야 할 정책이 있을 경우 goal을 종료한다.
