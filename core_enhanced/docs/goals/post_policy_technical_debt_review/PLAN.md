# Post Policy Technical Debt Review Plan

## Current State

- Active goal: `docs/post_policy_technical_debt_review_goal.md`.
- This is an audit goal. Do not modify runtime/core/server code.
- Required work memory:
  - `docs/goals/post_policy_technical_debt_review/PLAN.md`
  - `docs/goals/post_policy_technical_debt_review/EXPERIMENTS.md`
  - `docs/goals/post_policy_technical_debt_review/EXPERIMENT_NOTES.md`
  - `docs/goals/post_policy_technical_debt_review/REPORT.md`
- Canonical Unity docs are external:
  - `/mnt/f/unity projects/ark/docs/unity_core_contract.md`
  - `/mnt/f/unity projects/ark/docs/unity_client_implementation_goal.md`

## Guardrails

- Do not use `git restore`, `git reset`, or other git file modifier commands.
- Do not copy historical git contents into current files with `cp`.
- Do not revert files to past states.
- Treat dirty worktree files as user/current work.
- Do not perform production code, test code, live RON, or server code edits in this goal.
- If a finding requires policy decisions, record it and stop for user discussion.

## Checklist

1. Create work memory files - done.
2. Read goal, master notes, and policy docs - done.
3. Inspect current diff/stat and changed code surfaces - done.
4. Review source-of-truth drift - done.
5. Review runtime/snapshot/server alignment - done.
6. Review validation placement and cost - done.
7. Review movement/Rapier/backend boundaries - done.
8. Review Unity-facing contract gaps - done.
9. Record test-debt observations without executing the full test-verification goal - done.
10. Write prioritized `REPORT.md` - done.

## Completion Conditions

- Work memory is present and updated.
- Runtime code, live data, Unity-facing contract surfaces, and latest policy docs have been inspected.
- Findings are sorted by severity in `REPORT.md`.
- Each finding includes evidence, risk, recommended direction, and whether user policy is needed.
- No runtime/core/server code was modified.
- Follow-up goal candidates are identified.
- User is given a concise summary.
- 사용자와 의논하여 정해야 할 정책이 있을 경우 goal을 종료한다.

## Final State

- Audit completed as a documentation/reporting goal.
- No runtime/core/server/live RON edits were made by this audit.
- `REPORT.md` contains the findings, deferred items, and policy questions.
