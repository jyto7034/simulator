# Phase 26 CRA-P04-C04 Experiments

Status: complete
Implementation Complete: true
Candidate: CRA-P04-C04
Master Phase: 26 of 44
Date: 2026-08-10 (`Asia/Seoul`)

## Experiment Rules

- No production source, test implementation, live data or canonical content is inspected before sole activation and
  the complete activation protocol.
- Every command records selected test count; zero selected cases are not evidence.
- Add Trigger/Monolith assembly, UUID, queue-order and active-noninterruption characterization before refactoring.
- Preserve exact missing-definition/error order, trigger priority, Monolith accept/decline and checkpoint projection.
- Share private assembly while keeping Tail/FrontOfQueue an explicit caller choice.
- Preserve unrelated dirty work; no reset, restore, clean, stash or broad formatting.
- Phase 27 work is forbidden.

## Historical Hypothesis To Verify

Trigger evaluation and Monolith exchange independently look up a VN definition, convert presentation fields, allocate
a UUID, choose the entry scene, initialize blocking/source/choice state and enqueue the resulting session. Trigger
appends at the tail; Monolith inserts at the front. Both leave any active VN uninterrupted.

The refactor hypothesis is that one private assembler plus an explicit placement enum can remove the duplicated
construction and raw placement without changing UUID timing, result/error order, active/pending order or any public/
serialized representation. Current source is authoritative and may refine this hypothesis after activation.

## Required Experiment Sequence

1. Activation/status/prospective-fingerprint capture.
2. Complete producer, definition, UUID, runtime queue, projection and consumer inventory.
3. Baseline nonzero focused test discovery and runs.
4. Test-first Trigger/Monolith/order/noninterruption/projection matrix on the old implementation.
5. Smallest private shared assembly and explicit placement change.
6. Static duplicate/fallback/public/persistence/canonical comparison.
7. Focused then broad Core/server debug/release validation.
8. Independent 567-line completion-guide phase-exit audit.

## 2026-08-10 — E01 Initialization

- Phase 25's final 371-line audit was fully read and Master records it `complete — Complete / High`.
- Original P04-C04, re-audited and final candidate/order/test/dependency evidence was read before this Child was
  written. The evidence consistently requires shared construction with caller-explicit Tail/FrontOfQueue placement.
- Pre-initialization expanded status is 503 entries at normalized SHA-256
  `aadd6b3327bbda94719be9660aad2e3a7028e7126f30fa79945f1b2069984e0c`.
- This initialization creates only the three required Phase 26 Goal documents. Production inspection/mutation is
  forbidden until Master sole activation and the full Child/predecessor reread.

## Pending Evidence

- activation HEAD/status and prospective hashes;
- exact Trigger and Monolith producer/validation/mutation order;
- exact runtime-session field conversion, UUID allocation and definition lookup;
- exact active/pending queue mutation and priority behavior;
- exact checkpoint/public/serde/server/Unity consumers;
- baseline and test-first counts;
- implemented private assembler and explicit placement, with negative legacy search;
- focused/broad/static/canonical results;
- independent verdict.

## 2026-08-10 — E02 Activation Baseline And Prospective Fingerprints

### Activation Gate

- Master registers Phase 26 as the sole active Child and Phases 27–44 remain pending.
- The active 196-line PLAN, 61-line experiments, 71-line notes and final 371-line Phase 25 audit were read completely
  after activation.
- HEAD is `f318414fb0156a67e04c5479000c2a2683fc0c09`. Expanded activation status is 506 entries at normalized SHA-256
  `690ad9bec5c6bef0ee126405a7710309c301573c53b162ad5411b8e99046e466`; the three-row increase from readiness is
  exactly the three Phase 26 Goal artifacts.

### Prospective Source/Test/Data/Canonical Fingerprints

- Production: VN runtime `bd9c8355...`, VN flow `0f36ba21...`, VN triggers `52a24ae6...`, Monolith
  `b9daa6ea...`, facility `35ce3dfe...`, world root `c533e0ff...`, state `1e5f9177...`, active run
  `bea18fa6...`, snapshot `6e512c4d...`, VN data `a7a3d58a...`, trigger data `fdbb00d4...`.
- Tests: facility `3e13a4c0...`, map flow `bfbf36a2...`, node sessions `541eac53...`, snapshots/start
  `2144e86e...`, checkpoint validation `6c269025...`.
- Live data: triggers `4216706a...`, VNs `0c37bba3...`, facility templates `9e213ea4...`, run policy
  `83278e06...`.
- Canonicals: runtime `6e20d6a6...`, rulebook `6c53ecd5...`, server bridge `4d734af0...`, external Unity
  `a3224e24...`.
- Path discovery found current ownership candidates as `game/visual_novel.rs`,
  `world/visual_novel_flow.rs`, `world/visual_novel_triggers.rs` and `game/monolith.rs`; no production content was
  read before these hashes were captured. Most targets are already modified in the shared dirty tree, so hashes are
  ownership boundaries rather than attribution.

### Next Experiment

Production-detail inspection is now authorized. Inventory the exact producer/consumer/UUID/queue/projection graph,
then run nonzero baseline filters before adding characterization.

## 2026-08-10 — E03 Current Owner Graph And Baseline

### Assembly, Placement And Consumer Graph

- Production has exactly two direct `VisualNovelSession` constructors. Trigger evaluation resolves its definition,
  allocates the next VN UUID while constructing the seven-field session, activates when idle or appends to Tail when
  active, then marks repeat consumption. Matches are priority-descending and ID-ascending.
- Monolith Shop entry first proves exchange availability and resolves the policy VN definition/presentation/entry
  scene, changes exchange state `Available→Started`, then allocates the next VN UUID and constructs the same seven
  fields with `Map`, blocking true. It activates when idle or inserts at pending index zero when active.
- Both paths therefore share definition-to-runtime conversion, identity allocation and active-or-queue mechanics;
  their error wording, source/blocking inputs and explicit Tail/FrontOfQueue placement differ and must remain local.
- `visual_novel_flow.rs` owns scene/choice progression and FIFO promotion. `snapshot.rs` owns active-session DTO
  projection. `GameCoreState` owns active/pending runtime and UUID state; run checkpoints preserve the monotonic VN
  UUID counter and trigger consumption but deliberately do not rewind the top-level active/pending overlay.
- Monolith accept/decline is committed only while the active session's VN ID equals the policy exchange VN. The
  Monolith state machine, Shop transition, authored data, public DTO/server and Unity surfaces are consumers, not
  assembly owners.

### Nonzero Old-Implementation Baseline

- Facility module: 11 passed, 0 failed/ignored; includes Trigger activation, priority/Tail promotion and checkpoint
  UUID non-reuse.
- Snapshots/start module: 23 passed, 0 failed/ignored; includes session field/DTO projection, blocking, choices,
  run-start source and trigger checkpoint history.
- Monolith state module: 6 passed, 0 failed/ignored; includes exchange accept state and one-time residual consumption.
- Map-flow module: 42 passed, 0 failed/ignored; covers adjacent node entry/progression and Boss trigger consumers.
- Broad `visual_novel` name filter independently selected 26 and all passed; it overlaps the module counts above and
  is recorded as discovery evidence, not added to a unique-case total.
- Current gap: no world integration test drives Monolith exchange session start with an existing active/pending VN,
  and Trigger tests do not assert every constructed field plus exact UUID order in one populated-queue matrix.

### Next Experiment

Add behavior-observing Trigger and Monolith characterization on the unchanged production implementation, including
full session fields, exact UUID sequence, active equality, pending order and accept/decline consequences.

## 2026-08-10 — E04 Test-First Characterization On Old Production

- Added three behavior-observing world tests before any production edit. The populated Trigger case proves existing
  active equality, existing pending retention, priority-ordered Tail append, all seven session fields and exactly two
  UUID allocations. It passed 1/1 on the duplicated implementation.
- The populated Monolith case proves Shop entry leaves active VN exact, places the exchange session before two older
  pending sessions, preserves all seven fields, consumes exactly one UUID, changes exchange to `Started`, then
  promotes the exchange and commits exact accept consequences. It passed under the old implementation.
- The idle Monolith case proves the exchange becomes active rather than pending and decline preserves all residuals,
  leaves the corrupted key absent and closes the overlay. Together the two Monolith cases passed 2/2.
- The complete node-session module passed 28/28, zero failed/ignored, on unchanged production. Direct constructor
  fixtures remain test-only behavior inputs and do not assert any future factory/placement symbol.

### Next Experiment

Introduce a private resolved-definition seed plus one assembly authority and explicit Tail/FrontOfQueue placement,
preserving Monolith lookup-before-state-change and state-change-before-UUID order.

## 2026-08-10 — E05 Minimal Implementation And Focused Validation

- `visual_novel_flow.rs` now owns a private resolved-definition seed, the sole production
  `VisualNovelSession` allocator/constructor and one explicit placement method. `Tail` and `FrontOfQueue` are the
  complete typed placement vocabulary; idle placement still activates directly.
- Trigger retains priority matching, source/blocking ownership, missing-reference text and consumed-after-enqueue
  order. It resolves and assembles, then explicitly requests `Tail`.
- Monolith retains availability/definition lookup, exact missing-reference text, `Started` commit before UUID
  allocation and Shop/result order. It assembles only after that commit, then explicitly requests `FrontOfQueue`.
- Static search finds one production constructor, one queue `push` and one index-zero `insert`, all in the common
  owner; the two producers contain no raw construction or raw queue mutation.
- Focused post-change modules pass: node sessions 28, facility 11, snapshots/start 23, map flow 42 and Monolith state
  6, for 110 non-overlapping selected tests, zero failed/ignored. The overlapping broad `visual_novel` filter also
  passes 26/26. Debug all-target check, formatter and repository diff check pass.
- Current public/session/serde types, live RON and runtime/rulebook/server/Unity canonicals require no edit; current
  wording already specifies top-level active projection, Trigger ordering/noninterruption, blocking ownership and
  opaque server forwarding.

### Next Experiment

Run complete Core/server debug and release suites/checks, final hashes/static compatibility inventory and independent
phase-exit audit.

## 2026-08-10 — E06 Broad Gates, Static Closure And Review Handoff

### Complete Validation

- Focused post-change evidence remains green: node sessions 28, facility 11, snapshots/start 23, map flow 42 and
  Monolith state 6, totaling 110 non-overlapping passed tests with zero failed/ignored. The overlapping
  `visual_novel` discovery filter passes 26/26.
- Core complete debug and release test suites each pass 1,014 library tests plus official integrations
  `4 + 4 + 25 + 14 + 4 = 51`, with zero failures. Core debug and release all-target checks also pass.
- Server complete debug and release suites each pass 59 tests with zero failures. Server debug and release
  all-target checks pass.
- Core and server `cargo fmt --all -- --check` pass, as does repository-wide `git diff --check`.
- Non-strict Clippy succeeds. Core retains the established 82 library / 135 library-test warnings; server retains
  3 library / 7 library-test warnings. No warning points to the four Phase 26 implementation/test edits.

### Static And Compatibility Closure

- Production search has exactly one `VisualNovelSession { ... }` construction, in the common flow owner. The public
  struct declaration is not a constructor, and test-only fixtures are outside the production-ownership target.
- Raw pending `push` and index-zero `insert` each occur once, only in the common placement method. Trigger has two
  explicit `Tail` calls for its consumed/non-consumed paths; Monolith has one explicit `FrontOfQueue` call.
- Final scoped hashes: flow `6c5877f9...`, trigger producer `ae14bc09...`, Monolith/map producer `469cc1fe...`, and
  node-session tests `aaec2ab0...`. Adjacent public session, state and snapshot files remain activation-identical at
  `bd9c8355...`, `1e5f9177...` and `6e512c4d...`.
- Live trigger/VN/facility/run-policy data remain activation-identical at `4216706a...`, `0c37bba3...`,
  `9e213ea4...`, `83278e06...`. Runtime/rulebook/server/Unity canonicals remain activation-identical at
  `6e20d6a6...`, `6c53ecd5...`, `4d734af0...`, `a3224e24...`.
- Expanded status before the audit artifact is 507 paths at normalized SHA-256
  `e1bdaf3dbe52d73295bcfc0c3f616f8e170c6adde9ad92dd8d07a977b94a6e83`. The one path beyond activation's three
  Child artifacts is the previously clean Trigger source deliberately modified in this Phase; all other target
  paths were already dirty and remain preserved.
- No Phase 27 Goal directory or implementation symbol exists. Phase 26 is now `review_pending`; implementation
  completion remains false until the mandatory independent audit returns `Complete / High`.

### Review Handoff

Fully reread the 874-line authoring/execution guide, the 567-line completion-review guide, this complete Child and
the exact changed/consumer/canonical evidence. Create `COMPLETION_AUDIT.md`; do not activate Phase 27 unless the
verdict is `Complete / High` and all audit findings are closed.

## 2026-08-10 — E07 Independent Completion Audit And Exit

- The 874-line authoring/execution guide, 567-line completion-review guide and final Child documents (196/211/138)
  were fully reread. Current source, all three new tests, live loader/validation, projection/checkpoint, server and
  canonical evidence were independently rechecked rather than treating E01–E06 as proof.
- The audit independently reran node sessions 28/28, facility 11/11 and snapshots/start 23/23. Current hashes,
  one-constructor/two-placement static results, live data, public types, server transport and all broad results remain
  valid.
- The 337-line `COMPLETION_AUDIT.md` finds no material issue and returns `Complete / High`, improvement class `none`.
  All P26-C01–P26-C10 rows pass.
- The audit artifact brings expanded status to 508 paths at normalized SHA-256
  `3ecc8e709a86339b552edb3da50fa4f96255b11388184606085b859efa2c7eef`; repository diff validation remains
  green.
- Phase 26 transitions to `complete`, `Implementation Complete: true`. Master may record Phase 26 complete and make
  Phase 27 ready; Phase 27 production work still requires its separate initialization and sole-active protocol.
