# Server RunStart Fixture Contract Goal

Status: complete

Implementation complete: true

Created: 2026-07-12

Master goal: `docs/goals/cross_layer_contract_hardening_master/PLAN.md`

Master child: G04

Audit findings:

- DCA-H-08
- CPR-M-01 (alias of the same fixture baseline failure)

## Objective

Restore a trustworthy `game_server` regression baseline by making the server test fixture satisfy the same mandatory Standard RunStart Visual Novel trigger contract as production data, without weakening production validation or the six affected regression assertions.

## User-Visible Problem

Six server tests currently stop during run initialization because their static-data fixture omits the required Standard RunStart Visual Novel trigger. Reconnect, tick, resync, and message-order assertions therefore never execute, so later cross-layer work cannot rely on the server test gate.

## Goal Working Method

Before code exploration, modification, or validation, fully read:

1. this goal's `PLAN.md`, `EXPERIMENTS.md`, and `EXPERIMENT_NOTES.md`,
2. the three Master Goal documents,
3. `docs/README.md`, `docs/engineering_workflow.md`, and `docs/goal_completion_review_guide.md`,
4. DCA-H-08 and CPR-M-01 in the two audit inputs.

Meaningful investigation, changes, failures, and decisions must update all three child documents. Goal documents control scope and accepted policy, but runtime code, production live data, typed DTOs, server mapping, Unity consumers, and canonical documents remain the evidence hierarchy.

## Current Evidence

Master-creation baseline on 2026-07-11:

- `cargo fmt --all -- --check`: passed.
- Core `cargo test --all-targets`: 681 passed, 0 failed.
- `game_server` `cargo test --all-targets`: 19 passed, 6 failed.
- `cargo check -p game_server`: passed.
- Unity EditMode/PlayMode runner: not run.

All six server failures reported `Invalid static data: missing RunStart visual novel trigger for Standard` before reaching their intended assertions. The audit identifies `starter_policy_game_data_builder()` in `../game_server/src/game/player_game_actor/handlers.rs` as the fixture source. Current worktree state is heavily dirty, including user-owned changes in that file and the newly added Visual Novel runtime/data paths; no existing change may be reverted or overwritten.

## Source Of Truth And Ownership

- Production RunStart requirement and state transition: Core runtime and Visual Novel trigger validation/start flow.
- Production authored example: live Visual Novel and trigger RON loaded by the production loader.
- Fixture ownership: `game_server` test support code that constructs `GameDataBase` for actor tests.
- Regression ownership: the six server tests for attach/recovery, behavior snapshot, battle tick/update, resync, and final ordering.
- This child does not redefine Visual Novel gameplay policy or wire shape.

## Invariants

- Standard run initialization requires the accepted RunStart Visual Novel trigger contract; tests must not bypass or weaken it.
- Server fixtures that claim to represent a startable Standard game must be valid under the same Core validation path as production data.
- One canonical fixture construction path should own the minimal valid RunStart Visual Novel definition and trigger; do not duplicate ad-hoc trigger literals across six tests.
- Existing reconnect, tick, resync, and ordering assertions must execute and retain their original behavior intent.
- A filtered command that executes zero tests is not evidence.
- Existing dirty-worktree changes are user-owned and must be preserved.

## In Scope

- Trace the exact production RunStart requirement and fixture construction path.
- Establish the smallest complete reusable server test-data builder contract needed for a startable Standard game.
- Correct the fixture so all six affected tests reach their intended assertions.
- Add or strengthen a focused fixture/startup regression test if the existing six tests do not directly prove the invariant.
- Record exact test counts, current Core/server baseline, available Unity test entry points, and at least one actual Rust-serialized fixture path for later phases.
- Update the nearest durable workflow/data-loading documentation only if the implementation exposes a real missing contract; do not restate historical audit prose as policy.

## Explicitly Out Of Scope

- Battle revision, capability, reconnect semantics, or Unity behavior changes owned by G01/G06/G07/G08.
- New Visual Novel gameplay policy, trigger schema changes, or production content changes.
- Relaxing RunStart trigger cardinality or accepting missing triggers.
- Fixing unrelated dirty-worktree changes or other audit findings.
- Starting Phase 1 implementation.

## Implementation Plan

1. Record the current dirty worktree and reproduce the focused fixture failure with exact test counts.
2. Trace command entry through Core run start, trigger lookup/validation, production live RON, server fixture construction, and affected tests.
3. Identify whether an existing Core test-data builder can be reused without creating a cross-package ownership inversion.
4. Implement one long-term fixture construction seam that supplies a valid Standard RunStart Visual Novel definition and trigger.
5. Run the smallest focused server test first, then all six affected tests, `cargo check`, full server tests, and the relevant Core validation/start tests.
6. Inspect available Unity runners and locate at least one real Rust serialization fixture/generation path without mutating the Unity project.
7. Perform the child completion review using `docs/goal_completion_review_guide.md`; only then update the Master progress table and start R00.

## User-Visible Completion Conditions

- A startable Standard server fixture passes the same mandatory RunStart trigger validation as production data.
- The six previously blocked tests reach and pass their intended reconnect/tick/resync/order assertions.
- No validation, assertion, or production invariant was weakened.
- The fixture contract has one clear owner and no compatibility alias, fallback, or duplicate authority.
- Exact focused and broad test counts are recorded and nonzero.
- Core and server baselines are reproducible and pre-existing failures are distinguished from new failures.
- Available Unity fixture/test entry points and at least one real Rust-serialized fixture path are recorded for later phases.
- A completion review gives `Complete` with High long-term fit and no immediate correction.

## DTO, Data, And Migration

- Unity-facing DTO migration: none expected.
- Live RON schema or content migration: none expected.
- Save-data migration: none.
- Compatibility layer: not authorized.

If investigation shows any of these are required, stop and set `Status: policy_decision_required` before implementing them.

## Planned Verification

Exact test names and package working directories must be confirmed from current source before execution.

- Focused server fixture/run-start test.
- Each of the six previously failing server tests, with nonzero test counts.
- Full `game_server` test suite.
- `cargo check -p game_server` from the workspace that owns the package.
- Relevant Core Visual Novel trigger validation and Standard run-start focused tests.
- Relevant Core broad tests only if the fixture change touches shared Core test builders.
- Read-only inspection of Unity test runner entry points and real Rust fixture serialization paths.

## Current Validation Evidence

- Focused server minimal fixture: 1 passed.
- Focused server defense-derived fixture: 1 passed.
- Full `game_server --all-targets`: 25 passed, 0 failed.
- `cargo check -p game_server`: passed.
- Relevant Core trigger/start focused tests: 3 passed, 0 failed.
- Full `game_core --all-targets`: 681 passed, 0 failed.
- `cargo fmt --all -- --check`: passed after one recorded formatting correction.
- `git diff --check` for G04/master documents and the server fixture file: passed.
- Unity test framework 1.6.0, Editor 6000.3.4f1, `ArkClient.EditModeTests`, and `ArkClient.PlayModeTests` are present. Unity was not executed because G04 has no Unity change and batch execution mutates the external project cache/log surface.
- `player_state_snapshot_preserves_core_snapshot_root_fields` is a real Rust typed-fixture path: it obtains `RunSnapshotDto`, serializes it, wraps it in `PlayerStateSnapshotDto`, serializes again, and passed in the 25-test server suite.

The child completion review passed on 2026-07-12 with verdict `Complete` and High long-term fit. Phase 0 still requires the separate R00 review.

## Stop Conditions

Stop with `Status: policy_decision_required` and report choices if the correct fix requires changing:

- RunStart trigger cardinality or gameplay semantics,
- live RON schema/content,
- Unity-facing DTO shape,
- production loader behavior,
- save migration or compatibility support.

An unexpected technical detail within the accepted fixture boundary is recorded in `EXPERIMENT_NOTES.md` and resolved without asking for a new gameplay policy. Out-of-scope improvements are recorded and skipped.

## Completion Review

Do not mark this child complete from green tests alone. Apply `docs/goal_completion_review_guide.md`, audit old fixture paths and bypasses, classify long-term fit, record remaining risk, and then update the Master Goal. Phase 0 still requires the separate R00 review after this child passes.
