# Aggregate And State Map

- Goal: `current_runtime_architecture_deep_review`
- Status: `complete`
- Baseline: `BASELINE-20260822-064427-KST`

## Authority Layers

| Layer | Authority/type | Owned state | Mutation boundary | Notes |
| --- | --- | --- | --- | --- |
| Application root | `GameCoreState` | phase, current sessions, run, profile, roster, inventory, fragments, battle, checkpoint, UUID, currency | private `GameCore` handlers | external mutation blocked; internal world modules coordinate roots |
| Run aggregate | `RunState` | map/progression, facility/encounter progression, event/shop sessions, previews, attempts, recruitment, record progress | world use cases | run state also `Deref`s to `EncounterProgressionState`, obscuring explicit owner access |
| Map progression | `RunMap` + `MapProgression` + `FacilityTemplate` | authored topology, node state/visibility, current/available/completed identities | clone/plan then commit | selectability and transit are derived from graph and typed direction |
| Interactive node session | `node_session`, `active_node_content`, run-local event/shop session maps | in-progress content identity and domain-specific session | node handlers + root transaction | S08 confirms parallel representations can encode inconsistent combinations |
| Encounter progression | `EncounterProgressionState` | reward-source completion, Elite/Boss cycles, Monolith and BossOmen | run/application handlers | cohesive subaggregate, but `RunState::Deref` hides when it is being accessed |
| Growth/loadout | inventory, equipment, fragments; future Relic absent | run-owned copies/progress/selection compiled into battle profiles | maintenance/economy handlers and setup compilers | fragment compiler is a strong seam; Relic should be a separate aggregate/compiler |
| Battle session | `ActiveBattleSession` | battle core, deployment/tactical/projection/session metadata | battle-specific handlers | explicitly excluded from non-battle whole-state transaction clone |
| Permanent profile | `PlayerProfileState` | monotonic cross-run knowledge/progression | separate merge rules | intentionally preserved/merged across run-local rollback |
| Checkpoint | `RunCheckpointPayload` | selected RunState fields plus economy/roster/fragments/UUID | capture/validate/restore | schema generated from same macro field list as RunState subset |

## State Machine

`GameState` is a typed enum covering pre-run, starter selection, map, confirm, node, shop, reward, battle/result,
mastered-elite result, recoverable run-failure decision and terminal states. Phase identity is explicit, but several parallel
`Option` roots (`node_session`, `active_node_content`, `active_battle`, `pending_mastered_elite_sweep`, `run`) must agree with the
enum. Validation/transition coverage is assessed in later scenarios.

## Transaction And Recovery Mechanisms

| Mechanism | Scope | Commit/rollback behavior | Positive evidence | Open risk |
| --- | --- | --- | --- | --- |
| `with_state_transaction` | all non-battle `GameCoreState` roots | clones root, restores on error, merges monotonic profile | node confirm/complete/checkpoint handlers use it | O(state-size) copy and broad root coupling; boundary is conventionally invoked by handlers |
| `with_staged_battle_advance_state` | `ActiveBattleSession`, VN/trigger, UUID, phase/content | swaps in clones, publishes on success, restores originals on error | covers clock/kernel/result/VN projection as one transaction | clones the complete active battle per tick/advance |
| `with_staged_tactical_world_mutation` | complete `ActiveBattleSession` | mutate clone, commit revision/project DTO, then replace | typed deploy/withdraw/stance/relocate/swap path | cross-layer changes update kernel and application deployment/resource copies in one closure |
| direct active-battle handlers | live `ActiveBattleSession` | validate then mutate/revise/project in place | pause/resume/speed, targeting focus and manual activation have focused success/prevalidation tests | cursor/projection errors can occur after live mutation; convention differs from staged tactical path |
| narrow/manual non-battle staging | economy, roster/inventory or selected session roots | handler clones/plans selected roots and publishes in an authored order | efficient and often atomic for known errors | no outer command invariant; later node/failure/battle/projection work may still fail after publication |
| clone/plan/commit node completion | map/progression + encounter/recovery + final result | plan on clones, commit typed `StagedNodeCompletion` | distinguishes node/floor/run terminal outcomes | planner still reaches multiple root/domain services |
| `BattleEntrySnapshot` | battle-entry relevant root state | retreat/reentry restore | explicit snapshot type | exact coverage to inspect under failure scenario |
| `ActiveRunTransactionRollback` | non-battle root state + seed | host-captured rollback; permanent profile merged monotonic | exported port exists | host must call it around outer persistence transaction |
| `RunCheckpointPayload` | durable active-run subset | versioned JSON export, validate before restore, load counter monotonic | seed/template/power/economy validation | schema coupling and evolution assessed under A08/A10 |

## Map/Node Transition Trace

```text
ViewingMap
  -> SelectMapNode validates graph-derived selectability
  -> NodeConfirm + NodeSession (map location unchanged)
  -> ConfirmEnterNode inside with_state_transaction
     -> clone map/progression
     -> enter_node marks selected node Unavailable/current and clears available set
     -> commit run map/progression
     -> InNode / Gate transition / Battle / typed node content
  -> plan_complete_current_node on clones
  -> StagedNodeCompletion::{NodeCompleted, FloorAdvanced, RunComplete}
  -> typed commit + recovery/checkpoint + ViewingMap/terminal state
```

The map/progression pair has strong local invariants and graph-driven tests. Cross-root atomicity is supplied by application-level
snapshot rollback rather than by a domain aggregate API.

## Interactive Node State Product

The live node phase is currently a product of independently representable values:

```text
GameState
  x Option<NodeSession>
  x Option<ActiveNodeContent>
  x RunState.event_sessions/shop_sessions
  x Option<ActiveBattleSession>
```

Handlers and characterization tests restrict this product to valid combinations, but the types do not. The target migration is
one `ActiveNodeSession` sum type for identity/phase/content, with resumable session persistence delegated explicitly to the run
aggregate. This can be introduced behind current snapshot DTOs without changing facility topology or node gameplay.

## Skill Cast Authority Trace

```text
SkillDef + loader validation
  -> compiled UnitCombatProfile (fragment currently chooses Manual in code)
  -> RuntimeUnit activation/resonance/pending cast
  -> SkillCastSelection + captured SkillCastTarget/tile/start_seq
  -> ActiveSkillCast step progress
  -> projectile/area delivery runtimes
  -> append-only event log + application projection revision
```

The cast epoch and delivery lifecycle are cohesive inside `BattleCore`. Player focus/playback/revision live in
`ActiveBattleSession`, so a player-selected cast is necessarily an application transaction across both authorities. Current
handlers do not all use the same staging mechanism.
