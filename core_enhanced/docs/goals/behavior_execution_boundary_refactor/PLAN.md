# Behavior Execution Boundary Refactor

## Objective

Resolve P-007 by refactoring `GameCore::execute_with_source_command_id(...)` so the behavior execution pipeline is easier to read and safer to extend.

Current implementation centralizes action gating, payload validation, behavior dispatch, source command id routing, handler invocation, and roster sync in one broad method. The goal is not to change behavior; it is to make the execution stages explicit.

## Source Of Truth Order

1. Runtime behavior in `src/game/world.rs` and handler modules.
2. Existing allowed-action and payload-validation tests.
3. Unity/server command contracts.
4. Canonical docs.
5. Audit notes.

## Scope

- Split execution into named phases such as gate, validate, dispatch, and postprocess.
- Keep public command behavior stable.
- Keep source command id handling for battle commands correct.
- Strengthen tests where the refactor exposes unclear behavior.

## Non-Goals

- Do not redesign `PlayerBehavior`.
- Do not change Unity-facing command DTOs or server messages.
- Do not change allowed action policy, payload validation policy, or roster ordering semantics.
- Do not move every handler into a new command framework unless the current boundary cannot be made readable otherwise.

## Plan

1. Re-read:
   - `src/game/world.rs`
   - `src/game/world/helpers.rs`
   - `src/game/behavior.rs`
   - tests covering allowed actions, invalid actions, payload validation, battle commands, and roster sync.
2. Map current execution phases:
   - derive action kind;
   - allowed-action gate;
   - payload validation;
   - behavior dispatch;
   - source command id handoff;
   - post-dispatch roster sync.
3. Choose the smallest long-term structure:
   - extracted helper methods;
   - a local `BehaviorExecution`/`ValidatedBehavior` type;
   - or a dispatcher table if the match has become unmaintainable.
4. Record the chosen shape in `EXPERIMENT_NOTES.md` before editing.
5. Refactor in small steps and run focused tests after each step.
6. Add tests only around user-visible behavior and command contract boundaries, not around private helper shapes.

## Completion Conditions

- `execute_with_source_command_id(...)` no longer hides every execution responsibility in one large block.
- The code makes it obvious where actions are gated, where payloads are validated, where handlers are called, and where postprocessing occurs.
- Existing command results and errors are preserved.
- Battle command `source_command_id` behavior is preserved.
- Roster sync still occurs exactly where intended.
- No compatibility layer or dual behavior path is introduced.

## Implementation Result (2026-07-06)

Status: complete.

Implemented the smallest boundary refactor:

- `execute_with_source_command_id(...)` now reads as explicit execution stages:
  - derive action kind;
  - `gate_behavior_action(...)`;
  - `validate_behavior_payload(...)`;
  - `dispatch_behavior(...)`;
  - `postprocess_behavior_execution(...)`.
- `dispatch_behavior(...)` keeps the existing `PlayerBehavior` match and battle `source_command_id` handoff intact.
- `postprocess_behavior_execution(...)` keeps roster-order sync as the single successful-command postprocess stage.
- No `PlayerBehavior` redesign, DTO change, allowed-action policy change, payload validation change, source command id policy change, or compatibility layer was introduced.

## Validation Commands

- `cargo test -p game_core allowed_actions --lib -- --test-threads=1`
- `cargo test -p game_core invalid_action --lib -- --test-threads=1`
- `cargo test -p game_core payload --lib -- --test-threads=1`
- `cargo test -p game_core game::world::tests::combat --lib -- --test-threads=1`
- `cargo test -p game_core game::world::tests::snapshots_and_start --lib -- --test-threads=1`
- `cargo check -p game_core`
- `cargo fmt --check`

Adjust filters after reading actual test names.

## Stop Conditions

Complete the goal and report questions if:

- The refactor requires changing command DTO shape.
- Allowed-action policy is ambiguous or conflicts with current docs.
- A behavior currently depends on skipped validation or skipped roster sync.
- Source command id semantics need a new policy decision.
