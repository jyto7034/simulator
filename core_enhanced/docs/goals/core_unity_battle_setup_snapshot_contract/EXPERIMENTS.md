# Core Unity Battle Setup Snapshot Contract Experiments

This file records implementation attempts, failures, fixes, and verification results for the `battle_setup_snapshot` goal.

## 2026-06-16 Goal Document Setup

Goal:

- Create the working goal documents for implementing `/mnt/f/unity projects/ark/docs/core_unity_battle_setup_snapshot_contract.md`.
- Keep the implementation scope explicit before code changes begin.

Files read:

- `/mnt/f/unity projects/ark/docs/core_unity_battle_setup_snapshot_contract.md`
- `docs/goals/core_unity_battle_update_contract/PLAN.md`
- `docs/code_documentation_sync_guidelines.md`

Files changed:

- `docs/goals/core_unity_battle_setup_snapshot_contract/PLAN.md`
- `docs/goals/core_unity_battle_setup_snapshot_contract/EXPERIMENTS.md`
- `docs/goals/core_unity_battle_setup_snapshot_contract/EXPERIMENT_NOTES.md`

Result:

- Created the goal working directory and three required tracking documents.
- Defined implementation scope around explicit battle scene setup, not hard resync as the primary driver.
- Listed policy stop conditions before code work starts.
- Set expected tests around Unity-facing DTO shape, message order, live RON setup data, and `battle_update` boundary preservation.

Failure cause, if any:

- None. This was a documentation-only setup step.

Fix or next action:

- Next implementation slice should inventory runtime sources before adding DTOs.
- Do not start by copying the contract DTO blindly; first confirm which setup data already exists in runtime battle construction.

Verification:

- Not run. Documentation-only change.

## 2026-06-16 Runtime DTO And Transport Slice

Goal:

- Implement the first long-term `battle_setup_snapshot` slice for live DefenseRoute battle start.
- Keep setup separate from command_result payloads and from `battle_update.checkpoint`.

Files read:

- `src/game/world/combat.rs`
- `src/game/world/state.rs`
- `src/game/behavior.rs`
- `src/game/battle/core/mod.rs`
- `src/game/battle/scenario.rs`
- `src/game/events/combat.rs`
- `src/game/combat_setup/mission_policy.rs`
- `src/game/combat_setup/defense_object.rs`
- `src/game/combat_preview/types.rs`
- `../game_server/src/game/player_game_actor/messages.rs`
- `../game_server/src/game/player_game_actor/state.rs`
- `../game_server/src/game/player_game_actor/handlers.rs`
- `../game_resources/data/pve/encounters.ron`
- `../game_resources/data/map/battlefield_templates.ron`

Files changed:

- `src/game/behavior.rs`
- `src/game/world/state.rs`
- `src/game/world/combat.rs`
- `src/game/battle/core/mod.rs`
- `src/game/world/tests/combat.rs`
- `../game_server/src/game/player_game_actor/messages.rs`
- `../game_server/src/game/player_game_actor/state.rs`
- `../game_server/src/game/player_game_actor/handlers.rs`
- `/mnt/f/unity projects/ark/docs/core_unity_battle_setup_snapshot_contract.md`
- `docs/goals/core_unity_battle_setup_snapshot_contract/EXPERIMENTS.md`
- `docs/goals/core_unity_battle_setup_snapshot_contract/EXPERIMENT_NOTES.md`

Result:

- Added typed `LiveBattleSetupSnapshotDto` and nested setup DTOs.
- Added read-only `BattleCore::scenario()` accessor so setup can use the same runtime scenario source as battle execution.
- Added `ActiveBattleSession::battle_setup_snapshot_dto()`.
- `ConfirmEnterNode` live battle start now produces a setup snapshot on the initial `BattleAdvanced` result.
- Server now emits top-level `battle_setup_snapshot` side message before the first `battle_update`.
- Battle update continues to carry `BattleStart` and checkpoint current state.
- Battle commands after entry do not resend setup snapshot.
- External canonical setup snapshot contract now reflects the implemented `battlefield.tiles` and ids-only `catalog_refs` details.

Failure cause, if any:

- None. One test expectation initially guessed the wrong live RON abnormality/template ids; fixed after checking `encounters.ron` and battlefield templates.

Fix or next action:

- Run broader package tests after updating PLAN.
- Update external canonical Unity docs only if the implemented shape is considered materially different from the target contract.

Verification:

- `cargo check -p game_core`
- `cargo check -p game_server`
- `cargo check -p game_core --tests`
- `cargo check -p game_server --tests`
- `cargo test -p game_core battle_setup_snapshot -- --nocapture`
- `cargo test -p game_server battle_setup_snapshot -- --nocapture`
- `cargo test -p game_server live_battle_tick_pushes_delta_and_snapshot_after_confirm_enter -- --nocapture`
- `cargo test -p game_server paused_live_battle_tick_does_not_push_delta_until_resumed -- --nocapture`
- `cargo test -p game_core`
- `cargo test -p game_server`

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

## 2026-06-16 Review Follow-Up Fixes

Goal:

- Address review findings around duplicate defense object creation risk, runtime battlefield source drift, and WebSocket message ordering.

Files read:

- `src/game/world/state.rs`
- `src/game/combat_setup/defense_object.rs`
- `src/game/battle/core/sim.rs`
- `../game_server/src/game/player_game_actor/handlers.rs`
- `../game_server/src/game/player_game_actor/session.rs`
- `/mnt/f/unity projects/ark/docs/core_unity_battle_setup_snapshot_contract.md`

Files changed:

- `src/game/world/state.rs`
- `src/game/world/tests/combat.rs`
- `../game_server/src/game/player_game_actor/messages.rs`
- `../game_server/src/game/player_game_actor/handlers.rs`
- `../game_server/src/game/player_game_actor/session.rs`
- `/mnt/f/unity projects/ark/docs/core_unity_battle_setup_snapshot_contract.md`
- `docs/goals/core_unity_battle_setup_snapshot_contract/PLAN.md`
- `docs/goals/core_unity_battle_setup_snapshot_contract/EXPERIMENTS.md`
- `docs/goals/core_unity_battle_setup_snapshot_contract/EXPERIMENT_NOTES.md`

Result:

- `static_objects` no longer exports defense-object battle units.
- Setup battlefield dimensions/valid tiles/obstacles now come from runtime `BattleScenario`.
- Added internal `EnsureLiveBattleTick` actor message so live tick scheduling starts after WebSocket command response, side messages, and state snapshot are sent.
- Direct actor tests now verify no pushed `battle_update` occurs before `EnsureLiveBattleTick`.

Failure cause, if any:

- Initial compile caught a moved `actor` address in session auth and unused `ctx` arguments after moving tick scheduling out of command handlers.

Fix or next action:

- Complete. Remaining hard resync behavior stays out of this goal and should be handled by a separate transport recovery goal.

Verification:

- `cargo fmt`
- `cargo check -p game_core --tests`
- `cargo check -p game_server --tests`
- `cargo test -p game_core battle_setup_snapshot -- --nocapture`
- `cargo test -p game_server battle_setup_snapshot -- --nocapture`
- `cargo test -p game_server live_battle_tick_pushes_delta_and_snapshot_after_confirm_enter -- --nocapture`
- `cargo test -p game_server paused_live_battle_tick_does_not_push_delta_until_resumed -- --nocapture`
- `cargo test -p game_core`
- `cargo test -p game_server`
