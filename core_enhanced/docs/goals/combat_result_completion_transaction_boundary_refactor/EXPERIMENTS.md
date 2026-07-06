# Combat Result Completion Transaction Boundary Refactor Experiments

| Date | Attempt | Result | Notes |
| --- | --- | --- | --- |
| 2026-07-04 | Created goal from the follow-up direction in `combat_result_atomicity_regression_repair`. | Planned. | No code changes yet. |
| 2026-07-04 | Re-read `combat.rs`, `node_flow.rs`, `reward.rs`, and the focused combat result atomicity test. | Success. | Chose a player-victory planning object plus `commit_preplanned_node_completion` rename. |
| 2026-07-04 | Extracted player-victory planning/commit helpers and renamed node completion commit helper. | Success. | Behavior-preserving refactor; no DTO/reward timing change intended. |
| 2026-07-04 | Ran focused and broader validation. | Success. | Atomicity regression, combat module, map flow module, and `cargo check` passed. |

## Required Experiment Log

Record audit, implementation, and validation attempts here.

| Date | Attempt | Failure | Fix | Revalidation |
| --- | --- | --- | --- | --- |
| YYYY-MM-DD | command/test | what failed | what changed | command/result |
| 2026-07-04 | `cargo test -p game_core combat_result_completion_failure_does_not_partially_apply_rewards_on_retry --lib -- --test-threads=1` | None. | None. | Passed: 1 test. |
| 2026-07-04 | `cargo test -p game_core game::world::tests::combat --lib -- --test-threads=1` | None. | None. | Passed: 39 tests. |
| 2026-07-04 | `cargo test -p game_core game::world::tests::map_flow --lib -- --test-threads=1` | None. | None. | Passed: 24 tests. |
| 2026-07-04 | `cargo check -p game_core` | None. | None. | Passed. |
