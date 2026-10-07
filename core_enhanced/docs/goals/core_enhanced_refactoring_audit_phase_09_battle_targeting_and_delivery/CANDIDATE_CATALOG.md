# Core Enhanced Refactoring Audit Phase 09 — Candidate Catalog

Catalog Date: 2026-08-07
Status: complete; dispositions accepted by phase-exit audit `Complete / High`

Axes are ownership duplication, correctness/atomicity, lifecycle complexity, change amplification, test fragility,
failure blast radius and change frequency. Current dirty work makes frequency `unknown`; known subtotals do not
automatically decide disposition.

## Confirmed Candidates

### CRA-P09-C01 — Close Projectile Publication And Basic-Delivery Compatibility

- Work Type: `correctness_correction`
- Problem Evidence: projectile speed is an unrestricted `u32`; zero becomes zero flight instead of publication
  failure. Programmatic skill construction can accept zero optional limits that RON rejects. Basic definitions and
  weapon profiles accept different skill-only projectile policies that basic runtime silently ignores.
- Current Authority And Consumers: `DeliveryDef`, `BasicAttackDef`, `WeaponCombatProfile` and deserializers each own a
  partial rule; basic and skill runtimes defensively interpret the result.
- Target Authority: one typed positive scalar and delivery/consumer compatibility matrix validates identical RON and
  programmatic inputs, with field-specific typed errors. Basic attacks accept only target-locked fields they execute.
- Preserved Invariants: every current live bundle loads; positive flight uses current ceiling math; basic weapon
  override and skill TargetLocked/DirectionalCollision behavior stay exact; no implicit instant fallback is added.
- Removal Or Consolidation Scope: remove zero-speed fallback as an accepted contract, duplicated nonzero checks,
  misleading generic error text and silently ignored basic projectile fields. Runtime defensive guards may remain.
- Priority Axes And Total: `2 / 3 / 2 / 3 / 3 / 2 / unknown`; known subtotal `15/18`.
- Validation Safety Net: table-test zero speed and every optional zero limit through RON and `SkillDatabase::new`;
  reject DirectionalCollision/targetless/lifetime/kill/pierce policies for basic and weapon delivery with exact paths;
  reload all live data and retain positive flight boundaries.
- Policy/External Contract Decision: rejecting formerly constructible invalid programmatic values and skill-only basic
  fields is compatibility-significant; approve it as a publication correction before implementation.
- Readiness: `test_first_then_compatibility_approval`.
- Disposition: `priority_candidate`.

### CRA-P09-C02 — Resolve Max-Kills Projectile Delivery Exactly Once

- Work Type: `correctness_correction`
- Problem Evidence: a directional projectile reaching `max_kills` is removed by stop policy after a nonterminal
  impact was queued. No terminal step resolution follows, so `pending_delivery_count` remains one and the active cast
  leaks indefinitely. The current test omits the cast-state assertion.
- Current Authority And Consumers: projectile advancement owns motion/hits; kill-stop mutates the active projectile;
  `resolve_skill_step_delivery` alone owns cast pending-count and final cleanup.
- Target Authority: stop-policy evaluation returns a typed terminal outcome consumed by one delivery-resolution path;
  projectile removal and pending-count decrement happen exactly once for the same impact.
- Preserved Invariants: first accepted collision/effect ordering, max-kill exclusion, deterministic identity, impact
  event cause, source snapshot, repeat/step sequencing and no second terminal resolution.
- Removal Or Consolidation Scope: remove the side-effect-only kill-stop path and stale `terminal = false` handoff.
  Do not change collision geometry, damage/effects or authored kill semantics.
- Priority Axes And Total: `1 / 3 / 3 / 2 / 3 / 2 / unknown`; known subtotal `14/18`.
- Validation Safety Net: extend the existing max-kills test to assert projectile absence, active-cast absence and one
  step/cast completion; add same-timestamp multiple-collision and max-kills/max-hits/expiry boundary cases.
- Policy/External Contract Decision: none; this restores the existing terminal-delivery lifecycle.
- Readiness: `test_first_then_implementation`.
- Disposition: `priority_candidate`.

### CRA-P09-C03 — Make Skill Usefulness Evaluate Actual Delivery Targets

- Work Type: `correctness_correction`
- Problem Evidence: EnemySingle TileArea acquisition/autocast checks the anchor unit, while execution later applies
  footprint, board clipping, filter and effect usefulness. Schema-valid footprints excluding the anchor can cast to
  nobody or reject an area containing useful targets. Current live footprints include anchors.
- Current Authority And Consumers: target-usefulness helpers choose one representative target; area runtime owns the
  actual affected set. Automatic/manual acquisition and step no-op gates consume inconsistent views.
- Target Authority: a typed executable-usefulness plan resolves the exact delivery target set for the candidate
  anchor and is shared by acquisition, autocast and step gating.
- Preserved Invariants: current live target choices, stable unit ordering/ties, target filters, board clipping,
  damage/heal/buff usefulness and no-target/no-op semantics.
- Removal Or Consolidation Scope: remove anchor-only TileArea shortcuts and the first-hostile-step approximation where
  it substitutes for actual delivery. Do not make Phase 8 effect application part of targeting.
- Priority Axes And Total: `3 / 2 / 2 / 3 / 2 / 2 / unknown`; known subtotal `14/18`.
- Validation Safety Net: target-centered footprints excluding anchor, clipped cells, useless anchor/useful neighbor,
  useful anchor/empty area, friendly/hostile filters and multi-step usefulness; golden current live target choices.
- Policy/External Contract Decision: multi-step skills need an explicit rule for whether any executable step or only
  the next step gates cast usefulness. Freeze it before replacing the current first-step heuristic.
- Readiness: `tests_and_multi_step_policy_first`.
- Disposition: `candidate`.

### CRA-P09-C04 — Project Skill Effect Cells From Delivery Footprints

- Work Type: `correctness_correction`
- Problem Evidence: self-target range preview publishes cast acquisition cells as `effect_preview_cells`, ignoring
  TileArea footprints. Live non-player skills demonstrate 3x3/5x5 delivery behind a front-3x2 cast reach.
- Current Authority And Consumers: `range_preview.rs` derives checkpoint projection; skill runtime owns actual
  footprint/tracking/filter. Phase 10 transports the derived DTO.
- Target Authority: one delivery-aware preview projection returns actual step effect cells from the same footprint
  query used by runtime, while cast reach remains a separate field.
- Preserved Invariants: valid-board clipping, facing transforms, no obstacle filtering, current player nonspatial
  previews and runtime effect geometry. Projection must not mutate cast state.
- Removal Or Consolidation Scope: remove the cast-reach-as-effect-footprint fallback and duplicate footprint adapter.
  Do not redefine DTO shape or UI presentation in this Phase.
- Priority Axes And Total: `2 / 2 / 1 / 3 / 3 / 2 / unknown`; known subtotal `13/18`.
- Validation Safety Net: preview/runtime cell parity for 3x3 and 5x5, edges and all facings; Instant/Projectile have
  no area footprint; add a multi-step differing-footprint fixture after projection policy is chosen.
- Policy/External Contract Decision: Phase 10/direct consumer must decide per-step versus union projection and DTO
  compatibility. Current live self-spatial skills do not force the choice.
- Readiness: `after_C06_and_phase_10_projection_decision`.
- Disposition: `candidate`.

### CRA-P09-C05 — Extract One Continuous Projectile Motion Kernel

- Work Type: `behavior_preserving_refactor`
- Problem Evidence: basic and skill projectiles duplicate 1ms reevaluation, homing movement, continuous segment
  sweep, hit-time ceiling, interpolated impact position and expiry/next-event decisions.
- Current Authority And Consumers: two runtime records legitimately own different IDs, filters, stop policies and
  payloads; both call equivalent geometry/time snippets before separate effect execution.
- Target Authority: a pure typed motion query accepts start/end/time/collision candidates and returns ordered impact,
  position and next-time facts. Basic/skill lifecycle adapters retain their own payload and terminal policy.
- Preserved Invariants: target-locked versus directional paths, exact rounding, target-only basic sweep, ordered skill
  collisions, Pause, source/target snapshots, expiry and event identity.
- Removal Or Consolidation Scope: remove duplicate movement/sweep/timing code only. Do not merge records, hit limits,
  C02 terminal policy or Phase 8 committed-hit orchestration.
- Priority Axes And Total: `3 / 1 / 3 / 3 / 2 / 2 / unknown`; known subtotal `14/18`.
- Validation Safety Net: table/golden parity for zero distance, sub-ms and exact-ms hits, moving target, multiple
  bodies, board exit, expiry and relocation; compare public effects/events, not private runtime record shape.
- Policy/External Contract Decision: coordinate with Phase 8 C05 so motion and committed damage remain separate
  authorities. Public record visibility needs Phase 10/external inventory before narrowing.
- Readiness: `after_C01_C02_and_phase_8_damage_boundary`.
- Disposition: `priority_refactor_candidate`.

### CRA-P09-C06 — Centralize Typed Tile Reach Queries And Projections

- Work Type: `behavior_preserving_refactor`
- Problem Evidence: runtime target membership, manual skill reach, range preview and area materialization each repeat
  policy/pattern/facing/valid-tile interpretation. They agree today but already allowed C04 projection drift around
  which policy is being projected.
- Current Authority And Consumers: `TileRangePolicy`/`TileRangePattern` are data authority; four adapters independently
  resolve membership or cells against Phase 7 board state.
- Target Authority: one typed query exposes explicit `contains(tile)` and `materialize_valid_cells()` operations plus
  captured-origin/facing inputs, reused by selectors, cast validation, preview and area runtime.
- Preserved Invariants: current pattern geometry, anchor inclusion, facing transforms, valid-board clipping, stable
  ordering and intentional lack of obstacle/occupancy filtering.
- Removal Or Consolidation Scope: remove duplicate switch/transform/clipping loops, not the distinction between
  membership and materialized cells or between cast reach and delivery footprint.
- Priority Axes And Total: `3 / 1 / 1 / 2 / 2 / 2 / unknown`; known subtotal `11/18`.
- Validation Safety Net: cross-product parity for every range pattern/policy, all facings, edge clipping and membership
  equals set containment; retain production selector and preview integration tests.
- Policy/External Contract Decision: none for internal query extraction; DTO choice remains C04/Phase 10.
- Readiness: `characterization_then_implementation`.
- Disposition: `refactor_candidate`.

### CRA-P09-C07 — Retire Ghost Range And False Public Delivery Surfaces

- Work Type: `cleanup`
- Problem Evidence: basic/weapon `range_units` is serialized, copied and validated but not read for eligibility;
  `bodies_in_range` has no production caller. Public `basic_attack_delivery(base_uuid)` has no workspace caller and
  can disagree with runtime weapon override. Several runtime records/helpers are wider than workspace use requires.
- Current Authority And Consumers: tile range policy and runtime unit equipment are actual authorities; the ghost
  fields/methods remain public or serialized and may have external consumers.
- Target Authority: tile policy alone owns eligibility; runtime unit profile owns delivery; internal records expose
  only the visibility required by Core and behavior projection.
- Preserved Invariants: all current target sets and release behavior, weapon override, serialized-version policy and
  any verified server DTO/UI display requirement.
- Removal Or Consolidation Scope: remove or migrate `range_units`, unused range helper, base-UUID delivery helper and
  unjustified public visibility only after consumer inventory. Do not silently change live RON shape.
- Priority Axes And Total: `2 / 1 / 1 / 3 / 2 / 1 / unknown`; known subtotal `10/18`.
- Validation Safety Net: workspace plus external Rust/server symbol inventory, generated fixture diff, live RON
  migration/strict-load tests and target-set golden tests before/after cleanup.
- Policy/External Contract Decision: Phase 10 must confirm DTO/server use; serialized removal requires explicit
  compatibility/version approval; Unity remains unverified.
- Readiness: `blocked_on_phase_10_and_external_compatibility_inventory`.
- Disposition: `deferred_cleanup_candidate`.

### CRA-P09-C08 — Make Negative Target-Schema Tests Prove Their Rejection Cause

- Work Type: `correctness_correction`
- Problem Evidence: the inline-range/preset-conflict test catches any panic, but its fixture currently panics first on
  missing `delay_ms`. Several legacy-authority negative RON fixtures assert only `is_err` while omitting other required
  step fields, so unrelated schema failure can keep them green.
- Current Authority And Consumers: production typed deserialization/validation owns rejection; tests currently treat
  incidental parse failure as evidence of a specific contract.
- Target Authority: minimal otherwise-valid fixtures reach exactly one invalid boundary and assert typed object,
  field/path and cause. Shared builders keep all unrelated required fields current.
- Preserved Invariants: strict unknown/conflicting targeting rejection and current live schema. No production error
  is weakened to satisfy a test.
- Removal Or Consolidation Scope: replace catch-any-panic and bare `is_err` assertions; merge repeated valid boilerplate
  into a fixture helper. Remove a case only when a stronger case proves the same path/cause.
- Priority Axes And Total: `1 / 3 / 1 / 2 / 3 / 1 / unknown`; known subtotal `11/18`.
- Validation Safety Net: each case first parses when the one invalid field is repaired, then fails with the expected
  typed path/cause when restored; ensure filters execute nonzero tests.
- Policy/External Contract Decision: none; this corrects the safety net rather than gameplay.
- Readiness: `test_only_correction_ready`.
- Disposition: `test_correction_candidate`.

## Required Ordering

1. C02 begins with the missing cast-cleanup assertion and can be corrected independently.
2. C01/C08 establish publication and trustworthy negative-schema safety nets.
3. C06 characterizes one spatial query; C03 then uses it after the multi-step usefulness rule is frozen.
4. C04 follows C06 and Phase 10's projection-shape decision.
5. C05 follows C01/C02 and preserves the Phase 8 committed-damage boundary.
6. C07 follows Phase 10 and explicit external/serialized compatibility inventory.

## Rejected, Deferred And Unverified Signals

| Signal | Disposition | Evidence/reason |
| --- | --- | --- |
| Basic target selector and release recheck | `retain` | one tile-policy authority, deterministic ties and substantive tests |
| Manual target capture and explicit retarget | `retain` | no implicit fallback; captured identity/tile matches tests/canonical |
| TargetLocked versus DirectionalCollision distinction | `retain` | independent homing/fixed collision contracts, not duplicate policy |
| GroundFixed and Follow area modes | `retain` | explicit snapshot/tracking lifetimes and focused withdrawal/relocation tests |
| Released effects removed on caster withdrawal | `rejected_stale_canonical` | production and two nearer canonicals preserve released/fixed effects |
| Obstacle-aware targeting | `policy_or_feature_migration` | current policy deliberately uses board/range, not obstacle pathing |
| Live usefulness incident | `not_observed` | current EnemySingle TileArea footprints include anchors |
| Live player preview incident | `not_observed` | mismatching self-area skills are not current player active fragments |
| Phase 8 damage/effect executor | `deferred_phase_08_C05` | Phase 9 supplies motion/hit facts only |
| Server/DTO range meaning | `deferred_phase_10` | direct consumer not yet audited |
| Unity target/range consumers | `unverified_out_of_scope` | repository explicitly excluded |
