# Basic Attack Lifecycle Refactor Plan

## Objective

Collect basic attack scheduling, target selection handoff, windup/impact, and projectile lifecycle into a cohesive module without changing timeline/event contracts.

## Current Scope

In scope:

- Re-read runtime `sim.rs`, `commands.rs`, `targeting.rs`, timeline/event tests.
- Preserve instant and projectile basic attack behavior.
- Preserve Unity-facing timeline/event field names.
- Reduce scattered basic attack lifecycle helper ownership.

Out of scope:

- Targeting policy changes.
- Damage timing changes.
- Skill projectile runtime unification.
- Weapon archetype/balance changes.
- Timeline DTO rename.

## Plan

1. Document current basic attack event flow in `EXPERIMENT_NOTES.md`.
2. Identify helpers that can move without behavior change.
3. Add a basic attack lifecycle module under `battle/core`.
4. Move scheduling/launch helper entry points in small steps.
5. Run focused projectile, targeting, and battle core tests after each step.

## Completion Conditions

- Basic attack lifecycle has a clear module/entrypoint.
- `sim.rs` and `commands.rs` own fewer basic-attack-specific details.
- Instant/projectile behavior is preserved by tests.
- Unity-facing timeline contract is unchanged.
- Verification commands pass.
- 사용자와 의논하여 정해야 할 정책이 있을 경우 goal을 종료한다.

## Completion Evidence

- `src/game/battle/core/basic_attack.rs` owns basic attack target persistence/selection, pending auto-attack scan, AttackStart lifecycle, and AttackResolve lifecycle.
- `src/game/battle/core/sim.rs` delegates AttackStart/AttackResolve handling to the lifecycle module.
- Projectile launch/advance/damage details remain in `commands.rs`; no projectile timing policy was changed.
- Unity-facing timeline event names/fields are unchanged.
- Verification passed:
  - `cargo check -p game_core`
  - `cargo test -p game_core battle::core::tests -- --nocapture`
  - `cargo test -p game_core battle::core::commands::tests -- --nocapture`
  - `cargo test -p game_core --test skill_refactor_validation`
  - `cargo fmt`
  - `cargo check -p game_core`

## Stop Conditions

- Targeting/timing/timeline policy needs a new decision.
- Projectile and instant behavior disagree in a way that requires design choice.
- Unity-facing timeline contract must change.
- 사용자와 의논하여 정해야 할 정책이 있을 경우 goal을 종료한다.
