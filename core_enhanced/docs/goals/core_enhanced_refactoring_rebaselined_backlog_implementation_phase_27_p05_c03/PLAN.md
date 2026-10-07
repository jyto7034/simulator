# Phase 27 Goal — CRA-P05-C03 Consolidated Combat-Preview Resolution And Ghost-Floor Containment

Status: complete
Implementation Complete: true
Goal Type: phase
Execution Mode: implement_and_verify
Candidate ID: CRA-P05-C03
Master Phase: 27 of 44
Work Type: behavior-preserving refactor
Production Mutation Authorized: yes — limited to Phase 27 after the sole-active transition, complete required-
document reread and prospective fingerprint capture; no Phase 28 work is authorized
Repository Root: `/mnt/f/work/simulator`
Initialization Date: 2026-08-10 (`Asia/Seoul`)

## Objective And Completion Conditions

Consolidate duplicated combat-preview resolution and validated-instance-to-preview conversion into one private typed
authority that serves ordinary manifested, Boss, Event-authored and Monolith paths. Preserve exact node/category/
encounter/difficulty/seed inputs, all preview fields and ordering, validation/error timing, presentation-cache
lifetime, Event retry, Monolith invalidation, setup regeneration and serde/public behavior.

Inventory every public/internal caller and contain the historical ghost Floor input so it cannot become a second
gameplay authority. Under CRA-POL-006, do not remove or narrow public Rust symbols, request fields or signatures on a
workspace-only negative search. The default Phase result is private consolidation with current public compatibility
retained and Floor documented/proved as non-authoritative; any public removal, deprecation rollout or versioned API
migration requires evidence and authorization beyond this private refactor.

This Phase is complete only when the current resolver/converter/cache/caller graph is rediscovered; existing and new
characterization first prove ordinary/Boss/Event/Monolith parity, deterministic multi-seed output, full conversion
field completeness, cache hit/miss/invalidation and exact invalid-input errors; one private resolved input and one
private conversion/resolution path replace duplicated authority; P05-C01's battle-start cache non-authority remains
exact; no public/serde/RON/save/wire contract is narrowed; no Phase 28 work is prebuilt; focused and broad Core/server
debug/release validation passes; and independent review returns `Complete / High`.

## User Or Contract-Visible Problem

Historical audits found two manual mappings of more than twenty battlefield-instance fields into `CombatPreview`,
two world resolvers with different input/cache/validation ordering, and a threaded Floor input that generation does
not read. This makes ordinary, Event, Boss and Monolith parity dependent on synchronized private edits and exposes a
misleading affordance that could accidentally regain scaling authority.

The user-visible result must not change. The same authored encounter, deterministic seed, manifested difficulty,
mission/template, warning, spawn, reward/setup and error behavior must be produced. Serialized preview caches remain
presentation compatibility only and battle setup continues to regenerate authoritative input as established by
CRA-P05-C01.

## Dependency And Evidence Authority

- Master: `../core_enhanced_refactoring_rebaselined_backlog_implementation_master/PLAN.md`, Phase 27 registry row.
- Direct linear predecessor: Phase 26 CRA-P04-C04, final `Complete / High` in
  `../core_enhanced_refactoring_rebaselined_backlog_implementation_phase_26_p04_c04/COMPLETION_AUDIT.md`.
- Required semantic prerequisite: Phase 06 CRA-P05-C01, final `Complete / High` in
  `../core_enhanced_refactoring_rebaselined_backlog_implementation_phase_06_p05_c01/COMPLETION_AUDIT.md`; persisted
  preview caches no longer feed authoritative battle setup.
- Existing closed foundations: P05-C02 publication feasibility and P05-C04 typed setup plan remain regression guards,
  not implementation targets.
- Downstream dependency: Phase 28 CRA-P08-C05 may start only after this Phase is `Complete / High`.
- Original candidate:
  `../core_enhanced_refactoring_audit_phase_05_combat_preview_and_setup/CANDIDATE_CATALOG.md`.
- Reaudit:
  `../core_enhanced_refactoring_post_boss_runtime_reaudit_phase_05_combat_preview_setup_and_boss_publication/CANDIDATE_REAUDIT.md`.
- Final catalog/order/test/gates:
  - `../core_enhanced_refactoring_post_boss_runtime_reaudit_final_catalog_consolidation/REBASELINED_CANDIDATE_CATALOG.md`
  - `../core_enhanced_refactoring_post_boss_runtime_reaudit_final_catalog_consolidation/EXACT_LINEAR_IMPLEMENTATION_ORDER.md`
  - `../core_enhanced_refactoring_post_boss_runtime_reaudit_final_catalog_consolidation/TEST_REBASELINE_CATALOG.md`
  - `../core_enhanced_refactoring_post_boss_runtime_reaudit_final_catalog_consolidation/IMPLEMENTATION_WAVES_AND_GATES.md`
  - `../core_enhanced_refactoring_post_boss_runtime_reaudit_final_catalog_consolidation/APPROVED_POLICY_DECISIONS.md`

## Approved Policy And Current Disposition

- CRA-POL-006: current save/RON/wire/public API compatibility is preserved by default. Unknown deployed consumers
  prevent public symbol/field/signature removal on negative workspace search alone; private consolidation may proceed
  when observable contracts do not change.
- Priority axes: `3 / 1 / 2 / 3 / 3 / 2 / unknown`; known subtotal `14/18`.
- Final readiness: Tier B, after P05-C01 and a public caller inventory.
- Reaudited disposition: `still_valid`; Boss work did not remove the two mappings, two resolvers or unread Floor
  threading.
- Final exact order narrows the exit to ordinary/Boss/Event/Monolith parity plus caller inventory and removal of
  duplicate private mapping only.

## Approved Dirty-Boundary Exception

- On 2026-08-10 at `22:55:07 +0900`, external user state created
  `game_server/logs/app.log.2026-08-09` with the exact content
  `로그 손실을 한 경로의 예외로 명시적 승인` (57 bytes, SHA-256
  `614d2fbdbdb0b29def7b90d1fdbc65148aba63339eea72fef5998d4a8ec22730`).
- This is explicit approval to accept the loss of the original 688,108-byte untracked diagnostic log as a one-path
  dirty-boundary exception. It is an approval marker, not restoration of the original bytes, metadata or log
  contents.
- The exception covers only that original file loss. It does not waive preservation of any other activation or
  concurrent user-owned path, broaden Phase 27 production scope, weaken functional/compatibility gates or authorize
  Phase 28 before a fresh `Complete / High` audit.
- The new 57-byte marker is itself current user-owned untracked state and must be preserved unchanged.

## Historical Current And Target Authority

- Historical inputs: manifested map node/difficulty and Event-authored encounter/category own generation inputs.
- Historical production: `BattlefieldInstance`, two mapping blocks and two world resolvers duplicate conversion,
  validation and cache decisions.
- Historical ghost affordance: Floor is threaded through request/API/test surfaces but is not read by generation;
  manifested difficulty is the canonical scaling authority.
- Target private authority: one typed resolved-preview input distinguishes manifested and Event-authored combat; one
  validated-instance conversion owns every `CombatPreview` field; one resolver owns cache-key/validation order.
- Preserved public authority: existing public Rust symbols, request fields/signatures and serialized preview shape
  remain compatible unless an independently authorized migration is proven necessary.
- Battle setup authority remains the P05-C01/P05-C04 regenerated typed setup plan, not the presentation cache.

Current source is authoritative after activation and may refine names or exact module boundaries without changing
the approved behavior-preservation contract.

## In Scope

- every ordinary manifested, Boss, Event-authored and Monolith preview resolver/converter caller;
- exact resolved input: node identity, category, encounter, manifested difficulty, deterministic seed and context;
- validated `BattlefieldInstance` to `CombatPreview` field/order conversion;
- preview cache key, hit/miss, publication, replacement and invalidation ordering;
- Event retry and Monolith stage/invalidation behavior;
- Floor parameter/request-field read/write/caller inventory and proof of non-authority;
- invalid difficulty, encounter/category mismatch and generation error parity;
- deterministic multi-seed and full preview equality characterization;
- public Rust, serde, V3, RON, server and accessible/canonical external consumer compatibility;
- focused/broad Core/server debug/release, formatter, diff, static and canonical gates.

## Out Of Scope

- changing difficulty tuning, Floor scaling, seed derivation, encounter selection or map generation;
- changing battlefield template, spawn, mission, warning, reward, Boss binding or Monolith stage policy;
- changing Event retry/commit semantics, setup feasibility or typed setup-plan ownership;
- deleting/narrowing public `*_at_floor` APIs, `BattlefieldGenerationRequest` fields or package targets without the
  CRA-POL-006 consumer/migration gate;
- deleting the serialized presentation cache, changing V3/checkpoint schema or using cache as setup authority;
- merging preview presentation with battle setup/execution or absorbing CRA-P05-C05 cleanup;
- adding compatibility aliases, dual APIs, fallback, normalization or silent recovery;
- Phase 28 implementation or unrelated combat-preview/setup cleanup.

## Required Test-First Characterization Matrix

| Family | Required success evidence | Cache/order evidence | Failure/compatibility evidence |
| --- | --- | --- | --- |
| Ordinary manifested | exact input and complete preview equality across representative seeds/difficulties | miss→publish, hit parity; setup regenerates after P05-C01 | invalid difficulty/encounter mismatch exact error and no mutation |
| Boss | exact Boss preview/binding-facing fields and deterministic equality | no cache-derived setup authority | invalid Boss binding/setup guards unchanged |
| Event-authored combat | exact authored encounter/category/seed parity | retry after failed/retreated attempt resolves identically under current cache rules | wrong encounter/category and late failure remain exact |
| Monolith | exact manifested stage/difficulty preview parity | stage change/invalidation and regeneration exact | invalid stage/encounter behavior unchanged |
| Conversion completeness | every `BattlefieldInstance` field maps once with exact vector/order/value semantics | one conversion owner | no defaulted/dropped/duplicated field |
| Public/Floor compatibility | caller/symbol/request inventory | Floor cannot influence generated output or cache authority | current public signatures/serde remain exact; no removal on unknown deployment |

Tests must protect observable preview/state/error behavior, not a future helper name or private type layout. Existing
tests may satisfy rows only after their assertions, selected counts and current relevance are independently checked.

## Implementation Constraints

1. Capture expanded status, HEAD and prospective hashes before Phase 27 production-detail inspection.
2. Inventory all generation requests, public/internal functions, mappings, resolver callers, cache operations and
   consumers before selecting a boundary.
3. Prove whether Floor is read anywhere and classify each caller as workspace, canonical external, unknown deployment
   or test-only before changing any signature.
4. Run nonzero relevant baseline filters before adding or changing tests.
5. Add missing ordinary/Boss/Event/Monolith, multi-seed, cache and field-completeness characterization on old
   production before refactoring.
6. Preserve exact lookup/validation/cache/error order and P05-C01 setup regeneration.
7. Prefer one private typed input and conversion/resolution authority; do not create a second public API family.
8. Under CRA-POL-006, retain current public symbols/fields/signatures when deployed consumers remain unknown.
9. Remove only proven duplicate private mapping/resolver mechanics; do not absorb P05-C05.
10. Stop for independent audit before Phase 28 activation.

## DTO, Data, Save And Deployment

- DTO/serde migration: none authorized; exact `CombatPreview` public and serialized shape must remain.
- Live RON migration: none; encounters, templates, difficulty, Boss and Monolith content remain unchanged.
- Save/checkpoint migration: none; V3 presentation cache remains compatible and non-authoritative for setup.
- Public Rust deployment: preserve current symbols and request shape under CRA-POL-006. Caller inventory is evidence,
  not automatic breaking-change authorization.
- Server/Unity deployment: no wire change, version cutover, dual read/write or fallback is authorized.

## Validation Strategy

- Baseline: combat preview, setup, world combat, map/Event/Monolith, checkpoint/Continue and live publication suites
  with explicit nonzero counts.
- Test-first: exact four-family resolver parity, deterministic seeds, field completeness, cache transitions,
  invalid-input errors and Floor non-authority/public inventory.
- Focused implementation: rerun every producer/caller/cache/setup/persistence family.
- Broad Core: complete debug/release unit and official integration tests plus debug/release all-target checks.
- Broad server: complete debug/release tests/checks because preview/result/checkpoint types compile through server.
- Static: one private conversion/resolution authority, no dropped field, no new cache/setup authority, no public
  narrowing, no fallback/alias/Phase 28 symbol; scoped hashes/status and diff/formatter checks.
- Compatibility: public symbol/signature/serde inventory, V3/RON/server/Unity canonical hashes and source reads.
- Diagnostic: non-strict Clippy may inventory established debt; strict warning denial is not claimed unless globally
  green.

## Completion Evidence Matrix

| ID | Condition | Required evidence | Status |
| --- | --- | --- | --- |
| P27-C01 | complete preview resolver/converter/cache/caller graph identified | four families, inputs, mappings, public consumers | pass — completion audit |
| P27-C02 | dirty targets and concurrent work preserved | pre/post expanded status, scoped diff and hashes | pass — exact one-path loss exception plus activation-set reconstruction |
| P27-C03 | old behavior characterized first | nonzero four-family/seed/cache/field/error matrix | pass — completion audit |
| P27-C04 | one private resolved-input and conversion authority | source and negative duplicate searches | pass — completion audit |
| P27-C05 | cache/setup/error ordering exact | P05-C01 durable sequence and resolver transition tests | pass — completion audit |
| P27-C06 | ordinary/Boss/Event/Monolith parity exact | full preview/result/state equality | pass — completion audit |
| P27-C07 | ghost Floor contained without public narrowing | read/caller inventory, non-authority tests, unchanged API | pass — completion audit |
| P27-C08 | DTO/V3/RON/server/Unity compatibility exact | type/schema/consumer/canonical evidence | pass — completion audit |
| P27-C09 | focused and broad validation green | nonzero debug/release/Core/server/static counts | pass — completion audit |
| P27-C10 | downstream boundary and independent audit current | no Phase 28 prebuild and `Complete / High` review | pass — fresh `Complete / High` audit; Phase 28 may become ready |

## Activation Protocol

Before production inspection:

1. Master records Phase 27 as the sole `active` Child and Phases 28–44 remain pending.
2. Change all three Child headers from `initialized` to `active` and authorize only this Phase.
3. Fully reread this PLAN, `EXPERIMENTS.md`, `EXPERIMENT_NOTES.md`, final Phase 26 audit and P05-C01 dependency audit.
4. Capture HEAD, expanded status count/hash and prospective hashes for likely source/test/data/canonical targets using
   path discovery only, without reading their content.
5. Record that evidence in the Child before examining production details.

No Phase 27 source/test/data/canonical content inspection or mutation is authorized before all five steps.

## Stop Conditions

Stop and do not activate Phase 28 if evidence requires changing any public Rust symbol/field/signature, serialized
preview/V3/RON/wire shape, difficulty/Floor/seed/encounter policy, cache lifetime, setup authority, Event retry,
Monolith invalidation, Boss binding, error timing or consumer deployment assumption; adding migration/deprecation/
fallback/dual behavior; absorbing P05-C05; prebuilding Phase 28; user-owned dirty overlap cannot be separated;
required tests select zero cases; or independent audit is below `Complete / High`.

사용자와 의논하여 정해야 할 정책이 있을 경우 Goal을 종료한다.
