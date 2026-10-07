# Phase 19 CRA-P01-C03 Experiment Notes

Status: complete
Implementation Complete: true
Candidate: CRA-P01-C03
Master Phase: 19 of 44
Date: 2026-08-10 (`Asia/Seoul`)

## Locked Decisions

- CRA-POL-006: private consolidation may proceed only with exact public/save/RON/wire compatibility.
- Work type is behavior-preserving refactor; current successful production behavior is presumed correct until
  current-tree evidence proves otherwise.
- Preserve public `GameDataBaseParts`, `GameDataBuilder`, `GameDataBase::new`, catalog types/fields and fixture panic
  semantics. No negative consumer search authorizes removal.
- Consolidate only the private production parsed/validated/compiled→published mapping and proven duplicate Boss
  compile work.
- Characterize every field, Arc relationship, derived registry and failure-order boundary before structure changes.
- Phase 20 checkpoint conversion and all later cleanup/refactors are prohibited from this Child.

## Open Evidence Slots

- N02: activation expanded status, prospective target fingerprints and scoped dirty attribution.
- N03: current source→parse→validate→compile→preview→publish call graph and exact field lifecycle table.
- N04: public/fixture/server consumer inventory and compatibility classification.
- N05: pre-refactor field/Arc/registry/failure-order characterization results.
- N06: implemented private conversion, compile-once proof and obsolete mapping removal.
- N07: final focused/broad/static/canonical evidence and independent-review handoff.

## 2026-08-10 — N01 Policy, History And Activation Boundary

- Source/evidence: CRA-POL-006; original P01 candidate/boundary evidence; Boss-era candidate/test/boundary re-audit;
  Final exact order and test rebaseline; Master Phase 18 transition.
- Observation: current behavior is green; the defect class is coordinated private mapping and deterministic repeated
  compile work. Public fixture/programmatic construction is a separate, live compatibility seam.
- Observation: the exact required first proof is every parsed/validated/compiled-to-published field, Arc sharing,
  derived registry and fail-fast provenance/order, not a field-count snapshot or compile-only assertion.
- Impact: one private validated conversion is authorized; public API narrowing, loader/fixture merging and semantic
  error/publication changes are not.
- Disposition: activate Phase 19 only; do not inspect production until all Child documents and the final Phase 18
  audit are fully read and the dirty baseline is captured.

## 2026-08-10 — N02 Activation Dirty Baseline

- Source/evidence: expanded `git status --short --untracked-files=all`, scoped status/diff stats and SHA-256
  fingerprints after all mandatory reads and before production-content inspection.
- Observation: HEAD is `f318414fb0156a67e04c5479000c2a2683fc0c09`; expanded status is 464 entries at
  normalized SHA-256 `d43ed3c438fadacb896267830322e1f465b320e7486fdf80ed113d5f088e4bd8`.
- Prospective fingerprints: live RON loader `bd10f8a5...`, data module/public database `70f6fbf6...`, validation
  `c71c4208...`, Boss runtime data `a5fa4f77...`, combat preview `3038d69c...`, RON integration `75558ff2...`,
  live catalog audit `bf475aa3...`, data canonical `27cff557...`, live Boss RON `f696c9d0...` and bundle manifest
  `06b0f189...`.
- Observation: live loader, data module, validation, combat preview, RON integration and canonical are modified;
  Boss runtime source/data are untracked. Existing scoped diff is large and predates Phase 19. Live catalog audit
  and bundle manifest are currently clean but are still read-only prospective evidence until attribution.
- Impact: all Phase edits must be minimal, hash-tracked and reconciled against the activation content; path dirtiness
  alone cannot identify Phase ownership.
- Disposition: baseline gate complete; current production/test/data/canonical inspection may begin.

## 2026-08-10 — N03 Exact Production Lifecycle And Field Mapping

- Source/evidence: complete current `live_ron.rs` parsed/validation/publication implementation and `data/mod.rs`
  database/parts/builder/loader implementation.
- Observation: parse produces 18 non-map catalogs (`shops`, `rewards`, `events`, `visual_novels`,
  `visual_novel_triggers`, `boss_omen_chains`, raw `boss_runtime`, `abnormalities`, `corroded_employees`,
  `corroded_wave_presets`, `employee_definitions`, `equipment`, `consumables`, `buffs`, `skills`,
  `skill_fragments`, `pve_encounters`, `run_policy`) and a five-Arc `GameplayMapCatalogs` aggregate.
- Observation: validation order is run policy → top-level keys → fallible domain/cross-domain contracts (including
  Boss compile after Boss Omen local validation, followed later by PvE Boss-reference validation) → item UUIDs →
  pool IDs → equipment reward pools → reward effects → PvE enemy refs → run start → extra waves → map catalogs →
  map difficulty refs → map payload refs. Generated combat-preview validation occurs only after private candidate
  assembly and before publication.
- Observation: the compiled Boss database is currently dropped after the domain slice and recomputed during
  publication. All other parsed fields are destructured, wrapped as Arcs through public parts, destructured again,
  and installed; `ItemRegistry` is derived from the installed consumable/equipment Arcs.
- Impact: retain the compiled Boss value without moving its validation position, and collapse only the two private
  production mappings into one consuming validated-state conversion.
- Disposition: no field may be omitted, reordered semantically or published before every existing validation gate.

## 2026-08-10 — N04 Consumer And Compatibility Inventory

- Source/evidence: repository-wide Rust searches for `GameDataBaseParts`, `GameDataBuilder`, `load_live_embedded`
  and the server bootstrap wrapper.
- Observation: direct `GameDataBaseParts` construction is concentrated in `data/mod.rs`; its public type feeds
  `GameDataBase::new`, while `GameDataBuilder` is used across 33 Core/server source or test files. Programmatic
  construction preserves caller-supplied Arc identity and panic-oriented fixture validation.
- Observation: `game_server/src/main.rs` loads production data through `GameDataBase::load_live_embedded`; additional
  server handlers, Core tests and fixture generators also consume that official loader. None need to observe the
  private intermediate type.
- Impact: public parts/builder/new signatures, fields and semantics must remain byte-for-byte API compatible. A
  negative search cannot authorize their removal, and production/fixture validation lifecycles remain separate.
- Disposition: add only a crate-private validated production carrier and private consuming assembler.

## 2026-08-10 — N05 Test-First Safety Net

- Source/evidence: focused commands against the current dirty tree before production refactoring.
- Observation: `cargo test -p game_core --test ron_loading -- --nocapture` passed 25/25; `cargo test -p game_core
  game::data:: -- --nocapture` passed 189/189, with only expected `should_panic` output.
- Observation: exhaustive parsed→published semantic parity passes for all 18 non-Boss/raw-to-published catalog
  relationships, the compiled Boss identity/count contract, all five map catalogs and every derived consumable and
  equipment registry row. The first comparison used serialized RON text and failed only because `HashMap` iteration
  order is randomized; semantic `serde_json::Value` equality removed that test-harness artifact.
- Observation: public-parts characterization passes for all 18 catalog Arc identities, five map Arc identities and
  registry references to the exact supplied item allocations.
- Observation: the new official-publication compile-count test is the intended structural red: observed Boss
  compile delta is exactly 2 and required delta is 1. Thread-local test-only counting prevents parallel-test races.
- Impact: implementation may proceed only until that structural test turns green while all parity, public seam and
  existing fail-fast/provenance tests remain green.
- Disposition: preserve behavior; no unrelated production/data/canonical edits are authorized.

## 2026-08-10 — N06 Implemented Private Authority

- Source/evidence: `ValidatedGameplayRon`, `validate_domain_catalogs_with_compiled_boss`, `into_validated`, official
  `load_live_sources` and `from_validated_live_catalogs`; focused tests and obsolete-call search.
- Observation: the private validated state wraps the complete parsed aggregate plus the Boss database compiled at
  the same fail-fast point as before. It is constructible only after run-policy, top-key, domain, cross-item, pool,
  reward/PvE, run-start, extra-wave and map validations all finish successfully.
- Observation: `validate_fallible_domain_contracts` now returns its compiled Boss value after performing the exact
  previous sequence, including PvE Boss-reference validation. It is retained across later gates, not recomputed or
  published early.
- Observation: production consumes the validated state directly, creates the 18 final catalog Arcs and preserves
  all five existing map Arcs, derives `ItemRegistry` from final consumable/equipment allocations, then runs the
  unchanged generated-preview gate before returning the candidate Arc.
- Observation: the second `boss_runtime.compile()` and public-parts production round trip are gone. The public
  parts/builder/new fixture lifecycle remains unchanged and independently characterized.
- Impact: one private conversion authority now makes unvalidated production assembly inaccessible while retaining
  exact external behavior and failure ordering.
- Disposition: implementation complete; proceed through broad compatibility/static evidence only.

## 2026-08-10 — N07 Final Validation And Review Handoff

- Source/evidence: focused parity/compile/identity tests; data and RON families; complete Core/server debug/release;
  workspace check, formatter, diff checks; static call/API inventory; hashes and expanded status.
- Observation: focused structural and parity gates pass; data passes 192/192 and RON 25/25. Complete Core passes
  995 unit plus 51 integration tests in each profile; complete server passes 59/59 in each profile.
- Observation: workspace all-target check, formatter and repository diff check pass. The optional strict Clippy
  probe fails on broad pre-existing warning debt in unchanged regions; it identifies no added conversion/counter
  finding and is recorded as non-Phase baseline evidence rather than repaired across later candidates.
- Observation: only one production Boss compile remains, public fixture APIs remain present, live bundle/Boss RON
  hashes are unchanged and no public/save/wire/data migration occurred.
- Observation: expanded dirty status is unchanged at 464 entries and hash `d43ed3c4...`; final scoped hashes are
  recorded in E05. All edits were minimal within activation-known paths.
- Impact: P19-C01–C09 have direct evidence; no compatibility decision, source rewrite, fallback, duplicate private
  path or Phase 20 prebuild remains.
- Disposition: hand off as `review_pending`. Fully reread the completion guide and require `Complete / High` before
  completing P19-C10 or activating Phase 20.

## 2026-08-10 — N08 Independent Completion Audit

- Source/evidence: full completion-guide and current Child reread; `COMPLETION_AUDIT.md`; audit-current focused,
  static, canonical, consumer, data-hash and dirty-state checks.
- Observation: all P19-A01–A10 rows pass. One private validated carrier retains the only compiled Boss result through
  exact final mapping; official output/error/public fixture contracts remain fully characterized and green.
- Observation: no public/save/RON/wire migration, compatibility shim, duplicate compile/assembler, Phase 20 prebuild
  or dirty-scope loss remains. Strict global Clippy debt is transparently non-gating and outside Phase hunks.
- Impact: no material finding, policy decision, migration or correction blocks the linear exit.
- Disposition: `Complete / High`, improvement class `none`; allow the Master to mark Phase 19 complete and make
  Phase 20 ready, but not active until its own document/read protocol completes.
