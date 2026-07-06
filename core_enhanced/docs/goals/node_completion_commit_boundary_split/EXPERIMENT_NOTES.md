# Node Completion Commit Boundary Split Notes

## Initial Context

This goal follows a completed audit of `combat_result_completion_transaction_boundary_refactor`.

The current behavior appears safe, but the final commit helper is still broad:

```text
commit_preplanned_node_completion(...)
```

It currently covers map/progression mutation, boss omen source consumption, event-session cleanup, support effect application, save checkpoint persistence, floor advancement, run completion, active node cleanup, and game-state transition.

## Working Hypothesis

The desired refactor is likely a responsibility split, not a behavior repair.

The implementation should make combat-result completion easier to read by separating combat-result commit from support/save/gate/floor specific commit logic.

## Current Call Graph

Observed before editing:

```text
CompleteCombatResult player victory
  -> plan_player_victory_combat_result_completion(...)
     -> plan_complete_current_node(false)
  -> commit_planned_player_victory_combat_result_completion(...)
     -> commit_staged_combat_result_state(...)
     -> commit_preplanned_node_completion(...)

CompleteNode / support / reward / shop / headquarters flows
  -> handle_complete_node()
     -> plan_complete_current_node(true)
     -> commit_preplanned_node_completion(...)

Event scene End / Event choice End
  -> plan_complete_current_node(true)
  -> commit_preplanned_node_completion(...)

Gate transition
  -> handle_confirmed_gate_transition(...)
  -> already uses a separate flow and does not call commit_preplanned_node_completion(...)
```

`StagedNodeCompletion` variants:

```text
NodeCompleted
  normal node completion, support/rest/save completion, event node completion, combat result for non-terminal combat nodes.

FloorAdvanced
  terminal floor completion, including terminal event/combat nodes.

RunComplete
  final standard run completion.
```

`support_effect` is planned only when `plan_complete_current_node(true)` is used. Player-victory combat result uses `plan_complete_current_node(false)`, so it should carry `StagedSupportEffect::None`.

## Chosen Split

Use flow-specific commit entrypoints and shared low-level primitives:

```text
commit_interactive_node_completion(...)
  applies planned support/rest/save effects and checkpoint persistence.

commit_combat_result_node_completion(...)
  applies only the preplanned node/map/session/floor/run transition for combat result completion.

shared helpers
  apply variant-specific run/map/session mutations so behavior stays identical.
```

This keeps existing staged data and behavior while removing the combat result dependency on a broad helper whose name implies support/save/floor responsibilities.

## Final Shape

Implemented entrypoints:

```text
commit_interactive_node_completion(...)
  used by CompleteNode and Event scene/choice completion.
  preserves support/rest/savepoint effect handling.

commit_combat_result_node_completion(...)
  used only by player-victory combat result completion.
  accepts only CombatResultNodeCompletion, which has no support effect field.
```

Shared helpers apply the same `StagedNodeCompletion` variants as before, but the caller-facing names now communicate intent.

Hardening note:

```text
plan_complete_current_node(false)
  -> CombatResultNodeCompletion::try_from(staged)
  -> commit_combat_result_node_completion(combat_completion)
```

This intentionally rejects support-bearing staged completions before combat rewards are committed. The contract is no longer debug-build-only.

## Policy Notes

- No reward, retry, savepoint, gate, or floor policy is being changed by this goal.
- If a policy change appears necessary, stop and ask.

## Follow-Up Candidates

- After this split, a later cleanup may be able to rename or simplify `StagedNodeCompletion` variants.
- If support/save/floor completion still feels too coupled, create a separate node-flow decomposition goal rather than expanding this one.
