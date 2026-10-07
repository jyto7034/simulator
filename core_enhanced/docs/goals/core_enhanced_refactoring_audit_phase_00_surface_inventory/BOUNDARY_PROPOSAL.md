# Core Enhanced Refactoring Audit — Boundary Phase Proposal

Proposal Date: 2026-08-07
Status: user approval required before child creation or investigation
Source: [SURFACE_INVENTORY.md](SURFACE_INVENTORY.md)

## Boundary Design Rules

- Each Phase follows one authoritative state/data/contract lifecycle across code, data, tests and direct
  supporting consumer seams.
- Physical files that host several authorities (`world/combat.rs`, `world/state.rs`, `behavior.rs`,
  `battle/core/mod.rs`) are shared evidence seams; their existence does not justify merging all contracts.
- One Phase may identify multiple candidates inside its one authority family, but must not merge independent
  gameplay contracts into one candidate.
- Every Phase remains `investigation_only`, production mutation `no`, and must finish `Complete / High` before
  the next Phase becomes active.
- Candidate IDs are created by boundary Phases, not by Phase 0. A suggested prefix is `CRA-PNN-CNN`.

## Proposed Sequential Phases

| Phase | Proposed child name | Independent authority boundary | Required output |
| --- | --- | --- | --- |
| 1 | `core_enhanced_refactoring_audit_phase_01_live_data_loading_and_validation` | Embedded bundle, typed catalogs, strict parse/validation/publication | Data/loader candidates, source-count drift disposition, live/config/fixture readiness |
| 2 | `core_enhanced_refactoring_audit_phase_02_run_map_and_persistence` | `GameState`/`RunState`, map/floor/node progression, checkpoint, compendium/monolith/boss state and server persistence seam | Run/map/persistence candidates and rollback/lifetime safety nets |
| 3 | `core_enhanced_refactoring_audit_phase_03_roster_inventory_and_economy` | Employee roster/condition, inventory/item slots, fragments, growth/research, Enkephalin, grants/shop/reward/headquarters transactions | Domain ownership/atomicity candidates and economy safety nets |
| 4 | `core_enhanced_refactoring_audit_phase_04_interactive_node_and_visual_novel` | Event/facility/node sessions and VN session/trigger lifecycle | Session/lifecycle candidates, event/VN authority and focused tests |
| 5 | `core_enhanced_refactoring_audit_phase_05_combat_preview_and_setup` | PvE encounter/wave resolution, preview, mission/battlefield/player/enemy setup and run→battle handoff | Preview/setup/handoff candidates and generated-contract safety nets |
| 6 | `core_enhanced_refactoring_audit_phase_06_battle_lifecycle_and_result` | `ActiveBattleSession`, `BattleCore`, simulation clock/event queue/log, scenario, finish/finalize/result/retreat/recovery | Battle lifecycle/result candidates and characterization gates |
| 7 | `core_enhanced_refactoring_audit_phase_07_battle_spatial_and_tactical` | Battlefield/occupancy, continuous movement, deploy/withdraw, tactical resource/stance/relocation/swap/field work | Spatial/tactical candidates and timing/occupancy invariants |
| 8 | `core_enhanced_refactoring_audit_phase_08_battle_effects_and_stats` | Persistent→runtime stats, damage, buffs/control, cooldown, triggers, resonance, death/incapacitation | Effect/stat/lifecycle candidates and exact state-transition safety nets |
| 9 | `core_enhanced_refactoring_audit_phase_09_battle_targeting_and_delivery` | Basic attack/skill cast targeting, usefulness, tile range, projectile/area/step delivery and range projection | Targeting/delivery candidates and behavior-preservation boundaries |
| 10 | `core_enhanced_refactoring_audit_phase_10_command_projection_and_server_seam` | `PlayerBehavior`, `BehaviorResult`, capability/snapshot/DTO projection, admin/fixture/package targets and direct server mapping | Command/projection/transport candidates, consumer classification and fixture/test-harness disposition |
| Final | `core_enhanced_refactoring_audit_final_catalog_consolidation` | Cross-boundary competing authority, duplicate root causes, priority/readiness and disposition catalog | Master final candidate catalog and `Complete / High` audit |

## Dependency Rationale

1. Phase 1 establishes what data is actually published before any runtime candidate cites authored meaning.
2. Phase 2 establishes run/map/persistence lifetimes used by all node/economy/combat handoffs.
3. Phase 3 establishes roster/inventory/economy authority used by interactive nodes and combat setup.
4. Phase 4 closes independent non-combat session/VN lifecycle before battle-focused work.
5. Phase 5 establishes the run→preview→battle input seam.
6. Phase 6 establishes battle clock/event/result lifecycle shared by spatial, effects and targeting.
7. Phase 7 establishes position/occupancy/tactical authority used by damage and targeting delivery.
8. Phase 8 establishes stats/effects/death semantics consumed by targeting/delivery outcomes.
9. Phase 9 audits targeting and delivery with data, spatial and effect authorities already mapped.
10. Phase 10 audits derived commands/projections and the direct server consumer after all producer domains are
   known, avoiding DTO-driven redefinition of Core policy.
11. Final dedupes candidates that touched shared host files and resolves competing target authority.

This is a sequential evidence order, not a claim that every Phase mutates or owns the preceding Phase's code.

## Phase Evidence Contract

Every boundary Phase must produce at least:

- surface coverage table for its primary and shared paths;
- actual authority→mutation→producer→consumer trace;
- live-data/canonical/test evidence where applicable;
- Candidate ID or explicit rejected/deferred/unverified disposition for every investigated signal;
- work type (`cleanup`, `behavior_preserving_refactor`, `correctness_correction`,
  `policy_or_feature_migration`);
- current and target authority, removal/consolidation scope and preserved invariants;
- seven priority axes with evidence or `unknown`, plus a separate readiness verdict;
- characterization/focused/broad validation requirements;
- policy/external-contract decisions and consumer classification;
- no-production-mutation and dirty-preservation evidence;
- `COMPLETION_AUDIT.md` with `Complete / High` before next activation.

## Source Coverage Matrix

| Inventory surface | Primary Phase(s) | Shared/final handling |
| --- | --- | --- |
| crate roots, Cargo, `.cargo`, CI | 10 | Final validation/readiness |
| fixture binaries | 10 | consumer absence must be proven or left unverified |
| `game/data/**`, live RON, config | 1 | domain Phases consume confirmed published catalogs |
| run/map/state/persistence | 2 | 10 projects commands/snapshots after producer audit |
| roster/inventory/economy | 3 | 2/4/5 lifecycle seams; 10 DTO projection |
| event/facility/VN sessions | 4 | 2 node lifecycle; 10 projection |
| combat preview/setup/waves | 5 | 1 data, 2 run, 3 roster, 6 battle handoff |
| battle clock/event/result | 6 | 7~9 behavior and 10 transport derive from it |
| spatial/movement/tactical | 7 | 6 lifecycle, 9 target range, 10 checkpoint projection |
| effects/stats/damage/death | 8 | 3 persistent stats, 6 lifecycle, 9 delivery outcomes |
| targeting/skill delivery | 9 | 1 authored data, 7 spatial, 8 effects, 10 range DTO |
| behavior/capability/snapshot/server | 10 | Final competing-authority and duplicate-candidate audit |
| durable canonical docs | corresponding Phase | Final alignment classification |
| Goal/decision/archive/tmp history | supporting only when current evidence needs context | never current authority |
| IDE/Cate metadata | explicit runtime-audit out-of-scope | tooling follow-up only |
| Unity | explicit out-of-scope/unverified | no verified-consumer claim |

## Known Shared Files Requiring Duplicate-Root-Cause Control

| Shared file/surface | Phase seam | Final consolidation rule |
| --- | --- | --- |
| `world.rs` | 2~10 dispatch | one candidate per domain root cause; routing duplication may be Phase 10 only |
| `world/state.rs` | 2, 6, 7, 10 | distinguish authoritative state from DTO projection helpers |
| `world/combat.rs` | 5~10 | handoff, lifecycle, tactical, targeting and result candidates remain separate unless one mutation root cause is proven |
| `behavior.rs` | all producer domains + 10 | domain DTO evidence supports producer Phase; projection ownership candidate lives in Phase 10 |
| `battle/core/mod.rs` and `types.rs` | 6~9 | container proximity is not consolidation evidence |
| `data/run_policy_data.rs` | 1, 2, 5~9 | Phase 1 owns loading/validation; consuming policy calculation belongs to runtime Phase |
| `tests/ron_loading.rs` | 1 plus domain Phases | each assertion is cited only for the contract it directly protects |
| `world/tests/**` | 2~10 | named tests may support multiple seams but do not duplicate Candidate IDs |

## Activation Gate

Phase 0 can close after its completion audit proves coverage and no production mutation. The proposal then pauses
at `deferred_by_user`. No proposed child directory is created until the user approves the Phase count/names/scope
or explicitly delegates naming inside the frozen Master scope.
