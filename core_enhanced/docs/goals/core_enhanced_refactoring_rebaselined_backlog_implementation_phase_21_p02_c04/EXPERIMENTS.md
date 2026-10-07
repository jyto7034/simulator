# Phase 21 CRA-P02-C04 Experiments

Status: complete
Implementation Complete: true
Candidate: CRA-P02-C04
Master Phase: 21 of 44
Date: 2026-08-10 (`Asia/Seoul`)

## Experiment Protocol

- Execute only while Phase 21 is the sole active Master row.
- Preserve public/V3 progression fields and graph-derived admission under CRA-POL-006.
- Keep F-C01 before restore and Phase 20 conversion exact.
- Read all Child documents and the complete Phase 20 audit before production inspection.
- Record expanded dirty status and prospective target fingerprints before source/test/canonical inspection or edits.
- Add normal/BossOmen/admin/post-restore transition characterization before behavior-preserving changes.
- Complete only after an independent `Complete / High` phase-exit audit.

## 2026-08-10 — E01 Activation And Policy Boundary

### Inputs Reviewed Before Child Initialization

- Phase 20's final audit is `Complete / High`; Master changed Phase 21 from pending to ready only after that exit.
- F-C01 already owns strict graph-derived restored progression admission; Phase 20 now owns exact V3 conversion.
- CRA-POL-006 preserves public Rust/V3/map DTO fields and excludes migration/public narrowing.
- Original and Boss-era evidence classifies direct normal/Boss/admin mutation seams as encapsulation debt, not a
  current navigation defect.
- Final order requires a normal/BossOmen/admin transition table plus post-restore invariant.

### Activation Boundary

- Required Child documents were created and Master Phase 21 was registered as the sole active row.
- Phase 22 and later work remain pending and unauthorized.
- No Phase 21 production source, test, data or canonical was inspected or mutated during activation.

### Next Experiment

Fully read `PLAN.md`, this file and `EXPERIMENT_NOTES.md`, then fully reread the final 289-line Phase 20 audit.
Only afterward capture dirty baseline/fingerprints and inspect the current progression mutation graph.

## 2026-08-10 — E02 Mandatory Reads And Dirty Baseline

### Read Gate

- `PLAN.md` 180/180 lines, `EXPERIMENTS.md` 39/39 activation lines and `EXPERIMENT_NOTES.md` 38/38 activation
  lines were fully read after Master registered Phase 21 as the sole active Child.
- Phase 20's final `COMPLETION_AUDIT.md` was then reread in full, 289/289 lines. Its exact restored progression and
  F-C01 boundary is the required input, not a competing mutation authority.

### Activation Baseline

- HEAD is `f318414fb0156a67e04c5479000c2a2683fc0c09`.
- Expanded status contains 472 entries with normalized SHA-256
  `c6a90d5151d585c0e3ff4d1122679829eb439b6dfd7fcd926a0d5a25af04e0a0`.
- Prospective target hashes and dirty classifications are recorded in N02. Progression, node flow, combat tests,
  checkpoint tests and canonicals already overlap the dirty campaign; other listed paths are currently clean but
  remain fingerprinted before content inspection.
- No Phase 21 production/test/data/canonical content was read before this baseline was captured.

### Next Experiment

Read the complete current progression/map/BossOmen/admin/restore graph and canonical, build exact transition and
consumer tables and run focused baselines before adding characterization.

## 2026-08-10 — E03 Current Graph, Compatibility And Test-First Characterization

### Current Mutation Graph

- Normal entry/completion is already staged through `MapProgression::enter_node` and
  `complete_current_node`; graph-derived `selectable_node_ids` remains admission and projection authority.
- The only production field writes outside `progression.rs` are BossOmen exit availability in
  `boss_omen.rs` and two admin fixture paths in `world/admin/fixtures.rs`. The Boss path also reveals Gate node
  state/visibility; the admin paths also append or rewrite the paired node.
- F-C01 calls the existing `MapProgression::validate_checkpoint_state` after resolving and validating the authored
  facility and before Phase 20 restores the payload. No second invariant checker is needed.
- Runtime/server/Unity canonical inspection confirms public snapshots expose only graph-derived
  `map_navigation`; internal public/serde progression IDs remain required by strict V3 but are not external
  selection fields. No canonical wording change is warranted by a private owner transfer.

### Baseline And Characterization

- Pre-edit baselines were green with nonzero counts: progression 10, map flow 40, admin 16, BossOmen filter 11
  and checkpoint validation 6.
- Test-first assertions now freeze exact normal entry/completion IDs and node states, sorted graph equality,
  invalid unavailable/missing no-mutation, BossOmen completed-boss plus sole revealed Gate state, detached admin
  fixture state, authored SavePoint state and post-Continue V3 invariant/selectable equality.
- Focused characterization passed: progression 10, forced BossOmen 1, admin-enter 6, authored SavePoint 1 and
  checkpoint graph/Continue 1.
- A new invalid forced-terminal test then requested atomic missing-node rejection. Its first run failed at compile
  time exactly because `MapProgression::reveal_terminal_nodes` does not yet exist (`E0599`), establishing the
  planned private boundary before production implementation.

### Next Experiment

Implement the smallest crate-private progression methods for forced terminals, detached fixtures and authored
fixtures; transfer only the three direct production callers; then rerun focused parity and direct-write searches.

## 2026-08-10 — E04 Typed Paired Transition Boundary

### Implementation

- Added `MapProgression::reveal_terminal_nodes`. It selects the requested typed terminal resolution, preflights all
  referenced map nodes before mutation, makes each target `Available + Revealed`, and installs the same UUID-sorted
  availability vector. An empty resolution set remains `Ok(false)`; a missing target returns `MissingNode` without
  partial map/progression mutation.
- Added `activate_detached_fixture_node`. It owns the paired append plus `Unavailable + Revealed`, current and sole
  stored-availability update while retaining completed IDs exactly.
- Added `activate_authored_fixture_node`. It owns the paired authored-node `Unavailable + Revealed`, current,
  availability clear and completed-ID removal, with missing-node rejection before progression mutation.
- `complete_forced_boss_with_exit_gate` now delegates the precommitted exit reveal. Generic admin fixture creation
  and authored SavePoint entry delegate their paired changes. Their content/session/Boss lifecycle orchestration
  remains at the existing caller.
- Public/serde progression fields, normal entry/completion/selectability, F-C01 validation, Phase 20 conversion and
  every DTO/data/canonical remain unchanged.

### Immediate Result

- The test-first missing-terminal case changed from the expected `E0599` Red to 11/11 progression tests green.
- BossOmen, admin and checkpoint characterization remained green at 1, 6+1 and 1 cases.
- Production search finds no field write outside `progression.rs`; remaining outside references are reads. Whole
  progression replacement remains only generated-map construction.

## 2026-08-10 — E05 Focused, Broad And Compatibility Validation

### Focused

- `cargo fmt --all -- --check`: pass.
- progression 11/11; map flow 40/40; admin 16/16; BossOmen 11/11; checkpoint validation 6/6.
- Exact normal, forced exit, detached fixture, authored fixture and restored V3 assertions all pass.

### Broad

- Complete Core debug: 998 unit + 51 official integration tests passed.
- Complete Core release: the same 998 + 51 passed.
- Complete server debug/release: 59/59 in each profile.
- Core/server debug/release `cargo check`: all pass.
- Repository `git diff --check` and formatter check: pass.

### Static And Compatibility

- Public `MapProgression` still has exactly the same three public fields and derives unchanged serde.
- Production direct-write search returns only mutations inside `progression.rs`; BossOmen/admin callers contain no
  field writes. Canonical hashes remain exactly `6e20d6a6...`, `6c53ecd5...`, `4d734af0...`, `a3224e24...`.
- Non-strict all-target Clippy reports the established broad debt: Core lib 83 and lib-test 136 diagnostics, server
  lib 3. No diagnostic points at the Phase 21 production edits; strict global warning cleanup is out of scope.
- Pre-audit expanded status is 475 entries at SHA-256
  `30ebdf00d0be8dd3239bfc1a8f169ffe1be77b1e8d0c322a2ed13b2d24beaed3`. The increase from activation is exactly
  the three previously clean production/test files now legitimately modified by this Phase.

### Review Handoff

P21-C01–C09 claim complete. P21-C10 remains `review_pending` until the full completion guide and current Child are
independently reread and a `Complete / High` phase-exit audit is written.

## 2026-08-10 — E06 Independent Phase Exit

- The 567-line completion guide and current 180/153/157-line Child documents were fully reread before the audit.
- The audit independently reread policy, runtime, facility, Boss/admin, restore, snapshot, server/Unity, canonical,
  compatibility and dirty boundaries and reran 84 focused tests.
- `COMPLETION_AUDIT.md` returns `Complete / High`, improvement class `none`; P21-C01–C10 are complete.
- Audit-inclusive expanded status is 476 entries at normalized SHA-256
  `29e4646f9eaee517d07ac96593880c5b13776ab378e48653ad4ba2cfb1901523`.
- Phase 22 activation is allowed only after the final audit is fully read and the Master records Phase 21 complete.
