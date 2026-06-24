# Combat Result Completion Atomicity Experiments

| Date | Attempt | Result | Notes |
| --- | --- | --- | --- |
| 2026-06-23 | Re-read server log, Unity wire log, and runtime handler order. | Success. The observed failure is core-side: rewards are logged repeatedly after `CombatResult`, while no `CombatResult -> ViewingMap` transition appears. | Evidence recorded in `PLAN.md`. |
| 2026-06-23 | Compared combat reward application with reward-session atomic grant implementation. | Success. Reward session itself uses cloned state and commits after all effects, but `handle_complete_combat_result()` commits surrounding combat-result mutations before map node completion. | Goal should raise atomicity boundary, not replace grant executor. |
| 2026-06-23 | Added `combat_result_completion_failure_does_not_partially_apply_rewards_on_retry` before the fix. | Failed as intended. | The old implementation returned `InvalidAction` after mutating XP; assertion showed `experience_after = 20`, expected `0`. |
| 2026-06-23 | Implemented staged combat result completion and dynamic `CompleteCombatResult` allowed-action filtering. | Success. | Focused regression passed and `CompleteCombatResult` is hidden when local completion invariants fail. The regression now checks XP, Enkephalin, equipment inventory, map progression, node session, and game state. |
| 2026-06-23 | Ran focused and broad validation. | Success. | Combat, snapshot, map flow, node session, and full `game_core` package tests passed. |

## Required Future Experiments

- If a server probe is used later, record the exact server command, wire command, and result here.

## Failure Log Template

When a test or probe fails, append:

| Date | Attempt | Failure | Fix | Revalidation |
| --- | --- | --- | --- | --- |
| YYYY-MM-DD | command/test | what failed | what changed | command/result |
