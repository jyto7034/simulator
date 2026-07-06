# Experiment Notes

## Initial Audit Basis

- `GameCore::execute_with_source_command_id(...)` currently performs action gate, payload validation, dispatch, source-command handling, and roster sync.
- This is not known to be a runtime bug.
- The concern is future safety and reader cost when adding or changing behavior commands.

## Policy Notes

- Preserve current behavior unless a user-visible bug is discovered.
- Prefer explicit stages over clever dispatch abstractions.
- Tests should lock behavior/results, not helper function names.

## Chosen Refactor Shape

After re-reading `src/game/world.rs`, `src/game/world/helpers.rs`, `src/game/behavior.rs`, and the nearby allowed-action/source-command tests, use the smallest long-term structure:

```text
execute_with_source_command_id(...)
-> derive action kind
-> gate_behavior_action(...)
-> validate_behavior_payload(...)
-> dispatch_behavior(...)
-> postprocess_behavior_execution(...)
```

Rationale:

- A dispatcher table or new command framework would be larger than the current goal needs.
- `PlayerBehavior` shape and Unity/server command DTOs should remain unchanged.
- `source_command_id` is only relevant to selected live battle commands, so keep that handoff visible inside dispatch rather than hiding it in generic metadata.
- Roster sync should remain a single post-dispatch stage, preserving the current policy that successful command execution triggers roster-order reconciliation.

## Implementation Notes

- Implemented the chosen shape with private helper methods on `GameCore`:
  - `gate_behavior_action(...)`
  - `dispatch_behavior(...)`
  - `postprocess_behavior_execution(...)`
- Left `validate_behavior_payload(...)` in `helpers.rs`; the execute method now calls it as the named validation stage between gate and dispatch.
- Did not introduce a new dispatcher table or `ValidatedBehavior` wrapper because the existing match remains readable once surrounded by explicit stages.
- Did not change command DTOs, action policy, payload validation rules, source command id routing, or roster sync timing.

## Follow-Up Candidates Outside Scope

- Splitting `PlayerBehavior` by domain.
- Server-side command-result envelope refactor.
