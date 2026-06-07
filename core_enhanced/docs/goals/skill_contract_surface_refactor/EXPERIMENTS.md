# Skill Contract Surface Refactor Experiments

## Experiment Log

### E1. Current Contract Survey

- Status: in progress
- Attempt: `snapshot.rs`, `behavior.rs`, `sim.rs`, skill/buff data, live RON, tests를 검색해 호출 경로를 분류했다.
- Result:
  - `skill_catalog` DTO화와 readiness 분리는 작은 단위로 적용 가능하다.
  - Range preset은 RON schema 변경이지만 live skill 반복 제거 목적에 직접 부합한다.
  - BuffDatabase 통합은 호출부가 넓다. 먼저 분류하고, 과도하면 별도 goal로 기록한다.

### E2. Typed Skill Catalog DTO

- Status: applied
- Attempt: `SkillCatalogDto`, `SkillCatalogSkillDto`, `SkillCatalogStepDto`, `SkillCatalogTileAreaDto`를 Unity-facing DTO로 추가하고 `snapshot.rs`가 DTO를 `serde_json::to_value`로 직렬화하게 바꿨다.
- Result: `cargo check -p game_core` 통과. 기존 JSON shape는 유지하고 `delivery`는 typed enum의 snake_case serialization으로 고정했다.

### E3. Manual Skill Readiness Split

- Status: applied
- Attempt: `LiveBattleSkillReadinessDto`에 `target_required`, `target_available`, `target_block_reason`을 추가했다. `manual_activation_allowed`는 caster/resource/action 상태만 판단하고, target 없음은 `target_block_reason=no_valid_target`으로 분리했다.
- Result: `manual_fragment_readiness_keeps_button_enabled_when_target_is_missing` focused test 통과.

### E4. Skill Range Preset RON

- Status: applied
- Attempt: `SkillDatabase`에 RON raw loader를 추가했다. live RON은 `range_presets`와 `defense_tile_range_preset`을 사용할 수 있고, 로딩 후 런타임 `SkillDef`에는 해석된 `defense_tile_range`만 남는다.
- Result: `skill_database_resolves_range_preset_from_ron`, `skill_database_rejects_inline_range_and_preset_conflict`, `ron_loading` focused command 통과.

### E5. BuffDatabase Integration Scope

- Status: deferred pending user decision
- Attempt: `buffs::get`, `contains_name`, `BuffId::from_name` 호출부를 검색했다.
- Result: 호출부가 `BattleCore`, `sim.rs` tick/apply path, command path, timeline validation, data validation, tests에 넓게 퍼져 있다. 이번 goal에서 `GameDatabase` 주입식으로 완전 통합하면 BattleCore 생성자/effect runtime/validation 전반을 크게 흔든다.
- Decision: goal 지침에 따라 즉시 통합하지 않고 별도 goal 후보로 기록한다. 사용자와 의논 후 별도 단계로 진행하는 편이 안전하다.
