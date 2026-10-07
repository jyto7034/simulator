# Boss Runtime Implementation Readiness Report

Goal: `boss_runtime_implementation_readiness_audit`  
Audit date: 2026-08-08  
Primary verdict: `conditionally_ready`

## Verdict

Boss Runtime은 **선행 correctness를 먼저 닫는 조건으로 후속 구현 campaign을 계획할 준비가 됐다.** 현재
working tree에 곧바로 live Boss opt-in을 추가하거나 public Boss presentation을 배포할 준비가 됐다는 뜻은
아니다.

판정을 세 실행 경계로 나누면 다음과 같다.

| Boundary | Verdict | Meaning |
| --- | --- | --- |
| Correctness/refactor implementation | `ready_to_authorize_separately` | CRA-P06-C01/C02/C03의 범위와 acceptance가 source evidence로 확정됐다. |
| Isolated Boss foundation + test-fixture tracking slice | `ready_after_prerequisites` | 승인된 BP-01~BP-34와 owner/seam/test가 닫혔다. P06 correctness 뒤 별도 구현 Goal로 착수할 수 있다. |
| Official live Boss opt-in / public Boss mechanic wire | `not_yet_ready_to_publish` | Live opt-in은 P01-C02/P05-C02 publication gates, public Wire는 Core/server/stale Unity 동시 cutover가 먼저 필요하다. |

추가 사용자 정책 결정은 현재 없다. `RepathToProtectedObjective`, dynamic hot reload/LKG, fog/LOS/dynamic nav,
public telegraph lifecycle과 최종보스 고유 mechanic은 승인된 v1 범위가 아니며, 사용되면 strict validation
error이거나 별도 future owner로 남는다.

## Why the verdict is evidence-backed

- `BOUNDARY_AUDIT.md`가 live publication→preview/setup→scenario→BattleCore leaf execution→event/checkpoint→
  game_server→stale Unity의 실제 producer/consumer path를 고정했다.
- `POLICY_COVERAGE_MATRIX.md`가 `BP-01`~`BP-34`를 정확히 34개 행으로 current source/canonical, target authority,
  disposition, owner와 test-first acceptance에 연결했다.
- Current Core에는 Boss catalog/runtime가 없지만 existing movement, block, attack, skill, effect, projectile,
  damage/death, battle-time queue, append-only event log와 checkpoint projection은 leaf/foundation primitive로
  재사용 가능하다.
- Material conflict는 Boss 정책이 아니라 기존 correctness에 있다: non-total same-time order, world advance의
  partial mutation, advance API별 terminal/max-time divergence다.
- Current event만 사용하는 internal slice는 server/Unity Wire 변경 없이 표현 가능하다. Public duration
  presentation은 아직 consumer-complete하지 않으므로 별도 cutover다.

## Required dependency order

```text
G0: existing correctness prerequisites
    CRA-P06-C01 terminal/max-time parity
    CRA-P06-C02 BattleEvent total order
    CRA-P06-C03 advance failure no-mutation
                │
                ▼
G1: Boss definition/runtime foundation (no encounter opt-in)
    strict catalog + compiled definitions + activation/task/query/movement seams
    P01-C01 source inventory and P05-C04 typed binding coimplemented
                │
                ▼
G2: test-fixture minimal tracking vertical slice (existing public events only)
                │
        ┌───────┴────────┐
        ▼                ▼
G3a: official live opt-in    G3b: first public mechanic lifecycle
P01-C02 + P05-C02 gates      Core semantic DTO + P10-C03/server parity
approved content RON          + stale Unity synchronized consumer
```

This is a dependency graph, not authorization to create or execute these Goals. The full 51-candidate refactor catalog
is not a prerequisite.

### G0 — correctness prerequisites before runtime integration

| Gate | Minimum correction boundary | Acceptance before Boss scheduler integration |
| --- | --- | --- |
| CRA-P06-C01 | All Core/world advance entry points call one terminal/max-time finalization authority. | Direct step, until-time, next-event, server tick and exact-boundary cases produce the same status/result/event semantics. |
| CRA-P06-C02 | `BattleEvent::Ord` is a total deterministic order for every variant and stable payload key; unequal events never compare `Equal`. | Variant/payload insertion permutations drain identically; existing attack-before-end regression remains green. |
| CRA-P06-C03 | Validate cursor/request and stage fallible world follow-up before authoritative publication, or use an equivalent rollback-safe transaction. | Injected VN/reconcile/projection/revision/cursor failures leave battle time, Core state, resources, event log, revision and cursor unchanged. |

The Boss microstep budget belongs to G1, but it must build on G0's total order and atomic boundary. A large shared
transaction spanning player commands and autonomous Boss ticks is not required; reuse the primitive while keeping
their lifetimes separate.

## G1 — minimum Boss foundation boundary

### Authoring and publication

- Add a separate logical `boss_runtime` catalog with distinct Pattern, actor behavior and encounter definition
  collections. Use stable IDs, strict unknown-field/duplicate/ref checking and no deep overrides.
- Add the physical source to the single `GameplayRonSourceSet`/`ParsedGameplayRon`/`GameDataBaseParts` candidate path
  and update the exact source inventory once (CRA-P01-C01). Do not publish a partial catalog generation.
- Compile references, outcome/result exhaustiveness, transition uniqueness, loop boundedness, capabilities, parameter
  bindings and supported enum variants into immutable resolved definitions before `Arc<GameDataBase>` publication.
- The startup-only loader needs fail-closed candidate construction. Do not invent dynamic reload merely to implement
  BP-28; if reload is added later, its publisher owns generation pinning and last-known-good retention.
- `PveEncounter` owns optional `boss_runtime_id` plus typed actor bindings only. Abnormality/enemy records continue to
  own identity/stats/attack/skill metadata; the Boss graph is not embedded there.

### Setup and runtime ownership

- Resolve encounter/actor bindings during preview/setup into one immutable typed scenario plan (CRA-P05-C04). The
  battle must not repeat raw catalog lookup or argument binding after start.
- Create one Core-owned `BossEncounterRuntime` and one `BossActorBehaviorRuntime` for each bound actor. Absence of a
  binding must bypass this construction and preserve the current route-enemy path exactly.
- Runtime identity must include Phase, Pattern, State and Task activation epochs; a parent owns at most one active child
  at each single-owner scope. State-owned task admissions carry owner leases into existing leaf systems.
- Core active battle state owns target lock, route cursor, selector cooldown, condition event cursor/latch, deadlines,
  resolved parameter frames and RNG decisions. Unity receives none of these internal graph details.
- Current active battle is session-memory authoritative, not a durable SavePoint payload. Reconnect retains this Core
  runtime because the server retains the active battle. If durable Core restoration is later added, the same internal
  activation state must be captured in a Core-only checkpoint rather than inferred from the public Unity checkpoint.

### Scheduler, Tasks and leaf reuse

- Evaluate Boss conditions at the existing timestamp/event boundary. Collect all candidates from one snapshot, apply
  BP-16 scope order and BP-17/BP-32 node priority, stage the winner, then commit once.
- Add an explicit per-timestamp microstep budget. Static validation rejects zero-time/unbounded graph cycles; runtime
  overflow is an invariant failure with no partial current-transition mutation.
- Start only the closed Task/selector/condition variants required by the reference slice. Known future variants may
  exist in design vocabulary but must reject publication until implemented; there is no fallback/no-op registry.
- State task capability claims are intrinsic to Core Task types. All claims are preflighted together; the author cannot
  weaken claims or use numeric Task priority to choose a compatible subset.
- Existing movement/collision/blocking/attack/skill/effect/projectile/damage/death modules remain executors. Boss code
  submits typed intents/admissions and consumes typed outcomes; it does not duplicate those mechanics.

### Target, movement and route seam

- Store the activation target separately from `RuntimeUnit.current_target`. Acquire from one immutable snapshot using
  typed filters and lexicographic keys, with battle spawn sequence/runtime ID as the canonical Unit tie-break.
- Separate identity validity, movement feasibility and action eligibility. A live actor moving tile remains the same
  locked target; death/despawn/hostile-lifecycle exit is immediate `TargetInvalidated`; path/action failure does not
  silently retarget.
- Insert one explicit Boss movement-intent branch before the current planner's authored-route fallback. With no Boss
  intent/binding, execute the exact old branch.
- The first navigation surface is deterministic static cardinal-grid reachability/path cost over current battlefield
  cell/void/static-block constraints. Dynamic obstacles, LOS, roaming targets and generalized navigation are excluded.
- Maintain an actor-owned monotonic authored-route cursor. `RejoinForwardRoute` selects a reachable point at or after
  that cursor and never projects backward. Current continuous movement/collision performs the path segments and
  reports one closed result to the owning Task.

## G2 — minimum tracking vertical slice

The first vertical slice is a test-fixture reference behavior, not official final-boss content. It proves the complete
authoring→setup→runtime→leaf→event/checkpoint path while keeping the public Wire unchanged.

### Reference behavior

| Element | Minimum supported shape |
| --- | --- |
| Opt-in | One fixture encounter references one actor behavior; a paired encounter with no reference proves exact ordinary-route parity. |
| Encounter/actor | One encounter runtime, one bound actor runtime, one Phase. Multi-actor ownership is covered by foundation tests but not content complexity. |
| Phase baseline | Explicit `RouteCombat`-equivalent State delegates current authored route, block and basic-combat leaf behavior. |
| Selector | Only the closed selector strategy actually used by the fixture, preferably `FixedSequence`; other declared future strategies reject until implemented. |
| Pattern | Finite `TrackAndRejoin` named State graph: acquire one deployed hostile Unit, approach it, then rejoin the forward authored route. |
| Target query | Closed Unit filter plus meaningful lexicographic order and canonical stable-ID tie. Default `LockUntilInvalid`; no per-tick best-target churn or grace. |
| Task/results | Typed target acquisition/movement/rejoin Tasks with exhaustive reachable results such as `Reached`, `NoEligibleTarget`, `TargetInvalidated`, `PathUnavailable`, and `TimedOut`. Lifecycle exit cancellation is not forged as a Task result. |
| Recovery | Every result arm explicitly returns a named Pattern outcome or moves to a declared recovery/route State; no engine retry, fallback target, or hidden cooldown. |
| Presentation | Reuse current movement/attack/damage/death events and current checkpoint fields only. Do not expose Phase/Pattern/State/target/epoch. |

The exact content trigger and tuning values belong to the later approved reference-content Goal. The fixture can enter
the Pattern deterministically at battle start or an authored test event; this audit does not invent production balance.

### End-to-end slice trace

```text
strict Boss fixture RON
→ atomic candidate DB and resolved typed setup binding
→ BattleScenario spawn with actor behavior definition ID/frame
→ BossEncounterRuntime + BossActorBehaviorRuntime activation
→ fixed PatternCall / AcquireTarget from one Core snapshot
→ Boss movement intent before ordinary planner fallback
→ current movement/collision/blocking leaf and existing BattleEvents
→ typed Task result / deterministic graph transition / forward route rejoin
→ existing LiveBattleUpdate event delta + checkpoint
→ unchanged game_server envelope and stale Unity's already-supported movement/damage path
```

### Explicit slice exclusions

- No official Elite/NormalBoss/FinalBoss live encounter is opted in.
- No public Boss phase, objective, telegraph, skill-area, projectile or duration presentation event is added.
- No Unity gameplay inference, battle-event→Spine/VFX adapter, facing migration or same-UUID consumer repair is
  smuggled in. The concurrent caller-driven `ArkClient.SpinePresentation` primitive may be reused later but is not
  currently referenced by battlefield/runtime code.
- No weighted selector unless the approved reference actually needs it; no universal expression/script VM or generic
  plugin Task registry.
- No dynamic navigation, `RepathToProtectedObjective`, stealth/LOS/grace, summon/resurrection/reset, fog, deployable,
  camera/minimap or final-boss-specific enum.

## Test-first safety net

### Before changing production behavior

| Test family | First failing/characterization assertion | Why it gates the change |
| --- | --- | --- |
| Terminal parity | Same input through every advance API yields identical Running/Finished/Draw and final event boundary. | Prevents Boss terminal/phase arbitration from inheriting contradictory end semantics. |
| Total event order | Every unequal same-time variant/payload compares non-equal and permutation drains match. | Makes BP-16/BP-22/BP-34 deterministic independent of container insertion. |
| Advance atomic failure | Inject each post-step failure, including future `after_seq`, and assert a full pre/post state equality. | Establishes BP-31's no-partial-mutation substrate. |
| Ordinary route characterization | Existing preview→scenario route cells, block/basic attack and movement trace for non-opt-in fixtures. | Proves BP-01/BRA-I03 preservation before adding the intent seam. |
| Hard CC/released delivery | Retain unreleased cancel, stale cast/attack epoch no-op, and released projectile/delivery completion tests. | Locks the leaf lifecycle that State ownership will reuse. |

### Foundation and fixture acceptance

| Area | Required assertions |
| --- | --- |
| Strict data | Exact source inventory; unknown fields/IDs/enum variants, duplicate IDs, dangling refs, invalid cycles, missing outcomes/results, conflicting capabilities and bad parameter bindings reject the whole candidate. |
| Publication/setup | Candidate failure leaves published DB unchanged; all resolved IDs/frames are in the typed setup; no runtime raw lookup; no-binding setup is unchanged. Live opt-in later adds all manifested difficulty/setup feasibility coverage. |
| Activation lifecycle | Single parent ownership, non-reentry, new epoch on reentry, stale event no-op, exit exactly once, unreleased cancel and detached delivery continuation. |
| Arbitration | Cross-scope and same-node candidate permutations choose one winner; losing candidates do not mutate, emit, consume Pulse or start cooldown. Microstep limit is deterministic and rollback-safe. |
| Targeting | Same snapshot/seed yields same lexicographic target; equal metrics end at canonical ID; lock survives tactical relocation/reconnect; death invalidates immediately; no hidden retarget/grace. |
| Movement | Boss intent precedes route fallback only for opt-in actor; forward cursor never regresses; stable equal-cost path; reached/unreachable/timeout outcomes exactly once; ordinary route trace unchanged. |
| Time/RNG | Pause freezes Task deadlines and selector cooldowns; completion/domain result beats same-time timeout; selection RNG is committed once and reconnect does not reroll. |
| Atomicity | Capability, Task, Pattern and transition admission failures leave graph state, intents, leaf state, resources, cooldowns, RNG, events and revision unchanged. |
| Replay | Internal slice produces no new public variant/top-level field; existing seq/time/checkpoint and server envelope remain normalized-equivalent. |

### Interrupt and reconnect cases

- State/Pattern/Phase exit while target movement is unreleased: remove the intent/timeout and make queued epoch work
  no-op; do not execute the Task result mapping.
- Hard CC: existing leaf cancellation semantics apply, while battle-time deadline continues unless battle is paused.
- Target death/despawn at the same timestamp as reach/timeout: fixed event and scope order produces one documented
  result/transition with no duplicate completion.
- Same active battle reconnect: Core target/activation/deadline remains authoritative in server memory; Unity rebuilds
  only supported public movement/checkpoint state and never reacquires a gameplay target.
- Actual active-battle loss/restart: a new battle/setup creates new runtimes. The current system has no durable active
  Boss save restoration, so the slice must not claim otherwise.

## Existing test disposition

| Existing family | Disposition |
| --- | --- |
| Strict RON and PVE cross-reference tests | `retain` and `strengthen` with Boss inventory/reference/exhaustiveness failures. |
| Preview→scenario authored-route, block and movement tests | `retain` and `strengthen`; these are the non-opt-in parity oracle. |
| Hard CC, stale cast/attack epoch, released projectile/delivery tests | `retain` and `strengthen` with State owner/activation epoch. |
| Same-timestamp attack-before-end test | `retain` and broaden to complete total-order/permutation/microstep coverage. |
| World setup-loss/pause/revision overflow/tactical atomic tests | `retain` and broaden to whole advance state/event/revision/cursor equality. |
| Server setup/reconnect/resync tests | `retain`; strengthen only when public Boss fields/events appear. |
| Stale Unity revision/seq/checkpoint/movement/damage tests | `unverified` for current producer parity; retain as stale baseline, then update in a synchronized Wire Goal. |
| P07 movement cleanup/parity tests | `merge` only after the new intent/result seam has its invariant tests. |
| Private/ghost shape assertions | `remove` only after every protected invariant is transferred; no immediate deletion. |

No existing relevant test is deleted as part of a prerequisite or first slice merely because it is inconvenient.

## G3a — official live opt-in gate

Before an official encounter receives `boss_runtime_id` or actor binding:

1. CRA-P01-C02 validates every manifested difficulty/setup source, rather than the current floor-0/five-seed baseline
   only.
2. CRA-P05-C02 validates resolved spawn, battlefield/route, field/tactical and Boss encounter/actor feasibility before
   candidate publication.
3. The specific reference Boss content, selector variants, Tasks, target queries, outcomes and assets are separately
   approved; unsupported vocabulary fails publication.
4. Ordinary non-opt-in live encounter generation and transport snapshots remain equivalent.
5. Canonical gameplay/runtime/data documents are updated in the implementation Goal; this audit document does not
   become the canonical contract.

## G3b — first public Boss presentation cutover

The internal slice deliberately avoids this gate. When a later mechanic needs public duration state:

- Core defines semantic start/update/end/cancel/interrupt events with stable `presentation_instance_id`, cause,
  authoritative time/seq and the minimum reconnect-active checkpoint state. Add facing only with an explicit producer
  contract; never expose internal graph or target-selection state.
- Normal `BattleUpdate`, `BattleSetupSnapshot` and `BattleResync` must prove exact normalized JSON parity for the same
  Core generation. This is the point to execute CRA-P10-C03; manual server field enumeration cannot silently omit a
  new top-level producer field.
- Stale Unity must implement same-UUID setup-loss reset, lifecycle instance registry/readiness, start/end/cancel
  idempotence, checkpoint rebuild without replaying completed cues, death completion ownership, and actual asset/Spine
  mapping for the new semantic event. The current wire-independent `ArkClient.SpinePresentation` action/lifecycle
  abstraction is a possible leaf adapter, not evidence that this mapping or seq/checkpoint ownership already exists.
- Core, game_server and Unity consumer changes deploy as one strict producer-consumer cutover unless that future Goal
  explicitly approves/version-tests a compatibility window. No implicit dual schema or unknown-event success is
  assumed.

## Final-boss stress result and long-term fit

The three future concepts do not require widening v1 into a universal engine.

| Concept | Foundation extension points that fit | Separate future systems |
| --- | --- | --- |
| WhiteNight | Encounter phases/objectives, multi-actor binding, threshold typed events, mechanic adapter. | Third-side apostles, terminal VN question, resurrection/partial reset and run-local confession history. |
| Silent Orchestra | Battle-time phase deadlines, immunity/target predicates, wave/global outcome and encounter resource hooks. | Exact content/balance/reward and public presentation assets/lifecycles. |
| Apocalypse Bird | Persistent objectives, irreversible mechanic disable and activation-epoch hazard cancellation. | Fog/visibility, large-map navigation, tactical deployables, camera/minimap and exact egg/power content. |

Keep `BossEncounterRuntime` + per-actor runtime + finite typed graph + explicit mechanic adapters. Do not add
boss-specific node enums or placeholders for unapproved mechanics.

## Risks that remain visible rather than hidden

| Risk | Containment |
| --- | --- |
| Correctness work changes event/finalization behavior before Boss code begins. | Separate G0 implementation/review and preserve current characterization tests. |
| A new intent seam accidentally changes ordinary enemies. | Optional binding plus no-intent exact fallback and paired trace tests. |
| Pathfinding expands into a general navigation rewrite. | Limit G1/G2 to deterministic static-grid query required by tracking/rejoin; future nav is a separate owner. |
| Foundation becomes a generic scripting/task platform. | Closed variants, reference-driven implementation and strict rejection of unsupported vocabulary. |
| Internal state leaks into Wire to ease Unity debugging. | Public semantic lifecycle only; internal graph/target/epoch stay Core-owned. |
| Stale Unity appears compatible because unknown events do not crash. | Unknown-event parse tolerance is not presentation readiness; gate public events on synchronized lifecycle tests. |
| Existing refactor catalog becomes an open-ended prerequisite. | Only P06-C01/C02/C03 precede runtime integration; P01/P05 are scoped to source/setup/live publication; other named candidates keep their recorded independent/deferred dispositions. |

## Handoff boundary

A later implementation owner can begin by proposing a separate G0 correctness Goal with the three exact candidates and
acceptance above. Once G0 is complete, a Boss foundation Goal can implement G1 plus the G2 fixture slice without live
Boss content or public Wire changes. G3a, G3b and each final-boss mechanic remain independently authorized follow-ups.

This audit itself authorizes no production code, live RON, test, canonical, game_server or Unity mutation.
