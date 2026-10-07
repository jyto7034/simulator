# Phase 18 CRA-P09-C04 Experiment Notes

Status: complete
Implementation Complete: true

## Note Format

Each finding records date, source/evidence, observation, impact on the Phase, and disposition. Historical names and
line locations are not treated as current authority until the post-activation dirty/source gate.

## 2026-08-10 — N01 Activation Boundary

- Source/evidence: Master Phase 18 row; Phase 17 completion audit; CRA-POL-005/006; original and re-audited P09-C04
  candidate, test and boundary records; Final exact order and rebaselined tests.
- Observation: policy and dependencies are complete. The correction owns effect-preview content derived from spatial
  delivery footprints, while the existing collection shape and separate cast-range meaning are fixed invariants.
- Impact: no user policy question blocks investigation. Current runtime/preview origin and consumer details still
  require rediscovery after the mandatory Child/audit reads and dirty baseline.
- Disposition: initialize and activate Phase 18 only; no production inspection or mutation yet.

## Open Evidence Slots

- N02: activation expanded status, prospective target fingerprints and scoped dirty attribution.
- N03: current Core preview producer/caller/order and runtime footprint authority graph.
- N04: server/Unity/public/serde consumer and exact-serialization inventory.
- N05: intended-red decision table and failure classification.
- N06: implementation authority, removed fallback and invariant proof.
- N07: final validation, compatibility and audit findings.

## 2026-08-10 — N02 Activation Dirty Baseline

- Source/evidence: expanded `git status --short --untracked-files=all`, scoped status and SHA-256 fingerprints after
  the mandatory Child/predecessor reads and before detailed production-source inspection.
- Observation: activation HEAD is `f318414fb0156a67e04c5479000c2a2683fc0c09`; expanded status is 460 entries at
  normalized SHA-256 `c2ddedad20fa31a1969c626f7d863107dc1ef63e94ba667c7a57e6b6b259dd58`.
- Prospective fingerprints: range preview `e9e963f6...`, behavior/public projection `65349de8...`, Phase 16 tile
  query `73930102...`, area runtime `23e25cdf...`, Core test module `4aab68d4...`, server state `2f9263f7...`,
  targeting canonical `1b24581e...`, Unity preview Goal `f3be50af...` and live Skill RON `4087df05...`.
- Observation: every listed production/data/canonical path except the historical Unity Goal is already modified in
  the shared worktree. Existing content and hunks remain user/campaign-owned until reconciled; path dirtiness alone
  is not Phase 18 attribution.
- Impact: Phase 18 edits must be minimal and later isolated by pre/post hashes plus scoped reconstruction where an
  existing dirty file overlaps.
- Disposition: baseline frozen; detailed read-only producer/consumer and diff investigation may begin.

## 2026-08-10 — N03 Current Producer And Delivery Graph

- Source/evidence: full `range_preview.rs`; public DTO definitions; deployed checkpoint and pending-deployment
  callers; TileArea geometry/materialization runtime; skill publication validation; current live Skill RON.
- Observation: `self_target_skill_preview` is the sole incorrect producer. It maps cast reach to effect cells.
  Both checkpoint and pending-deployment surfaces call that producer, so one correction covers both.
- Observation: runtime materializes every TileArea footprint through the Phase 16 `TileReachQuery`. Caster anchors
  use current position/facing; target/impact anchors must be rotation-invariant in published data. The preview has no
  runtime-selected target/impact context and must not invent one.
- Observation: all four live SelfUnit skills are statically projectable: Pale Benediction, White Resonance and Root
  Blessing use caster/self cast-target anchors; Opening Focus is Instant-only. No live RON edit is needed.
- Impact: a pure iterator over spatial steps can share the exact geometry query and preserve runtime. If a future
  SelfUnit definition uses retargeted cast-target or impact-context anchoring, truthful preview is unavailable with
  `RequiresRuntimeTarget` rather than a fabricated footprint.
- Disposition: implement only the SelfUnit effect producer; leave PlayerUnit/PlayerTile candidate preview and
  AutomaticUnit unavailable semantics unchanged.

## 2026-08-10 — N04 Transport And Unity Consumer Graph

- Source/evidence: `LiveBattleActiveSkillRangePreviewDto`, game-server result/state passthrough and tests; external
  Unity `BattlefieldRangePreviewParser.cs`, `ArkMockSnapshots.cs` and transport/probe canonicals.
- Observation: public/server/Unity shape is already one `effect_preview_cells` array. Unity enumerates final
  `*_cells` arrays and does not calculate geometry from skill metadata. No consumer requires per-step fields.
- Impact: changing only array content for SelfUnit previews is exactly CRA-POL-005. DTO tags, names, nesting,
  nullability and message ordering stay untouched; external Unity source needs no mutation.
- Disposition: preserve all public types and exact serialization; validate current server and external fixture/parser
  shape after Core behavior is green.

## 2026-08-10 — N05 Intended-Red Classification

- Source/evidence: three `effect_preview_` tests added before production changes.
- Observation: all three selected tests failed behaviorally, with current cast cells on the left and approved
  footprint/empty/runtime-union results on the right. No compile failure or zero-test filter is counted.
- Impact: the safety net directly exercises public preview production and actual runtime area events, including
  union/dedup/order, all facings, board corners, nonspatial exclusion and observation purity.
- Disposition: valid intended red; production correction authorized within the active Child.

## 2026-08-10 — N06 Implemented Authority And Invariants

- Source/evidence: `self_target_effect_preview_cells`, expanded focused tests, runtime event parity and corrected
  combat canonical.
- Observation: one preview-specific aggregator owns effect-cell content. It reuses `TileReachQuery`; it does not
  duplicate pattern rotation, valid-tile clipping or runtime effect/target filtering.
- Observation: final ordering is explicitly `(y, x)` after all steps are unioned, so it remains deterministic when
  step order changes. Spatial steps clipped to no valid cells contribute nothing; a skill whose every spatial cell
  is clipped reports `NoValidCells`. A skill with no spatial step can remain available with an empty effect array.
- Observation: existing cast-data validity/reason semantics are retained without reusing cast cells as effect
  content. Runtime-dependent target/impact anchors use the existing unavailable reason.
- Impact: CRA-POL-005 is implemented with no parallel old-content fallback, per-step carrier or stateful preview
  simulation. Runtime delivery and external shapes remain untouched.
- Disposition: focused behavior is green; proceed to broad and compatibility validation.

## 2026-08-10 — N07 Final Validation And Review Handoff

- Source/evidence: final focused/range-preview/Core debug-release/live-RON/server debug-release/workspace/static
  runs; post hashes; external Unity read-only consumer reinspection; corrected canonical.
- Observation: all required commands pass with nonzero behavioral counts. Runtime geometry/data/public DTO source
  hashes are unchanged; only the preview producer, tests, server shape assertion and canonical changed for Phase 18.
- Observation: the expanded status count/hash is unchanged from activation despite content edits because every
  Phase target was already dirty and the three Child files already existed when the baseline was captured.
- Impact: no baseline failure, compatibility migration, live data drift, pending consumer decision or cross-phase
  implementation remains. P18-C01–C09 have direct evidence.
- Disposition: hand off as `review_pending`. The completion guide must be fully reread and P18-C10 decided by an
  independent audit before Master/Phase 19 transition.

## 2026-08-10 — N08 Independent Completion Audit

- Source/evidence: full 567-line completion guide reread; `COMPLETION_AUDIT.md`; audit-current focused/range/server,
  formatter, diff and hash checks; read-only Unity call-site inspection.
- Observation: all P18-A01–A10 rows pass. Runtime/data/public/transport/consumer authorities align with CRA-POL-005
  and CRA-POL-006, and no obsolete fallback, parallel DTO, persistence state or Phase 19 prebuild remains.
- Observation: the possible anchor/`tile_origin` divergence was outside the published skill input space; strict
  validation rejected it, and the test-only probe was fully removed before fresh green runs.
- Impact: no material finding, baseline failure, policy decision or compatibility migration blocks phase exit.
- Disposition: `Complete / High`, improvement class `none`; allow the exact linear Master transition to Phase 19
  readiness.
