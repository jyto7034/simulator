# Dependency Direction Map

- Goal: `current_runtime_architecture_deep_review`
- Status: `complete`
- Baseline: `BASELINE-20260822-064427-KST`

## Desired Direction For Evaluation

```text
host / transport / persistence adapters
  -> application use cases and transaction ports
     -> run/map/economy/battle domain APIs
        -> deterministic domain values and compiled immutable inputs

authoring DTO + loader + validation
  -> compiler/publication
     -> compact compiled domain inputs

projection DTO <- application/domain query models
```

Domain kernels should not depend on transport DTOs, host I/O or the complete authoring database. The application layer may
coordinate multiple aggregates but should use bounded command/query interfaces.

## Current Static Direction

| Source | Depends on | Direction assessment | Evidence |
| --- | --- | --- | --- |
| `world.rs` / `world/**` | behavior DTO/error, all run/map/battle/economy domains, `GameDataBase`, tracing | expected application fan-in, but broad | central behavior routing and handlers |
| `events/combat.rs` | combat setup, battle core, roster/inventory/fragments, full `GameDataBase`, `GameError` | setup service spans authoring/domain/application error | imports and `ResolvedCombatSetupPlan` |
| `battle/core/**` | ability/stats/battle types, `resources::Position`, data authoring types, full `GameDataBase`, behavior DTO/error | inward dependency is not clean: kernel imports application projection/error and authoring root | `core/mod.rs`, `sim.rs`, `types.rs` imports |
| `battle/core/movement/**` | battle IDs, resource `Position`, Rapier backend | mostly bounded subsystem, coupled to shared coordinate type | imports and module visibility |
| `data/**` | many authored DBs and some domain types | expected authoring aggregation; publication object is very broad | `GameDataBase` fields/builder |
| `behavior.rs` | domain IDs/enums and projection DTOs | transport/application contract collects most public shapes | public command/result/error surface |
| `src/game/mod.rs` | publicly exposes nearly every module | compiler does not enforce a narrow consumer surface | 28 public modules |

## Battle Boundary Trace

```text
world/combat
  -> CombatExecutor (setup across preview/data/roster/inventory/fragments)
  -> BattleCore + BattleExecutionState
  -> BattleLiveCommand / BattleStepOutcome
  -> world ActiveBattleSession (deployment/resource/playback/focus/revision)
  -> behavior projection DTOs
```

Positive seams:

- `ResolvedCombatSetupPlan` is crate-private and explicitly hands validated setup to world ownership.
- `BattleLiveCommand` and `BattleLiveCommandOutcome` form a typed mutation seam.
- Battle application mutations stage and publish one cloned session; result projection happens before publish.

Direction concerns to validate:

- `BattleCore` stores `Arc<GameDataBase>` rather than a battle-scoped compiled catalog and imports `GameError`,
  `ActionDisabledReasonCode` and projection DTOs from `behavior`.
- `BattleCore` exposes mutable `units`, `battlefield`, `game_data`, event log and sequence fields publicly, so module/crate
  consumers can bypass the typed command seam.
- Active battle authority is split between application session state and kernel state; tactical handlers update both under a
  clone transaction. Movement/skill/boss scenarios must show whether that split is stable or causes synchronization burden.

## Skill Extension Trace

```text
skill/fragment authoring
  -> `SkillDef` / `SkillFragmentEffectDef`
  -> loader validation
  -> `UnitCombatProfile`
  -> `RuntimeUnit` + cast/target/delivery runtimes
  -> event log
  -> readiness/catalog/checkpoint DTOs
  -> world focus/activation use case
```

Target and delivery additions have intentional semantic fan-out, and enum exhaustiveness plus loader validation provide useful
compiler/static protection. Activation policy crosses authoring, compiled profile, kernel scheduling and application interaction.
The existing `UnitCombatProfile` is a good seam, but the kernel still reads skills/buffs through full `GameDataBase` and returns
behavior-layer errors/readiness DTOs. S03 therefore favors a battle-scoped compiled catalog and application transaction façade,
not replacement of the skill algorithms.

## Boss And Movement Extension Traces

- Boss authoring has a real compile boundary: raw IDs/graphs become indexed immutable definitions and resolved setup before the
  kernel. This is the strongest current example of the desired authoring → compiler → domain direction.
- The setup resolver still accepts full `GameDataBase`; its output is appropriately narrow. Moving resolution behind a compiled
  encounter-catalog port would improve isolation without changing the graph.
- Movement's planner/input/output/backend are crate-internal and cohesive. The engine consumes battle-neutral-ish movement values,
  but `BattleCore` constructs inputs and stores a concrete Rapier world. A port at construction is a testability option, not a
  current product requirement.
- `EnemyMovementPlan` is read by planner, blocking/progress and boss cursor logic. Extracting route queries would make a new route
  representation compiler-enforced without teaching each consumer its storage shape.

## Run, Growth And Node Extension Traces

- `EncounterProgressionState` is a useful domain owner for reward-source completion, encounter cycles, Monolith and BossOmen.
  `RunState` implements `Deref/DerefMut` to it, however, so world code can write `run.monolith` as if the child state were a root
  field. Removing this convenience in favor of an explicit encounter-progression port would make dependency/ownership visible.
- Fragment inventory/loadout compiles into `UnitCombatProfile`, which is the desired run-growth → battle direction. Future Relic
  and fragment forms should follow the same immutable compiler path.
- Node authoring is partly data-driven (`MapNodeKindId`, facility slots/constraints), while reusable runtime categories are
  closed enums. This is appropriate, but phase/session/persistence are split across `GameState`, `NodeSession`,
  `ActiveNodeContent` and `RunState` session maps.
- World handlers call domains directly and choose transaction scope individually. The missing layer is not another gameplay
  engine; it is a narrow use-case/transaction port between central routing and existing domain APIs.

## Compiler Enforcement Snapshot

- External callers cannot access private `GameCore.state`.
- World submodules can access broad state because they are sibling implementation modules.
- Many battle internals are private or `pub(in crate::game::battle::core)`, but the root `BattleCore` still has several public
  mutable fields and the enclosing `battle::core` module is public.
- No crate split prevents authoring/DTO/application dependencies from entering the battle kernel.
- No type-level route currently forces every mutating `PlayerBehavior` through a publish-after-projection transaction, although
  several local helpers implement that shape successfully.
