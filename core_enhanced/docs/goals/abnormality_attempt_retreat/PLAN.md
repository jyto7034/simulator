# Abnormality Attempt / Retreat / Re-entry Plan

## Objective

Implement abnormality entry attempts as durable run state instead of treating retreat as immediate node completion.

## Completion Conditions

- Combat abnormality nodes can be entered up to 3 times.
- Confirming entry starts exactly one attempt; selecting/previewing a node does not.
- Retreat before `BattleEnd` consumes one attempt.
- Retreat with remaining attempts returns to `NodeConfirm` for the same abnormality and does not complete the node.
- Retreat after all attempts are used consumes the node with a retreated failure outcome.
- Non-retreat failure and victory continue to consume the node through combat result completion.
- Node confirm and in-battle snapshots expose attempt state and retreat availability.
- Legacy tests that asserted retreat immediately consumes the node are replaced.
- If code review exposes a policy that must be decided with the user, stop the goal.

## Status

- Complete.

## Implementation Notes

- Attempt state belongs in `RunState`, keyed by `MapNodeId`. `active_battle` is per-entry and `node_session` is transient UI/session context.
- Unity contract currently says retreat is for non-boss live combat, so boss retreat remains invalid in this goal.
- Consumable duration policy is handled by the next goal, but this goal must not keep decrementing consumables on retreat because that would contradict re-entry semantics.
