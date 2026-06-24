# Core Unimplemented Policy Implementation Master

## Objective

Implement the unimplemented confirmed policies recorded in `docs/goals/core_component_refactor_master/POLICY_DECISIONS.md`.

This master goal coordinates the subgoals below:

1. `core_tile_validity_typed_dto_cleanup`
   - Audit `Battlefield::in_bounds` call sites during the tile/range pass.
   - Move remaining server/admin snapshot JSON wrapper shape assembly to typed DTOs.
2. `core_skill_targeting_explicit_cleanup`
   - Remove `SkillStepDef.range_units`.
   - Remove `SkillCastTargetingDef::FirstStepTarget`.
   - Replace internal test/fixture convenience with explicit cast-targeting helpers that do not infer from steps.
3. `core_battlefield_occupancy_cleanup`
   - Remove `Battlefield` single-tile `occupant` projection and related single-owner tile semantics.
   - Keep unit position/body state as the source of truth.

## Required Startup Protocol

At the beginning of this goal and every resumed run, read:

- `docs/goals/core_unimplemented_policy_implementation_master/PLAN.md`
- `docs/goals/core_unimplemented_policy_implementation_master/EXPERIMENTS.md`
- `docs/goals/core_unimplemented_policy_implementation_master/EXPERIMENT_NOTES.md`
- `docs/goals/core_component_refactor_master/POLICY_DECISIONS.md`
- the active subgoal's `PLAN.md`, `EXPERIMENTS.md`, and `EXPERIMENT_NOTES.md`
- `docs/refactor_preparation_plan.md`

Do not start code search, edits, or validation before reading those documents.

## Source Of Truth Order

1. Actual runtime code.
2. Live RON/data.
3. Unity-facing snapshot/command/WebSocket contracts.
4. Latest policy documents.
5. Goal documents and historical notes.

If code/data contradicts goal text, treat code/data as evidence and update the goal notes. If the contradiction requires a new gameplay/UX/schema/balance decision, complete this goal with a policy-decision report instead of guessing.

## Execution Order

1. Run `core_tile_validity_typed_dto_cleanup`.
2. Run `core_skill_targeting_explicit_cleanup`.
3. Run `core_battlefield_occupancy_cleanup`.
4. Re-read all subgoal outputs and compare against `POLICY_DECISIONS.md`.
5. Run broad validation for the combined blast radius.
6. Record final status in this master goal.

## Common Implementation Rules

- Code changes must follow the long-term direction, not temporary patches.
- Do not add compatibility layers, fallback paths, dual schemas, or ignored legacy tests.
- Tests should pin user-visible behavior, Unity-facing DTOs, data validation, live RON loading, and real gameplay flows.
- For every small change group, run focused validation or `cargo check`.
- Run broad validation at the end.
- Out-of-scope improvements go to `EXPERIMENT_NOTES.md`; skip them unless required to reduce current blast radius.

## Policy Decision Completion Condition

If implementation discovers a required decision involving Unity-facing DTO shape, live RON schema, save migration, UX meaning, balance policy, existing content deletion/replacement, or failure/reward/consumption timing, do not choose silently.

Do not pause, block, or leave this goal active when a new policy decision is required. Treat the policy-decision report as a valid completed deliverable, mark the goal complete, and return to the user for discussion.

Operationally, this means the goal status must become `complete`, not `blocked`, `paused`, or merely "ended". The completed output is the policy-decision report itself.

Complete this goal with:

- `사용자와 정책 논의 필요`
- the blocking area,
- concrete options,
- recommendation,
- code/data evidence.

## Completion Conditions

- All three subgoals are complete or explicitly completed with a policy-decision report.
- `POLICY_DECISIONS.md` unimplemented section is updated to reflect implemented/deferred status.
- No confirmed unimplemented policy is silently skipped.
- Focused and broad validation results are recorded.
- Final report includes:
  - changed contracts,
  - removed legacy,
  - tests updated,
  - remaining risk,
  - validation commands.
