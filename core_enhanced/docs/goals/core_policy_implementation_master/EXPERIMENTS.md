# Experiments

## Log

| Date | Scope | Attempt | Result | Follow-up |
| --- | --- | --- | --- | --- |
| 2026-06-22 | Goal setup | Created implementation master/subgoal structure from confirmed policy categories. | Pending document verification. | Keep subgoal docs synchronized during implementation. |
| 2026-06-22 | Subgoal 1: static data/map/scenario | Completed `docs/goals/core_policy_static_data_map_scenario` implementation and final validation. | Success. Static RON/run/map/scenario policies were implemented or confirmed: run policy data source, map DTO state SoT, authored Defense routes, starter loadout live data, legacy random-event data removal, battlefield archetype policy, FacilityEntity validation, server preview validation, battle record debug-artifact boundary, and map generation policy RON. | Continue with `docs/goals/core_policy_grant_economy_rewards`. |
| 2026-06-22 | Subgoal 2: grant/economy/rewards | Completed `docs/goals/core_policy_grant_economy_rewards` implementation and final validation. | Success. Canonical grant executor coverage, atomic reward claim, reward tag removal, explicit equipment pools, abnormality reward validation, Enkephalin overflow checks, skill-fragment/research/XP diffs, duplicate conversion removal, and grant migration for admin/shop/headquarters/maintenance/starter/equipment-combination flows were implemented. | Continue with `docs/goals/core_policy_employee_item_growth`. |
| 2026-06-22 | Subgoal 3: employee/item/growth | Completed `docs/goals/core_policy_employee_item_growth` implementation and final validation. | Success. Employee grade removal, data-driven growth policy, starter live loadout source, fragment equip limits, deterministic fragment dismantle repair, bound equipment locks, and trauma/final HP policy were implemented or verified. | Continue with `docs/goals/core_policy_buff_status_stats`. |
| 2026-06-22 | Subgoal 4: buff/status/stats | Completed `docs/goals/core_policy_buff_status_stats` implementation and final validation. | Success. Hard CC active-buff SoT, explicit buff expiration reasons, death cleanup, control status stack validation, and movement-speed stat source policies were implemented. | Continue with `docs/goals/core_policy_movement_battle_runtime`. |
| 2026-06-22 | Subgoal 5: movement/battle runtime | Completed `docs/goals/core_policy_movement_battle_runtime` implementation and final validation. | Success. Runtime movement/max-time policy moved to live run policy, opponent nearest-open fallback removed, authored overlap spawn fixed, `UnitWithdrawn` timeline event added, same-time attack resolve ordering pinned, and existing gates/stop reasons verified. | Continue with `docs/goals/core_policy_skill_targeting_projectile`. |
| 2026-06-22 | Subgoal 6: skill/targeting/projectile | Completed `docs/goals/core_policy_skill_targeting_projectile` implementation and final validation. | Success. Explicit skill cast targeting, live skill `range_units` removal, skill catalog range cleanup, TileArea valid-tile clipping, canonical projectile miss events, and `SplashClusterFirst` removal were implemented. | Continue with `docs/goals/core_policy_unity_server_contract`. |
| 2026-06-22 | Subgoal 7: unity/server contract | Completed `docs/goals/core_policy_unity_server_contract` implementation and final validation. | Success. Shared gameplay command payloads, typed run snapshot DTOs, combat result timeline attachment ownership, BehaviorResult domain contracts, BattleResync setup removal, server fixture repair, and battle record export verification were completed. | Continue with `docs/goals/core_policy_validation_test_harness`. |
| 2026-06-22 | Subgoal 8: validation/test harness | Completed `docs/goals/core_policy_validation_test_harness` implementation, policy coverage cross-check, and broad validation. | Success. Timeline versions 6/7 are no longer accepted by normal validation; live skill audit coverage moved to a versioned manifest; final policy coverage checklist was written. | Master goal implementation run complete. |

## Failed Approaches

See subgoal `EXPERIMENTS.md` files for scoped failed approaches. No master sequencing failure yet.

## Validation Commands

- `cargo test --lib -- --test-threads=1` - passed after static data/map/scenario subgoal, 498 tests.
- `cargo test --test ron_loading` - passed after static data/map/scenario subgoal, 17 tests.
- `cargo test --test ron_loading` - passed after grant/economy/rewards subgoal, 18 tests.
- `cargo check -p game_server` - passed after grant/economy/rewards subgoal.
- `cargo test --lib -- --test-threads=1` - passed after grant/economy/rewards subgoal, 500 tests.
- `cargo test --test ron_loading` - passed after employee/item/growth subgoal, 18 tests.
- `cargo check -p game_server` - passed after employee/item/growth subgoal.
- `cargo test --lib -- --test-threads=1` - passed after employee/item/growth subgoal, 503 tests.
- `cargo test --test ron_loading` - passed after buff/status/stats subgoal, 18 tests.
- `cargo check -p game_server` - passed after buff/status/stats subgoal.
- `cargo test --lib -- --test-threads=1` - passed after buff/status/stats subgoal, 507 tests.
- `cargo test --test ron_loading` - passed after movement/battle runtime subgoal, 18 tests.
- `cargo check -p game_server` - passed after movement/battle runtime subgoal.
- `cargo test --lib -- --test-threads=1` - passed after movement/battle runtime subgoal, 512 tests.
- `cargo test --test ron_loading` - passed after skill/targeting/projectile subgoal, 18 tests.
- `cargo check -p game_server` - passed after skill/targeting/projectile subgoal.
- `cargo test --lib -- --test-threads=1` - passed after skill/targeting/projectile subgoal, 514 tests.
- `cargo check --lib` - passed during unity/server contract subgoal command DTO migration.
- `cargo check -p game_server` - passed during unity/server contract subgoal command DTO migration.
- `cargo test -p game_server player_game_actor -- --test-threads=1` - passed during unity/server contract subgoal, 22 tests.
- `cargo test --lib snapshots_and_start -- --test-threads=1` - passed during unity/server contract subgoal, 8 tests.
- `cargo check --lib` - passed after unity/server contract subgoal final changes.
- `cargo check -p game_server` - passed after unity/server contract subgoal final changes.
- `cargo test --lib snapshots_and_start -- --test-threads=1` - passed after unity/server contract subgoal final changes, 8 tests.
- `cargo test --lib combat:: -- --test-threads=1` - passed after unity/server contract subgoal final changes, 40 tests.
- `cargo test -p game_server player_game_actor -- --test-threads=1` - passed after unity/server contract subgoal final changes, 22 tests.
- `cargo test --test live_skill_catalog_audit -- --test-threads=1` - passed after validation/test harness manifest migration, 3 tests.
- `cargo test --lib game::battle::validation::validator::tests::normal_validation_rejects_legacy_timeline_versions -- --test-threads=1` - passed after validation/test harness legacy-version removal.
- `cargo test --lib -- --test-threads=1` - passed after validation/test harness final cross-check, 515 tests.
- `cargo test --test ron_loading -- --test-threads=1` - passed after validation/test harness final cross-check, 18 tests.
- `cargo check -p game_server` - passed after validation/test harness final cross-check.
