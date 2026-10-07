# Phase 00 Boss Delta Surface Inventory

Purpose: assign current Boss Runtime authority/data/test/consumer seams to later evidence owners without treating a
new module or search hit as a candidate.

Current baseline hashes are in `CURRENT_TREE_BASELINE.md`. Historical Boss claims are inputs only; each boundary
Phase must reread the current source and the original candidate's full invariant.

## Current Authority Path

| Stage | Current source/data evidence | Current role observed in P00 | Reaudit owner |
| --- | --- | --- | --- |
| embedded source inventory | `data/live_ron.rs:gameplay_ron_sources!` includes `data/boss_runtime/base.ron` | Boss catalog participates in the same embedded source set | P01 |
| raw schema | `data/boss_runtime_data.rs:BossRuntimeCatalogData` and authored enums | serde/deny-unknown raw Pattern, behavior, encounter graph | P01/P05 inventory; P06~P09 semantics |
| raw compile | `BossRuntimeCatalogData::compile` | builds indexed immutable patterns/behaviors/encounters and rejects graph/parameter defects | P01 primary; P05 feasibility |
| atomic candidate validation | `ParsedGameplayRon::validate_contract`, `pve_data::validate_boss_runtime_references` | compiles Boss source and checks PVE binding before database publication | P01 primary; P05 cross-check |
| production live content | `game_resources/data/boss_runtime/base.ron`, `data/pve/encounters.ron` | one Little Red pattern/behavior/encounter and one `suppress_little_red` opt-in | P01 data; P05 setup/content seam |
| typed setup | `combat_setup/boss_runtime.rs::resolve_boss_runtime_setup` | maps every actor slot to a resolved opponent `ScenarioUnitRef`; no binding returns `None` | P05 |
| scenario handoff | `events/combat.rs`, `battle/scenario.rs` | typed optional setup enters validated `BattleScenario` before Core construction | P05 |
| private runtime instance | `battle/core/boss_runtime.rs::BossEncounterRuntime`, `BattleCore::new_from_scenario` | battle-session actor/control/pattern/task/epoch/target/route state | P06 lifecycle; P07 movement; P08/P09 leaf seams |
| deterministic scheduling | `battle/enums.rs::BossRuntimeWake`, `core/sim.rs`, `run_boss_runtime_boundary` | private wake has total order; boundary uses staged Core clone and commits only on successful finite microsteps | P06 |
| combat arbitration | `boss_runtime_allows_combat_action`, `basic_attack.rs`, `sim.rs` | resident `RouteCombat` permits ordinary combat; other Boss tasks suppress ordinary action | P06/P08 |
| target/movement | `boss_movement_directive`, `movement/planner.rs`, path/route helpers | Boss runtime selects goal/suppress/fallback; existing continuous movement remains leaf executor | P07/P09 |
| leaf cancellation/resume | `cancel_boss_owned_work`, `resume_boss_route_combat` | cancels unreleased Boss-owned action/movement work and delegates released projectile/damage/death to existing leaves | P08/P09 |
| public projection | `BattleEvent` has no serde derive; `BossRuntimeWake` is internal and processing is a no-op outside boundary trigger | no current public Boss event type observed | P10 verifies entire producer/server/Unity boundary |
| server seam | only current `boss_runtime` server hit is game-data injection in `player_game_actor/handlers.rs` | no Boss-specific result/snapshot mapping found by scoped search | P10 |
| external Unity seam | no `boss_runtime` text in the three current Unity canonical files or searched Assets; generic Little Red compendium text exists | current accessible evidence is common opponent/event/checkpoint consumption, not a Boss lifecycle consumer | P10; deployment remains unverified |

## Live Data Facts To Reverify

- `base.ron` contains exactly one Pattern `little_red_pursuit`, one behavior
  `little_red_reference_behavior` and one encounter `little_red_reference_encounter`.
- Pattern flow is `AcquireTarget -> MoveToTarget -> RejoinForwardRoute`, then authored outcome routes lead to
  resident `RouteCombat`.
- Query is `EnemyUnit`, filters `ActiveHostile + Reachable`, `MinPathCost`, `LockUntilInvalid`.
- `suppress_little_red` alone has `boss_runtime: Some`; it binds `primary_abnormality` to its same-named wave at
  spawn index 0.
- Current `data_loading_contract.md`, `core_runtime_contract.md` and `game_rulebook.md` describe this same replaceable
  reference and no public Boss lifecycle.

P01/P05 must prove exact publication/reachability from current code and production data. P00 does not convert these
facts into candidate closure.

## Boss 19 Impact Allocation

| Boundary | Historical relation members | Why current Boss code intersects | Required full-scope check |
| --- | --- | --- | --- |
| P01 publication | P01-C01 coimplemented; P01-C02 prerequisite | new embedded source and compile/reference checks participate in candidate publication | original all-domain source inventory and every manifested difficulty, not Boss-only subset |
| P05 preview/setup | P05-C02 prerequisite; P05-C04 coimplemented | every live Boss binding resolves typed setup before world commit | full ordinary/Boss setup feasibility and entire typed plan authority |
| P06 lifecycle/result | P06-C01/C02/C03 prerequisite | private wake ordering and staged boundary consume terminal/order/atomic advance substrate | original max-time, all event variants and complete world mutation scope |
| P07 spatial/tactical | P07-C01/C02 independent; P07-C03/C04 deferred | Boss selects target/movement directives through ordinary movement backend | full tactical final-state/atomic commands and complete production backend/tick state inventory |
| P08 effects/stats | P08-C01/C05 independent | Boss cancellation/resume deliberately retains effect/damage/death leaves | full occurrence and committed-damage authorities, not only Boss calls |
| P09 targeting/delivery | P09-C03/C06 independent; P09-C05 deferred | Boss uses path/target query and continuous movement but not skill delivery policy | actual skill delivery/usefulness/projection and projectile motion scope remain separate |
| P10 server/public | P10-C02/C04 independent; P10-C03 conditional | current Boss state is private while existing setup/update/checkpoint are public | all command-result/overlay/battle DTO duplication and accessible external consumers |

The four deferred/conditional rows are not all cleanup: current historical authority says P07-C03/C04 are cleanup,
P09-C05 is a behavior-preserving refactor, and P10-C03 is a behavior-preserving refactor with a conditional public
Boss-wire gate. P09 and Final must normalize the contradictory Boss summary wording.

## Other 32 Current-Tree Impact Allocation

| Phase | Current changed surfaces that can affect non-Boss candidates | P00 disposition |
| --- | --- | --- |
| P01 | `live_ron.rs`, `data/mod.rs`, `pve_data.rs`, many production RON files | all P01 rows plus publication contributions require current impact review |
| P02 | world active-run/combat/snapshot/state/helpers and server actor state/messages/handlers | F-C01 contribution and all P02 rows require current impact review |
| P03 | employee/equipment/reward/item/skill-fragment data, world equipment/facility tests, live data | F-C01/F-C02 contributions and all P03 rows require current impact review |
| P04 | behavior/world node flow/visual novel/event reward paths | F-C02 contribution and all P04 rows require current impact review |
| P05 | combat preview/setup modules and world combat tests | all P05 rows require current impact review |
| P06 | battle sim/enums/event log/scenario/world combat | all P06 rows require current impact review |
| P07 | battlefield/movement plus untracked tactical relocation/stance/swap and shared tests | all P07 rows require current impact review; do not absorb the tactical campaign itself |
| P08 | ability/buffs/death/validation/stats plus field-work/tactical seams | all P08 rows require current impact review |
| P09 | basic attack/targeting/area/range/skill runtime and projectile-adjacent sim | all P09 rows require current impact review |
| P10 | behavior/world command/result/snapshot and server mapping/state/messages | all P10 rows require current impact review |

Shared changed files prove an impact-review obligation, not a merge or a problem.

## Current Test Family Inventory

| Family | Current representative evidence | Assigned review |
| --- | --- | --- |
| Boss compiler unit matrix | `boss_runtime_data.rs` compile success plus duplicate/dangling/capability/graph/query/loop/cooldown negatives | P01 schema/compile meaning; P06~P09 authored semantic assertions |
| embedded candidate negatives | `live_ron.rs` unknown field, compiler defect, dangling PVE ref and unbuildable setup rejection | P01 publication; P05 setup feasibility |
| typed setup unit tests | no binding, immutable resolved frame, route/spawn rejection | P05 |
| scenario parity | `events/combat.rs` ordinary no-binding parity and opt-in setup tests | P05 |
| private runtime kernel | `boss_runtime.rs` 20+ focused tests covering initialization, target identity, same-time spawn, deadlines, stale wake, phase reentry/timing, cancellation, route cursor, movement, invalidation and atomic error | P06 lifecycle; P07~P09 exact leaf assertions |
| world atomicity/contract | `world/tests/combat.rs` setup/advance atomic snapshots, paired ordinary opt-out, pursuit/rejoin and public-shape absence | P05~P10 by assertion owner |
| production live reference | `world/tests/combat.rs` live success, Pause/reconnect/terminal, failure/timeout and non-opt-in parity | P05/P06/P07/P10 |
| official live RON | `tests/ron_loading.rs::live_little_red_is_the_only_boss_runtime_reference_and_preserves_existing_content` | P01/P05 |
| server | historical broad server suite plus current mapping source; no Boss-named focused test found | P10 decides whether existing typed producer tests are sufficient |
| Unity | current accessible source/canonicals searched, no Boss-specific consumer hit | P10 classifies common consumer and unknown deployed state |

Test names, historical green counts and private snapshot helpers are not automatically retained. Each Phase reads
setup/action/assertions and returns `retain`, `strengthen`, `merge`, `remove` or `unverified` with a surviving safety
net.

## Canonical Allocation

| Canonical | Current Boss section | Owner |
| --- | --- | --- |
| `data_loading_contract.md` | strict Boss source, compile, PVE binding and atomic candidate | P01/P05 |
| `game_rulebook.md` | live Little Red reference behavior and explicit non-claims | P05/P07 |
| `core_runtime_contract.md` | private setup/runtime/scheduling/event/checkpoint boundary | P06/P07/P10 |
| `skill_target_contract.md` | no Boss-specific rule found in P00 search; existing skill authority remains relevant | P09 confirms no drift |
| external Unity canonicals | no Boss runtime lifecycle text; common battle transport remains current boundary | P10 |

Historical Goal descriptions are not canonical and are never rewritten by this campaign.

## New Candidate Discovery Queue

P00 confirms material new surfaces but no new candidate yet. Later Phases must examine these possible seams against
actual root-cause evidence:

- raw schema/compiler/publication duplication or a missing atomic validation edge;
- typed setup versus scenario/world duplicate feasibility checks;
- private graph state versus `BattleCore` clone/advance atomicity and wake scheduling;
- Boss target lock/path/movement directives versus existing movement/target query authority;
- cancellation/resume behavior versus effect/projectile/damage/death leaf ownership;
- private debug snapshots or scenario setup leaking into public/save/server/Unity contracts;
- ordinary opt-out paths becoming a parallel fallback rather than a single shared authority;
- tests coupled only to current graph/helper placement.

The existence of these seams is not a defect. A Phase may issue a `CRR-Pxx-Cyy` only with current
authority→producer→consumer or correctness evidence, a target authority, preserved invariant, work type, axes,
safety net and gate. Final deduplicates cross-Phase proposals.

