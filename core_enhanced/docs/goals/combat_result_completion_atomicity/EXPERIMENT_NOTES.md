# Combat Result Completion Atomicity Notes

## Initial Judgment

This is a transaction-boundary bug, not a Unity command mapping bug.

The long-term direction is to make combat result completion a staged all-or-nothing operation. A narrow guard before `handle_complete_node()` would reduce the current symptom, but it would not fully enforce the intended contract because earlier post-battle and reward mutations would still be coupled only by call order.

Implemented direction: staged state plus node-completion preflight. The fix does not add a retry marker or compatibility fallback.

## Important Observations

- `apply_reward_session_with_context()` already follows a good clone/preflight/commit pattern for reward internals.
- `apply_post_battle_resolution()` mutates roster state and can grant survival XP before combat node completion succeeds.
- `decrement_consumables_after_combat_node()` mutates employee consumable state before node completion succeeds.
- `handle_complete_node()` is shared by non-combat node flows and applies support-node effect logic before map progression completion. A combat-specific transaction should avoid accidentally expanding support-node semantics.
- `ActionScheduler` currently treats `CombatResult` state as sufficient to advertise `CompleteCombatResult`.
- Runtime `get_allowed_actions()` now filters `CompleteCombatResult` through `can_complete_combat_result_locally()`, so the stored high-level scheduler action is no longer the final snapshot source for this command.

## Recommended Implementation Shape

Prefer one of these long-term shapes after reading the code again:

1. Add a dedicated `complete_combat_result_transaction()` that clones or stages all affected state, applies post-battle/reward/map completion against staged values, then commits all staged values and transition state together.
2. Extract reusable map node completion planning from `handle_complete_node()` into a pure/staged helper, then let combat result completion combine that staged completion with staged post-battle and reward effects.

Avoid:

- applying rewards first and hoping node completion succeeds;
- completing the node first and then applying rewards without a staged rollback story;
- adding an idempotency flag that masks duplicate XP while leaving other partial mutations possible;
- treating `InvalidAction` as success;
- leaving `allowed_actions` and handler invariants knowingly divergent.

## Policy Questions To Raise If Encountered

Complete the goal and report these rather than deciding silently:

- Should battle record files be considered part of the transaction, or are they debug artifacts allowed before final commit?
- If a combat result cannot complete due to corrupt map progression, should the user receive a recoverable diagnostic state or a hard error?
- If post-battle survival XP and combat reward XP are both granted, should their diffs be merged into one displayed result or remain separate internal events?

Resolved during this goal:

- `CompleteCombatResult` disappears from runtime `allowed_actions` when lower-level local completion invariants fail. This follows the completion condition that snapshots must not advertise commands the handler rejects due to predictable local invariants.

## Out Of Scope

- Redesigning all node completion flows.
- Changing combat reward balance.
- Changing Unity button mapping.
- Changing live RON reward content unless validation proves current content is invalid.
- Broad server transport rewrite.

## Follow-Up Candidates

- Consider applying the same staged result-completion structure to non-player combat result branches for conceptual symmetry. This was not required to fix the observed repeated combat reward grant bug.
