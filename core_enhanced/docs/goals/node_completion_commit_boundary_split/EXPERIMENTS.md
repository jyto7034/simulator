# Node Completion Commit Boundary Split Experiments

Use this file to record implementation attempts, failures, test results, and final validation.

## Trial Log

- 2026-07-06: Re-read goal docs first, then inspected `combat.rs`, `node_flow.rs`, `event_node.rs`, support helpers, and existing combat/map/support tests. Chosen approach: split commit entrypoints by caller intent while keeping shared low-level variant application helpers.
- 2026-07-06: Applied the split:
  - `handle_complete_node()` and Event node end paths now call `commit_interactive_node_completion(...)`.
  - player-victory combat result now calls `commit_combat_result_node_completion(...)`.
  - the old broad `commit_preplanned_node_completion(...)` entrypoint was removed.
  - support effect application and savepoint checkpoint persistence retained the previous order: apply support effect, clear node/session state, transition, then save checkpoint if needed.
- 2026-07-06: Hardened the combat-result path after review feedback. Replaced debug-only support-effect assertions with `CombatResultNodeCompletion::try_from(StagedNodeCompletion)`, so support-bearing completions are rejected during planning before reward commit.

## Validation Log

- 2026-07-06: `cargo check -p game_core` passed.
- 2026-07-06: `cargo test -p game_core combat_result_completion_failure_does_not_partially_apply_rewards_on_retry --lib -- --test-threads=1` passed.
- 2026-07-06: `cargo test -p game_core game::world::tests::combat --lib -- --test-threads=1` passed.
- 2026-07-06: `cargo test -p game_core game::world::tests::map_flow --lib -- --test-threads=1` passed.
- 2026-07-06: `cargo test -p game_core game::world::tests::support --lib -- --test-threads=1` passed.
- 2026-07-06: `cargo check -p game_core` passed after formatting.
- 2026-07-06: After replacing debug assertions with `CombatResultNodeCompletion`, reran:
  - `cargo test -p game_core combat_result_completion_failure_does_not_partially_apply_rewards_on_retry --lib -- --test-threads=1` passed.
  - `cargo test -p game_core game::world::tests::combat --lib -- --test-threads=1` passed.
  - `cargo test -p game_core game::world::tests::map_flow --lib -- --test-threads=1` passed.
  - `cargo check -p game_core` passed.

Warnings observed were unrelated pre-existing warnings:

- `auth_server/Cargo.toml: unused manifest key: env`.
- unused imports in `src/game/world/tests/mod.rs`.
