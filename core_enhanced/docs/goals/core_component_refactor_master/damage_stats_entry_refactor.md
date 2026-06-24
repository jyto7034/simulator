# 피해/스탯/전투 진입 스탯 시스템 리팩토링 감사

- 기준 문서: `docs/refactor_preparation_plan.md`
- 컴포넌트: 피해/스탯/전투 진입 스탯 시스템
- 상태: Initial audit documented

## 읽은 범위

- Runtime code:
  - `src/game/stats.rs`
  - `src/game/battle/damage.rs`
  - `src/game/battle/stat_pipeline.rs`
  - `src/game/employee.rs`
  - `src/game/battle/types.rs`
  - `src/game/battle/core/build.rs`
  - `src/game/battle/core/commands.rs`
  - `src/game/battle/core/triggers.rs`
  - `src/game/battle/core/movement/steering.rs`
  - `src/game/battle/core/movement/types.rs`
  - `src/game/battle/validation/unit_stats.rs`
  - `src/game/battle/validation/state.rs`
- Live data:
  - `../game_resources/data/equipments/base.ron`
  - `../game_resources/data/artifacts/base.ron`
  - `../game_resources/data/abnormalities/base.ron`
  - `../game_resources/data/enemies/corroded_employees.ron`
- Policy/contract:
  - `docs/refactor_preparation_plan.md`
  - `docs/game_rulebook.md`
  - `/mnt/f/unity projects/ark/docs/unity_core_contract.md`
  - `/mnt/f/unity projects/ark/docs/core_unity_battle_transport_contract.md`
- Prior goal notes used only as historical context:
  - `docs/goals/stats_pipeline_unification/*`
  - `docs/goals/stat_pipeline_inventory_contract_repair/*`

## 현재 구조 요약

전투 진입 스탯은 `src/game/battle/stat_pipeline.rs`가 명시적인 두 단계로 계산한다. 1단계는 직원 base profile에 skill fragment, run HP/trauma condition, active consumable battle modifier를 적용한다. 2단계는 `BattleUnitDraft`의 combat profile에 growth stack, 장착 아이템 Permanent effect, 장비 enhancement modifier, artifact Permanent effect를 적용한 뒤 current HP ratio를 보존한다.

`Employee::combat_profile_for_battle()`는 `stat_pipeline::employee_combat_profile_for_battle()`로 위임한다(`src/game/employee.rs:213`). `BattleUnitDraft::effective_stats()`도 `stat_pipeline::effective_stats_for_draft()`로 위임한다(`src/game/battle/types.rs:461`). 이 점은 이전 stats pipeline 정리 goal의 완료 조건과 현재 코드가 일치한다.

전투 runtime에서는 `RuntimeUnit.stats`가 HP, attack, defense, magic resist, attack interval, move speed를 들고 있고, `RuntimeUnit.incoming_damage_modifiers`가 별도 damage modifier bucket으로 있다. Damage 계산은 `DamageRequest`와 `DamageContext`를 받아 `calculate_damage()`가 순수 계산을 수행하고, 실제 HP 변경/죽음/timeline 기록은 `BattleCore::apply_damage_result_and_record()`가 담당한다.

## Source Of Truth 판단

전투 진입 final stat order의 source of truth는 `src/game/battle/stat_pipeline.rs`다. 파일 상단 주석이 employee profile stage와 draft final-stat stage를 현재형으로 고정하고(`src/game/battle/stat_pipeline.rs:1`), 실제 함수도 그 순서를 따른다.

`UnitStats::apply_modifier()`가 개별 stat modifier 적용의 source of truth다. Permanent effect는 `UnitStats::apply_permanent_effects()`를 통해 전투 시작 전 stat으로 반영되고, runtime stat 변경은 `BattleCommand::ApplyModifier`가 같은 `UnitStats::apply_modifier()`를 호출한다.

피해량 계산의 source of truth는 `calculate_damage()`다. 기본 공격, 스킬/command damage, target usefulness preview 모두 동일한 함수로 최종 피해를 계산한다. 실제 HP mutation과 death timeline은 `apply_damage_result_and_record()`가 source of truth다.

이동 속도는 현재 source-of-truth가 갈라져 있다. `UnitStats.move_speed_units_per_ms`는 stat modifier와 validation/timeline에 나타나지만, 실제 movement displacement는 `UnitBody.move_speed`를 사용한다(`src/game/battle/core/movement/steering.rs:14`). 전투 spawn도 final stats를 계산한 뒤 `combat_profile.movement.speed_units_per_ms`로 `stats.move_speed_units_per_ms`를 다시 덮어쓴다(`src/game/battle/core/build.rs:92`).

## 리팩토링 후보

### 1. `MoveSpeedUnitsPerMs` final stat과 `UnitBody.move_speed`의 source-of-truth 분리

`UnitStats`는 `MoveSpeedUnitsPerMs` modifier를 지원한다(`src/game/stats.rs:308`). live equipment data에도 `time_dial`이 Permanent `MoveSpeedUnitsPerMs +400`을 가진다(`../game_resources/data/equipments/base.ron:247`). 따라서 이 필드는 단순 미래용 placeholder가 아니다.

그러나 battle spawn은 `unit.effective_stats()`로 Permanent effects가 적용된 `stats`를 계산한 뒤, `combat_profile.movement.speed_units_per_ms`를 다시 `stats.move_speed_units_per_ms`에 대입한다(`src/game/battle/core/build.rs:77`, `:92`). `UnitBody` 역시 이 덮어쓴 movement speed에서 만들어진다(`src/game/battle/core/build.rs:100`). 이러면 장비/아티팩트 Permanent `MoveSpeedUnitsPerMs` modifier가 final stats pipeline에서 계산되어도 실제 spawn stat/body에 반영되지 않을 수 있다.

runtime stat 변경도 비슷하다. `BattleCommand::ApplyModifier`는 `target.stats.apply_modifier(modifier)`만 호출하고 `target.body.move_speed`는 갱신하지 않는다(`src/game/battle/core/commands.rs:1342`). 반면 movement steering은 `unit.body.move_speed`를 기준으로 이동량을 계산한다(`src/game/battle/core/movement/steering.rs:16`). `RuntimeUnit::can_move()`는 `stats.move_speed_units_per_ms > 0`을 보므로(`src/game/battle/core/types.rs:437`), stats와 body가 엇갈리면 "움직일 수 있는지"와 "얼마나 움직이는지"가 다른 source를 보게 된다.

판단: high-priority source-of-truth 리팩토링 후보. final stat의 move speed를 `UnitBody` 생성과 runtime modifier 모두의 단일 입력으로 삼거나, move speed를 stat modifier 대상에서 제외하고 이동 전용 profile로만 둬야 한다. 어느 쪽이 공식 밸런스인지 결정이 필요하다. 사용자와 정책 논의 필요.

검증 후보:

- `time_dial` 같은 live equipment를 장착한 직원의 spawn `UnitSpawned.stats.move_speed_units_per_ms`와 첫 `MovementSegmentStarted.ends_at_ms`가 modifier를 반영하는지 확인한다.
- runtime `ModifyStats(MoveSpeedUnitsPerMs)` 적용 후 다음 movement tick의 segment duration 또는 velocity가 바뀌는지 테스트한다.
- 이미 진행 중인 movement segment의 속도를 즉시 재계산할지, 다음 segment부터 반영할지 정책을 고정한다.

### 2. `spawn_scenario_group()`의 combat profile 이중 계산과 final stat 후처리

`spawn_scenario_group()`는 먼저 `unit.combat_profile()`을 호출하고, 바로 다음 줄에서 `unit.effective_stats()`를 호출한다(`src/game/battle/core/build.rs:77`). 그런데 `effective_stats_for_draft()` 내부도 다시 `draft.combat_profile(game_data)`를 호출한다(`src/game/battle/stat_pipeline.rs:70`). 그 뒤 spawn은 별도로 resonance, movement, incoming damage modifiers, basic attack, skill id 등을 첫 번째 `combat_profile`에서 읽고, stats는 두 번째 계산 결과에서 읽는다.

현재 대부분의 계산은 deterministic이라 즉시 drift가 발생하지는 않는다. 하지만 "final battle-entry profile"이 stats와 non-stat combat profile로 나뉘어 두 번 계산되고, spawn이 그 사이에서 movement speed를 덮어쓰는 구조는 source-of-truth 경계가 약하다.

판단: 리팩토링 후보. `effective_stats_for_draft()`가 이미 계산한 base profile을 반환하거나, `BattleEntryProfile` 같은 단일 결과를 만들어 spawn이 한 entry point에서 stats/basic attack/movement/resonance/incoming modifiers를 모두 받도록 정리하는 편이 장기적으로 안전하다. 단, 과도한 abstraction이 되지 않도록 1번 move speed 버그를 고칠 때 필요한 최소 형태로 검토한다.

검증 후보:

- employee weapon profile, skill fragment, consumable, equipment Permanent, enhancement, artifact Permanent가 모두 들어간 fixture에서 spawn된 `RuntimeUnit`의 `stats`, `basic_attack`, `incoming_damage_modifiers`, `body.move_speed`를 같이 검증한다.

### 3. `calculate_damage()`가 death command를 만들고 runtime은 이를 무시하는 dual death source

`calculate_damage()`는 target HP가 0이 되면 `BattleCommand::UnitDied`를 `DamageResult.triggered_commands`에 넣는다(`src/game/battle/damage.rs:511`). 그러나 `BattleCore::process_commands()`는 `BattleCommand::UnitDied`를 "HP reaching 0에서 파생된다"며 무시한다(`src/game/battle/core/commands.rs:1292`). 실제 death timeline과 death trigger는 `apply_damage_result_and_record()`가 HP 적용 후 `finalize_unit_death()`를 호출하면서 처리한다.

이 구조는 현재 동작을 깨지는 않지만, `DamageResult.triggered_commands`라는 이름 아래 실행되지 않는 death command가 섞인다. 새 damage caller가 `DamageResult.triggered_commands`를 별도 처리한다고 믿으면 death source-of-truth가 흐려질 수 있다.

판단: 리팩토링 후보. `calculate_damage()`는 `target_killed`와 `target_remaining_hp`까지만 반환하고, `BattleCommand::UnitDied` 생성은 제거하는 방향이 더 선명하다. 이미 runtime death source는 `apply_damage_result_and_record()`에 있다.

검증 후보:

- `calculate_damage()` unit test에서 lethal damage의 `target_killed`만 확인하고 `triggered_commands`에 `UnitDied`가 없음을 고정한다.
- 기본 공격/스킬/command damage death trigger test가 그대로 통과하는지 확인한다.

### 4. `DamageContext`의 unused fields

`DamageContext`는 `attacker_attack`과 `target_max_hp`를 가진다(`src/game/battle/damage.rs:253`). 현재 `calculate_damage()` 안에서는 `request.base_damage`, `target_current_hp`, resistance, modifiers가 실제 계산에 쓰이고, `attacker_attack`과 `target_max_hp`는 직접 참조되지 않는다. `DamageRequest.time_ms`도 damage roll이 request 생성 전에 계산되므로 damage 함수 안에서는 사용되지 않는다.

이 필드들은 앞으로 percent-of-max-health damage나 attacker_attack 기반 effects를 넣기 위한 placeholder일 수 있지만, 현재는 source-of-truth를 흐리는 context noise다. 특히 call site들이 매번 target max HP와 attacker attack을 snapshot하는데 실제 계산에는 영향이 없다.

판단: 제거 또는 용도 명시 후보. 지금 쓰지 않는다면 제거하는 편이 `refactor_preparation_plan.md`의 과도한/미래형 구조 방지 기준에 맞다. 단, 가까운 정책에서 max HP 기반 damage가 예정되어 있다면 TODO가 아니라 명시된 effect 도입 시점에 다시 추가한다.

검증 후보:

- unused field 제거 후 `cargo check -p game_core`.
- damage tests 전체.

### 5. Runtime `StatChanged`와 movement presentation 계약

Unity-facing 계약은 movement segment를 immutable event로 본다. `/mnt/f/unity projects/ark/docs/unity_core_contract.md:1154`는 이미 받은 `MovementSegmentStarted`의 target/ends_at_ms가 나중에 바뀐다고 가정하지 말라고 한다. 따라서 runtime move speed modifier를 실제 movement에 반영하려면 `UnitBody.move_speed`만 바꾸는 것으로 부족하고, active movement segment를 언제 중단/재시작할지 또는 다음 tick부터 반영할지 정해야 한다.

판단: 1번 후보의 정책 세부 항목. 사용자와 정책 논의 필요.

검증 후보:

- 이동 중 `MoveSpeedUnitsPerMs` buff/debuff가 들어오는 테스트.
- 새 `MovementSegmentStarted` 또는 `MovementStopped` event가 필요한지 확인하는 Unity-facing timeline test.

## 기존 기능 조합으로 단순화 가능한 후보

- pre-battle final stat 계산은 이미 `stat_pipeline.rs`로 모여 있다. 여기서 새 pipeline을 더 크게 만들기보다, spawn/build 쪽에서 pipeline 결과를 다시 덮어쓰지 않도록 단일 entry result를 쓰는 편이 기존 구조를 살리는 방향이다.
- damage preview와 실제 damage는 이미 `calculate_damage()`를 공유한다. target usefulness를 위한 별도 근사 damage 계산을 만들 필요는 없다.
- Permanent effect 적용은 `UnitStats::apply_permanent_effects()`가 validation까지 포함해 처리한다. 장비/아티팩트별 별도 Permanent parser를 만들 필요는 없다.

## 레거시/fallback/dual schema 제거 후보

- `DamageResult.triggered_commands` 안의 `BattleCommand::UnitDied`는 실행되지 않는 derived command로 보이며 제거 후보.
- `DamageContext.attacker_attack`, `DamageContext.target_max_hp`, `DamageRequest.time_ms`는 현재 계산에 쓰이지 않는 context payload로 제거 또는 명시적 사용 후보.
- `spawn_scenario_group()`에서 `effective_stats()` 뒤 `stats.move_speed_units_per_ms`를 다시 쓰는 후처리는 final stat pipeline과 중복되는 fallback/override 후보.

## 하지 않거나 보류한 항목

- `UnitStats.move_speed_units_per_ms`를 즉시 삭제하지 않는다. live equipment data가 이 modifier를 사용하고, `RuntimeUnit::can_move()`도 이 필드를 본다.
- buff/status runtime 전체는 10번 버프/상태이상 시스템에서 별도로 감사한다. 이 문서에서는 stat modifier가 runtime unit/body/damage에 어떻게 연결되는지만 다룬다.
- active consumable modifier는 deployment cost, trauma mitigation, run HP loss mitigation, battle stat modifier를 모두 포함한다. 이것은 12번 아이템/소모품 경제와 11번 직원 상태 감사에서도 다시 봐야 하므로, 여기서는 battle-entry stat에 들어오는 효과만 기록한다.

## 필요한 테스트와 검증 명령

문서 감사만 수행했으므로 이번 단계에서 테스트는 실행하지 않았다.

후속 구현 시 우선 검증:

```bash
cargo test -p game_core battle::stat_pipeline::tests -- --nocapture
cargo test -p game_core battle::types::tests -- --nocapture
cargo test -p game_core battle::core::movement -- --nocapture
cargo test -p game_core battle::damage::tests -- --nocapture
cargo check -p game_core
```

정책 확정 후 추가할 테스트:

- live `MoveSpeedUnitsPerMs` equipment modifier가 spawned unit stats와 movement body speed에 반영되는지.
- runtime `MoveSpeedUnitsPerMs` modifier가 active movement segment에 즉시/다음 tick/다음 segment 중 어느 시점부터 반영되는지.
- `DamageResult` lethal damage가 death command를 중복 생성하지 않는지.
- `DamageContext` unused fields 제거 후 damage feedback/critical/modifier tests가 그대로 통과하는지.

## 사용자와 정책 논의 필요

- `MoveSpeedUnitsPerMs` modifier가 공식적으로 실제 이동 속도를 바꾸는지, 아니면 표시/미래용 stat으로 남겨야 하는지. 사용자와 정책 논의 필요.
- 이동 중 move speed modifier가 들어오면 현재 segment를 끊고 새 movement event를 내보낼지, 다음 movement tick/segment부터 반영할지. 사용자와 정책 논의 필요.

