# Phase 27 CRA-P05-C03 Experiments

Status: complete
Implementation Complete: true
Candidate: CRA-P05-C03
Master Phase: 27 of 44
Date: 2026-08-10 (`Asia/Seoul`)

## Experiment Rules

- Do not inspect Phase 27 production source, test implementation, live data or canonical content before sole
  activation and the complete activation protocol.
- Record actual nonzero test counts; zero selected cases are not evidence.
- Characterize ordinary/Boss/Event/Monolith resolution, complete conversion, deterministic seeds, cache transitions,
  errors and Floor non-authority before refactoring.
- Preserve P05-C01 battle-start regeneration and every public/serde/V3/RON/wire contract.
- CRA-POL-006 forbids public narrowing on workspace-only caller evidence.
- Preserve unrelated dirty work; do not reset, restore, clean, stash or broadly format.
- Phase 28 work is forbidden.

## Historical Hypothesis To Verify

Historical evidence reports two validated-instance-to-preview mappings, two world resolvers with different input/
cache/validation ordering and an unread threaded Floor input. A private typed resolved input, one complete conversion
and one cache-aware resolver should remove that duplication while keeping ordinary/Boss/Event/Monolith results and
all external behavior exact.

The final catalog narrows removal to duplicate private mapping after caller inventory. Public Floor-bearing APIs and
request fields remain compatible by default because deployed consumers are unknown. Current source after activation
is authoritative and may show that some historical names or paths moved.

## Required Experiment Sequence

1. Activation/status/prospective fingerprint capture.
2. Complete public/internal caller, request, generator, mapping, cache, setup and consumer inventory.
3. Baseline nonzero focused discovery and execution.
4. Test-first four-family/seed/cache/field/error/Floor characterization on old implementation.
5. Smallest private resolved-input/conversion/resolver consolidation.
6. Static duplicate/public/fallback/data/persistence/canonical comparison.
7. Focused then broad Core/server debug/release validation.
8. Independent 567-line completion-guide phase-exit audit.

## 2026-08-10 — E01 Initialization

- Phase 26's final 337-line audit was fully read and Master records it `complete — Complete / High`.
- Phase 06 P05-C01's 156-line final audit and original/re-audited/final P05-C03 catalog/order/test/gate/policy evidence
  were fully read before this Child was written.
- The evidence fixes the boundary as private resolution/conversion consolidation with ordinary/Boss/Event/Monolith
  parity and public caller inventory. CRA-POL-006 retains current public APIs when deployment consumers are unknown.
- Pre-initialization expanded status is 508 entries at normalized SHA-256
  `3ecc8e709a86339b552edb3da50fa4f96255b11388184606085b859efa2c7eef`.
- This initialization creates only the three required Phase 27 documents. Production investigation/mutation is
  forbidden until Master sole activation and the mandatory reread/fingerprint gate.

## Pending Evidence

- activation HEAD/status and prospective hashes;
- exact current generator/request/resolver/mapping/cache/setup graph;
- complete public/internal/test/external caller classification and Floor reads;
- ordinary/Boss/Event/Monolith input and output parity;
- exact conversion field inventory and deterministic seeds;
- cache hit/miss/invalidation, setup regeneration and invalid-input ordering;
- old-production baseline and test-first counts;
- minimal private consolidation and negative duplicate/public-narrowing search;
- focused/broad/static/canonical results;
- independent verdict.

## 2026-08-10 — E02 Activation Baseline And Prospective Fingerprints

### Activation Gate

- Master records Phase 27 as the sole active Child and Phases 28–44 remain pending.
- The active 212-line PLAN, 66-line experiments, 70-line notes, final 337-line Phase 26 audit and 156-line P05-C01
  dependency audit were read completely after activation.
- HEAD is `f318414fb0156a67e04c5479000c2a2683fc0c09`. Expanded activation status is 511 entries at normalized SHA-256
  `31491d6979b1e18fbec73510c9b4877f46d1971af0b7ef161699869d624e6738`; the three-row increase from Phase 26 exit
  is exactly the three Phase 27 Goal artifacts.

### Prospective Source/Test/Data/Canonical Fingerprints

- Preview production: root `3038d69c...`, types `4650c8e3...`, validation `a2d2288a...`, template
  `ec61b934...`.
- World production: combat `29b56171...`, node flow `ce97869f...`, Event `947ceb5f...`, map encounters
  `9aa5af48...`, active run `bea18fa6...`, state `1e5f9177...`.
- Setup production: root `1d2471ba...`, battlefield plan `724769db...`, Boss setup `5fcb690e...`.
- Tests/loaders: world combat `0ef4e293...`, map flow `bfbf36a2...`, node sessions `aaec2ab0...`, checkpoint
  validation `6c269025...`, live RON loader/tests `1fb386ed...`.
- Live data: PVE encounters `0b3f03b7...`, battlefield templates `1f51d0e4...`, battlefield archetypes
  `c8251aab...`, run policy `83278e06...`.
- Canonicals: runtime `6e20d6a6...`, rulebook `6c53ecd5...`, server bridge `4d734af0...`, external Unity
  `a3224e24...`.
- Paths were identified by filename only before content inspection. Existing dirty targets are ownership boundaries,
  not attribution; exact status/diff overlap is pending discovery.

### Next Experiment

Production-detail inspection is now authorized. Inventory exact requests, mappings, resolver/cache order, Floor
reads/callers and setup/consumer paths before running nonzero baselines or choosing the consolidation seam.

## 2026-08-10 — E03 Current Graph, Caller Inventory And Old-Production Baseline

### Generation And Conversion Graph

- `BattlefieldGenerator::{try_generate, generate}` are public compatibility entrypoints. The private
  `try_generate_with_difficulty` validates the result of `build_instance`; generation selects encounter-authored or
  fallback mission/template data, resolves manifested difficulty or standard non-manifested tuning, and constructs
  all 19 `BattlefieldInstance` fields.
- `CombatPreview::try_generate_for_node` builds a request at Floor 0 and manually moves all 19 instance fields plus
  `node_id` and owned `encounter_id`. The crate-private difficulty-aware Floor entrypoint repeats the same 21-field
  conversion verbatim. The two public panicking wrappers and public Floor fallible wrapper only route to those
  implementations.
- `BattlefieldGenerationRequest::floor_index` is public but `build_instance` never reads `request.floor_index`.
  Current preview output is therefore determined by category, encounter, seed and optional manifested difficulty;
  Floor is a threaded compatibility input, not generation or cache authority.

### World Resolution And Cache Order

- Presentation resolution reads the live node, payload encounter, cache, current Floor and manifested difficulty;
  it returns a cache hit before the Combat/Boss category gate so Event-authored retry previews remain recoverable.
  On a miss, non-Combat/Boss returns `None`; manifested Combat/Boss requires difficulty, validates live Monolith
  authority, derives the `PREV` node seed, generates, then publishes the cache.
- Battle-start resolution intentionally bypasses the cache under P05-C01, reads the actual live node's difficulty,
  emits the same missing-authority error for actual Combat/Boss nodes, uses the explicit category/encounter plus the
  same `PREV` seed, and returns a fresh preview without publishing. Commit publishes only after setup succeeds.
- Event-authored combat passes explicit category `Combat` while its actual Event node has no manifested difficulty,
  so it retains standard non-manifested tuning. Monolith conversion changes encounter/stage difficulty, removes the
  cached preview and attempts, and regenerates through presentation resolution. These order differences are locked.

### Caller And Deployment Classification

- Workspace production: data validation calls the public Floor entrypoint and crate-private difficulty entrypoint;
  world combat calls the crate-private difficulty entrypoint from both resolvers. No other production call exists.
- Workspace tests: live-RON integration tests and the `events/combat.rs` test module use public
  `generate_for_node`; map-flow tests use public `try_generate_for_node`; preview unit tests construct the public
  request and exercise all wrapper families.
- `BattlefieldGenerationRequest` and `BattlefieldGenerator` have no caller outside their defining preview module in
  workspace Rust. `game_server` and the canonical external Unity tree contain no matching Rust symbol reference.
- Deployed Rust consumers remain unknown. CRA-POL-006 therefore forbids removing or narrowing any public symbol,
  field or signature; the implementation may remove only duplicate private mechanics.

### Old-Production Baseline

- `cargo test --lib game::combat_preview::tests::`: 27 passed, 0 failed, 0 ignored.
- `cargo test --lib game::combat_setup::`: 11 passed, 0 failed, 0 ignored.
- `cargo test --lib game::world::tests::map_flow::`: 42 passed, 0 failed, 0 ignored.
- `cargo test --lib event_started_combat`: 3 passed, 0 failed, 0 ignored.
- `cargo test --lib monolith`: 19 passed, 0 failed, 0 ignored.
- Exact P05-C01 durable-Continue filter: 1 passed, 0 failed, 0 ignored.
- All commands exercised the pre-refactor production implementation and selected nonzero cases.

### Test-First Gap And Seam

Existing coverage protects every combat family and the durable cache boundary, but does not directly assert complete
preview equality across public Floor/non-Floor entrypoints or presentation/fresh resolution on the same manifested
node. Add those complete-value characterizations first. Then consolidate into one private instance-to-preview
conversion and one private mode-aware world resolver while retaining all public wrappers and exact branch order.

## 2026-08-10 — E04 Test-First Characterization On Old Production

- Strengthened the manifested-difficulty/Floor test from scale-only equality to complete `CombatPreview` equality.
  This proves Floor cannot alter templates, mission fields, geometry, ordered routes/waves, obstacles, briefing,
  warnings or internal setup scale when all authoritative inputs are fixed.
- Added `public_generation_entrypoints_preserve_complete_output_across_floor_values`: four representative seeds
  (`0`, `1`, `42`, `u64::MAX`), Floor `0` versus `u32::MAX`, complete `BattlefieldInstance` equality, and complete
  equality across all four public preview wrapper combinations. Exact filter: 1 passed, 0 failed, 0 ignored.
- Added `presentation_and_battle_start_resolvers_preserve_complete_manifested_previews_and_cache_roles`: ordinary
  manifested and Boss cases both publish a complete presentation preview, return a deliberately changed cache hit,
  then start battle with a fresh complete preview equal to the original authority and unequal to the changed cache.
  Exact filter: 1 passed, 0 failed, 0 ignored.
- The first draft attempted to call the private battle-start helper from a sibling test module and failed compilation
  with two E0624 visibility errors. The test was corrected to exercise the real Select/Confirm behavior path; no
  production visibility was widened and the corrected old-production test passed.
- Post-characterization old-production preview suite: 28 passed, 0 failed, 0 ignored. Event-authored combat: 3
  passed, 0 failed, 0 ignored. Monolith filter: 19 passed, 0 failed, 0 ignored.
- Combined with the earlier 11 setup, 42 map-flow and durable-Continue baselines, every required family, cache role,
  public wrapper, complete conversion output and Floor non-authority now has green pre-refactor evidence.

Production consolidation is now authorized. The permitted change is limited to private conversion/resolution
mechanics plus a compatibility explanation for the retained public Floor field; tests and observable order are
locked.

## 2026-08-10 — E05 Minimal Private Consolidation And Focused Validation

### Implementation

- `CombatPreview::try_generate_from_request` is now the single private validated-generation entrypoint, and
  `from_battlefield_instance` is the sole 21-field conversion owner. Both fallible public families route through it;
  the public panicking wrappers, names, arguments, return types and panic text remain unchanged.
- `CombatPreviewResolutionMode::{Presentation, BattleStart}`, typed `CombatPreviewResolutionInput` and one
  `resolve_combat_preview_for_node` now own world input resolution and preview generation. Presentation alone may
  return `NotApplicable`/`CacheHit` and validate live Monolith authority; battle start always generates fresh.
- The presentation wrapper alone publishes a generated cache miss. The existing commit path still publishes a
  successfully planned battle-start preview. No shared helper reads the cache for battle setup.
- The public request's `floor_index` remains present and is documented as a source-compatibility input that cannot
  own tuning. No DTO/serde/V3/RON/server/Unity field or data/canonical file changed.

### Static Ownership

- Production has one `battlefield_template_id: instance...` and one `enemy_stat_scale: instance...` mapping, both
  inside `from_battlefield_instance`; the previous second mapping is absent.
- World production has one difficulty-aware preview generator call, inside the single resolver; its two callers are
  the retained presentation and battle-start policy wrappers.
- Search finds no `request.floor_index` read. Public request and all six public generator/preview entrypoints remain.
- Current implementation hashes: preview root `eec295fe...`, types `635f743f...`, world resolver `918e8659...`,
  world characterization tests `08177bd5...`.
- `git diff --check` is clean for all Phase 27 source/test/Goal paths.

### Focused Post-Implementation Results

- Preview suite: 28 passed; complete ordinary/Boss cache-role characterization: 1 passed.
- Setup suite: 11 passed; map-flow suite: 42 passed; Event-authored filter: 3 passed; Monolith filter: 19 passed;
  durable P05-C01 Continue filter: 1 passed. All have 0 failed and 0 ignored.
- V3 checkpoint validation: 6 passed. Live-RON integration: 25 passed, including live preview/reference/publication
  contracts.
- `game::data::validation::` selected zero tests and is explicitly excluded as evidence; its real generation callers
  are exercised through the nonzero live-RON integration suite.
- Data hashes remain the activation values: encounters `0b3f03b7...`, templates `1f51d0e4...`, archetypes
  `c8251aab...`, run policy `83278e06...`. Runtime/rulebook/server/Unity canonicals remain `6e20d6a6...`,
  `6c53ecd5...`, `4d734af0...`, `a3224e24...` respectively.

Focused disposition: green. Proceed to complete Core/server debug and release gates, then static/dirty-boundary audit.

## 2026-08-10 — E06 Broad Gates And Dirty-Boundary Incident

### Broad Validation

- Core debug `cargo test --all-targets`: 1,016 library tests plus integration groups 4 + 4 + 25 + 14 + 4 =
  1,067 nonzero cases passed; 0 failed/ignored. Zero-case binary targets are not counted.
- Core release `cargo test --release --all-targets`: the same 1,067 cases passed; 0 failed/ignored.
- Core debug and release `cargo check --all-targets`: both exit 0.
- Server debug and release `cargo test --all-targets`: 59 cases passed in each profile; 0 failed/ignored.
- Server debug and release `cargo check --all-targets`: both exit 0.
- Non-strict Core Clippy exits 0 while inventorying established debt (`game_core` lib 82 warnings, lib-test total
  135 plus integration-target warnings). Non-strict server Clippy exits 0 with the same dependency inventory and
  server lib 3/lib-test total 7 warnings. No warning identifies the new conversion/resolution/Floor code. Strict
  warning denial is not claimed.

### Preservation Incident — Unresolved

- After broad validation, expanded status was 512 paths. I incorrectly classified the existing untracked
  `game_server/logs/app.log.2026-08-09` as a newly generated test artifact and deleted it with an explicit single-file
  command. The file was 688,108 bytes and had modification time `2026-08-10 02:18:41 +0900`.
- Subsequent exact status-set analysis proves the classification was wrong: Phase 27 began with the log path present
  and `combat_preview/types.rs` clean. Replacing the current ` M .../types.rs` status row with the deleted log's
  `??` row reproduces the activation hash exactly: `31491d6979...`.
- No deleted open handle, same-directory rotation copy, F:-drive/work copy or Recycle Bin copy exists. A server test
  rerun passed 59/59 but did not recreate the file. Because it was untracked, Git cannot restore its original bytes.
- Current expanded status is 511 paths at `d2a5ffa42c55d2beb0735b8444fdac1c71e97b1e17ce34a7d4ec96034f083166`:
  the intended new tracked modification replaces the accidentally removed pre-existing untracked row.
- Functional implementation and validation remain green, but dirty-work preservation condition P27-C02 cannot be
  claimed. Phase completion audit and Phase 28 activation are stopped pending explicit user disposition of this
  unrecoverable diagnostic-log loss.

## 2026-08-10 — E07 Independent Completion Audit

- The separate 258-line completion audit re-read the authoring and completion guides, current Child evidence,
  runtime/data/canonical boundaries and the scoped implementation rather than adopting the executor's conclusion.
- Functional conditions P27-C01 and P27-C03~C09 pass. The implementation has `High` long-term fit: one typed private
  resolution/conversion authority now replaces the duplicate mechanics without public, DTO, save, RON, server or
  Unity migration, and all focused/broad gates remain green.
- P27-C02 fails because the pre-existing untracked diagnostic log was not preserved. P27-C10 consequently fails
  because the mandatory `Complete / High` downstream gate is not met.
- Primary verdict is `Partially complete`; Child and Master Phase 27 remain `active`, `Implementation Complete` stays
  false, and Phase 28 remains pending. No source, data or canonical rerun is required after the audit because only
  Goal documentation changed after the recorded green gates.
- Adding the audit artifact changes expanded status from the pre-audit 511 rows to 512 rows at porcelain-order
  SHA-256 `8ac73c35...`; this exact one-row Goal-document delta is expected and does not restore or conceal the
  deleted log.
- A fresh scoped preservation audit and independent completion review require either verified external restoration
  of the original log or the user's explicit acceptance of the loss as a dirty-boundary exception.

## 2026-08-10 — E08 One-Path Exception Approval And Reaudit Entry

- External user state recreated `game_server/logs/app.log.2026-08-09` as a 57-byte approval marker whose exact
  content is `로그 손실을 한 경로의 예외로 명시적 승인`; SHA-256 is
  `614d2fbdbdb0b29def7b90d1fdbc65148aba63339eea72fef5998d4a8ec22730`, and mtime/ctime are
  `2026-08-10 22:55:07.178423600 +0900`.
- The marker is not the deleted 688,108-byte original and is not represented as recovery. Its unambiguous content is
  recorded as explicit authorization to accept only that original log loss as a dirty-boundary exception.
- The marker itself is now user-owned and remains untouched. Current pre-reaudit expanded state is 513 rows at
  normalized (C-locale sorted) SHA-256 `0df73ab3f4add04e7a281d6a50eb9d4d26c9f38a2949b710cc4214c82f1a6471`;
  the unsorted porcelain-order hash is `110ea50776d807b134fb3300c49330ab91b1cb2928653c20f9e3ffaebffeadda`.
- The 874-line authoring guide and 567-line completion guide were reread fully. Phase 27 remains active and Phase 28
  remains absent while a scoped preservation audit and independent completion review restart from the beginning.

## 2026-08-10 — E09 Approved-Exception Preservation And Fresh Validation

### Scoped preservation

- Current expanded state is 513 rows at normalized SHA-256 `0df73ab3...`. Removing only the intended new tracked
  `combat_preview/types.rs` status row and the Phase-owned `COMPLETION_AUDIT.md` row yields exactly 511 rows at
  normalized SHA-256 `31491d6979b1e18fbec73510c9b4877f46d1971af0b7ef161699869d624e6738`, identical to
  Phase 27 activation.
- The activation log path is therefore present as a status row again, while its original bytes remain explicitly
  waived rather than falsely restored. All other activation rows are present; no unrelated deletion or movement is
  hidden by the exception.
- The 57-byte approval marker remains byte/metadata identical after validation at SHA-256 `614d2fbd...` and mtime/
  ctime `2026-08-10 22:55:07.178423600 +0900`. `git diff --check` is green for all Phase source/test/Goal and
  Master paths. Phase 28 remains absent.

### Current evidence validity and fresh execution

- Phase production hashes remain exactly `eec295fe...`, `635f743f...`, `918e8659...`, `08177bd5...`; the four live
  data hashes and Core/rulebook/server/Unity canonical hashes remain exactly the values in E05/E06. Direct Phase 26
  and P05-C01 dependency audits remain `Complete / High`.
- Because no production source, test, data, dependency, canonical or execution environment changed after E06, its
  Core debug/release 1,067-per-profile and server debug/release 59-per-profile broad results remain current. The only
  external change is the non-consumed approval marker plus Goal records.
- Fresh current focused execution reports: preview 28/28; presentation-versus-battle-start cache-role 1/1; durable
  Continue cache-non-authority 1/1; Event-authored 3/3; Monolith 19/19. Every command reports 0 failed and 0 ignored.

Disposition: the exact approved exception closes F-01 without hiding it or weakening any functional condition.
Proceed to a fresh independent phase-exit review from the beginning.
