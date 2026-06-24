# Experiments

This file records audit attempts, failed review approaches, validation commands, and final verification results for the `core_policy_implementation_master` completion review.

## Log

| Date | Scope | Attempt | Result | Follow-up |
| --- | --- | --- | --- | --- |
| 2026-06-23 | Goal setup | Created the completion review goal using `docs/goal_completion_review_guide.md` as the procedure. | Pending audit execution. | Start by reading the master goal and all subgoal docs, then build the item inventory for `AUDIT_REPORT.md`. |
| 2026-06-23 | Resume protocol | Re-read `PLAN.md`, `EXPERIMENTS.md`, `EXPERIMENT_NOTES.md`, and `AUDIT_REPORT.md` before any code search, edit, or validation. | Success. Required audit documents were treated as the current goal source of truth before continuing. | Keep this protocol for future compact/resume turns. |
| 2026-06-23 | Master/subgoal inventory | Read `docs/goal_completion_review_guide.md`, the master goal docs, and all 8 completed subgoal `PLAN.md` / `EXPERIMENTS.md` / `EXPERIMENT_NOTES.md` files. | Success. Built the item inventory for the audit table. | Verify every item against runtime/data/contract evidence. |
| 2026-06-23 | Runtime/data/contract evidence scan | Ran broad `rg` searches for removed fields, variants, fallbacks, compatibility paths, DTO shapes, live RON policy fields, and validation harness changes across `src`, `tests`, `../game_server/src`, and `../game_resources/data`. | Success. Most completed items have runtime/data evidence. Found three audit findings: duplicate `MapProgressionSnapshotDto` id-list projection, unimplemented `Timeline naming` rename, and stale final-validation text in `POLICY_COVERAGE.md`. | Record findings in `AUDIT_REPORT.md`; run validation after report update. |
| 2026-06-23 | Audit report | Replaced the placeholder audit report with item-by-item verdicts, detailed findings, cross-component findings, correction candidates, follow-up candidates, validation results, and policy questions. | Success. `AUDIT_REPORT.md` now satisfies the review deliverable shape. | Finalize validation log and close the review goal if completion conditions are met. |
| 2026-06-23 | Audit rerun validation | Ran focused and broad validation after the audit report update. | Success. `cargo check --lib`, live skill catalog audit, RON loading, legacy timeline validator test, `cargo check -p game_server`, full single-thread lib tests, and `git diff --check` passed. | Goal can be marked complete after final requirement audit. |

## Failed Approaches

None for the audit workflow yet.

## Validation Commands

Recorded implementation-subgoal validation was audited in the target docs.

- `cargo check --lib` - passed. Existing warning: `/mnt/f/work/simulator/auth_server/Cargo.toml: unused manifest key: env`.
- `cargo test --test live_skill_catalog_audit -- --test-threads=1` - passed, 3 tests.
- `cargo test --test ron_loading -- --test-threads=1` - passed, 18 tests.
- `cargo test --lib game::battle::validation::validator::tests::normal_validation_rejects_legacy_timeline_versions -- --test-threads=1` - passed, 1 test.
- `cargo check -p game_server` - passed. Existing warning: `/mnt/f/work/simulator/auth_server/Cargo.toml: unused manifest key: env`.
- `cargo test --lib -- --test-threads=1` - passed, 519 tests.
- `git diff --check` - passed.

## Review Inventory Notes

Initial target subgoals from `docs/goals/core_policy_implementation_master/PLAN.md`:

1. `docs/goals/core_policy_static_data_map_scenario`
2. `docs/goals/core_policy_grant_economy_rewards`
3. `docs/goals/core_policy_employee_item_growth`
4. `docs/goals/core_policy_buff_status_stats`
5. `docs/goals/core_policy_movement_battle_runtime`
6. `docs/goals/core_policy_skill_targeting_projectile`
7. `docs/goals/core_policy_unity_server_contract`
8. `docs/goals/core_policy_validation_test_harness`

The audit must not trust these subgoals just because they are marked complete. Each item needs runtime/data/contract/test evidence.

Current audit inventory status:

- All 8 completed subgoals were read.
- `AUDIT_REPORT.md` now contains item-by-item verdicts.
- Findings requiring follow-up:
  - `MapViewDto` list removal is incomplete at the broader snapshot-contract level because `MapProgressionSnapshotDto` still exposes `available_node_ids` / `completed_node_ids`.
  - `Timeline naming` was not implemented as the confirmed policy described; current code keeps serialized/client-facing `Timeline` for compatibility.
  - `POLICY_COVERAGE.md` contains stale final broad validation wording.
