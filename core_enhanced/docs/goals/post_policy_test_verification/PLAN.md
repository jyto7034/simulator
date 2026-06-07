# Post Policy Test Verification Plan

## Current State

- Active goal: `docs/post_policy_test_verification_goal.md`.
- This is a verification/audit goal. Do not modify production code, test code, or live RON.
- Required work memory:
  - `docs/goals/post_policy_test_verification/PLAN.md`
  - `docs/goals/post_policy_test_verification/EXPERIMENTS.md`
  - `docs/goals/post_policy_test_verification/EXPERIMENT_NOTES.md`
  - `docs/goals/post_policy_test_verification/REPORT.md`
- Canonical Unity docs are external:
  - `/mnt/f/unity projects/ark/docs/unity_core_contract.md`
  - `/mnt/f/unity projects/ark/docs/unity_client_implementation_goal.md`

## Guardrails

- Do not use `git restore`, `git reset`, or other git file modifier commands.
- Do not copy historical git contents into current files with `cp`.
- Do not revert files to past states.
- Treat dirty worktree files as user/current work.
- Do not edit production code, test code, live RON, or server code.
- Do not count a 0-test cargo filter as passing verification.
- If a failure requires user policy decisions, record it and stop for user discussion.

## Verification Strategy

1. Read the goal document and relevant policy/master notes.
2. Read completed sub-goal `EXPERIMENTS.md` files and extract recorded verification commands, counts, pass/fail notes, and 0-test filters.
3. Use `cargo test -- --list` style commands where useful to verify current test inventory without changing code.
4. Run focused verification commands only when they are reasonably scoped and useful evidence.
5. Avoid immediately running the full test suite until focused inventory and check results are understood.
6. Record every executed command, result, and test count where available.

## Checklist

1. Create work memory files - done.
2. Read goal, policy docs, and master verification notes - done.
3. Extract completed-goal verification commands and suspicious zero-test filters - done.
4. Inspect current test inventory by policy area - done.
5. Run or list focused test commands and record counts - done.
6. Review live RON/data validation coverage - done.
7. Review Unity-facing DTO coverage - done.
8. Write `REPORT.md` with passing, failed, suspicious, and coverage gap sections - done.
9. Run documentation sanity checks - done.

## Current Verification Scope

- `cargo check -p game_core` and `cargo check -p game_server` were used as compile baselines.
- `cargo test -p game_core -- --list` was used to inventory current game_core test targets.
- Focused `-- --list` commands were used for policy-area filters so 0-test filters could be separated from real coverage.
- Actual integration test commands were run for `ron_loading`, `skill_refactor_validation`, `skill_test_suite`, `live_item_skill_activation`, and `live_skill_catalog_audit`.
- `cargo test -p game_core` was run and currently fails on one lib test.
- `cargo test -p game_server -- --list` was run and currently fails to compile under test configuration.
- No production code, test code, server code, or live RON was modified.
- Documentation sanity checks found no conflict markers, no leftover pending placeholders, and no trailing whitespace in the goal docs.

## Completion Conditions

- Work memory is present and updated.
- Major verification commands were run or explicitly not run with reasons.
- 0-test commands and suspicious successes are separated from real passing evidence.
- Coverage gaps are summarized for all required policy areas.
- Failed/unstable/brittle/stale tests are reported without fixes.
- Follow-up test cleanup goal candidates are listed.
- 사용자와 의논하여 정해야 할 정책이 있을 경우 goal을 종료한다.

## Stop Conditions

- `cargo check` fails in a way that makes further test interpretation unreliable.
- Full test execution is not feasible due to time/environment/external dependency constraints.
- A test failure requires user decision between old and new policy.
- 사용자와 의논하여 정해야 할 정책이 있을 경우 goal을 종료한다.
