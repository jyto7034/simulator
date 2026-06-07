# Skill Contract Surface Refactor Plan

## Objective

`docs/skill_contract_surface_refactor_goal.md`를 기준으로 Unity-facing 스킬 계약 표면의 기술 부채를 줄인다.

## Current Plan

1. 현재 snapshot/readiness/RON/buff 호출 경로를 실제 코드 기준으로 확인한다.
2. `skill_catalog`를 typed DTO로 전환한다.
3. `LiveBattleSkillReadinessDto`를 확장해 manual activation 가능 여부와 target 상태를 분리한다.
4. live skill RON의 반복 `defense_tile_range`를 preset 구조로 분리한다.
5. `BuffDatabase` 통합 범위를 분류한다. 작은 변경으로 가능하면 적용하고, 범위가 크면 별도 goal 필요성을 기록한다.
6. `unity_core_contract.md`, `skill_target_contract.md`, 필요 문서를 갱신한다.
7. focused test부터 실행하고, 이후 goal의 검증 명령을 순서대로 수행한다.

## Initial Findings

- `skill_catalog`는 `src/game/world/snapshot.rs`에서 `json!`로 직접 조립된다.
- `live_skill_readiness`는 target 부재를 `manual_activation_allowed=false`로 합친다.
- `../game_resources/data/skills/base.ron`에는 동일한 5x5 `defense_tile_range` 패턴이 반복된다.
- `BuffDatabase`는 `include_str! + static REGISTRY`이고 호출부가 `BattleCore`, validation, tests에 넓게 퍼져 있다. 즉시 주입식 통합은 범위가 클 수 있다.

## Completion Checklist

- [x] `skill_catalog` typed DTO화.
- [x] 실제 snapshot과 `unity_core_contract.md` 일치.
- [x] `manual_activation_allowed`와 target 상태 분리.
- [x] target 없음만으로 수동 스킬 버튼이 비활성화되지 않음.
- [x] range preset 또는 명확한 반복 제거 구조 도입.
- [x] range source of truth 충돌 validation.
- [x] BuffDatabase 통합 또는 별도 goal 필요성 기록.
- [x] 레거시 테스트 ignored 없이 삭제/교체.
- [x] 필요한 문서 갱신.
- [x] focused/full 검증 통과.
- [x] 사용자와 의논하여 정해야 할 정책은 BuffDatabase 통합 범위로 분리해 보고 항목에 기록.
