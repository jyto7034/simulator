# Server RunStart Fixture Contract Experiments

Status: complete

Implementation complete: true

Created: 2026-07-12

## 2026-07-12 — Goal startup and evidence intake

Hypothesis:

The six known `game_server` failures share one invalid server fixture and can be restored by aligning that fixture with the production Standard RunStart trigger invariant, without changing gameplay or transport policy.

Minimal experiment:

Read the Master Goal documents, common workflow/review documents, and both audit inputs before exploring code. Record the current worktree before any modification.

Changed files:

- Created this child goal's three documentation files only.

Commands and actual results:

- Read all of the Master `PLAN.md`, `EXPERIMENTS.md`, and `EXPERIMENT_NOTES.md`.
- Read `docs/README.md`, all 385 lines of `docs/engineering_workflow.md`, and all 177 lines of `docs/goal_completion_review_guide.md`.
- Read the complete `docs/archive/audits/documentation_code_alignment_audit_2026-07-10.md` historical audit input (then 921 lines, before its later promotion and archive headers were added).
- Read all 821 lines of the report now archived at `docs/archive/audits/comprehensive_project_review_2026-07-10.md`.
- `git status --short`: succeeded and showed a heavily dirty worktree, including user-owned changes in Core Visual Novel paths and `../game_server/src/game/player_game_actor/handlers.rs`.
- No tests were executed during document intake.

Actual test count:

- 0. This is a read-only startup record and is not counted as validation success.

Observed result:

- G04 is the only executable child and owns DCA-H-08/CPR-M-01.
- The historical baseline is 19 server tests passed and 6 failed before their intended assertions because the Standard RunStart trigger is absent from the fixture.
- Production validation must not be relaxed.
- The overlapping server fixture file already contains user-owned modifications and must be patched surgically.

Failure cause:

- Not yet independently reproduced in the current worktree. Audit evidence attributes it to `starter_policy_game_data_builder()` omitting the required trigger catalog.

Revert or correction:

- None.

Decision and next step:

Trace the current production and fixture paths, reproduce at least one focused failure, and determine the canonical reusable fixture seam before editing code.

## 2026-07-12 — Focused baseline reproduction

Hypothesis:

The current server fixture still reaches Core with an empty Visual Novel trigger database, so a Standard start fails before the server test's intended assertion.

Minimal experiment:

Run only `execute_behavior_returns_updated_state_snapshot`, first with an exact filter and then with a nonzero substring filter after checking the actual module-qualified test name.

Changed files:

- Goal documentation only; no production or test code changed before reproduction.

Commands and actual results:

- `cargo test -p game_server execute_behavior_returns_updated_state_snapshot -- --exact --nocapture --test-threads=1`
  - 0 tests executed, 25 filtered out in the library target.
  - Not accepted as validation evidence because `--exact` omitted the module path.
- `cargo test -p game_server execute_behavior_returns_updated_state_snapshot -- --nocapture --test-threads=1`
  - 1 test executed.
  - 0 passed, 1 failed, 24 filtered out.
  - Failure: `Invalid static data: missing RunStart visual novel trigger for Standard`.

Actual test count:

- First command: 0, invalid evidence.
- Corrected command: 1.

Observed result:

- `starter_policy_game_data_builder()` starts from `GameDataBuilder::empty()`, adds starter equipment/fragment data and two Visual Novel definitions, but does not replace the empty trigger database.
- Both `empty_game_data()` and defense-derived server fixtures consume this builder.
- Core already has a focused negative test proving a missing RunStart trigger is rejected at run start.
- Live RON has exactly one Standard and one Endless trigger, each `OncePerRun`, `PersistAcrossCheckpointLoad`, priority 100, and blocking.

Failure cause:

The server test fixture was updated for the new pre-run Visual Novel definition but not for its authoritative RunStart trigger.

Revert or correction:

The zero-test command is retained as failed validation evidence. The next patch will construct Visual Novel definitions and matching RunStart triggers together and inject both through the public `GameDataBuilder` APIs.

Decision and next step:

Patch only the server test fixture seam. Do not weaken runtime lookup and do not implement G12's supported-mode startup cardinality validation in G04.

## 2026-07-12 — Paired server RunStart fixture

Hypothesis:

If the server starter fixture constructs its two Visual Novel definitions and their mode-specific RunStart triggers as one pair, both the minimal actor test and live-defense-derived tests will reach their intended assertions without changing Core behavior.

Minimal experiment:

Surgically extend the existing user-modified server test module with trigger imports, one paired fixture function, and one typed trigger constructor. Re-run one minimal and one defense-derived test.

Changed files:

- `../game_server/src/game/player_game_actor/handlers.rs`
  - `starter_policy_game_data_builder()` now consumes one paired VN/trigger fixture.
  - Standard and Endless definitions and triggers share local ID values.
  - Both triggers use the live contract: `OncePerRun`, `PersistAcrossCheckpointLoad`, priority 100, blocking.

Commands and actual results:

- `cargo test -p game_server execute_behavior_returns_updated_state_snapshot -- --nocapture --test-threads=1`
  - 1 passed, 0 failed, 24 filtered out.
- `cargo test -p game_server paused_live_battle_tick_does_not_push_delta_until_resumed -- --nocapture --test-threads=1`
  - 1 passed, 0 failed, 24 filtered out.

Actual test count:

- 2 focused invocations, one nonzero test each.

Observed result:

- The previously reproduced minimal failure now reaches and passes its original command-result/snapshot assertions.
- The defense-derived pause/tick path also starts the run and reaches its original assertion.
- No production Core, live RON, DTO, or Unity code changed.

Failure cause:

- Resolved for both fixture paths: empty trigger data no longer accompanies the authored test VN definitions.

Revert or correction:

- None required after the focused tests. Broad formatting, check, and suite validation remain pending.

Decision and next step:

Run formatting validation, full server check/tests, relevant Core trigger/start tests, then audit Unity/Rust fixture entry points and perform the child completion review.

## 2026-07-12 — Formatting correction and broad baseline

Hypothesis:

The fixture-only correction should preserve the full Core baseline and restore all six blocked server tests without new failures.

Minimal experiment:

Run formatting validation, correct only the reported formatting, then execute server check/full tests, relevant Core focused tests, and the complete Core all-targets suite. Inspect Unity runner availability and a real Rust serialization path read-only.

Changed files:

- Formatting-only adjustment to the new imports and paired helper signature in `../game_server/src/game/player_game_actor/handlers.rs`.
- Goal documents updated with actual evidence.

Commands and actual results:

- First `cargo fmt --all -- --check`: failed with two rustfmt diffs in the new import wrapping and paired helper signature.
- Applied exactly the reported formatting.
- Second `cargo fmt --all -- --check`: passed.
- `cargo check -p game_server`: passed.
- `cargo test -p game_server --all-targets -- --test-threads=1`: 25 passed, 0 failed. All six audit-named tests appear in and pass this result.
- `cargo test -p game_core run_start_visual_novel -- --nocapture --test-threads=1`: 2 passed, 0 failed; 638 library tests filtered out. The two executed tests prove live trigger loading and missing-trigger rejection.
- `cargo test -p game_core start_new_game_opens_pre_run_hq_visual_novel_without_run_or_map -- --nocapture --test-threads=1`: 1 passed, 0 failed; 639 library tests filtered out.
- `cargo test -p game_core --all-targets -- --test-threads=1`: 681 passed, 0 failed across 640 library tests and 41 integration tests.
- `git diff --check -- ../game_server/src/game/player_game_actor/handlers.rs docs/goals/server_runstart_fixture_contract docs/goals/cross_layer_contract_hardening_master`: passed.
- `cmd.exe /c where Unity.exe`: failed to find Unity on PATH; this is recorded as unavailable PATH discovery, not as a Unity test failure.
- Read-only PowerShell discovery found `C:\Program Files\Unity\Hub\Editor\6000.3.4f1\Editor\Unity.exe`; `Test-Path` returned `True` and the project requires 6000.3.4f1.
- Unity manifest contains `com.unity.test-framework` 1.6.0; EditMode and PlayMode test assemblies are present.
- Unity runner was not executed because G04 changes no Unity source/contract and execution would mutate the external project's cache/log state.
- Read-only source inspection proved `player_state_snapshot_preserves_core_snapshot_root_fields` serializes the actual Rust `RunSnapshotDto` and server `PlayerStateSnapshotDto`; that test passed in the 25-test suite.

Actual test count:

- Server focused: 2 nonzero invocations, 2 passed.
- Server broad: 25 passed.
- Core focused: 3 passed.
- Core broad: 681 passed.
- Unity: not run; runner and assemblies identified.

Observed result:

- The historical 19-pass/6-fail server baseline is now 25-pass/0-fail.
- The complete Core baseline remains 681-pass/0-fail.
- Existing test assertions were reached and retained; production validation was not changed.
- No duplicate legacy server fixture function or compatibility path remains in the changed module.

Failure cause:

- The only post-implementation failure was formatting in newly added code; it was corrected without behavioral change.

Revert or correction:

- Applied rustfmt's exact import/signature layout. Re-ran the formatting gate successfully.

Decision and next step:

Move G04 to `review_required`, perform the guide-based child completion audit, and if it passes update the Master while preserving supported-mode startup cardinality as a G12 follow-up rather than falsely closing that broader CPR-M-01 aspect.

## Experiment Recording Template

### YYYY-MM-DD — Short title

Hypothesis:

Minimal experiment:

Changed files:

Commands and actual results:

Actual test count:

Observed result:

Failure cause:

Revert or correction:

Decision and next step:
