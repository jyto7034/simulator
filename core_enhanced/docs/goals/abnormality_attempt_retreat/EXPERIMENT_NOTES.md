# Abnormality Attempt / Retreat / Re-entry Experiment Notes

## Findings

- Current `ConfirmEnterNode` calls `MapProgression::enter_node`, which reveals/selects the node but does not mark it completed.
- Current `RetreatBattle` creates a draw outcome, decrements consumables, then calls `handle_complete_node`; that is the legacy behavior to remove for attempts with remaining entries.
- `RunState` is the correct owner for attempt counters because it persists across active battle sessions within the current map.
- Canonical Unity contract says retreat is for non-boss live combat. Boss retreat remains invalid in this goal.
- Adding a new `BehaviorResult` variant would require game server mapping changes. The implementation reuses the existing `NodePreview` result for retreat-with-remaining-attempts and exposes attempt state through the authoritative snapshot.
- `RetreatBattle` is only available while `active_battle` exists. Once `BattleEnd` has finalized into `CombatResult`, `RetreatBattle` is no longer allowed.

## Follow-up Candidates

- The next consumable duration goal should formalize that retreat/re-entry does not decrement combat-node durations, while victory/failure node resolution does.
- Threat warning false-rumor `Disproved` can now key off attempt state and same-node re-entry, but remains out of scope here.
