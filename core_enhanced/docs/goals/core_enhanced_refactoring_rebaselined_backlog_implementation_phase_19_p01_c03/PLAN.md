# Phase 19 Goal — CRA-P01-C03 Consolidate The Private Parsed-To-Published Mapping

Status: complete
Implementation Complete: true
Goal Type: phase
Execution Mode: implement_and_verify
Candidate ID: CRA-P01-C03
Master Phase: 19 of 44
Work Type: behavior-preserving refactor
Production Mutation Authorized: yes — the user's Master execution request remains active; authorization is limited
to this Phase after Phase 18's `Complete / High` exit
Repository Root: `/mnt/f/work/simulator`
Activation Date: 2026-08-10 (`Asia/Seoul`)

## Objective And Completion Conditions

Replace the repeated private production parsed/validated/compiled-to-published catalog mapping with one validated
conversion authority. The production path must compile derived Boss definitions exactly once, retain that validated
result through publication and perform each parsed-to-Arc/database field mapping once. The public
`GameDataBaseParts` and `GameDataBuilder` fixture/programmatic APIs remain distinct and unchanged.

This Phase is complete only when every current parsed, validated, compiled, published and derived-registry field is
inventoried; tests first freeze exact field and Arc-identity parity plus failure ordering; the single private
conversion proves domain validation precedes publication; duplicate Boss compilation and only the proven private
mapping boilerplate are removed; catalog types/names, registry derivation, official live output, source/error
provenance, generated-preview gate and fixture panic semantics remain exact; public Rust/RON/save/wire APIs are not
narrowed; focused and broad validation passes; and an independent phase-exit audit returns `Complete / High`.

## User Or Contract-Visible Problem

The current production result is correct, but one catalog addition requires synchronized edits across the parsed
aggregate, loader destructuring, public parts carrier and final database construction. Boss runtime made the
amplification concrete: validation compiles a Boss catalog for cross-reference checks, drops it, and publication
compiles the same deterministic definition again before repeating the field mapping.

This is omission/change-amplification risk rather than a current gameplay defect. The target is private authority
consolidation with zero externally observable behavior change.

## Dependency And Policy Authority

- Master: `../core_enhanced_refactoring_rebaselined_backlog_implementation_master/PLAN.md`, Phase 19 registry row.
- Direct linear predecessor: Phase 18 CRA-P09-C04, reviewed `Complete / High` in
  `../core_enhanced_refactoring_rebaselined_backlog_implementation_phase_18_p09_c04/COMPLETION_AUDIT.md`.
- Approved cross-cutting policy: CRA-POL-006 in
  `../core_enhanced_refactoring_post_boss_runtime_reaudit_final_catalog_consolidation/APPROVED_POLICY_DECISIONS.md`.
- Stabilized publication dependencies: P04-C01, P08-C04 and P09-C01 are already `Complete / High` in the Master.
- Original evidence:
  - `../core_enhanced_refactoring_audit_phase_01_live_data_loading_and_validation/CANDIDATE_CATALOG.md`
  - `../core_enhanced_refactoring_audit_phase_01_live_data_loading_and_validation/BOUNDARY_AUDIT.md`
  - `../core_enhanced_refactoring_audit_phase_01_live_data_loading_and_validation/EXPERIMENTS.md`
- Re-audited evidence:
  - `../core_enhanced_refactoring_post_boss_runtime_reaudit_phase_01_live_data_loading_and_publication/CANDIDATE_REAUDIT.md`
  - `../core_enhanced_refactoring_post_boss_runtime_reaudit_phase_01_live_data_loading_and_publication/TEST_REAUDIT.md`
  - `../core_enhanced_refactoring_post_boss_runtime_reaudit_phase_01_live_data_loading_and_publication/BOUNDARY_AUDIT.md`
- Final order/test/gate evidence:
  - `../core_enhanced_refactoring_post_boss_runtime_reaudit_final_catalog_consolidation/EXACT_LINEAR_IMPLEMENTATION_ORDER.md`
  - `../core_enhanced_refactoring_post_boss_runtime_reaudit_final_catalog_consolidation/TEST_REBASELINE_CATALOG.md`
  - `../core_enhanced_refactoring_post_boss_runtime_reaudit_final_catalog_consolidation/IMPLEMENTATION_WAVES_AND_GATES.md`

CRA-POL-006 permits internal/private consolidation only while current external contracts remain exact. Public API
removal or narrowing cannot be inferred from a negative workspace search; it requires consumer inventory plus a
separate deprecation/versioned migration or breaking approval. No such migration is approved here.

## Priority And Current Disposition

- Priority axes: `2 / 1 / 1 / 3 / 1 / 2 / unknown`; known subtotal `10/18`.
- Re-audited disposition: `still_valid`, `characterize_private_conversion_then_implementation`.
- Current live-tree disposition: discovery pending until all three Child documents and the complete final Phase 18
  audit are fully read and the scoped dirty baseline refreshes historical source/test/data/canonical targets.

## Current And Target Authority

- Historical current authority: the private parsed aggregate owns candidate catalogs; validation compiles derived
  Boss data transiently; the production loader manually destructures fields, compiles Boss again, forms a public
  parts value and the private constructor manually destructures/rebuilds the final immutable database.
- Target authority: one private validated production conversion value owns validated parsed inputs and retained
  compiled derivatives, then maps every production catalog into the final immutable database exactly once.
- Public fixture/programmatic authority remains `GameDataBaseParts`/`GameDataBuilder`; production and fixture
  lifecycles must not be merged.
- Exact current types, field count, call graph and failure order must be rediscovered only after the mandatory
  Child/predecessor read and dirty-baseline gate. Historical names are evidence, not mutation instructions.

## In Scope

- inventory the complete production source→parse→domain-validation→derived-compile→cross-reference/
  feasibility-validation→publication chain;
- enumerate every parsed catalog, compiled derivative, final `Arc` field and derived registry with an exact parity
  table;
- capture expanded dirty status, scoped diffs and SHA-256 fingerprints before production edits;
- add test-first characterization proving official live publication field equality, expected Arc sharing/identity,
  derived item/other registry parity and unchanged ordering;
- prove parse/domain/cross-reference/generated-preview failures occur at the same stage with exact source/domain/
  object/path/cause provenance before any publication result exists;
- retain compiled Boss definitions from the validated private path so production compiles them exactly once;
- introduce the smallest private validation-state/conversion authority needed to make invalid publication
  unrepresentable inside production;
- remove only repeated private production mapping/destructuring and duplicate deterministic compile work proven
  obsolete by the safety net;
- update the nearest live-data canonical only if the implementation changes internal authority worth documenting.

## Out Of Scope

- removing, renaming, narrowing or changing visibility/signature/fields of public `GameDataBase`,
  `GameDataBaseParts`, `GameDataBuilder` or any public catalog type;
- merging fallible production loading with panic-oriented fixture/programmatic construction;
- changing embedded source inventory, RON schemas/defaults, validation rules, fail-fast/first-error order, error
  payloads, warnings or generated-preview feasibility policy;
- changing Arc allocation/identity guarantees, item-registry derivation, catalog contents, balance or runtime
  gameplay behavior;
- aggregating errors, adding recovery/fallback/partial publication, migrating saves/wire or deleting unknown
  external tooling such as `config/game_balance.toml`;
- Phase 20 checkpoint conversion or any later behavior-preserving/cleanup candidate;
- broad formatting, data rewrite or dirty-worktree normalization.

## Preserved Invariants

1. Official embedded RON and physical/source inventory remain byte-for-byte and set-for-set unchanged.
2. Parse, domain validation, compiled-reference validation and generated-preview gates occur in the same causal order.
3. The first failure retains exact source/domain/object/field/cause provenance and prevents publication atomically.
4. Every final catalog field has the same public type/name/content and expected `Arc` sharing behavior.
5. Derived item and other registries contain the same exact keys, values and source catalog identities.
6. Boss definitions compile exactly once on the successful production path without changing compiler semantics.
7. `GameDataBaseParts`, `GameDataBuilder`, `GameDataBase::new` and their panic/default/fixture semantics remain exact.
8. Server startup and all Core consumers receive the same immutable published database behavior.
9. No RON/save/checkpoint/DTO/event/command/wire/public-Rust migration or compatibility shim is introduced.
10. Pre-existing user changes are preserved without reset, restore, clean, stash or broad unrelated formatting.

## Implementation And Dirty-Worktree Strategy

After all three Child documents exist and Master records sole activation, read them completely and reread the full
Phase 18 completion audit. Capture expanded status plus source/test/data/canonical hashes before detailed production
inspection. Treat every existing hunk as user/campaign-owned until scoped attribution proves otherwise.

Use characterization-first proof in this order:

1. Trace the current production loader and validation/compile/publication call graph without editing it.
2. Build an exact field lifecycle table: parsed source, validation owner, compiled form, final field/Arc and registry
   consumer.
3. Run current official live, data-module, Boss compiler/reference, preview/publication and fixture/builder baselines.
4. Add parity and failure-order tests before refactoring; tests must pass on current correct behavior and become the
   invariant net for structural change.
5. Introduce one private validated conversion state and retain compiled derivatives through publication.
6. Route only the production path through it; remove only mapping/compile duplication whose callers are proven
   transferred.
7. Re-run focused, complete Core debug/release, live RON/catalog, server and static/compatibility checks.

Because this is behavior-preserving, an intended behavioral red is not required. Test-first means the complete
characterization must exist and pass before production structure changes; a structural mutation test or compile
counter may be added only if it observes production behavior without creating test-only authority.

## Migration, Deployment And Compatibility

- No migration or compatibility interval is approved or necessary for a private-only refactor.
- Public Rust shapes and visibility, RON, V3/save/checkpoint, DTO/wire and Unity contracts remain exact.
- `GameDataBaseParts` and builder consumers are preserved even if no production caller exists in the workspace.
- If consolidation requires a public field/signature/visibility change, different validation ordering, new error
  shape, source normalization or altered Arc/registry behavior, stop as `policy_decision_required` rather than
  widening this Phase.
- No data rewrite, dual path or fallback private conversion may remain at completion.

## Validation Plan

- Baseline: exact official live loader, RON suite, data-module families, Boss compile/reference tests,
  preview/publication negatives and fixture/builder panic/default tests.
- Test-first parity: exact parsed/validated/compiled→published field table, public values/types, required Arc identity
  relationships, derived registries and successful production compile count.
- Failure order: representative parse, domain validation, Boss compile/reference and generated-preview failures with
  exact provenance and proof no final database escapes.
- Focused final: private conversion, compile-once and every-field/registry characterization.
- Broad: complete Core debug/release, official integrations and complete game server debug/release.
- Compatibility/static: public item inventory, official RON hashes/source set, call graph/obsolete duplication search,
  scoped status/diff/hashes, formatter and diff check.
- Zero-test filters, compile-only success, snapshots without semantic equality and workspace-only negative consumer
  searches do not count as sufficient proof.

## Completion Evidence Matrix

| ID | Condition | Required evidence | Status |
| --- | --- | --- | --- |
| P19-C01 | complete private publication graph and field lifecycle identified | source/caller and exact field table | complete |
| P19-C02 | dirty targets and concurrent work preserved | pre/post status, scoped diff and hashes | complete |
| P19-C03 | current correct behavior characterized before refactor | field/Arc/registry/failure-order tests green first | complete |
| P19-C04 | one private validated production conversion exists | source/state/caller inventory | complete |
| P19-C05 | Boss derived catalog compiles once | direct successful-path proof and obsolete-call audit | complete |
| P19-C06 | every final field and derived registry is identical | exact parity table and official live output | complete |
| P19-C07 | validation/error/preview ordering is exact | negative provenance and atomic-publication matrix | complete |
| P19-C08 | public/fixture/RON/save/wire contracts are unchanged | API/hash/consumer and builder/fixture evidence | complete |
| P19-C09 | focused and broad validation is green | nonzero debug/release/Core/server/static counts | complete |
| P19-C10 | canonical/downstream boundary and audit are current | no-prebuild audit and `Complete / High` review | complete |

## Stop Conditions

Stop and do not activate Phase 20 if the production/fixture boundary cannot remain private, if any observable value,
Arc/registry identity, validation/error order or public surface must change, if current field mapping is not actually
equivalent, if user-owned dirty work cannot be separated safely, or if the independent audit is below
`Complete / High`.
