# Phase 39 CRA-P07-C03 Experiment Notes

Status: complete
Implementation Complete: true

## Scope Decision

- This Child owns exactly CRA-P07-C03: assertion transfer and consumer-gated consolidation to the existing
  production-default Rapier movement backend.
- It preserves Phase 08 final tactical aggregation, Phase 09 staged world publication and completed Boss Runtime
  target/route/directive authority.
- It does not pull forward Phase-40 ghost movement fields/outputs or later cleanup.

## Authority Boundary

- Prospective retained authority: default Rapier-backed `BattleCore` continuous movement plus neutral production
  clamp/static-obstacle policy.
- Prospective false authority: Direct implementation, backend selector, parity-as-authority tests and test-only
  Rapier wall/introspection surfaces, subject to current discovery.
- Target: tests and runtime use one production movement leaf; Boss retains intent/route ownership and delegates
  translation without a Boss-specific backend.

## Direct Dependency Decisions

- Phase 08 derives tactical effects from full final state and does not change backend ownership.
- Phase 09 publishes tactical world mutations atomically and leaves movement semantics exact.
- Boss P03/Master own target lock, route cursor, directive/suppression/fallback and ordinary-leaf integration; P07-C03
  remains a separate cleanup rather than a Boss prerequisite or superseded candidate.
- Phase 38 is the direct linear predecessor at `Complete / High` and creates no movement placeholder.

## Compatibility Decision

- CRA-POL-006 is binding. Workspace-only absence cannot authorize public movement API removal.
- Private consolidation may proceed when it does not narrow public/API behavior, but Phase completion cannot retain a
  false public backend solely to avoid requesting a required decision.
- If current inventory proves public narrowing is required and deployment remains unknown, transfer assertions and
  record an exact policy question. Do not add a shim or generalize Phase-37/38 approvals.

## Required First Proof

The cleanup cannot begin with deletion. Every one of the historical 17 Direct-dependent tests must be named and its
assertions classified. Unique behavior must pass through retained Rapier/default production Core, including actual
authored-route and Boss ordinary-leaf seams where applicable.

## Dirty Baseline

- Pre-Child expanded status: 573 rows.
- Canonical C-sort SHA-256:
  `a086d6ad75aae503d2da3649bc84bf8d38605e21b0466c9b5f15ed1c6251b149`.
- Phase 39/40 paths absent; active implementation Child count 0.
- Protected marker remains 57 bytes at SHA-256
  `614d2fbdbdb0b29def7b90d1fdbc65148aba63339eea72fef5998d4a8ec22730`, with exact recorded timestamps.
- Existing status rows and concurrent changes are user/campaign-owned.

## Current Evidence Boundary

At initialization, only historical/final documents and completed dependency audits have been read. No current
Phase-39 production file content, symbol graph, test body, manifest consumer, data or canonical has been inspected.
Those facts remain `discovery pending` until sole activation, full Child/dependency reread and prospective
fingerprints.

## Sole Activation — 2026-08-11

- Phase 39 is the sole active Child; Master Phase 39 is `active`, and Phase 40 remains pending and absent.
- The three initialized files are the only status-set additions: 576 rows at canonical C-sort SHA-256
  `93625e86d7ea3bce8539875054846dc9d48b1aea3e82526d6c060bfc1326376e`.
- No current production content was inspected during activation. Full active-document/dependency reread and
  filename-only prospective fingerprints remain mandatory before discovery.

## Prospective Fingerprint Gate — 2026-08-11

- Active Child and all direct technical/Boss/linear audits plus historical/final candidate records were reread after
  activation.
- The prospective alternate-backend and assertion carriers (`engine.rs`, `rapier_backend.rs`, `blocking.rs`,
  `planner.rs`) are already dirty; activation hashes and line counts now bound any attributable hunk.
- Movement `mod.rs`, `lifecycle.rs`, `path.rs`, `steering.rs`, `types.rs` and battlefield field are clean. They are not
  presumptive cleanup targets merely because they are adjacent.
- Boss/Core/world/range consumers and canonicals are already dirty from completed work and must be preserved unless
  current evidence requires a narrow test-only transfer.
- Current content discovery is now authorized; production mutation still waits for complete backend/test/consumer
  inventory, nonzero baselines and assertion-transfer design.

## Current 17-Assertion Disposition — 2026-08-11

| # | Current Direct-dependent carrier | Retained disposition before cleanup |
| --- | --- | --- |
| 1 | `movement_backends_canonicalize_unit_order_before_resolution` | add a retained Rapier test comparing complete results for reversed and canonical input order, plus first emitted ID |
| 2 | `movement_backends_match_ground_static_obstacle_stop_policy` | existing `rapier_ground_static_obstacle_does_not_slide_along_blocking_wall` carries the absolute stop/no-slide invariant; Direct-distance parity is false authority |
| 3 | `movement_backends_match_airborne_static_obstacle_policy` | existing retained Rapier airborne-obstacle test carries exact bypass behavior; backend equality is false authority |
| 4 | `movement_backends_match_board_clamp_policy` | strengthen retained ground Rapier board test to the exact `(0.35, 0.35)` endpoint; backend equality is false authority |
| 5 | `direct_engine_moves_toward_point_goal` | strengthen retained Rapier point-goal test with the exact `(1.0, 0.0)` velocity assertion |
| 6 | `direct_engine_allows_moving_candidates_to_overlap` | add retained Rapier moving-candidates test with equal exact endpoints |
| 7 | `direct_engine_allows_mover_to_overlap_locked_unit_body` | existing `rapier_engine_ignores_unit_colliders_when_correcting_translation` is stronger: the mover passes through a locked unit body to the full endpoint |
| 8 | `direct_engine_clamps_movement_to_board_bounds` | same strengthened retained exact ground-clamp control as #4 |
| 9 | `direct_engine_allows_overlapping_units_at_board_bounds` | add retained Rapier boundary-overlap control proving no depenetration and no movement output |
| 10 | `direct_engine_static_obstacle_blocks_swept_movement` | existing retained Rapier large-step static-obstacle correction test proves no tunneling before the expanded obstacle edge |
| 11 | `direct_engine_airborne_policy_ignores_static_obstacles` | existing retained Rapier airborne-obstacle exact endpoint survives |
| 12 | `direct_engine_airborne_policy_still_clamps_to_board_bounds` | existing retained Rapier airborne exact clamp survives |
| 13 | `direct_engine_reports_attack_target_reached` | add retained Rapier `TargetReached` identity assertion |
| 14 | `battle_core_can_run_one_direct_continuous_tick_with_explicit_goal` | convert the existing Rapier Core twin into an unselected default-Core control, then retire the Direct duplicate only after the default control passes |
| 15 | `battle_core_records_immutable_movement_segments_per_tick` | remove its Direct selector so the complete existing immutable/adjacent timestamp assertions run on default Core |
| 16 | `battle_core_does_not_record_movement_segment_without_body_motion` | remove its Direct selector so the negative publication assertion runs on default Core |
| 17 | `authored_defense_route_enemy_progresses_past_spawn_boundary` | remove its Direct selector so route progress and `MovementSegmentStarted` are proved through actual production-default Core |

This matrix accounts for every historical carrier. Existing survivors are accepted only where their absolute
Rapier assertion is at least as strong as the unique behavior; Direct/Rapier equality itself is never retained as
an authority.

## Current Public/Consumer Gate — 2026-08-11

- Production has one actual backend choice, but the enum, Direct implementation, Rapier types, Rapier selector and
  inspection/sync surface are publicly reachable through public modules.
- In-tree consumers and accessible `/mnt/f/work` Rust consumers are zero outside the implementation/tests, and
  `game_server` is the only path dependency. Historical docs are references, not consumers.
- The package is not manifest-marked non-publishable and no external/deployed consumer inventory exists. Therefore
  the target public removal still requires exact Phase-39 approval or a migration/deprecation lifetime.
- Assertion transfer does not narrow that public surface and is authorized now. No public symbol, module or carrier
  will be removed before the transferred retained controls pass and the policy decision is recorded.

## Retained Transfer Result And Stop — 2026-08-11

- All planned retained controls pass before cleanup: Rapier 19/19, engine 19/19, default authored route 1/1 and Boss
  movement family 25/25.
- The old Direct/parity suite remains present, so the green result proves assertion transfer before carrier deletion
  rather than only testing a post-removal replacement helper.
- Core immutable/no-motion segment publication and authored-route progress now exercise the actual unselected
  production default. The Rapier suite carries every unique engine-level assertion identified in the 17-row matrix.
- No public narrowing has occurred. Phase 39 is `policy_decision_required` for the exact breaking cleanup described
  in `PLAN.md`; no Phase-40 work is authorized.

## Cleanup Approval And Resume — 2026-08-11

- The user approved the exact Phase-39 public breaking cleanup and all later registered cleanup candidates. Phase 39
  resumes as the sole active Child with no shim/deprecation obligation.
- Approval is removal authority, not evidence waiver: delete only the inventoried alternate backend/selector/wall/
  inspection surface after retained proof, preserve every movement/Boss/world invariant, and complete focused/broad
  validation plus independent audit before Phase 40.
- Later-phase advance approval does not pull their code into this Child. In particular, Phase 43 must preserve the
  accepted tile-range candidate filter and current `Nearest` ordering while deciding only the ghost `range_units`
  and false public helper/serialized surfaces after its own inventory.

## Cleanup Result — 2026-08-11

- `BattleCore` now stores the sole Core-private Rapier world directly. There is no backend enum, Direct
  implementation, selector, fallback, public backend trait, test-only board world or public Rapier inspection API.
- The manual clamp is a neutral private helper with unchanged arithmetic. Retained Rapier tests still own the exact
  collision/clamp/overlap/terrain assertions; default-Core tests own event publication, authored route and repeated
  two-tick log/final-state determinism.
- Cleanup removed 16 obsolete test carriers and added one stronger default-path control. The 17-row assertion matrix
  above remains the semantic accounting: rows 1~14 lost only their obsolete Direct/parity carriers, while rows
  15~17 continue unchanged on the unselected production default.
- Debug and release all-target totals are both 1,056/1,056. Movement 61/61, Boss 25/25, tactical 34/34, range
  52/52, event 76/76, world 293/293, Rapier 17/17 and the exact authored route 1/1 are green.
- No live data, serialized DTO/event shape, save, server/Unity or canonical contract changed. The protected marker
  and all recorded non-target hashes remain exact.

## Out-Of-Scope Handling

Independent defects or improvements discovered during current inspection are recorded here with evidence and an
owner, but are not implemented, reordered or turned into new Goals automatically.
