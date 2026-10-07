# Requirement Preservation Map

- Goal: `current_runtime_architecture_deep_review`
- Status: `complete`
- Baseline: `BASELINE-20260822-064427-KST`
- Normative gameplay source: `docs/game_rulebook.md`

Current behavior is not copied into this map as a requirement. Each row distinguishes policy, implementation alignment,
technical detail, drift/unknown and rewrite disposition.

## Capability Inventory

| ID | Rulebook capability | Policy section | Production entry/owner | Implementation status | Evidence/tests | Preservation disposition |
| --- | --- | --- | --- | --- | --- | --- |
| REQ-001 | Explicit `Standard`/`Endless` mode, pre-run selection, tutorial gate and mode-owned progression | Rulebook 52-141 | `PlayerBehavior::StartNewGame`; `GameCore::handle_start_new_game`; `RunProgression` | `latest_policy_match` for inspected path | `world.rs:147-249, 669-805`; `map/progression.rs:26-120`; start/mode/endless/standard tests | preserve gameplay meaning; internal orchestration/state representation may change |
| REQ-002 | Exploration/map/node selection and briefing | Rulebook 142-337 | `MapProgression`, `FacilityMapGenerator`, `world/node_flow.rs`, combat preview | `latest_policy_match` for inspected graph/select/confirm/complete and representative event/shop/reward paths | map progression, map-flow, event/shop/reward tests; E-007/E-016/E-017 | preserve topology, selectability, typed node lifecycle and briefing meaning; representation may change |
| REQ-003 | DefenseRoute battle, tactical commands, setup, waves and targeting | Rulebook 338-718 | `world/combat.rs`, `ActiveBattleSession`, `CombatExecutor`, `BattleCore` | `latest_policy_match` for inspected lifecycle, tactical, target/delivery/interrupt and kernel paths | extensive unit/world/integration tests; E-008/E-009/E-010 | preserve observable semantics and deterministic kernel oracle; boundary/implementation may change |
| REQ-004 | Reward, growth, compendium, equipment and skill fragments | Rulebook 719-1005 | `EconomyGrantState`, `EncounterProgressionState`, `MonolithRunState`, inventory/loadout/fragment compiler and Maintenance handlers | `latest_policy_match` for inspected current paths; future Relic/form policy is not implemented | E-013/E-014 and focused grant/fragment/monolith tests | preserve current gameplay and compiler seams; evolve growth schemas explicitly when drafts become policy |
| REQ-005 | SavePoint, Maintenance and Headquarters non-combat nodes | Rulebook 1006-1074 | map content routing, typed node sessions, node completion transaction, checkpoint and HQ handlers | `latest_policy_match` for inspected lifecycle and atomic node completion | E-017; node session/checkpoint/maintenance/HQ tests | preserve distinct node roles, capability ownership and atomic completion; session representation may change |
| REQ-006 | Node results, retreat, employee condition and recoverable/unrecoverable run failure | Rulebook 1075-1173 | combat result staging, `BattleEntrySnapshot`, `EmployeeCondition`, recovery-path evaluator, failure decision state | `partial_drift`: core result/retreat/failure paths exist; Breakdown node-collapse deadline is absent | E-015/E-017; combat/result/retreat/failure tests | preserve result/rollback/failure semantics; add explicit checkpointed condition-consequence ledger |
| REQ-007 | BossOmen/final-boss progression and legacy-removal policy | Rulebook 1188-1266 | `EncounterProgressionState.boss_omen`, map overlay/capstone and battle-local `boss_runtime` | `latest_policy_match` for inspected run/battle boundary; production content remains bootstrap | E-011/E-013; boss omen/facility/boss runtime tests | preserve approved run policy and battle-local deterministic graph; bootstrap content is not final design |

## REQ-007 Boundary Classification

- FinalBoss chain eligibility, provisional/active/completed state, cycle, capstone map replacement, reward and checkpoint rollback
  are run-progression responsibilities.
- Battle Boss Runtime is an opt-in behavior graph over immutable resolved setup. It does not and should not own chain, egg,
  reward, profile or checkpoint state.
- Current live Boss Runtime evidence is one intentionally small Little Red pursuit reference; it proves the pipeline, not fitness
  for final boss content.
- S01 found a viable migration seam: resolve run/encounter policy into a battle-local variant/entry phase and retain the compiled
  graph as the deterministic execution oracle.

## REQ-001 Trace

```text
PlayerBehavior::StartNewGame { game_mode }
  -> GameCore::execute_with_transport_context
  -> gate_behavior_action
  -> execute_validated_behavior_domain
  -> execute_map_run_behavior
  -> handle_start_new_game
  -> PreRunBriefing + visual-novel trigger + pending_game_mode
  -> SelectStarterEmployees
  -> RunProgression::new(run_seed, game_mode, standard_floor_count)
  -> generate_current_floor_map
  -> RunState + ViewingMap
```

Positive protection:

- `GameMode` is a required typed command field and stored in `RunProgression`.
- Endless entry checks the monotonic player-profile tutorial flag before mutating run state.
- Standard and Endless use an enum-owned progression state, and tests pin tutorial gating, requested-mode persistence,
  unbounded Endless gate progression and Standard terminal completion.

Open architecture questions:

- `pending_game_mode`, pre-run visual novel, roster reset, profile gate and run construction are orchestrated directly by
  `GameCore`; change amplification is assessed under A01/A05 rather than treated as policy drift.
- Rulebook references external Unity canonical that the user has declared stale. That reference does not override the Goal's
  source precedence and will not be used as current policy evidence.

## REQ-002 Trace

```text
RunProgression + FacilityRunState + current power anchor
  -> FacilityMapGenerator::generate_with_requirements
  -> assign_map_encounters / BossOmen overlay
  -> RunMap + MapProgression::from_map
  -> current_map_view derives selectable ids, threat and readiness
  -> SelectMapNode validates and opens NodeConfirm without moving
  -> ConfirmEnterNode transaction revalidates and enters node
  -> typed node handler / battle / gate
  -> staged completion commits node, next floor or run terminal
```

Positive protection:

- Facility topology and directional transit drive selectable nodes; current position is not inferred by Unity.
- Selection and confirmation are separate, and combat admission is revalidated before node consumption.
- Node completion plans typed terminal outcomes on cloned map/progression and commits through a common boundary.
- Tests cover transit-only completed nodes, concealed selectability, forward-only edges, invalid entry no-mutation, terminal
  boundaries, Endless gate progression and Standard completion.

Completed classification:

- Event/shop/reward and new-node application paths were traced under S07/S08. Inventory/economy staging is represented in
  REQ-004~006 and E-016/E-017.
- Whole-root snapshot rollback is positive correctness evidence but its copy cost and change coupling remain A05/A09 questions.

## REQ-003 Trace

```text
Combat/Boss node confirm
  -> plan setup from roster/inventory/fragments/data/preview
  -> ResolvedCombatSetupPlan { BattleCore, BattleExecutionState, outcome policies }
  -> ActiveBattleSession { kernel + deployment/resource/playback/focus/revision }
  -> server tick or typed tactical command
  -> staged ActiveBattleSession/BattleCore mutation
  -> revisioned event delta + checkpoint projection
  -> named terminal outcome
  -> staged CombatResult / node completion / failure decision
```

Positive protection:

- Only DefenseRoute live setup is admitted in inspected path.
- Typed commands represent deploy, withdraw, stance, relocation, swap and skill activation.
- Tick advance and tactical commands clone a stage, validate cursor/revision, project the result, then publish once.
- Stable `BattleExecutionState` owns terminal outcome/finalization; event log is observation rather than RNG authority.
- Broad tests pin tactical rollback, pause/reconnect, targeting, delivery, same-time ordering, boss/wave/summon behavior and
  result transitions.

Architecture questions:

- Every tick/command stage clones the whole active session, including a large `BattleCore`; resource scaling must be measured.
- Battle domain imports full authored database and behavior-layer types; a target architecture may need a compiled battle
  context and kernel-local error/query models.
- SRC-04's selected target/cast/delivery/interrupt responsibilities match the current typed implementation. Preserve the
  separation of cast selector, target meaning, acquisition reach, step retarget, delivery footprint and cast epoch.
- Tactical mutation atomicity is not uniform: deployment/movement commands stage a complete session, while skill focus,
  activation and playback mutate the live session before all cursor/projection work has succeeded. This is an application
  boundary issue, not a policy requirement to preserve.

## REQ-004~006 Trace And Drift Classification

Current growth and non-combat features are not one domain despite sharing `GameCoreState`:

```text
authored reward/equipment/fragment/encounter data
  -> strict catalogs and deterministic selection
  -> EconomyGrantState / Inventory / SkillFragmentInventory / EncounterProgressionState
  -> loadout compiler or typed interactive node session
  -> application transaction + node completion
  -> RunState/checkpoint/snapshot
```

Positive protection:

- Rewards stage economy, inventory, fragment and employee experience changes together.
- Monolith and encounter completion use explicit run ledgers rather than inferring progress from inventory or event logs.
- Fragment ownership/loadout compiles into `UnitCombatProfile`; the battle does not mutate growth state.
- SavePoint, Maintenance and HeadquartersContact are distinct categories and active session types. Node confirmation/completion
  and HQ terminal choices are wrapped in `with_state_transaction`.
- Retreat owns an explicit battle-entry snapshot; failure candidates and checkpoint choice are typed states rather than a
  generic `GameOver` branch.

Current drift and architecture limits:

- Rulebook Breakdown is a checkpointed node-completion deadline. Current `EmployeeCondition` derives Breakdown only from
  `trauma == 100`; no collapse-deadline ledger or remaining-node projection exists.
- Interactive node authority is represented in parallel by `GameState`, `NodeSession`, `ActiveNodeContent` and RunState session
  maps. Valid combinations are enforced by handler convention/tests rather than one sum type.
- `RunState` dereferences to `EncounterProgressionState`, allowing implicit `run.monolith`/`run.boss_omen` access and hiding the
  real aggregate owner.
- Narrow inventory/economy staging is valuable, but not every mutating command family automatically enters the root transaction.
  Preserve the observable atomic policy while unifying the application port.
