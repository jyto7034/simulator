# Phase 01 Goal — CRA-P09-C08 Exact-Cause Negative Target Tests

Status: complete
Implementation Complete: true
Goal Type: phase
Execution Mode: implement_and_verify
Candidate ID: CRA-P09-C08
Master Phase: 01 of 44
Work Type: correctness correction (test)
Production Mutation Authorized: yes — the user's 2026-08-09 request to read and perform the Master PLAN explicitly
activates the implementation campaign and Phase 01; authorization is limited to this Phase until `Complete / High`
Repository Root: `/mnt/f/work/simulator`
Activation Date: 2026-08-09 (`Asia/Seoul`)

## Objective And Completion Conditions

Repair the negative target-schema safety net so every retained CRA-P09-C08 case proves the rejection it names.
An otherwise-valid fixture must parse when its single defect is repaired, then fail at the intended typed
object/field/path and cause when that defect is restored. Catch-any-panic and bare `is_err` assertions in the exact
candidate scope must no longer let unrelated missing fields satisfy a named contract.

This Phase is complete only when the current affected tests and runners are identified from the live tree, their
pre-existing dirty state is preserved, the repaired cases execute nonzero counts, the focused and affected broad
validation passes, the scoped false-positive patterns are removed or explicitly dispositioned, and an independent
completion review returns `Complete / High`.

## User Or Contract-Visible Problem

The audited inline-range/preset-conflict fixture omits required `delay_ms`. Its catch-any-panic assertion therefore
passes on an unrelated missing-field failure instead of proving the intended targeting conflict. Other legacy
negative RON cases may similarly use bare `is_err` while omitting unrelated required step fields. Such green tests
cannot safely guard the later strict publication correction CRA-P09-C01.

## Dependency And Authority

- Master: `../core_enhanced_refactoring_rebaselined_backlog_implementation_master/PLAN.md`, Phase 01 registry row.
- Direct predecessor: none; the Master initialization and explicit user activation are the only entry gate.
- Downstream dependency: CRA-P09-C01 relies on these exact-cause negatives before publication rejection expands.
- Historical current evidence:
  - `../core_enhanced_refactoring_post_boss_runtime_reaudit_phase_09_battle_targeting_delivery_and_projectiles/CANDIDATE_REAUDIT.md`
  - `../core_enhanced_refactoring_post_boss_runtime_reaudit_phase_09_battle_targeting_delivery_and_projectiles/TEST_REAUDIT.md`
  - `../core_enhanced_refactoring_audit_phase_09_battle_targeting_and_delivery/CANDIDATE_CATALOG.md`
- Final execution evidence:
  - `../core_enhanced_refactoring_post_boss_runtime_reaudit_final_catalog_consolidation/EXACT_LINEAR_IMPLEMENTATION_ORDER.md`
  - `../core_enhanced_refactoring_post_boss_runtime_reaudit_final_catalog_consolidation/TEST_REBASELINE_CATALOG.md`

Historical evidence identifies the candidate but does not substitute for current code/test inspection.

## Priority And Current Disposition

- Priority axes: `1 / 3 / 1 / 2 / 3 / 1 / unknown`; known subtotal `11/18`.
- Final disposition at activation: `still_valid`, `correctness_correction (test)`, implementation ready.
- Current live-tree disposition: discovery pending until the post-initialization scoped search and baseline run.

## Current And Target Authority

- Current production authority: `core_enhanced/src/game/data/skill_data.rs` owns raw typed deserialization,
  `try_resolve_skill_range_source` and `validate_skill_contracts` rejection.
- Current test authority: the same file's `tests` module, specifically
  `skill_database_rejects_implicit_and_legacy_targeting_authority` and
  `skill_database_rejects_inline_target_range_and_preset_conflict`; current related constructor panic tests are
  inventoried separately and are not automatically in C08 scope.
- Target authority: minimal otherwise-valid fixtures plus typed/exact assertions that identify object/path/cause.
  Shared builders may own unrelated valid boilerplate only when they do not hide the tested defect.
- Consumers: the P09 negative publication/target tests and downstream CRA-P09-C01 implementation safety net.

## In Scope

- identify every current test case that is part of CRA-P09-C08, including the confirmed inline conflict case and
  related legacy-authority negatives described by the audit;
- capture target-file status, diff and content fingerprints before editing;
- repair missing unrelated required fields and isolate one intended defect per retained fixture;
- replace catch-any-panic/bare-error assertions with the strongest existing typed/path/cause assertion supported by
  the current API;
- add a repaired-control proof for each affected case and consolidate valid fixture boilerplate only where useful;
- run focused nonzero tests, the smallest affected package suite and the completion audit.

## Out Of Scope

- changing runtime validation, accepted targeting combinations, live RON schema/content or error contracts merely
  to make a test pass;
- implementing CRA-P09-C01 or any later P09 targeting/delivery candidate;
- introducing new public DTO/wire/save behavior, compatibility aliases, fallback parsing or test-only production
  branches;
- broad cleanup of unrelated `is_err`, panic or fixture patterns outside the evidence-linked P09 boundary;
- unrelated formatting or dirty-worktree cleanup.

## Preserved Invariants And Policy

1. Strict unknown/conflicting targeting rejection and the current live schema remain unchanged.
2. A valid control differs from its negative only by the named defect wherever the API permits this direct proof.
3. Failure evidence must name the intended object/path/cause; unrelated parse failure is not acceptance evidence.
4. Runtime code and production data are not weakened to accommodate tests.
5. Existing V3, RON, DTO/wire, public API and Boss-private contracts remain exact under CRA-POL-006.
6. Pre-existing and concurrent user changes are preserved without reset, restore, clean, stash or broad formatting.

Direct gameplay policy IDs: none. CRA-POL-006 applies globally as a no-migration/no-contract-drift guard. Open
policy decisions: none.

## Implementation And Dirty-Worktree Strategy

After these three Child documents exist, record the repository status and exact status/diff/content fingerprints for
every discovered target. Treat every pre-existing change as user-owned. Edit only assertion/fixture lines that can
be distinguished and safely preserved. If an affected target has overlapping edits whose intent cannot be separated,
stop before mutation and record the collision.

Use test-first proof in this order:

1. Establish the current focused runner and nonzero baseline.
2. Demonstrate that the audited assertion can accept the unrelated `delay_ms`/schema failure.
3. Repair the control fixture and prove the otherwise-valid form succeeds.
4. Restore only the named defect and assert its exact typed/path/cause failure.
5. Apply the same control/negative discipline to the remaining evidence-linked bare-error cases.
6. Re-run focused and affected broad validation, then inspect the scoped diff and legacy assertion search.

## Migration, Deployment And Compatibility

- DTO/wire: no shape, tag, payload, ordering or consumer change.
- Live RON/data: no schema or authored production-data migration; live data remains untouched.
- Save/checkpoint: no V3 or migration change.
- Public Rust/API: no public surface change.
- Deployment: test-only correction in the current workspace; no rollout or external consumer action.
- Temporary compatibility: none permitted or required.

If current evidence shows the correction requires any production contract change, stop as
`policy_decision_required`; do not broaden this Phase.

## Validation Plan

- Baseline: `cargo test -p game_core skill_database_rejects_inline_target_range_and_preset_conflict -- --nocapture`
  followed by the implicit/legacy test filter, each requiring a nonzero executed count.
- First proof: repaired fixture parses; the single restored defect fails with exact object/path/cause, never
  catch-any-panic.
- Focused regression: all affected negative target/publication tests with actual pass/fail/skip counts.
- Affected broad check: smallest owning package test suite and `cargo check` target justified by current metadata.
- Static completion checks: scoped search for remaining catch-any-panic/bare `is_err`, production-diff absence or
  explicit justification, target-file dirty-preservation comparison.
- Omitted by default: Unity, game_server, live WebSocket and performance validation because this Phase changes no
  runtime/data/wire behavior. Reconsider if the actual diff crosses that boundary.

## Completion Evidence Matrix

| ID | Condition | Required evidence | Status |
| --- | --- | --- | --- |
| P01-C01 | exact CRA-P09-C08 target set identified | `skill_data.rs` assertion inventory and E01 | pass |
| P01-C02 | pre-existing target changes preserved | E01 baseline hash/diff plus E04 signatures/status | pass |
| P01-C03 | every repaired control succeeds | 14 control/one-defect RON pairs; module 13/13 | pass |
| P01-C04 | every retained negative proves exact cause | exact RON strings/DataValidationError triples; module 13/13 | pass |
| P01-C05 | no contract/migration drift | Phase edits remain under `#[cfg(test)]`; no data/DTO/save/canonical edit | pass |
| P01-C06 | affected broad validation passes | RON 25/25, lib 930/930, package 979/979, all-target check | pass |
| P01-C07 | no scoped false-positive assertion remains | file-wide `catch_unwind`/bare `.is_err()` search clean | pass |
| P01-C08 | independent review is Complete / High | `COMPLETION_AUDIT.md` | pass |

## Stop Conditions

Stop this Phase and do not activate Phase 02 if the candidate no longer reproduces; the intended failure cannot be
distinguished from a different schema failure; current APIs cannot expose an exact cause without changing an
external/runtime contract; target edits overlap inseparably with user work; a new policy or migration is required;
the focused runner executes zero tests; or the completion review is not `Complete / High`.

사용자와 의논하여 정해야 할 정책이 있을 경우 Goal을 종료한다.
