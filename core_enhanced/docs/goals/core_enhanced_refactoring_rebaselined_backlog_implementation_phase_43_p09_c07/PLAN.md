# Phase 43 CRA-P09-C07 Ghost Range And False Public Delivery Surface Cleanup Goal

Status: complete
Implementation Complete: true
Goal Type: phase
Execution Mode: implement_and_verify
Production Mutation Authorized: yes — the user's 2026-08-11 instruction approves every remaining registered
cleanup after candidate-specific inventory and retained behavior guards
Parent Master: `core_enhanced_refactoring_rebaselined_backlog_implementation_master`
Phase: 43 of 44
Candidate / Work Type: CRA-P09-C07 / `cleanup`
Priority Axes / Known Subtotal: `2 / 1 / 1 / 3 / 2 / 1 / unknown = 10/18`
Direct Linear Dependency: Phase 42 CRA-P06-C07 `Complete / High`
Direct Technical Dependencies: Phase 16 CRA-P09-C06, Phase 17 CRA-P09-C03, Phase 18 CRA-P09-C04 and Phase 29
CRA-P09-C05 are `Complete / High`; Phase-10 transport/projection work is also complete
Direct Policy: CRA-POL-006 plus the user's explicit retained tile-range/`Nearest` boundary
Repository / Start Commit: `/mnt/f/work/simulator` / `f318414fb0156a67e04c5479000c2a2683fc0c09`
Initialization Date: 2026-08-11 (`Asia/Seoul`)

## Objective

Reconstruct the current target-eligibility, authored range, runtime weapon-delivery and public helper graph, then
remove or migrate only the proven ghost `range_units` and false public range/delivery surfaces. Preserve
`TileRangePolicy`, every current tile-membership predicate and target set, the current deterministic `Nearest`
selection rule, runtime equipped-weapon override, live RON publication semantics and DTO/wire behavior.

This Phase does not redesign targeting, convert tile eligibility to world-distance eligibility, alter range preview,
change delivery mechanics or stage package cleanup.

## Initialization And Activation Gate

Before activation or current production inspection:

1. read this PLAN, `EXPERIMENTS.md` and `EXPERIMENT_NOTES.md` completely;
2. verify Phase 42's 225-line audit at `dc2abc41...` is `Complete / High` and Master Phase 43 is `ready`;
3. retain the original P09-C07 catalog/boundary evidence, post-Boss C07 reaudit and final catalog/order/wave/test
   records as hypotheses rather than current facts;
4. register this Child as the sole active Phase in Master and Child documents;
5. reread all active documents, then capture filename-only prospective source/data/test/canonical fingerprints
   before opening current candidate content; and
6. inventory every definition, copy, validation, deserialize/default path, reader, helper caller, DTO/server/Unity/
   tool/fixture and accessible external consumer before any production mutation or candidate test.

No target symbol is deleted merely because an old audit or negative workspace search called it unused.

## Historical Evidence To Refresh

- Original P09 evidence reported serialized `BasicAttackDef.range_units` and `WeaponCombatProfile.range_units` were
  authored, copied and positive-validated but not read by production eligibility.
- It reported public `bodies_in_range` had no production caller and public base-UUID
  `BattleCore::basic_attack_delivery` had no workspace caller while consulting base abnormality data rather than the
  equipped runtime weapon profile.
- The then-current real authorities were `TileRangePolicy` membership for eligibility and the runtime unit's
  equipment/profile for attack delivery.
- Post-Boss reaudit kept the candidate valid, recorded explicit tests where tile range won over `range_units`, and
  left serialized/public external consumers unverified.
- Final order requires P09-C03/C04/C05/C06 to be complete first, then target-set/live-RON goldens plus consumer and
  fixture inventory. All dependencies are now complete.
- The user later approved the remaining serialized/public breaking cleanup without a shim after this inventory, and
  explicitly required current tile-range eligibility and `Nearest` selection to remain.

Historical names locate candidates; they do not prove current shape, use, serialization or removal scope.

## Current Questions To Resolve

1. Which current structs still define `range_units`, with what visibility, serde/default/alias behavior and version
   or authored source?
2. Which publication adapters copy or validate those values, and does any runtime, preview, DTO, server, tool,
   fixture or external source now read them?
3. What exact predicate selects basic targets today, and where is deterministic `Nearest` scoring/tie-breaking
   implemented?
4. How does equipped weapon profile selection override base basic-attack delivery at construction/release time?
5. Do `bodies_in_range`, `basic_attack_delivery` or any wider runtime record visibility have current callers,
   downstream imports, tests or deployment significance?
6. Does removing a serialized field require an intentional live-RON migration in every producer, generated fixture,
   example and strict unknown-field test?
7. Are any candidate names retained display/config facts even when not eligibility authorities?

Any real gameplay/display/persistence/DTO/deployed consumer narrows or blocks removal; it does not authorize a
fallback or silent behavior change.

## Required Authority Model

The target authority is:

`authored tile range policy/pattern + current body/tile/facing`
→ `typed membership query`
→ `eligible hostile set`
→ `current deterministic selection policy (including Nearest)`
→ `windup/revalidation/release`

and independently:

`runtime unit + equipped weapon/base profile resolution`
→ `actual basic-attack delivery profile`
→ `instant/projectile release behavior`.

No scalar world-distance field may become an eligibility authority, and no base-UUID metadata helper may override
the runtime unit's resolved equipment.

## Preserved Invariants

### Targeting and range

- `TileRangePolicy`, `TileRangePattern`, facing transforms, valid-board clipping and current membership stay exact.
- Every current basic target set, release-time revalidation result, score and stable identity tie stays exact.
- The current `Nearest` meaning and deterministic order remain unchanged.
- Occupancy/obstacle behavior changes only if already part of the retained typed query; no new filter is added.

### Delivery and lifecycle

- Equipped weapon override, base fallback and instant/projectile delivery behavior remain exact.
- Windup, reacquisition, release, projectile motion, impact, event order and damage handoff remain unchanged.
- P09-C03/C04 actual delivery-set usefulness/preview and P09-C05 motion-kernel contracts are retained.

### Serialization and external boundaries

- Every live RON producer is deliberately migrated if and only if a serialized field is removed; strict load and
  generated fixture expectations must be explicit.
- No DTO, server payload, Unity range preview, save/checkpoint or canonical shape changes unless current inventory
  proves it is the same approved candidate surface and exact consumer migration is included.
- No serde alias, ignored legacy key, permissive default, dual shape or compatibility shim is introduced.

### Dirty work and scope

- Existing user/completed-campaign changes remain attributed by activation fingerprints.
- Protected marker `game_server/logs/app.log.2026-08-09` remains exact.
- Phase 44 Cargo package/target/dependency work remains untouched.

## Prospective Current Evidence Paths

These historical carrier classes are prospective only until post-activation filename/hash capture:

- Core definitions/publication: `ability.rs`, equipment-data definitions/validation, abnormality/live-data adapters;
- runtime targeting/delivery: battle core basic attack, targeting, types, build/setup and tile-range modules;
- projections/consumers: range preview, world behavior DTO construction and server mappings;
- authored sources: equipment, abnormality, corroded-employee, skill-fragment or other live RON that writes the
  candidate fields;
- tests: tile-range cross-products, target-set/Nearest, weapon override/release, range preview, live-RON and server
  serde/transport suites;
- accessible external sources and generated fixtures under the available workspace tree; and
- runtime/gameplay/data/server canonicals plus Cargo manifests and protected marker.

The active Phase must replace this list with exact current paths, hashes and status before content inspection.

## Implementation Sequence

### P43-1 — Current graph and compatibility inventory

Inventory definitions, serde/default/validation, authored producers, publication copies, runtime reads, public helper
callers, DTO/server/Unity/tool/fixture/external consumers and resolved weapon authority. Classify every candidate
surface as retained, removed, visibility-narrowed, or migrated.

### P43-2 — Pre-cleanup assertion transfer

With the old surface intact, add or strengthen controls that prove:

- target sets and `Nearest` selection are independent of any ghost scalar range;
- edge/facing tile eligibility uses the retained typed query;
- equipped weapon delivery wins over base metadata exactly as production does;
- any retained helper/display consumer receives the same value; and
- every affected official live RON/fixture parses before a deliberate field migration.

Every mandatory filter must select a nonzero test.

### P43-3 — Minimal approved cleanup or migration

Remove only zero-authority/zero-consumer fields, helpers or visibility. If `range_units` remains serialized and the
inventory confirms no retained meaning, remove the field plus all authored/copy/validation writers in one strict
migration with no alias/default. Localize test-only helpers at their real owners. Retain any surface with a current
meaningful consumer.

### P43-4 — Focused parity and legacy absence

Repeat target-set/Nearest, tile query, weapon override/delivery, range preview, live-RON and affected server/fixture
tests. Search for removed members, legacy keys, aliases, fallback distance checks, base/runtime delivery divergence
and compatibility shims.

### P43-5 — Broad/static validation

Run Core debug/release all targets, affected server debug/release tests, workspace debug/release checks, normal
Clippy, workspace format check, diff check, sorted test-list fingerprints, manifests/canonicals, dirty status and
protected-marker verification.

### P43-6 — Independent exit audit

Create `COMPLETION_AUDIT.md`, independently apply `goal_completion_review_guide.md`, rerun direct retained-authority
controls and broad debug evidence, and require `Complete / High` before Phase 44 becomes ready.

## Minimum Validation Contract

- exact target-set and current `Nearest` selection tests before and after cleanup;
- typed tile range membership/materialization and edge/facing parity;
- equipped weapon/base delivery-resolution and instant/projectile release tests;
- official live-RON strict load plus every directly affected generated fixture/example;
- affected range-preview/world/server serialization tests;
- exact removed-symbol/key/fallback/alias search with non-doc executable scope;
- `cargo test -q -p game_core --all-targets` and `--release`;
- affected `game_server` debug/release all-target suites when server carriers or package compatibility are in scope;
- workspace debug/release all-target checks, normal Clippy, format and whole-worktree diff checks; and
- current test-list, manifest, canonical, status and marker hashes.

## Omitted Validation

- Unity build/runtime is omitted only if exact inventory proves no external source/fixture/schema change and server/
  canonical parity directly covers the retained shape; otherwise it becomes mandatory or a recorded block.
- Save migration probes are omitted only if no checkpoint/persistence carrier contains the candidate surface.
- Performance benchmarks are omitted because scalar/public cleanup must not change the retained targeting/motion hot
  path; any algorithm delta is out of scope and stops the Phase.
- Phase-44 metadata/dependency/standalone tool validation remains deferred to Phase 44.

## Completion Conditions

- Every current field/helper/visibility, writer, validation, reader and consumer is named and classified.
- Tile eligibility and current `Nearest` selection are directly protected with the old surface present.
- Runtime equipped-weapon delivery authority and release behavior are directly protected.
- Any serialized field removal includes exact official data/fixture migration with no compatibility fallback.
- Only proven ghost/false public range or delivery surfaces are removed or narrowed.
- Target sets, delivery, preview, DTO/wire, save and runtime behavior remain exact.
- Focused/broad debug/release/static checks are green with exact counts.
- Non-target hashes, unrelated dirty work and protected marker are exact.
- A separate completion audit returns `Complete / High`.

## Completion Audit Evidence Matrix

| ID | Condition | Required evidence | Status |
| --- | --- | --- | --- |
| P43-C01 | current range/delivery/public graph is exact | definitions, serde/defaults, writers, validators, readers, callers | pass |
| P43-C02 | retained eligibility authority is exact | tile query → target set → current Nearest/revalidation | pass |
| P43-C03 | dirty work is preserved | activation/current hashes, scoped diffs and marker | pass — activation status-hash anomaly explicitly corrected by exact current/path evidence |
| P43-C04 | every candidate surface is classified | field/helper/visibility/data/consumer disposition matrix | pass |
| P43-C05 | current behavior is characterized | target-set, Nearest, tile edge/facing, delivery override baselines | pass |
| P43-C06 | useful assertions precede cleanup | retained production/data controls green with old shape | pass |
| P43-C07 | public/serialized compatibility gate is exact | package/deployment/external/data/fixture inventory and approval | pass |
| P43-C08 | only ghost/false surface is removed | exact API/data delta; no fallback, alias or shim | pass |
| P43-C09 | targeting and delivery behavior is exact | pre/post target/release/preview parity | pass |
| P43-C10 | DTO/data/save/server/Unity boundary is exact | hashes, migrations/searches and affected integration checks | pass |
| P43-C11 | broad/static validation is green | both profiles, workspace, format/diff and Clippy | pass |
| P43-C12 | independent audit/next boundary is current | `Complete / High`; Phase 44 absent until exit | pending |

## Stop Conditions

Stop and do not activate Phase 44 if a candidate surface has an unhandled runtime/display/DTO/save/deployed
consumer; scalar distance participates in eligibility; `Nearest`, tile membership, target order, weapon override or
release behavior changes; live data needs a migration beyond the approved candidate; a fallback/alias is required;
dirty overlap cannot be isolated; Phase-44 work is required; a mandatory test selects zero; the marker changes; or
the independent audit is below `Complete / High`.

## Initialization Dirty Baseline

- Expanded status before this Child: 594 rows at canonical C-sort SHA-256
  `e44f1ce833fcb160065d50b1ce8ed32de872167aaef37eaf919691766625c844`.
- Phase 43 path was absent and active implementation Child count was zero.
- Phase 42 completion audit is 225 lines at SHA-256
  `dc2abc41135453991d616ce9f501adf698d143dd3e6542a8b9db05a4c8d2762f`, verdict `Complete / High`.
- Protected marker is 57 bytes at SHA-256
  `614d2fbdbdb0b29def7b90d1fdbc65148aba63339eea72fef5998d4a8ec22730`.
- These three new Child documents are the only expected status-set additions before activation evidence.
