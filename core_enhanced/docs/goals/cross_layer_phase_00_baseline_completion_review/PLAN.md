# Cross-Layer Phase 00 Baseline Completion Review

Status: complete

Implementation complete: true

Created: 2026-07-12

Master goal: `docs/goals/cross_layer_contract_hardening_master/PLAN.md`

Review ID: R00

Reviewed child: G04 `server_runstart_fixture_contract`

## Objective

Independently verify that Phase 0 established a reproducible, trustworthy Core/server/Unity evidence baseline and that G04 restored the six blocked server regressions without weakening production contracts or hiding the remaining G12 startup-cardinality work.

## Review Mode

This goal is read-only by default. It may update review and Master orchestration documents, but it must not silently change production code, fixtures, live data, server behavior, Unity behavior, or canonical contracts.

If an immediate correction is required, record a `Partially complete`, `Document-only complete`, `Policy decision needed`, or Low-fit verdict and create a bounded correction child. Do not fix broad issues inside R00.

## Required Inputs

- Three Master Goal documents and current Phase 0/G04 progress.
- G04 `PLAN.md`, `EXPERIMENTS.md`, `EXPERIMENT_NOTES.md`, and `COMPLETION_REVIEW.md`.
- `docs/goal_completion_review_guide.md`.
- DCA-H-08 and CPR-M-01 audit text.
- Current runtime, live trigger RON, server fixture, server tests, Unity runner metadata, and Rust serialization path.

## Expected Phase Outcome

- A startable server fixture supplies paired pre-run Visual Novel definitions and mode-specific RunStart triggers.
- Missing Standard trigger remains a Core error.
- The six previously blocked server tests execute their intended assertions.
- Core and server broad baselines are reproducible from the current worktree.
- Unity runner/version/test assemblies are known even though no Unity change requires execution in Phase 0.
- At least one real typed Rust snapshot serialization path is proven.
- Zero-test commands and unavailable runner paths are classified honestly.
- CPR-M-01's early supported-mode cardinality concern remains assigned to G12 rather than being falsely closed by G04.

## Review Procedure

1. Restate Phase 0 and G04 expected behavior and source-of-truth ownership.
2. Inspect the current runtime rejection path, live RunStart RON, server paired fixture, and six server tests.
3. Search for the old definition-only fixture, validation bypasses, aliases, ignored tests, assertion weakening, and duplicate trigger owners in the changed server module.
4. Re-run formatting and server check.
5. Independently re-run full `game_server --all-targets` and `game_core --all-targets` with exact test counts.
6. Reconfirm Unity Editor/project version, test assemblies, and actual Rust snapshot serialization path.
7. Perform the guide's cross-component and long-term-direction reviews.
8. Write `COMPLETION_REVIEW.md` with exact verdicts and remaining risk.
9. Only if all required items are Complete or evidence-backed Not applicable, mark R00 and Phase 0 complete and unlock Phase 1.

## Completion Conditions

- G04 is `Complete` with High or acceptable Medium fit and no immediate correction.
- Server broad result is 25 passed / 0 failed or a documented larger all-green count caused by intentional added tests.
- Core broad result is 681 passed / 0 failed or a documented larger all-green count caused by intentional added tests.
- Formatting and server check pass.
- All six audit-named tests are present in the broad result.
- No legacy/bypass/compatibility path remains for the fixture.
- Unity runner/test entry points and Rust serialization path are evidenced.
- G12 ownership of early supported-mode RunStart cardinality is explicit in the Master.
- `COMPLETION_REVIEW.md` reports Phase 0 `Complete` with High long-term fit and no immediate correction.

## Stop Conditions

- Any production or policy change is needed to make Phase 0 pass.
- A required Core/server runner is unavailable or broad tests fail.
- The server fixture cannot be separated from a new DTO/live-schema/gameplay decision.
- G04's result depends on reverting user-owned dirty-worktree changes.

If a policy decision is required, set `Status: policy_decision_required`; do not mark the review complete.

## Review Result

- Verdict: Complete.
- Long-term fit: High.
- Immediate correction: none.
- Core reproduced baseline: 681 passed / 0 failed.
- Server reproduced baseline: 25 passed / 0 failed.
- Phase 1 may begin with G01.
- G12 retains the CPR-M-01 supported-mode RunStart cardinality follow-up.
