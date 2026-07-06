# Experiment Notes

## Initial Audit Basis

- `src/game/battle/core/mod.rs` and `src/game/battle/core/commands.rs` are large and mix runtime flow with detailed command/event/projectile/death handling.
- Size alone is not the issue. The concern is mixed abstraction levels and unclear policy ownership.

## Candidate Boundaries To Verify

- Event recording helpers.
- Damage and HP delta application.
- Death finalization and death-trigger commands.
- Basic attack projectile launch/advance/hit/miss.
- Live command handling.
- Test fixture/setup helpers.

## Boundary Map After Initial Code Read

### Current Responsibilities

- `src/game/battle/core/mod.rs`
  - Owns `BattleCore` state, scenario runtime state, public debug/runtime queries, resonance mutation, hard-CC checks, pending autocast scheduling, and a large embedded unit-test section.
  - Already delegates meaningful behavior to existing submodules: `basic_attack`, `movement`, `skill_runtime`, `targeting`, `triggers`, `sim`, and `build`.
- `src/game/battle/core/commands.rs`
  - Mixes several policy layers:
    - trigger command materialization and triggered ability dispatch;
    - damage-source snapshot creation and crit roll determinism;
    - HP delta recording and death finalization;
    - basic attack instant/projectile damage application;
    - basic attack projectile launch/advance/hit/miss runtime;
    - generic `BattleCommand` execution;
    - a large embedded test fixture/test section.
- Existing adjacent modules already provide useful boundaries:
  - `skill_runtime/projectile.rs` owns skill projectile runtime, but imports the generic projectile flight helper from `commands.rs`.
  - `movement/*` owns continuous movement and movement event lifecycle.
  - `basic_attack.rs` owns target selection, attack-start scheduling, and release timing, but not projectile travel/hit handling.

### Proposed Target Modules

- `core/projectile_math.rs`
  - Shared projectile travel-time helpers used by both basic attack and skill projectile code.
  - This avoids leaving skill projectile math dependent on the broad `commands` module.
- `core/basic_attack_projectile.rs`
  - Owns `ProjectileLaunch`, basic attack projectile launch, reevaluation, miss recording, and locked-target hit handling.
  - This mirrors the existing `skill_runtime/projectile.rs` boundary and removes projectile runtime detail from `commands.rs`.
- Follow-up candidates after the first extraction:
  - `core/damage_runtime.rs` for damage-source snapshots, damage result application, HP delta recording, and crit roll determinism.
  - `core/death_runtime.rs` for death finalization, death-trigger command generation, and buff cleanup on death.
  - `core/command_runtime.rs` for generic `BattleCommand` execution if damage/death extraction leaves a clean dispatch shell.

### Extraction Risks

- Basic attack projectile hit handling depends on damage helpers and trigger helpers that currently remain in `commands.rs`.
  - Mitigation: move projectile code first but keep those helper calls on `BattleCore`; do not change damage behavior.
- `ProjectileLaunch` is constructed by `resolve_committed_basic_attack`, so moving it requires a module import but not a DTO/event change.
- `projectile_flight_ms_for_delivery` is used by `skill_runtime/projectile.rs`; moving it to `projectile_math.rs` should be behavior-neutral and focused.
- The embedded tests in `commands.rs` currently reach private helpers such as `projectile_flight_ms` and `ProjectileLaunch`.
  - Mitigation: either import from the new modules in the same test module or leave tests in place while moving production ownership.

## Implemented Boundaries

- `core/projectile_math.rs`
  - Owns shared projectile flight-time calculation.
  - `skill_runtime/projectile.rs` now depends on this narrow math module instead of depending on `commands.rs`.
- `core/basic_attack_projectile.rs`
  - Owns basic attack projectile runtime:
    - `ProjectileLaunch`;
    - projectile id allocation and launch event recording;
    - reevaluation scheduling;
    - locked-target moving body sweep;
    - projectile hit/miss event recording;
    - impact-time damage application handoff.
- `core/damage_runtime.rs`
  - Owns battle-runtime damage application:
    - `DamageSourceSnapshot` creation and materialization;
    - deterministic crit roll seeding;
    - basic attack damage snapshot calculation;
    - `HpChanged` event recording for damage and healing/delta effects;
    - death handoff when HP reaches 0.
- `core/death_runtime.rs`
  - Owns death policy runtime:
    - death-trigger command collection;
    - movement interruption and `MovementStopped(Died)`;
    - active buff expiry due to target/caster death;
    - graveyard snapshot capture;
    - `UnitDied` event recording;
    - movement retick scheduling after death.
- `core/movement/types.rs`
  - Now owns `ActiveMovementSegment`, the runtime/presentation sampling state for immutable movement segments.
  - This removes a movement-specific type from `core/mod.rs` without changing movement event semantics.

## Final Responsibility Shape

- `commands.rs` remains the broad command dispatch and basic attack release shell.
  - It still owns trigger command materialization because both damage/death/projectile paths depend on it and moving it together would create a larger dispatch refactor than this goal needs.
  - It no longer owns projectile travel, projectile hit/miss runtime, damage snapshot materialization, HP delta recording, or death finalization.
- `mod.rs` remains large, mostly because it contains `BattleCore` state, public runtime/debug queries, resonance/autocast helpers, and a large embedded test section.
  - A movement-owned runtime type was moved out.
  - No `mod.rs` test fixture split was performed because the plan forbids mechanical splitting, and no clean behavior boundary in those tests was required to complete this goal safely.

## Policy Notes

- Preserve event log shape and timeline semantics unless a concrete bug is found and approved.
- Do not move code just to reduce line count.
- Each extraction should have a focused behavior test before broad checks.
