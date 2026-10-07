# P02 Wave Cutover Execution Plan

- Goal ID: `GOAL-CWBS-P02-WAVE-CUTOVER`
- Phase ID: `PHASE-CWBS-02-WAVE-CUTOVER`
- Final Execution Nodes: `AG-CWBS-02-WAVE-CUTOVER` (`WP-CWBS-04 + WP-CWBS-05`) and `WP-CWBS-03`
- Accepted Handoff / Goal Set: `HA-CWBS-P2G60-002 / GS-CWBS-P2G50-002`
- Run Mode / Root Target: `goal-run / GOAL-CWBS-MASTER`
- Authorization Ref: `GEXAUTH-CWBS-001`
- Current Pass: `GEX-20 complete / Audit Pass 1`
- Direct Dependency: `GOAL-CWBS-P01-RUNTIME-FOUNDATION / Complete / High / Audit Pass 3`
- Execution Directory:
  `core_enhanced/docs/goals/conditional_wave_and_boss_summon_policy/execution/p02_wave_cutover`
- Document Set: `PLAN.md`, `EXPERIMENTS.md`, `EXPERIMENT_NOTES.md`; fresh GEX-20 creates `COMPLETION_AUDIT.md`

## Objective and Scope

Cut over live Wave authoring and runtime to a strict typed trigger/dependency/deadline/actor-slot contract, one-shot
binding-aware evaluator and the P01 Spawn Kernel. Extract an encounter-independent Boss actor behavior adapter for P03.

Covered revisions: `PER-CWBS-WV-01-CODE-003` through `PER-CWBS-WV-07-CODE-003`.

In scope: PVE schema/compiler/publication, all live PVE rows, scenario/runtime trigger registry, binding epoch, HP/point first
evaluation, prospective death/despawn observation, group/deadline/cancel/gate state, Kernel consumer and authority-free actor
adapter.

Out of scope: summon domain/BossTask Summon, named terminal outcome cutover, Unity, content promotion and deployment.

## Current Boundary and Audit Target Plan

- Current Wave schema has implicit/default fields and scenario runtime supports only battle-start/time events.
- Current group insertion is the old sequential consumer; P02 must consume the P01 Kernel without retaining a legacy spawn path.
- Current Boss defeated condition uses bound actor presence/liveness as a level fact; P02 requires prospective post-bind death
  observation and restore-safe latches.
- Live PVE rows must migrate atomically with the strict reader; omitted defaults and fallback activation are forbidden.
- P01 is current and reusable; P02 invalidates only Wave schema/RON/compiler/runtime, binding observer, ordering and adapter seams.

Current pre-mutation identities and the frozen GEX-10 target record are recorded in `EXPERIMENTS.md`.

## GEX-10 Target Record

- Strict PVE schema/compiler/live RON cutover: implemented
- Wave post-bucket runtime/Kernel consumer: implemented
- prospective actor binding/death and authority-free Boss adapter: implemented
- deterministic priority/deadline/cancellation/required gate: implemented
- focused current validation: passed; exact commands and counts are in `EXPERIMENTS.md`
- Full baseline: not repeated by contract; reserved for P06
- Implementation complete: `true`
- Completion audit: `Complete / High / Audit Pass 1`

## Validation and Completion

- malformed/default/cycle/source/deadline and live-load fixtures
- HP/point already-true at binding; pre-bind death/despawn and restore no-refire fixtures
- time/group/deadline/priority and required gate/cancel tests
- P01 Kernel consumer atomicity and existing Little Red/Boss adapter regression
- no repeated full baseline in P02; run focused invalidated seams and reserve integrated full Core/server suites for P06

Completion requires strict live cutover with no legacy/default path, prospective binding semantics, deterministic full-bucket
Wave ordering/gating and an independently valid adapter intermediate state.

## Stop Conditions

- the closed trigger/dependency graph cannot represent the approved policy without arbitrary expressions/fallback
- prospective death cannot be attached to an authoritative post-bind event
- live schema migration needs a policy/scope change or production promotion
- P01 audit becomes stale or scoped dirty changes cannot be preserved safely
