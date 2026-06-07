# BuffDatabase And Consumable Modifier Goal

## Goal Mode Working Method

이 goal은 `C:\Users\blast\Desktop\using-codex-goals-effectively-ko.md`의 원칙을 따른다.

- 목표는 완료 여부를 판단할 수 있는 체크리스트와 검증 명령으로 정의한다.
- 큰 변경을 한 번에 밀어붙이지 않고, focused test로 작은 피드백 루프를 먼저 만든다.
- 작업 중 `docs/goals/buff_database_and_consumable_modifier/PLAN.md`, `EXPERIMENTS.md`, `EXPERIMENT_NOTES.md`를 유지한다.
- `PLAN.md`: 구현 순서, 현재 판단, 남은 체크리스트, 완료 조건을 기록한다.
- `EXPERIMENTS.md`: 시도한 접근, 결과, 실패 이유, 채택/폐기 근거를 기록한다.
- `EXPERIMENT_NOTES.md`: 진행 중 떠오른 설계 판단, 위험, 후속 질문을 시간순으로 기록한다.

## Objective

전투 상태이상 buff와 아이템 사용 효과의 경계를 명확히 하고, `BuffDatabase`의 source of truth를 `GameDataBase` 흐름으로 통합한다.

현재 정책:

- 전투 상태이상 buff는 `poison`, `stun`, `freeze`, `silence`처럼 전투 중 실시간으로 적용되는 효과다.
- 아이템 사용 효과는 전투 전 직원에게 적용되는 런/다음 전투 modifier다.
- 전투 상태이상 buff와 아이템 사용 modifier는 수명주기와 적용 위치가 다르므로 억지로 하나의 runtime으로 합치지 않는다.
- `Stun`/`Freeze`는 hard CC로 상호배제된다.
- `Silence`는 스킬 사용만 막는 soft CC로 유지한다.
- 전투 내부 buff는 `duration_ms`, `tick_interval_ms` 기반이다.
- 아이템 사용 modifier는 `NextCombatNode`, `CombatNodes(n)` 같은 전투 노드 단위 duration을 사용한다.

## Problems

### 1. BuffDatabase Source Of Truth 분산

`src/game/battle/buffs.rs`는 기존에 `include_str! + static REGISTRY`로 `../game_resources/data/buffs/base.ron`을 직접 읽었다.

이 구조는 다음 문제가 있다.

- 다른 게임 데이터는 대부분 `GameDataBase`에 묶여 있지만 buff만 전역 static registry에 남아 있다.
- 테스트나 커스텀 `GameDataBase`에서 buff 데이터를 교체하기 어렵다.
- `skill_data` validation, timeline validation, BattleCore runtime이 모두 전역 accessor에 의존한다.
- 장기적으로 live RON/API의 source of truth를 추적하기 어렵다.

### 2. ActiveConsumableModifier 명칭 정리

`Employee.active_consumable_modifier`는 실제 전투 상태이상 buff가 아니다.

이 값은 Safezone 아이템 사용으로 직원에게 부여되는 다음 전투/런 modifier다.

현재 이름은 다음 오해를 만들 수 있다.

- BattleCore의 `BuffApplied`/`BuffTick`과 같은 시스템으로 오해할 수 있다.
- `BuffDatabase`에 넣어야 하는 효과처럼 보일 수 있다.
- Unity-facing snapshot에서 전투 상태이상과 아이템 modifier가 섞여 보일 수 있다.

장기적으로는 `ActiveConsumableModifier` 또는 동등하게 명확한 이름을 사용한다.

### 3. 아이템 modifier 적용 지점 명확화 필요

아이템 사용 효과는 이미 일부 연결되어 있다.

- `BattleHpSetup`: 전투 시작 Battle HP 보정.
- `OffenseBoost`: 전투 시작 공격력 보정.
- `InitialSkillCharge`: 전투 시작 스킬 게이지 보정.
- `DeployCostReduction`: 실시간 배치 비용 보정.
- `TraumaMitigation`: 전투불능 후 트라우마 증가량 감소.
- `RunHpLossMitigation`: 전투불능 후 Run HP 손실 감소.
- `DeathPrevent`: 사망 판정 직전 1회 생존 후 modifier 소모.

이번 goal에서는 이 흐름을 공식 계약으로 정리하고, 코드/문서/테스트 이름이 같은 정책을 말하게 만든다.

## Scope

### In Scope

- `BuffDatabase`를 `GameDataBase`/`GameDataBuilder`/`GameDataBaseParts`에 포함한다.
- `BuffDatabase` 로딩/검증을 다른 데이터베이스와 같은 흐름으로 맞춘다.
- BattleCore runtime, timeline validation, skill validation에서 가능한 한 전역 `buffs::get()`/`contains_name()` 의존을 제거한다.
- 전역 accessor가 남아야 하는 경우, live path에 남긴 이유를 `EXPERIMENTS.md`에 기록하고 compatibility layer가 되지 않게 범위를 제한한다.
- `ActiveConsumableModifier` 계열 이름을 유지하고 구 `ActiveConsumableBuff` 표현을 제거한다.
- Unity-facing snapshot과 `unity_core_contract.md`가 새 명칭과 의미를 반영한다.
- 아이템 modifier와 전투 buff의 차이를 `game_rulebook.md`에 명시한다.
- 관련 테스트는 내부 구조보다 실제 게임 흐름과 Unity-facing 계약을 고정하도록 갱신한다.

### Out Of Scope

- 새로운 buff scripting runtime 추가.
- 복잡한 상태이상 조합 룰 추가.
- 새로운 아이템 효과 대량 추가.
- 섭취 아이템 획득 밸런스 조정.
- 전투 상태이상과 아이템 modifier를 하나의 효과 엔진으로 통합.
- UI 구현.

## Required Reading

작업 전 다음 파일을 실제 코드 기준으로 읽는다. 문서를 무조건 신뢰하지 말고 코드와 live RON/API를 기준으로 판단한다.

- `src/game/battle/buffs.rs`
- `src/game/data/mod.rs`
- `src/game/data/skill_data.rs`
- `src/game/battle/core/mod.rs`
- `src/game/battle/core/sim.rs`
- `src/game/battle/core/commands.rs`
- `src/game/battle/validation/buffs.rs`
- `src/game/battle/timeline.rs`
- `src/game/employee.rs`
- `src/game/data/consumable_data.rs`
- `src/game/world/maintenance.rs`
- `src/game/world/snapshot.rs`
- `src/game/behavior.rs`
- `src/game/world/tests.rs`
- `tests/ron_loading.rs`
- `tests/skill_refactor_validation.rs`
- `tests/skill_test/**`
- `../game_resources/data/buffs/base.ron`
- `../game_resources/data/consumables/base.ron`
- `docs/game_rulebook.md`
- `docs/unity_core_contract.md`
- `docs/refactor_preparation_plan.md`

## Implementation Plan

1. 작업 기억 문서를 만든다.
   - `docs/goals/buff_database_and_consumable_modifier/PLAN.md`
   - `docs/goals/buff_database_and_consumable_modifier/EXPERIMENTS.md`
   - `docs/goals/buff_database_and_consumable_modifier/EXPERIMENT_NOTES.md`

2. 현재 호출부를 분류한다.
   - `buffs::get`
   - `buffs::contains_name`
   - `BuffId::from_name`
   - `BuffDatabase`
   - `BuffKind`
   - `ActiveConsumableModifier`

3. `BuffDatabase`를 `GameDataBase`에 통합한다.
   - `GameDataBuilder::empty()`는 빈 `BuffDatabase` 또는 live 기본값 중 어느 쪽이 테스트와 공식 데이터 흐름에 맞는지 코드 기준으로 판단한다.
   - live RON 로딩 경로에서 `../game_resources/data/buffs/base.ron`이 `GameDataBase`에 포함되게 한다.
   - 데이터 검증 시 buff id 참조 검증이 `GameDataBase.buff_data`를 기준으로 동작하게 한다.

4. BattleCore에 buff registry를 명시적으로 공급한다.
   - `BattleCore`가 이미 `game_data`를 가진다면 `game_data.buff_data`를 사용한다.
   - validator가 `GameDataBase` 없이 동작해야 한다면, 별도 `BuffRegistry` reference를 주입하는 쪽을 검토한다.
   - 단순히 새 global wrapper를 만들지 않는다.

5. 전역 accessor 제거 또는 격리.
   - live path에서 `buffs::get()`을 `game_data.buff_data.get(...)` 또는 명시 registry reference로 바꾼다.
   - test helper에서만 남기는 경우에도 이유를 기록한다.
   - legacy compatibility layer로 감싸지 않는다.

6. `ActiveConsumableModifier` 명칭을 정리한다.
   - 권장명: `ActiveConsumableModifier`.
   - 필드명 권장: `active_consumable_modifier`.
   - Unity-facing JSON 필드명을 바꾸면 클라이언트 계약 변경이므로 `unity_core_contract.md`와 테스트를 함께 갱신한다.
   - 만약 JSON 필드명 유지가 필요하다는 근거가 발견되면 사용자와 의논하기 위해 goal을 종료한다.

7. 아이템 modifier 적용 순서를 문서와 테스트로 고정한다.
   - 전투 시작 profile 보정.
   - 배치 비용 보정.
   - 전투 중 스킬/행동.
   - 전투불능 후 Trauma/Run HP 감소 보정.
   - DeathPrevent 판정.
   - 전투/보스 노드 완료 후 duration 감소.

8. 테스트를 최신 계약 중심으로 갱신한다.
   - 전역 static registry에 의존하는 테스트는 최신 `GameDataBase` 기반 테스트로 교체한다.
   - 레거시 이름 또는 구조를 고정하는 테스트는 ignored 처리하지 말고 삭제/교체한다.

9. 문서를 갱신한다.
   - `game_rulebook.md`: 전투 buff와 consumable modifier의 차이.
   - `unity_core_contract.md`: snapshot/result 필드명과 의미.
   - `refactor_preparation_plan.md`: BuffDatabase source of truth debt 제거 상태.

## Design Rules

- 코드 수정은 장기적인 방향으로 한다. 임시방편, 최소 수정, 특정 테스트만 맞추는 패치를 피한다.
- 처음에는 이 코드가 장기적인 방향인지 판단하기 어려울 수 있으므로, 작은 trial and error를 수행하고 결과를 기록한다.
- 레거시는 과감하게 제거한다. compatibility layer로 감싸서 남기지 않는다.
- 문서를 무조건 신뢰하지 않는다. 실제 코드와 live RON/API를 읽으면서 더 나은 개선안이 보이면 근거를 기록하고 적용한다.
- 단, 더 나은 개선안이 게임 정책이나 Unity-facing 계약을 바꾸는 경우에는 임의 적용하지 말고 사용자에게 이유와 선택지를 보고하고 goal을 종료한다.
- 사용자와 의논하여 정해야 할 정책이 있을 경우 goal을 종료한다.

## Validation Plan

focused test를 먼저 통과시킨 뒤 전체 검증으로 확장한다.

권장 순서:

1. `cargo test -p game_core buffs -- --nocapture`
2. `cargo test -p game_core consumable -- --nocapture`
3. `cargo test -p game_core ron_loading -- --nocapture`
4. `cargo test -p game_core skill_refactor_validation -- --nocapture`
5. `cargo test -p game_core skill_test_suite -- --nocapture`
6. `cargo test -p game_core -- --nocapture`
7. `cargo test -p game_server -- --nocapture`
8. `cargo check -p game_core`
9. `cargo check -p game_server`

서버 request/result 또는 snapshot field가 바뀌면 `game_server` 테스트를 반드시 포함한다.

## Completion Criteria

완료 조건:

1. `docs/goals/buff_database_and_consumable_modifier/PLAN.md`, `EXPERIMENTS.md`, `EXPERIMENT_NOTES.md`가 존재하고 최신 작업 판단을 기록한다.
2. `BuffDatabase`가 `GameDataBase` 구성 흐름에 포함된다.
3. live path의 buff lookup source of truth가 `GameDataBase.buff_data` 또는 명시적으로 주입된 registry로 정리된다.
4. `include_str! + static REGISTRY` 기반 live buff source of truth가 제거된다.
5. skill/effect/timeline validation이 최신 buff source of truth를 기준으로 동작한다.
6. `ActiveConsumableBuff` 의미가 `ActiveConsumableModifier` 또는 동등하게 명확한 명칭으로 정리된다.
7. 전투 상태이상 buff와 아이템 사용 modifier가 코드/문서/snapshot에서 혼동되지 않는다.
8. 아이템 modifier 적용 순서와 duration 감소 시점이 테스트로 고정된다.
9. Unity-facing 계약이 변경된 경우 `unity_core_contract.md`와 server mapping/test가 함께 갱신된다.
10. 레거시 테스트는 ignored 처리 없이 삭제 또는 최신 계약 테스트로 교체된다.
11. 필요한 문서가 최신 정책으로 갱신된다.
12. 검증 명령이 통과한다.
13. 사용자와 의논하여 정해야 할 정책이 있을 경우 goal을 종료한다.

## Stop Conditions

다음 상황이 발견되면 임의로 계속 진행하지 말고 goal을 종료하고 질문 목록을 보고한다.

- `active_consumable_modifier` JSON 필드명 대신 구 `active_consumable_buff`를 유지해야 하는 Unity 클라이언트 의존이 확인되는 경우.
- timeline validator가 `GameDataBase`를 받을 수 없어서 public API를 크게 바꿔야 하는 경우.
- `BuffDatabase`를 `GameDataBase`에 넣으면 테스트용 빈 데이터와 live 기본 데이터 중 어느 쪽이 공식 정책인지 결정해야 하는 경우.
- 전투 상태이상 buff와 consumable modifier를 하나의 효과 엔진으로 합쳐야 한다는 강한 근거가 발견되는 경우.
- 새로운 상태이상 종류나 아이템 효과 정책을 추가로 확정해야 하는 경우.

## Goal Command

```text
/goal docs/buff_database_and_consumable_modifier_goal.md를 기준으로, 전투 상태이상 BuffDatabase와 아이템 사용 consumable modifier의 경계를 명확히 하고 BuffDatabase source of truth를 GameDataBase 흐름으로 통합하라. 먼저 docs/goals/buff_database_and_consumable_modifier/PLAN.md, EXPERIMENTS.md, EXPERIMENT_NOTES.md를 만들고, src/game/battle/buffs.rs, src/game/data/mod.rs, src/game/data/skill_data.rs, src/game/battle/core/mod.rs, src/game/battle/core/sim.rs, src/game/battle/core/commands.rs, src/game/battle/validation/buffs.rs, src/game/battle/timeline.rs, src/game/employee.rs, src/game/data/consumable_data.rs, src/game/world/maintenance.rs, src/game/world/snapshot.rs, src/game/behavior.rs, src/game/world/tests.rs, tests/ron_loading.rs, tests/skill_refactor_validation.rs, tests/skill_test/**, ../game_resources/data/buffs/base.ron, ../game_resources/data/consumables/base.ron, docs/game_rulebook.md, docs/unity_core_contract.md, docs/refactor_preparation_plan.md를 실제 코드 기준으로 읽고 현재 계약을 확인하라. 전투 상태이상 buff는 poison/stun/freeze/silence 같은 전투 중 실시간 효과로 유지하고, 아이템 사용 효과는 Employee에 적용되는 런/다음 전투 modifier로 분리하라. 두 시스템을 억지로 하나의 runtime으로 합치지 말라. BuffDatabase는 include_str! + static REGISTRY live source of truth를 제거하고 GameDataBase/GameDataBuilder/GameDataBaseParts 흐름에 포함하라. BattleCore runtime, skill validation, timeline validation은 GameDataBase.buff_data 또는 명시적으로 주입된 registry를 기준으로 buff를 조회하게 하라. 단순 global wrapper나 compatibility layer로 감싸지 말라. ActiveConsumableBuff는 ActiveConsumableModifier 또는 동등하게 명확한 이름으로 정리하고, Unity-facing snapshot/result 필드명이 바뀌면 unity_core_contract.md와 server mapping/test를 함께 갱신하라. 아이템 modifier 적용 순서는 전투 시작 profile 보정, 배치 비용 보정, 전투 중 스킬/행동, 전투불능 후 Trauma/Run HP 감소 보정, DeathPrevent 판정, 전투/보스 노드 완료 후 duration 감소로 고정하라. 코드 수정은 장기적인 방향으로 하고 임시방편, 최소 수정, 특정 테스트만 맞추는 패치를 피하라. 처음 구조 판단이 어려우면 작은 trial and error를 수행하고 PLAN/EXPERIMENTS/EXPERIMENT_NOTES에 계획, 실험, 생각을 기록하라. 레거시는 과감하게 제거하고 compatibility layer로 감싸지 말라. 문서를 무조건 신뢰하지 말고 실제 코드와 live RON/API를 읽으면서 더 나은 개선안이 보이면 근거를 기록하고 적용하라. 단 Unity-facing 필드명 유지 여부, timeline validator public API 변경, 테스트용 빈 buff data와 live 기본 buff data 정책, 전투 buff와 consumable modifier 통합 여부, 새로운 상태이상/아이템 효과 정책처럼 사용자와 의논해야 할 정책이 발견되면 goal을 종료하고 질문 목록을 보고하라. 테스트는 내부 구조가 아니라 GameDataBase 기반 buff lookup, 전투 상태이상 runtime/validation, consumable modifier 사용/덮어쓰기/적용순서/duration 감소, Unity-facing snapshot 계약을 고정하도록 갱신하라. 검증은 cargo test -p game_core buffs -- --nocapture, cargo test -p game_core consumable -- --nocapture, cargo test -p game_core ron_loading -- --nocapture, cargo test -p game_core skill_refactor_validation -- --nocapture, cargo test -p game_core skill_test_suite -- --nocapture, cargo test -p game_core -- --nocapture, cargo test -p game_server -- --nocapture, cargo check -p game_core, cargo check -p game_server 순서로 수행하라. 완료 조건은 1) 작업 기억 문서 생성, 2) BuffDatabase가 GameDataBase 흐름에 포함됨, 3) live buff lookup source of truth 정리, 4) include_str! + static REGISTRY 제거, 5) validation이 최신 buff source를 사용, 6) ActiveConsumableBuff 명칭/의미 정리, 7) 전투 buff와 consumable modifier 혼동 제거, 8) 아이템 modifier 적용 순서와 duration 감소 테스트 고정, 9) Unity-facing 계약 갱신, 10) 레거시 테스트 ignored 없이 삭제/교체, 11) 필요한 문서 갱신, 12) 검증 명령 통과, 13) 사용자와 의논하여 정해야 할 정책이 있을 경우 goal을 종료함이다.
```
