# Core Policy Implementation Master Completion Review

## Objective

Audit `docs/goals/core_policy_implementation_master` and every subgoal it marks complete. The goal is to verify whether each completed item is truly implemented in runtime code, live RON/data, Unity/server-facing contracts, and tests, and whether the implementation follows the project's long-term refactor direction.

This is an audit goal, not an implementation goal. Do not silently fix broad issues while reviewing. Produce precise findings and correction/refactor candidates that can become dedicated follow-up goals.

## Audit Guide

Use `docs/goal_completion_review_guide.md` as the required review procedure.

The review must apply:

- item-by-item evidence collection,
- source-of-truth order,
- legacy/fallback audit,
- long-term direction review,
- cross-component review,
- validation result recording.

## Target Master Goal

- Master: `docs/goals/core_policy_implementation_master`
- Policy source: `docs/goals/core_component_refactor_master/POLICY_DECISIONS.md`
- Refactor criteria: `docs/refactor_preparation_plan.md`
- Review guide: `docs/goal_completion_review_guide.md`

## Source Of Truth Order

For every reviewed item, verify facts in this order:

1. Actual runtime code.
2. Live RON/data loaded by runtime.
3. Unity-facing snapshot/command/WebSocket contracts.
4. Latest policy documents.
5. Goal documents and historical notes.

If a master/subgoal document says an item is complete but code/data/contract behavior does not prove it, classify the item as incomplete or risky.

## Subgoals To Review

Review these completed subgoals from the master in order:

1. `docs/goals/core_policy_static_data_map_scenario`
2. `docs/goals/core_policy_grant_economy_rewards`
3. `docs/goals/core_policy_employee_item_growth`
4. `docs/goals/core_policy_buff_status_stats`
5. `docs/goals/core_policy_movement_battle_runtime`
6. `docs/goals/core_policy_skill_targeting_projectile`
7. `docs/goals/core_policy_unity_server_contract`
8. `docs/goals/core_policy_validation_test_harness`

## Per-Subgoal Review Steps

For each subgoal:

1. Read `PLAN.md`, `EXPERIMENTS.md`, and `EXPERIMENT_NOTES.md`.
2. Extract every item marked complete or described as implemented.
3. Map each item to:
   - policy source,
   - runtime files,
   - live RON/data files,
   - Unity/server DTO or command contracts,
   - validation tests/commands.
4. Read the corresponding runtime code and data.
5. Search for old fields, old variants, removed schemas, fallback paths, compatibility layers, ignored tests, debug-only bypasses, and dual-source state.
6. Decide whether tests pin user-visible behavior and external contracts rather than private helper layout.
7. Run focused validation when needed to verify the reviewed item.
8. Assign both:
   - primary verdict,
   - long-term fit rating.

## Verdict Categories

Use exactly one primary verdict per reviewed item:

- Complete: implemented in code/data/contracts, covered by meaningful validation, no material legacy path remains.
- Partially complete: main path works, but an edge path, data loader, DTO, validation, or test gap remains.
- Document-only complete: goal docs mark it complete, but runtime/data behavior does not prove it.
- Policy decision needed: implementation requires a gameplay/UX/schema/balance decision not settled by current policy.
- Not applicable: the item became obsolete because a later confirmed policy removed the need.

Also assign long-term fit:

- High: design simplifies ownership and matches the expected future model.
- Medium: acceptable now, but follow-up simplification or consolidation is likely.
- Low: works by patching around the old model or leaving a confusing ownership split.

## Long-Term Direction Review

For each completed item, inspect whether the implementation is actually long-term and not just a local patch.

Check:

- Source of truth was reduced rather than duplicated.
- No new synchronized duplicate state was introduced.
- Legacy code/data/tests were removed instead of hidden behind compatibility layers.
- The implementation is not a special-case patch for one test.
- Names, types, and module locations match domain responsibility.
- Existing functions were composed where appropriate instead of adding a parallel feature.
- The structure can absorb future content/policy changes without another migration.
- Tests protect durable behavior rather than helper internals.

Record:

- Long-term fit: high / medium / low.
- Improvement class: none / follow-up refactor / immediate correction recommended.
- Concrete evidence.

## Cross-Component Review

After all subgoals are reviewed individually, perform a second pass across the whole master goal.

Check:

- competing sources of truth across subgoals,
- contradictions between DTO/schema assumptions,
- tests that pass only because components are not exercised together,
- live RON/data still following older policy,
- Unity-facing docs/contracts still describing older behavior,
- debug exports/logs accidentally used as runtime source.

Cross-component findings must be listed separately from per-item notes.

## Policy Decision Handling

If review discovers a missing policy decision, do not decide it silently.

Record:

- `사용자와 정책 논의 필요`,
- affected item/subgoal,
- code/data evidence,
- concrete options,
- recommended long-term choice if there is one.

The review goal may still complete after producing the policy question list. Do not convert review findings into implementation without user approval.

## Deliverables

Create and maintain:

- `docs/goals/core_policy_implementation_master_completion_review/PLAN.md`
- `docs/goals/core_policy_implementation_master_completion_review/EXPERIMENTS.md`
- `docs/goals/core_policy_implementation_master_completion_review/EXPERIMENT_NOTES.md`
- `docs/goals/core_policy_implementation_master_completion_review/AUDIT_REPORT.md`

`AUDIT_REPORT.md` must contain:

- item-by-item audit table,
- detailed evidence sections for incomplete or risky items,
- cross-component findings,
- immediate correction candidates,
- follow-up refactor candidates,
- validation commands and results,
- explicit policy questions, if any.

## Completion Conditions

- Every completed subgoal item in `core_policy_implementation_master` has a verdict.
- Every verdict has runtime/data/contract/test evidence or a recorded missing-evidence finding.
- Every reviewed item has a long-term fit rating.
- Legacy/fallback/dual-schema searches were performed for each reviewed area.
- Cross-component review was completed.
- Validation commands were run or explicitly skipped with reason.
- `AUDIT_REPORT.md` summarizes final findings, risks, and recommended next goals.

## Non-Goals

- Do not implement correction candidates in this goal.
- Do not rewrite policy documents except to record review findings.
- Do not change live RON/data unless the user explicitly turns this audit into an implementation goal.
- Do not update Unity external docs from this audit without a dedicated contract-sync goal.
