# Phase 16 Goal — CRA-P09-C06 Centralize Typed Tile Reach Queries And Projections

Status: complete
Implementation Complete: true
Goal Type: phase
Execution Mode: implement_and_verify
Candidate ID: CRA-P09-C06
Master Phase: 16 of 44
Work Type: behavior-preserving refactor
Production Mutation Authorized: yes — the user's Master execution request remains active; authorization is limited
to this Phase after Phase 15's `Complete / High` exit
Repository Root: `/mnt/f/work/simulator`
Activation Date: 2026-08-10 (`Asia/Seoul`)

## Objective And Completion Conditions

Replace repeated tile range policy/pattern/facing/valid-board adapters with one typed query/projection authority.
The authority must expose distinct membership and stable materialization operations over captured origin/facing and
the current valid-tile set, then be reused by production targeting, manual cast validation, range preview and area
runtime/usefulness consumers without changing any current result.

This Phase is complete only when the current producer/caller graph is identified; a test-first cross product proves
exact membership/materialization parity for every policy, representative pattern, facing, anchor-inclusion and board
edge/void clipping case; the smallest domain query replaces duplicate transforms/clipping loops; cast reach remains
distinct from delivery footprint; obstacle/occupancy filtering is not introduced; current selector/cast/preview/area
integrations remain exact; public/RON/save/wire shapes do not change; focused and broad validation pass; and the
independent phase-exit audit returns `Complete / High`.

## User Or Contract-Visible Problem

Historical audit and current reaudit evidence found that runtime target membership, manual cast reach, preview cell
materialization, skill simulation and TileArea runtime/usefulness each adapt `TileRangePolicy`, `TileRangePattern`,
facing and valid-board clipping separately. Those paths currently agree, but the duplicated adapters already made
it possible for later usefulness/preview semantics to drift around which footprint is being queried. Phase 16 does
not correct those later semantics; it provides one trustworthy typed spatial authority first.

## Dependency And Authority

- Master: `../core_enhanced_refactoring_rebaselined_backlog_implementation_master/PLAN.md`, Phase 16 registry row.
- Direct linear predecessor: Phase 15 CRA-P09-C01, reviewed `Complete / High` in
  `../core_enhanced_refactoring_rebaselined_backlog_implementation_phase_15_p09_c01/COMPLETION_AUDIT.md`.
- Additional stabilized seam: Phase 14 CRA-P09-C02 terminal delivery is `Complete / High`.
- Candidate-specific policy: none; this is a behavior-preserving internal refactor.
- Cross-cutting compatibility policy: CRA-POL-006.
- Downstream dependency: Phase 17 CRA-P09-C03 and Phase 18 CRA-P09-C04 require this typed footprint query before
  changing usefulness or preview meaning.
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

No policy decision is open. If current callers disagree in a user-visible way rather than only duplicating an
equivalent adapter, stop for policy/scope review instead of choosing a winner inside this refactor.

## Priority And Current Disposition

- Priority axes: `3 / 1 / 1 / 2 / 2 / 2 / unknown`; known subtotal `11/18`.
- Final disposition at activation: `still_valid`, behavior-preserving refactor, characterization first.
- Current live-tree disposition: complete — independent phase-exit audit `Complete / High`, improvement `none`.

## Current And Target Authority

- Historical current authority: `TileRangePolicy` and `TileRangePattern` own data geometry, while targeting, cast
  validation, preview, usefulness and area runtime each adapt origin/facing/materialization/clipping.
- Target authority: one typed spatial query captures policy/pattern, anchor/origin, facing and valid-board context and
  offers explicit membership and stable valid-cell materialization. Consumers select cast reach versus delivery
  footprint but do not reimplement transform or clipping semantics.
- Exact current type/function/file names and whether the authority belongs beside tile-range data or battle spatial
  consumers must be rediscovered after the Child read/dirty gate; historical names are evidence, not mutation
  instructions.

## In Scope

- inventory all `TileRangePolicy`/`TileRangePattern` membership, rotation, facing, materialization and valid-tile
  clipping producers and callers;
- capture expanded dirty status, scoped diffs and fingerprints before edits;
- add cross-product characterization for pattern/policy, four facings, anchor inclusion, center/edge/void clipping,
  stable ordering and membership equals materialized-set containment;
- preserve whole-field versus pattern policy behavior and intentional lack of obstacle/occupancy filtering;
- introduce the smallest typed query/projection authority and migrate production selector, cast gate, preview and
  area/usefulness adapters that duplicate the same semantics;
- retain separate typed inputs for cast reach and delivery footprint;
- retain existing integration tests and update the nearest targeting/delivery canonical only after code is real.

## Out Of Scope

- Phase 17 CRA-P09-C03 changes to actual delivery-target usefulness or multi-step usefulness policy;
- Phase 18 CRA-P09-C04 effect-preview union/footprint correction or DTO meaning;
- changing any tile pattern geometry, anchor-inclusion, facing transform, stable ordering or valid-board clipping;
- adding obstacle, occupancy, line-of-sight, path cost, Boss navigation or physics filtering;
- changing selection rules, range tuning, live RON, projectile lifecycle, damage/effects or event ordering;
- public Rust, RON, save/checkpoint, DTO/wire/server/Unity shape migration;
- Phase 29 projectile motion extraction or unrelated cleanup/refactor;
- broad formatting, data rewrite or dirty-worktree normalization.

## Preserved Invariants

1. Every current membership answer is byte-for-byte/logically identical for the same board, anchor and facing.
2. Every current materialized cell list retains exact membership, valid-board clipping and deterministic ordering.
3. Pattern anchor inclusion and all four facing transforms remain exact.
4. Whole-field policy continues to mean current valid tiles and no obstacle/occupancy filtering is added.
5. Cast acquisition reach and step delivery footprint remain distinct inputs even when they share query mechanics.
6. Selector, manual cast, preview, area runtime and usefulness consumers retain current visible behavior.
7. Current public Rust, RON, V3 checkpoint, DTO/wire/server/Unity shapes remain exact under CRA-POL-006.
8. Phase 17/18 policy corrections remain unimplemented until their own activation.
9. Pre-existing user changes are preserved without reset, restore, clean, stash or broad formatting.

## Implementation And Dirty-Worktree Strategy

After all three Child documents exist and Master records sole activation, read them completely and reread the full
Phase 15 completion audit. Capture expanded status and prospective target diff/hash before inspecting production
source/tests/live data. Treat every existing hunk as user-owned until reconciled.

Use test-first proof in this order:

1. Trace every policy/pattern/facing/materialization/clipping adapter and classify exact input/output/order semantics.
2. Run current tile-range, targeting, cast, preview and area/usefulness baselines.
3. Add one cross-product characterization table over existing public/production behavior and prove nonzero selection.
4. Introduce the smallest typed query and migrate one consumer at a time without changing expected results.
5. Remove only duplicate transform/clipping adapters proven equivalent; keep cast reach and delivery footprint owners.
6. Re-run focused, complete Core, affected server/release and static/dirty/compatibility checks.

## Migration, Deployment And Compatibility

- This Phase is internal and behavior-preserving. No schema, data, API, checkpoint, DTO or wire migration is
  authorized or expected.
- Current live content requires no rewrite. Any need to alter a current pattern or preview/usefulness result is a
  stop condition and belongs to Phase 17/18 policy work.
- Core/game_server validate from one repository state. Unity/runtime probes are omitted unless inspection finds a
  serialized shape or message-order change, which triggers a policy stop.

## Validation Plan

- Baseline: tile-range unit tests, selector/manual-cast paths, range preview, area runtime/usefulness and current live
  RON/skill suites.
- First proof: cross product of policy/pattern × facing × anchor inclusion × center/edge/void board with stable ordered
  cells and `contains(tile) == materialized_cells.contains(tile)` after clipping.
- Focused final: same cross product through the typed query plus representative production consumers.
- Broad: complete affected Core integration families, Core lib, game_server targets and release.
- Static: caller/adapter inventory, duplicate transform/clipping search, public/serde/save/wire comparison, scoped
  status/diff/hashes, canonical comparison, formatter and diff check.
- Zero-test filters, helper-only predicates, compile-only success and snapshot-only updates do not count as proof.

## Completion Evidence Matrix

| ID | Condition | Required evidence | Status |
| --- | --- | --- | --- |
| P16-C01 | complete producer/caller graph identified | policy/pattern/facing/clipping adapter inventory | complete |
| P16-C02 | dirty targets and concurrent work preserved | pre/post status, scoped diff and hashes | complete |
| P16-C03 | current geometry is frozen before refactor | nonzero cross-product characterization | complete |
| P16-C04 | one typed membership/materialization authority exists | source/caller and duplicate-adapter audit | complete |
| P16-C05 | production consumers use the authority without semantic collapse | selector/cast/preview/area integration parity | complete |
| P16-C06 | valid ordering/clipping/filter invariants remain exact | edge/void/facing/whole-field positive matrix | complete |
| P16-C07 | public/data/save/wire compatibility remains exact | shape/live-content/consumer audit | complete |
| P16-C08 | focused and broad validation passes | nonzero Core/live/server/release/static counts | complete |
| P16-C09 | Phase 17/18 seam and canonical are current | downstream owner and no-prebuild audit | complete |
| P16-C10 | phase-exit review is Complete / High | `COMPLETION_AUDIT.md` | complete |

## Stop Conditions

Stop and do not activate Phase 17 if callers currently disagree on geometry/order/clipping; choosing between them
would change behavior; cast reach and delivery footprint cannot stay distinct; a public/RON/save/wire migration is
required; live content must change; obstacle/occupancy filtering becomes necessary; dirty overlap is inseparable; a
required runner selects zero tests; or completion review is not `Complete / High`.

사용자와 의논하여 정해야 할 정책이 있을 경우 Goal을 종료한다.
