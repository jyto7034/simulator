# 행동/상태 게이트 시스템 Refactor Audit

## Scope

- 기준 문서: `docs/refactor_preparation_plan.md`
- Runtime code:
  - `src/game/behavior.rs`
  - `src/game/managers/action_scheduler.rs`
  - `src/game/world.rs`
  - `src/game/world/helpers.rs`
  - `src/game/world/snapshot.rs`
  - `src/game/resources/action.rs`
  - `src/game/resources/state.rs`
- Unity-facing contract:
  - `/mnt/f/unity projects/ark/docs/unity_core_contract.md`
- Tests:
  - `src/game/managers/action_scheduler.rs` unit tests
  - `src/game/world/tests/snapshots_and_start.rs`
  - `src/game/world/tests/map_flow.rs`
  - `src/game/world/tests/support.rs`
  - `src/game/world/tests/equipment.rs`
  - `src/game/world/tests/combat.rs`

이 컴포넌트는 live RON/data를 직접 읽어 command availability를 결정하지 않는다. live data가 만든 맵/노드/보상/전투 상태가 `GameState`와 active node content에 반영된 뒤, 이 컴포넌트가 command gate와 payload validation을 수행한다.

## Current Structure

- `src/game/behavior.rs:214`의 `ActionKind`는 상태 게이트와 snapshot `allowed_actions`에서 쓰는 payload-less capability다.
- `src/game/behavior.rs:261`의 `PlayerBehavior`는 GameServer/Unity에서 core로 들어오는 command request 계약이다.
- `src/game/behavior.rs:425`의 `PlayerBehavior::kind()`는 payload가 있는 `PlayerBehavior`를 payload-less `ActionKind`로 매핑한다.
- `src/game/managers/action_scheduler.rs:21`의 `ActionScheduler::get_allowed_actions_for_context`가 `GameState`와 `AllowedActionContext`를 읽어 허용 action 목록을 만든다.
- `src/game/world.rs:130`의 `execute_with_source_command_id`는 `behavior.kind()`로 state gate를 먼저 통과시킨 뒤 `validate_behavior_payload`와 handler dispatch를 실행한다.
- `src/game/world/helpers.rs:138`의 `validate_behavior_payload`는 일부 command의 payload identity/existence를 pre-dispatch에서 검사하고, 나머지는 handler validation에 맡긴다.
- `src/game/resources/action.rs:5`의 `ActionValidator`는 현재 허용된 `ActionKind` set을 캐시하며, `src/game/world.rs:401`과 `src/game/world/snapshot.rs:120`을 통해 snapshot/export 계약에 노출된다.
- `src/game/resources/state.rs:15`는 기본 allowed actions가 `GameState`로 결정되고, GameCore가 현재 노드 세션 내용을 더해 최종 보정한다고 설명한다.

Unity 계약도 이 구조를 source of truth로 본다. `/mnt/f/unity projects/ark/docs/unity_core_contract.md:273`은 Unity UI가 `game_state_context.type`과 `allowed_actions`를 기준으로 화면/버튼을 열어야 한다고 명시하고, `:275`는 `allowed_actions` 값이 `ActionKind` variant 이름 그대로 PascalCase라고 고정한다.

## Source Of Truth Judgment

- Command request shape의 source of truth는 `PlayerBehavior`다.
- Payload-less availability contract의 source of truth는 `ActionKind`다.
- `PlayerBehavior::kind()`는 두 계약 사이의 derived bridge다. 별도 데이터 source는 아니지만, 새 command를 추가할 때 반드시 함께 갱신되어야 하는 compile-time 매핑이다.
- State별 allowed action 정책의 source of truth는 `ActionScheduler::get_allowed_actions_for_context(GameState, AllowedActionContext)`다.
- Runtime snapshot의 `allowed_actions`는 `ActionValidator`에 캐시된 derived projection이다. 이 값은 Unity-facing 계약 표면이므로 실제 노출 source지만, 정책 source는 아니다.
- `AllowedActionContext`의 현재 source는 `GameCore` helper다. `src/game/world/helpers.rs:104`가 `current_reward_can_skip()`과 `is_in_maintenance_node()`를 읽어 scheduler context를 만든다.
- `BehaviorResult`는 command 결과 DTO 계약이다. `src/game/behavior.rs:817` 주석은 client가 core result 값을 반영만 해야 한다고 못박는다.

## Refactor Candidates

### 1. `ActionValidator.allowed_actions` 캐시 drift guard 또는 on-demand derivation 검토

근거:

- `ActionScheduler::get_allowed_actions_for_context`가 `GameState`와 context에서 목록을 만들지만, 실제 실행 gate와 snapshot은 `ActionValidator` 캐시를 읽는다.
- `src/game/world/helpers.rs:98`의 `refresh_allowed_actions`와 `src/game/world/helpers.rs:600`의 `transition_to`가 캐시를 갱신한다.
- `src/game/world/support.rs` 등 일부 active node content 변경 path는 `refresh_allowed_actions()`를 직접 호출해야 한다.
- `src/game/world/snapshot.rs:120`은 캐시된 값을 `allowed_actions`로 Unity에 노출한다.

판단:

- 현재 구조는 동작하지만, `GameState`, `active_node_content`, `ActionValidator`가 같은 availability 정책을 나눠 가진 derived state가 된다.
- 큰 리팩토링 없이 우선 가능한 개선은 `allowed_actions_for_state_context`를 public/internal read helper로 승격하고, snapshot/export와 execution gate가 같은 helper 결과와 캐시가 어긋나지 않는지 debug/test invariant를 두는 것이다.
- 더 강한 개선은 `ActionValidator`를 저장소가 아니라 `ActionScheduler` 호출 façade로 바꿔 allowed actions를 on-demand 파생하는 것이다. 다만 snapshot, admin dump, tests의 호출면이 넓으므로 별도 구현 goal에서 blast radius를 확인해야 한다.

정책 영향:

- `allowed_actions` snapshot shape 자체를 바꾸거나 battle runtime에서 snapshot 외 source로 availability를 바꾸는 것은 Unity-facing 계약 변경이다. `사용자와 정책 논의 필요`.

### 2. Pre-dispatch payload validation과 handler validation의 책임 경계 명문화

근거:

- `src/game/world/helpers.rs:138`의 `validate_behavior_payload`는 starter selection, reward id, map node id, equipment, consumable, skill fragment, maintenance payload를 pre-dispatch에서 검사한다.
- 같은 함수의 `:143-168`은 battle command, support command, shop payload-less command 등을 `Ok(())`으로 넘긴다.
- `PurchaseItem`/`SellItem`은 payload가 있지만 `:214`에서 pre-dispatch 검증 없이 shop handler에 맡긴다.
- `src/game/world.rs:148-152`는 state gate, payload validation, handler dispatch를 별도 단계처럼 표현한다.

판단:

- 이것은 반드시 나쁜 중복은 아니다. 일부 payload는 handler context 없이는 검증할 수 있고, 일부는 handler가 세션/전투 runtime을 잡은 뒤 검사해야 한다.
- 문제는 어떤 검증이 pre-dispatch에 있어야 하는지 기준이 코드에 드러나지 않는다는 점이다. 새 command 추가 시 임의로 `Ok(())`에 넣거나 handler와 중복 검증할 위험이 있다.
- 권장 방향은 `validate_behavior_payload`의 책임을 "cheap, side-effect-free identity/existence validation only"로 제한한다고 문서화하고, battle/shop/support semantics는 handler가 최종 검증한다고 명시하는 것이다.
- 반대로 모든 payload validation을 command handler로 이동하는 것도 단순하지만, state gate 다음 즉시 명확한 error를 주는 현재 테스트/UX 기대가 바뀔 수 있다.

정책 영향:

- error timing, error code, Unity-visible rejection behavior가 달라질 수 있는 이동은 `사용자와 정책 논의 필요`.

### 3. Command surface coverage invariant 추가

근거:

- `ActionKind`, `PlayerBehavior`, `PlayerBehavior::kind`, `ActionScheduler`, `validate_behavior_payload`, `GameCore::execute` dispatch가 모두 같은 command surface를 다른 모양으로 다룬다.
- Rust match exhaustiveness가 `PlayerBehavior::kind`와 `GameCore::execute` 누락을 어느 정도 막지만, `ActionKind`가 scheduler 어디에도 노출되지 않거나 test가 없는 상태는 compile-time으로 잡히지 않는다.
- `src/game/managers/action_scheduler.rs:227`의 `test_all_game_states_coverage`는 모든 `GameState`가 panic 없이 allowed actions를 계산하는지만 확인한다.

판단:

- trait/registry를 추가해 command를 한 곳에 몰아넣는 것은 아직 과한 추상화일 수 있다. 현재 command별 handler가 world, shop, reward, combat로 자연스럽게 나뉘어 있고, Rust exhaustiveness가 중요한 보호막을 이미 제공한다.
- 대신 focused invariant test가 더 적합하다. 예를 들어 `ActionKind`의 live command가 최소 하나의 scheduler state/context에서 노출되는지, 또는 intentionally hidden/debug-only action이면 명시 목록에 들어가는지 검증한다.
- `PlayerBehavior` variant는 payload가 필요해 전수 instance 생성이 어렵다. 따라서 `ActionKind` coverage와 scheduler coverage를 우선 고정하고, `execute` dispatch는 compile-time match에 맡긴다.

### 4. `AllowedActionContext`를 flow context helper와 통합 검토

근거:

- `AllowedActionContext`는 현재 `reward_can_skip`, `in_maintenance_node` 두 필드뿐이다.
- 두 필드는 `GameState` 자체가 아니라 `active_node_content`와 현재 node session에서 유도된다.
- 1번 `world_run_progression_refactor.md`에서도 `GameState`, `ActiveNodeContent`, allowed action context가 parallel state로 커질 위험을 기록했다.

판단:

- 즉시 새 abstraction을 만들기보다는 `GameCore`의 read-only flow context helper 후보로 묶어보는 것이 낫다.
- 같은 helper가 snapshot context, selected_event, allowed_actions, validation에서 반복되는 active node 판단을 제공하면 source-of-truth가 선명해진다.
- 이 항목은 월드/런 진행 시스템 문서의 parallel state 후보와 cross-reference한다.

### 5. `BehaviorResult` DTO 비대화는 component 15에서 재검토

근거:

- `BehaviorResult`는 command result DTO, node preview, shop/reward/support/maintenance/battle update 결과를 모두 담는다.
- `src/game/behavior.rs:817`의 주석상 client 반영 계약의 source다.
- `/mnt/f/unity projects/ark/docs/unity_core_contract.md:125`도 `result_type`이 Rust `BehaviorResult` variant 이름이라고 고정한다.

판단:

- 이 컴포넌트에서 `BehaviorResult`를 쪼개면 Unity/server contract 변경과 충돌한다.
- DTO shape refactor는 15번 Unity/server-facing DTO/계약 표면 감사에서 다룬다.

정책 영향:

- `ActionKind`, `PlayerBehavior`, `BehaviorResult`, `allowed_actions` 이름/shape 변경은 모두 `사용자와 정책 논의 필요`.

## Existing Composition Simplification Candidates

- Maintenance action availability는 별도 node-specific gate를 늘리기보다 기존 `AllowedActionContext.in_maintenance_node`와 active node content 판정으로 계속 조합하는 편이 맞다.
- Reward skip availability도 별도 `GameState` variant를 추가하기보다 기존 `RewardMode`/`RewardSessionState.can_skip`에서 `AllowedActionContext.reward_can_skip`을 유도하는 현재 방향이 더 단순하다.
- `RequestDeploymentRangePreview`는 현재 core action gate에 들어 있다. Unity 쪽 과거 goal 문서 일부에는 preview query를 gameplay action gate와 분리하려는 논의가 보이지만, 현재 core runtime과 `unity_core_contract.md`가 더 강한 source다. 이 정책을 바꾸려면 `사용자와 정책 논의 필요`.

## Legacy, Fallback, Compatibility Removal Candidates

- `RecoverBattleSetupLoss`는 이름상 recovery/fallback처럼 보이지만 현재 Unity scene/setup loss recovery 계약의 live command다. 이 컴포넌트에서는 legacy 제거 후보가 아니다.
- `selected_event`라는 snapshot key는 `/mnt/f/unity projects/ark/docs/unity_core_contract.md:279`에서 과거 명칭이지만 client-facing 계약으로 유지된다고 명시되어 있다. 이름 정리는 15번 계약 표면 감사에서만 다룬다.
- command/result 이름의 compatibility rename은 이 컴포넌트에서 하지 않는다. Unity-facing DTO shape 변경이므로 `사용자와 정책 논의 필요`.

## Deferred Or Not Doing

- Command registry/trait 도입은 보류한다. 현재는 Rust exhaustive match와 모듈별 handler 분리가 이미 안전장치를 제공하며, registry는 command 추가 경로를 더 간단하게 만든다는 근거가 아직 부족하다.
- `ActionValidator` 즉시 삭제는 보류한다. `allowed_actions`는 snapshot/admin/test에 넓게 노출되어 있어, 먼저 drift invariant와 호출면 확인이 필요하다.
- `BehaviorResult` 분해는 보류한다. 이 문서의 범위보다 Unity/server-facing 계약 변경 범위가 크다.

## Tests And Verification To Add When Refactoring

문서 감사 단계에서는 코드를 바꾸지 않았으므로 테스트를 실행하지 않았다. 실제 리팩토링 시 우선순위 검증은 아래와 같다.

- `cargo test -p game_core managers::action_scheduler`
- `cargo test -p game_core resources::action`
- `cargo test -p game_core world::tests::snapshots_and_start`
- `cargo test -p game_core world::tests::map_flow`
- `cargo test -p game_core world::tests::support`
- `cargo test -p game_core world::tests::equipment`
- `cargo test -p game_core world::tests::combat`

추가하면 좋은 focused test:

- `ActionScheduler`가 모든 live `ActionKind`를 적어도 하나의 state/context에서 노출하거나 명시적으로 non-user-facing 예외로 기록하는 coverage test.
- `GameCore` state/content mutation 뒤 `get_allowed_actions()`가 `ActionScheduler::get_allowed_actions_for_context`의 현재 context 결과와 일치하는 invariant test.
- Maintenance node 진입, reward can-skip 변경, battle setup loss recovery 이후 snapshot `allowed_actions`가 기대 state/context와 일치하는 flow test.

## Policy Discussion Required

- `ActionKind`, `PlayerBehavior`, `BehaviorResult`, `allowed_actions` 이름이나 JSON shape 변경: `사용자와 정책 논의 필요`.
- `allowed_actions`를 snapshot source에서 제거하거나 battle runtime side-channel availability로 대체하는 정책: `사용자와 정책 논의 필요`.
- pre-dispatch validation과 handler validation 이동으로 Unity-visible error timing/error code가 달라지는 변경: `사용자와 정책 논의 필요`.
- `RequestDeploymentRangePreview`를 gameplay `allowed_actions` gate에서 제외하는 변경: `사용자와 정책 논의 필요`.
