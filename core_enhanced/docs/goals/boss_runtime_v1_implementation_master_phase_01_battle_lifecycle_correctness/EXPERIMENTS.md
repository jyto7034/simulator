# Boss Runtime v1 Phase 01 — Experiments

## 2026-08-09 — Activation baseline

### Exact commands

```text
git status --short --untracked-files=all | tee /tmp/boss_p01_precreation_status.txt | sha256sum
wc -l /tmp/boss_p01_precreation_status.txt
sha256sum <seven P01 source/test owners>
git status --short -- <seven P01 source/test owners>
```

### Actual result

- Pre-creation status: 229 entries, SHA-256
  `42952ec0f51504b5462f824c02b1e4c1551d1d13a0c9d5c12f6e93c113d0d108`.
- Current owner hashes:
  - `sim.rs`: `c009f02bdac21366b3d27f0ee73b95d0efd72c81fa572861a67d07af4fd4c998`
  - `battle/core/mod.rs`: `5508d5adc96258647e2efa9ab7e7d8dfb4ea050d12ee09a5db4214f462a18476`
  - `battle/enums.rs`: `479c029a4538012c7c29b9412c2e098539c08951a60994a98e8bca3bbe42a0fc`
  - `world/combat.rs`: `18bc0e91df1e82f95dfaf15de37bcc8b89dd5b23e7361754119252e5a9c68526`
  - `world/state.rs`: `61b90446bf05da979ab981c101ea07500cf6b342b653278aecf3f24394acf094`
  - `world/visual_novel_triggers.rs`: `52a24ae63cb7d9d43182d3f2c294d35ac438375ad363845e3c60a2552c2c7819`
  - `world/tests/combat.rs`: `decf647987aa18d69766069af74e8c955659febeb10d5f7adaaf7fafceeba4c5`
- All owners except `visual_novel_triggers.rs` were already modified before P01. These are preserved baseline changes,
  not P01 attribution.
- No production mutation or test runner occurred in this activation unit.

### Decision

Activate one P01 child. Add failing M1 tests before changing terminal production symbols.

## 2026-08-09 — M1 failing characterization

### Mutation

Added three tests beside the existing max-time test in `battle/core/mod.rs`:

- `direct_until_and_by_steps_share_terminal_outcome_at_runtime_cap`
- `exact_cap_satisfied_condition_wins_and_unsatisfied_condition_draws`
- `terminal_state_clamps_clock_skips_late_event_and_finalizes_once`

No production symbol changed before the failing runs.

### Exact failing commands and results

```text
cargo test -p game_core --lib game::battle::core::tests::direct_until_and_by_steps_share_terminal_outcome_at_runtime_cap -- --exact --nocapture
```

Exit `101`: `0 passed`, `1 failed`, `0 ignored`, `864 filtered out`. Direct stepping returned terminal time 10 but
left `execution.last_event_time_ms()` at the late event time 100 instead of the cap 10.

```text
cargo test -p game_core --lib game::battle::core::tests::exact_cap_satisfied_condition_wins_and_unsatisfied_condition_draws -- --exact --nocapture
```

Exit `101`: `0 passed`, `1 failed`, `0 ignored`, `864 filtered out`. Direct stepping returned Draw even when
`SurviveUntil(10)` was satisfied exactly at the cap and the expected winner was Player.

### Decision

The tests reproduce both divergent clock and winner authority. Implement a state-owned terminal outcome and one cap
helper shared by direct/until/by before adding the world acceptance test.

## 2026-08-09 — M1 first implementation compile correction

- The first focused rerun after introducing `finish_at_time_limit` failed to compile (`E0596`):
  `compute_winner` requires mutable Core access while the new helper took `&self`.
- No test ran in that attempt. The helper receiver was corrected to `&mut self`; no behavior or scope decision changed.

## 2026-08-09 — M1 world acceptance first compile correction

- Added `live_world_tick_reaches_combat_result_at_runtime_cap_without_revision_churn` to exercise the actual
  server-used tick adapter through terminal projection and state transition.
- Its first compile attempt failed with `E0594`: `GameDataBase::run_policy` is itself an `Arc`, so mutating through
  only the outer uniquely owned `Arc<GameDataBase>` was insufficient.
- No test ran. The fixture now obtains unique mutable access to both the outer database and inner run-policy Arc
  before `GameCore` construction. Production behavior was not changed by this correction.

## 2026-08-09 — M1 implementation and focused closure

### Production change

- Replaced `BattleExecutionState.finished: bool` with `terminal_outcome: Option<BattleFinishSignal>` and exposed a
  read-only accessor.
- `finish_signal` now stores the terminal time/winner and clamps the authoritative clock.
- Direct stepping checks the cap before adopting a queued event time; direct and until/by paths share
  `finish_at_time_limit`, which evaluates the win condition at the cap before Draw fallback.
- Reaching the cap after draining its same-time bucket now terminates rather than returning repeated `Running`.
- `finalize_battle_execution` consumes only the state-owned outcome and no longer accepts caller finish data. All
  internal Core/world callers were migrated.

### Exact focused results

- Three new Core terminal tests: `3 passed`, `0 failed`, `0 ignored` across separate exact runs.
- New world server-tick cap test: `1 passed`, `0 failed`, `0 ignored`, `865 filtered out`.
- Four retained exact regressions (run-battle cap, full-vs-step parity, same-time attack/end, by-step no future jump):
  `4 passed`, `0 failed`, `0 ignored` across separate exact runs.

### Decision

M1 behavior is locally closed. Broad Core regression waits until M2 comparator work so the shared scheduler surface is
validated once. Proceed to failing all-variant/order tests without changing existing priority bands.

## 2026-08-09 — M2 failing totality characterization

### Mutation

Added a 19-pair fixture covering every current `BattleEvent` variant. Each pair is unequal while holding time and the
old primary tie fields constant wherever possible, exposing omitted cause/payload fields. The test asserts
`cmp == Equal iff Eq`, antisymmetry and reflexivity.

### Exact command and result

```text
cargo test -p game_core --lib game::battle::enums::tests::battle_event_cmp_equal_iff_eq_for_every_live_variant -- --exact --nocapture
```

Exit `101`: `0 passed`, `1 failed`, `0 ignored`, `866 filtered out`. Pair 0 reproduced the defect immediately:
two `SpawnGroup` values differing only in root cause compared `Equal`.

### Decision

Implement an explicit variant discriminator and collision-free complete-payload comparison while retaining current
time/priority semantics. Re-run the full 19-pair matrix before adding permutation cases.

## 2026-08-09 — M2 implementation and focused closure

- Added an explicit 19-value variant discriminator after the preserved time/priority bands. AutoCastEnd ranks before
  ManualCastEnd in priority band 8.
- Retained the established per-variant primary identifiers and added ManualCastEnd's missing caster key.
- Equal values return `Equal` immediately. Remaining unequal ties use the full derived payload representation, not a
  hash; equality-equivalent floats are normalized by the equality guard and NaN remains invalid by the existing
  event invariant.
- Added 720 insertion-permutation drains, explicit auto/manual and manual/manual cast-end order checks, and a 19×19
  cross-variant equality/antisymmetry matrix.
- Focused enum module result: `5 passed`, `0 failed`, `0 ignored`, `865 filtered out`. The retained same-time
  AttackResolve/BattleEnd exact regression also passed once.

## 2026-08-09 — M3 source query correction

- One read-only `rg` query embedded a literal newline without multiline mode and failed before matching. The corrected
  single-line alternative resolved `GameState`, `ActiveNodeContent` and tactical-focus owners. No mutation or test
  result relies on the failed query.

## 2026-08-09 — M3 failing future-cursor characterization

### Test support and mutation

- Added Debug/Clone derives to active attempt/deployment/resource structs needed for exact test snapshots and later
  scoped staging.
- Added a test-only total-order-normalized BattleCore event-queue snapshot.
- Added `mutating_advance_rejects_future_cursor_without_any_state_change`, comparing clock, terminal/finalized state,
  log, queue, revision, employee/deployment/tactical state, playback/focus, VN/consumed state, UUID continuation,
  GameState/content and active-battle presence.

### Exact command and result

```text
cargo test -p game_core --lib game::world::tests::combat::mutating_advance_rejects_future_cursor_without_any_state_change -- --exact --nocapture
```

Exit `101`: `0 passed`, `1 failed`, `0 ignored`, `870 filtered out`. The rejected call moved battle time from 0 to 10
and revision from 1 to 2 before `InvalidBattleResyncSeq`; other captured state stayed equal in this fixture.

### Decision

First move cursor and revision prevalidation ahead of all advance mutation. Then add late VN/reconcile failures to
prove the staged transaction rather than mistaking early validation alone for C03 closure.

## 2026-08-09 — M3 invalid-trigger fixture correction

- The first VN rollback test attempted to construct missing-VN trigger data through `GameDataBuilder`. The builder's
  strict publication validation correctly panicked on the missing reference before `GameCore`/advance existed.
- This is a fixture-construction failure, not rollback evidence: `0 passed`, `1 failed`, `0 ignored`, `873 filtered
  out` in that run.
- The test now loads a valid published database and, while its outer/inner Arcs are still uniquely owned before
  `GameCore` construction, injects the invalid runtime fixture directly. This deliberately bypasses publication only
  in the private unit test and exercises the real late trigger lookup error without a production failpoint.

## 2026-08-09 — M3 late-failure characterization

### VN trigger failure

The corrected exact VN test exited `101`: `0 passed`, `1 failed`, `0 ignored`, `873 filtered out`. The invalid trigger
returned `InvalidStaticData` as intended, but battle time changed from 0 to 10 before the error. Revision and the other
captured world/VN fields did not change in this fixture.

### Employee reconcile failure

```text
cargo test -p game_core --lib game::world::tests::combat::advance_rolls_back_core_and_world_when_employee_reconcile_fails -- --exact --nocapture
```

Exit `101`: `0 passed`, `1 failed`, `0 ignored`, `873 filtered out`. A real deployed employee was marked dead and
removed only from the attempt roster. Advance returned `UnitNotFound`, but time moved 0→50, the movement queue changed,
and a new movement event was appended to the log before reconciliation failed.

### Decision

Early cursor/revision checks are necessary but insufficient. Implement a scoped staged clone of ActiveBattleSession
plus VN trigger state/UUID continuation; publish it only after trigger, reconcile and DTO construction all succeed.

## 2026-08-09 — M3 staged-clone compile probe

- Added Clone only to the exact staged authority chain: battlefield, movement backend/Rapier world, BattleCore,
  RuntimeUnit and ActiveBattleSession (attempt/deployment/resource support was already added).
- The first `cargo check -p game_core --lib` failed with one missing bound: `RuntimeUnit: Clone`. No other BattleCore,
  Rapier or active-session field failed the derive in that compile.
- Added `#[derive(Clone)]` to `RuntimeUnit`; this is required for exact staged Core state and does not create a public
  wire/data contract.

## 2026-08-09 — M3 staged transaction implementation

### Production change

- Exact Clone now spans BattleCore, RuntimeUnit, BattlefieldLayout, the active Rapier/direct movement backend and
  ActiveBattleSession. The successful compile probe proves every owned runtime field is cloneable.
- `with_staged_battle_advance_state` stages active battle, VN active/queue, trigger-consumed sets, UUID manager,
  GameState and active node content. Any returned error restores the original authorities; success drops the
  originals and keeps the stage.
- One `advance_active_battle_staged` now owns elapsed/by, pause-aware server tick and next-event-bucket orchestration.
  It performs simulation-delta/remainder, revision/cursor validation, Core step/finalize, triggers, reconcile,
  projection, VN attachment and optional result transition inside the stage.
- Cursor validation was extracted as `ensure_battle_update_cursor_valid` and is reused by DTO construction.
- The prior two duplicated advance pipelines were removed. No public DTO/data/save shape changed.

### Strengthened tests

- Atomic snapshot now includes the full internal Core unit/tactical/item/buff/cast/proc/projectile/area/movement/
  field-work/block/counter state in addition to queue/log and all world/VN/revision fields.
- VN failure now runs through the actual server-tick adapter and first activates a valid higher-priority trigger,
  proving partial VN/UUID/consumed changes also roll back when the following missing-VN trigger fails.
- Added finishing future-cursor preservation and elapsed-vs-bucket exact success projection/log/queue/revision parity.

### Actual validation

- `cargo check -p game_core --lib`: pass after the one recorded RuntimeUnit Clone correction.
- Seven M3 exact tests (future cursor, finishing cursor, elapsed+bucket revision overflow, VN rollback, reconcile
  rollback, elapsed/bucket parity and server terminal path) pass; each exact run executed one test.
- `game::world::tests::combat`: `81 passed`, `0 failed`, `0 ignored`, `795 filtered out`.
- `game::battle::core::tests`: `100 passed`, `0 failed`, `0 ignored`, `776 filtered out`.
- `game::battle::enums::tests`: `5 passed`, `0 failed`, `0 ignored`, `865 filtered out` before the final cross-matrix
  strengthening; the exact strengthened totality test also passed afterward.

### Decision

All three P01 milestone behaviors are implemented and focused suites are green. Update the canonical to the actual
terminal/order/transaction contract, then run formatter, full Core, server and workspace gates.

## 2026-08-09 — First formatter check

`cargo fmt --all -- --check` exited `1` and reported only four P01-owned formatting differences in
`battle/core/mod.rs`, `battle/enums.rs` and `world/combat.rs`. No formatter write was run across the dirty tree; the
four exact hunks were corrected with scoped patches before rerunning the check.

## 2026-08-09 — Broad validation output handling

- The first `cargo test -p game_core --lib` output exceeded the direct tool-output allowance while the process was
  still running. A subsequent authoritative session poll returned the completed result: `876 passed`, `0 failed`,
  `0 ignored`, `0 filtered out`.
- `cargo test -p game_server` completed successfully: its library target reported `56 passed`, `0 failed`,
  `0 ignored`; its binary and doc-test targets each reported zero tests and success.
- The first `cargo test --workspace --all-targets` output exceeded the available model context during continuation.
  That truncation is not treated as a pass or failure. The process result must be recovered from its live session or
  the gate rerun with quieter output before P01 can close.
- The original workspace session was still available. Its authoritative final poll returned exit `0` without a
  rerun: Core library `876 passed`; live integration targets `3 + 4 + 24 + 13 + 4 passed`; server library
  `56 passed`; all listed binary/example targets completed successfully with zero tests. Therefore the required
  `cargo test --workspace --all-targets` gate is green.

## 2026-08-09 — Completion pre-audit

- The final current-tree enum module rerun reports `5 passed`, `0 failed`, `0 ignored`, `871 filtered out`; this
  supersedes the earlier pre-strengthening filtered count.
- `cargo fmt --all -- --check` is green after the four recorded scoped formatting corrections. `git diff --check`
  across all P01 production/test/canonical/Goal owners is also green.
- The current DTO projection has only two fallible preconditions: cursor validity and revision increment capacity.
  Both are prevalidated and have exact no-mutation coverage for elapsed/bucket/finishing cases. Real failures after
  Core mutation are covered separately at VN trigger lookup and employee reconciliation, proving the staged rollback
  rather than only early rejection.
- Workspace tests created untracked `game_server/logs/app.log.2026-08-09` (20,844 bytes) after the activation
  baseline. It was removed as this run's generated log; it was not present in or owned by the user baseline.
- Comparing current status with `/tmp/boss_p01_precreation_status.txt` leaves exactly the three child documents plus
  two P01-owned newly dirty Clone-chain files: `battle/battlefield/mod.rs` and
  `battle/core/movement/rapier_backend.rs`. After excluding those P01-owned additions, the status is exactly the
  original 229 entries with SHA-256
  `42952ec0f51504b5462f824c02b1e4c1551d1d13a0c9d5c12f6e93c113d0d108`.
