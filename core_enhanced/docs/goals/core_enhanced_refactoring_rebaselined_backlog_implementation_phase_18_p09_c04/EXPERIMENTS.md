# Phase 18 CRA-P09-C04 Experiments

Status: complete
Implementation Complete: true

## 2026-08-10 — E01 Activation And Evidence Freeze

### Activation Evidence

- Master Phase 17 is `complete — Complete / High`; Phase 18 is the only dependency-satisfied next row.
- The final Phase 17 audit was fully read before readiness and records actual delivery-set semantics with no pending
  Phase 18 implementation or public-shape change.
- Original and re-audited P09-C04 evidence still identifies cast acquisition reach being published as effect cells.
- CRA-POL-005 resolves the historical open choice as one deterministic deduplicated union, while CRA-POL-006 freezes
  the existing DTO/server/Unity shape.
- Phase 16 supplies the typed geometry/materialization seam and Phase 17 fixes actual-delivery meaning; both direct
  dependencies have `Complete / High` exits.
- No production source, test, data or canonical was inspected or mutated during Child initialization.

### Approved Decision Table

| Step contribution | `effect_preview_cells` contribution |
| --- | --- |
| TileArea spatial delivery | runtime-equivalent footprint after origin/facing and board clipping |
| multiple TileArea steps | deterministic deduplicated union of every footprint |
| Instant without spatial area | none |
| Projectile without spatial area | none |
| cast acquisition reach | none; remains in its separate range field |

### Next Experiment

Fully read `PLAN.md`, `EXPERIMENTS.md` and `EXPERIMENT_NOTES.md`, then reread all 246 lines of the final Phase 17
completion audit. Only afterward capture the activation dirty baseline and inspect current preview/runtime/transport
source, tests, live data and canonicals.

## 2026-08-10 — E02 Activation Read And Dirty Baseline

- All 188 lines of `PLAN.md`, 31 lines of this experiment log and 25 lines of `EXPERIMENT_NOTES.md` were fully read.
- All 246 lines of the final Phase 17 completion audit were then reread. It confirms the Phase 16 query and Phase 17
  actual-delivery authority are stable and that no preview union or compatibility placeholder was prebuilt.
- Master contains exactly one `active` registry row: Phase 18.
- Activation HEAD is `f318414fb0156a67e04c5479000c2a2683fc0c09`. Expanded status contains 460 entries
  with normalized SHA-256 `c2ddedad20fa31a1969c626f7d863107dc1ef63e94ba667c7a57e6b6b259dd58`.
- Prospective production/canonical paths are already dirty and therefore user/campaign-owned until scoped diff
  attribution proves otherwise. Their activation fingerprints are recorded in N02.

### Next Experiment

Read current scoped diffs and full relevant source/tests/data/canonical, map the Core/server/Unity consumer graph and
run the current focused baselines before adding intended-red projection tests.

## 2026-08-10 — E03 Current Graph, Baseline And Intended Red

### Current Authority And Consumers

- `range_preview.rs` has two producers: deployed-unit checkpoints and pending-deployment all-facing previews.
  `SelfUnit` currently materializes cast reach and writes it directly to `effect_preview_cells`; manual unit/tile
  retains cast cells only and automatic unit remains unavailable until a runtime target exists.
- Runtime TileArea resolves caster/cast-target/impact anchors, projects the authored `footprint` with the caster
  facing and Phase 16 `TileReachQuery`, clips to valid board cells and publishes the result in
  `SkillAreaDeclared.shape.TilePattern.affected_tiles`.
- The public Core DTO owns one `Vec<Position>` effect collection. The server forwards it through the same DTO/serde
  payload. Unity's `BattlefieldRangePreviewParser` reads final `*_cells` arrays and performs no local geometry;
  its mock snapshot contains the same `effect_preview_cells` field.
- The newer read-only manual `SkillTargetPreviewResultDto` carries eligibility only and has no effect-cell field.
  Changing that DTO is outside CRA-P09-C04 and unnecessary for the historical SelfUnit defect.
- Live SelfUnit skills have deterministic caster/self-centered TileArea steps or Instant-only delivery. No live
  SelfUnit preview depends on an impact-context or runtime-selected enemy anchor.

### Baseline And Red

- Pre-test baseline `cargo test range_preview -- --nocapture`: 9 passed, 0 failed; 977 Core lib tests filtered out.
- Added production-seam tests only; no production behavior changed.
- `cargo test effect_preview_ -- --nocapture`: 3 selected tests ran and all 3 failed at the intended assertions;
  986 Core lib tests were filtered out.
- Failures prove the exact defect: a one-cell cast pattern appeared instead of the six-cell deduplicated union, an
  Instant-only skill incorrectly published a cast cell, and public preview disagreed with the union of actual
  `SkillAreaDeclared` runtime cells at the first center/facing case.

### Next Experiment

Add one pure self-cast effect-footprint projector in `range_preview.rs`. It must iterate TileArea steps, reject only
runtime-dependent anchors, use `TileReachQuery`, sort row-major after union/dedup and leave manual/automatic, DTO and
runtime paths unchanged. Then rerun the red matrix before expanding compatibility coverage.

## 2026-08-10 — E04 Delivery-Footprint Projection And Expanded Green

### Implementation

- `SelfUnit` preview now iterates every step and admits only `DeliveryDef::TileArea` into effect cells. Instant and
  Projectile contribute nothing.
- Caster and reusable self/cast-target anchors resolve from the preview caster position. Retargeted cast-target and
  impact-context anchors return the existing `RequiresRuntimeTarget` unavailable result rather than inventing a
  fixed location.
- Every spatial step uses Phase 16 `TileReachQuery` with the runtime caster facing and valid-board clipping. The
  combined cells are sorted `(y, x)` and deduplicated once after union, so authored step order cannot affect output.
- Missing/invalid cast input retains `MissingRangeData`; cast reach cells no longer become effect cells or suppress
  an otherwise valid edge-clipped footprint merely because acquisition cells are off-board.
- Manual unit/tile cast cells, automatic target-dependent unavailability, public DTOs, server mapping, runtime
  delivery and live RON were not changed.

### Expanded Safety Net

- `cargo test effect_preview_ -- --nocapture`: 6 passed, 0 failed; 986 Core lib tests filtered out.
- Coverage now includes both step orders, overlapping asymmetric caster and rotation-invariant self CastTarget
  footprints, exact row-major dedup, explicit 3x3/5x5 corner clipping, Instant/Projectile exclusion,
  runtime-dependent unavailable behavior and missing-cast-data preservation.
- The production-runtime integration compares public preview to the union of actual
  `SkillAreaDeclared.affected_tiles` at four center facings and four clipped corner/facing cases, and compares the
  complete transaction debug snapshot before/after preview to prove observation purity.
- `cargo test range_preview -- --nocapture`: 13 passed before the final missing-data case; all original baseline
  cases remained green. The final focused filter includes that preservation case.
- Server's existing deployment-preview JSON test now explicitly asserts the unchanged nested `cast_cells` and
  `effect_preview_cells` arrays; the focused server test passes 1/1.
- During expansion, a proposed CastTarget fixture used a non-rotation-invariant footprint and was rejected by the
  existing publication gate before behavior ran. The fixture was corrected to a valid rotation-invariant cross;
  this compile/setup-independent validation rejection is not counted as policy red or production failure.

### Next Experiment

Run the complete debug/release Core suites, live/RON/skill integrations, complete debug/release server suites and
workspace/static/format/diff checks. Reinspect public/serde/Unity shape and live hashes, then update the completion
matrix and hand off to independent review.

## 2026-08-10 — E05 Final Validation, Compatibility And Audit Handoff

### Final Behavioral Evidence

- Final focused `effect_preview_`: 6/6; 986 Core lib tests filtered out.
- Final complete range-preview family: 14/14; 978 Core lib tests filtered out.
- Complete Core lib: debug 992/992 and release 992/992.
- Debug integrations: skill runtime 14/14, RON 25/25, live catalog 4/4, live item 4/4 and authored skill suite 4/4.
- Release integrations: skill runtime 14/14 and RON 25/25.
- Complete game server: debug 59/59 and release 59/59; zero-test binary/doc harnesses are not counted as behavioral
  evidence.
- Root `cargo check --workspace --all-targets`, `cargo fmt --all -- --check` and repository-wide
  `git diff --check` pass.

### Compatibility, Data And Dirty Evidence

- Public `LiveBattleActiveSkillRangePreviewDto` source is unchanged at activation hash `65349de8...`; its one
  `effect_preview_cells: Vec<Position>` field, adjacent `cast_cells`, derives and nesting did not change.
- Phase 16 tile query `73930102...`, runtime area `23e25cdf...` and live Skill RON `4087df05...` retain activation
  hashes. No runtime geometry, targeting, delivery, data or schema was edited.
- Server production mapping remains the existing Core DTO serialization; only its existing deployment-preview test
  gained exact nested-array assertions. Current server file hash is `18f31016...`.
- External Unity was inspected read-only. `BattlefieldRangePreviewParser` consumes final `*_cells` arrays without
  local geometry and `ArkMockSnapshots` retains the same field. Their current hashes are `96c8a7e9...` and
  `f20a48e4...`; no external file was mutated.
- Final range-preview source/test hash is `1a8fce57...`, Core test module `18fce294...`, corrected canonical
  `21fd770d...`.
- Expanded status remains exactly the activation 460 entries with normalized SHA-256
  `c2ddedad20fa31a1969c626f7d863107dc1ef63e94ba667c7a57e6b6b259dd58`; all Phase edits landed inside paths
  already represented at activation. No reset, restore, clean, stash, destructive deletion, data rewrite or broad
  formatting was used.

### Phase Boundary

- P18-C01 through P18-C09 are complete. P18-C10 remains pending the required independent review.
- Phase 19 remains pending and unauthorized. No P01-C03 refactor, Phase 10 DTO consolidation or per-step preview
  placeholder was prebuilt.
- Status transitions to `review_pending`; `Implementation Complete` remains false until a `Complete / High` audit.

### Next Experiment

Reread all 567 lines of `goal_completion_review_guide.md`, independently inspect policy, implementation, tests,
canonical, public/server/Unity compatibility and dirty scope, then create `COMPLETION_AUDIT.md`. Do not activate
Phase 19 unless the verdict is `Complete / High`.

## 2026-08-10 — E06 Independent Audit And Completion Transition

### Audit Result

- All 567 lines of `goal_completion_review_guide.md` were reread before the independent audit.
- `COMPLETION_AUDIT.md` connects P18-A01–A10 to policy, runtime, data, DTO/transport/persistence, Unity consumer,
  legacy, dirty preservation, canonical and exact validation evidence. Every row passes.
- One audit hypothesis—whether presentation anchor and footprint `tile_origin` could diverge—was exercised with a
  temporary invalid definition. Existing publication validation rejected the combination before preview; the test
  was removed, the final production hash was restored, and fresh focused/range-preview runs passed 6/6 and 14/14.
- Primary verdict is `Complete`; long-term fit is `High`; improvement class is `none`.

### Completion And Boundary

- P18-C10 is complete. Child status changes from `review_pending` to `complete`; `Implementation Complete` changes
  from false to true.
- The audit artifact has SHA-256 `338070bf27a16dde78df599538d4f30245c11d6888d43486ada44489820b2a1a`.
- Audit-artifact-inclusive expanded status is 461 entries with normalized SHA-256
  `47d1fdad5b52dd69c80ef0577cfca02492e0dc6d3ab476f04fabcbc8c7d777fb`.
- Phase 19 may become ready only through the Master transition. No Phase 19 production inspection or mutation was
  performed by the Phase 18 audit.
