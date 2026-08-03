# Data Loading Contract

이 문서는 core live RON authoring, embedded data loading, loader ownership, fixture, validation의 canonical 계약이다.
Runtime에서 검증된 data를 소비한 뒤의 결정론과 event/snapshot 경계는 [core_runtime_contract.md](core_runtime_contract.md)를 따른다.

## 문서 책임

| 구분 | 내용 |
| --- | --- |
| 문서 등급 | Data canonical |
| 소유 범위 | live RON authoring strictness, embedded bundle entrypoint, loader ownership, schema publication, fixture 경계, validation gate와 live file 색인 |
| 소유하지 않는 범위 | 검증된 data를 소비한 뒤의 gameplay 의미, runtime state transition, 타겟팅 실행 알고리즘, Unity wire payload |
| 함께 확인할 문서 | [game_rulebook.md](game_rulebook.md), [core_runtime_contract.md](core_runtime_contract.md), [skill_target_contract.md](skill_target_contract.md), domain data module과 validation test |

## Source Of Truth

실제 source of truth는 runtime code와 live RON/data다. 이 문서는 깨진 링크를 막기 위한 별도 계획 문서가 아니라, 현재 코드가 이미 따르는 data loading 경계를 요약하는 계약 문서다.

공식 production live data entrypoint는 [src/game/data/mod.rs](../src/game/data/mod.rs)의 `GameDataBase::load_live_embedded()`다.

- `game_server` startup은 `GameDataBase::load_live_embedded()`로 위임한다.
- live RON을 읽는 integration/helper test도 같은 entrypoint로 위임한다.
- loader는 `include_str!` 기반 embedded bundle을 사용한다. RON 수정은 재컴파일이 필요하며, 이 경로는 hot reload나 filesystem data-pack API가 아니다.
- `data/bundle.ron`의 `schema_version`은 현재 정확히 `1`이어야 한다. 누락, 다른 버전, unknown
  manifest field는 거부하며 이전 버전 alias/fallback은 없다.
- 공식 loader는 manifest 외 21개 gameplay RON source를 하나의 source set으로 parse한 뒤 local
  contract, key/UUID uniqueness, cross-catalog reference, map payload, RunStart, extra-wave,
  generated combat-preview/briefing contract를 모두 검증한다.
- Parse/local/cross 검증이 끝난 parts로 private candidate `GameDataBase`를 조립하고, 그
  candidate로 generated combat-preview/briefing 검증을 실행한 뒤 성공한 Arc만 호출자에게
  반환한다. Candidate는 최종 검증 전에 외부로 publish되지 않는다. Production 경로는
  fixture용 `GameDataBuilder`/`GameDataBase::new()`의 panic wrapper를 다시 통과하지 않는다.
- 실패는 `LiveRonError`의 source file, domain, optional object id/field path, cause로 반환된다.
  `game_server`는 이를 startup `InvalidData`로 바꾸고 HTTP/WebSocket bind 전에 종료한다.

### Durable runtime documents are not live RON

Local active-run and encounter-compendium profile files are separate host-owned runtime documents, not
content packs and not alternate live-data loaders. The active-run envelope accepts exactly version 3 and wraps
a strict Core payload version 2; the encounter-compendium envelope accepts exactly version 1. Both deny unknown
fields, verify SHA-256 before bootstrap, and use independent file names/locks/transactions under the OS
application persistent-data root. Active-run checkpoint data owns `power_model_version` and
`current_floor_power_anchor`, while only `loads_used` and `run_peak_certified_power` are host monotonic metadata.
Restore validates the checkpoint power generation plus map/recruitment references against the current published
`GameDataBase`. Compendium restore validates ID format and defeated/discovered subset
without rejecting dormant catalog IDs; only newly committed IDs require current catalog membership.
Unsupported versions, malformed Base64/JSON, checksum failures, and invalid current-plus-backup combinations
are explicit system issues; no legacy alias, implicit migration, empty-profile fallback, or RON compatibility
reader is provided.

## Live RON Authoring

Live RON은 gameplay source data다. 새/core live schema는 가능한 한 authoring typo를 deserialize 단계에서 잡아야 하며, gameplay 의미가 있는 필드는 조용히 무시하거나 legacy alias로 받아들이지 않는다.

- Core gameplay schema는 `#[serde(deny_unknown_fields)]`를 기본으로 사용한다.
- Unknown field, stale field, removed alias는 compatibility layer 없이 reject한다.
- Gameplay 의미가 바뀌는 필드는 명시 authoring을 우선한다.
- 단순 표시, optional presentation hint, harmless authoring convenience는 명시적 `serde(default)`를 둘 수 있다.
- `serde(default)`를 둔 gameplay field는 코드 이름, validation, 테스트가 기본값의 의도를 설명해야 한다.
- 중복 top-level id/UUID, pool id, cross-item UUID는 first/last winner를 만들지 않고 bundle을
  거부한다.
- Live-authored `basic_attack` data does not rely on `BasicAttackDef` runtime defaults. Abnormality and corroded employee live RON must pass through authored input validation where combat-meaningful fields are explicit: `range_units`, `range_policy`, `defense_tile_range`, `damage_type`, `targeting_profile`, `air_capable`, `range_role`, `interval_ms`, `windup_ms`, `ranged_reposition_ms`, and `delivery`.
- Live skill `cast_targeting` is required, and `EnemySingle` requires an explicit `rule` even when
  the intended value is `Nearest`. Missing rule, weapon-profile inheritance variant, and unknown /
  dual targeting authority are strict load failures.
- Skill cast acquisition explicitly authors `reach` and `air_capable` plus optional `target_range`;
  missing reach/air authority fails loading, and `global` rejects a competing target pattern. Steps
  reject removed range/air fields, while every `TileArea` requires its own
  concrete `footprint`. Target/impact-centered footprints require `tile_origin: Anchor` and are
  rotation-invariant until an explicit orientation contract exists.
- `defense_tile_range: None` is still an explicit authored value. `range_policy: Pattern` may use a preset/resolution layer to fill the runtime tile pattern, but the authoring field itself must be present.
- Enum casing은 도메인 내부에서 일관되어야 한다. 별도 `rename_all`이 없는 live RON enum은 Rust variant 이름을 그대로 사용하고, `snake_case`로 선언된 enum은 RON도 snake_case를 사용한다.
- String id는 기존 live data convention을 따른다: item/profile/skill/event/preset id는 lower snake_case, Lobotomy-style abnormality id는 catalog id 형식을 유지한다.
- Live RON loading과 focused schema tests는 gameplay 시작 전 authoring mistake를 잡는 gate다.

현재 strictness가 고정된 핵심 도메인:

- PvE encounter/wave authoring은 unknown top-level/nested fields를 reject한다.
- Event scene/panel/employee-check/success-action authoring은 unknown fields를 reject한다.
  Panel/image/text 비어 있음, scene-level legacy `required_roll`, DC 8/12/16/20 이외 값,
  invalid/duplicate choice key 또는 equipment Event metadata, choice/scene identity 중복, 둘 이상의
  reachable checked scene, checked-scene return cycle를 포함한 graph defect, unsupported action,
  broken encounter/reward/key reference도 bundle publication 전에 거부한다.
- Employee definition은 Event aptitude를 필수로 authoring하고 생성 시 stored
  `EmployeeProfile.event_aptitude`로 복사한다. Equipment의 `event_check_modifiers`는 aptitude별
  +1/+2만, `event_choice_keys`는 valid lower-snake-case unique key만 허용한다. Event가 요구하는
  key는 published equipment catalog에 provider가 없으면 bundle 전체를 거부한다.
- Live `sealed_archive_recovery_01`은 `event_sealed_archive` map definition으로 명시적으로
  참조되며 `archive_scanner`/`maintenance_tech` provider path를 사용한다. WhiteNight는 별도
  advance-only Event로 유지된다.
- Visual Novel definition과 trigger authoring은 unknown/removed fields를 reject한다. Definition은
  scene/presentation만 authoring하고 `blocking`은 trigger에만 필수로 존재한다;
  removed definition `default_blocking`은 compatibility field로 수용하지 않는다.
- Run policy의 recovery는 top-level `save_point_trauma_recovery`와
  `gate_trauma_recovery`를 exact tagged `TraumaRecovery`로 authoring한다. 허용 variant는
  `Flat(amount: 1..100)`과 `CurrentPercent(percent: 1..100)`뿐이며 variant에 다른 variant의 field를
  섞거나 제거된 `support`/Rest recovery group을 작성하면 reject한다. Production live 값은 각각
  `CurrentPercent(percent: 15)`와 `CurrentPercent(percent: 10)`이다.
- Corroded employee profile/range preset authoring은 unknown fields를 reject한다.
- Abnormality and corroded employee `basic_attack` authoring rejects missing combat fields instead of falling back to runtime `BasicAttackDef::default()`.
- Shop authoring은 reusable merchandise preset과 Shop별 preset reference/local entry를 사용한다.
  모든 Shop은 positive `offer_count`/buy percentage와 충분한 unique candidate를 가져야 한다.
  Entry quantity/weight/optional buy percentage는 양수이고 equipment quantity는 1이다. Missing typed
  merchandise, duplicate composition, inconsistent tags, invalid refresh/sell policy, zero-price item,
  quote overflow와 same/cross-Shop arbitrage를 bundle publication 전에 reject한다. Prototype
  `shop_type`, `visible_items`, `stock_items`, `can_reroll`은 unknown field이며 compatibility schema가
  없다.
- Run policy, skill raw schema, reward equipment pools, boss omen chains, corroded wave presets, facility
  templates/node-assignment profiles/node definitions are also strict by domain-owned validation/tests.
- Reward/Event `GrantConsumable`은 `consumable_id`와 양수 `quantity`를 명시해야 한다. Missing
  quantity, zero quantity, unknown consumable은 publication 전에 거부하며 runtime은 per-copy
  UUID나 implicit quantity 1을 보정하지 않는다.
- Run growth policy는 비어 있거나 0인 XP requirement를 거부한다. Battle tier threshold는 비어
  있을 수 없고 `min_level > 0`인 strictly increasing 순서여야 한다. Runtime은 authored XP
  curve 범위를 넘으면 마지막 requirement를 반복하고, level에 맞는 가장 높은 tier threshold를
  선택한다.
- Consumable `rarity/live_pool`, generated Skill identity/timing/step fields, Abnormality and
  CorrodedEmployee core combat fields, CorrodedWave role balance fields, PVE manual enemy
  `tier/count` and wave `time_ms` are presence-required. Current live generated data already authors
  them explicitly.
- Option omission, empty collection defaults, targeting/repeat/presentation defaults are retained
  only where omission itself is the canonical neutral/optional meaning; they are not legacy schema
  acceptance.

## Main Bundle

`GameDataBase::load_live_embedded()`가 소유하는 main live bundle은 gameplay data를 `GameDataBase` 안으로 모으는 경로다. 아래 표가 main bundle의 live RON group, runtime loader/type, authored file 위치를 연결하는 canonical index다.

Live data root는 `/mnt/f/work/simulator/game_resources/data`다.

| Live RON group | Runtime loader/types | Live files |
| --- | --- | --- |
| Abnormalities | [data/abnormality_data.rs](../src/game/data/abnormality_data.rs) | `/mnt/f/work/simulator/game_resources/data/abnormalities/base.ron` |
| Boss omen chains | [data/boss_omen_data.rs](../src/game/data/boss_omen_data.rs) | `/mnt/f/work/simulator/game_resources/data/boss_omen/chains.ron` |
| Buffs | [data/mod.rs](../src/game/data/mod.rs), [battle/buffs.rs](../src/game/battle/buffs.rs) | `/mnt/f/work/simulator/game_resources/data/buffs/base.ron` |
| Consumables | [data/consumable_data.rs](../src/game/data/consumable_data.rs) | `/mnt/f/work/simulator/game_resources/data/consumables/base.ron` |
| Corroded employees | [data/corroded_employee_data.rs](../src/game/data/corroded_employee_data.rs) | `/mnt/f/work/simulator/game_resources/data/enemies/corroded_employees.ron` |
| Corroded waves | [data/corroded_wave_data.rs](../src/game/data/corroded_wave_data.rs) | `/mnt/f/work/simulator/game_resources/data/enemies/corroded_wave_presets.ron` |
| Employees | [data/employee_data.rs](../src/game/data/employee_data.rs) | `/mnt/f/work/simulator/game_resources/data/employees/definitions.ron` |
| Equipment | [data/equipment_data.rs](../src/game/data/equipment_data.rs) | `/mnt/f/work/simulator/game_resources/data/equipments/base.ron` |
| Events | [data/event_data.rs](../src/game/data/event_data.rs) | `/mnt/f/work/simulator/game_resources/data/events/story/base.ron` |
| PvE encounters | [data/pve_data.rs](../src/game/data/pve_data.rs) | `/mnt/f/work/simulator/game_resources/data/pve/encounters.ron` |
| Rewards | [data/reward_data.rs](../src/game/data/reward_data.rs) | `/mnt/f/work/simulator/game_resources/data/events/rewards/base.ron` |
| Run policy | [data/run_policy_data.rs](../src/game/data/run_policy_data.rs) | `/mnt/f/work/simulator/game_resources/data/run/policy.ron` |
| Shops | [data/shop_data.rs](../src/game/data/shop_data.rs) | `/mnt/f/work/simulator/game_resources/data/events/shops/base.ron` |
| Skills | [data/skill_data.rs](../src/game/data/skill_data.rs) | `/mnt/f/work/simulator/game_resources/data/skills/base.ron` |
| Skill fragments | [data/skill_fragment_data.rs](../src/game/data/skill_fragment_data.rs) | `/mnt/f/work/simulator/game_resources/data/skill_fragments/base.ron` |
| Visual novels | [data/visual_novel_data.rs](../src/game/data/visual_novel_data.rs) | `/mnt/f/work/simulator/game_resources/data/visual_novels/base.ron` |
| Visual novel triggers | [data/visual_novel_trigger_data.rs](../src/game/data/visual_novel_trigger_data.rs) | `/mnt/f/work/simulator/game_resources/data/visual_novel_triggers/base.ron` |

Employee loading은 하나의 strict `EmployeeDefinitionDatabase`만 publication한다. 각 definition은 전역적으로 유일하고 비어 있지 않은 `employee_definition_id`, 이름, `starter_eligible`, explicit `initial_loadout`, role/background, 두 personality 축, `CombatAptitude`, base combat stats와 양수 `basic_attack_speed_percent`를 가져야 한다. starter-eligible definition 수와 initial loadout의 장비/스킬 파편 참조도 startup validation에서 확인한다. 제거된 starter/recruitment 두 파일이나 필드 alias를 dual-read하지 않는다.

## Map And Battlefield Catalogs

Facility templates, node-assignment profiles, map node definitions, battlefield archetypes and battlefield
templates are domain-owned types but not separate production readers. Their five live files are parsed once
inside the same versioned source set, validated before publication, stored in `GameDataBase`, and used by
facility generation/navigation or combat preview through that exact instance.

| Catalog | Runtime owner | Live file |
| --- | --- | --- |
| Facility templates/pools | [map/facility.rs](../src/game/map/facility.rs) | `/mnt/f/work/simulator/game_resources/data/map/facility_templates.ron` |
| Node assignment profiles/schedules | [map/facility.rs](../src/game/map/facility.rs) | `/mnt/f/work/simulator/game_resources/data/map/node_assignment_profiles.ron` |
| Map node definitions | [map/types.rs](../src/game/map/types.rs) | `/mnt/f/work/simulator/game_resources/data/map/node_definitions.ron` |
| Battlefield archetypes | [combat_preview/mod.rs](../src/game/combat_preview/mod.rs) | `/mnt/f/work/simulator/game_resources/data/map/battlefield_archetypes.ron` |
| Battlefield templates | [combat_preview/mod.rs](../src/game/combat_preview/mod.rs) | `/mnt/f/work/simulator/game_resources/data/map/battlefield_templates.ron` |

### Facility Template And Assignment Authoring

- `facility_templates.ron` owns versioned semantic slots, allowed categories, authored tags, typed terminal
  roles, directed connections, optional connection reveal gates, manifestation rules and template pools.
  Duplicate/missing slot references, unknown tags, invalid terminal category/role, unreachable terminal,
  directional deadlock, invalid manifestation dependency and Standard guardian-dominator violations reject
  the bundle.
- `node_assignment_profiles.ron` owns Standard exact Floor schedules and Endless contiguous/open-ended
  stages/cycles, category min/max/exact constraints, remaining-category weights and tagged fixed assignments.
  Each profile has a required `difficulty_profile_id` that must resolve in the same live RunPolicy publication.
  Missing/overlapping schedule coverage, unknown template/profile/pool reference and an assignment with no
  complete feasible slot matching reject publication; runtime does not repair categories.
- `node_definitions.ron` chooses kind/content only within an already assigned category. It has no depth range
  or procedural topology authority. Structural Start nodes are synthesized from Start slots and intentionally
  have no node-definition pool entry.
- Removed `generation_policy.ron`, depth/lane slot schema, min/max depth gates and procedural generator are not
  accepted as compatibility input or fallback.
- `run/policy.ron` owns required `combat_difficulty.power_model_version = 2`, whose certified roster membership is
  `alive && physical_damage <= 4 && trauma < 100`, plus non-zero certified-power weights,
  qualitative threat thresholds, exact Standard/open-ended Endless anchor schedules, and complete ordinary/role
  profiles. Easy/Standard/Hard must occur once with quotas totaling 100; every special role must occur once;
  Hard and role reward multipliers must exceed Standard. Removed `floor_combat_scaling`, last-stage clamp and
  missing-profile defaults are rejected rather than migrated.
- 같은 `run/policy.ron`의 required `monolith` group은 Floor 4+ merchant ID, page/refresh/use/attempt 수,
  residual threshold, price/weight, stage stat/reward multiplier와 exchange VN ID를 소유한다. Live bundle
  validation은 merchant가 unknown-exploration context의 3-offer/1-refresh Shop인지, VN이 존재하는지,
  모든 Elite primary abnormality가 unique fragment를 가지는지 검사한다.
- Monolith role과 stage multiplier는 runtime의 단일 difficulty resolver가 합성한다. Stage stat은
  target power와 적 최대 HP/공격/방어에 적용하되 generated wave budget/extra waves에는 적용하지 않고,
  reward multiplier는 role+stage를 합성한 뒤 authored 원수량에 한 번만 적용한다. 현재 결과는 stage 2
  `target 220/HP 165/ATK 145/DEF 145/wave 200/reward 250%`, stage 3
  `275/207/182/182/200/313%`다.
- Standard `unknown_exploration_shop`과 Endless-only `wandering_monolith_shop`은 서로 다른 live ID다.
  `ShopMerchandise::Monolith`는 runtime-generated typed offer이며 static Shop preset/local entry에 저작할
  수 없다. 방랑 상인의 static pool은 repeatable consumable만 제공하고 Core가 eligible monolith를
  first page에 합성한다.

### Battlefield Template Authoring

- Live 전장 데이터의 authoring source는 `battlefield_templates.ron`의 ASCII `rows`와 선택적
  `routes.overlay`다.
- ASCII row의 공백은 전장 밖이다. 전장은 사각형일 필요가 없다.
- `width`는 가장 긴 row 길이, `height`는 row 개수로 계산한다.
- 공백이 아닌 타일은 `valid_tiles`가 되고, `tiles`에는 좌표와
  `Ground`/`Platform`/`Obstacle` kind가 함께 노출된다.
- `P`는 지상 배치, `T`는 플랫폼 배치, `#`은 장애물, `N/L/Q/A/B/W/R`은 적 출현 구역,
  `X/Y/Z`는 route endpoint/전술 marker로 해석된다.
- route가 필요한 템플릿은 terrain과 같은 크기의 `routes.overlay`를 작성한다. Overlay는 route의
  시작/종료 marker와 방향 화살표를 표현하며 유효 타일 위에만 놓을 수 있다.

Test-only cached catalogs and `RunPolicyData::builtin()` may support minimal fixtures, but are not
production loaders and must parse/validate the same current schema rather than provide compatibility
or fallback data.

## Fixture Builders

`GameDataBuilder::empty()`와 fixture helper는 테스트용 최소 DB를 만드는 도구다.
`GameDataBuilder::live_defaults()`는 embedded live buff data가 필요한 fixture convenience이며, production live bundle loader가 아니다.

Production/server code는 live data가 필요하면 `GameDataBase::load_live_embedded()`를 사용한다.

## Validation Gates

Live data loading은 gameplay 시작 전에 authoring mistake를 잡는 gate다.

- Parse 단계는 exact manifest version, unknown/missing fields, raw preset resolution을 검사한다.
- Final parsed-bundle validator는 domain shape, duplicate keys, cross-catalog references, map/event/
  reward/RunStart/extra-wave contracts를 검사한다.
- Generated combat preview validation은 모든 encounter를 모든 run-policy scaling stage의 첫
  floor와 고정된 다섯 seed로 생성하고 briefing warning까지 검사한다.
- Publication은 위 단계가 모두 성공한 뒤 한 번만 일어난다. 실패 시 partial bundle,
  fallback bundle, server bind가 없다.
- `tests/ron_loading.rs`와 live catalog tests는 실제 embedded bundle이 runtime 계약을 만족하는지 확인한다.

## Validation Surfaces

- Cross-reference와 generated preview validation: [data/validation.rs](../src/game/data/validation.rs), [combat_preview/validation.rs](../src/game/combat_preview/validation.rs)
- 전체 embedded bundle loading: [tests/ron_loading.rs](../tests/ron_loading.rs)
- Live skill catalog와 activation: [tests/live_skill_catalog_audit.rs](../tests/live_skill_catalog_audit.rs), [tests/live_item_skill_activation.rs](../tests/live_item_skill_activation.rs)
- Skill runtime/data integration: [tests/skill_runtime_contract.rs](../tests/skill_runtime_contract.rs)
- Battle contract validation: [battle/validation](../src/game/battle/validation)

Live RON schema를 바꿀 때는 해당 [data module](../src/game/data), [data/mod.rs](../src/game/data/mod.rs), [data/validation.rs](../src/game/data/validation.rs)를 함께 확인한다. Range/skill authoring 의미가 바뀌면 [skill_target_contract.md](skill_target_contract.md)와 관련 skill validation도 함께 갱신한다.

Live RON schema를 바꿀 때는 보통 다음을 확인한다.

```text
cargo test -p game_core --test ron_loading -- --nocapture
cargo check -p game_core
```

변경이 server startup까지 닿으면 `cargo check -p game_server`도 확인한다.
