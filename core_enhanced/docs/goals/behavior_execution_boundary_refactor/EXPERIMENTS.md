# Experiments

| Date | Trial | Result | Notes |
| --- | --- | --- | --- |
| 2026-07-06 | Goal document creation from audit review. | Planned. | Covers P-007: `execute_with_source_command_id` is a broad execution gateway. |
| 2026-07-06 | Source re-read. | Success. | Re-read `src/game/world.rs`, `src/game/world/helpers.rs`, `src/game/behavior.rs`, and focused allowed-action/source-command/roster test locations before editing. |
| 2026-07-06 | Smallest boundary refactor. | Success. | Kept `PlayerBehavior` and DTOs unchanged. Split `execute_with_source_command_id(...)` into explicit gate, payload validation, dispatch, and postprocess stages. |

## Validation Log

- `cargo test -p game_core allowed_actions --lib -- --test-threads=1` passed: 3 tests.
- `cargo test -p game_core invalid_action --lib -- --test-threads=1` passed with 0 matching tests; retained only as a weak filter check.
- `cargo test -p game_core payload --lib -- --test-threads=1` passed: 1 test.
- `cargo test -p game_core game::world::tests::snapshots_and_start --lib -- --test-threads=1` passed: 11 tests.
- `cargo test -p game_core source_command --lib -- --test-threads=1` passed with 0 matching tests; source-command coverage is covered by the broader combat module run below.
- `cargo test -p game_core live_defense_deploy_and_withdraw_manage_cost_and_redeploy_lock --lib -- --test-threads=1` passed: 1 test. This test asserts deploy `source_command_id` is preserved on the emitted `UnitDeployed` timeline event.
- `cargo test -p game_core roster_order --lib -- --test-threads=1` passed: 2 tests.
- `cargo test -p game_core game::world::tests::combat --lib -- --test-threads=1` passed: 39 tests.
- `cargo check -p game_core` passed.
- `cargo fmt --check` passed.

Known unrelated warning:

- `src/game/world/tests/mod.rs` still has an existing unused import warning for `buffs::BuffDatabase`.
