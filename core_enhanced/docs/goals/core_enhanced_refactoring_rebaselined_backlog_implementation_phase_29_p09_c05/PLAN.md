# Phase 29 Goal — CRA-P09-C05 One Continuous Projectile Motion Kernel

Status: complete
Implementation Complete: true
Goal Type: phase
Execution Mode: implement_and_verify
Candidate ID: CRA-P09-C05
Master Phase: 29 of 44
Work Type: behavior-preserving refactor
Production Mutation Authorized: yes — limited to Phase 29 after the Master sole-active transition, complete
required-document reread and prospective fingerprint capture; no Phase 30 work is authorized
Repository Root: `/mnt/f/work/simulator`
Initialization Date: 2026-08-10 (`Asia/Seoul`)

## Objective And Completion Conditions

Consolidate the duplicated continuous projectile movement, sweep, hit-time, interpolated impact-position and
next-event/expiry calculations used by basic and skill projectiles into one private pure typed motion authority.
Basic and skill lifecycle adapters must retain their distinct runtime IDs, guidance/collision policies, candidate
selection, payloads, effects, stop policies, event identity and terminal-delivery ownership.

Preserve exact zero-distance, sub-millisecond and exact-millisecond rounding; live moving-target sampling; target-
only basic sweep; homing TargetLocked versus fixed DirectionalCollision paths; ordered multi-body skill collision;
board exit, expiry and next-reevaluation timing; Pause/battle-time behavior; source/target snapshots; deterministic
per-hit identity; Phase 14 exactly-once terminal delivery; Phase 28 committed-hit/effect authority; and Boss
released-work behavior.

This Phase is complete only when the current two motion/state/event graphs are rediscovered, a shared behavior table
characterizes both adapters before refactoring, one pure typed kernel owns only common motion facts, all adapter-
specific lifecycle remains explicit, current public/serialized surfaces are inventoried and retained under
CRA-POL-006, focused and broad Core/server debug/release validation passes, no Phase 30 work is prebuilt, and an
independent review returns `Complete / High`.

## User Or Contract-Visible Problem

Historical and post-Boss audits found two independently maintained implementations of the same continuous-motion
mechanics. Basic and skill projectiles each reevaluate every millisecond, sample moving bodies, advance a homing or
fixed segment, sweep continuously, ceil fractional hit time, interpolate impact position and choose expiry or the
next event. Their payload and lifecycle policies legitimately differ, but duplicated geometry/time code can drift.

The user-visible result must not change. The same projectile inputs must yield the same time, position and collision
facts as before, while basic remains a locked-target committed attack and skill projectiles retain directional/
homing multi-hit, cast-pending and authored stop-policy behavior.

## Dependency And Evidence Authority

- Master: `../core_enhanced_refactoring_rebaselined_backlog_implementation_master/PLAN.md`, Phase 29 registry row.
- Direct linear predecessor and committed-hit seam: Phase 28 CRA-P08-C05, final 320-line `Complete / High` in
  `../core_enhanced_refactoring_rebaselined_backlog_implementation_phase_28_p08_c05/COMPLETION_AUDIT.md`.
- Required terminal-delivery predecessor: Phase 14 CRA-P09-C02, final 246-line `Complete / High` in
  `../core_enhanced_refactoring_rebaselined_backlog_implementation_phase_14_p09_c02/COMPLETION_AUDIT.md`.
- Required publication predecessor: Phase 15 CRA-P09-C01, final 229-line `Complete / High` in
  `../core_enhanced_refactoring_rebaselined_backlog_implementation_phase_15_p09_c01/COMPLETION_AUDIT.md`.
- Boss consumer seam: the completed private Boss runtime delegates released basic/skill projectile work to ordinary
  leaves; its final completion audit must be reread during activation and current source verified after the gate.
- Historical/final evidence:
  - `../core_enhanced_refactoring_audit_phase_09_battle_targeting_and_delivery/{CANDIDATE_CATALOG,TEST_AUDIT,BOUNDARY_AUDIT}.md`
  - `../core_enhanced_refactoring_post_boss_runtime_reaudit_phase_09_battle_targeting_delivery_and_projectiles/{CANDIDATE_REAUDIT,TEST_REAUDIT,BOUNDARY_AUDIT}.md`
  - `../core_enhanced_refactoring_post_boss_runtime_reaudit_final_catalog_consolidation/{REBASELINED_CANDIDATE_CATALOG,EXACT_LINEAR_IMPLEMENTATION_ORDER,TEST_REBASELINE_CATALOG,IMPLEMENTATION_WAVES_AND_GATES,APPROVED_POLICY_DECISIONS}.md`

## Approved Policy And Historical Disposition

- CRA-POL-003: projectile speed and every present optional limit are strictly positive before atomic publication;
  basic/weapon definitions reject policies their runtime cannot execute. The motion kernel must not add zero-speed
  normalization, instant fallback or a permissive runtime publication path.
- CRA-POL-006: current public Rust, RON, save, DTO, wire, package and external-consumer compatibility is preserved.
  Private consolidation cannot narrow runtime records or public helpers on workspace-only evidence.
- Historical priority axes: `3 / 1 / 3 / 3 / 2 / 2 / unknown`; known subtotal `14/18`.
- Final readiness: Tier B after P09-C01, P09-C02 and P08-C05.
- Reaudited disposition: `still_valid`, normalized as `behavior_preserving_refactor`; Boss adds a required released-
  work consumer but no competing projectile implementation.
- Open policy decisions: none. A newly required change to rounding, collision, targeting, terminal lifecycle,
  public shape or published-data meaning stops this Phase.

## Historical Current And Target Authority

- Historical basic path: one target-locked runtime record owns homing advance, live-target sweep, impact/miss and
  scheduling before handing a hit to committed basic damage.
- Historical skill path: a separate runtime record owns TargetLocked/DirectionalCollision advance, candidate
  ordering, hits/pierces/kills/expiry and cast delivery before handing effects to skill execution.
- Historical shared mechanics: one-millisecond reevaluation, segment travel, continuous sweep, fractional impact
  and ceiling time, interpolated position and next/expiry decision.
- Target authority: one private pure typed query consumes already-authoritative motion inputs/collision candidates
  and returns motion/ordered-impact/position/next-time facts without mutating runtime state.
- Preserved owners: adapters retain target/candidate selection, IDs, payloads, event emission, hit bookkeeping,
  removal/reinsertion, terminal outcome, cast pending count and damage/effect execution.

These are historical pointers only. Current production after activation is authoritative and may refine the exact
module, input/output type and which facts can genuinely be shared.

## In Scope

- complete basic-projectile and skill-projectile launch/advance/impact/expiry scheduling graphs;
- common time step, travel distance, homing/fixed endpoint, segment sweep, hit fraction, ceiling time and impact
  interpolation mechanics;
- collision-candidate input/output ordering and representation needed by both adapters;
- zero distance, sub-ms, exact-ms, moving body, relocation, board exit, expiry and same-time boundaries;
- basic target-only versus skill multi-body filter/order and TargetLocked versus DirectionalCollision distinction;
- current runtime/public visibility, serde/save/checkpoint/server/Unity and unknown consumer inventory;
- Phase 14 stop/terminal delivery, Phase 28 committed hit, Boss released work and Pause behavior as preserved seams;
- characterization-first focused tests, debug/release broad validation and independent completion audit.

## Out Of Scope

- changing projectile speed, collision radii, reevaluation cadence, ceiling/interpolation math or board-exit rules;
- merging basic and skill runtime records, projectile IDs, payloads, source snapshots or event variants;
- changing target acquisition, retarget, guidance policy, collision filters, stable ordering or target validity;
- changing max-hits, max-kills, max-pierces, expiry, non-piercing or cast pending/terminal semantics;
- changing Phase 28 formula, trigger, resonance, HP/death or committed-damage ordering;
- changing skill effects, TileArea lifecycle, range/usefulness/preview or Phase 7 movement backend;
- changing public Rust visibility, DTO/event tags, serialization, V3/checkpoint, live RON, server/Unity shape;
- adding fallback, dual motion paths, compatibility aliases, silent normalization or test-only production branches;
- Phase 30 CRA-P10-C01 implementation or future command-route placeholders.

## Required Test-First Characterization Matrix

| Case | Common motion proof | Adapter-specific proof | Preserved exclusion |
| --- | --- | --- | --- |
| zero distance | immediate exact position/time fact | basic target hit; skill accepted candidate semantics | no divide/loop fallback |
| sub-ms and exact-ms | ceiling hit time and interpolated impact point exact | public impact event time/position identity | no rounding policy change |
| moving target | live body position changes each reevaluation | basic/skill TargetLocked homing parity | launch snapshot remains source-only |
| fixed directional | segment/board progression exact | skill DirectionalCollision alone uses fixed aim/path | basic never gains directional policy |
| multiple bodies | stable ordered impact facts | skill filters, pierce/hit/kill rules own acceptance | basic sweeps locked target only |
| relocation/withdraw/death | current adapter validity rules stay exact | hit/miss/exclusion and released-work behavior | kernel does not own lifecycle |
| board exit/expiry/same-time | next time, terminal motion and expiry boundary exact | one adapter resolution and event ordering | Phase 14 owns cast completion |
| committed effects | motion fact hands off once | basic to Phase 28; skill to existing effect/cast path | kernel never executes effects |
| Boss release/cancel | ordinary released leaf continues | unreleased work cancels, released projectile survives | no Boss-specific motion fork |

Tests must compare observable event/effect/state facts or pure numeric facts shared by both adapters, not require a
future helper name or merge their runtime records. Every new behavior characterization must pass on old production
before the production extraction.

## Implementation Constraints

1. Capture current expanded status, HEAD and prospective source/test/data/canonical hashes before production-detail
   inspection.
2. Inventory both complete producer/runtime/event/scheduling/effect paths and their public/serialized consumers.
3. Classify every candidate input and output as shared motion fact or adapter-owned lifecycle fact.
4. Run nonzero old-production focused baselines and add the common parity matrix before refactoring.
5. Prefer a pure typed query in the existing projectile geometry/math domain; no BattleCore mutation in the kernel.
6. Preserve exact integer/floating conversion, ceiling and quantization semantics from current production.
7. Keep candidate filtering, stable acceptance order and terminal policy in the appropriate adapter unless current
   evidence proves a pure fact is truly identical.
8. Preserve one scheduling/removal/reinsertion and one impact/effect handoff per adapter.
9. Retain public/serialized surfaces under CRA-POL-006; remove only proven private duplicate computation.
10. Stop for independent audit before Phase 30 activation.

## DTO, Data, Save And Deployment

- DTO/serde migration: none authorized; existing projectile impact/event/checkpoint shapes remain exact.
- Live RON migration: none; CRA-POL-003-published positive delivery definitions remain byte-compatible.
- Save/checkpoint migration: none; projectile runtimes remain private battle-lifetime state unless current evidence
  proves otherwise, in which case the Phase stops before changing shape.
- Public Rust deployment: inventory runtime/helper visibility but retain it absent migration evidence.
- Server/Unity deployment: no wire/version/rollout change is authorized.
- Boss deployment: private Boss runtime continues to cancel only unreleased work and delegate released leaves.

## Validation Strategy

- Baseline: projectile math, complete basic projectile/commands, skill projectile/runtime contract, released-work,
  live RON/catalog and Boss projectile families with nonzero counts.
- Test-first: common numeric and observable adapter parity matrix on pre-refactor production.
- Focused implementation: rerun every basic/skill motion, collision, stop, committed-hit, skill-effect and Boss leaf
  family in debug, with representative release tests in release.
- Broad Core: complete debug/release official unit/integration tests and all-target checks.
- Broad server: complete debug/release tests/checks because projectile events remain public consumers.
- Static: one pure motion authority, no duplicate sweep/timing/interpolation helpers, two explicit lifecycle adapters,
  no public narrowing/fallback/Phase 30 seam.
- Compatibility: live RON and source/data/canonical/server/Unity fingerprints and consumer reads.
- Diagnostic: non-strict Clippy may inventory established debt; strict warning denial is not claimed unless green.

## Completion Evidence Matrix

| ID | Condition | Required evidence | Status |
| --- | --- | --- | --- |
| P29-C01 | both complete motion/lifecycle/caller graphs identified | input/output/owner/consumer inventory | complete — audit pass |
| P29-C02 | dirty targets and concurrent work preserved | pre/post status, scoped diffs, hashes and protected marker | complete — audit pass |
| P29-C03 | old motion behavior characterized first | common nonzero timing/moving/collision/expiry parity table | complete — audit pass |
| P29-C04 | one private pure typed motion authority exists | source/static ownership and no BattleCore mutation | complete — audit pass |
| P29-C05 | numeric time/position/sweep behavior is exact | zero/sub/exact-ms, interpolation and board/expiry tests | complete — audit pass |
| P29-C06 | basic lifecycle remains distinct and exact | target-only homing, miss/withdraw/death and committed hit tests | complete — audit pass |
| P29-C07 | skill lifecycle remains distinct and exact | guidance/order/pierce/hit/kill/expiry/cast tests | complete — audit pass |
| P29-C08 | Phase 14/28 and Boss seams remain exact | terminal exactly once, one hit/effect handoff and released-work tests | complete — audit pass |
| P29-C09 | public/data/save/server/Unity compatibility is exact | consumer inventory, unchanged shapes/data/canonicals | complete — audit pass |
| P29-C10 | focused/broad debug/release/static validation is green | actual nonzero counts and failure classification | complete — audit pass |
| P29-C11 | downstream boundary and independent audit are current | no Phase 30 prebuild and `Complete / High` review | complete — audit pass |

## Completion Audit Result

- `COMPLETION_AUDIT.md` is a fresh 380-line phase-exit review under the complete authoring/completion guides.
- Primary verdict: `Complete`; long-term fit: `High`; improvement class: `none`.
- Combined gate: `Complete / High`. Phase 29 is complete; Phase 30 may become `ready` only.

## Activation Protocol

Before production inspection:

1. Master records Phase 28 `complete — Complete / High`, Phase 29 as the sole `active` Child and Phases 30~44
   pending.
2. Change all three Child headers from `initialized` to `active` and authorize only Phase 29.
3. Fully reread this PLAN, `EXPERIMENTS.md`, `EXPERIMENT_NOTES.md`, final Phase 28/14/15 audits and the final Boss
   runtime completion audit.
4. Capture HEAD, expanded status count/normalized hash and prospective hashes for likely source/test/data/canonical
   targets using path discovery only, without reading their content.
5. Record that evidence in the Child before examining production details.

No Phase 29 source/test/data/canonical content inspection or mutation is authorized before all five steps.

## Stop Conditions

Stop and do not activate Phase 30 if evidence requires changing motion cadence, rounding/interpolation, collision or
target policy, stable ordering, expiry/terminal delivery, committed hit/effects, released-work semantics, public
Rust symbols/visibility, serialized DTO/V3/RON/wire shape, CRA-POL-003 publication behavior or deployment
assumptions; adding fallback/dual paths/aliases; merging runtime records; absorbing targeting, movement backend,
damage or Phase 30 work; user-owned overlap cannot be separated; required tests select zero; or independent audit is
below `Complete / High`.

사용자와 의논하여 정해야 할 정책이 있을 경우 Goal을 종료한다.
