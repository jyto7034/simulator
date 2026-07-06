# Combat Result Completion Transaction Boundary Refactor Notes

## Initial Judgment

The current runtime appears safe for the original duplicate reward bug, but the code is easy to misread:

```text
commit_staged_combat_result_state(...)
commit_staged_node_completion(...)
```

That ordering repeatedly triggered false-positive audits because it looks like node completion can still reject local invariants after reward state has been committed.

The refactor should make the existing intent obvious without changing behavior.

## Guardrails

- Preserve player-visible behavior.
- Preserve reward, XP, research, consumable, node completion, and state transition timing.
- Preserve the existing all-or-nothing test contract.
- Do not add an idempotency marker or retry flag.
- Do not hide a real bug behind naming changes. If a new failing runtime path is found, record it and treat it as a behavior repair.

## Design Candidates To Evaluate

1. Extract a dedicated `plan_combat_result_completion()` and `commit_planned_combat_result_completion()`.
   - Most explicit.
   - Likely touches more code but gives the clearest transaction boundary.

2. Rename/split node completion commit helpers.
   - Smaller.
   - Useful if the main confusion is that `commit_staged_node_completion()` still sounds failure-prone after reward commit.

3. Expand staged combat result state to include preplanned node completion and research updates.
   - May be the best middle path if current helper extraction is awkward.
   - Must avoid cloning or committing unrelated global state more broadly than necessary.

## Chosen Refactor Shape (2026-07-04)

Use a focused player-victory combat result planning object plus clearer node completion naming.

Planned shape:

```text
plan_player_victory_combat_result_completion(...)
  -> fallible
  -> calls plan_complete_current_node(false) before reward/post-battle commit
  -> builds staged roster/inventory/fragment/enkephalin state
  -> evaluates research rewards and output DTO diffs
  -> returns one planned completion object

commit_planned_player_victory_combat_result_completion(...)
  -> applies the already planned staged state
  -> applies planned research state
  -> commits the preplanned node completion
  -> applies Endless response-complete transition if planned
```

Also rename the node completion commit helper from:

```text
commit_staged_node_completion(...)
```

to:

```text
commit_preplanned_node_completion(...)
```

Rationale:

- The main false-positive audit came from reading `commit_staged_combat_result_state()` before `commit_staged_node_completion()`.
- A combat-result-specific planning object makes the broad transaction boundary visible.
- The node completion rename communicates that local completion validity was already checked by planning.
- This is narrower than redesigning every node flow and should preserve behavior.

Important constraint:

- Do not change non-player combat result behavior in this goal unless compilation requires adapting the renamed helper.
- Do not change reward, XP, research, consumable, node completion, or DTO timing.

## Implementation Notes (2026-07-04)

Implemented the chosen shape.

Key outcome:

- The player-victory branch no longer contains the whole planning and commit sequence inline.
- The handler now delegates to a planning helper and then a commit helper, making the intended transaction boundary visible.
- `commit_preplanned_node_completion(...)` communicates that node completion validity is expected to have been established earlier by planning.

Behavior intentionally unchanged:

- Non-player combat result branches still use their existing flow.
- Reward, XP, research, consumable, node completion, and transition timing are intended to remain the same.
- No new user-visible DTO fields or compatibility paths were introduced.

Residual caveat:

- `commit_preplanned_node_completion(...)` still returns `Result` because it is shared with non-combat node flows and can apply support effects there. For the player-victory combat result path, the planned node completion is created with `apply_support_effect = false`, so the original false-positive audit path remains guarded by planning.

## Follow-Up Candidates

- Non-player combat result branches may deserve a similar readability cleanup later. Do not include them unless required by this refactor.
- If the refactor reveals duplicated planning logic between combat result and event combat completion, record it separately.
