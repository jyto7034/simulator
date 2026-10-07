# Phase 20 CRA-P02-C03 Experiments

Status: complete
Implementation Complete: true
Candidate: CRA-P02-C03
Master Phase: 20 of 44
Date: 2026-08-10 (`Asia/Seoul`)

## Experiment Protocol

- Execute only while Phase 20 is the sole active Master row.
- Preserve strict current V3, all values/exclusions and public/save/wire behavior under CRA-POL-006.
- Keep F-C01 validation before every restore mutation; do not merge checkpoint and next-Floor lifetimes.
- Read all Child documents and the complete Phase 19 audit before production inspection.
- Record expanded dirty status and prospective target fingerprints before source/test/canonical inspection or edits.
- Add semantic golden and bidirectional characterization before behavior-preserving structural changes.
- Complete only after an independent `Complete / High` phase-exit audit.

## 2026-08-10 — E01 Activation And Policy Boundary

### Inputs Reviewed Before Child Initialization

- Phase 19's final audit is `Complete / High`; Master changed Phase 20 from pending to ready only after that exit.
- F-C01 is already `Complete / High` and owns strict current-document graph admission before restore.
- CRA-POL-006 preserves the current strict V3 field set, public Rust/save/wire surfaces and Boss/private exclusions;
  it authorizes no migration, repair, compatibility reader or public narrowing.
- Original and Boss-era re-audit evidence still classifies manual capture/restore copies as change amplification,
  not a current behavior defect. Next-Floor carry-forward remains deliberately separate.
- Final order requires golden exact V3 field sets/exclusions and bidirectional parity, including active-battle/Boss
  exclusion, after F-C01.

### Activation Boundary

- Required Child documents were created and Master Phase 20 was registered as the sole active row.
- Phase 21 and later work remain pending and unauthorized.
- No Phase 20 production source, test, serialized fixture or canonical was inspected or mutated during activation.

### Next Experiment

Fully read `PLAN.md`, this file and `EXPERIMENT_NOTES.md`, then fully reread the final 233-line Phase 19 audit.
Only afterward capture dirty baseline/fingerprints and inspect the current checkpoint conversion graph.

## 2026-08-10 — E02 Mandatory Reads And Dirty Baseline

### Read Gate

- `PLAN.md` 195/195 lines, `EXPERIMENTS.md` 41/41 activation lines and `EXPERIMENT_NOTES.md` 41/41 activation
  lines were fully read after Master registered Phase 20 as the sole active Child.
- Phase 19's final `COMPLETION_AUDIT.md` was then reread in full, 233/233 lines. Its `Complete / High` transition and
  private live-data seam do not conflict with checkpoint conversion.

### Activation Baseline

- HEAD is `f318414fb0156a67e04c5479000c2a2683fc0c09`.
- Expanded status contains 468 entries with normalized SHA-256
  `3571680ccb302552f804e18af9681ed9a3afa9d252d4d195b96746ea57e9d6dc`.
- Every prospective Core/server/test/canonical checkpoint target is already modified or untracked. Exact activation
  hashes and dirty classifications are recorded in N02; existing content and hunks remain user/campaign-owned until
  scoped attribution proves otherwise.
- No Phase 20 production/test/canonical content was read before this baseline was captured.

### Next Experiment

Read the complete current capture/export/validate/restore/Continue/host graph and canonical, build the exact
field/lifetime table and run valid/F-C01/server baselines before adding golden characterization.

## 2026-08-10 — E03 Graph Inventory And Pre-refactor Baseline

### Conversion And Ownership Result

- The Core V3 document is private and strict: top-level `version` plus a flat 21-field checkpoint.
- Capture and restore currently repeat the same thirteen checkpoint-owned `RunState` fields by hand. Six additional
  root fields and two root power fields have special restore semantics but belong to the same checkpoint document.
- F-C01 validation, explicit-load accounting, Continue bootstrap, host envelope/monotonic metadata, transient
  clearing and next-Floor carry-forward are distinct authorities and will not move into a generic conversion.
- The canonical contracts already require the current split and do not authorize a field, version, wire or public
  Rust change.

### Baseline Commands And Results

- `cargo test -p game_core game::world::tests::checkpoint_validation --lib -- --nocapture`: 4 passed.
- `cargo test -p game_core game::world::tests::facility --lib -- --nocapture`: 11 passed.
- `cargo test -p game_server active_run --lib -- --nocapture`: 16 passed.

### Test-first Gap

The current suite does not freeze the exact serialized key set and does not perform Continue restore followed by
fresh capture and semantic V3 comparison. Add those behavior-green characterizations before production refactoring.

## 2026-08-10 — E04 Test-first Exact V3 Characterization

### Added Before Production Refactor

- Exact top-level keys are frozen as `checkpoint` and `version`, with version exactly `3`.
- Exact flat checkpoint keys are frozen at 21, and each of the thirteen run-owned plus eight root-owned values is
  compared with its live capture source.
- Ten deliberately excluded root/lifecycle fields are asserted absent and individually rejected by strict V3 when
  injected: active battle/node content, recovery restriction, compendium, game state, load count, node session,
  player, run peak and active VN.
- A valid Continue now restores, verifies transient clearing, performs a fresh checkpoint capture and compares the
  entire semantic V3 document plus host monotonic metadata with the original.

### Result On The Manual Implementation

The first compile exposed only a test ownership-pattern mistake; borrowing the diagnostic string corrected it.
The full characterization family then passed 6/6 before production structure changed. This is the expected
behavior-green test-first result for a refactor of current correct behavior.

## 2026-08-10 — E05 Single Typed Conversion Authority

### Implementation

- `define_run_checkpoint_payload!` declares the thirteen checkpoint-owned `RunState` fields once and generates the
  unchanged flat `RunCheckpointPayload`, private `capture` and consuming `restore_into_state` conversion pair.
- The pair also keeps the existing eight root-owned values and special restore rules adjacent: power generation,
  checkpoint-scoped VN history, enkephalin/inventory/roster/fragments, VN-counter-preserving UUID restore and
  transient node/content/battle clearing.
- `save_run_checkpoint` now delegates to `RunCheckpointPayload::capture`; restore delegates to
  `restore_into_state` and retains the existing `ViewingMap` transition afterward.
- F-C01 validation, explicit-load count timing, Continue bootstrap, export wrapper, host persistence and next-Floor
  carry-forward were not moved or changed.

### Structural Result

Static search finds one payload declaration/implementation, one capture caller and one restore caller. The two
separate manual thirteen-field mappings are gone; no alternate constructor, compatibility reader or fallback was
introduced.

## 2026-08-10 — E06 Final Validation And Review Handoff

### Focused And Broad Results

- Checkpoint validation: 6/6 debug and 6/6 release.
- Facility/SavePoint: 11/11 debug; snapshot/start: 23/23 debug; server active-run family: 16/16 debug.
- Complete Core: 997 unit plus 51 integration tests in debug and again in release.
- Complete server: 59/59 in debug and 59/59 in release.
- Core/server checks, repository formatter check and scoped diff check pass.

### Diagnostic Qualification

An initial concurrent debug/release Core run collided on a shared debug combat-trace file and one process observed
an incomplete JSON write after 996 passes. The debug process completed 997/997 and a later isolated release run
completed 997/997 plus all 51 integrations, proving an orchestration collision rather than a product regression.

Strict global Clippy remains blocked by existing repository warning debt (83 library and 136 library-test promoted
diagnostics). Scoped non-strict output contains no diagnostic in the Phase 20 conversion or new test lines. Server
non-strict output reports three pre-existing diagnostics in unchanged active-run/handler regions.

### Handoff

P20-C01–C09 are evidenced complete. No canonical edit is necessary because existing runtime/rulebook/Unity
canonicals already state the preserved ownership and lifetime split. P20-C10 awaits the independent phase-exit
audit; Phase 21 remains pending and no Phase 21 production was inspected or changed.

## 2026-08-10 — E07 Independent Phase-exit Result

- `COMPLETION_AUDIT.md` was created under the 567-line completion guide and then read in full, 289/289 lines.
- All ten audit rows pass. Primary verdict is `Complete`, long-term fit is `High`, improvement class is `none`.
- Final audit SHA-256 is `49537a48630aedbe272724e8d04b1b4bbe646c1083746c5ec6b0c3ddb2f8dfc2`.
- Audit-inclusive expanded status is 469 entries at normalized SHA-256
  `bdb00dad13a14aab20434ff5da7d3ce254ef059a5b985bf1462c201d0c5df478`.
- Phase 20 transitions to complete. Phase 21 may become ready only through the Master transition and remains
  production-inactive until its own required documents are initialized, registered active and fully read.
