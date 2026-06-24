# Core Component Refactor Master Goal

## Objective

`core_enhanced` 프로젝트를 큰 컴포넌트 축으로 순회하면서 각 컴포넌트의 runtime code, live RON/data, Unity-facing 계약, 테스트를 직접 읽고 리팩토링 후보를 발견한다.

각 컴포넌트 감사 결과는 `docs/goals/core_component_refactor_master/<component_name>_refactor.md` 문서로 남긴다. 이 문서는 `docs/refactor_preparation_plan.md`를 리팩토링 판단 기준으로 삼아 해당 컴포넌트의 source-of-truth 판단, 기존 기능 조합으로 단순화 가능한 부분, 레거시 제거 후보, 필요한 검증 방법, 정책 논의 필요 항목을 기록하는 작업 산출물이다.

## Working Method

이 master goal은 모든 컴포넌트를 순회하는 감사 목표다. 컴포넌트별 산출물은 sub goal이 아니라 `<component_name>_refactor.md` 감사 문서다.

진행 방식:

1. 아래 component map을 기준으로 한 컴포넌트를 고른다.
2. 해당 컴포넌트의 runtime code를 먼저 읽는다.
3. 관련 live RON/data, Unity-facing snapshot/command/result 계약, 최신 정책 문서, 테스트를 순서대로 확인한다.
4. `docs/refactor_preparation_plan.md`의 기준을 참고하되, 문서를 무조건 신뢰하지 않고 실제 코드와 데이터의 증거를 우선한다.
5. 해당 컴포넌트 이름을 반영한 `<component_name>_refactor.md`를 만들거나 갱신한다.
6. 리팩토링 후보, 하지 않는 판단, 보류 사유, 정책 논의 필요 항목, 검증 계획을 문서화한다.
7. 작은 trial and error를 수행한 경우 `EXPERIMENTS.md`에 시도, 실패 원인, 수정, 재검증 결과를 남긴다.
8. goal 범위를 벗어난 개선안은 `EXPERIMENT_NOTES.md`에 후속 후보로 기록한다.
9. 한 컴포넌트 감사를 마치면 아래 진행 상태를 갱신하고 다음 컴포넌트로 이동한다.
10. 모든 컴포넌트 순회가 끝나면 생성된 모든 `<component_name>_refactor.md` 문서를 실제 코드, live RON/data, Unity-facing 계약, 테스트와 다시 대조한다.
11. 최종 재검토에서 근거가 약하거나 코드와 어긋나는 판단은 수정, 강등, 삭제하고 그 결과를 `EXPERIMENTS.md`에 기록한다.

## Goal Documents

이 goal은 시작 시 아래 파일을 만들고 계속 갱신한다.

- `docs/goals/core_component_refactor_master/PLAN.md`: 계획, 컴포넌트 목록, 완료 조건, 중단 조건.
- `docs/goals/core_component_refactor_master/EXPERIMENTS.md`: 시도, 실패, 성공, 테스트 실패와 재검증 결과.
- `docs/goals/core_component_refactor_master/EXPERIMENT_NOTES.md`: 작업 중 판단, 정책 논의 필요 항목, 범위 밖 후속 후보.

컴포넌트별 감사 문서는 같은 폴더에 둔다.

- 예: `world_run_progression_refactor.md`
- 예: `skill_targeting_range_refactor.md`

모든 컴포넌트별 refactor 문서는 `docs/refactor_preparation_plan.md`의 리팩토링 기준, source-of-truth 기준, 레거시 제거 기준, 테스트 기준을 기본 판단 프레임으로 사용한다. 단, 실제 runtime code, live RON/data, Unity-facing 계약이 더 강한 source of truth이며, 충돌이 있으면 코드와 데이터 근거를 우선 기록한다.

각 컴포넌트 감사 문서는 최소한 아래 항목을 포함한다.

- 읽은 코드와 데이터 범위.
- 현재 구조 요약.
- source-of-truth 판단.
- 리팩토링 후보.
- 기존 기능 조합으로 단순화 가능한 후보.
- 레거시, fallback, compatibility layer, dual schema 제거 후보.
- 하지 않거나 보류한 항목과 이유.
- 필요한 테스트와 검증 명령.
- 사용자 결정이 필요한 정책 논의 필요 항목.

## Refactor Criteria

이 goal의 canonical 리팩토링 기준 문서는 `docs/refactor_preparation_plan.md`다. 모든 컴포넌트 감사 문서는 해당 문서를 기준으로 source-of-truth 중복, 기존 기능 조합 가능성, 레거시 제거, 테스트/검증 방향을 판단한다.

리팩토링 후보:

- 같은 데이터나 정책을 둘 이상 source에서 동기화하며 관리한다.
- 기존 범위, 판정, 필터링, delivery, validation 조합으로 표현 가능한데 별도 특수 로직으로 유지한다.
- live path에서 쓰지 않는 legacy, fallback, compatibility layer가 최신 정책과 충돌한다.
- Unity/client가 core 최종 DTO 대신 RON/data/runtime 규칙을 재추론해야 한다.
- 테스트가 사용자-visible behavior나 계약이 아니라 과거 내부 구현 모양만 고정한다.

리팩토링 후보가 아닌 것:

- 수명주기와 책임이 실제로 다른 타입을 겉모양만 맞추기 위해 통합한다.
- 아직 변형이 충분하지 않은데 미래 확장성을 이유로 trait/strategy/registry를 추가한다.
- 클라이언트 계약 변경 없이 내부 DTO를 쪼개 adapter만 늘린다.
- 게임 룰, 보상, 실패, 밸런스, UX 의미를 사용자 확인 없이 임의 확정한다.

레거시 원칙:

- legacy behavior를 살리기 위한 compatibility layer, fallback path, dual schema는 기본적으로 만들지 않는다.
- 정말 필요하면 이유와 제거 조건을 이 문서에 기록하고 사용자 확인을 받는다.
- 새 정책과 충돌하는 테스트는 기대값만 바꾸지 않고 삭제하거나 최신 정책 테스트로 교체한다.
- ignored test로 레거시를 보존하지 않는다.

## Source Of Truth

항상 아래 순서로 확인한다.

1. 실제 runtime code
2. live RON/data: `../game_resources/data`
3. Unity-facing snapshot/command/result 계약: `/mnt/f/unity projects/ark/docs`
4. 최신 정책 문서: `docs/game_rulebook.md`, `docs/skill_target_contract.md`, `docs/refactor_preparation_plan.md`

문서와 코드가 충돌하면 코드를 먼저 확인한다.

문서보다 더 나은 개선안이 코드 증거로 보이면 근거를 기록하고 적용한다. 단, Unity-facing DTO shape, live RON schema, 저장 데이터 migration, UX 의미 변화, 밸런스 기준, 기존 콘텐츠 삭제/대체, 실패/보상/소비 시점 변화처럼 사용자 결정이 필요한 정책은 임의로 확정하지 않는다. 이런 항목은 즉시 질문하지 않고 해당 컴포넌트의 refactor 문서에 `사용자와 정책 논의 필요`라고 명시한 뒤 다음 감사 작업을 계속한다.

## Component Map

| # | Component | Primary code | Audit focus |
| --- | --- | --- | --- |
| 1 | 월드/런 진행 시스템 | `src/game/world.rs`, `src/game/world/*` | 런 시작, 직원 선택, 맵 보기, 노드 진입/완료, 전투 결과 처리, Shop/Reward/Support/Maintenance 흐름 |
| 2 | 맵/노드 시스템 | `src/game/map/*` | Act 맵 생성, 노드 상태, payload, 진행 가능 노드, 노드 소비/완료 |
| 3 | 행동/상태 게이트 시스템 | `src/game/behavior.rs`, `src/game/managers/action_scheduler.rs`, `src/game/world/helpers.rs` | `PlayerBehavior`, `ActionKind`, allowed actions, payload validation, command result |
| 4 | 전투 런타임 시스템 | `src/game/battle/core/*` | `BattleCore`, 이벤트 큐, live battle tick, 배치/철수/스킬/후퇴/일시정지/배속, timeline/event log |
| 5 | 전장/시나리오/웨이브 시스템 | `src/game/combat_preview/*`, `src/game/combat_setup/*`, `src/game/battle/scenario.rs`, `src/game/wave_resolution.rs` | 전투 미리보기, battlefield instance, deployment zone, route, spawn wave, PVE encounter handoff |
| 6 | 이동/저지 시스템 | `src/game/battle/core/movement/*` | route-following, world position, Rapier/Direct backend, blocking engagement, movement events |
| 7 | 스킬/타겟팅/범위 시스템 | `src/game/ability.rs`, `src/game/data/skill_data.rs`, `src/game/battle/tile_range.rs`, `src/game/range_preview.rs`, `src/game/battle/core/skill_runtime/*` | skill definition, `defense_tile_range`, `TileArea`, projectile skill, cast target, range preview |
| 8 | 기본 공격/투사체/공격 판정 시스템 | `src/game/battle/core/basic_attack.rs`, `src/game/battle/core/commands.rs`, `src/game/battle/core/targeting.rs`, `src/game/battle/core/target_usefulness.rs` | 기본 공격 스케줄링, 타겟 선택, target usefulness, projectile hit/miss, 피해 적용 |
| 9 | 피해/스탯/전투 진입 스탯 시스템 | `src/game/stats.rs`, `src/game/battle/damage.rs`, `src/game/battle/stat_pipeline.rs`, `src/game/employee.rs` | 성장, 장비, 파편, 소모품, artifact, damage modifier, 최종 전투 스탯 계산 |
| 10 | 버프/상태이상 시스템 | `src/game/battle/buffs.rs`, `src/game/battle/validation/buffs.rs`, BattleCore active buff runtime | poison/stun/freeze/silence, reapply policy, hard CC validation |
| 11 | 직원/성장/신뢰도 시스템 | `src/game/employee.rs`, `src/game/growth.rs`, `src/game/employee_trust.rs` | 직원 프로필, HP/트라우마, 성장 stack, 신뢰도 |
| 12 | 아이템/장비/스킬 파편 경제 시스템 | `src/game/resources/inventory.rs`, `src/game/resources/item_slot.rs`, `src/game/skill_fragment.rs`, `src/game/world/maintenance.rs` | 장비 장착/해제, 소모품, artifact, 파편 장착/강화/개화/분쇄, Maintenance 작업 |
| 13 | 보상/상점/지원 노드 시스템 | `src/game/reward.rs`, `src/game/reward_policy.rs`, `src/game/world/reward.rs`, `src/game/world/shop.rs`, `src/game/world/support.rs` | reward option, shop purchase/sell/reroll, Medical/Rest, HeadquartersContact 일부 |
| 14 | 데이터/RON 로딩 및 검증 시스템 | `src/game/data/*`, `tests/ron_loading.rs`, `../game_resources/data/*` | live RON schema, per-database indexes, cross-reference validation, generated preview validation |
| 15 | Unity/server-facing DTO/계약 표면 | `src/game/behavior.rs`, `src/game/world/snapshot.rs`, battle timeline/event DTOs | Unity snapshot, command result, battle setup/update/range preview/admin catalog |
| 16 | 검증/테스트 하네스 | `src/game/battle/validation/*`, `tests/*`, `src/game/world/tests/*` | battle event validation, live RON loading, skill test suite, world flow tests |

## Initial Audit Priority

초기 우선순위:

1. 스킬/타겟팅/범위 시스템
2. 기본 공격/투사체/공격 판정 시스템
3. 이동/저지 시스템
4. 피해/스탯/전투 진입 스탯 시스템
5. 아이템/장비/스킬 파편 경제 시스템
6. 월드/행동 게이트/노드 흐름
7. 데이터/RON 검증 시스템
8. Unity/server-facing DTO/계약 표면

이 순서는 고정이 아니다. 실제 코드를 읽는 도중 더 큰 source-of-truth 중복이나 선행 의존성이 확인되면 갱신한다.

## Progress

| # | Component | Refactor document | Status |
| --- | --- | --- | --- |
| 1 | 월드/런 진행 시스템 | `world_run_progression_refactor.md` | Initial audit documented |
| 2 | 맵/노드 시스템 | `map_node_refactor.md` | Initial audit documented |
| 3 | 행동/상태 게이트 시스템 | `behavior_state_gate_refactor.md` | Initial audit documented |
| 4 | 전투 런타임 시스템 | `battle_runtime_refactor.md` | Initial audit documented |
| 5 | 전장/시나리오/웨이브 시스템 | `battlefield_scenario_wave_refactor.md` | Initial audit documented |
| 6 | 이동/저지 시스템 | `movement_blocking_refactor.md` | Initial audit documented |
| 7 | 스킬/타겟팅/범위 시스템 | `skill_targeting_range_refactor.md` | Initial audit documented |
| 8 | 기본 공격/투사체/공격 판정 시스템 | `basic_attack_projectile_judgement_refactor.md` | Initial audit documented |
| 9 | 피해/스탯/전투 진입 스탯 시스템 | `damage_stats_entry_refactor.md` | Initial audit documented |
| 10 | 버프/상태이상 시스템 | `buff_status_effect_refactor.md` | Initial audit documented |
| 11 | 직원/성장/신뢰도 시스템 | `employee_growth_trust_refactor.md` | Initial audit documented |
| 12 | 아이템/장비/스킬 파편 경제 시스템 | `item_equipment_skill_fragment_refactor.md` | Initial audit documented |
| 13 | 보상/상점/지원 노드 시스템 | `reward_shop_support_refactor.md` | Initial audit documented |
| 14 | 데이터/RON 로딩 및 검증 시스템 | `data_ron_validation_refactor.md` | Initial audit documented |
| 15 | Unity/server-facing DTO/계약 표면 | `unity_server_contract_refactor.md` | Initial audit documented |
| 16 | 검증/테스트 하네스 | `validation_test_harness_refactor.md` | Initial audit documented |

## Final Cross-Check

모든 컴포넌트 순회가 끝난 뒤 별도 최종 재검토를 수행한다.

재검토 기준:

- 각 `<component_name>_refactor.md`의 code evidence가 실제 파일과 라인 수준에서 여전히 맞는지 확인한다.
- 리팩토링 후보가 실제 runtime path에 영향을 주는지, dead code나 테스트 전용 코드만 근거로 삼지 않았는지 확인한다.
- source-of-truth 판단이 live RON/data, Unity-facing 계약, 최신 정책 문서와 충돌하지 않는지 확인한다.
- 기존 기능 조합으로 단순화 가능하다고 적은 항목이 gameplay 의미, 이벤트/DTO, 검증 관점에서도 동등한지 다시 확인한다.
- `사용자와 정책 논의 필요` 항목이 임의 구현 후보처럼 적혀 있지 않은지 확인한다.
- 한 컴포넌트 문서의 제안이 다른 컴포넌트 문서의 source-of-truth 판단과 충돌하지 않는지 확인한다.

재검토 결과:

- 타당한 항목은 유지한다.
- 근거가 부족한 항목은 보류 또는 삭제로 바꾼다.
- 코드와 어긋난 항목은 수정하고 `EXPERIMENTS.md`에 불일치와 정정 내용을 기록한다.
- 컴포넌트 간 충돌이 있으면 관련 문서 모두에 cross-reference를 남긴다.

### 2026-06-21 Cross-Check Result

수행한 대조:

- 16개 component refactor 문서가 모두 존재하는지 확인했다.
- `Progress` 표가 모든 컴포넌트를 `Initial audit documented`로 표시하는지 확인했다.
- 모든 component refactor 문서가 `docs/refactor_preparation_plan.md`를 기준으로 언급하는지 확인했다.
- 반복 후보를 실제 코드/데이터/Unity 문서에 다시 대조했다.

확인된 반복 후보와 연결 문서:

- Live RON loader 중복: `data_ron_validation_refactor.md`, `validation_test_harness_refactor.md`.
- Starter skill fragment code injection: `data_ron_validation_refactor.md`, `item_equipment_skill_fragment_refactor.md`, `validation_test_harness_refactor.md`.
- Debug event log export side effect: `world_run_progression_refactor.md`, `validation_test_harness_refactor.md`, `docs/refactor_preparation_plan.md`.
- Unity live battle transport ownership: `battle_runtime_refactor.md`, `movement_blocking_refactor.md`, `unity_server_contract_refactor.md`, `validation_test_harness_refactor.md`.
- Skill range/preview source-of-truth: `skill_targeting_range_refactor.md`, `basic_attack_projectile_judgement_refactor.md`, `unity_server_contract_refactor.md`.
- Reward data/effect/tag policy: `reward_shop_support_refactor.md`, `data_ron_validation_refactor.md`, `validation_test_harness_refactor.md`.

정정 결과:

- 최종 대조에서 코드와 명백히 어긋나는 component 문서는 발견하지 못했다.
- `사용자와 정책 논의 필요` 항목은 구현 지시가 아니라 정책 결정 후보로 유지되어 있음을 확인했다.
- 테스트/검증은 문서 감사만 수행했기 때문에 실행하지 않았다.

## Completion Conditions

- component map의 모든 컴포넌트를 직접 읽고 감사한다.
- 각 컴포넌트마다 `<component_name>_refactor.md` 문서를 만들거나 갱신한다.
- 각 컴포넌트 refactor 문서는 `docs/refactor_preparation_plan.md`를 리팩토링 판단 기준으로 삼았음을 명시한다.
- 모든 컴포넌트 순회 후 생성된 refactor 문서와 실제 코드, live RON/data, Unity-facing 계약, 테스트를 꼼꼼히 비교/대조하는 최종 재검토를 수행한다.
- 리팩토링 후보는 code evidence, source-of-truth 판단, 기존 기능 조합 가능성, 검증 방법을 포함한다.
- 리팩토링 후보가 없는 컴포넌트도 "후보 없음"과 그 근거를 문서에 남긴다.
- 적용한 변경이 있다면 작은 변경 단위마다 focused test 또는 `cargo check`를 돌리고, 마지막에 넓은 테스트를 돌린다.
- 테스트 실패는 `EXPERIMENTS.md`에 실패 원인, 수정 내용, 재검증 결과를 기록한다.
- 완료 시 변경 요약, 제거한 레거시, 새로 고정한 계약, 갱신한 테스트, 남은 위험, 실행한 검증 명령을 보고한다.
- 정책 판단이 필요한 항목은 임의로 구현하지 않고 해당 컴포넌트 refactor 문서에 `사용자와 정책 논의 필요`라고 명시한다.
- 정책 논의 필요 항목이 있어도 즉시 goal을 종료하지 않고, 문서화한 뒤 다음 컴포넌트 감사를 계속한다.

## Stop Conditions

- 기존 dirty worktree 변경을 되돌려야만 진행할 수 있다.
- 도구, 빌드, 테스트 환경 문제로 더 이상 코드를 읽거나 문서화할 수 없다.
- 사용자와 정책 논의가 필요한 항목은 stop condition이 아니라 문서화 대상이다.
