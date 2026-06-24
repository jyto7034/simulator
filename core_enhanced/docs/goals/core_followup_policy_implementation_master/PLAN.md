# Core Follow-Up Policy Implementation Master

## Objective

Implement the confirmed but unimplemented follow-up policies recorded in `docs/goals/core_component_refactor_master/POLICY_DECISIONS.md`, grouped by implementation difficulty and blast radius.

This master goal does not re-open already settled policy choices. It coordinates subgoals that turn the confirmed policies into long-term runtime/server contracts.

## Subgoals By Difficulty

1. `core_snapshot_error_contract_cleanup` - easy to medium
   - Remove `PlayerStateSnapshotDto` / `RunSnapshotDto` top-level field drift risk.
   - Rename or split `PositionOccupied` / `position_occupied` meanings so static obstacle blocking does not imply unit tile occupancy.
2. `core_battlefield_layout_extraction` - hard
   - Rename `Battlefield` to a static layout object such as `BattlefieldLayout`.
   - Remove dynamic unit position ownership from the layout object.
   - Make DTO tile `position` derive from `RuntimeUnit.body.position`.
   - Rename valid-tile APIs and keep walkable/void-tile meaning explicit.
3. `core_runtime_unit_lifecycle` - very hard
   - Add `RuntimeUnitLifecycle { Active, Withdrawn, Dead }`.
   - Make lifecycle the canonical active/dead/withdrawn participation gate.
   - Update movement, targeting, skill, damage candidate, and checkpoint filters.
4. `core_redeploy_lifecycle_policy` - very hard
   - Implement withdraw/death redeploy identity and HP policies.
   - Redeploy creates a new `RuntimeUnit` and new `UnitInstanceId`.
   - Old withdrawn/dead runtime units remain in the battle registry for references.
5. `core_inactive_unit_effect_resolution` - hard
   - Implement projectile/delayed-effect behavior around `Withdrawn` and `Dead` lifecycle states.
   - Preserve launch/lock/schedule snapshots where policy requires it.
6. `core_lifecycle_event_snapshot_contract` - medium to hard
   - Add/adjust `UnitWithdrawn` and `UnitDied` event position contracts.
   - Keep official checkpoint `units` Active-only.
   - Separate debug/admin/replay inactive unit visibility from gameplay presentation.

## Required Startup Protocol

At the beginning of this master goal and every resumed run, read:

- `docs/goals/core_followup_policy_implementation_master/PLAN.md`
- `docs/goals/core_followup_policy_implementation_master/EXPERIMENTS.md`
- `docs/goals/core_followup_policy_implementation_master/EXPERIMENT_NOTES.md`
- `docs/goals/core_component_refactor_master/POLICY_DECISIONS.md`
- `docs/refactor_preparation_plan.md`
- `docs/goal_completion_review_guide.md`
- the active subgoal's `PLAN.md`, `EXPERIMENTS.md`, and `EXPERIMENT_NOTES.md`

Do not start code search, edits, or validation before reading those documents.

## Source Of Truth Order

1. Actual runtime code.
2. Live RON/data.
3. Unity-facing snapshot/command/WebSocket contracts.
4. Latest policy documents.
5. Goal documents and historical notes.

If code/data contradicts goal text, treat code/data as evidence and update `EXPERIMENT_NOTES.md`. If the contradiction requires a new gameplay/UX/schema/balance decision, complete this goal with a policy-decision report instead of guessing.

## Execution Order

For every subgoal below:

1. Read that subgoal's `PLAN.md`, `EXPERIMENTS.md`, and `EXPERIMENT_NOTES.md`.
2. Execute the subgoal according to its own completion and policy-decision rules.
3. When the subgoal is complete, immediately run a completion review using `docs/goal_completion_review_guide.md`.
4. Compare the implementation against actual runtime code, live RON/data, Unity-facing contracts, tests, the subgoal documents, and `POLICY_DECISIONS.md`.
5. Record the review result in this master goal and in the active subgoal notes.
6. If the review finds an implementation gap that is still inside the subgoal scope, fix it before moving to the next subgoal.
7. If the review finds a new policy decision requirement, complete this master goal with a policy-decision report and return to the user.
8. If the review finds only out-of-scope follow-up work, record it in `EXPERIMENT_NOTES.md` and move to the next subgoal.

Subgoal order:

1. Run and review `core_snapshot_error_contract_cleanup`.
2. Run and review `core_battlefield_layout_extraction`.
3. Run and review `core_runtime_unit_lifecycle`.
4. Run and review `core_redeploy_lifecycle_policy`.
5. Run and review `core_inactive_unit_effect_resolution`.
6. Run and review `core_lifecycle_event_snapshot_contract`.
7. Re-read all subgoal outputs and review reports, then compare the combined result against `POLICY_DECISIONS.md`.
8. Run broad validation for the combined blast radius.
9. Record final status in this master goal.

## Per-Subgoal Review Gate

Each subgoal must pass a review gate before the master goal proceeds to the next subgoal.

Use `docs/goal_completion_review_guide.md` as the review procedure. The review must check:

- the implemented runtime behavior, not only checked boxes in goal documents,
- live RON/data compatibility,
- Unity-facing snapshot/command/WebSocket contract shape,
- tests that pin user-visible behavior instead of implementation details,
- absence of compatibility layers, fallback paths, dual schemas, and ignored legacy tests,
- whether the result follows the long-term direction rather than a narrow local patch,
- whether a simpler or more source-of-truth-aligned design is available after reading the code.

The review output must include:

- item-by-item verdict,
- code/data/contract evidence,
- remaining gaps,
- whether each gap is in-scope fix, out-of-scope follow-up, or `사용자와 정책 논의 필요`,
- focused validation commands run for the subgoal.

## Common Implementation Rules

- Code changes must follow the long-term direction, not temporary patches or test-only fixes.
- Do not add compatibility layers, fallback paths, dual schemas, or ignored legacy tests.
- Tests should pin user-visible behavior, Unity-facing DTOs, data validation, live RON loading, and real gameplay flows.
- For every small change group, run focused validation or `cargo check`.
- Run broad validation at the end.
- Out-of-scope improvements go to `EXPERIMENT_NOTES.md`; skip them and move to the next target unless required to reduce current blast radius.

## Policy Decision Completion Condition

If implementation discovers a required decision involving Unity-facing DTO shape, live RON schema, save migration, UX meaning, balance policy, existing content deletion/replacement, or failure/reward/consumption timing, do not choose silently.

Even when a policy appears already decided in `POLICY_DECISIONS.md`, treat that decision as insufficient if actual runtime code, live data, Unity-facing contracts, or tests reveal a missing edge case, ambiguous ownership boundary, conflicting behavior, or broader gameplay meaning. In that case, do not continue by stretching the old policy to fit the code.

Settled-looking policy is not automatically sufficient. If the implementation pass shows that the chosen policy does not answer the concrete runtime behavior, DTO shape, data validation rule, lifecycle edge case, or test contract in front of the code, the correct deliverable is a policy-decision report, not more implementation.

Do not pause, block, or leave this goal active when a new policy decision is required. Treat the policy-decision report as a valid completed deliverable, mark the goal complete, and return to the user for discussion.

Operationally, this means the goal status must become `complete`, not `blocked`, `paused`, or merely "ended". The completed output is the policy-decision report itself.

This termination rule is identical to the subgoal termination rule. If any subgoal hits this condition, the master goal also completes with the same policy-decision report instead of continuing to later subgoals.

Complete this goal with:

- `사용자와 정책 논의 필요`
- the blocking area,
- concrete options,
- recommendation,
- code/data evidence.

This rule is mandatory. Do not silently defer a required policy choice to `EXPERIMENT_NOTES.md` and proceed with implementation when the current code shows that the confirmed policy is not enough to choose a correct long-term behavior.

## Completion Conditions

- All subgoals are complete or explicitly completed with a policy-decision report.
- Each completed subgoal has a `docs/goal_completion_review_guide.md` based review result recorded before the next subgoal begins.
- `POLICY_DECISIONS.md` implementation status is updated for each follow-up policy.
- No confirmed unimplemented policy is silently skipped.
- Focused and broad validation results are recorded.
- Final report includes:
  - changed contracts,
  - removed legacy,
  - tests updated,
  - remaining risk,
  - validation commands.

## Current Master Status

Status: complete; all listed subgoals are implemented, reviewed, cross-audited, and broad serial validation passed.

Latest policy-decision report: `docs/goals/core_inactive_unit_effect_resolution/POLICY_DECISION_REPORT.md`.

Latest completed review: `docs/goals/core_lifecycle_event_snapshot_contract/REVIEW.md`.

Previous completed review: `docs/goals/core_inactive_unit_effect_resolution/REVIEW.md`.

Previous completed review: `docs/goals/core_redeploy_lifecycle_policy/REVIEW.md`.

Earlier completed review: `docs/goals/core_runtime_unit_lifecycle/REVIEW.md`.

Confirmed and implemented policy: withdrawal cleanup emits explicit typed event-log reasons: `BuffExpireReason::TargetWithdrawn`, `BuffExpireReason::CasterWithdrawn`, and `BattleLogEvent::SkillCastCancelled { ..., reason: SkillCastCancelReason::Withdrawn }`. `SkillCastInterrupted` remains external-interruption only.

Implemented policy: death redeploy creates a fresh runtime unit and starts at `max(1, floor(max_hp * 0.60))` current HP, clamped by max HP. Withdraw redeploy creates a fresh runtime unit and carries only `min(max_hp, withdrawn_hp + floor(max_hp * 0.30))` HP. Persistent trauma/injury remains post-battle-only.

Newly confirmed policy: withdrawal is a strong defensive/evasion action against incoming hostile projectiles fired by the opponent. Projectiles flying toward a target that becomes `Withdrawn` cancel/miss and do not damage old runtime HP, do not update redeploy HP locks, and do not convert old withdrawn units into `Dead`.

Latest implementation: `core_inactive_unit_effect_resolution` hardened basic attack projectile impact lifecycle guards and added focused tests for locked basic attack and skill projectile impact against withdrawn targets.

Latest implementation: `core_lifecycle_event_snapshot_contract` added `UnitWithdrawn`/`UnitDied` event positions, kept checkpoint units Active-only, and added a debug/replay-only inactive unit query.

Final review: `docs/goals/core_followup_policy_implementation_master/REVIEW.md`.

Final validation:

- `cargo check`
- `cargo test -- --test-threads=1`

Remaining follow-up risks:

- Add a typed server/admin transport endpoint for inactive runtime inspection only if external tooling needs it.
- Isolate battle record/debug artifact outputs so parallel full `cargo test` cannot contend on generated JSON files.
- Ensure Unity/external consumers adopt battle event log schema version 27 and the new `UnitWithdrawn`/`UnitDied` position fields.
