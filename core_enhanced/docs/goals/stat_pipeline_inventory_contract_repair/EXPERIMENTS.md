# Stat Pipeline And Inventory Contract Repair Experiments

## Log

- Created goal working docs after user corrected the process.
- Process correction: before this goal directory was created, `src/game/battle/stat_pipeline.rs` had already been edited to add a module-level battle-entry pipeline comment and replace the weak consumable/equipment test. Do not revert with git. Continue from the current worktree and record validation here.
- Added explicit `GameError::InventoryItemNotRemovable` and `InventoryRemoveError` so artifact UUID removal is no longer represented as silent `None`.
- Updated shop sell path to return the explicit not-removable error for owned artifact UUIDs.
- Ran `cargo fmt`: passed.
- Attempted `cargo test -p game_core battle::stat_pipeline::tests resources::inventory::tests -- --nocapture`: failed because Cargo accepts only one test name filter. Re-run filters separately.
- Ran stat pipeline and inventory focused tests in parallel: both failed to compile because `assert_eq!(Result<Item, InventoryRemoveError>, Err(...))` required `Item: PartialEq`. Do not add `PartialEq` to the broad `Item` enum for a test convenience; rewrite assertions with `matches!`.
- Ran `cargo test -p game_core battle::stat_pipeline::tests -- --nocapture`: passed, 3 tests.
- Ran `cargo test -p game_core resources::inventory::tests -- --nocapture`: passed, 6 tests.
- Ran `cargo check -p game_core`: passed.
- Ran `cargo test -p game_core game::world::tests::equipment -- --nocapture`: passed, 20 tests.
- Ran `cargo test -p game_core game::world::tests::snapshots_and_start -- --nocapture`: initially passed 7 tests before adding the artifact sell regression test.
- Added `selling_owned_artifact_reports_not_removable_instead_of_missing_item` to lock the Unity-facing command behavior.
- Re-ran `cargo test -p game_core game::world::tests::snapshots_and_start -- --nocapture`: passed, 8 tests.
- Re-ran `cargo check -p game_core`: passed.
- Ran `cargo test -p game_core --test ron_loading`: passed, 16 tests.
- Ran `cargo test -p game_core`: passed, 455 unit tests plus integration suites (`live_item_skill_activation`, `live_skill_catalog_audit`, `ron_loading`, `skill_refactor_validation`, `skill_test_suite`).
- User reported `cargo run`/server compile failure: `game_server/src/game/player_game_actor/state.rs` did not cover `GameError::InventoryItemNotRemovable`.
- Added `inventory_item_not_removable` mapping to `PlayerGameActorError`.
- Ran `cargo check -p game_server`: passed.
