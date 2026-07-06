# Combat Result Atomicity Regression Repair Notes

## Initial Judgment

This is high risk because it touches rewards, roster state, node completion, floor advance, run completion, and result commands.

There is an existing completed goal: `docs/goals/combat_result_completion_atomicity`. This new goal should not duplicate work blindly. First verify whether the current runtime still satisfies that completed goal's contract.

## Re-Audit Judgment (2026-07-04)

The older completed goal remains true for the original player-victory duplicate reward bug. The current implementation does not need another code repair for that observed failure shape.

Important nuance:

- `handle_complete_combat_result()` still commits staged combat result state before calling `commit_staged_node_completion()`.
- This looks like a regression at a glance.
- In the current runtime, the node-completion work that can reject local invariants is already performed earlier by `can_complete_combat_result_locally()` and `plan_complete_current_node(false)`.
- The combat result path explicitly plans node completion with `apply_support_effect = false`, so support-node side effects and checkpoint writes are not part of this combat completion transaction.
- The focused regression still proves failed local completion cannot duplicate XP, Enkephalin, equipment rewards, map progression, node session, or game state.

Therefore this re-audit completes as "safe by current code/test evidence", not as a new implementation patch.

Future risk:

- If combat result completion later gains any fallible work after `commit_staged_combat_result_state()` such as support effects, checkpoint persistence, battle-record persistence treated as authoritative, or fallible transition logic, the current split-looking commit order must be revisited.
- At that point, prefer a combined combat-result/node-completion commit helper or a larger staged state object rather than an idempotency marker.

## Follow-Up Goal Direction

If this area is worked on again, the goal should not be written as "fix the atomicity bug" unless a new failing runtime path is demonstrated.

The better goal is a design clarity refactor:

- Make planning/preflight explicitly fallible.
- Make commit/application explicitly prevalidated.
- Rename or split helpers so `commit_*` does not appear to discover new local invalidity after reward state is committed.
- Keep current behavior and tests intact.

This is worth doing because the current code is correct but easy to misread. The repeated audit false positive is evidence that the design communicates its transaction boundary poorly.

## Guardrails

- Do not make a narrow patch for one test.
- Do not add an idempotency flag to hide partial mutation.
- Do not move mutations earlier in the flow unless they are staged.
- Do not alter reward balance, XP values, or node policy.

## Follow-Up Candidates

- If this goal confirms only a documentation mismatch, clean up the stale refactor audit note instead of changing code. Done for this re-audit by documenting the evidence in this goal.
- If non-player combat result branches have similar risks, record them separately unless they are required for the current repair.
