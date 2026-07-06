# Combat Result Atomicity Regression Repair Experiments

| Date | Attempt | Result | Notes |
| --- | --- | --- | --- |
| 2026-07-04 | Created goal after refactor audit found possible mismatch with completed atomicity goal. | Planned. | No code changes yet. |
| 2026-07-04 | Re-read `combat.rs`, `node_flow.rs`, `reward.rs`, current atomicity tests, and the older completed atomicity goal. | Success. | Current code still has split-looking commit order, but node-completion invariants are preflighted before reward/post-battle commit and combat completion uses `StagedSupportEffect::None`. |
| 2026-07-04 | Ran focused regression `combat_result_completion_failure_does_not_partially_apply_rewards_on_retry`. | Success. | The original retry/duplicate reward failure shape remains covered. |
| 2026-07-04 | Ran surrounding combat and map flow tests plus `cargo check`. | Success. | No runtime repair was required during this re-audit. |

## Required Experiment Log

Record both audit and implementation attempts here.

| Date | Attempt | Failure | Fix | Revalidation |
| --- | --- | --- | --- | --- |
| YYYY-MM-DD | command/test | what failed | what changed | command/result |
| 2026-07-04 | `cargo test -p game_core combat_result_completion_failure_does_not_partially_apply_rewards_on_retry --lib -- --test-threads=1` | None. | None. | Passed: 1 test. |
| 2026-07-04 | `cargo test -p game_core game::world::tests::combat --lib -- --test-threads=1` | None. | None. | Passed: 39 tests. |
| 2026-07-04 | `cargo test -p game_core game::world::tests::map_flow --lib -- --test-threads=1` | None. | None. | Passed: 24 tests. |
| 2026-07-04 | `cargo check -p game_core` | None. | None. | Passed. |
