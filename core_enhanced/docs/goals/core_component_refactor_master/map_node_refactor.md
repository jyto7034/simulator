# Map / Node Refactor Audit

## Component

- Component: 맵/노드 시스템
- Primary code: `src/game/map/*`
- 기준 문서: `docs/refactor_preparation_plan.md`
- Source-of-truth 확인 순서: runtime code, live RON/data, Unity-facing 계약, 최신 정책 문서

## 읽은 코드와 데이터 범위

Runtime code:

- `src/game/map/types.rs`
- `src/game/map/generator.rs`
- `src/game/map/progression.rs`
- `src/game/map/executor.rs`
- `src/game/map/session.rs`
- `src/game/map/mod.rs`
- `src/game/world/node_flow.rs`
- `src/game/world/map_content.rs`
- `src/game/world/map_encounters.rs`
- `src/game/combat_setup/mission_policy.rs`
- `src/game/world/snapshot.rs`

Live data:

- `../game_resources/data/map/node_definitions.ron`
- `../game_resources/data/pve/encounters.ron`
- `../game_resources/data/events/shops/base.ron`
- `../game_resources/data/events/rewards/base.ron`

Tests and validation references:

- `src/game/map/generator.rs` module tests
- `src/game/map/progression.rs` module tests
- `src/game/map/executor.rs` module tests
- `src/game/map/session.rs` module tests
- `src/game/world/tests/map_flow.rs`
- `src/game/world/tests/node_sessions.rs`
- `src/game/world/tests/snapshots_and_start.rs`
- `tests/ron_loading.rs`

## 현재 구조 요약

`types.rs`가 map domain model의 중심이다. `MapNodeCategory`, `MapNodePayload`, `MapNodeState`, `MapNodeDefinition`, `RunMap`, `MapViewDto`가 여기서 정의된다.

Live authoring data는 `../game_resources/data/map/node_definitions.ron`이다. 각 node definition은 `kind_id`, `category`, `weight`, depth bound, tags, payload를 가진다.

`MapGenerator::generate_with_definitions`는 deterministic seed로 visual start node, playable rows, boss node, row edge를 만든다. 노드 kind/category/payload는 `MapNodeDefinitionDatabase`에서 고르지만, early-depth category distribution, pre-boss support/combat bias, row combat cap, forced support-before-boss 같은 구조 정책은 generator 코드에 있다.

`MapProgression`은 current/available/completed node id를 갖고, `enter_node`와 `complete_current_node`가 `RunMap` 안의 `MapNode.state`도 함께 갱신한다.

`MapNodeExecutor::enter`와 `NodeSession`은 node를 runtime entry result/session projection으로 바꾼다. 실제 Shop/Reward/Support/Maintenance/Combat 실행은 world layer가 담당한다.

`world/node_flow.rs`는 `MapGenerator + assign_map_encounters + MapProgression`을 조합해 run act map을 만들고, select/confirm/complete flow를 오케스트레이션한다.

## Source-of-Truth 판단

Map authoring source는 live RON의 `node_definitions.ron`이다. 단, 이 RON은 node kind/category/payload/weight source이고, 전체 map 구조와 row composition policy의 유일한 source는 아니다.

Map generation structure policy의 canonical source는 현재 `MapGenerator` 코드다. `MapGenerationConfig`, depth별 category 선택, row repair, edge generation은 코드가 결정한다.

Runtime map progression의 canonical source는 `RunState.map`과 `RunState.map_progression`의 조합이다. `MapViewDto`와 snapshot JSON은 이 조합의 Unity-facing projection이다.

`NodeSessionKind`는 현재 `MapNodeCategory`에서 1:1로 파생된다. 독립 source라기보다는 projection에 가깝다.

## 리팩토링 후보

### 1. `MapNodeCategory`와 `MapNodePayload` 호환성 load-time validation 추가

Evidence:

- `MapNodeDefinitionDatabase::validate_contract`는 empty database, duplicate kind id, boss 존재, weighted non-boss 존재만 검사한다.
- `MapNodePayload`는 Support, Maintenance, HeadquartersContact, Encounter, Shop, Reward payload를 표현한다.
- `world/map_content.rs::try_enter_map_content_node`는 category를 먼저 match한 뒤 payload가 맞지 않으면 `Ok(None)`을 반환한다.
- Combat/Boss encounter assignment도 `MapNodePayload::Encounter { encounter_id: None }`인 경우에만 payload를 채운다.

판단:

- `category`와 `payload`가 함께 node behavior를 결정하므로 둘의 mismatch는 live data 오류다.
- runtime에서 조용히 `Ok(None)`으로 지나가면 node definition 오류가 늦게 드러난다.
- `docs/refactor_preparation_plan.md` 기준상 runtime fallback보다 load-time validation failure가 더 낫다.

권장:

- `MapNodeDefinitionDatabase::validate_contract`에 category-payload compatibility 검증을 추가한다.
- 예: `Support`는 `MapNodePayload::Support`, `Maintenance`는 `Maintenance`, `HeadquartersContact`는 `HeadquartersContact`, `Shop`은 `Shop`, `Reward`는 `Reward`, `Combat | Boss`는 `Encounter`, `Start`는 `None`.
- `SupportNodeMode::LimitedChoice`는 non-empty choices, `FullChoice`는 choices 무시 또는 empty 허용 같은 세부 정책은 support component에서 다시 확인한다.

검증:

- `cargo test -p game_core map::generator::tests::builtin_node_definitions_validate_contract`
- `cargo test --test ron_loading live_map_content_pools_are_safe_and_resolve`

### 2. Map generation policy의 코드/RON 분산 명확화

Evidence:

- `node_definitions.ron`은 각 node definition의 weight와 depth bound를 갖는다.
- `MapGenerator::roll_node_definition`은 early depth에서 Combat/Support/HeadquartersContact를 50/25/25로 먼저 고른 뒤 해당 category 안에서 RON weight를 적용한다.
- pre-boss depth에서는 Support 65%, Combat 35%로 category를 먼저 고른다.
- `repair_row_distribution`은 row combat cap을 적용하고, pre-boss row에 Support가 없으면 강제로 Support를 넣는다.
- `roll_safe_definition`은 safe category 후보 순서를 코드 배열로 가진다.

판단:

- 이것은 현재 source-of-truth duplication이라고 단정하기보다, map generation policy가 code와 data로 나뉘어 있는 상태다.
- 노드별 등장 weight를 조정하려면 RON을 보고, row/category 정책을 조정하려면 code를 봐야 한다.
- 밸런스/UX 의미가 강하므로 data-driven으로 옮기는 것은 즉시 구현하지 않는다.

권장:

- 단기적으로는 code policy 유지.
- 대신 `MapGenerationPolicy` 같은 명시적 내부 struct/const로 category distribution, row repair, safe category preference를 모아 "코드 내부 source"를 더 읽기 쉽게 만드는 후보로 둔다.
- RON schema로 옮기려면 `사용자와 정책 논의 필요`.

검증:

- `cargo test -p game_core map::generator`
- `cargo test -p game_core world::tests::map_flow`

### 3. `MapNode.state`와 `MapProgression` 병렬 runtime state invariant 정리

Evidence:

- `MapNode`는 `state: MapNodeState`를 가진다.
- `MapProgression`은 `current_node_id`, `available_node_ids`, `completed_node_ids`를 가진다.
- `enter_node`는 선택 노드의 `MapNode.state`를 `Revealed`로 바꾸고 `available_node_ids`를 선택 노드 하나로 좁힌다.
- `complete_current_node`는 node state를 `Completed`로 바꾸고, outgoing nodes를 `Available`로 바꾸며, progression vectors도 갱신한다.
- `MapViewDto`는 per-node state와 `available_node_ids`/`completed_node_ids`를 모두 노출한다.
- visual start node는 `Completed` 상태로 생성되지만 `completed_node_ids`에는 들어가지 않는다.

판단:

- `MapNode.state`와 `MapProgression`은 서로 같은 의미의 일부를 표현하므로 drift 위험이 있다.
- 다만 Unity map view는 node별 state와 id lists를 함께 쓰는 projection일 수 있어, DTO 중복 자체가 canonical duplication은 아니다.

권장:

- 즉시 DTO shape를 줄이지 않는다.
- `MapProgression`에 invariant check helper 또는 `apply_to_map`/`view_state_for_node` 계열 helper를 추가해 state/id-list consistency를 테스트로 고정하는 후보로 둔다.
- visual start node의 `Completed` 상태가 UI-only sentinel인지 명확히 문서화하거나 test 이름에 드러낸다.

검증:

- `cargo test -p game_core map::progression`
- `cargo test -p game_core world::tests::map_flow`
- Unity-facing DTO shape 변경은 `사용자와 정책 논의 필요`.

### 4. `NodeSessionKind`의 독립 의미 여부 재검토

Evidence:

- `NodeSessionKind`는 `MapNodeCategory`와 현재 1:1 mapping이다.
- `NodeSession`은 `category`, `session_kind`, `payload`를 모두 가진다.
- `MapNodeEnterResult`는 category, payload, session을 함께 반환한다.

판단:

- 현재 코드만 보면 `session_kind`는 `category`의 derived projection이다.
- 별도 의미가 없다면 source-of-truth를 늘리는 필드일 수 있다.
- 다만 `NodeSession`은 `BehaviorResult::NodePreview`와 snapshot에 노출될 수 있어 Unity-facing 계약이다.

권장:

- 즉시 제거하지 않는다.
- Unity/server-facing DTO 감사에서 `session_kind` 소비 여부를 확인한다.
- 제거하거나 rename하려면 `사용자와 정책 논의 필요`.

검증:

- `rg -n "session_kind|NodeSessionKind" src tests '/mnt/f/unity projects/ark/docs'`

### 5. Encounter assignment가 `kind_id.contains("elite")`에 의존하는 문제

Evidence:

- `world/map_encounters.rs::encounter_candidates_for_map_node`는 `kind_id.contains("elite")`로 elite 여부를 판단해 difficulty range를 바꾼다.
- `CombatMissionPolicy::preferred_node_types_for_map_node`도 `kind_id.contains("elite")`를 본다.
- `node_definitions.ron`에는 `combat_elite` kind id와 `tags: ["combat", "elite"]`가 이미 있다.

판단:

- elite semantics가 string naming convention과 tags에 동시에 암시돼 있다.
- 현재 tags는 `MapNodeDefinition`에는 있지만 generated `MapNode`에는 복사되지 않으므로 encounter assignment 단계에서는 kind_id 문자열을 보고 있다.
- 이는 source-of-truth duplication과 implicit convention 위험이 있다.

권장:

- 단기 후보: `MapNode`에 tags를 복사하지 않고도 generation 시점에 encounter assignment에 definition metadata를 넘길 수 있는지 검토한다.
- 더 단순한 후보: `MapNodeKindId` helper나 `MapNodeDefinition` method로 `is_elite` 판정을 한 곳에 모은다.
- RON schema나 runtime DTO에 tags를 노출하는 방향은 `사용자와 정책 논의 필요`.

검증:

- `cargo test -p game_core world::tests::map_flow generated_map_assigns_pve_encounters_to_combat_and_boss_nodes`
- `cargo test -p game_core combat_setup::mission_policy`

## 기존 기능 조합으로 단순화 가능한 후보

- `MapNodeCategory`와 `NodeSessionKind`는 현재 같은 category mapping으로 표현된다. Unity-facing 계약만 허용한다면 `session_kind`는 기존 category projection으로 대체할 수 있다.
- `MapNode.state`는 일부 상태를 `MapProgression`으로부터 도출할 수 있다. 다만 node별 state는 Unity map rendering에 직접 유용하므로, 삭제보다 invariant/helper 정리가 우선이다.

## 레거시, fallback, compatibility layer, dual schema 제거 후보

- `try_enter_map_content_node`에서 category/payload mismatch를 `Ok(None)`으로 넘기는 흐름은 live data 오류를 숨기는 fallback처럼 동작할 수 있다. load-time validation으로 막는 쪽이 좋다.
- non-live combat node flow는 world combat layer에서 이미 reject하므로, map layer에서 replay-only compatibility를 추가할 필요는 없다.

## 하지 않거나 보류한 항목과 이유

- `MapNode.state`를 즉시 삭제하지 않는다. Unity-facing `MapViewDto`에 노출되는 projection이며 UI map rendering 의미가 있다.
- `NodeSessionKind`를 즉시 삭제하지 않는다. Unity-facing DTO/command-result 소비 여부를 확인해야 한다.
- map generation policy를 곧바로 RON으로 옮기지 않는다. map balance와 UX 의미가 바뀔 수 있다.

## 사용자와 정책 논의 필요

- Map generation category distribution, row repair, safe-node preference를 live RON/data-driven policy로 옮길지 여부.
- `NodeSessionKind`를 Unity-facing 계약에서 제거하거나 `MapNodeCategory`로 대체할지 여부.
- `MapViewDto`에서 per-node state와 `available_node_ids`/`completed_node_ids` 중복 projection을 줄일지 여부.
- node definition tags를 generated map/runtime DTO에 보존하거나 노출할지 여부.

## 필요한 테스트와 검증 명령

문서 감사만 수행했으므로 아직 테스트는 실행하지 않았다.

후속 구현이 생기면 우선순위는 아래와 같다.

- `cargo test -p game_core map::generator`
- `cargo test -p game_core map::progression`
- `cargo test -p game_core map::session`
- `cargo test -p game_core map::executor`
- `cargo test -p game_core world::tests::map_flow`
- `cargo test --test ron_loading live_map_content_pools_are_safe_and_resolve`

## 후속 대조 필요

- `world_run_progression_refactor.md`: `GameState`, `ActiveNodeContent`, node session projection과 map progression의 관계.
- `behavior_state_gate_refactor.md`: `NodeConfirm`에서 `SelectMapNode` 재선택 허용 등 state/action policy.
- `battlefield_scenario_wave_refactor.md`: map encounter assignment, combat preview, battle scenario handoff.
- `unity_server_contract_refactor.md`: `MapViewDto`, `NodeSession`, snapshot `map_progression` shape.
