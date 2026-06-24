# World / Run Progression Refactor Audit

## Component

- Component: 월드/런 진행 시스템
- Primary code: `src/game/world.rs`, `src/game/world/*`
- 기준 문서: `docs/refactor_preparation_plan.md`
- Source-of-truth 확인 순서: runtime code, live RON/data, Unity-facing 계약, 최신 정책 문서

## 읽은 코드와 데이터 범위

Runtime code:

- `src/game/world.rs`
- `src/game/world/state.rs`
- `src/game/world/node_flow.rs`
- `src/game/world/map_content.rs`
- `src/game/world/combat.rs`
- `src/game/world/support.rs`
- `src/game/world/snapshot.rs`
- `src/game/world/helpers.rs`
- `src/game/managers/action_scheduler.rs`
- `src/game/resources/state.rs`
- `src/game/resources/action.rs`

Live data:

- `../game_resources/data/map/node_definitions.ron`
- `../game_resources/data/employees/starter_candidates.ron`
- `../game_resources/data/pve/encounters.ron`
- `../game_resources/data/events/shops/base.ron`
- `../game_resources/data/events/rewards/base.ron`

Tests and validation references:

- `src/game/world/tests/map_flow.rs`
- `src/game/world/tests/snapshots_and_start.rs`
- `src/game/world/tests/support.rs`
- `src/game/world/tests/equipment.rs`
- `src/game/world/tests/combat.rs`
- `src/game/world/tests/node_sessions.rs`
- `tests/ron_loading.rs`

## 현재 구조 요약

`GameCore::execute_with_source_command_id`가 모든 `PlayerBehavior`의 runtime entry point다. 먼저 `ActionValidator`가 `ActionKind` 허용 여부를 보고, 그 다음 `validate_behavior_payload`가 payload를 검증한 뒤 각 world handler로 dispatch한다.

런 시작은 `handle_start_new_game`과 `handle_select_starter_employees`가 담당한다. 시작 직원 후보는 live RON의 starter candidate data에서 오고, starter count, starter enkephalin, max acts는 `RUN_SYSTEM_POLICY` 상수에서 온다.

맵 흐름은 `node_flow.rs`가 중심이다. `generate_current_act_map`이 `MapGenerator`와 `assign_map_encounters`를 조합해 act map을 만들고, `handle_select_map_node`는 `NodeSession`과 preview를 만들며 `NodeConfirm`으로 이동한다. `handle_confirm_enter_node`는 `MapProgression::enter_node`로 노드를 소비하고, combat이면 live battle로 넘기고 safe node면 `try_enter_map_content_node`로 Shop/Reward/Support/Maintenance/HeadquartersContact를 라우팅한다.

노드 완료는 `handle_complete_node`가 담당한다. support effect를 적용하고 `MapProgression::complete_current_node` 결과에 따라 `NodeCompleted`, `ActComplete`, `RunComplete`를 반환한다. 전투 결과는 `handle_complete_combat_result`가 사후 보상, 부상, XP, consumable 감소를 처리한 뒤 노드 완료 흐름으로 복귀한다.

Unity-facing snapshot은 `snapshot.rs`가 `GameState`, `node_session`, `active_node_content`, `RunState`, inventory, roster를 JSON projection으로 묶어 만든다.

## Source-of-Truth 판단

현재 월드/런 진행의 canonical runtime state는 `GameCoreState` 안의 `game_state`, `run`, `node_session`, `active_node_content`, `active_battle` 조합이다. `GameState`만으로는 support/maintenance/headquarters 같은 safe node 세부 상태를 완전히 설명하지 못하고, `active_node_content`와 allowed action context가 함께 최종 상태 의미를 만든다.

`BehaviorResult`와 snapshot JSON은 canonical source가 아니라 runtime state의 projection이다. 이 projection들은 Unity-facing 계약이므로 shape 변경은 별도 정책 판단이 필요하다.

`RUN_SYSTEM_POLICY`는 현재 여러 런 정책의 canonical source다. 같은 값들이 world tests에서 기대값으로 반복 참조되지만, 테스트가 상수를 직접 참조하고 있어 값 duplication은 크지 않다. 다만 starter setup, live deployment, post-battle injury/XP, support healing, headquarters supplies가 한 상수 묶음에 섞여 있어 정책 축이 커지고 있다.

Live RON의 `map/node_definitions.ron`은 node category, weight, payload의 canonical data다. runtime `map_content.rs`는 payload를 해석해 session으로 바꾸는 canonical execution logic이다.

## 리팩토링 후보

### 1. `RUN_SYSTEM_POLICY` 정책 묶음 분리 검토

Evidence:

- `src/game/world.rs`의 `RUN_SYSTEM_POLICY`는 setup, live deployment, post battle, support, headquarters policy를 한 상수에 함께 둔다.
- `handle_select_starter_employees`는 starter enkephalin과 default max acts를 사용한다.
- `handle_map_combat_node`는 live deployment policy를 `ActiveBattleSession`에 주입한다.
- `apply_post_battle_resolution`은 survival XP, trauma, HP loss policy를 사용한다.
- `support.rs`와 `headquarters.rs`는 support/headquarters 값을 사용한다.
- `src/game/world/tests/*`는 이 상수를 많이 참조해 현재 값들을 고정한다.

판단:

- 같은 데이터가 둘 이상의 canonical source로 동기화되는 상태는 아니다.
- 그러나 정책 축이 서로 다른 수치들이 하나의 world 상수에 섞여 있어 변경 이유가 다른 값들이 같은 위치에 모인다.
- live RON schema로 옮기면 schema/balance 의미가 생기므로 즉시 구현하지 않는다.

권장:

- 단기적으로는 코드 상수 유지.
- 다음 refactor 후보로 `RunSetupPolicy`, `PostBattleResolutionPolicy`, `SupportPolicy`, `HeadquartersPolicy`, `LiveBattleDeploymentPolicy`의 소유 컴포넌트를 더 명확히 나누는 것을 검토한다.
- data-driven으로 이동하려면 `사용자와 정책 논의 필요`.

검증:

- `cargo test -p game_core world::tests::snapshots_and_start`
- `cargo test -p game_core world::tests::map_flow`
- `cargo test -p game_core world::tests::combat`

### 2. `GameState` + `ActiveNodeContent` + allowed action context의 병렬 상태 정리 검토

Evidence:

- `GameState`는 `ViewingMap`, `NodeConfirm`, `InNode`, `InShop`, `InReward`, `CombatResult`, `InBattle` 등을 표현한다.
- `ActiveNodeContent`는 Shop/Reward/Support/Maintenance/HeadquartersContact/CombatBattle 같은 현재 노드 세부 session을 따로 가진다.
- Shop/Reward는 `GameState::InShop`/`InReward`로 전환하지만, Support/Maintenance/HeadquartersContact는 `active_node_content`를 설정한 뒤 `refresh_allowed_actions`로 `InNode`의 allowed actions를 context 보정한다.
- `ActionScheduler::get_allowed_actions_for_context`는 `reward_can_skip`, `in_maintenance_node` 같은 context를 받아 `GameState` 기반 action list를 보정한다.
- Snapshot은 `game_state_context`, `current_node_session`, `selected_event`, `allowed_actions`를 각각 projection한다.

판단:

- 이것은 당장 source-of-truth duplication이라고 단정하기 어렵다. `GameState`는 coarse flow, `ActiveNodeContent`는 node-specific session이므로 수명주기가 다르다.
- 다만 최종 allowed actions와 snapshot flow 의미가 `GameState` 단독이 아니라 여러 필드 조합에서 파생되므로, 수정 시 여러 projection이 같이 바뀌어야 한다.

권장:

- 즉시 타입 통합은 하지 않는다.
- 대신 `GameCore` 내부에 `current_flow_context` 또는 `active_node_flow_view`처럼 `GameState + node_session + active_node_content + active_battle`을 읽기 전용으로 묶어 allowed actions/snapshot/validation이 같은 helper를 사용하도록 하는 작은 리팩토링을 검토한다.
- 이 후보는 `행동/상태 게이트 시스템`과 `Unity/server-facing DTO/계약 표면` 컴포넌트 감사에서 다시 대조해야 한다.

검증:

- `cargo test -p game_core world::tests::snapshots_and_start`
- `cargo test -p game_core world::tests::map_flow`
- `cargo test -p game_core world::tests::support`
- Unity snapshot shape를 바꾸지 않는 범위에서만 진행한다.

### 3. Safe node content preview와 execution 규칙의 분산 감사

Evidence:

- `map_content.rs`는 Shop/Reward/Support/Maintenance/HeadquartersContact payload를 session과 `BehaviorResult`로 바꾼다.
- `support.rs`는 SupportState projection뿐 아니라 MaintenanceState projection과 maintenance preview를 만든다.
- `maintenance_materials_snapshot`은 `"fragment_dust"`와 `"equipment_dust"` material id를 직접 문자열로 가진다.
- 실제 장비/파편 조작은 `maintenance.rs`에 있다.

판단:

- Maintenance가 Support category와 분리된 뒤에도 일부 preview/build logic이 `support.rs`에 남아 있다.
- 이것은 runtime behavior 중복은 아니지만 파일 책임과 변경 축이 섞인 상태다.

권장:

- `maintenance_options`, `maintenance_state_result_with_deliveries`, maintenance preview helper를 `maintenance.rs` 또는 별도 maintenance projection 모듈로 이동하는 후보로 남긴다.
- `"fragment_dust"`/`"equipment_dust"` 문자열은 inventory/skill fragment component 감사에서 canonical material id로 고정할지 확인한다.

검증:

- `cargo test -p game_core world::tests::support`
- `cargo test -p game_core world::tests::equipment`

### 4. Reward 정책 흐름의 source-of-truth 감사 필요

Evidence:

- Map reward node는 `resolve_map_reward`에서 reward pool을 고르고 `Forbidden`과 `Experience` tag를 제외한다.
- Combat reward는 `CombatExecutor::resolve_rewards_for_mission`을 통해 reward mode와 reward list를 만든다.
- Combat result completion은 reward 안의 `GrantExperience`를 별도 추출해 생존 직원에게 적용한다.
- `tests/ron_loading.rs`는 live PVE reward가 forbidden legacy reward를 주지 않도록 검증한다.

판단:

- reward source가 실제로 중복인지, map reward와 combat reward의 수명주기가 다른지는 `보상/상점/지원 노드 시스템` 컴포넌트에서 더 정확히 봐야 한다.
- 현재 컴포넌트에서는 world flow가 reward source들을 조합하는 위치라는 점만 확인했다.

권장:

- 이 항목은 현재 컴포넌트에서 구현하지 않고 `reward_shop_support_refactor.md`에서 재검토한다.

검증:

- `cargo test --test ron_loading live_pve_references_resolve`
- `cargo test --test ron_loading live_map_content_pools_are_safe_and_resolve`

### 5. Battle record export의 gameplay source-of-truth 여부 명확화

Evidence:

- `RunState::record_battle`은 `battle_records/run_<seed>/<battle_uuid>.json` 파일을 생성하고 `battle_records` 벡터에도 push한다.
- `docs/refactor_preparation_plan.md`는 debug/export 산출물을 golden fixture나 gameplay source-of-truth로 보지 않는다는 기준을 둔다.

판단:

- `battle_records` 벡터는 run snapshot/result flow의 runtime state로 보인다.
- JSON file export는 debug/archive side effect에 가깝고, gameplay source-of-truth로 사용하면 안 된다.

권장:

- 즉시 제거하지 않는다.
- 이후 전투 결과/테스트 하네스 감사에서 battle record file export가 테스트 golden이나 gameplay source로 쓰이는지 확인한다.
- 파일 생성 policy를 바꾸려면 저장/디버그 산출물 의미가 바뀌므로 `사용자와 정책 논의 필요`.

검증:

- `rg -n "battle_records|record_battle|BattleRecordExport" src tests docs`

## 기존 기능 조합으로 단순화 가능한 후보

현재 월드/런 진행 컴포넌트만 놓고는 명확한 "기존 기능 조합으로 특수 로직을 제거" 후보를 확정하지 않았다.

다만 `GameState`, `ActiveNodeContent`, `AllowedActionContext`, snapshot projection이 함께 최종 flow 의미를 만드는 구조는 작은 읽기 전용 flow-context helper로 중복 판단과 projection을 줄일 수 있어 보인다. 이 변경은 public DTO shape를 바꾸지 않고 내부 projection/allowed-action 계산의 반복을 줄이는 방향이어야 한다.

## 레거시, fallback, compatibility layer, dual schema 제거 후보

- `handle_map_combat_node`는 non-live combat mission을 명시적으로 reject하며, 공식 흐름이 DefenseRoute live battle only라는 현재 정책과 맞다. compatibility path를 되살릴 필요는 없다.
- `tests/ron_loading.rs`의 forbidden legacy reward 방지 검증은 live data validation으로 유효하다.
- `battle_records/*.json` export가 legacy replay/golden source로 쓰이는지 후속 감사가 필요하다.

## 하지 않거나 보류한 항목과 이유

- `GameState`와 `ActiveNodeContent`를 하나의 enum으로 즉시 합치지 않는다. coarse flow와 node session detail은 수명주기가 다를 수 있고, Unity-facing snapshot/allowed action 계약까지 영향이 크다.
- `RUN_SYSTEM_POLICY`를 live RON으로 즉시 이동하지 않는다. 밸런스, schema, migration, Unity display 의미가 엮일 수 있다.
- Reward 흐름을 현재 컴포넌트에서 정리하지 않는다. reward/shop/support component 감사와 겹친다.

## 사용자와 정책 논의 필요

- `RUN_SYSTEM_POLICY`의 setup, support, post-battle, headquarters, live deployment 수치를 live RON/data-driven policy로 옮길지 여부.
- `battle_records/run_<seed>/*.json` file export를 유지, debug-only gate, 삭제 중 무엇으로 볼지 여부.

## 필요한 테스트와 검증 명령

문서 감사만 수행했으므로 아직 테스트는 실행하지 않았다.

후속 구현이 생기면 우선순위는 아래와 같다.

- `cargo test -p game_core world::tests::snapshots_and_start`
- `cargo test -p game_core world::tests::map_flow`
- `cargo test -p game_core world::tests::support`
- `cargo test -p game_core world::tests::equipment`
- `cargo test -p game_core world::tests::combat`
- `cargo test --test ron_loading live_map_content_pools_are_safe_and_resolve`
- `cargo test --test ron_loading live_pve_references_resolve`

## 후속 대조 필요

- `behavior_state_gate_refactor.md`: allowed action source-of-truth, payload validation, handler-level validation 중복 여부.
- `reward_shop_support_refactor.md`: reward filtering, reward mode, combat XP reward, support/maintenance node 책임 분리.
- `unity_server_contract_refactor.md`: snapshot `game_state_context`, `selected_event`, `allowed_actions` projection이 Unity-facing 계약과 일치하는지.
- `validation_test_harness_refactor.md`: battle record export와 debug/golden fixture 분리.
