# Core Unity Battle Update Contract Experiments

This file records implementation attempts, failed approaches, fixes, and verification results for the core/server -> Unity battle update contract goal.

## 2026-06-15 Initial Setup

Result: goal working directory created.

Files created:

- `docs/goals/core_unity_battle_update_contract/PLAN.md`
- `docs/goals/core_unity_battle_update_contract/EXPERIMENTS.md`
- `docs/goals/core_unity_battle_update_contract/EXPERIMENT_NOTES.md`

No runtime code has been changed yet.

Initial target contract:

- `/mnt/f/unity projects/ark/docs/core_unity_battle_update_contract.md`

Initial verification:

- Not run. Goal document creation only.

## 2026-06-15 UnitDeployed timeline event slice

Goal:

- Make player deployment appear as an explicit presentation timeline event instead of relying on command result payload or generic `UnitSpawned` semantics.

Files read:

- `src/game/battle/timeline.rs`
- `src/game/battle/recording.rs`
- `src/game/battle/core/build.rs`
- `src/game/battle/core/mod.rs`
- `src/game/battle/validation/deaths.rs`
- `src/game/battle/validation/spawns.rs`
- `src/game/world/combat.rs`
- `src/game/world/state.rs`
- `src/game/world/tests/combat.rs`

Files changed:

- `src/game/battle/timeline.rs`
- `src/game/battle/core/build.rs`
- `src/game/battle/core/mod.rs`
- `src/game/battle/validation/deaths.rs`
- `src/game/battle/validation/spawns.rs`
- `src/game/world/tests/combat.rs`

Result:

- Added `TimelineEvent::UnitDeployed { employee_uuid, unit_instance_id, position, facing }`.
- `BattleCore::deploy_player_unit` now records `UnitDeployed` at the command battle time after the concrete battlefield position is known.
- Existing `UnitSpawned` remains in the timeline, preserving spawn semantics while adding a distinct player deployment presentation event.
- Focused core and world tests now assert the deployment event shape.

Failure cause, if any:

- First focused test run failed because battle validation modules had exhaustive matches that did not yet account for `UnitDeployed`.
- Two exploratory test filters were too narrow/wrong and executed 0 tests; they were replaced with exact test names.

Fix or next action:

- Treated `UnitDeployed` as a referenced unit event in death/reference validation.
- Re-ran focused tests with exact names.
- Next slice should add typed `events_delta` / `checkpoint` DTO shape and enforce initial `checkpoint.at_seq == events_delta.to_seq` before changing server transport.

Verification:

- `cargo test -p game_core live_deploy -- --nocapture`
  - Initial run failed on non-exhaustive `TimelineEvent::UnitDeployed` matches in validation.
  - Re-run passed, but only matched one existing deploy-cost test.
- `cargo test -p game_core apply_live_command_deploys_and_withdraws_player_unit -- --nocapture`
  - Passed, 1 test.
- `cargo test -p game_core defense_combat_node_smoke_writes_debug_event_log_export -- --nocapture`
  - Passed, 1 test.
- `cargo check -p game_core`
  - Passed.

## 2026-06-15 BattleUpdate DTO and checkpoint slice

Goal:

- Add the target `battle_update` / `events_delta` / `checkpoint` DTO shape inside `game_core`.
- Route existing live `BattleAdvanced` and requested `BattleState` payload construction through the new DTO builder so the next contract has one internal source of truth.
- Enforce the initial checkpoint policy: `checkpoint.at_seq == events_delta.to_seq`.

Files read:

- `src/game/behavior.rs`
- `src/game/world/state.rs`
- `src/game/world/combat.rs`
- `src/game/world/tests/combat.rs`
- `src/game/battle/core/mod.rs`
- `src/game/battle/core/sim.rs`

Files changed:

- `src/game/behavior.rs`
- `src/game/world/state.rs`
- `src/game/world/combat.rs`
- `src/game/world/tests/combat.rs`
- `src/game/battle/core/mod.rs`
- `src/game/battle/core/sim.rs`

Result:

- Added `LiveBattleUpdateDto` with serialized `type: "battle_update"`, `battle_uuid`, `server_battle_time_ms`, `events_delta`, and `checkpoint`.
- Added `LiveBattleEventDeltaDto { after_seq, to_seq, events }`.
- Added `LiveBattleStateCheckpointDto { at_seq, battle_time_ms, playback, units, deployment }`.
- Added `LiveBattleUnitCheckpointDto` for current unit state.
- `BattleAdvanced` and `BattleState` now build their legacy fields from the new battle update DTO path.
- Timeline seq now starts at 1 for newly created/reset battles so the first Unity-facing stream request can use `after_seq: 0` and receive first event `seq: 1`.
- Removed the old `event_log_delta_after` helper after `BattleState` moved to `battle_update_dto_after`.

Failure cause, if any:

- First test pass after adding DTOs emitted dead-code warnings because the new DTO builder was only used by tests.
- A deployment test filter was wrong and executed 0 tests.

Fix or next action:

- Routed existing production `BattleAdvanced` and `BattleState` construction through the new DTO builder instead of adding `allow(dead_code)`.
- Removed the obsolete helper.
- Re-ran focused tests with exact names.
- Next slice should expose the DTO as the actual server/WebSocket transport shape and simplify command results away from state-change payloads.

Verification:

- `cargo test -p game_core defense_live_battle_state_request_returns_timeline_delta_without_advancing_cursor -- --nocapture`
  - Passed, 1 test.
- `cargo test -p game_core live_defense_deploy_and_withdraw_manage_cost_and_redeploy_lock -- --nocapture`
  - Passed, 1 test.
- `cargo test -p game_core apply_live_command_deploys_and_withdraws_player_unit -- --nocapture`
  - Passed, 1 test.
- `cargo test -p game_core defense_combat_node_smoke_writes_debug_event_log_export -- --nocapture`
  - Passed, 1 test.
- `cargo check -p game_core`
  - Passed.
- `cargo check -p game_server`
  - Passed.

## 2026-06-15 Server battle_update transport slice

Goal:

- Stop pushing live battle ticks as legacy `notification { notification_type: "battle_delta" }`.
- Let battle-related command results acknowledge acceptance only, then send real battle changes through a separate `battle_update` server message.
- Preserve checkpoint authority by carrying the core-built `LiveBattleUpdateDto` through `BehaviorResult` instead of reconstructing a partial update in the server.

Files read:

- `../game_server/src/game/player_game_actor/messages.rs`
- `../game_server/src/game/player_game_actor/state.rs`
- `../game_server/src/game/player_game_actor/handlers.rs`
- `../game_server/src/game/player_game_actor/session.rs`
- `src/game/behavior.rs`
- `src/game/world/combat.rs`
- `src/game/world/state.rs`

Files changed:

- `../game_server/src/game/player_game_actor/messages.rs`
- `../game_server/src/game/player_game_actor/state.rs`
- `../game_server/src/game/player_game_actor/handlers.rs`
- `../game_server/src/game/player_game_actor/session.rs`
- `src/game/behavior.rs`
- `src/game/world/combat.rs`
- `src/game/world/state.rs`

Result:

- Added `PlayerGameServerMessage::BattleUpdate`, serialized as top-level `type: "battle_update"` by the existing tagged server message enum.
- Live battle tick now pushes `BattleUpdate` directly instead of the old `battle_delta` notification wrapper.
- Battle-related command results now return `CommandAccepted` with `command_id` and `accepted_at_battle_time_ms`.
- The actual battle change is delivered as a side `BattleUpdate` message after the command result.
- Core battle result variants now carry `battle_update: LiveBattleUpdateDto` so the server receives checkpoint units, deployment, playback, and event delta without reconstructing an incomplete DTO.
- Removed obsolete event-log delta helpers after all live update paths moved to the battle update builder.

Failure cause, if any:

- Initial server actor test run failed because tests still expected `BattleAdvanced` command results and `battle_delta` notifications.

Fix or next action:

- Updated handler tests to assert `CommandAccepted` and `BattleUpdate`.
- Next slice should add/confirm resync request behavior and review whether `RequestBattleState` should be renamed or supplemented with explicit `request_battle_resync`.

Verification:

- `cargo test -p game_server player_game_actor -- --nocapture`
  - Initial run failed on legacy expectations.
  - Re-run passed after updating command/tick expectations.
  - Re-run after adding top-level `battle_update` serialization coverage passed, 16 tests.
- `cargo test -p game_core defense_live_battle_state_request_returns_timeline_delta_without_advancing_cursor -- --nocapture`
  - Passed, 1 test.
- `cargo test -p game_core live_defense_deploy_and_withdraw_manage_cost_and_redeploy_lock -- --nocapture`
  - Passed, 1 test.
- `cargo check -p game_core`
  - Passed.
- `cargo check -p game_server`
  - Passed.

## 2026-06-16 Catch-up resync request slice

Goal:

- Add an explicit client-facing `request_battle_resync` request for catch-up resync.
- Preserve the contract rule that resync returns authoritative `battle_update` data instead of reconstructing state from command payloads.
- Avoid silently pretending hard resync is implemented while setup snapshot transport is still absent.

Files read:

- `/mnt/f/unity projects/ark/docs/core_unity_battle_update_contract.md`
- `../game_server/src/game/player_game_actor/messages.rs`
- `../game_server/src/game/player_game_actor/session.rs`
- `../game_server/src/game/player_game_actor/handlers.rs`
- `src/game/behavior.rs`
- `src/game/world/combat.rs`

Files changed:

- `../game_server/src/game/player_game_actor/messages.rs`
- `../game_server/src/game/player_game_actor/session.rs`
- `../game_server/src/game/player_game_actor/handlers.rs`

Result:

- Added `PlayerBehaviorRequest::RequestBattleResync { known_seq, need_setup }`.
- Catch-up resync (`need_setup: false`) maps to the existing core `RequestBattleState { since_seq: Some(known_seq) }`, which now returns `CommandAccepted` plus a side `battle_resync`.
- Added `PlayerGameServerMessage::BattleResync { battle_uuid, setup, update }`, serialized as top-level `type: "battle_resync"` with nested `update.type: "battle_update"`.
- Hard resync (`need_setup: true`) returns `hard_battle_resync_not_supported` instead of silently returning a partial response without setup.
- Added server tests for request deserialization/conversion and actor-level resync response shape.

Failure cause, if any:

- Initial mapping would have lost `need_setup` intent by converting hard resync into a full battle state fetch. This was corrected before verification.

Fix or next action:

- Made request conversion fallible with `try_into_player_behavior`.
- Session command handling now reports conversion errors with the original request id.
- Hard resync remains a documented follow-up because the project does not yet expose `battle_setup_snapshot`.

Verification:

- `cargo test -p game_server player_game_actor -- --nocapture`
  - Passed, 17 tests.
- `cargo check -p game_server`
  - Passed.
- `cargo check -p game_core`
  - Passed.
- `cargo test -p game_server`
  - Passed, 18 tests before removing the legacy serializer.
- `cargo test -p game_core`
  - Passed:
    - lib tests: 444 passed.
    - `live_item_skill_activation`: 3 passed.
    - `live_skill_catalog_audit`: 3 passed.
    - `ron_loading`: 16 passed.
    - `skill_refactor_validation`: 10 passed.
    - `skill_test_suite`: 14 passed.

## 2026-06-16 Remove server legacy battle payload serializer

Goal:

- Remove the remaining server-side path that could serialize battle `BehaviorResult` variants as legacy command payloads such as `BattleAdvanced`, `timeline_delta`, and `last_timeline_seq`.

Files read:

- `../game_server/src/game/player_game_actor/state.rs`
- `../game_server/src/game/player_game_actor/handlers.rs`

Files changed:

- `../game_server/src/game/player_game_actor/state.rs`

Result:

- Removed unused legacy battle payload structs from `player_game_actor/state.rs`.
- `behavior_result_payload` now rejects battle results with `battle_update_transport_required` if a caller bypasses the battle update mapping.
- Replaced legacy serialization tests with tests that prove battle results route to `CommandAccepted` plus a side `battle_update`.

Failure cause, if any:

- First compile failed because the rewritten test helper still needed `LiveBattleDeploymentDto` in scope.

Fix or next action:

- Added the missing test import and reran verification.

Verification:

- `cargo test -p game_server player_game_actor -- --nocapture`
  - Passed, 16 tests.
- `cargo check -p game_server`
  - Passed.
- `cargo test -p game_server`
  - Passed, 17 tests.

## 2026-06-16 Source command correlation slice

Goal:

- Add the contract-level link between `command_result` acknowledgements and the presentation events they cause.
- Keep the command id out of domain `PlayerBehavior` payloads while still allowing server request ids to appear on Unity-facing timeline entries.

Files read:

- `src/game/world.rs`
- `src/game/world/combat.rs`
- `src/game/battle/timeline.rs`
- `src/game/battle/recording.rs`
- `src/game/battle/core/mod.rs`
- `src/game/battle/core/sim.rs`
- `../game_server/src/game/player_game_actor/handlers.rs`

Files changed:

- `src/game/battle/timeline.rs`
- `src/game/battle/recording.rs`
- `src/game/battle/core/mod.rs`
- `src/game/battle/core/sim.rs`
- `src/game/battle/validation/validator.rs`
- `src/game/world.rs`
- `src/game/world/combat.rs`
- `src/game/world/tests/combat.rs`
- `../game_server/src/game/player_game_actor/handlers.rs`

Result:

- Added optional `TimelineEntry.source_command_id`, skipped during serialization when absent.
- Added battle recording source-command context so entries recorded during a command can inherit the current source id.
- Added `BattleCore::apply_live_command_with_source_command_id`.
- Added `GameCore::execute_with_source_command_id`; existing `execute` delegates to it with no source id.
- Server actor command execution now passes the WebSocket `request_id` to core as the source command id.
- Live deploy/withdraw/manual skill activation calls now run through the source-command live command API.
- Focused live deployment test now proves `UnitDeployed` carries the accepted command id.

Failure cause, if any:

- No compile failure in this slice. Existing validation test fixtures needed explicit `source_command_id: None` because `TimelineEntry` is constructed directly there.

Fix or next action:

- Leave hard resync/setup snapshot for a separate setup snapshot goal unless the user expands this goal; the source-command correlation required by the current battle update contract is now implemented.

Verification:

- `cargo check -p game_core`
  - Passed.
- `cargo check -p game_server`
  - Passed.
- `cargo test -p game_core defense_combat_node_smoke_writes_debug_event_log_export -- --nocapture`
  - Passed, 1 focused test.
- `cargo test -p game_server player_game_actor -- --nocapture`
  - Passed, 16 tests.

## 2026-06-16 Remove core legacy battle result fields

Goal:

- Remove the remaining internal compatibility surface where battle `BehaviorResult` variants still carried duplicate `timeline_delta`, `last_timeline_seq`, `playback`, `battle_time_ms`, and `deployment` fields outside `battle_update`.
- Make core tests read presentation events from `battle_update.events_delta` and current/recovery state from `battle_update.checkpoint`.

Files read:

- `src/game/behavior.rs`
- `src/game/world/combat.rs`
- `src/game/world/tests/combat.rs`
- `../game_server/src/game/player_game_actor/state.rs`

Files changed:

- `src/game/behavior.rs`
- `src/game/world/combat.rs`
- `src/game/world/tests/combat.rs`
- `src/game/battle/timeline.rs`
- `src/game/battle/recording.rs`
- `../game_server/src/game/player_game_actor/state.rs`

Result:

- Battle `BehaviorResult` variants now carry `battle_update` as the single battle update payload source.
- Core live battle tests now assert event timing through `events_delta` and current state through `checkpoint`.
- Server legacy-transport rejection test fixture was updated to the new core result shape.
- Old `timeline_delta` wording was removed from runtime comments and test names.

Failure cause, if any:

- First focused compile failed because tests still destructured removed fields from `BattleUnitDeployed`, `BattleState`, `BattlePlaybackChanged`, `BattleAdvanced`, `BattleUnitWithdrawn`, and `BattleSkillActivated`.
- Server actor test compile then failed because a test fixture still populated removed `BattleAdvanced` fields.

Fix or next action:

- Rewrote the affected tests to use `battle_update.events_delta` and `battle_update.checkpoint`.
- Updated the server fixture to only populate `battle_update` plus non-legacy metadata.

Verification:

- `cargo test -p game_core defense_combat_node_smoke_writes_debug_event_log_export -- --nocapture`
  - Passed, 1 focused test.
- `cargo test -p game_server player_game_actor -- --nocapture`
  - Passed, 16 tests.
- `cargo test -p game_core`
  - Passed: 444 lib tests, 3 live item tests, 3 live skill catalog audit tests, 16 RON loading tests, 10 skill refactor validation tests, 14 skill suite tests.
- `cargo test -p game_server`
  - Passed, 17 tests.

## Experiment Log Template

Use this format for each implementation attempt:

```text
## YYYY-MM-DD <short attempt name>

Goal:

Files read:

Files changed:

Result:

Failure cause, if any:

Fix or next action:

Verification:
```
