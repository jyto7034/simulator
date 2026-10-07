# Representative Change Scenario Matrix

- Goal: `current_runtime_architecture_deep_review`
- Status: `complete`
- Baseline: `BASELINE-20260822-064427-KST`

이 문서는 새 기능의 바람직한 설계를 먼저 가정하지 않는다. 각 시나리오를 current code에 실제로 대입해 owner,
representation, validation, runtime, projection/persistence, test와 compiler guard를 추적한 뒤 disposition을 정한다.

## Scenario Summary

| ID | Representative change | Intended owner | Current owner/surface | Investigation status | Current disposition |
| --- | --- | --- | --- | --- | --- |
| S01 | boss phase/summon/omen/special terminal condition | battle encounter policy + boss runtime; run progression for omen only | compiled Boss DSL, `BattleScenario`, setup resolver, run BossOmen progression | `inspected` | retain battle-local DSL; add explicit encounter/run orchestration instead of expanding one boss monolith |
| S02 | collision/occupancy/route/unit movement mode | battle movement kernel | movement planner/input/engine/backend/blocking plus shared mobility/route types | `inspected` | current subsystem is extensible; introduce explicit policy values before adding a third mode |
| S03 | activation/resonance/target/delivery/interrupt combination | skill definition compiler + battle skill runtime; application only for player commands | `ability.rs`, `data/skill_data.rs`, `battle/core/{sim,target_usefulness,skill_runtime/**}`, `world/combat.rs` | `inspected` | refactor boundary before cross-cutting activation policy; ordinary target/delivery extension remains implementable |
| S04 | run relic and monolith progression | run/growth aggregate | current Monolith plus future Relic pressure inspected | `inspected` | preserve Monolith ledger; add a separate Relic aggregate/compiler and common transaction port |
| S05 | fragment growth/awakening/corrosion/bloom form | fragment inventory/loadout + compiled combat profile | current fragment data/inventory/loadout and future form pressure inspected | `inspected` | evolve the bounded fragment schema to explicit forms; preserve loadout-to-profile compiler |
| S06 | EGO corrosion lifecycle and combat/run consequence | employee/run lifecycle plus battle setup/result policy | current condition/attempt/summon primitives and future lifecycle pressure inspected | `inspected` | add a cross-layer use case with typed battle command and run consequence; do not put the lifecycle in Boss Runtime |
| S07 | generic dice/tool and equipment combat trigger | event resolution/tool inventory + compiled combat trigger input | current event d20/consumable/economy paths and future generic dice pressure inspected | `inspected` | extract pure roll planning/receipt; consumer commits receipt and effect in the common transaction |
| S08 | roguelike node/session/reward/shop/facility | run/map/application use case boundaries | graph/category/payload/session/state/capability/checkpoint paths inspected | `inspected` | retain graph and typed sessions; replace parallel root options/Deref leakage with explicit node-use-case and run ports |
| S09 | long battle, many entities/deliveries and reconnect delta | battle kernel + active-battle application session | full clone and delta scan measured at 1K/10K/100K history and 10/100/1,000 units | `inspected` | retain kernel; rewrite/refactor session transaction and bounded indexed event archive |
| S10 | save/continue equivalent to uninterrupted execution | run checkpoint + deterministic identity/continuation | strict restore tests plus public uninterrupted/continued next-command equivalence probe | `inspected` | retain validator/identity/port; version target DTO and add command-tape differential oracle |

## S03. Skill activation, resonance, target, delivery and interrupt

### Proposed change pressure used for the trace

SRC-10 asks for authored `Manual | Auto` fragment policy now, with any runtime toggle or triggered mode deferred until it has
server-authoritative command, scheduling, idempotency, reconnect and consumption semantics. SRC-04 requires cast selection,
target meaning, reach, step retarget, delivery geometry and interrupt epoch to remain distinct. The representative change is:

> Add a fragment-authored activation policy and one skill combining player-selected target, delayed delivery and interrupt,
> without allowing rejected commands, stale cast-end events or reconnect to duplicate/partially commit the cast.

### Owner and representation trace

| Concern | Current representation/owner | Compiler/static guard | Runtime/test guard |
| --- | --- | --- | --- |
| activation kind | `SkillActivationMode` in `ability.rs`; copied into `UnitCombatProfile` and `RuntimeUnit` | enum forces many matches, but `Triggered` exists without a complete selectable publication/runtime authority | manual/auto scheduling tests; active fragment currently hard-codes `Manual` in `SkillFragmentLoadout::apply_to_profile` |
| cast selector | `SkillCastSelection` separate from `SkillTarget` | loader rejects incompatible selection/target pairs and player selection with `RetargetOnStep` | manual focus/preview/activation tests and readiness projection |
| acquisition range | `SkillTargetReach` + optional `TileRangePattern` in `SkillCastTargetingDef` | validation rejects missing pattern/global conflicts | common tile-reach query and automatic/manual admission |
| step identity/retarget | `SkillStepDef::{id,target,targeting,when,repeat}` | duplicate IDs and invalid combinations rejected | cast state tracks per-step progress/deferred delivery and captured anchor |
| delivery | exhaustive `DeliveryDef::{Instant,Projectile,TileArea}` | projectile scalar/stop/collision and area/anchor contracts validated | dedicated cast/projectile/area runtime modules and contract tests |
| usefulness/admission | `battle/core/target_usefulness.rs` | effect enum exhaustiveness catches new effect classes in several matches | same actual-delivery predicate is reused for auto/manual admission and execution |
| interrupt identity | pending cast stores `start_seq`; end event has parent cause | typed pending/event structures | `InterruptCast` removes matching pending cast; stale end is ignored; tests cover manual/auto interrupt |
| player interaction | `PlayerBehavior`, focus/readiness/capability DTOs and `world/combat.rs` | typed payload and authority DTO | preview is read-only; focused rejection tests preserve revision/focus/event seq |

### Change amplification

- A new target or delivery variant deliberately crosses authoring type, loader validation, execution, usefulness, projection and
  tests. Exhaustive Rust matches catch much of an omitted branch. This is legitimate semantic fan-out, not by itself a design
  defect.
- Fragment-authored `Manual | Auto` is not yet a data-owned policy: active fragments set `Manual` in application/domain logic.
  Adding it requires fragment schema/publication, loadout compilation, runtime profile, readiness projection and tests, but the
  existing `UnitCombatProfile` seam keeps the battle kernel input simple.
- A runtime Auto/Manual toggle is cross-cutting by nature: command/capability, pending-auto scheduling cancellation, focus,
  revision/idempotency and reconnect all change. It should be a new application use case and kernel command, not another mutable
  field exposed to world code.
- Skill execution itself is decomposed into cast/projectile/area modules, yet the supporting `sim.rs` and battle-core root remain
  large and depend on full `GameDataBase` plus behavior-layer error/DTO types. An isolated compiled `BattleSkillCatalog` would
  reduce unrelated context and test construction without replacing the algorithms.

### Transaction and failure boundary result

Deploy/withdraw/stance/relocate/swap use `with_staged_tactical_world_mutation`, which creates the update DTO before publishing the
cloned session. `ActivateSkill`, begin/cancel focus and playback handlers mutate the live `ActiveBattleSession` directly. For
activation, the order is validate → mutate `BattleCore` → reconcile session → increment revision → build cursor-based DTO.
`battle_update_dto_after` can still reject an invalid future cursor after the cast and revision have committed. Existing
"atomic" skill tests exercise prevalidation rejection, focus preservation and successful commit, not this post-mutation error
class. This is evidence of a convention-based, non-uniform application transaction policy (ARCH-REVIEW-004), not a flaw in the
skill cast epoch design itself.

### Disposition

- New target/delivery/interrupt combinations: implement in the current typed design, preferably after narrowing the compiled
  skill catalog boundary.
- Fragment-authored Manual/Auto: implement after moving activation selection into fragment data/publication; no subsystem
  rewrite needed.
- Runtime activation toggle or other multi-state policy: first unify all mutating battle use cases behind one staged transaction
  port and one typed kernel command path.
- Full core rewrite: S03 provides no supporting evidence. It supports an application-boundary refactor and a battle data/input
  boundary extraction while preserving current target/delivery algorithms and tests.

- Evidence: E-010, ROUTE-005, ROUTE-006, ARCH-REVIEW-003, ARCH-REVIEW-004

## S01. Boss phase, summon, omen and special terminal condition

### Representative pressures

- A simple Nothing There-style shared-HP phase chain, including phase skip when a threshold was already crossed.
- A summon/signaling cycle and a signal-owned terminal outcome.
- Normal-boss egg re-entry that starts from an authored later phase and preserves run/checkpoint reward identity.
- The bounded Little Red design section as a deliberately hard case: run-wide call rights, allied contracted actor, target
  reservation, phase-gated damage, same-node hostile continuation and encounter-specific player support action.

### Current boundary trace

```text
BossRuntimeCatalogData
  -> strict graph compiler / reference validation
  -> CompiledBossEncounterDefinition + argument frames
  -> PveEncounter actor-slot binding + setup resolver
  -> ResolvedBossEncounterSetup
  -> BattleScenario { waves, terminal outcomes, optional boss runtime }
  -> BossEncounterRuntime { phase/actor/control/pattern/state/task }
  -> existing movement/summon/combat leaves + emitted signals
```

Simple phases, current tasks, summon cycles and `BossSignalLatched` outcomes are data changes. Compilation rejects duplicate,
dangling, cyclic, non-exhaustive and unsupported graphs before world commit. Phase transition timing, activation IDs, stale wakes,
target locks, cancellation and summon lifecycle have focused deterministic tests. Live production data currently opts in only the
small `suppress_little_red` pursuit reference; the CWBS graph is explicitly nonproduction.

The DSL is intentionally closed. A new task kind crosses authored enum, compiled enum, validation/result/capability matrices,
runtime active-task state, admission/advance/cancellation and tests. Rust exhaustiveness catches much of the omission. The
present `RepathToProtectedObjective` authoring variant is an explicit future vocabulary marker that publication rejects until its
compiled/runtime semantics exist, which is a useful migration pattern.

### Domain split exposed by the hard case

The full Little Red policy is not one boss-AI feature. It includes run progression and currency, an encounter-time player command,
an allied/neutral actor and combat ownership rules, plus post-battle branching. Current Boss actor setup deliberately binds only
unique opponent spawns, so forcing all of it into `BossEncounterRuntime` would blur battle facts with run policy. Likewise the egg
is run inventory/reward/checkpoint state; only its resolved "entry phase/variant" belongs in battle setup.

The appropriate target is three collaborating boundaries:

1. `BossBehaviorGraph`: battle-local AI/phase/pattern/task interpretation.
2. `EncounterOrchestration`: battle-local commands, neutral/allied actor roles, scripted facts and named terminal outcomes.
3. `RunEncounterArc`: omen/egg/call-right/reward/checkpoint progression, resolved into immutable encounter setup.

### Disposition

- Keep and modularize the strict compiler/runtime rather than replacing it.
- Add new battle-local vocabulary only when a mechanic is reusable and has a closed result/capability contract.
- Do not let BossOmen, egg inventory/reward or profile unlock state enter the battle interpreter.
- Add an authored encounter variant/entry-phase field to resolved setup for egg rematches; do not serialize live Boss graph state.
- S01 supports a bounded encounter-orchestration addition and physical module split, not a whole-core rewrite.

- Evidence: E-011, ROUTE-007~ROUTE-010, REQ-007

## S02. Movement collision, occupancy, route and mobility mode

### Current boundary trace

```text
authored MovementDef + MobilityKind + EnemyMovementPlan
  -> UnitCombatProfile / RuntimeUnit / BattleScenario
  -> planner chooses MovementGoal (boss directive first, ordinary policy second)
  -> MovementTickInput projects bodies, terrain policy, locks, goals and static obstacles
  -> MovementEngine / RapierMovementWorld resolves deterministic body motion
  -> MovementOutput applies body/facing/target and append-only segments
  -> separate logical block-state reconciliation owns DefenseRoute interception
```

The subsystem has useful internal seams: typed input/output, canonical unit ordering, a backend-neutral continuous resolver,
separate path/steering/blocking/lifecycle modules and tests for insertion-order determinism, void/static collision, airborne
terrain, overlap and block priority. Boss motion reuses `MovementGoal` and the same leaf instead of owning a parallel physics path.

Current policies are deliberately small:

- `MobilityKind` is Ground/Airborne and is projected to Ground/Airborne terrain policy.
- `EnemyMovementPlan` is only `PathAlongCells`.
- Rapier unit colliders do not physically push/depenetrate one another; DefenseRoute blocking is a separate capacity/radius fact.
- `BattleCore` stores the concrete Rapier backend even though an internal `MovementEngine` trait exists.

### Change fitness

- A new static-terrain collision policy fits as a new explicit engine-input policy; it should not be inferred from role or speed.
- A unit-unit occupancy/collision mode must first decide whether it is physical collision, logical blocking, spawn occupancy or
  targeting. These are currently separate and should remain so.
- A new route form must provide progress, remaining distance, endpoint and boss cursor semantics because blocking/target priority
  consume those facts. A `RoutePlan` query interface would avoid repeating matches outside the planner.
- A third mobility kind has legitimate cross-domain effects on targetability and blocking. Compile-time exhaustive matches are
  valuable; a bag of booleans would weaken the model.
- Backend injection at the `BattleCore` construction boundary would make isolated movement tests lighter, but no alternate
  production backend is currently required.

### Disposition

Implement after small refactors: introduce `RoutePlan` queries and explicit collision/terrain/occupancy policies, then extend the
existing planner/input/engine pipeline. Preserve logical blocking and deterministic test oracles. No movement subsystem rewrite is
currently justified.

- Evidence: E-012, ROUTE-004, ARCH-REVIEW-003

## S04. Run relic and Monolith progression

### Current asset and future pressure

`MonolithRunState` is a cohesive run ledger for per-abnormality victories, remanifestation nodes, residual exchange, shop
guarantee and corrupted-key progression. Its methods validate and resolve natural victory, conversion, purchase and staged
remanifestation outcomes. It is preserved through `EncounterProgressionState`, floor transitions and checkpoint payloads. The
cost is integration fan-out: map conversion, encounter assignment, shop generation, VN exchange, combat result, snapshots and
capabilities all know Monolith details.

SRC-05 describes a different mechanic: run-local horizontal modifiers with typed fragment/weapon/tag/universal compatibility,
explicit mutation hooks, exclusivity and equipment-opened slots. Current production has no Relic state or catalog. Copying the
Monolith integration pattern directly would add another set of `GameCore`/snapshot/checkpoint/shop/reward switches and would
also tempt runtime code to mutate raw `SkillDef` or equipment definitions.

### Target ownership and change path

```text
authored RelicDefinition + compatibility/exclusive group
  -> publication validation
  -> RunRelicState { inventory, opened slots, selected loadout }
  -> RelicLoadoutCompiler
     -> immutable BattleModifierSet for battle setup
     -> explicit non-battle effect plan for shop/reward/run commands
  -> snapshot/checkpoint projection through one RunState schema descriptor
```

Monolith remains inside encounter progression because its identity is abnormality encounter mastery/remanifestation. Relics are
their own growth aggregate even when acquired from Monolith rewards. Battle modifiers are compiled values; neither the relic
aggregate nor application handler should mutate shared authored catalog objects.

### Disposition

- Preserve Monolith ledger methods, deterministic node identity and existing characterization tests.
- Introduce `RunRelicState` and an immutable battle modifier compiler behind a typed run-aggregate port.
- Make reward/shop acquisition and loadout changes use the same application transaction as inventory and checkpoint-visible
  state.
- Do not rewrite the full run system. S04 supports extracting run progression/growth ports before implementing Relics.

- Evidence: E-013, ROUTE-011~013, ARCH-REVIEW-001

## S05. Fragment growth, awakening, corrosion and bloom form

### Current boundary trace

```text
SkillFragmentEffectDef
  -> strict database/reference validation
  -> SkillFragmentInventory { copies, dust, upgrade/awakening progress }
  -> per-employee SkillFragmentLoadout
  -> apply_to_profile
  -> UnitCombatProfile / battle setup
```

This separation is a strong migration seam. Ownership/progression and employee selection are not battle runtime state, and the
compiler chooses the effective skill before battle. The present progression model is ordinal and singular: upgrade level,
awakening availability and one irreversible `awakened: bool`; active fragments choose awakened, else highest upgrade, else base,
and currently force Manual activation.

SRC-06's future pressure may remove upgrade semantics and make awakened/corrosion/bloom variants selectable forms. A Boolean
cannot represent multiple unlocked forms plus one selected form, and treating corrosion as just another bool would hide its
equipment-set, command and post-attempt consequences.

### Disposition

- Keep inventory/loadout/profile compilation and strict fragment publication.
- If the policy is approved, migrate the bounded schema to stable `FragmentFormId`, an unlocked-form set and an explicit
  selected form. Make selection a Maintenance command and checkpointed run fact.
- Put activation mode and form-specific compiled skills in fragment authoring/compiler output, not handler literals.
- Keep EGO corrosion orchestration separate from ordinary form selection (S06).
- No battle/core or whole-project rewrite is justified; this is a versioned fragment-schema migration.

- Evidence: E-014, ROUTE-011/013

## S06. EGO corrosion lifecycle

### Boundary decomposition

SRC-08 requires a same-origin fragment/weapon/armor/accessory set, full resonance and explicit input; it withdraws an employee,
summons the allied original abnormality with one corrosion skill, despawns it at the end, writes trauma 100 and starts a
Breakdown consequence. Current code already has useful pieces: typed equipment/fragment ownership, derived employee condition,
live withdraw/summon primitives, attempt-local condition staging, named terminal facts and battle result commit.

It does not have the EGO corrosion lifecycle. `EmployeeCondition` only stores physical damage and trauma; Breakdown is derived
from trauma 100 and currently has no node-collapse deadline. Summon runtime is battle-local, while equipment-set eligibility and
the deadline are run/application facts.

The feature therefore needs three collaborating owners:

1. `CorrosionEligibility`: pure query over compiled employee loadout, origin IDs, resonance and current battle state.
2. `BattleCorrosionCommand`: one typed kernel/application command that withdraws the employee and owns the summoned ally's
   lifecycle and presentation signal.
3. `ConditionConsequenceState`: run/checkpoint deadline ledger started during result commit, not another scalar hidden in
   `EmployeeCondition`.

Presentation freezing should be represented by a reasoned battle clock gate (`PlayerPause`, disconnect/presentation freeze,
etc.) so restoration precedence is explicit rather than by adding another Boolean pause flag.

### Disposition

Implement only after all mutating battle/non-battle commands share a publish-after-projection transaction port. Reuse summon,
withdraw, condition and result primitives, but coordinate them through a new application use case. This is a cross-layer feature,
not evidence that Boss Runtime or BattleCore should absorb run policy, and not evidence for a full rewrite.

- Evidence: E-015, ROUTE-014, REQ-006, ARCH-REVIEW-004

## S07. Generic dice, tool item and combat trigger

### Current oracle and missing abstraction

The Event d20 path is a good characterization oracle: a node-seeded roll is deterministic, natural 1/20 is explicit, the exact
receipt is committed in `EventSessionState`, duplicate resolution reuses it, and rejected input does not consume a roll. Current
consumables and reward grants also plan against cloned inventory/roster/economy roots before publication.

SRC-09 asks for consumer-defined faces/critical rules, stable request identity, provenance and atomic effect resolution. There is
no generic dice service or Giant Sap-style run ledger today. The dice component must not own event/tool effects; otherwise every
consumer rule moves into a new central switch.

### Target change path

```text
consumer builds RollRequest { stable request_id, distribution, critical policy, seed context }
  -> pure DiceResolver::plan -> RollReceipt
  -> consumer validates effect plan
  -> one application transaction commits consumer ledger + receipt + effect
  -> snapshot exposes receipt/provenance
```

Equipment combat triggers should likewise compile from authored equipment into immutable battle trigger inputs. Run-limited tool
uses belong in a run/checkpoint ledger, not `BattleCore` or the global data catalog.

### Disposition

Extract a small pure roll planner and shared receipt type, using Event d20 tests as the compatibility oracle. Add tool/equipment
consumer owners separately and commit through the common transaction boundary. No subsystem rewrite is needed.

- Evidence: E-016, ROUTE-015, ARCH-REVIEW-001/004

## S08. Roguelike node, session, reward, shop and facility extension

### Current extension surface

Facility topology is data-driven through `FacilityTemplate`, slot constraints, graph edges and `MapNodeKindId`. A new content
kind that fits an existing category can often remain a data/payload change. A genuinely new category crosses the closed
`MapNodeCategory` and usually also `MapNodePayload`, `ActiveNodeContent`, `GameState`/context, entry routing, capability/action,
snapshot and tests. Thirty-nine production/test files mention `MapNodeCategory`; exhaustive matches provide protection, while
the semantic fan-out remains real.

Interactive authority is split among parallel roots:

- `NodeSession` carries identity/category/payload at the application root.
- `ActiveNodeContent` carries the live typed session at the application root.
- resumable Event/Shop sessions live again in maps inside `RunState`.
- `GameState` independently records InNode/InShop/InReward/Battle phases.
- checkpoint uses an explicit selected RunState field list.

The combinations are protected mostly by handlers and tests, not by one representable `ActiveNodeSession` state. In addition,
`RunState: Deref<Target = EncounterProgressionState>` makes `run.monolith`/`run.boss_omen` look like direct fields, obscuring
which aggregate owns mutation and persistence. This reduces navigation cost locally but weakens ownership clarity for an AI-led
solo codebase.

### Transaction and extension result

`with_state_transaction` gives strong rollback for node confirm/complete, HQ resolution, sweep and checkpoint load. Economy
grant state and several inventory/loadout handlers use narrower staging. Invocation is still handler-local: reward/maintenance/
shop/event families do not automatically enter one root transaction, and root projection/VN attachment can remain fallible after
domain mutation. A new node author must choose the correct convention manually.

The target is not an open-ended plugin framework. It is:

- one explicit `ActiveNodeSession` sum type that owns identity, phase and typed content together;
- an explicit `RunContext`/`RunMutation` port instead of `Deref` field leakage;
- a node-kind registry/compiler that maps authored kind to a closed reusable category handler and validates payload shape;
- one application command transaction that includes effect, node completion, projection and durable-publication plan.

### Disposition

Preserve the facility graph/generator, deterministic node seeding, typed content sessions, reward grant planner and node tests.
Refactor the application/session boundary incrementally, then add new roguelike nodes behind the registry/use-case port. S08 is
strong evidence for a bounded `world` application-layer redesign, but the cohesive map and content engines argue against a
whole-core rewrite.

- Evidence: E-017, ROUTE-016/017, REQ-002/005/006, ARCH-REVIEW-001/004

## S09. Long battle, retained state and reconnect delta

### Current scaling

The ordinary live fixture is light: its one-ms tick p95 was 10.848us and current tiny-history reconnect tail p95 was 1.298us.
The architecture pressure appears as state grows. At 1K/10K/100K retained events, a `BattleCore` clone p95 grew from 13.576us
to 125.243us to 1.879ms. Asking for only the last 10 events grew from 0.802us to 3.940us to 96.916us because
`entries_after_seq` scans the complete `Vec`. A full 100K delta took 2.741ms p95 and the serialized log was 12.78MB.

Correctness staging clones the complete `ActiveBattleSession`, of which `BattleCore` is only one part. Synthetic staged
one-ms work grew near-linearly from 2.930us at 10 units to 207.524us at 1,000 units. The probe did not include an event-heavy
production boss or allocations, so it proves complexity and headroom pressure rather than current player-visible slowness.

### Disposition

- Preserve deterministic simulation, typed event semantics, revision/cursor contract and publish-after-projection rollback.
- Index event sequence and split the bounded reconnect/presentation archive from result/debug facts.
- Generalize the application mutation port first, then replace complete-session cloning with touched-root staging,
  copy-on-write immutable data or a validated journal behind that port.
- Add deterministic event/time/byte retention and an event-heavy boss/projectile/effect benchmark before release.
- Do not rewrite movement, skill execution or Boss Runtime to address a session/archive mechanism.

- Evidence: E-020, `PERFORMANCE_BUDGET.md`, ARCH-REVIEW-002, GAP-001

## S10. Save/continue equivalence

### Current evidence

Core V4 separates permanent profile/transport identity from checkpoint-owned run state, validates the full graph before
publication and reconstructs seed/UUID authority on continue. Existing tests pin strict fields, reject legacy/extra/missing
shapes, reject 19 graph corruptions before mutation, compare restore-recapture semantics and replay a gate deterministically.

E-021 exercised the public boundary: a completed SavePoint was exported, continued into a fresh `GameCore` constructed with a
different seed, and compared with the uninterrupted source. Run snapshot, next-node selection result and node-confirm result
were all equal. This is a viable old/new architecture oracle, not proof for every future battle or platform.

### Disposition

- Preserve seed ownership, namespaced UUID streams, strict graph validator, monotonic exception rules and export/continue seam.
- Define a target checkpoint DTO independent of `GameCoreState` layout; keep current V4 as an explicit import oracle rather than
  maintaining a dual mutable authority.
- Add deterministic command tapes spanning battle result, event/shop/reward, Monolith and future Relic/form/corrosion state.
- Compare canonical observable state/results/events, then re-export and continue again to detect hidden divergence.
- Treat cross-platform physics equivalence and retention policy as isolated gaps, not reasons for a big-bang rewrite.

- Evidence: E-019/E-021, `DETERMINISM_CONTINUATION_MAP.md`, GAP-002
