# Phase 18 CRA-P09-C04 Completion Audit

Audit Mode: phase_exit
Audit Target: Phase 18 of 44 — CRA-P09-C04 Project Effect Preview From Delivery Footprints
Current Status: review_pending
Implementation Complete Claim: false pending this audit
Policy Source: CRA-POL-005 and cross-cutting CRA-POL-006
Goal Documents: `PLAN.md`, `EXPERIMENTS.md`, `EXPERIMENT_NOTES.md`
Repository/Commit/Dirty State: `/mnt/f/work/simulator`; HEAD
`f318414fb0156a67e04c5479000c2a2683fc0c09`; expanded dirty status 460 entries with normalized
SHA-256 `c2ddedad20fa31a1969c626f7d863107dc1ef63e94ba667c7a57e6b6b259dd58`
Audit Date: 2026-08-10 (`Asia/Seoul`)

## Completion Evidence Matrix

| ID | Completion condition or invariant | Policy basis | Runtime/data/contract evidence | Validation evidence | Verdict |
| --- | --- | --- | --- | --- | --- |
| P18-A01 | The complete preview producer/consumer graph is identified | CRA-POL-005; completion guide §§5, 8 | deployed checkpoint and pending-deployment all-facing producers converge on `range_preview.rs`; Core DTO, server serde, Unity parser/mock and canonical are inventoried | static caller/field search plus server 1/1 and read-only Unity inspection | pass |
| P18-A02 | Cast acquisition reach never substitutes for effect footprint | CRA-POL-005 | SelfTarget now obtains effect cells only from `DeliveryDef::TileArea`; manual unit/tile retain their separate `cast_cells`; automatic unit remains runtime-target dependent | initial three intended failures; focused 6/6; range family 14/14 | pass |
| P18-A03 | Every statically projectable spatial step participates in one union | CRA-POL-005 | one crate-private helper iterates every step, admits only TileArea and resolves caster or reusable self cast-target anchors | differing two-step footprints in both authored orders and runtime parity integration | pass |
| P18-A04 | Runtime geometry, facing and valid-board clipping are reused | CRA-POL-005; Phase 16 seam | preview and runtime both call `TileReachQuery` with authored footprint, resolved anchor tile, caster facing and battlefield valid tiles | center all four facings, four clipped corner/facing cases and explicit 3x3/5x5 corner case | pass |
| P18-A05 | Union output is deduplicated and canonically ordered | CRA-POL-005 | cells are combined before one `(y, x)` sort and dedup; step order is not an authority | overlap/reverse-order exact-vector assertions and runtime event union comparison | pass |
| P18-A06 | Nonspatial delivery adds no false geometry and unknown future geometry is not fabricated | CRA-POL-005 | Instant/Projectile are skipped; retargeted cast-target/impact-context areas return `RequiresRuntimeTarget` | nonspatial-only empty-array and runtime-dependent-unavailable focused tests | pass |
| P18-A07 | Preview is observational and runtime delivery is unchanged | CRA-POL-005; CRA-POL-006 | helper only reads skill/layout/position/facing; runtime area source retains activation hash | exact `transaction_debug_snapshot` before/after preview and actual `SkillAreaDeclared.affected_tiles` parity | pass |
| P18-A08 | Public DTO, server/wire, Unity and persistence shapes remain exact | CRA-POL-006 | the existing one `Vec<Position>` effect field and adjacent cast field are unchanged; server serializes the Core DTO; Unity consumes existing `*_cells`; no persisted authority was added | DTO hash unchanged, server JSON 1/1, server debug/release 59/59, Unity hashes recorded | pass |
| P18-A09 | Live data, canonical and adjacent Phase boundaries match the implementation | CRA-POL-005; CRA-POL-006 | Skill RON is byte-identical; canonical states union/nonspatial/runtime-dependent semantics; Phase 19 remains pending and Phase 34 retains later DTO ownership | RON debug/release 25/25, live suites, canonical/hash and Master registry inspection | pass |
| P18-A10 | Existing dirty work is preserved and validation is broad/current | Goal invariant; completion guide §§10–11 | activation/current status and scoped hashes distinguish Phase edits; external Unity stayed read-only; no destructive normalization | Core debug/release 992/992, affected integrations, server debug/release 59/59, workspace/fmt/diff checks | pass |

## Approved Policy Restatement

CRA-POL-005 keeps the existing single public `effect_preview_cells` collection. Its content is the deterministic,
deduplicated union of every statically knowable TileArea delivery footprint for the previewed cast, using the same
typed geometry, facing transform and valid-board clipping as runtime. Cast acquisition cells remain separate and
cannot stand in for delivery geometry. Instant and Projectile deliveries without a TileArea add no cells. The
policy does not approve per-step carriers or a DTO/server/Unity migration.

CRA-POL-006 therefore freezes field names, types, tags, nesting, nullability and transport/persistence boundaries.
The Phase is a content correction inside the existing carrier, not a schema or rollout change.

## Runtime And State Evidence

### Producer and selection graph

- `range_previews_for_player_runtime_unit` builds preview state for a deployed player runtime unit.
- `range_previews_for_combat_profile` builds the same preview for each facing of a pending deployment.
- Both select the active skill from authoritative game data and enter the same cast-selection projection.
- Manual unit/tile preview continues to materialize only cast acquisition cells. Automatic-unit preview continues
  to return `RequiresRuntimeTarget`. The Phase changes only the SelfTarget effect-content producer.
- SelfTarget still validates that the authored cast query can be formed. Missing data remains
  `MissingRangeData`, while effect geometry no longer reuses or depends on the materialized acquisition-cell list.

### Effect footprint authority

- `self_target_effect_preview_cells` is the only new aggregation authority.
- It iterates every authored step and accepts only `DeliveryDef::TileArea { footprint, area }`.
- Caster anchors resolve to the preview caster tile. Reused SelfUnit/CastTarget anchors in a SelfTarget cast resolve
  to that same tile. Retargeted cast-target and impact-context origins are unavailable rather than guessed.
- Published target/impact-centered footprints are rotation-invariant; caster-centered footprints use the provided
  caster facing. When a rotation-invariant footprint lacks facing, both preview and runtime use `Up` as the neutral
  projection orientation.
- Each step calls the Phase 16 `TileReachQuery` and materializes only current battlefield valid tiles. There is no
  copied rotation, clipping, obstacle, occupancy or path adapter.
- The full vector is sorted by `(y, x)` and deduplicated once. If a skill has spatial steps but every cell clips out,
  it reports `NoValidCells`; a skill with no spatial step remains a valid empty effect projection.

### Runtime parity and purity

- Runtime TileArea continues to resolve its geometry in `skill_runtime/area.rs`, calls the same tile query and emits
  the materialized cells in `SkillAreaDeclared.shape.TilePattern.affected_tiles`.
- The integration test executes the real delivery at four center facings and four corner/facing combinations,
  unions the actual emitted affected-tile sets and compares the exact ordered preview vector.
- That integration also compares the complete transaction debug snapshot before and after preview. Unit, cast,
  event, area, projectile and queue state are unchanged.
- Target selection, usefulness, filters, caster inclusion, effects, tracking and release timing are not simulated or
  mutated by preview.

## Data Evidence

- `game_resources/data/skills/base.ron` retains activation SHA-256
  `4087df054a663e20ddab14f184a206d532f8df7b472c656bbd708835298f1770`.
- The four live SelfUnit skills were inspected. Pale Benediction, White Resonance and Root Blessing have statically
  projectable caster/self TileArea footprints; Opening Focus is Instant-only. No live definition requires invented
  retarget or impact geometry.
- Existing publication validation pairs Caster area anchors with Caster `tile_origin`, all target/impact anchors with
  Anchor `tile_origin`, and requires target/impact-centered footprints to be rotation-invariant until an explicit
  orientation policy exists.
- During the audit, a hypothesis that presentation anchor and footprint tile origin could diverge was tested with a
  temporary test-only `CastTarget + Caster tile_origin` definition. Publication rejected it at the exact
  `delivery.TileArea.area.tile_origin` path before preview ran. The temporary test was removed; production returned
  to its recorded final hash. This is evidence that the suspected combination is outside the published input space,
  not a hidden runtime mismatch.
- RON publication remains strict and whole-bundle. No data rewrite, fallback or compatibility alias was introduced.

## DTO/Transport/Persistence Evidence

- `LiveBattleActiveSkillRangePreviewDto` still contains exactly one
  `effect_preview_cells: Vec<Position>` beside `cast_cells: Vec<Position>` with unchanged serde derives.
- Its source retains activation hash
  `65349de8daf5230cd00041b9f3da4b66e76239245e608b98a2623312b2276139`.
- The game server continues to pass the Core deployment-preview result into ordinary serde serialization. No
  production mapper, envelope, tag, setup/update/resync nesting or message order changed.
- The server test now asserts both existing nested arrays explicitly; it does not introduce a golden shape different
  from production.
- Effect preview is derived read-only projection state. No save/checkpoint writer, reader, version, migration or
  synchronized runtime authority was added. Existing deployed checkpoint serialization carries the same DTO shape.

## Consumer Evidence

- Confirmed workspace consumers: deployed-player checkpoint projection, pending-deployment four-facing command,
  Core behavior DTO/serde and game-server result serialization.
- Canonical external consumer: the Unity repository at `/mnt/f/unity projects/ark` was inspected read-only.
  `BattlefieldRangePreviewParser` reads existing `cells` and every `*_cells` array without calculating geometry;
  `ArkMockSnapshots` retains `cast_cells` and `effect_preview_cells` under the same active-skill object.
- Unity's `Available && Cells.Count > 0` check controls whether a range overlay is drawable. It is not an
  authoritative skill-usability decision. An Instant-only empty effect footprint therefore yields no overlay, which
  is consistent with CRA-POL-005 rather than a wire break.
- External hashes remain `96c8a7e96aafbb055a1aecfee0b8d485015660991ad2eaa817aef517928b1fa4`
  for the parser and `f20a48e4493d05561ea5dd9cde6039d7f460b65d5d95815d25d4f6f370549b13` for the mock.
- Inaccessible/unknown consumers: none known beyond the inventoried public serialized contract. No negative search
  was used to remove or narrow a public symbol.
- Explicitly excluded: per-step UI preview, Phase 34/P10-C03 DTO de-duplication and any versioned consumer redesign.

## Legacy/Compatibility Audit

- The former SelfTarget cast-reach-to-effect assignment no longer exists. The only nonempty effect assignment is the
  delivery-aware helper result.
- Manual cast cells remain acquisition cells and never enter the effect collection. Automatic target-dependent
  projection remains unavailable rather than receiving a placeholder.
- There is no parallel old-content fallback, per-step shadow DTO, legacy field alias, dual read/write, version flag,
  ignored test or debug-only projection path.
- Phase 16 geometry and runtime area sources retain activation hashes; Phase 17 actual-delivery semantics are not
  copied into preview as a competing target/effect simulator.
- No public Rust symbol, RON field, V3/save field, event shape or wire envelope was removed or narrowed.

## Scoped Diff And Dirty Preservation

- Activation and final pre-audit expanded status are both 460 entries with normalized SHA-256
  `c2ddedad20fa31a1969c626f7d863107dc1ef63e94ba667c7a57e6b6b259dd58`.
- Activation hashes were recorded before production inspection. Final relevant hashes are range preview
  `1a8fce57...`, Core test module `18fce294...`, server state/test `18f31016...` and canonical `21fd770d...`.
- Unchanged seam hashes are public behavior DTO `65349de8...`, Phase 16 tile query `73930102...`, runtime area
  `23e25cdf...` and live Skill RON `4087df05...`.
- Every Phase target was already represented in the dirty status when the activation baseline was captured, so the
  status path set remains stable while content hashes identify the Phase-owned corrections.
- The audit hypothesis probe changed only a temporary test hunk in `range_preview.rs`; after removal its exact final
  hash returned to `1a8fce573905e0676f6f428121f2a778512eb7e28f44341105528bde6c94e338`.
- No reset, restore, clean, stash, destructive deletion, broad formatting, live-data rewrite or external Unity write
  was used. Existing user/campaign changes and untracked paths remain intact.

## Canonical Alignment

- `core_enhanced/docs/skill_target_contract.md` is the nearest targeting/delivery projection canonical.
- It records the existing one effect collection, union of every known TileArea footprint, shared typed query,
  valid-board clipping, `(y, x)` order/dedup, cast/effect separation, nonspatial exclusion and truthful
  `RequiresRuntimeTarget` behavior.
- It also records the negative boundary: preview does not simulate filters, occupants, obstacles, effects,
  persistent tracking or future motion and does not mutate battle/cast state.
- Public DTO/server/Unity shape and the absence of a per-step carrier are explicit. Current implementation, approved
  policy and canonical therefore describe the same contract.

## Validation Results

All commands ran in the current dirty worktree.

- Baseline before test edits: `cargo test range_preview -- --nocapture` — 9 passed, 0 failed; 977 Core lib tests
  filtered out.
- Intended red after tests only: `cargo test effect_preview_ -- --nocapture` — 3 selected tests ran and all 3 failed
  at the intended content/parity assertions; 986 Core lib tests were filtered out.
- Final focused: `cargo test -p game_core effect_preview_ -- --nocapture` — 6 passed, 0 failed, 0 ignored; 986 lib
  tests filtered out. Re-executed during this independent audit.
- Final complete preview family: `cargo test -p game_core range_preview -- --nocapture` — 14 passed, 0 failed,
  0 ignored; 978 lib tests filtered out. Re-executed after the audit hypothesis test was removed.
- Exact server shape seam: `cargo test -p game_server deployment_range_preview_result_does_not_request_state_snapshot
  -- --nocapture` — 1 passed, 0 failed, 0 ignored; 58 lib tests filtered out. Re-executed during this audit.
- Complete Core lib debug: 992 passed, 0 failed, 0 ignored.
- Complete Core lib release: 992 passed, 0 failed, 0 ignored.
- Debug integrations: skill runtime 14/14, RON loading 25/25, live catalog 4/4, live item activation 4/4 and authored
  skill suite 4/4.
- Release integrations: skill runtime 14/14 and RON loading 25/25.
- Complete game server debug: 59 passed, 0 failed. Complete game server release: 59 passed, 0 failed. Zero-test
  binary/doc harnesses are not counted as behavioral evidence.
- `cargo check --workspace --all-targets` — passed; compile-only evidence is used only as a workspace static gate.
- `cargo fmt --all -- --check` — passed again during the audit.
- Repository-wide `git diff --check` — passed again during the audit.

The production tree did not change after complete debug/release/integration/server validation. The audit probe was
test-only, rejected by publication before behavior, removed, hash-verified and followed by fresh focused and full
range-preview runs.

## Baseline Failures

- No unrelated baseline failure remains.
- The initial three failures were the intended CRA-P09-C04 characterizations: cast reach instead of multi-step
  footprint union, false cast cells for an Instant-only effect projection and disagreement with actual runtime area
  event cells.
- The first audit hypothesis command used directory name `core_enhanced` as a package ID and selected no tests; the
  package was corrected to `game_core`. It is a command/setup error, not behavioral evidence.
- The temporary invalid anchor/tile-origin fixture was rejected by an existing publication invariant. It was not a
  production regression and is recorded as a negative input-space proof, not counted among the intended red tests.
- No ignored test, skipped required environment, known flaky failure or external blocker is hidden by the verdict.

## Cross-Component Findings

- Phase 16's typed query is the shared geometry/materialization seam and remains unchanged.
- Phase 17's actual-delivery meaning prevents preview from treating acquisition anchors as affected targets; Phase
  18 consumes only spatial footprints and introduces no usefulness authority.
- Core DTO and server/Unity consumers use the same final single-collection shape. Live RON and runtime delivery use
  the same footprint definitions without migration.
- Phase 34 remains the later owner of server DTO authority de-duplication. No Phase 34 implementation or placeholder
  was pulled forward.
- Audit hypothesis P18-F01 (possible anchor/tile-origin divergence) is closed by strict publication validation and
  the restored production hash.
- Unresolved material cross-component findings: none.

## Long-Term Direction Review

| Candidate ID | Work type | Current→target authority result | Preserved-invariant result | Removal/consolidation result | Validation evidence | Verdict |
| --- | --- | --- | --- | --- | --- | --- |
| CRA-P09-C04 | correctness correction | cast-reach substitution replaced by one delivery-footprint union producer | cast range, runtime delivery/state, data and public shape preserved | obsolete content fallback removed; Phase 16 query reused; no per-step/legacy carrier | focused red→green, runtime event parity, Core/server debug/release, Unity/static audit | pass |

- Fit: High.
- Improvement class: none.
- Reason: the correction narrows authority to authored delivery footprints, reuses the existing geometry primitive,
  remains pure and introduces neither synchronized state nor a compatibility interval. Exact runtime-event parity
  and public consumer inspection protect behavior rather than merely checking private helper shape.

## Primary Verdict

- Complete.
- Long-term fit: High.
- Combined phase-exit verdict: `Complete / High`.

All evidence-matrix rows pass. There is no unresolved material finding, policy question, scope decision, external
blocker or required compatibility layer.

## Status Transition

- from: `review_pending`, `Implementation Complete: false`
- to: `complete`, `Implementation Complete: true`
- P18-C10: complete through this audit
- next Phase activation: allowed only after Master records Phase 18 `complete — Complete / High`, then initializes,
  registers and fully reads the Phase 19 Child documents and fully reads this final audit under the linear protocol

## Remaining Risk And Required Action

- Retargeted cast-target or impact-context footprints remain unavailable when preview inputs cannot determine their
  anchor. Predicting future target/impact positions would require a separately approved projection policy.
- Unity intentionally flattens the existing cell arrays for overlay rendering. A future need to style cast and
  effect cells independently or show step-specific areas requires a versioned consumer/DTO decision.
- Preview represents initial spatial footprints, not future persistent tracking, occupancy, filters, effects or
  collision outcomes.
- Immediate correction required: none.
- Out-of-scope follow-up required for this verdict: none.
- Policy question: none.
