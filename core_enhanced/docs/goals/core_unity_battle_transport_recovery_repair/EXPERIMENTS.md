# Core Unity Battle Transport Recovery Repair Experiments

This file records implementation attempts, failed approaches, fixes, and verification results for the battle transport recovery repair goal.

## 2026-06-16 Initial Setup

Goal:

- Create the goal working documents for repairing the remaining core/server -> Unity battle transport issues.

Files created:

- `docs/goals/core_unity_battle_transport_recovery_repair/PLAN.md`
- `docs/goals/core_unity_battle_transport_recovery_repair/EXPERIMENTS.md`
- `docs/goals/core_unity_battle_transport_recovery_repair/EXPERIMENT_NOTES.md`

Code changes:

- None.

Result:

- Goal scope, completion conditions, stop conditions, source-of-truth order, and verification requirements are documented.
- The goal is scoped as a follow-up repair to the implemented `battle_update` and `battle_setup_snapshot` contracts.

Failure cause, if any:

- None. Document creation only.

Verification:

- Not run. No runtime code changed.

## 2026-06-16 Transport Recovery Implementation

Goal:

- Remove full `state_snapshot` after battle side-message commands.
- Implement setup-loss recovery for `request_battle_resync { need_setup: true }` and active-battle attach.
- Validate future `known_seq` / `since_seq`.

Files read:

- `../game_server/src/game/player_game_actor/session.rs`
- `../game_server/src/game/player_game_actor/handlers.rs`
- `../game_server/src/game/player_game_actor/messages.rs`
- `../game_server/src/game/player_game_actor/state.rs`
- `src/game/behavior.rs`
- `src/game/world.rs`
- `src/game/world/combat.rs`
- `src/game/world/state.rs`
- `src/game/world/snapshot.rs`
- `src/game/world/node_flow.rs`
- `src/game/map/progression.rs`
- `/mnt/f/unity projects/ark/docs/core_unity_battle_update_contract.md`
- `/mnt/f/unity projects/ark/docs/unity_core_contract.md`
- `/mnt/f/unity projects/ark/docs/unity_client_implementation_goal.md`

Files changed:

- `../game_server/src/game/player_game_actor/session.rs`
- `../game_server/src/game/player_game_actor/handlers.rs`
- `../game_server/src/game/player_game_actor/messages.rs`
- `../game_server/src/game/player_game_actor/state.rs`
- `src/game/behavior.rs`
- `src/game/world.rs`
- `src/game/world/combat.rs`
- `src/game/world/state.rs`
- `src/game/world/snapshot.rs`
- `src/game/world/helpers.rs`
- `src/game/managers/action_scheduler.rs`
- `src/game/world/tests/combat.rs`
- `/mnt/f/unity projects/ark/docs/core_unity_battle_update_contract.md`
- `/mnt/f/unity projects/ark/docs/unity_core_contract.md`
- `/mnt/f/unity projects/ark/docs/unity_client_implementation_goal.md`

Result:

- Added `RecoverBattleSetupLoss` behavior/action.
- `request_battle_resync { need_setup: true }` maps to setup-loss recovery.
- Catch-up resync with `need_setup: false` still maps to `RequestBattleState { since_seq: Some(known_seq) }` and returns `battle_resync`.
- Future `known_seq` is rejected as `invalid_battle_resync_seq`.
- Battle command mapping now carries `send_state_snapshot`.
- Battle side-message commands do not produce a trailing full `state_snapshot`.
- Non-battle and admin commands still produce full `state_snapshot`.
- Active-battle attach now recovers to `node_confirm` instead of attaching into a battle with no setup snapshot.
- Existing live tick behavior continues to push `battle_update` without state snapshot.
- External Unity docs were adjusted to match current runtime identity: same-node re-entry starts a fresh setup/timeline but may reuse the node-scoped `battle_uuid`.

Failures and fixes:

- Initial focused command `cargo test -p game_core live_defense_battle_state_rejects_future_known_seq live_defense_setup_loss_recovery_returns_to_node_confirm_and_restarts_timeline -- --nocapture` failed because cargo accepts only one test filter. Re-ran with `live_defense_`.
- `cargo test -p game_server player_game_actor -- --nocapture` initially failed because a message test still expected `need_setup: true` to return `hard_battle_resync_not_supported`. Updated it to expect `RecoverBattleSetupLoss`.
- `cargo test -p game_core` initially failed because `test_in_battle_allows_live_battle_actions` still expected 8 actions. Updated it to include `RecoverBattleSetupLoss`.

Verification:

- `cargo check -p game_core` passed.
- `cargo check -p game_server` passed.
- `cargo test -p game_core live_defense_ -- --nocapture` passed: 8 tests.
- `cargo test -p game_server player_game_actor -- --nocapture` passed: 19 tests after focused changes, then 19 again after snapshot assertions.
- `cargo fmt -p game_core -p game_server` completed.
- `cargo test -p game_core` passed: 459 lib tests plus all integration suites.
- `cargo test -p game_server` passed: 20 tests.
