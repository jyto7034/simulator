# 데이터/RON 로딩 및 검증 시스템 리팩토링 감사

- 컴포넌트: 데이터/RON 로딩 및 검증 시스템
- 기준 문서: `docs/refactor_preparation_plan.md`
- 상태: Initial audit documented

## 읽은 범위

- Runtime code:
  - `src/game/data/mod.rs`
  - `src/game/data/validation.rs`
  - `src/game/data/abnormality_data.rs`
  - `src/game/data/equipment_data.rs`
  - `src/game/data/skill_data.rs`
  - `src/game/data/skill_fragment_data.rs`
  - `src/game/data/reward_data.rs`
  - `src/game/data/shop_data.rs`
  - `src/game/data/pve_data.rs`
  - `src/game/data/corroded_employee_data.rs`
  - `src/game/data/corroded_wave_data.rs`
  - `src/game/map/types.rs`
  - `src/game/combat_preview/mod.rs`
  - `src/game/battle/buffs.rs`
  - `src/game/battle/validation/validator.rs`
  - `../game_server/src/main.rs`
- Live RON/data:
  - `../game_resources/data/abnormalities/base.ron`
  - `../game_resources/data/enemies/corroded_employees.ron`
  - `../game_resources/data/enemies/corroded_wave_presets.ron`
  - `../game_resources/data/employees/starter_candidates.ron`
  - `../game_resources/data/employees/recruitment_candidates.ron`
  - `../game_resources/data/equipments/base.ron`
  - `../game_resources/data/artifacts/base.ron`
  - `../game_resources/data/consumables/base.ron`
  - `../game_resources/data/buffs/base.ron`
  - `../game_resources/data/skills/base.ron`
  - `../game_resources/data/skill_fragments/base.ron`
  - `../game_resources/data/pve/encounters.ron`
  - `../game_resources/data/events/rewards/base.ron`
  - `../game_resources/data/events/shops/base.ron`
  - `../game_resources/data/map/node_definitions.ron`
  - `../game_resources/data/map/battlefield_archetypes.ron`
  - `../game_resources/data/map/battlefield_templates.ron`
  - legacy files under `../game_resources/data/events/*legacy*`
- Tests/contracts:
  - `tests/common/mod.rs`
  - `tests/ron_loading.rs`
  - `src/game/world/tests/mod.rs`
  - `tests/live_skill_catalog_audit.rs`
  - `tests/live_item_skill_activation.rs`
  - `docs/refactor_preparation_plan.md`
  - `docs/game_rulebook.md`
  - `docs/code_documentation_sync_guidelines.md`
  - `/mnt/f/unity projects/ark/docs/unity_core_contract.md`

## 현재 구조 요약

`GameDataBase`는 abnormality, enemy profile, wave preset, employee candidate, artifact, consumable, equipment, shop, reward, PVE, skill, buff, skill fragment DB를 묶고, UUID 기반 `ItemRegistry`를 만든다(`src/game/data/mod.rs:112`, `src/game/data/mod.rs:563`). `GameDataBase::new()`는 각 DB의 `validate_indexes()`를 호출한 뒤 cross-reference validation을 수행하고, item registry를 만든 뒤 shop item UUID를 검증한다(`src/game/data/mod.rs:653`, `src/game/data/mod.rs:667`, `src/game/data/mod.rs:681`).

Cross-reference validation은 이미 `src/game/data/validation.rs`로 분리되어 있다. 과거 `data_validation_refactor` goal의 완료 조건과 현재 코드가 맞는다(`docs/goals/data_validation_refactor/PLAN.md:33`, `src/game/data/mod.rs:64`, `src/game/data/validation.rs:36`).

서버의 공식 live data 로딩은 `../game_server/src/main.rs` 안의 private `load_game_data_from_ron()`이 각 RON 파일을 `include_str!`로 읽고, `GameDataBuilder`에 직접 넣는 방식이다(`../game_server/src/main.rs:157`). integration test도 `tests/common/mod.rs::load_game_data_from_ron()`에서 거의 같은 로딩 코드를 반복한다(`tests/common/mod.rs:350`). world module test에는 또 다른 `live_game_data_from_ron()` 복사본이 있다(`src/game/world/tests/mod.rs:260`).

Map node definitions와 battlefield archetype/template RON은 `GameDataBase`에 포함되지 않는다. `MapNodeDefinitionDatabase::builtin()`과 `BattlefieldGenerationPolicyDatabase::builtin()`/`BattlefieldTemplateDatabase::builtin()`이 각각 직접 `include_str!`로 로딩한다(`src/game/map/types.rs:159`, `src/game/combat_preview/mod.rs:91`, `src/game/combat_preview/mod.rs:192`).

`tests/ron_loading.rs`는 live RON loading, catalog roster, reward, PVE authoring, map content pool, generated preview threat warning 등 많은 정책을 고정한다. 이 중 일부는 `GameDataBase::new()`의 runtime validation에도 들어 있지만, 일부는 test에만 있다.

## Source-of-truth 판단

- Runtime canonical data bundle은 `GameDataBase`다.
- 서버 live loader source는 현재 `../game_server/src/main.rs::load_game_data_from_ron()`이다.
- Test live loader source는 `tests/common/mod.rs::load_game_data_from_ron()`와 `src/game/world/tests/mod.rs::live_game_data_from_ron()`로 중복된다.
- Map definitions와 battlefield generation policy/template는 `GameDataBase` 밖의 builtin singleton loader가 source다.
- Derived indexes(`by_id`, `by_uuid`, `ItemRegistry`)는 canonical data가 아니라 lookup cache/projection이다. 중복 source-of-truth로 보지 않는다.
- `tests/ron_loading.rs`는 live data 계약을 고정하지만, runtime server startup validation과 같은 source는 아니다.

## 리팩토링 후보

### 1. 공식 live RON loader가 core/server/tests에 중복되어 있다

서버는 `../game_server/src/main.rs:157`의 private function에서 live RON을 로딩한다. Integration tests는 `tests/common/mod.rs:350`에 같은 파일 목록과 builder wiring을 복사하고, world module tests는 `src/game/world/tests/mod.rs:260`에 또 다른 복사본을 둔다. 세 경로 모두 `shops`, `rewards`, `abnormalities`, `corroded_employees`, `corroded_wave_presets`, starter/recruitment candidates, equipment, artifact, buff, skills, skill fragments, PVE encounters를 직접 deserialize한다.

이 구조는 live RON 파일 추가/삭제, DB 추가, `SkillFragmentDatabase::with_builtin_starter()` 같은 post-processing 변경 시 여러 로더를 같이 바꿔야 한다. 하나가 누락되면 서버와 테스트가 서로 다른 live data bundle을 검증할 수 있다.

판단: 높은 우선순위 source-of-truth 후보. `game_core` 쪽에 `GameDataBuilder::live_ron()` 또는 `LiveGameDataLoader` 같은 공식 loader를 두고, server와 tests가 같은 함수를 호출하게 하는 것이 좋다. 단, 경로와 `include_str!` 위치가 crate boundary에 걸리므로 Cargo/package 구조를 확인해야 한다.

필요 검증:

- server startup이 새 loader를 사용한다는 compile/check.
- `cargo test -p game_core --test ron_loading -- --nocapture`
- `cargo test -p game_core world::tests::combat -- --nocapture` 중 live RON 사용 test.
- `cargo check -p game_server`

### 2. `GameDataBuilder::live_defaults()` 이름이 실제 live data 범위와 맞지 않는다

`GameDataBuilder::live_defaults()`는 `Self::empty().with_buffs(BuffDatabase::live_default())`만 수행한다(`src/game/data/mod.rs:197`). 나머지 DB는 empty이고, `GameDataBuilder::empty()`도 `SkillFragmentDatabase::with_builtin_starter(vec![])`를 사용하므로 builtin starter fragment만 들어간 partial data다(`src/game/data/mod.rs:177`, `src/game/data/mod.rs:193`). Tests와 validator 일부는 이 이름을 보고 live-ish data로 사용한다(`src/game/battle/validation/validator.rs:343`, `src/game/battle/core/mod.rs:470`, `src/game/battle/core/commands.rs:2095`).

판단: naming/source boundary 후보. 이 함수가 "test에서 live buff DB만 필요한 convenience"라면 이름을 `with_live_buffs_for_tests` 계열로 좁히거나, 공식 live loader와 혼동되지 않게 해야 한다. 반대로 진짜 live defaults가 필요하다면 모든 live RON을 읽어야 한다.

정책 영향은 작지만, public API 의미 변경과 테스트 fixture 범위에 걸린다.

### 3. 기본 스킬 파편이 live RON이 아니라 코드에서 주입된다

`SkillFragmentDatabase::with_builtin_starter()`는 RON에서 `starter_basic_attack_enhancement`가 없으면 `SkillFragmentMetadata::starter_basic_attack()`을 push한다(`src/game/data/skill_fragment_data.rs:455`). 직원 기본 loadout도 같은 hard-coded id를 baseline으로 넣고, 장착 검증은 이 id를 inventory에 없어도 허용한다(`src/game/skill_fragment.rs:15`, `src/game/skill_fragment.rs:707`, `src/game/skill_fragment.rs:749`).

이 구조는 starter basic attack fragment가 "항상 존재하는 코드 builtin"이면 합리적이다. 하지만 live RON/data가 gameplay content의 source of truth라는 기준에서는 기본 파편 정의가 RON과 코드 양쪽에 걸친다. 특히 `with_builtin_starter()`는 server/test live loader에서 RON fragment list를 읽은 뒤 다시 코드 주입을 수행한다(`../game_server/src/main.rs:226`, `tests/common/mod.rs:427`).

판단: 정책 후보. starter basic attack fragment를 live RON에 명시하고 코드 주입을 제거할지, 아니면 코드 builtin으로 공식화하고 문서/검증을 그 의미에 맞게 고정할지 결정해야 한다. 사용자와 정책 논의 필요.

### 4. Map/battlefield live RON이 `GameDataBase` 검증 경로 밖에 있다

Map node definitions는 `MapNodeDefinitionDatabase::builtin()`이 직접 `node_definitions.ron`을 로딩하고 validate한다(`src/game/map/types.rs:159`, `src/game/map/types.rs:170`). Battlefield archetype policy와 templates도 combat preview module의 private builtin singleton에서 직접 로딩한다(`src/game/combat_preview/mod.rs:91`, `src/game/combat_preview/mod.rs:192`). 이 데이터들은 공식 runtime에 쓰이지만 `GameDataBase::new()` validation에는 포함되지 않는다.

`tests/ron_loading.rs`가 map content pools를 확인하고 generated preview validation을 돌리지만, server startup loader는 `GameDataBase`만 만들고 `validate_generated_combat_preview_contracts()`나 map/battlefield builtin validation 전체를 명시적으로 호출하지 않는다(`../game_server/src/main.rs:212`, `tests/ron_loading.rs:636`, `tests/ron_loading.rs:1029`).

판단: source-of-truth 후보. live data bundle validation을 `GameDataBase` 하나에 모두 넣을 필요는 없지만, server startup과 tests가 같은 "validate all live RON contracts" entry point를 호출해야 한다. map/battlefield RON을 `LiveGameData` 같은 상위 bundle에 포함하거나, 최소한 `validate_live_static_data()` entry point로 묶는 것이 좋다.

### 5. Generated combat preview 계약 검증은 test에서만 강하게 실행된다

`GameDataBase::new()`는 PVE enemy/reward reference와 mission compatibility는 검증하지만, 모든 encounter/seed에 대해 generated combat preview가 threat warning과 battlefield instance contract를 만족하는지는 별도 메서드 `validate_generated_combat_preview_contracts()`에 있다(`src/game/data/mod.rs:720`, `src/game/data/validation.rs:84`). `tests/ron_loading.rs`는 이 메서드를 호출한다(`tests/ron_loading.rs:1029`). 서버 live loader는 호출하지 않는다(`../game_server/src/main.rs:212`).

따라서 `cargo test --test ron_loading`은 실패하지만 서버 startup은 통과하는 live data 오류 종류가 남을 수 있다. Preview generation 자체는 runtime에서 실패하므로 조용한 fallback은 아니지만, fail-fast 시점이 test/first runtime use로 밀린다.

판단: validation entry point 후보. server startup 또는 official live data loader에서 generated preview contract validation까지 실행할지 결정해야 한다. 이 검증은 비용이 있으므로 startup 성능과 validation 범위 정책을 함께 봐야 한다. 사용자와 정책 논의 필요.

### 6. Reward explicit tags가 effect semantics와 불일치해도 validation이 막지 않는다

`RewardMetadata::resolved_tags()`는 explicit `tags`가 있으면 effect에서 추론한 tags를 사용하지 않는다(`src/game/data/reward_data.rs:36`). PVE reward policy, map reward filtering, tests는 `resolved_tags()`를 기준으로 Forbidden/Experience/featured tag 판단을 한다(`src/game/world/map_content.rs:172`, `src/game/data/validation.rs:385`, `src/game/reward_policy.rs:54`). Validation은 reward effect target 참조는 확인하지만 explicit tags가 effects와 의미상 맞는지는 확인하지 않는다(`src/game/data/validation.rs:244`).

즉 `GrantExperience` effect에 `Currency` tag를 붙이거나, `ForbiddenAbnormalityGrant`에 Forbidden tag를 빼는 식의 drift가 생기면 tag 기반 정책이 runtime effect와 엇갈릴 수 있다.

판단: data validation 후보. explicit tag를 허용하더라도 "effect-inferred mandatory tags must be included" 또는 "forbidden effect must always imply Forbidden" 같은 invariant를 추가하는 것이 좋다. 다만 tags를 큐레이션/분류 목적으로 effect와 다르게 둘 수 있는지에 따라 정책 판단이 필요하다. 사용자와 정책 논의 필요.

### 7. Legacy random event RON 파일이 live data tree에 남아 있다

`../game_resources/data/events/rewards/legacy_random_event_rewards.ron`, `../game_resources/data/events/shops/legacy_random_event_shops.ron`, `../game_resources/data/events/legacy_random_events.ron`, `../game_resources/data/events/legacy_event_pools.ron` 같은 파일이 live data tree에 존재한다. 서버 live loader와 tests/common live loader는 base files만 읽기 때문에 이 legacy 파일들은 공식 runtime data가 아니다(`../game_server/src/main.rs:158`, `tests/common/mod.rs:352`).

`refactor_preparation_plan.md`는 live path에서 안 쓰는 legacy와 compatibility path를 제거하라고 한다. 다만 이 파일들이 설계 참고/보관 목적인지, 곧 삭제 가능한 낡은 콘텐츠인지 명시가 없다.

판단: legacy cleanup 후보. live loader가 읽지 않는 데이터는 `tests_bak`처럼 명확히 quarantine하거나 삭제해야 다음 작업자가 source of truth로 착각하지 않는다. 기존 콘텐츠 삭제/대체에 해당하므로 사용자와 정책 논의 필요.

### 8. Validation 실패 표현이 `panic!/assert!`에 묶여 있다

Index/validation 실패는 대부분 `panic!`, `assert!`, `expect()`로 fail-fast한다(`src/game/data/mod.rs:74`, `src/game/data/validation.rs:95`, `src/game/data/equipment_data.rs:326`). Live data가 invalid이면 startup/test에서 즉시 실패하는 정책 자체는 좋다. 그러나 invalid fixture를 세밀하게 테스트하거나 server startup error를 사용자 친화적으로 보고하려면 typed error가 부족하다.

판단: 낮은 우선순위 리팩토링 후보. 지금 당장 trait/큰 error hierarchy를 만들 필요는 없다. 다만 official live loader를 만들 때는 `Result<Arc<GameDataBase>, GameDataLoadError>`를 반환하고, 내부 DB validation은 단계적으로 typed error로 옮길 수 있다. 단순 문서 감사 단계에서는 보류한다.

## 기존 기능 조합으로 단순화 가능한 후보

- Server/test/world-test live RON loader 복사본은 하나의 official loader로 합칠 수 있다.
- Map/battlefield builtin validation과 `GameDataBase` validation은 하나의 "validate all live static data" entry point로 묶을 수 있다.
- Reward tag validation은 `RewardTag::from_effects()`를 mandatory baseline으로 재사용하면 explicit tag drift를 줄일 수 있다.
- Generated preview contract validation은 `tests/ron_loading.rs`와 server startup이 같은 helper를 호출하게 만들 수 있다.

## 레거시/fallback/dual schema 제거 후보

- `data/validation.rs` 분리는 이미 완료되어 있으므로 추가 이동 작업은 후보가 아니다.
- `GameDataBuilder::live_defaults()`가 full live data처럼 보이는 이름은 정리 후보지만, compatibility layer를 둘 이유는 없다. 새 이름/loader로 정리하고 호출부를 바꾸는 편이 낫다.
- legacy random event RON files는 live loader가 읽지 않으므로 source-of-truth에서 제거하거나 격리할 후보다.
- `serde(default)` 사용은 일괄 제거 후보가 아니다. 일부 default는 현재 schema authoring 편의다. 다만 live RON에서 필수 정책을 숨기는 default인지 component별로 분류해야 한다.

## 하지 않거나 보류한 항목

- `by_id`, `by_uuid`, `ItemRegistry`는 derived lookup cache이므로 source-of-truth 중복으로 보지 않는다.
- `tests/ron_loading.rs`의 카탈로그 roster 고정 test는 data validation으로 모두 옮기지 않는다. "Plague Doctor standalone scenario 없음" 같은 콘텐츠 정책은 live content audit test로 유지할 수 있다.
- Map/battlefield RON을 반드시 `GameDataBase` struct 필드로 넣어야 한다고 단정하지 않는다. 핵심은 server/tests가 같은 validation entry point를 쓰는 것이다.
- `panic!` 기반 validation을 즉시 typed error 체계로 크게 바꾸는 것은 과도한 리팩토링일 수 있다. official loader 경계부터 좁게 정리한 뒤 판단한다.

## 필요한 테스트와 검증 명령

구현 리팩토링 착수 시 필요한 focused test:

- `cargo check -p game_core`
- `cargo check -p game_server`
- `cargo test -p game_core --test ron_loading -- --nocapture`
- `cargo test -p game_core --test live_skill_catalog_audit -- --nocapture`
- `cargo test -p game_core --test live_item_skill_activation -- --nocapture`
- `cargo test -p game_core world::tests::combat -- --nocapture`

추가로 고정해야 할 테스트:

- server loader와 test loader가 같은 official live data loader를 호출하는 compile-level or integration test.
- explicit reward tags가 effect-inferred mandatory tags와 충돌할 때 validation이 실패하는 test.
- generated preview contract validation이 official live static data validation entry point에 포함되는 test.
- legacy RON file이 live loader source가 아님을 문서/검증에서 명확히 하는 test 또는 cleanup evidence.

이번 감사 단계에서는 코드 변경이 없으므로 Rust test는 실행하지 않았다.

## 사용자와 정책 논의 필요

- starter basic attack fragment를 live RON에 명시할지, 코드 builtin으로 공식화할지: 사용자와 정책 논의 필요.
- server startup에서 generated combat preview contract validation까지 실행할지, test-only validation으로 둘지: 사용자와 정책 논의 필요.
- reward explicit tags가 effects와 달라도 되는지, mandatory inferred tags를 검증해야 하는지: 사용자와 정책 논의 필요.
- legacy random event RON files를 삭제할지, 별도 archive/quarantine 위치로 옮길지: 사용자와 정책 논의 필요.
