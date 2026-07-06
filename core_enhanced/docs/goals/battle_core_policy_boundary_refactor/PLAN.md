# Battle Core Policy Boundary Refactor

## Objective

Resolve P-008 by decomposing the largest battle core files along policy boundaries, not by arbitrary file size.

`src/game/battle/core/mod.rs` and `src/game/battle/core/commands.rs` contain many abstraction levels: runtime state, command processing, event recording, projectile handling, damage/death application, movement interaction, and test fixture helpers. The goal is to make battle runtime flow easier to follow while preserving behavior.

## Source Of Truth Order

1. Runtime battle code and focused battle tests.
2. Battle transport DTO/event contracts.
3. Live RON/data that drives battle setup, skills, attacks, and waves.
4. Canonical battle/rulebook docs.
5. Audit notes.

## Scope

- Identify cohesive battle policy boundaries.
- Extract code by responsibility, for example:
  - event recording;
  - damage and death application;
  - projectile advancement and hit/miss handling;
  - live command handling;
  - test fixture/setup utilities.
- Keep public behavior and event log shape stable unless an actual bug is found and approved.

## Non-Goals

- Do not rewrite battle simulation.
- Do not change attack targeting, damage balance, projectile semantics, or movement policy.
- Do not change Unity-facing battle event names or DTO shape.
- Do not split files mechanically without improving ownership boundaries.

## Plan

1. Re-read:
   - `src/game/battle/core/mod.rs`
   - `src/game/battle/core/commands.rs`
   - adjacent battle modules
   - tests covering projectiles, damage, death, movement, deployment, skills, and event logs.
2. Produce a boundary map in `EXPERIMENT_NOTES.md` before editing:
   - current responsibilities;
   - proposed target modules;
   - dependencies that make extraction risky.
3. Pick one boundary at a time and refactor in small, testable steps.
4. Prefer moving cohesive private helpers over changing data ownership first.
5. Run focused tests after each extraction.
6. Stop and report if a clean extraction requires a gameplay or DTO policy change.

## Completion Conditions

- Battle core code is split by meaningful policy ownership, not arbitrary line count.
- Event log output, damage/death behavior, projectile behavior, deployment, and skill behavior remain stable.
- Tests cover the behavior most likely to regress during each extraction.
- New module names communicate responsibility clearly.
- No compatibility layer, duplicate battle path, or temporary forked runtime is introduced.

## Implementation Summary

- Added `src/game/battle/core/projectile_math.rs` for shared projectile flight-time math used by basic attack and skill projectile runtime.
- Added `src/game/battle/core/basic_attack_projectile.rs` for basic attack projectile launch, reevaluation, locked-target hit handling, and miss recording.
- Added `src/game/battle/core/damage_runtime.rs` for damage-source snapshots, crit roll materialization, HP delta recording, and damage result application.
- Added `src/game/battle/core/death_runtime.rs` for death finalization, death-trigger commands, buff cleanup on death, graveyard capture, and movement invalidation after death.
- Moved `ActiveMovementSegment` into `src/game/battle/core/movement/types.rs`, because it is movement runtime state consumed by movement engine and presentation sampling.
- Kept `src/game/battle/core/commands.rs` as the `BattleCommand` dispatch and basic attack release shell; no Unity-facing event or DTO shape changed.
- Did not split `src/game/battle/core/mod.rs` test fixtures in this goal because the plan forbids mechanical file splitting; the only `mod.rs` extraction performed was a movement-owned runtime type.

## Validation Commands

Start with focused filters and broaden:

- `cargo test -p game_core projectile --lib -- --test-threads=1`
- `cargo test -p game_core damage --lib -- --test-threads=1`
- `cargo test -p game_core death --lib -- --test-threads=1`
- `cargo test -p game_core movement --lib -- --test-threads=1`
- `cargo test -p game_core skill --lib -- --test-threads=1`
- `cargo test -p game_core game::battle --lib -- --test-threads=1`
- `cargo test -p game_core game::world::tests::combat --lib -- --test-threads=1`
- `cargo check -p game_core`
- `cargo fmt --check`

Adjust filters after reading actual test names.

## Stop Conditions

Complete the goal and report questions if:

- Extraction requires changing battle event DTO shape.
- Projectile, movement, targeting, or damage policy ambiguity is discovered.
- Runtime behavior changes are necessary rather than pure boundary cleanup.
- Existing tests are too weak to verify a planned extraction safely.
