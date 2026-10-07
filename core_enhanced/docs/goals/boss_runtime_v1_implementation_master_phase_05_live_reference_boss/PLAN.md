# Boss Runtime v1 Phase 05 — Live Reference Boss Goal

Status: complete
Implementation Complete: true
Goal Type: phase
Execution Mode: implement_and_verify
Production Mutation Authorized: yes — only within this Phase scope
Master Goal: `../boss_runtime_v1_implementation_master/PLAN.md`
Direct Prerequisite: P04 `Complete / High`, `ready_for_p05_decision`
Approved Content Decision: Master `P05_CONTENT_DECISION_PACKET.md`, option 1 approved 2026-08-09
Simulator Start Commit: `f318414fb0156a67e04c5479000c2a2683fc0c09`
Precreation Dirty Baseline: 261 sorted entries, SHA-256 `832ea5972d60e3613d9dd3252dab0299853617c41f88c426f1e9f75d17ea73a8`

## Objective

Publish one replaceable Little Red live reference Boss through the already implemented Boss Runtime v1 path and
prove strict data publication, deterministic execution, ordinary-content parity and existing-event consumer
sufficiency without inventing final character content or a public Boss lifecycle.

## User/Contract-Visible Problem

P04 proves the complete TrackAndRejoin path only with test-authored content. Production Boss RON is empty and no
official live encounter opts into the runtime, so the Master still lacks a real live-data publication and actual
encounter feasibility proof. The user has authorized a disposable reference implementation so this integration can
be validated now and redesigned later.

## Audit Finding ID

없음. This Phase implements Master BRM-C07 and the P05 parts of BP-01~BP-29 rather than correcting a new audit ID.

## Current Runtime And Live Data Evidence

- P04 completion audit is `Complete / High` and proves raw strict authoring through world/Core/server using test-only
  content.
- `game_resources/data/boss_runtime/base.ron` currently contains empty `patterns`, `actor_behaviors` and
  `encounters`; no production `PveEncounter.boss_runtime` binding exists.
- Decision evidence identifies `suppress_little_red` / `f-01-57_little_red` as the approved reference while keeping
  current stats, waves, rewards and `little_red_hunt_the_prey` skill.
- Current external Unity evidence uses the common opponent capsule and existing unit identity/HUD/movement/damage
  projections. This Phase must revalidate that evidence before completion and must not mutate Unity.

## Source Of Truth And Ownership

1. Current runtime code and authoritative world/Core mutation
2. Production live RON and `GameDataBase::load_live_embedded()` strict atomic publication
3. Core DTO/game_server mapping and accessible external Unity consumer
4. Current gameplay/runtime/data/targeting and external transport canonicals
5. Master decision packet and this Phase execution evidence

Boss catalog authoring owns the compiled definition; `PveEncounter.boss_runtime` owns encounter/actor opt-in;
typed setup captures the binding; `BattleCore` owns execution; existing leaf movement/combat owns physical action;
existing Core DTO and game_server own transport. Unity does not infer Boss graph state.

## Invariants

- Encounters without `boss_runtime` remain semantically unchanged.
- Little Red's current stats, wave composition/timing, rewards, automatic skill and ordinary leaf combat remain
  unchanged.
- The reference is one finite one-shot pursuit and then ordinary `RouteCombat`; no implicit retarget or hidden retry.
- Invalid live Boss data rejects the whole candidate before publication; no fallback, alias or partial generation.
- Internal Phase/Pattern/State/Task/target/epoch state remains private.
- Existing event/checkpoint/server/Unity shape remains unchanged unless evidence proves the approved behavior cannot
  be represented; that discovery is a stop condition and P06 decision input.
- User-owned dirty changes and the external Unity dirty tree are preserved.

## Approved Policy And Evidence

The user approved option 1 on 2026-08-09 and authorized agent-authored missing details as replaceable test content.
The bounded package is:

- identity: `suppress_little_red`, abnormality `f-01-57_little_red`;
- exact bound actor: `primary_abnormality`, spawn index 0;
- finite `AcquireTarget → MoveToTarget → RejoinForwardRoute → RouteCombat` behavior using only implemented vocabulary;
- deterministic reachable hostile minimum-path target with `LockUntilInvalid` and exhaustive no-target,
  invalidation, path failure and timeout exits;
- current balance, rewards, wave timing/composition and `little_red_hunt_the_prey` remain unchanged;
- existing generic opponent capsule and existing public transport only;
- no bounty marker/reward, Big Bad Wolf relationship, dedicated art/VFX or final content claim.

## Open Policy Decisions

None inside the approved P05 package. P06 activation versus `not_applicable_to_v1` is an evidence-derived Master
disposition after this Phase, not an invitation to add public fields during P05.

## In Scope

- CRA-P01-C02: validate every manifested live difficulty before publication.
- CRA-P05-C02: validate every resolved live battle setup and Boss binding before publication.
- Author the bounded Boss Pattern, actor behavior and encounter definitions in production Boss RON.
- Opt only `suppress_little_red` into the runtime and add the required authored route to its exact primary wave.
- Add focused strict-invalid, live-loader, actual world lifecycle, ordinary parity and reconnect/termination tests.
- Update the closest gameplay/runtime/data canonical only for behavior actually live after implementation.
- Revalidate game_server and the accessible existing Unity consumer without changing public shape or Unity files.

## Out Of Scope

- Final Little Red content/lore, tuning redesign, dedicated visual/animation/VFX or bounty UI/reward/lifecycle
- New selector, Task, condition, mechanic, public Boss DTO/event/checkpoint field or save schema
- All other Boss identities and the three future final-boss concepts
- Independent refactor catalog items, performance optimization without measured correctness need and Unity mutation
- Hot reload/LKG, dual catalogs, compatibility aliases and in-place conversion of an already active battle

## Implementation Scope

1. Characterize the current live Little Red encounter and strict publication/setup paths from current source.
2. Capture a focused baseline before the first production mutation.
3. Add live Boss RON and exact PvE binding/route as one atomic content migration.
4. Extend strict validation only where the manifested-difficulty or full-setup gates are currently missing.
5. Add actual live-data end-to-end lifecycle and paired ordinary regression tests.
6. Update canonical contracts, revalidate external consumer sufficiency and run focused then broad suites.
7. Perform an independent completion audit under `goal_completion_review_guide.md`.

## DTO, Data And Save Migration

- DTO: none expected; existing events/checkpoint only. Any required new public meaning stops P05.
- Data/live RON: one strict atomic generation changes from empty Boss catalog/no opt-in to the approved reference and
  exact encounter binding. No schema version change is expected unless current source evidence proves otherwise.
- Save/checkpoint: none. Active Boss graph state remains session-internal; existing in-memory reconnect projection is
  verified, not durable cross-process restore.

## Deployment And Compatibility

The embedded bundle and binary are one deployment generation. New battles after publication may use the reference;
already active battles are not converted. There is no dual-read, fallback, alias or compatibility window. The
external Unity consumer remains on the existing shape and is verified read-only.

## Focused Validation

Exact commands will be fixed after current runner discovery and recorded with actual counts. Required surfaces are:

- production embedded bundle inventory/loading and strict invalid Boss/PvE cases;
- every manifested difficulty and every resolved setup/Boss binding feasibility;
- actual `suppress_little_red` start, pursuit, rejoin, ordinary combat, failure/interrupt/Pause/reconnect/terminal flow;
- paired non-opt-in live encounter behavior;
- unchanged Core/server serialized shape and accessible Unity existing-event consumption.

## Broad Validation

- `cargo check -p game_core --lib`
- full `game_core` tests including `ron_loading` and live catalog integration targets
- full `game_server` tests
- `cargo test --workspace --all-targets -- --quiet`
- formatter and `git diff --check`

Commands may be narrowed or split only to recover runner counts/output, never to weaken required coverage.

## Omitted Validation

- Dedicated art/VFX/bounty presentation tests are omitted because those features are explicitly unapproved and
  absent.
- Durable active-battle save migration is omitted because active battle remains in-memory and no save shape changes.
- Other Boss content and independent CRA candidates are outside this Phase.

## Completion Conditions

1. Production Boss RON contains exactly the approved bounded reference and strict compilation succeeds.
2. `suppress_little_red` alone has the exact validated actor binding and required authored route.
3. Every manifested live difficulty and every resolved live setup validates before publication.
4. The actual live reference starts and completes the finite pursuit/rejoin/RouteCombat lifecycle deterministically.
5. No-target, invalidation, path-unavailable, timeout, interruption, Pause, reconnect and terminal paths remain
   exhaustive and nonduplicating for the live reference.
6. Current stats, waves, rewards, skill and ordinary leaf combat are preserved.
7. Non-opt-in live encounters remain unchanged.
8. No new public Boss field/variant/save shape, compatibility/fallback or Unity-side rule inference is added.
9. Existing Core/server/Unity event/checkpoint path is proven sufficient or P05 stops with concrete P06 evidence.
10. Gameplay/runtime/data canonicals describe exactly the implemented reference boundary and disposable-content
    status without claiming final Little Red design.
11. Focused and broad suites pass with exact counts and baseline failures distinguished.
12. Scoped diff and dirty-user/external-Unity preservation pass.
13. Independent completion audit returns `Complete / High` and explicitly decides the P06 handoff.

## Completion Audit Evidence Matrix

| ID | Requirement | Evidence required | Status |
| --- | --- | --- | --- |
| P05-C01 | Strict production Boss publication | live RON + compiler/loader tests | pass |
| P05-C02 | Exact Little Red opt-in and route | PvE RON + setup feasibility tests | pass |
| P05-C03 | Manifested-difficulty validation | validator source + exhaustive live test | pass |
| P05-C04 | Full resolved-setup feasibility | actual setup generation matrix | pass |
| P05-C05 | Live pursuit/rejoin lifecycle | world/Core end-to-end tests | pass |
| P05-C06 | Exhaustive failure/Pause/reconnect/terminal semantics | focused live reference matrix | pass |
| P05-C07 | Existing content preservation | before/after assertions and data audit | pass |
| P05-C08 | Ordinary non-opt-in parity | paired live tests | pass |
| P05-C09 | DTO/server/Unity sufficiency | shape/mapping tests + external source evidence | pass |
| P05-C10 | No save/compatibility/placeholder expansion | scoped source/schema audit | pass |
| P05-C11 | Canonical and broad validation alignment | current docs + exact test results | pass |
| P05-C12 | Dirty preservation | exact status/owner baselines | pass |
| P05-C13 | Independent Phase verdict and P06 disposition | completion audit | pass |

## Stop Conditions

- The approved reference requires final-content meaning, balance changes, dedicated assets/VFX/bounty semantics,
  new runtime vocabulary or a new public lifecycle to function.
- Current source proves an active-battle in-place conversion or compatibility window is required.
- Existing user changes overlap a required owner and cannot be merged without overwriting intent.
- Required Core/server/external consumer or validation environment is inaccessible.
- Strict validation cannot close every manifested difficulty and actual setup without a separate policy decision.
- Any request would expand P05 beyond the approved disposable reference package.

If a stop condition is reached, affected production work stops and the evidence is recorded immediately; this Goal
is not declared complete.
