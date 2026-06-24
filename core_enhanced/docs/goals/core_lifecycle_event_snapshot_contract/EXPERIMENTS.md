# Experiments

| Date | Attempt | Result | Follow-up |
|---|---|---|---|
| 2026-06-23 | Goal setup | Created the subgoal document for lifecycle event and snapshot contracts. | Run after lifecycle, redeploy, and inactive effect goals. |
| 2026-06-23 | Runtime/event contract inventory | Read event log enum, withdrawal/death emission, checkpoint DTO construction, gameplay `BattleState`, and existing admin/debug surfaces. Found event payload positions missing and no separate inactive unit debug query. | Add event position fields and a debug-only inactive unit query without changing gameplay checkpoint. |
| 2026-06-23 | Event/snapshot implementation | Added `world_position` and `position` to `UnitWithdrawn`/`UnitDied`, bumped event log version, sourced fields from `RuntimeUnit.body`, added `debug_inactive_units()`, and updated rulebook text. | Run broad validation and review. |
| 2026-06-23 | Focused validation | `cargo fmt`, `cargo test apply_live_command_deploys_and_withdraws_player_unit`, `cargo test apply_hp_delta_records_died_stop_with_latest_continuous_position`, `cargo test debug_inactive_units_lists_retained_non_active_units_with_lifecycle_and_position`, and `cargo check` passed. | Run full `cargo test` and completion review. |
| 2026-06-23 | Parallel broad validation | `cargo test` failed once in `retreat_from_live_defense_battle_reenters_until_attempts_are_exhausted` while parsing a battle record JSON file: EOF while parsing an object. | Re-ran the failed test alone to distinguish code regression from parallel debug artifact race. |
| 2026-06-23 | Failure isolation | `cargo test retreat_from_live_defense_battle_reenters_until_attempts_are_exhausted` passed. | Re-run broad validation serially because the failure points to parallel file artifact contention. |
| 2026-06-23 | Serial broad validation | `cargo test -- --test-threads=1` passed: 525 lib tests plus integration suites and doc tests. | Record completion review. |
