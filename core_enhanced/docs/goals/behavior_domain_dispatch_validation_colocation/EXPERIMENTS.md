# Experiments

| Date | Trial | Result | Notes |
| --- | --- | --- | --- |
| 2026-07-06 | Goal document creation. | Planned. | Follow-up to `behavior_execution_boundary_refactor`; target is domain-level validation/dispatch colocation without changing behavior. |
| 2026-07-06 | Source re-read before editing. | Success. | Re-read the prior behavior execution goal, `world.rs`, `helpers.rs`, domain handlers, `behavior.rs`, and focused test locations before editing. |
| 2026-07-06 | Domain bucket mapping. | Success. | Mapped every current `PlayerBehavior` variant to map/run, support/headquarters, equipment/maintenance, shop, reward, event, or battle. No general bucket needed. |
| 2026-07-06 | Domain execution refactor. | Success. | Replaced global validate+dispatch with `execute_validated_behavior_domain(...)` and domain execution helpers. Removed the old full-enum `validate_behavior_payload(...)` wrapper. |

## Validation Log

- `cargo check -p game_core` passed after the first edit; it exposed an unused `PlayerBehavior` import in `helpers.rs`, which was removed.
- `cargo test -p game_core allowed_actions --lib -- --test-threads=1` passed: 3 tests.
- `cargo test -p game_core game::world::tests::snapshots_and_start --lib -- --test-threads=1` passed: 11 tests.
- `cargo test -p game_core game::world::tests::node_sessions --lib -- --test-threads=1` passed: 10 tests.
- `cargo test -p game_core game::world::tests::equipment --lib -- --test-threads=1` passed: 23 tests.
- `cargo test -p game_core game::world::tests::map_flow --lib -- --test-threads=1` passed: 25 tests.
- `cargo test -p game_core live_defense_deploy_and_withdraw_manage_cost_and_redeploy_lock --lib -- --test-threads=1` passed: 1 test. This covers live battle deploy `source_command_id` propagation.
- `cargo test -p game_core game::world::tests::combat --lib -- --test-threads=1` passed: 39 tests.
- Initial `cargo fmt --check` failed on a formatting-only line break in `src/game/world.rs`; ran `cargo fmt`.
- `cargo fmt --check` passed after formatting.
- `cargo check -p game_core` passed after formatting.

Known unrelated warning:

- `src/game/world/tests/mod.rs` still has an existing unused import warning for `buffs::BuffDatabase`.
