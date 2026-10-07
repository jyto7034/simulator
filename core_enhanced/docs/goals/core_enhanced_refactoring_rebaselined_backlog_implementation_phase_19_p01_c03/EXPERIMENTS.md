# Phase 19 CRA-P01-C03 Experiments

Status: complete
Implementation Complete: true
Candidate: CRA-P01-C03
Master Phase: 19 of 44
Date: 2026-08-10 (`Asia/Seoul`)

## Experiment Protocol

- Execute only while Phase 19 is the sole active Master row.
- Preserve public construction APIs and all externally observable publication behavior under CRA-POL-006.
- Read all Child documents and the complete Phase 18 audit before production inspection.
- Record expanded dirty status and prospective target fingerprints before source/test/data inspection or edits.
- Add exact passing characterization before behavior-preserving structural changes.
- Never merge production and fixture lifecycles or use a public-API negative search as removal authority.
- Complete only after an independent `Complete / High` phase-exit audit.

## 2026-08-10 — E01 Activation And Policy Boundary

### Inputs Reviewed Before Child Initialization

- Phase 18 final result is `Complete / High`; Master changed Phase 19 from pending to ready only after that exit.
- CRA-POL-006 preserves public Rust, RON, save, DTO/wire and external-consumer contracts unless separately migrated.
- Original P01-C03 evidence classifies repeated private production mapping as change amplification, not a current
  runtime defect, and explicitly preserves `GameDataBaseParts`/builder.
- Re-audit confirms one new concrete instance: Boss compilation is performed during validation and repeated for
  publication. It remains inside P01-C03 rather than becoming a separate candidate.
- Final order places P01-C03 first in behavior-preserving Stage C after publication semantics stabilized. Required
  proof is exact parsed/validated/compiled→published field/Arc/registry/failure-order parity and compile-once Boss
  publication without public API narrowing.

### Activation Boundary

- Required Child documents were created and Master Phase 19 was registered as the sole active row.
- Phase 20 and later work remain pending and unauthorized.
- No Phase 19 production source, test, live data or canonical was inspected or mutated during activation.

### Next Experiment

Fully read `PLAN.md`, this file and `EXPERIMENT_NOTES.md`, then fully reread the final 254-line Phase 18 audit.
Only afterward capture the dirty baseline/fingerprints and inspect the current production publication graph.

## 2026-08-10 — E02 Mandatory Reads And Dirty Baseline

### Read Gate

- `PLAN.md` 196/196 lines, `EXPERIMENTS.md` 42/42 activation lines and `EXPERIMENT_NOTES.md` 41/41 activation
  lines were fully read after Master registered Phase 19 as the sole active Child.
- Phase 18's final `COMPLETION_AUDIT.md` was then reread in full, 254/254 lines. Its `Complete / High` transition and
  Phase 19 activation boundary are consistent with the Master.

### Activation Baseline

- HEAD is `f318414fb0156a67e04c5479000c2a2683fc0c09`.
- Expanded status contains 464 entries with normalized SHA-256
  `d43ed3c438fadacb896267830322e1f465b320e7486fdf80ed113d5f088e4bd8`.
- All likely production/test/canonical targets except current live catalog audit and bundle manifest are already
  modified or untracked. Their activation hashes and dirty classification are recorded in N02; existing content and
  hunks remain user/campaign-owned until scoped attribution proves otherwise.
- No production content was read before this baseline was captured.

### Next Experiment

Read the complete current publication source/test/canonical/data graph, build the exact lifecycle table and run
behavior-preserving baselines before adding characterization tests.

## 2026-08-10 — E03 Graph Characterization And Structural Red

### Current Lifecycle

- The official path is `GameplayRonSourceSet` → `parse_gameplay_ron_sources` → `ParsedGameplayRon` →
  `validate_domain_catalogs` → public `GameDataBaseParts` assembly → private `from_validated_parts` → generated
  combat-preview validation → `Arc<GameDataBase>` publication.
- `ParsedGameplayRon` carries 18 non-map catalogs plus a five-Arc map aggregate. Validation compiles Boss Runtime
  once in its existing fail-fast position so PvE references can be checked, but discards that value. Publication
  then compiles the same Boss source a second time and maps all fields through public fixture parts.
- Public `GameDataBaseParts`, `GameDataBase::new` and `GameDataBuilder` form a distinct programmatic/fixture seam.
  `GameDataBuilder` appears in 33 Core/server Rust files; the server executable's production bootstrap calls
  `load_live_embedded`. These public seams are compatibility constraints, not consolidation targets.

### Test-First Evidence

- Existing baselines were green: `ron_loading` 25/25 and the `game::data::` slice 189/189.
- Added exhaustive publication parity for all 18 catalogs, five map catalogs and every consumable/equipment registry
  entry; it passes. The initial RON-string comparison exposed nondeterministic `HashMap` serialization order, so the
  assertion was corrected to semantic `serde_json::Value` equality before judging behavior.
- Added public-parts Arc/registry identity coverage; it passes and locks all 18 catalog Arcs, five map Arcs and
  registry references to the exact supplied allocation.
- Added thread-local test instrumentation around Boss compilation and an official-loader assertion. It fails with
  observed delta `2` versus required `1`, providing the intended structural red without changing runtime semantics.

### Authorized Implementation

Introduce one private validated live-catalog state that retains the Boss compiled at its existing validation point,
then consume that state directly into the private production candidate. Keep validation order, public construction
APIs, generated-preview gate, item registry semantics and every published value/Arc contract unchanged.

## 2026-08-10 — E04 Private Validated Conversion And Focused Green

### Implementation

- Added crate-private `ValidatedGameplayRon`, which owns the parsed aggregate and the `BossRuntimeDatabase` produced
  at the existing validation position.
- Split the internal validator so test characterization can still request validation-only behavior while the
  official consuming path returns the compiled Boss only after every existing domain/cross/map gate succeeds.
- Replaced production's public-parts round trip with `from_validated_live_catalogs`. It consumes each parsed catalog
  into its final Arc once, installs the retained compiled Boss, derives `ItemRegistry` from the final item Arcs and
  leaves generated-preview validation immediately before successful publication.
- Removed the second production Boss compile and obsolete private `from_validated_parts`; public parts, builder and
  `GameDataBase::new` were not changed.

### Focused Results

- Compile-once structural red changed from observed 2 to 1 and passes.
- Exhaustive parsed→published field/map/Boss/registry parity passes 1/1.
- Public-parts catalog/map Arc and registry allocation identity passes 1/1.
- Complete `game::data::` slice passes 192/192, including representative parse, domain, Boss compile/reference and
  generated-preview rejection/provenance tests.
- Live RON integration passes 25/25. The canonical now records only the new internal validated-state authority;
  public, RON, save and wire contracts remain unchanged.

## 2026-08-10 — E05 Broad Validation And Review Handoff

### Final Test Evidence

- Core all-targets debug and release each pass 995 unit tests plus 51 integration tests (4 live item, 4 live catalog,
  25 RON, 14 skill runtime and 4 authored skill-suite tests).
- Game server all-targets debug and release each pass 59 tests.
- `cargo check --workspace --all-targets`, `cargo fmt --all -- --check`, scoped and repository-wide
  `git diff --check` all pass.
- A non-required strict `cargo clippy ... -D warnings` probe is not green: it promotes 83 existing Core library
  warnings and 136 library-test warnings in unchanged regions (unusual literal grouping, argument count and similar
  repository debt). No warning cited the new validated conversion or compile-count instrumentation. The Phase did
  not broaden into unrelated Clippy cleanup; compile/check/test/format/diff gates are green.

### Compatibility And Dirty Evidence

- Static search finds one production `boss_runtime.compile()` call, at its original validation point; no
  `from_validated_parts` remains. The official loader has one `into_validated` and one private consuming assembler.
- Public `GameDataBaseParts`, `GameDataBuilder` and `GameDataBase::new(parts)` remain present with unchanged
  signatures and the complete identity test passes.
- `bundle.ron` remains `06b0f189...` and live Boss RON remains `f696c9d0...`, exactly their activation hashes.
  No live data, manifest, schema, save, DTO or wire file was changed by Phase 19.
- Final hashes: live loader `1fb386ed...`, data module `859e9d1d...`, Boss source/test instrumentation `6ddd7bae...`
  and canonical `30ed5fd5...`.
- Expanded status remains exactly the activation 464 entries with normalized SHA-256
  `d43ed3c438fadacb896267830322e1f465b320e7486fdf80ed113d5f088e4bd8`. Phase targets were already represented
  as modified/untracked at activation, so no concurrent path was added, removed, reset, restored, cleaned or
  stashed.

### Phase Boundary

- P19-C01 through P19-C09 are complete. P19-C10 remains pending the required independent review.
- Phase 20 remains pending and unauthorized; no checkpoint conversion or later cleanup was prebuilt.
- Status changes to `review_pending`; `Implementation Complete` remains false until a `Complete / High` audit.

### Next Experiment

Reread all 567 lines of `goal_completion_review_guide.md`, independently inspect the implementation, policy,
tests, canonical, public/data compatibility and dirty scope, then create `COMPLETION_AUDIT.md`. Do not activate
Phase 20 unless the verdict is `Complete / High`.

## 2026-08-10 — E06 Independent Audit And Completion Transition

### Audit Result

- All 567/567 lines of `goal_completion_review_guide.md` and all 504 current Child-document lines were reread before
  the independent pass; policy, exact-order and Master gates were rechecked separately.
- `COMPLETION_AUDIT.md` connects P19-A01–A10 to the private loader, strict data/error gates, public fixture seam,
  official consumers, canonical, dirty preservation and exact validation counts. Every row passes.
- Audit-current parity/identity passes 2/2 and compile-once passes 1/1. Scoped production Clippy has no diagnostic;
  scoped test diagnostics are confined to two pre-existing non-Phase lines.
- Primary verdict is `Complete`; long-term fit is `High`; improvement class is `none`.

### Completion And Boundary

- P19-C10 is complete. Child status changes from `review_pending` to `complete`; `Implementation Complete` changes
  from false to true.
- The audit artifact has SHA-256 `37da7360400dde88d8406dd5b8eb10ccad4b6f6a88ca274b5851439da54b985e`.
- Audit-artifact-inclusive expanded status is 465 entries with normalized SHA-256
  `94926edc986880b6f431129999f7d07d2bf8b535bcdf1cc77fff8123eea8f136`.
- Phase 20 may become ready only through the Master transition. No checkpoint conversion or other Phase 20
  production work was performed by the Phase 19 audit.
