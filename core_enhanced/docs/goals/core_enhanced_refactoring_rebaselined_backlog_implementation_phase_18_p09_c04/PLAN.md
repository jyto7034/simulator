# Phase 18 Goal — CRA-P09-C04 Project Effect Preview From Delivery Footprints

Status: complete
Implementation Complete: true
Goal Type: phase
Execution Mode: implement_and_verify
Candidate ID: CRA-P09-C04
Master Phase: 18 of 44
Work Type: correctness correction
Production Mutation Authorized: yes — the user's Master execution request remains active; authorization is limited
to this Phase after Phase 17's `Complete / High` exit
Repository Root: `/mnt/f/work/simulator`
Activation Date: 2026-08-10 (`Asia/Seoul`)

## Objective And Completion Conditions

Implement CRA-POL-005 without changing the existing public projection shape. For a previewed multi-step cast,
`effect_preview_cells` must be the canonical deterministic deduplicated union of every step's actual spatial
delivery footprint after the same origin/facing transforms and valid-board clipping used by runtime. Cast
acquisition reach remains separately represented and must never substitute for effect footprint. Instant or
projectile delivery without a spatial area contributes no area cells.

This Phase is complete only when the current preview producer and all Core/server/Unity-facing consumers are
inventoried; test-first cases reproduce cast-reach-as-effect-footprint drift; center/edge/all-facing parity proves
preview cells equal runtime TileArea cells; multi-step differing/overlapping footprints prove union,
deduplication and canonical order; nonspatial steps contribute no cells; projection is side-effect free; current
range cells and DTO/tag/nesting/serialization remain exact; obsolete fallback/duplicate footprint adapters are
removed only where proven; focused and broad validation pass; and an independent phase-exit audit returns
`Complete / High`.

## User Or Contract-Visible Problem

Historical and re-audited evidence found that `self_target_skill_preview` publishes cast acquisition cells as
`effect_preview_cells` without reading step delivery footprints. Published examples can cast over a front 3x2
reach but apply a 3x3 or 5x5 TileArea, so the effect field can describe a different geometry from execution. Those
examples were assigned to non-player units at audit time, making this a confirmed projection-contract defect rather
than an asserted current player-checkpoint incident.

The public carrier already exposes one effect-cell collection. Approved policy freezes its meaning as a union; this
Phase corrects content while preserving its exact external shape.

## Dependency And Policy Authority

- Master: `../core_enhanced_refactoring_rebaselined_backlog_implementation_master/PLAN.md`, Phase 18 registry row.
- Direct linear predecessor: Phase 17 CRA-P09-C03, reviewed `Complete / High` in
  `../core_enhanced_refactoring_rebaselined_backlog_implementation_phase_17_p09_c03/COMPLETION_AUDIT.md`.
- Required spatial seam: Phase 16 CRA-P09-C06 crate-private typed tile reach/materialization query.
- Required meaning seam: Phase 17 CRA-P09-C03 actual delivery-set semantics; preview must not recreate its retired
  anchor/first-step approximations.
- Approved direct policy: CRA-POL-005 in
  `../core_enhanced_refactoring_post_boss_runtime_reaudit_final_catalog_consolidation/APPROVED_POLICY_DECISIONS.md`.
- Cross-cutting compatibility policy: CRA-POL-006.
- Historical evidence:
  - `../core_enhanced_refactoring_audit_phase_09_battle_targeting_and_delivery/CANDIDATE_CATALOG.md`
  - `../core_enhanced_refactoring_audit_phase_09_battle_targeting_and_delivery/TEST_AUDIT.md`
  - `../core_enhanced_refactoring_audit_phase_09_battle_targeting_and_delivery/BOUNDARY_AUDIT.md`
  - `../core_enhanced_refactoring_post_boss_runtime_reaudit_phase_09_battle_targeting_delivery_and_projectiles/CANDIDATE_REAUDIT.md`
  - `../core_enhanced_refactoring_post_boss_runtime_reaudit_phase_09_battle_targeting_delivery_and_projectiles/TEST_REAUDIT.md`
  - `../core_enhanced_refactoring_post_boss_runtime_reaudit_phase_09_battle_targeting_delivery_and_projectiles/BOUNDARY_AUDIT.md`
- Final order/test/gate evidence:
  - `../core_enhanced_refactoring_post_boss_runtime_reaudit_final_catalog_consolidation/EXACT_LINEAR_IMPLEMENTATION_ORDER.md`
  - `../core_enhanced_refactoring_post_boss_runtime_reaudit_final_catalog_consolidation/TEST_REBASELINE_CATALOG.md`
  - `../core_enhanced_refactoring_post_boss_runtime_reaudit_final_catalog_consolidation/IMPLEMENTATION_WAVES_AND_GATES.md`

CRA-POL-005 is explicit: retain one `effect_preview_cells` collection; union all actual spatial step footprints;
deduplicate and canonicalize deterministically; use runtime geometry/clipping/facing; keep cast reach distinct;
exclude nonspatial instant/projectile steps; preserve exact DTO/server/Unity serialization; add no per-step preview.
No projection-policy question remains open unless the current previewed-cast model lacks enough authoritative inputs
to determine a step origin/facing without simulating or mutating future runtime state.

## Priority And Current Disposition

- Priority axes: `2 / 2 / 1 / 3 / 3 / 2 / unknown`; known subtotal `13/18`.
- Final disposition at activation: `still_valid`, correctness correction, union parity tests first.
- Current live-tree disposition: discovery pending until all three Child documents and the complete final Phase 17
  audit are fully read and the scoped dirty baseline refreshes historical source/test/canonical targets.

## Current And Target Authority

- Historical current authority: range preview materializes cast targeting policy/pattern and reuses those cells as
  effect cells; runtime step delivery separately owns TileArea footprint geometry.
- Target authority: one side-effect-free delivery-aware preview projection obtains every spatial step footprint from
  the same typed Phase 16 geometry/materialization authority as runtime, then performs one deterministic
  deduplicating union for the existing public collection.
- The exact current type/function/file names, origin rules, ordering primitive and transport consumers must be
  rediscovered after the Child read/dirty gate. Historical names are evidence, not mutation instructions.

## In Scope

- inventory preview producers and consumers across Core checkpoint/world projection, game server setup/update/resync
  transport, tests, canonicals and accessible Unity contract fixtures;
- capture expanded dirty status, scoped diffs and fingerprints before production edits;
- add intended-red tests before production changes for self-target cast range differing from 3x3/5x5 TileArea
  delivery footprints;
- cover center, valid-board edges/corners and every facing with preview/runtime footprint parity;
- cover multi-step differing and overlapping footprints, duplicate removal and exact canonical cell order;
- prove Instant and non-area Projectile steps add zero effect cells, including mixed spatial/nonspatial skills;
- reuse the Phase 16 typed query, board clipping and facing transforms rather than copying an adapter;
- preserve effect projection purity and current range-preview meaning/order;
- compare public Rust and serde shapes plus exact server/Unity JSON or fixtures available in the workspace;
- update the nearest projection/targeting canonical only after the implementation is real.

## Out Of Scope

- a per-step preview collection, new DTO field/tag/nesting, versioned wire migration or UI presentation redesign;
- changing cast acquisition reach, targeting, usefulness, delivery execution, filters, tracking, effects or balance;
- inventing predictive future retarget positions, collision paths, obstacle/occupancy filtering or target-value cells;
- changing Phase 16 geometry/materialization semantics or Phase 17 usefulness rules;
- Phase 10 command-result/server DTO authority consolidation or P10-C03 server DTO de-duplication;
- Phase 29 projectile motion consolidation, public/serialized ghost-surface cleanup or unrelated refactoring;
- broad formatting, data rewrite or dirty-worktree normalization.

## Preserved Invariants

1. `effect_preview_cells` remains exactly one collection with unchanged public/serde field shape.
2. Cast acquisition range remains a separate projection and retains its current meaning and deterministic order.
3. Effect cells are only actual spatial delivery footprints, never cast reach or target anchor occupancy.
4. Each spatial step uses runtime-equivalent origin, facing transform, geometry and valid-board clipping.
5. The multi-step result is a deduplicated deterministic union with one canonical order independent of insertion.
6. Instant/projectile delivery without a TileArea contributes no area cells.
7. Projection does not mutate battle/cast/unit/area/projectile state or apply effects.
8. Targeting, selection, usefulness, release timing, filters, tracking and runtime effect geometry remain unchanged.
9. Server/Unity serialization, DTO tags/nesting/nullability and message order remain exact under CRA-POL-006.
10. Pre-existing user changes are preserved without reset, restore, clean, stash or broad formatting.

## Implementation And Dirty-Worktree Strategy

After all three Child documents exist and Master records sole activation, read them completely and reread the full
Phase 17 completion audit. Capture expanded status and prospective target diff/hash before inspecting production
source/tests/live data. Treat every existing hunk as user-owned until reconciled.

Use test-first proof in this order:

1. Trace the current preview input model, producer, ordering, checkpoint carrier and server/Unity consumers.
2. Trace runtime TileArea origin/facing/materialization for each supported step target/tracking form.
3. Run current range/effect preview, tile geometry, runtime area and serialization baselines.
4. Add parity/union decision tables and prove nonzero intended failures caused by cast-reach substitution.
5. Introduce the smallest delivery-aware projection that reuses existing typed geometry and canonical ordering.
6. Remove only the cast-reach fallback and duplicate adapter proven obsolete; keep range projection intact.
7. Re-run focused, complete Core, affected server/serialization/release and static/dirty/compatibility checks.

## Migration, Deployment And Compatibility

- CRA-POL-005 approves a semantic correction to existing effect-cell content, not a schema migration.
- No dual read/write, compatibility alias, fallback field or per-step placeholder is permitted.
- Exact server/Unity serialized structure must remain unchanged. Current field content may change only where required
  to replace cast reach with the approved footprint union.
- If an external consumer depends on the incorrect content rather than the stable shape, record concrete evidence
  and stop for an explicit versioned-consumer decision; do not silently preserve parallel old semantics.
- Live RON is input to the projection and must not be rewritten for this correction.

## Validation Plan

- Baseline: current range/effect preview, Phase 16 query, TileArea runtime, checkpoint/world projection and transport.
- First red: range-versus-3x3/5x5 mismatch and multi-step union/dedup/order tables through the public projection seam.
- Focused final: center/edge/corner/all-facing runtime parity; spatial-only, nonspatial-only and mixed-step cases;
  no-mutation and exact range-field preservation.
- Broad: complete affected Core integration families, Core lib, game_server targets and release.
- Compatibility: Rust/serde field inventory plus exact setup/update/resync or available golden JSON comparison and
  accessible Unity fixture/consumer inspection.
- Static: producer/caller inventory, cast-reach fallback/duplicate adapter search, scoped status/diff/hashes,
  canonical comparison, formatter and diff check.
- Zero-test filters, helper-only predicates, compile-only success and snapshot-only updates do not count as proof.

## Completion Evidence Matrix

| ID | Condition | Required evidence | Status |
| --- | --- | --- | --- |
| P18-C01 | complete preview producer/consumer graph identified | Core/server/Unity/canonical inventory | complete |
| P18-C02 | dirty targets and concurrent work preserved | pre/post status, scoped diff and hashes | complete |
| P18-C03 | cast-reach substitution defect reproduced | nonzero intended-red 3x3/5x5 mismatch tests | complete |
| P18-C04 | one delivery-aware effect projection authority exists | source/caller and fallback-removal audit | complete |
| P18-C05 | preview/runtime spatial cells match | center/edge/corner/all-facing parity | complete |
| P18-C06 | multi-step union is exact and deterministic | differing/overlap/dedup/order matrix | complete |
| P18-C07 | nonspatial/range/no-mutation invariants hold | mixed-step and unchanged range/state tests | complete |
| P18-C08 | public/server/Unity compatibility remains exact | shape and serialized-structure comparison | complete |
| P18-C09 | focused and broad validation passes | nonzero Core/server/release/static counts | complete |
| P18-C10 | downstream seam/canonical and audit are current | no-prebuild audit and `Complete / High` review | complete |

## Stop Conditions

Stop and do not activate Phase 19 if preview inputs cannot determine approved footprints without predicting mutable
future state; origin/facing semantics conflict with runtime; union ordering requires an unapproved external change;
current range meaning would change; a DTO/tag/nesting/nullability/message-order migration is required; accessible
server/Unity consumers require a content-compatibility decision not covered by CRA-POL-005; dirty overlap is
inseparable; a required runner selects zero tests; or completion review is not `Complete / High`.

사용자와 의논하여 정해야 할 정책이 있을 경우 Goal을 종료한다.
