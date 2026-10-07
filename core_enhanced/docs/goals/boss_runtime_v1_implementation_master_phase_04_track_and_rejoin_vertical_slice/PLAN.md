# Boss Runtime v1 — Phase 04 TrackAndRejoin Vertical Slice Goal

Status: complete
Implementation Complete: true
Goal Type: phase
Execution Mode: implement_and_verify
Production Mutation Authorized: yes — only this child scope
Master Goal: `../boss_runtime_v1_implementation_master/PLAN.md`
Master Phase: P04
Activated: 2026-08-09
Simulator Start Commit: `f318414fb0156a67e04c5479000c2a2683fc0c09`

## Objective

P02의 strict authoring/setup과 P03의 audited Core runtime을 하나의 test-fixture `TrackAndRejoin` encounter로
연결한다. Authored catalog/binding부터 world setup, exact spawn identity, target acquisition, approach, forward
route rejoin, ordinary combat 재개, existing event/checkpoint와 game_server projection까지 실제 end-to-end 경로를
검증한다. Paired no-binding fixture는 같은 ordinary route authority가 변하지 않았음을 증명한다.

## Direct dependency and handoff

- P03: `../boss_runtime_v1_implementation_master_phase_03_runtime_kernel_and_target_movement/COMPLETION_AUDIT.md`
  - Verdict: `Complete / High`
  - Handoff: `ready_for_p04`
- P02 compiled definition/setup and P01 lifecycle/order/atomicity remain authoritative dependencies.
- Refactor sub-track: P04 adds no independent CRA candidate. Post-slice cleanup candidates remain separate and cannot
  become this Phase's completion gate.
- Runtime policy: BP-01~BP-34 portions assigned to P04 in the Master coverage table; P04 does not publish official
  content or open a public Boss lifecycle contract.

The current dirty tree is user-owned. Findings, failing characterization, fixture corrections, implementation changes
and exact validation results are written immediately after each bounded unit rather than reconstructed at Phase end.

## Contract-visible problem

P02 and P03 each pass in isolation, but no one representative path currently proves that strict authored data and PvE
binding reach the runtime through the real world setup transaction and then remain compatible with existing event,
checkpoint and server transport. A fixture-only vertical slice must expose integration defects before P05 publishes
the first live Boss.

## Current runtime and live-data basis

- P03 supplies the optional Core runtime and existing-leaf target/movement/action seam with `Complete / High`.
- Production `game_resources/data/boss_runtime/base.ron` remains valid and empty; no production encounter opts in.
- Current public event/checkpoint/game_server shapes have no Boss graph field. P04 must prove the reference slice can
  remain existing-event-only.
- The external Unity project is a stale consumer baseline. It is read-only and is not current producer authority; P04
  may record relevant source evidence but does not claim or require Unity parity for an unchanged Wire.
- Exact ownership is now fixed: `game/world/tests/combat.rs` owns the real world fixture, the existing Core
  `active_battle_reconnect_projection` owns read/reconnect projection, and game_server's existing message mapper owns
  unchanged transport. Nonzero pre-mutation anchors are recorded in `EXPERIMENTS.md`.

## Source of truth and ownership

- Boss authoring/compiler and typed frames: P02 Boss data modules.
- Encounter opt-in and exact actor mapping: PvE binding plus private combat setup transaction.
- Lifecycle, target lock, task result and movement intent: P03 `BattleCore` Boss runtime.
- Movement, blocking, basic attack, skill, projectile, effect, damage and death: existing Core leaf modules.
- Event/checkpoint producer: current Core/world battle projection; server is a transport mapper, not Boss authority.
- Test fixture definitions/bindings: test-only builders or fixture data selected after current ownership discovery.

## In scope

- One strict test-fixture encounter, actor behavior, Phase and finite `TrackAndRejoin` Pattern using only the P03
  supported selector/query/Task/condition vocabulary
- Exact PvE encounter binding through the atomic live-bundle/world combat setup path without production live opt-in
- A paired no-binding ordinary-route fixture with normalized event/checkpoint/terminal comparison as applicable
- End-to-end success path: acquire one deployed hostile, approach without target churn, rejoin only forward authored
  route cells and resume existing RouteCombat leaf authority
- No-target, invalidated-target, path-unavailable, timeout and actor/encounter lifecycle-interrupt paths
- Reconnect/read projection proof from authoritative current session without reroll, duplicate result or route-cursor
  regression
- Existing event/checkpoint to normal game_server transport equivalence with no public Boss variant or field
- Scoped corrections discovered by this vertical slice when they remain inside P02/P03/P04 approved semantics
- Closest canonical update only if the verified end-to-end current contract adds information not already recorded

## Out of scope

- Official production Boss definitions, encounter binding, identity, balance, assets or production live RON mutation
- New Boss task/selector/condition/mechanic vocabulary or all three future final-boss concepts
- Public Boss graph/lifecycle event, DTO, checkpoint/save field, game_server schema or Unity consumer mutation
- P05 reference identity/publication gates and conditional P06 synchronized public consumer cutover
- Independent or post-slice CRA cleanup candidates, generic fixture frameworks and unrelated refactors
- Persistent Core restore across process/data generation, hot reload, LKG or cross-version migration

## Milestone P04-M1 — current path and fixture boundary

1. Record exact precreation status and postcreation owner hashes without normalizing existing dirty work.
2. Trace one real PvE combat entry from test-authored live bundle through private setup, Core initial execution, world
   battle state, event/checkpoint projection and game_server mapping.
3. Identify the smallest fixture placement that exercises production loaders/builders without modifying production
   live data or adding fixture-only production branches.
4. Fix exact focused and broad runner targets with nonzero baseline counts.

## Milestone P04-M2 — success and paired ordinary slice

1. Author one valid finite `TrackAndRejoin` test fixture using the existing compiled vocabulary and exact actor slot.
2. Drive spawn → activation → target lock → MoveToTarget → RejoinForwardRoute → RouteCombat through ordinary world
   advance, not private runtime helpers alone.
3. Prove target identity remains stable through movement and the route cursor never regresses.
4. Run the paired no-binding encounter through the same setup/projection path and prove ordinary route/combat authority
   and public transport shape remain unchanged.

## Milestone P04-M3 — failure, interruption and reconnect matrix

1. Exercise no eligible target and invalidated target through authored named outcomes without implicit reacquisition.
2. Exercise static path unavailable and exact battle-time timeout with exhaustive single result and no duplicate exit.
3. Exercise actor death/withdraw/despawn or battle terminal interruption at the actual supported lifecycle seam and
   prove stale callbacks cannot reenter.
4. Re-read/reconnect the active battle projection and continue the same authoritative Core session without target
   reroll, activation/result duplication or cursor rollback.

## Milestone P04-M4 — transport and exit audit

1. Compare normal/update/setup or reconnect transport paths relevant to the unchanged public shape and fix any server
   re-enumeration drift inside approved existing fields.
2. Search Core/server/public DTO/save/canonical and stale Unity evidence for accidental new Boss public surface,
   fixture-only production special cases, fallback or duplicate authority.
3. Run focused, full Core, server and workspace gates with exact actual counts.
4. Perform the required completion review and allow P05 readiness only on `Complete / High`.

## Preserved invariants

- `PveEncounter.boss_runtime == None` and `BattleScenario.boss_runtime == None` retain the ordinary enemy path.
- P04 fixture data never becomes production live content or an implicit reference-Boss identity decision.
- Runtime consumes the compiled definition/frame retained by setup and performs no raw catalog reopen.
- Target identity, static path feasibility, leaf action eligibility and lifecycle validity remain distinct outcomes.
- Existing Core event queue, battle clock/Pause, terminal result, world transaction and revision remain authoritative.
- Losing transitions, failed admissions, lifecycle interruption and reconnect reads do not mutate/reroll hidden state.
- Existing leaf event/projectile/damage/death ownership and already released delivery continuation remain unchanged.
- Internal graph/activation/target/task identity remains absent from public Wire, checkpoint/save and stale Unity state.
- No invalid fixture or runtime state becomes fallback, silent recovery, retry, dual schema or compatibility alias.

## DTO, data, save and deployment

- DTO/Wire migration: none expected and prohibited. Existing movement/attack/damage/death events and checkpoint fields
  must be sufficient; contrary evidence activates a stop condition rather than an implicit schema expansion.
- Data/live RON migration: none. The reference data is test-only and production Boss collections stay empty.
- Save/checkpoint migration: none. Reconnect proof reads the existing authoritative in-memory session projection; it
  does not introduce durable graph restoration.
- Deployment: test/verification plus only necessary scoped corrections behind the existing explicit setup opt-in.
  No production encounter behavior changes.
- Producer-consumer rollout: unchanged public producer shape means existing server transport remains one deployment
  unit; stale Unity is not modified. A real public lifecycle requires the separate P06 atomic cutover.

## Focused validation

- Strict fixture compile and typed setup identity through the actual bundle/setup entry
- Authoring-to-world success path and paired no-binding ordinary path
- Stable target, monotonic cursor and RouteCombat resume event sequence
- No-target, invalidation, path-unavailable, timeout and lifecycle-interrupt outcomes
- Exact result/transition cardinality, stale callback and reconnect/non-reroll behavior
- Actual Core projection through the relevant game_server mapping with exact unchanged public shape

The exact existing setup/reconnect/server anchors and counts are fixed in `EXPERIMENTS.md`. New P04 test names are
recorded when added. A zero-match filter is failure. Every actual command and pass/fail/ignored/filtered count is
recorded at execution time.

## Broad validation

After focused closure, run at minimum the current equivalents of:

```text
cargo fmt --all -- --check
cargo test -p game_core --lib
cargo test -p game_server
cargo test --workspace --all-targets
```

The path audit must determine whether an additional named integration target is required for the real server mapping.
It is added rather than substituted. Unity tests are omitted because P04 forbids a public contract change and the
external project is stale; source evidence must still show existing movement/damage presentation sufficiency.

## Completion conditions

1. Strict authored fixture and exact actor binding traverse the real atomic setup path into the P03 runtime.
2. The success slice reaches target acquisition, stable approach, forward-only rejoin and ordinary combat resume.
3. A paired no-binding fixture preserves ordinary route/combat and normalized public projection behavior.
4. No-target, invalidation, path-unavailable and timeout each take one explicit authored exhaustive result path.
5. Lifecycle interruption cancels owned unreleased work, preserves released leaf work and cannot double-complete.
6. Same-time ordering, Pause and checked deadlines remain deterministic in the vertical slice.
7. Reconnect/read projection does not reroll target/selector, regress route cursor or replay a completed lifecycle.
8. Existing event/checkpoint/server transport is sufficient and contains no new Boss public field/variant.
9. Production Boss data remains empty and no official identity/balance/asset decision is encoded.
10. No raw lookup, fallback, compatibility shim, fixture-only production branch or unrelated refactor enters P04.
11. Canonical, focused and broad tests pass with actual counts and dirty user work is preserved.
12. Independent completion audit returns `Complete / High` and handoff `ready_for_p05_decision`.

## Completion audit evidence matrix

| ID | Required evidence | Planned source | Status |
| --- | --- | --- | --- |
| P04-C01 | Strict authoring→typed setup→runtime identity | real bundle/setup fixture and Core state | pass |
| P04-C02 | Success TrackAndRejoin lifecycle | world/Core event and movement trace | pass |
| P04-C03 | Paired no-binding parity | ordinary fixture trace/projection | pass |
| P04-C04 | No-target/invalidation/path/timeout exhaustiveness | outcome matrix | pass |
| P04-C05 | Lifecycle/released-work cardinality | interrupt and stale-event matrix | pass |
| P04-C06 | Pause/same-time/deadline determinism | battle-time fixtures | pass |
| P04-C07 | Reconnect/non-reroll/cursor stability | repeated projection and continuation | pass |
| P04-C08 | Existing public transport sufficiency | Core DTO/server mapping comparison | pass |
| P04-C09 | Empty production data/no official identity | live RON and binding search | pass |
| P04-C10 | Legacy/fallback/scope closure | scoped source/data/DTO search | pass |
| P04-C11 | Canonical/tests/dirty preservation | final commands and hashes | pass |
| P04-C12 | Phase completion verdict | `COMPLETION_AUDIT.md` | pass — Complete/High |

## Stop conditions

- The approved finite fixture cannot express or complete the slice using the P03-supported compiled vocabulary.
- Correct success/failure/reconnect semantics require a new gameplay or presentation policy not fixed by BP-01~BP-34.
- Existing event/checkpoint transport is insufficient and a public Boss lifecycle schema/Unity cutover is required.
- Reconnect correctness requires durable graph serialization rather than the approved current in-memory session model.
- Current user edits overlap a required fixture/runtime/server owner and cannot be reconciled at symbol/hunk scope.
- A required current Core/server runner or source is inaccessible.

No stop condition became active. Completion audit verdict is `Complete / High`; handoff is
`ready_for_p05_decision`, but P05 activation still requires explicit reference identity/content approval.
