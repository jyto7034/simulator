# Post Policy Code Repair Master Plan

## Objective

Execute `docs/goal.md`, which points to `docs/post_policy_code_repair_master_goal.md`.

The master order is:

1. Complete Phase 1 test/contract green work before broad Phase 2 refactors.
2. Then perform non-policy-gated technical debt repair.
3. Stop and ask if resource deletion or a genuinely new policy choice is required.

## Guardrails

- Do not use `git restore`, `git reset`, or other git file modifier commands.
- Do not copy historical git contents into current files with `cp`.
- Do not revert user/current worktree changes.
- Treat the dirty worktree as current state.
- Do not weaken strict battlefield route validation to make tests pass.
- Do not count 0-test cargo filters as passing evidence.
- Record test counts for verification commands.
- External Unity canonical docs:
  - `/mnt/f/unity projects/ark/docs/unity_core_contract.md`
  - `/mnt/f/unity projects/ark/docs/unity_client_implementation_goal.md`

## Phase 1 Checklist

1. Reproduce current `game_core` failing test.
2. Fix `authored_protect_unit_tactical_plan_overrides_default_defense_contract` without weakening route validation.
3. Reproduce and fix `game_server` test cfg compile failure.
4. Add/refresh Unity-facing DTO serialization assertions for deployment costs, mobility, and preferably damage feedback.
5. Remove or replace 0-test `tests/unit_test.rs`.
6. Add focused coverage for attempt/retreat edge cases.
7. Add focused coverage for threat warning false-rumor lifecycle.
8. Run Phase 1 integration gate.

## Phase 2 Checklist

Pending until Phase 1 is green.

## Completion Conditions

- Phase 1 green issues are fixed and verified.
- Non-policy-gated Phase 2 items are completed or explicitly deferred with reason.
- Policy-gated deletion of external resource files is not performed without user approval.
- Final `REPORT.md` lists changes, removed legacy, contracts, tests, risks, and commands with counts.
