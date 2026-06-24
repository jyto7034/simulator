# 전장/시나리오/웨이브 시스템 Refactor Audit

## Scope

- 기준 문서: `docs/refactor_preparation_plan.md`
- Runtime code:
  - `src/game/combat_preview/mod.rs`
  - `src/game/combat_preview/types.rs`
  - `src/game/combat_preview/template.rs`
  - `src/game/combat_preview/validation.rs`
  - `src/game/combat_preview/threat.rs`
  - `src/game/combat_setup/battlefield_plan.rs`
  - `src/game/combat_setup/enemy_spawns.rs`
  - `src/game/combat_setup/defense_object.rs`
  - `src/game/combat_setup/mission_policy.rs`
  - `src/game/combat_setup/scenario_groups.rs`
  - `src/game/battle/scenario.rs`
  - `src/game/wave_resolution.rs`
  - `src/game/events/combat.rs`
  - `src/game/world/combat.rs`
  - `src/game/data/pve_data.rs`
  - `src/game/data/corroded_wave_data.rs`
  - `src/game/data/validation.rs`
- Live RON/data touchpoints:
  - `../game_resources/data/pve/encounters.ron`
  - `../game_resources/data/map/battlefield_templates.ron`
  - `../game_resources/data/map/battlefield_archetypes.ron`
  - `../game_resources/data/enemies/corroded_wave_presets.ron`
  - `../game_resources/data/enemies/corroded_employees.ron`
- Unity-facing contract:
  - `/mnt/f/unity projects/ark/docs/unity_core_contract.md`
  - `/mnt/f/unity projects/ark/docs/core_unity_battle_transport_contract.md`
  - `docs/game_rulebook.md`
- Tests:
  - `src/game/combat_preview/mod.rs` module tests
  - `src/game/world/tests/combat.rs`
  - `src/game/world/tests/map_flow.rs`
  - `tests/ron_loading.rs`

## Current Structure

- `src/game/combat_preview/types.rs:209`의 `BattlefieldInstance`가 generated battlefield, tiles, valid tiles, deployment zones, spawn zones, routes, spawn waves, briefing, warnings를 한 번에 담는다.
- `src/game/combat_preview/types.rs:241`의 `CombatPreview`는 `BattlefieldInstance`를 전투 전 UI/브리핑용 DTO로 투영한다.
- `src/game/combat_preview/mod.rs:280`의 `BattlefieldGenerator::try_generate`가 instance를 만들고 `validate_instance`를 통과시킨다.
- `src/game/combat_preview/mod.rs:299`의 `build_instance`는 authored PVE encounter override, battlefield archetype/size, node type, mission variant, template parse, static obstacle override, defense route, spawn wave, enemy briefing, threat warning을 조합한다.
- `src/game/combat_preview/template.rs:12`는 shared RON ASCII template rows와 optional `routes.overlay`를 canonical tile/zone/route structures로 파싱한다.
- `src/game/combat_preview/validation.rs:7`은 generated instance의 dimensions, tile set, obstacle, deployment/spawn zone, route, wave route/zone references, pathability를 검증한다.
- `src/game/combat_preview/mod.rs:645`의 `spawn_waves_for`는 encounter authored waves가 있으면 그것을 resolved `SpawnWave`로 바꾸고, 없으면 empty fallback wave를 만든다.
- `src/game/wave_resolution.rs:12`의 `resolve_pve_wave_enemy_data`는 `PveWaveSource::Manual`과 `GeneratedCorroded`를 모두 `PveWaveEnemyData` 목록으로 확정한다.
- `src/game/combat_setup/battlefield_plan.rs:23`의 `BattleStartPlan::from_preview`는 preview의 dimensions, valid tiles, obstacles를 battle scenario field spec으로 옮긴다.
- `src/game/events/combat.rs:233`의 `build_battle_scenario_from_preview`는 preview와 encounter를 검증하고, player start group, tactical plan, win condition, defense object, enemy spawn groups를 조립한다.
- `src/game/combat_setup/enemy_spawns.rs:29`의 `enemy_spawn_groups_from_preview`는 preview의 `spawn_waves`를 sorted timed scenario groups로 변환한다.
- `src/game/battle/scenario.rs:204`의 `BattleScenario::validate`는 battlefield, groups/events, tactical plan, win condition을 다시 검증한다.
- `src/game/world/combat.rs:50`의 `combat_preview_for_node`는 generated preview를 run state cache에 저장하고 같은 node의 source로 재사용한다.
- `src/game/world/combat.rs:122`의 combat node entry path는 cached preview를 사용해 rewards를 resolve하고 `CombatExecutor::build_battle_with_combat_preview`로 live battle을 시작한다.

Unity-facing 정책은 pre-battle preview와 live battle setup을 분리한다. `docs/game_rulebook.md:202`는 전투 시작 후 scene construction source를 `battle_setup_snapshot.battlefield`, `routes`, `deployment_zones`, `spawn_zones`로 고정하고, `:203`은 `combat_preview`를 전투 전 브리핑/미리보기 source로 제한한다. `/mnt/f/unity projects/ark/docs/unity_core_contract.md:636`은 authored `routes.overlay`가 없는 Defense template에 대해 deterministic generated route를 허용하되 `[start, end]`만 가진 직선 fallback은 공식 계약이 아니라고 명시한다.

## Source Of Truth Judgment

- Battlefield authoring source of truth는 `../game_resources/data/map/battlefield_templates.ron`의 ASCII `rows`와 optional `routes.overlay`다.
- Archetype/size default policy source는 `../game_resources/data/map/battlefield_archetypes.ron`이다. 단, random archetype selection policy는 아직 `src/game/combat_preview/mod.rs:589`의 hard-coded `seed % 6` code policy에 남아 있다.
- Generated runtime battlefield source of truth는 `BattlefieldInstance`다. `CombatPreview`와 `battle_setup_snapshot`은 각각 pre-battle UI와 live battle scene construction을 위한 파생 DTO다.
- PVE encounter wave authoring source는 `PveEncounter.waves`이며, generated corroded wave는 `CorrodedWavePreset` + preview seed + wave index로 deterministic하게 `SpawnWaveEnemyEntry`로 확정된다.
- Battle scenario field/group source는 battle start 시점의 `CombatPreview`다. `BattleStartPlan`과 `enemy_spawn_groups_from_preview`가 preview에서 scenario로 handoff한다.
- Tactical objective, win condition, reward policy는 preview가 아니라 `PveEncounter`와 `CombatMissionPolicy`/`CombatRewardPolicy`에서 온다. 이는 battlefield/wave DTO와 수명주기가 달라 단순 중복으로 보지 않는다.
- Unity runtime scene construction source는 `battle_setup_snapshot`이다. `combat_preview`가 in-battle snapshot에도 남아 있지만 runtime actor/current-state source로 쓰면 안 된다.

## Refactor Candidates

### 1. Live RON wave route/spawn-zone reference validation을 preview generation에만 두지 않기

근거:

- `PveWaveData`는 `spawn_zone_ids`와 `route_id`를 authoring할 수 있다 (`src/game/data/pve_data.rs:61`).
- load-time authoring contract는 empty id, duplicate zone id, empty route id를 잡는다 (`src/game/data/pve_data.rs:362`).
- live cross-reference validation은 enemy/preset/reward와 mission compatibility를 잡지만, authored `spawn_zone_ids`와 `route_id`가 selected battlefield template에 실제로 존재하는지는 직접 검증하지 않는다 (`src/game/data/validation.rs:321`, `:390`).
- 이 참조 오류는 `BattlefieldGenerator::try_generate`가 preview를 만들고 `validate_instance`를 돌릴 때 잡힌다 (`src/game/combat_preview/mod.rs:280`, `src/game/combat_preview/validation.rs:176`).
- `tests/ron_loading.rs:1028`의 live preview threat-warning test가 모든 live encounter와 여러 seed를 생성하지만, 함수명과 직접 목적은 threat warning contract다.

판단:

- 현재 runtime은 오류를 숨기지 않고 preview generation에서 실패하므로 live gameplay fallback은 아니다.
- 그러나 source-of-truth 관점에서 authored wave reference 검증이 "preview side effect"에 묶여 있어 데이터 검증 의도가 잘 드러나지 않는다.
- `GameDataBase::validate_generated_combat_preview_contracts`를 threat warning 전용에서 live preview contract umbrella로 확장하거나, 별도 `validate_pve_battlefield_wave_contract`를 추가해 route/spawn-zone reference 검증 목적을 명시하는 것이 좋다.
- 이 변경은 behavior 변경 없이 validation intent를 강화하는 리팩토링 후보다.

필요 검증:

- `cargo test -p game_core --test ron_loading live_pve_references_resolve`
- `cargo test -p game_core --test ron_loading live_combat_previews_include_required_ad_ap_threat_warning_tags`
- invalid authored wave route/spawn-zone를 가진 fixture가 validation에서 명확한 panic/error를 내는 focused test.

### 2. `spawn_waves_for`의 empty fallback wave 정책 정리

근거:

- live `PveEncounterDatabase::validate_indexes`는 encounter가 wave를 하나 이상 가져야 한다고 assert한다 (`src/game/data/pve_data.rs:362`).
- `tests/ron_loading.rs:531`도 live encounter의 node type과 wave references를 확인한다.
- 그럼에도 `spawn_waves_for`는 encounter가 없거나 authored waves가 비어 있으면 enemy_entries가 빈 fallback `wave_0`을 만든다 (`src/game/combat_preview/mod.rs:645`).
- `enemy_spawn_groups_from_preview`는 empty enemy entries를 invalid static data로 거부한다 (`src/game/combat_setup/enemy_spawns.rs:54`).

판단:

- "encounter가 없는 combat preview"는 map generation/debug/test path에서 아직 쓰일 수 있으므로 즉시 제거하면 blast radius가 있다.
- 하지만 official map combat node는 `MapNodePayload::Encounter`를 통해 encounter id를 넘기고, live encounter는 waves가 필수다.
- fallback wave는 live battle까지 갈 수 없는 placeholder이므로, 목적을 test/debug preview용으로 더 명확히 이름 붙이거나, official combat entry path에서 encounter 없는 preview가 battle start까지 가지 못한다는 invariant를 고정해야 한다.
- live path에서 더 이상 필요 없다고 확인되면 fallback wave 대신 `InvalidStaticData`로 실패시키는 것이 `docs/refactor_preparation_plan.md`의 fallback 제거 기준에 맞다.

정책 영향:

- encounter 없는 combat preview를 계속 UI/debug에서 허용할지, live code에서 완전히 금지할지는 콘텐츠/UX 의미가 있어 `사용자와 정책 논의 필요`.

필요 검증:

- encounter 없는 generated preview가 어디까지 허용되는지 map preview/UI test로 고정.
- official combat entry가 encounter 없는 combat node를 battle start로 진행하지 못한다는 focused world-flow test.

### 3. Archetype random selection policy를 RON default policy와 합치기

근거:

- `BattlefieldGenerationPolicyDatabase`는 `battlefield_archetypes.ron`을 읽고 size classes와 archetypes를 validate한다 (`src/game/combat_preview/mod.rs:91`).
- 하지만 non-boss random archetype selection은 `archetype_for`의 hard-coded `seed % 6` table에 있다 (`src/game/combat_preview/mod.rs:589`).
- `SplitRoom`은 archetype policy와 templates에는 존재하지만 random selection table에는 들어가지 않는다 (`src/game/combat_preview/mod.rs:133`, `../game_resources/data/map/battlefield_templates.ron:286`).
- `CombatMissionPolicy::try_fallback_node_type_for_archetype`는 `SplitRoom` fallback을 explicit error로 둔다 (`src/game/combat_setup/mission_policy.rs:77`).

판단:

- 현재 구조는 "어떤 archetype들이 존재하는가"와 "random generation에서 어떤 archetype을 뽑는가"가 다른 source에 있다.
- 의도된 제외일 수 있지만, 새 archetype을 추가하거나 배제할 때 RON과 code selection table을 같이 바꿔야 한다.
- 장기적으로 `battlefield_archetypes.ron`에 selection weight/enabled-for-random 같은 필드를 추가하면 source-of-truth를 줄일 수 있다.

정책 영향:

- RON schema 변경, selection weight, `SplitRoom` 포함 여부, 밸런스/콘텐츠 노출 변경이 필요하므로 `사용자와 정책 논의 필요`.

필요 검증:

- random preview seed distribution이 기대 archetype set만 생성하는 test.
- live encounter authored battlefield override가 random policy와 독립적으로 유지되는 test.

### 4. Defense route fallback의 책임 이름 명확화

근거:

- `ensure_defense_route`는 Defense node이고 parsed template routes가 비어 있으면 deterministic `generated_defense_route`를 만든다 (`src/game/combat_preview/mod.rs:383`).
- generated route는 BFS-like path로 valid non-obstacle cells를 따라 만든다 (`src/game/combat_preview/mod.rs:442`).
- `validate_instance`는 Defense spawn wave가 route를 가져야 하고, route cells가 valid/adjacent/obstacle-free여야 한다고 검증한다 (`src/game/combat_preview/validation.rs:141`, `:176`).
- Unity contract는 deterministic generated route를 공식 허용한다 (`/mnt/f/unity projects/ark/docs/unity_core_contract.md:636`).
- `src/game/world/tests/combat.rs:973`은 generated setup route cells와 runtime enemy movement plan cells가 일치함을 검증한다.

판단:

- 이 fallback은 낡은 compatibility layer가 아니라 공식 계약의 deterministic generation path다.
- 제거 대상이 아니다.
- 다만 함수 이름만 보면 "route 없는 Defense template을 조용히 보정"하는 fallback처럼 보인다. `ensure_or_generate_defense_route` 같은 명명이나 module-level doc으로 policy를 더 명확히 하면 좋다.
- 더 나아가 authored route를 요구하는 template군과 generated route를 허용하는 template군을 RON/policy로 나누는 것은 콘텐츠 authoring 정책 변경이다.

정책 영향:

- generated route 허용 범위를 줄이거나 authored route mandatory로 바꾸는 결정은 `사용자와 정책 논의 필요`.

필요 검증:

- generated route가 setup snapshot, scenario tactical plan, runtime movement plan에서 같은 cells를 쓰는 test는 이미 있다. 리팩토링 시 이 test를 유지한다.

### 5. `FacilityEntity` placeholder를 live schema에서 유지할지 정리

근거:

- `PveWaveEnemyData`와 `EnemyKind`는 `FacilityEntity` variant를 갖는다 (`src/game/data/pve_data.rs:40`, `src/game/combat_preview/types.rs:98`).
- `wave_enemy_entries`는 FacilityEntity entry를 만들 수 있다 (`src/game/combat_preview/mod.rs:780`).
- 그러나 `enemy_spawn_groups_from_preview`는 `EnemyKind::FacilityEntity`를 만나면 `MissingResource("FacilityEntityProfile")`로 실패한다 (`src/game/combat_setup/enemy_spawns.rs:163`).
- data validation도 live PVE wave가 FacilityEntity를 쓰면 panic한다 (`src/game/data/validation.rs:369`).

판단:

- runtime live path에서는 금지되어 있고 validation도 막는다. 따라서 현재 live gameplay에는 문제를 숨기는 fallback이 아니다.
- 하지만 schema에는 아직 variant가 열려 있어 새 content author가 사용 가능한 것으로 오해할 수 있다.
- FacilityEntity가 가까운 로드맵에 없다면 live schema에서 제거하거나, 명시적인 deferred/future-only 주석과 validation error를 정책 문서에 연결하는 것이 좋다.

정책 영향:

- RON schema variant 삭제 또는 future content 유지 여부는 콘텐츠/저장 데이터/Unity DTO 의미가 있어 `사용자와 정책 논의 필요`.

필요 검증:

- `tests/ron_loading.rs`의 live PVE reference test가 FacilityEntity 금지를 계속 고정한다.
- schema 삭제를 택하면 deserialization/test fixture 영향 범위를 14번 데이터/RON 컴포넌트에서 함께 검토한다.

### 6. Scenario handoff validation을 preview identity 이상으로 강화

근거:

- `build_battle_scenario_from_preview`는 encounter id, authored node type, authored mission variant가 preview와 맞는지 검증한다 (`src/game/events/combat.rs:284`).
- battlefield dimensions/valid tiles/obstacles는 `BattleStartPlan::from_preview`를 통해 preview에서 scenario로 들어간다 (`src/game/combat_setup/battlefield_plan.rs:23`).
- enemy spawn groups도 preview spawn waves에서 만들어진다 (`src/game/combat_setup/enemy_spawns.rs:29`).
- `BattleScenario::validate`는 최종 scenario의 group/event/tactical/win-condition consistency를 검증한다 (`src/game/battle/scenario.rs:204`).

판단:

- preview가 scenario의 source로 쓰이는 구조 자체는 좋다. 같은 route/wave를 encounter에서 다시 계산하지 않는다.
- 다만 `validate_preview_matches_encounter`라는 이름은 identity check만 하며, handoff 전체 검증은 `BattleScenario::validate`에 분산되어 있다.
- 리팩토링 후보는 새 정책을 만들기보다 `build_battle_scenario_from_preview`의 validation boundary를 문서/함수명으로 선명히 하는 것이다. 예: `validate_preview_identity_against_encounter`와 `scenario.validate()`의 역할을 분리해 이름으로 드러낸다.

필요 검증:

- authored route id, spawn zone id, generated route cells가 preview -> setup snapshot -> scenario tactical plan에서 유지되는 focused tests.
- current tests 중 `generated_defense_route_setup_snapshot_matches_runtime_route_cells`와 live RON preview tests를 유지한다.

## Existing Composition Simplification Candidates

- Generated corroded wave는 `PveWaveSource::GeneratedCorroded` + preset + seed로 preview 단계에서 `SpawnWaveEnemyEntry`로 확정되고, battle scenario는 preview entries만 사용한다. 이 구조는 기존 기능 조합이 잘 된 상태이므로 다시 encounter/preset을 battle setup에서 재조회하는 방향은 피한다.
- `battle_setup_snapshot.routes`와 runtime `EnemyMovementPlan::PathAlongCells`는 preview route cells에서 함께 파생된다. Unity가 route를 재계산하지 않는 현재 계약과 맞으므로 유지한다.
- `CombatPreview`와 `BattlefieldInstance`의 field shape가 비슷한 것은 source-of-truth 중복이 아니라 pre-battle DTO projection이다. 단, battle runtime scene construction은 `battle_setup_snapshot`만 source로 사용해야 한다.
- `PveEncounter`의 tactical plan/win condition이 preview에 들어가지 않는 것도 의도된 분리다. battlefield/wave preview와 mission objective policy는 수명주기가 다르다.

## Legacy, Fallback, Compatibility Removal Candidates

- `spawn_waves_for`의 encounter 없는/empty authored wave fallback은 live path에서 필요한지 재확인이 필요하다. 필요 없다면 runtime placeholder보다 load-time/runtime validation failure가 낫다.
- `FacilityEntity`는 live data validation이 금지하는 future placeholder다. 유지할 정책 근거가 없다면 schema에서 제거하는 후보지만, 정책 결정이 필요하다.
- `SplitRoom`은 RON policy/templates에는 존재하지만 random fallback node type이 disabled 되어 있다. dead content인지 future content인지 정책 확인 전에는 제거하지 않는다.
- generated defense route는 공식 계약에 들어간 path라 제거 후보가 아니다.

## Deferred Or Not Doing

- `CombatPreview`와 `battle_setup_snapshot`을 하나로 합치지 않는다. 전자는 pre-battle briefing source, 후자는 live battle scene construction source다.
- `BattlefieldInstance`와 `CombatPreview`를 당장 분리/통합하지 않는다. `BattlefieldInstance`는 runtime generation intermediate이고 `CombatPreview`는 serialized DTO다.
- `CombatMissionPolicy`를 즉시 RON registry로 옮기지 않는다. 현재 문서와 code comment는 변형이 충분해질 때까지 explicit table/function policy를 유지한다고 본다.
- scenario validation을 전부 data validation으로 끌어올리지 않는다. 일부 검증은 preview seed/template selection 결과가 필요하므로 generated preview validation 단계가 적절하다.
- reward policy는 13번 보상/상점/지원 노드 시스템에서 더 깊게 다룬다.
- threat warning 자체는 combat preview에 걸쳐 있지만, 경고 tag 산출 로직은 14번 데이터/RON 검증과 15번 Unity 계약 표면에서 다시 볼 수 있다.

## Tests And Verification To Add When Refactoring

문서 감사 단계에서는 코드를 바꾸지 않았으므로 테스트를 실행하지 않았다. 실제 리팩토링 시 우선순위 검증은 아래와 같다.

- `cargo test -p game_core --test ron_loading`
- `cargo test -p game_core world::tests::combat`
- `cargo test -p game_core world::tests::map_flow`
- `cargo test -p game_core combat_preview`

추가하면 좋은 focused test:

- authored `spawn_zone_ids`가 selected template에 없으면 live preview contract validation이 명확히 실패하는 test.
- authored `route_id`가 selected template routes에 없으면 live preview contract validation이 명확히 실패하는 test.
- encounter 없는 combat preview가 허용된다면 battle start로는 진행되지 않는 test.
- generated corroded wave가 preview seed와 wave index에 대해 deterministic한 entries/appearance seeds를 만드는 test는 현재 module tests에 있으므로, 함수 이동 시 유지한다.
- setup snapshot routes, scenario tactical enemy path, runtime movement plan이 같은 cells를 공유하는 test는 유지한다.

## Policy Discussion Required

- encounter 없는 combat preview/empty wave fallback을 UI/debug feature로 유지할지, official runtime에서 완전히 금지할지 결정: `사용자와 정책 논의 필요`.
- battlefield archetype random selection을 RON weight/enabled policy로 옮길지, hard-coded `seed % 6` table을 유지할지 결정: `사용자와 정책 논의 필요`.
- `SplitRoom`을 future content로 유지할지, random/live generation에서 활성화할지, 제거할지 결정: `사용자와 정책 논의 필요`.
- `FacilityEntity` PVE enemy schema variant를 future placeholder로 유지할지 제거할지 결정: `사용자와 정책 논의 필요`.
- generated defense route 허용 범위를 유지할지 authored route mandatory로 바꿀지 결정: `사용자와 정책 논의 필요`.
