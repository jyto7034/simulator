# Domain Model Map

- Goal: `current_runtime_architecture_deep_review`
- Status: `complete`
- Baseline: `BASELINE-20260822-064427-KST`

## System Context

```text
host/server adapter
  -> GameCore::execute* / checkpoint and reconnect entry points
     -> action/capability preflight
     -> central PlayerBehavior routing
        -> run/map | headquarters | equipment/maintenance | shop | reward | event | visual novel | battle
     -> GameCoreState roots
        -> domain state + active session + projection state
     -> GameDataBase (immutable Arc catalog)
        -> authored data databases and compiled indexes
```

`GameCore` is the current application root. Its fields are private, but its private `state` is a broad `GameCoreState` shared by
world submodules. `src/game/mod.rs` publicly exposes most domain and implementation modules.

## Current Responsibility Inventory

| Domain/capability | Current modules/types | Authority observed | Boundary status | Evidence |
| --- | --- | --- | --- | --- |
| Application command routing | `world.rs`, `PlayerBehavior`, `BehaviorResult`, `ActionKind` | `GameCore::execute_with_transport_context` | explicit root, broad central dispatcher | `world.rs:147-649` |
| Run lifecycle/mode | `world.rs`, `world/active_run.rs`, `world/state.rs`, `map/progression.rs` | `GameCoreState.run`, `RunProgression`, `pending_game_mode` | typed mode/progression; orchestration spans root state | E-006 |
| Map/node flow | `map/**`, `world/node_flow.rs`, `world/map_content.rs`, `world/map_encounters.rs` | `RunState`/`MapProgression` plus root node session/content | graph/progression core is cohesive; application commit crosses roots | E-007, REQ-002 |
| Battle application session | `world/combat.rs`, `world/state.rs` | `GameCoreState.active_battle` | owns setup, deployment/resource/playback/focus/revision and transactional publish | E-008/E-009 |
| Battle simulation | `battle/core/**` | `BattleCore` and `BattleExecutionState` | deterministic kernel seam exists; imports app/data types and exposes mutable fields | E-009, dependency map |
| Roster/inventory/equipment/fragments | `employee.rs`, `resources/**`, `skill_fragment.rs`, world handlers | multiple roots in `GameCoreState`; fragment loadout compiles to combat profile | good compiler seam; mutation transaction varies by handler | E-013/E-014 |
| Encounter/Monolith progression | `encounter_progression.rs`, `monolith.rs`, BossOmen and world integrations | `RunState.encounter_progression` | cohesive ledger, implicit ownership through `RunState::Deref` | E-011/E-013 |
| Shop/reward/event/facility/HQ | separate world handler modules plus shared root/session state | `GameCoreState` and `RunState` | strong typed engines/sessions; parallel state and handler-selected transactions | E-016/E-017 |
| Visual novel | `visual_novel.rs`, `world/visual_novel_*` | root visual-novel runtime and trigger state | explicit overlay/trigger state cross-cuts inspected pre-run/node/battle paths; remains application orchestration | E-006/E-010/E-017/E-018 |
| Authored data | `data/**`, `GameDataBase` | immutable `Arc<GameDataBase>` | many typed catalogs behind one root | heading/declaration inventory |
| Persistence/continuation | `world/active_run.rs`, checkpoint state | versioned active-run payload + monotonic profile merge | explicit host boundary | `active_run.rs:1-157` |

## Ubiquitous-Language Notes

- `GameMode`, `RunProgression`, `GameState`, `NodeSession`, `ActiveBattleSession`, `PlayerProfileState` and
  `GameDataBase` are explicit types aligned with Rulebook concepts.
- “World” currently means application orchestration, aggregate storage, command routing, projection and several domain
  handlers; it is not a single gameplay domain term. S04/S08 confirm this is the primary architectural refactor boundary.
- “Core” is overloaded between the whole package, `GameCore` application root and `BattleCore` simulation kernel. This is a
  naming/context signal; scenario evidence is required before it becomes a finding.

## Initial Boundary Hypothesis

`GameCoreState` holds at least player, phase, node content/session, run, visual novel, UUID, currency, inventory, roster,
fragments, permanent profile, active battle and checkpoint/power roots. Private module visibility prevents external mutation,
but correctness of multi-root operations currently depends on application handlers coordinating those roots. S01~S08 show
deliberate, well-tested domain engines behind an over-broad application context. The leading target is to keep `GameCore` as a
façade while extracting typed use-case/aggregate transaction ports, not to replace the full domain model.
