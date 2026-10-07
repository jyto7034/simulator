# P03 Summon Cutover Execution Plan

- Goal ID: `GOAL-CWBS-P03-SUMMON-CUTOVER`
- Phase ID: `PHASE-CWBS-03-SUMMON-CUTOVER`
- Final Execution Nodes: `AG-CWBS-03-SUMMON-CUTOVER` (`WP-CWBS-06 + WP-CWBS-07`)
- Accepted Handoff / Goal Set: `HA-CWBS-P2G60-002 / GS-CWBS-P2G50-002`
- Run Mode / Root Target: `goal-run / GOAL-CWBS-MASTER`
- Authorization Ref: `GEXAUTH-CWBS-001`
- Current Pass: `GEX-20 complete / Audit Pass 1 / Complete / High`
- Direct Dependency: `GOAL-CWBS-P02-WAVE-CUTOVER / Complete / High / Audit Pass 1`
- Execution Directory:
  `core_enhanced/docs/goals/conditional_wave_and_boss_summon_policy/execution/p03_summon_cutover`
- Document Set: `PLAN.md`, `EXPERIMENTS.md`, `EXPERIMENT_NOTES.md`; fresh GEX-20 creates `COMPLETION_AUDIT.md`

## Objective and Scope

Complete the independent summoned-unit definition/provenance/counter/batch domain and connect it to a strict
`BossTask::Summon` producer through the P01 Spawn Kernel and P02 authority-free actor behavior seam.

Covered revisions: `PER-CWBS-SM-01-CODE-003` through `PER-CWBS-SM-04-CODE-006` and
`PER-CWBS-BR-01-CODE-003` through `PER-CWBS-BR-05-CODE-003`.

In scope: summon definitions and scaling, per-summoner/definition battle-lifetime counters, provenance, atomic batches,
pattern-scoped handles, exact batch-clear signals, Around/SpawnZone placement, closed value-bearing occupancy,
Fail/bounded-Wait insufficient-space handling, removal-first Persist/Despawn/Kill semantics and authority-free behavior
execution.

Out of scope: recursive summon, `SkillEffect::Summon`, summon victory gating, independent reward/compendium/fragment
sources, Boss encounter/phase/Wave/terminal authority, reachability inference or fallback, Unity, production content opt-in,
promotion and deployment.

## Current Boundary and Target Plan

- The current catalog/compiler/task result set has no summon vocabulary; the change must be a strict reader/writer cutover
  for new opt-in data without aliases or implicit defaults.
- P01 already owns registration, exact-once request settlement, atomic distinct-cell placement input and cancellation;
  P03 extends only the summon-specific occupancy/result/lifetime consumer.
- P02 exposes Boss actor binding without phase/task authority. Summoned actors reuse the existing behavior/task/query/
  movement machinery with explicit route context and remain outside encounter actor slots and phase transitions.
- A request may commit only after summoner/state validity, both limits, all-occupant permission and all distinct placements
  are simultaneously valid. Failure and cancellation create no batch UUID and consume no total count.
- Removal settlement precedes waiter allocation at the same simulation timestamp. Waiters use wait-start then request-ID
  order; available space wins an equal timeout boundary, followed by cancellation/limit recheck.
- The live Boss Runtime catalog remains non-opted-in unless an existing fixture is explicitly migrated; representative
  summon definitions and graphs may remain focused test fixtures until P06 placeholder integration.

Current pre-mutation identities and dirty attribution are recorded in `EXPERIMENTS.md`.

## Validation and Completion

- catalog/schema/compiler rejection matrix: missing/unknown definitions and behaviors, non-positive limits/count/wait,
  invalid occupancy values/combinations, recursive summon graphs and invalid signal/handle references
- definition isolation/scaling and source/provenance/attribution fixtures
- success-only per-summoner limits, atomic multi-member batches and all four task results
- deterministic Around/SpawnZone candidate ordering, every-occupant validation, distinct cells, no partial/eviction/fallback
- bounded multi-waiter arbitration including same-bucket removal/space/timeout and cancellation/limit recheck
- death/despawn/withdrawal/state-exit/lifetime/signal removal, batch-clear exact-once and handle scope fixtures
- authority-free summon behavior and existing Little Red/Wave/Kernel focused regressions
- no repeated full baseline in P03; the inherited 1,152-pass baseline is reserved for P06

Completion requires the opt-in summon path to be internally complete with no legacy/fallback path, while the old terminal
contract remains valid and summon state does not become durable SavePoint state.

## GEX-10 Implemented Result

- Added a strict independent summoned-unit RON catalog with definition-owned combat stats, skill/basic attack/display,
  positive per-summoner limits, removal policy and actor behavior reference.
- Added `BossTask::Summon`, its four exhaustive results, placement/occupancy/wait/lifetime/removal-signal schema and
  compiler publication gates, including recursive graph and encounter-bound signal rejection.
- Added ActiveBattle-only request, counter, batch, handle, signal epoch and deterministic UUID continuation state.
- Added atomic Kernel-backed multi-member commit, deterministic Around/SpawnZone placement, FIFO waiter settlement and
  removal-before-allocation ordering.
- Added independent summoned-unit provenance and encounter scaling exactly once; reward/compendium/Wave/phase/terminal
  authority remains excluded.
- Added authority-free per-unit Boss actor behavior reuse with explicit route context and no encounter actor slot.
- Kept the live catalog empty and non-opted-in; P03 does not promote production content or alter SavePoint payloads.
- Corrected two P02-era survival fixture assumptions exposed by broader focused validation. The test-only waves now use
  `required_for_victory = false`, which is the current survival Wave contract; production behavior was not changed.

The exact focused commands, counts, initial failures and rebaseline results are recorded in `EXPERIMENTS.md`. Current
target hashes are also recorded there and are the input to fresh GEX-20.

## Stop Conditions

- atomic batch/counter/handle state cannot share one authoritative settlement boundary with the P01 Kernel
- every-occupant placement or removal-first waiter arbitration cannot be made deterministic in simulation time
- authority-free behavior reuse requires granting encounter phase, Wave or terminal authority
- implementation requires a policy/scope change, production opt-in, migration, Unity or external deployment
- P02 dependency becomes stale or overlapping dirty work cannot be preserved safely
