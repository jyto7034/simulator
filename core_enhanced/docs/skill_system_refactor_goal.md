# Skill System Refactor Goal

이 goal은 현재 스킬 시스템을 장기적으로 안전하게 확장할 수 있도록 문서 계약, 데이터 검증, Unity-facing 스킬 표시 계약, 레거시 데이터 중복을 정리하기 위한 리팩토링 계획서다.

목적은 새 스킬 런타임을 만드는 것이 아니다. 현재 `SkillDef -> SkillFragment -> EmployeeProfile -> BattleCore runtime` 흐름을 유지하면서, source of truth 중복과 늦은 검증을 줄이고, Unity가 스킬 정보를 추론하지 않아도 되게 만드는 것이다.

## Goal Mode Working Method

이 문서는 `C:\Users\blast\Desktop\using-codex-goals-effectively-ko.md`의 goal 운용 원칙을 따른다.

작업자는 goal 시작 시 아래 작업 기억장치를 만들고 계속 갱신한다.

```text
docs/goals/skill_system_refactor/PLAN.md
docs/goals/skill_system_refactor/EXPERIMENTS.md
docs/goals/skill_system_refactor/EXPERIMENT_NOTES.md
```

각 파일의 역할:

- `PLAN.md`: 구현 순서, 현재 판단, 남은 체크리스트, 완료 조건을 기록한다.
- `EXPERIMENTS.md`: 시도한 접근, 실패/성공 결과, 테스트 실패 원인과 해결을 기록한다.
- `EXPERIMENT_NOTES.md`: 작업 중 발견한 의심, 정책 질문, 기술 부채 후보를 시간순으로 기록한다.

이 파일들은 최종 정책 문서가 아니라 goal 실행 중의 작업 기억장치다. goal 완료 후 유지해야 할 내용만 `game_rulebook.md`, `skill_target_contract.md`, `unity_core_contract.md`로 옮긴다.

## Engineering Principles

- 코드 수정은 장기적인 방향으로 한다. 임시방편, 최소 수정, 특정 테스트만 맞추는 패치는 피한다.
- 처음에는 어떤 구조가 장기적인 방향인지 확실하지 않을 수 있다. 작은 단위로 trial and error를 수행하고, 실패한 접근과 이유를 `EXPERIMENTS.md`에 남긴다.
- 레거시는 과감하게 제거한다. 과거 스킬 타겟팅, TFT식 범위, generated RON 중복을 compatibility layer로 감싸지 않는다.
- 문서를 무조건 신뢰하지 않는다. 실제 코드와 live RON/API를 읽으면서 더 나은 개선안이 있으면 근거를 기록하고 적용한다.
- 문서에 적힌 내용보다 코드에서 더 단순하고 안전한 개선안이 보이면, 왜 더 나은지 기록한 뒤 적용한다. 단, 정책 판단이 필요한 경우 사용자에게 질문하고 goal을 종료한다.
- 새 추상화는 실제 반복이 확인된 뒤 만든다. 단순 validation function/table로 충분한 곳에 trait, strategy, resolver 계층을 먼저 만들지 않는다.
- 테스트는 내부 구조가 아니라 실제 스킬 사용 흐름, 데이터 로딩 실패, Unity-facing 계약을 고정한다.
- 정책 변경으로 실패하는 테스트가 과거 geometric AoE, 구형 스킬 타겟팅, TFT식 범위, generated RON 호환 같은 레거시 동작을 고정하고 있다면 업데이트보다 삭제를 우선한다. 최신 정책을 검증하는 새 focused test로 교체하고, 레거시 테스트를 ignored 처리해 보존하지 않는다.

## Source Of Truth

우선 읽을 파일:

- `src/game/ability.rs`: `SkillDef`, `SkillTarget`, `SkillStepDef`, `SkillEffectDef`, activation/delivery 계약.
- `src/game/data/skill_data.rs`: `SkillDatabase` validation.
- `src/game/data/skill_fragment_data.rs`: fragment effect metadata.
- `src/game/data/mod.rs`: cross-database validation.
- `src/game/skill_fragment.rs`: active fragment 장착, 성장, 직원 combat profile 적용.
- `src/game/data/abnormality_data.rs`: 환상체 원본 스킬 참조.
- `src/game/data/corroded_employee_data.rs`: 침식 직원 스킬 참조.
- `src/game/battle/tile_range.rs`: DefenseRoute tile range pattern.
- `src/game/battle/core/targeting.rs`: 기본 공격/스킬 range 판정.
- `src/game/battle/core/sim.rs`: manual/auto skill activation, skill step execution.
- `src/game/battle/core/commands.rs`: skill damage command 처리.
- `src/game/battle/buffs.rs`: buff registry.
- `src/game/world/combat.rs`: DefenseRoute deploy-time skill contract validation.
- `src/game/world/snapshot.rs`: Unity-facing employee/skill snapshot.
- `src/game/behavior.rs`: `ActivateSkill` command/result contract.
- `../game_resources/data/skills/base.ron`: live skill data.
- `../game_resources/data/skill_fragments/base.ron`: live fragment data.
- `docs/game_rulebook.md`: 스킬/파편/DefenseRoute 정책.
- `docs/skill_target_contract.md`: skill targeting 계약.
- `docs/unity_core_contract.md`: Unity command/snapshot/result 계약.

필요하면 `../game_server`의 `/game` request/result mapping도 확인한다. server mapping 변경이 필요하면 core 계약과 함께 갱신한다.

## Current Findings

현재 검토에서 확인한 문제:

- `docs/skill_target_contract.md`는 타일 기반 스킬 범위를 제거된 레거시처럼 설명하지만, 현재 공식 전투인 `DefenseRoute`에서는 명일방주식 tile range가 필수다.
- 직원 active fragment skill의 `defense_tile_range` 계약은 배치 시점에만 검증된다. 잘못된 스킬을 장착하거나 Unity에 표시한 뒤 deploy에서 실패할 수 있다.
- `AbnormalityMetadata.skill_id`, `CorrodedEmployeeProfileMetadata.skill_id`가 실제 `SkillDatabase`에 존재하는지 교차 검증되지 않는다.
- `ModifyDamage`는 같은 step의 `Damage`에만 적용되는 step-local modifier지만, 독립 효과처럼 보여 RON 작성자가 오해하기 쉽다. `Damage` 없이 쓰면 조용히 no-op 된다.
- Unity-facing snapshot은 `effective_skill_id` 중심이라 스킬 이름, 발동 방식, 게이지, tile range, target policy, 활성 가능 여부를 추론해야 한다.
- `buffs.rs`의 buff registry가 하드코딩되어 있어 컨텐츠 확장 시 Rust 코드 수정이 반복될 수 있다.
- `../game_resources/data/skills/base.generated.ron`은 실제 live 로드 경로가 아니어서 `legacy_base.generated.ron`으로 분리했다. 공식 source of truth는 `base.ron`이다.
- 수동 스킬은 cast 시작 후 대상이 사라져도 종료 시점에 게이지가 소모될 수 있다. 의도한 정책이면 문서화가 필요하다.
- 스킬 피해 command path와 기본공격 피해 path가 분리되어 있다. 향후 `DefenseMitigation` 같은 효과를 붙일 때 중앙 피해 처리 정책을 확인해야 한다.
- 현재 `DeliveryDef::Area`는 `Circle`, `Line`, `Box`, `Rectangle`, `Cone` 같은 연속좌표 shape로 실제 AoE 피격 대상을 수집한다. 이는 DefenseRoute의 명일방주식 tile-based 스킬 범위 정책과 맞지 않는다.

## Objective

완료 후 다음이 가능해야 한다.

1. DefenseRoute tile range 계약이 코드와 문서에서 일관된다.
2. 직원 active fragment skill, 환상체 skill, 침식 직원 skill의 참조 오류가 data load 단계에서 빠르게 실패한다.
3. `ModifyDamage`의 의미가 코드, RON validation, 문서에서 명확하다.
4. Unity가 스킬 UI를 만들 때 스킬 표시/발동 정보를 추론하지 않아도 된다.
5. live skill RON source of truth가 명확하고, 사용하지 않는 generated/legacy 데이터가 공식 데이터처럼 남지 않는다.
6. DefenseRoute에서 `defense_tile_range`가 표시 범위, 시전 가능 범위, 타겟 후보 범위, 실제 피격 범위의 단일 source of truth가 된다.
7. 새 추상화 없이 현재 스킬 런타임을 유지하면서 검증과 계약만 단단해진다.

## In Scope

- `skill_target_contract.md`와 `game_rulebook.md`의 DefenseRoute tile range 설명 정리.
- `SkillDatabase` 또는 `GameDataBase` 단계의 cross-database validation 강화.
- active skill fragment가 참조하는 player skill의 DefenseRoute 계약 검증 강화.
- abnormality/corroded employee skill_id 존재 검증 추가.
- `ModifyDamage` no-op 방지 validation 또는 명칭/문서 정리.
- Unity-facing snapshot 또는 별도 static skill contract에 필요한 스킬 표시 정보 추가.
- live RON에서 쓰지 않는 generated/legacy 데이터가 공식 source of truth처럼 보이지 않게 유지.
- DefenseRoute 공식 스킬에서 geometric `Area(shape: Circle/Line/Box/Rectangle/Cone)` 의존 제거.
- `defense_tile_range` 안의 타일에 속한 유닛을 실제 스킬 피격 대상으로 수집하는 runtime 추가.
- 정책 변경으로 깨지는 geometric AoE/구형 타겟팅/legacy compatibility tests 삭제 또는 최신 focused tests로 교체.
- focused tests와 문서 갱신.

## Out Of Scope

- 새 스킬 런타임 작성.
- 투사체, buff, tile range 알고리즘 전체 재작성.
- 스킬 밸런스 수치 조정.
- 신규 환상체/스킬 컨텐츠 대량 추가.
- Unity 클라이언트 UI 구현.
- 이동 시스템 내부 리팩토링.
- 조건부 BattleScenario 이벤트 구현.

## Implementation Plan

1. 문서/코드 계약 재확인

- `skill_target_contract.md`, `game_rulebook.md`, `unity_core_contract.md`가 현재 DefenseRoute 단일 전투 정책과 맞는지 확인한다.
- `TileRangePattern`, `SkillCastTargetingDef`, `SkillStepDef.defense_tile_range`의 실제 사용처를 다시 읽는다.
- DefenseRoute에서 `defense_tile_range`는 표시 범위, 시전 가능 범위, 타겟 후보 범위, 실제 피격 범위가 모두 같은 계약임을 문서에 명확히 쓴다.
- Rapier2D/연속좌표는 이동, 충돌, 투사체 보조에 사용하되, DefenseRoute 스킬 AoE 피격 판정의 source of truth로 쓰지 않는다.

2. 데이터 검증 강화

- `GameDataBase::validate_skill_fragment_skill_references`를 확장하거나 별도 validation function을 추가한다.
- active fragment의 `imitation_skill_id`, `upgrade_skill_ids`, `awakened_skill_id`가 DefenseRoute player skill로 사용할 수 있는지 검증한다.
- `AbnormalityMetadata.skill_id`가 존재하는지 검증한다.
- `CorrodedEmployeeProfileMetadata.skill_id`가 존재하는지 검증한다.
- 검증 실패 메시지는 어떤 데이터 id와 어떤 skill_id가 문제인지 명확히 출력한다.

3. `ModifyDamage` 계약 정리

- `ModifyDamage`가 같은 step 안의 `Damage`에만 적용되는 step-local modifier인지 확인한다.
- 정책이 명확하면 `SkillDatabase` validation에서 `ModifyDamage`만 있고 `Damage`가 없는 step을 reject한다.
- 이름 변경이 client/RON schema에 큰 영향을 주면 먼저 문서와 validation만 강화한다.
- `ModifyDamage`를 독립 지속 효과로 바꾸는 것은 별도 정책이므로 이번 goal에서 임의 구현하지 않는다.

4. DefenseRoute 스킬 범위 단일화

- `SkillStepDef.defense_tile_range`를 DefenseRoute 스킬 범위의 단일 source of truth로 사용한다.
- 단일 타겟 스킬은 `defense_tile_range` 안의 유효 대상 중 하나를 선택한다.
- 범위 스킬은 `defense_tile_range` 안의 유효 대상 전부에게 효과를 적용한다.
- 힐/버프 범위 스킬도 같은 방식으로 `hit_targets` 또는 target policy에 맞는 유닛만 수집한다.
- 현재 `DeliveryDef::Area { shape: Circle/Line/Box/Rectangle/Cone }`는 DefenseRoute 공식 live skill에서 제거하거나 validation 실패로 처리한다.
- `DeliveryDef`에는 geometric shape가 아니라 “범위 내 전체 대상에게 적용”을 표현하는 단순한 계약을 둔다. 이름은 코드 판독 후 가장 단순한 쪽을 선택하되, 과도한 새 추상화는 피한다.
- persistent area가 필요한 경우에도 tick마다 geometric shape가 아니라 `defense_tile_range` tile membership으로 대상을 다시 수집한다.
- target lost 정책은 수동 스킬 시작 후 대상이 사라져도 게이지를 소모하는 현재 정책을 유지한다. 단, 실제 피격 대상이 없으면 효과는 발생하지 않는다.

5. Unity-facing skill metadata 보강

- 현재 snapshot이 `effective_skill_id`만 내려주는지 확인한다.
- Unity가 스킬 버튼과 range preview를 만들기 위해 필요한 최소 정보를 정한다.
- 권장 정보는 `skill_id`, `display_name`, `activation_mode`, `resonance_current`, `resonance_max`, `target_policy`, `defense_tile_range`, `manual_activation_allowed`, `can_activate_reason`이다.
- 정적 정보는 skill catalog, 현재 충전량/사용 가능 여부는 snapshot으로 제공하는 하이브리드 방식을 기본값으로 한다.
- Unity-facing 계약 변경은 `unity_core_contract.md`와 focused JSON shape test에 반영한다.

6. 레거시/중복 RON 정리

- `../game_resources/data/skills/base.generated.ron`이 실제 로드되는지 확인한다.
- 로드되지 않고 공식 데이터처럼 보이는 중복이면 `legacy` 또는 `archive` 이름으로 분리하거나 제거한다.
- live loader, tests, docs가 해당 파일을 참조하지 않도록 정리한다.

7. Buff registry RON화

- buff registry는 RON 기반 데이터로 전환한다.
- 기존 hard-coded buff는 live RON의 초기 데이터로 옮긴다.
- skill validation은 Rust hard-coded registry가 아니라 loaded buff database를 기준으로 unknown buff를 검증한다.
- 단, 복잡한 buff scripting runtime을 새로 만들지 않는다. 이번 goal은 buff id/kind/duration/tick 같은 기존 registry 정보를 data-driven으로 옮기는 데 집중한다.

8. 피해 처리 경로 점검

- skill damage와 basic attack damage가 어떤 modifier path를 공유하는지 확인한다.
- `DefenseMitigation` 같은 incoming damage effect를 붙일 때 basic attack과 skill damage 모두에 적용될 수 있는 최소 변경점을 기록한다.
- 이번 goal에서 소비 아이템 효과 구현으로 확장하지 않는다. 필요한 경우 별도 goal로 분리한다.

## Test Requirements

focused test를 먼저 작성/갱신한다.

- skill fragment가 존재하지 않는 skill_id를 참조하면 `GameDataBase` 생성이 실패한다.
- abnormality가 존재하지 않는 skill_id를 참조하면 `GameDataBase` 생성이 실패한다.
- corroded employee가 존재하지 않는 skill_id를 참조하면 `GameDataBase` 생성이 실패한다.
- player active fragment skill이 DefenseRoute에 필요한 `defense_tile_range`를 갖지 않으면 data validation이 실패한다.
- `ModifyDamage`가 같은 step에 `Damage` 없이 작성되면 validation이 실패한다.
- DefenseRoute 범위 스킬은 `defense_tile_range`에 포함된 타일의 유닛만 피격한다.
- DefenseRoute 범위 스킬은 geometric `Circle/Cone/Box/Rectangle/Line` shape로 피격 대상을 수집하지 않는다.
- live skill RON의 공식 DefenseRoute 스킬이 geometric Area를 사용하면 validation이 실패하거나 TileRange 기반 delivery로 마이그레이션되어야 한다.
- buff registry RON을 로드하고, unknown buff id를 참조하는 skill은 validation에서 실패한다.
- Unity snapshot 또는 skill contract JSON에 수동 스킬 표시/발동에 필요한 metadata가 포함된다.
- `skill_target_contract.md`의 핵심 문장이 최신 정책과 일치한다.
- 기존 테스트가 최신 DefenseRoute tile-based 스킬 정책과 충돌하면 기대값만 낮춰서 통과시키지 않는다. 레거시 정책을 고정하는 테스트는 삭제하고, 최신 정책을 검증하는 테스트로 교체한다.

검증 순서:

```text
cargo test -p game_core skill -- --nocapture
cargo test -p game_core skill_fragment -- --nocapture
cargo test -p game_core employee_roster_snapshot -- --nocapture
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

- Unity skill metadata를 snapshot에 넣을지 static catalog로 분리할지 코드 근거만으로 결정하기 어렵다.
- `ModifyDamage`를 단순 validation으로 막을지, schema 이름을 바꿀지 정책 결정이 필요하다.
- DefenseRoute 범위 스킬의 “범위 내 전체 대상”을 표현할 `DeliveryDef` 이름/schema가 기존 client/RON과 크게 충돌한다.
- persistent tile area에서 anchor/facing/tracking을 어떻게 해석해야 할지 코드 근거만으로 결정하기 어렵다.
- geometric Area가 DefenseRoute가 아닌 적 전용/특수 전투 경로에서 실제로 필요하다는 live 근거가 발견된다.
- `base.generated.ron`이 외부 도구나 Unity 작업에서 아직 필요하다는 근거가 발견된다.
- 사용자와 의논하여 정해야 할 정책이 있을 경우 goal을 종료한다.

## Completion Criteria

1. DefenseRoute tile range 계약이 `skill_target_contract.md`, `game_rulebook.md`, 코드에서 일관된다.
2. skill fragment, abnormality, corroded employee의 skill_id 참조 오류가 data load 단계에서 실패한다.
3. 직원 active fragment skill의 DefenseRoute range 계약이 deploy 이전에 검증된다.
4. `ModifyDamage` no-op 오해가 validation 또는 문서로 제거된다.
5. Unity가 수동 스킬 UI를 표시하기 위한 최소 skill metadata/readiness를 추론 없이 얻을 수 있다.
6. DefenseRoute 스킬 범위는 `defense_tile_range`가 표시/시전/타겟/피격 범위의 단일 source of truth가 된다.
7. geometric Area shape는 DefenseRoute 공식 live skill 경로에서 제거된다.
8. buff registry가 RON 기반으로 전환되고 unknown buff validation이 loaded data 기준으로 동작한다.
9. 사용하지 않는 generated/legacy skill RON이 공식 live source of truth처럼 남지 않는다.
10. 필요한 focused tests와 문서가 갱신된다.
11. `cargo test -p game_core -- --nocapture`, `cargo check -p game_core`, `cargo check -p game_server`가 통과한다.
12. game_server mapping을 변경했다면 `cargo test -p game_server -- --nocapture`가 통과한다.
13. 사용자와 의논하여 정해야 할 정책이 있을 경우 goal을 종료한다.

## Goal Command

```text
/goal docs/skill_system_refactor_goal.md를 기준으로, 현재 스킬 시스템의 문서 계약, 데이터 검증, Unity-facing 스킬 표시 계약, DefenseRoute tile-based 스킬 범위, 레거시 RON 중복을 정리하라. 먼저 src/game/ability.rs, src/game/data/skill_data.rs, src/game/data/skill_fragment_data.rs, src/game/data/mod.rs, src/game/skill_fragment.rs, src/game/data/abnormality_data.rs, src/game/data/corroded_employee_data.rs, src/game/battle/tile_range.rs, src/game/battle/core/targeting.rs, src/game/battle/core/sim.rs, src/game/battle/core/commands.rs, src/game/battle/buffs.rs, src/game/world/combat.rs, src/game/world/snapshot.rs, src/game/behavior.rs, ../game_resources/data/skills/base.ron, ../game_resources/data/skill_fragments/base.ron, docs/game_rulebook.md, docs/skill_target_contract.md, docs/unity_core_contract.md를 꼼꼼히 읽고 현재 계약을 재확인하라. DefenseRoute에서는 명일방주식 tile range가 공식 스킬/평타 범위 계약이며, SkillStep.defense_tile_range가 표시 범위, 시전 가능 범위, 타겟 후보 범위, 실제 피격 범위의 단일 source of truth가 되어야 한다. 단일 타겟 스킬은 defense_tile_range 안의 유효 대상 중 하나를 선택하고, 범위 스킬은 defense_tile_range 안의 유효 대상 전부에게 효과를 적용하게 하라. 기존 DeliveryDef::Area의 Circle/Line/Box/Rectangle/Cone 같은 geometric shape는 DefenseRoute 공식 live skill 경로에서 제거하거나 validation 실패로 처리하고, Unity timeline/contract도 tile pattern 또는 affected tiles를 기준으로 갱신하라. skill fragment active skill, abnormality skill_id, corroded employee skill_id는 data load 단계에서 SkillDatabase와 교차 검증되게 하라. 직원 active fragment skill은 deploy 이전에 DefenseRoute에 필요한 defense_tile_range 계약을 만족하는지 검증하라. ModifyDamage가 같은 step의 Damage에만 적용되는 step-local modifier라면 Damage 없이 작성된 step을 validation에서 막거나 문서/명칭을 명확히 하라. Unity가 수동 스킬 버튼과 range preview를 추론 없이 만들 수 있도록 정적 skill 정보는 catalog, 현재 충전량/사용 가능 여부는 snapshot으로 제공하는 하이브리드 metadata/readiness 계약을 보강하라. buff registry는 RON 기반 데이터로 전환하되 복잡한 buff scripting runtime은 새로 만들지 말고 기존 registry 정보를 data-driven으로 옮기는 데 집중하라. 사용하지 않는 generated/legacy skill RON이 공식 source of truth처럼 남아 있다면 compatibility layer 없이 정리하라. 정책 변경으로 실패하는 테스트가 geometric AoE, 구형 스킬 타겟팅, TFT식 범위, generated RON 호환 같은 레거시 동작을 고정한다면 ignored 처리로 보존하지 말고 삭제하거나 최신 focused test로 교체하라. 코드 수정은 장기적인 방향으로 하고 임시방편을 피하라. 처음 구조 판단이 어려우면 작은 trial and error를 수행하고 docs/goals/skill_system_refactor/PLAN.md, EXPERIMENTS.md, EXPERIMENT_NOTES.md에 기록하라. 문서를 무조건 신뢰하지 말고 실제 코드와 live RON/API를 읽으면서 더 나은 개선안이 보이면 근거를 기록하고 적용하라. 단, DeliveryDef schema 이름/구조, persistent tile area anchor/facing/tracking 해석, geometric Area가 DefenseRoute 외 live 경로에서 실제로 필요한 경우, base.generated.ron 외부 의존처럼 사용자와 의논해야 할 정책이 발견되면 goal을 종료하고 질문 목록을 보고하라. 검증은 focused tests를 먼저 실행한 뒤 cargo test -p game_core -- --nocapture, cargo check -p game_core, cargo check -p game_server를 수행하고, game_server mapping을 변경했다면 cargo test -p game_server -- --nocapture도 수행하라. 완료 조건은 1) DefenseRoute tile range 계약이 문서와 코드에서 일관됨, 2) skill_id 참조 오류가 data load 단계에서 실패함, 3) active fragment skill range 계약이 deploy 이전에 검증됨, 4) ModifyDamage no-op 오해가 제거됨, 5) Unity skill metadata/readiness 계약이 보강됨, 6) DefenseRoute 스킬 범위가 defense_tile_range 단일 source of truth로 동작함, 7) geometric Area shape가 DefenseRoute 공식 live skill 경로에서 제거됨, 8) buff registry가 RON 기반으로 전환됨, 9) legacy/generated skill RON 중복과 레거시 테스트가 정리됨, 10) 필요한 문서와 tests가 갱신됨, 11) 검증 명령이 통과함, 12) 사용자와 의논하여 정해야 할 정책이 있을 경우 goal을 종료함이다.
```
