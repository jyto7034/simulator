# Core Damage Snapshot Contract

## Objective

Refactor battle damage calculation so damage source context is captured once at the cause point and reused consistently, while target defensive context is evaluated at the actual apply/impact point.

This goal started design-first. Implementation is allowed only inside the confirmed policy bounds recorded below.

## Source of Truth Order

1. Actual battle runtime code:
   - `src/game/battle/core/basic_attack.rs`
   - `src/game/battle/core/commands.rs`
   - `src/game/battle/core/sim.rs`
   - `src/game/battle/core/skill_runtime/projectile.rs`
   - `src/game/battle/damage.rs`
   - `src/game/battle/timeline.rs`
2. Live RON/data that drives skills, basic attacks, buffs, items, and fragments:
   - `../game_resources/data/skills/base.ron`
   - `../game_resources/data/abnormalities/base.ron`
   - `../game_resources/data/equipments/base.ron`
   - `../game_resources/data/skill_fragments/base.ron`
3. Unity-facing timeline/snapshot/server contracts.
4. Confirmed policy docs and this goal.

Do not preserve legacy behavior through compatibility layers, fallback paths, or dual schemas unless explicitly recorded here with a removal condition and approved.

## Initial Runtime Findings

- `AttackResolve` scheduling now drains same-timestamp resolve events before winner finalization, but `handle_basic_attack_resolve_event()` still resolves each event against current live attacker/target state.
- Instant basic attack creates `BasicAttackDamageSnapshot`, but that snapshot is built at `resolve_basic_attack()` time, not at `AttackStarted` or `AttackResolve` scheduling time.
- Basic attack projectile stores partial launch context:
  - `attacker_owner_at_launch`
  - `air_capable_at_launch`
  - attacker/target ids, positions, guidance, damage type
- Basic attack projectile impact still reads target defense/resist/current HP live at impact time.
- If projectile attacker is dead at impact, attacker attack can be read from `graveyard`, but on-attack/on-hit trigger behavior is reduced.
- Skill damage is built as `BattleCommand::ApplyDamage`; `process_commands()` reads source/target state live at command application time.
- Skill projectile runtime stores caster owner and delivery metadata but does not store a full damage source snapshot.
- `TimelineEvent::HpChanged` records damage result metadata, but no event currently records reusable source snapshot data.

## Long-Term Policy Direction

Use a split damage context model:

- `DamageSourceSnapshot`: captured at the point where the damaging intent becomes committed.
- `DamageTargetContext`: evaluated at the point where damage is applied.

Recommended source snapshot commit points:

| Damage path | Source snapshot point | Target context point |
| --- | --- | --- |
| Instant basic attack | `AttackStarted` or attack resolve scheduling, before same-timestamp deaths can erase attacker context | `AttackResolve` apply time |
| Basic attack projectile | projectile launch | projectile impact |
| Instant skill step | `SkillStepExecuted` / step command creation | `ApplyDamage` command processing |
| Skill projectile | skill projectile launch | projectile impact |
| Buff tick / DOT | each tick event | each tick apply time |
| Environment damage | event creation point if it has a source; otherwise explicit environment source | apply time |

The initial implementation should prefer source snapshot + live target context rather than fully snapshotting both sides. This preserves reactive defenses/shields that happen before impact while removing source-side drift caused by death, buff expiry, stat changes, or ordering inside the same timestamp.

## Proposed Architecture

1. Add explicit source snapshot types near `damage.rs` or battle core types:
   - `DamageSourceSnapshot`
   - source unit id
   - source side
   - attack value or base power source
   - damage source kind
   - damage type
   - base damage
   - modifiers owned by the source event/skill step
   - crit roll seed inputs or precomputed crit roll
   - on-attack sourced effects that should be frozen with the source
2. Use an apply-time target context builder:
   - reads target side, defense, magic resist, incoming modifiers, current HP, max HP
   - reads target-side on-hit effects at apply time unless later policy decides these must be snapshot too
3. Change `BattleCommand::ApplyDamage` to carry a source snapshot instead of re-reading source attack from `units`/`graveyard`.
4. Change instant basic attack to create the source snapshot before damage can be invalidated by same-timestamp death.
5. Change basic attack projectile record/runtime to carry source snapshot from launch to impact.
6. Change skill command generation/projectile launch to carry source snapshot from step execution/launch.
7. Keep `HpChanged` result metadata as result output. Add timeline snapshot fields only if required for Unity/debug/replay; this is policy-sensitive and must be confirmed before changing serialized event shape.
8. Update validators/tests to pin behavior rather than private helper layout:
   - same timestamp mutual attacks both use committed source snapshot
   - projectile from dead attacker uses launch source snapshot
   - source attack buff applied after launch does not affect already-launched projectile
   - target defense buff before impact does affect damage
   - skill projectile uses launch/step source snapshot

## Policy-Sensitive Decisions

Implementation must stop and ask before changing:

- serialized `TimelineEvent` shape
- Unity-facing DTO shape
- live RON schema
- whether target-side on-hit effects are apply-time live or snapshot-time frozen
- whether source on-attack triggers are snapshot as effects only or also execute even after source death
- whether crit roll is frozen at source snapshot time or calculated at apply time from stored seed inputs
- whether existing debug event log exports need migration or regeneration

## Plan

1. [x] Re-read all current damage paths and classify where source and target context are created.
2. [x] Define the minimal source snapshot contract that covers basic attacks, skill damage, projectile damage, buff ticks, and environment damage.
3. [x] Decide exact source snapshot commit points per path and record any policy-sensitive ambiguity.
4. [x] Implement one path at a time:
   - [x] instant basic attack
   - [x] basic attack projectile
   - [x] skill `ApplyDamage`
   - [x] skill projectile
   - [x] buff tick commands
5. [x] After each path, run focused tests for that path.
6. [x] Add timeline/validator tests only for user-visible behavior and debug contract stability.
7. [x] Run broad validation at the end.
8. [x] Update `EXPERIMENTS.md` and `EXPERIMENT_NOTES.md` continuously.

## Completion Conditions

- Source-side damage data is no longer recomputed from live attacker state at apply/impact time for basic attacks, projectiles, and skill damage.
- Dead attackers with already-committed damage use the committed source snapshot rather than ad hoc live/graveyard fallback.
- Target defensive state remains apply/impact-time live unless a later confirmed policy says otherwise.
- Tests cover same-timestamp attacks, projectile-after-death, source stat drift, target defense timing, and skill projectile source context.
- No hidden compatibility path keeps the old live-source recomputation behavior.
- Any required DTO/timeline schema policy decision is either confirmed and implemented, or recorded as `사용자와 정책 논의 필요` with the implementation goal completed as a policy-decision report.
- Final report lists changed contracts, removed legacy behavior, updated tests, remaining risks, and validation commands.

## Confirmed Policies

Confirmed by user on 2026-06-22:

- Crit roll is frozen from source snapshot inputs.
- If the source dies after damage is committed, committed source-side damage modifiers/bonus damage remain active, but source-side command/ability triggers do not execute after source death.
- Skill multi-hit creates target-specific source snapshots.
- Delayed skill projectile damage uses the projectile launch source snapshot.
- Internal `BattleEvent::AttackResolve` carries the source snapshot directly. Serialized `TimelineEvent`, Unity DTO, and live RON schema are not changed by this goal.

## Current Status

Status: completed.

The policy-decision block in `EXPERIMENT_NOTES.md` has been resolved by the confirmed policies above. Runtime implementation stayed within those bounds: source snapshots are internal runtime state, serialized `TimelineEvent`, Unity DTO, and live RON schema were not changed.

## Implementation Summary

- Added `DamageSourceSnapshot` and `DamageBonusSnapshot` as the internal source-side damage contract.
- Changed `BattleCommand::ApplyDamage` to carry `DamageSourceSnapshot`.
- Changed internal `BattleEvent::AttackResolve` and `SkillProjectileImpact` to carry source snapshots.
- Stored source snapshots in basic attack projectile records and skill projectile runtimes.
- Preserved target defense/HP/on-hit evaluation at apply or impact time.
- Preserved committed source-side damage modifiers/bonus damage after source death while suppressing source-side command/ability triggers when the source is no longer live.
- Kept serialized timeline, Unity-facing DTOs, and live RON schema stable.

## Verification

- `cargo fmt`
- `cargo check --lib`
- `cargo test --lib game::battle::core::commands::tests -- --test-threads=1`
- `cargo test --lib -- --test-threads=1`

All final validation commands passed. The only remaining warning is the pre-existing workspace manifest warning from `auth_server/Cargo.toml` (`unused manifest key: env`).
