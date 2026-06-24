# Buff, Status, Stat Policy Implementation

## Objective

Make hard CC and movement-speed behavior derive from authoritative active buff/effect/stat state, with precise timing and explicit events.

## Policies Covered

- `hard CC action restriction source`
- `hard CC replacement event`
- `clear active buffs on death`
- `non-periodic control status max stacks`
- `move speed stat source`
- `move speed runtime update timing`

## Plan

1. Read buff/effect runtime state, action gate integration, death handling, stat aggregation, movement timing, and tests.
2. Ensure hard CC action restriction is derived from active buff/effect state only.
3. Emit explicit existing-CC end events when hard CC is replaced.
4. Clear active buffs immediately on death.
5. Validate `Stun`, `Freeze`, and `Silence` or equivalent non-periodic control statuses as `max_stacks == 1`.
6. Treat `MoveSpeedUnitsPerMs` as the official movement-speed stat.
7. Apply move-speed stat changes on the next movement tick for clear animation/timing behavior.
8. Update focused tests for hard CC timing, replacement, death cleanup, max stack validation, and speed changes.

## Completion Conditions

- No duplicated hard CC action-lock state exists outside active buff/effect state.
- Replacement emits explicit end/start timeline or domain events as appropriate.
- Death clears active buffs immediately and consistently.
- Control status stack invariants fail validation.
- Movement speed stat changes affect actual movement at the confirmed timing.

## Completion Report

Completed on 2026-06-22 by implementation/verification.

### Implemented

- Added required `BuffExpireReason` on `TimelineEvent::BuffExpired`.
- Changed hard CC replacement to emit `BuffExpired(reason=Replaced)` before the replacing `BuffApplied`.
- Removed hard CC duration writes into `ActionLocks` and `next_action_time`; active `Stun`/`Freeze` buffs now drive movement, basic attack, cast, and resonance gates.
- Added death cleanup for active buffs where the dead unit is either target or caster, with `TargetDied`/`CasterDied` reasons.
- Enforced `max_stacks == 1` for `Stun`, `Freeze`, and `Silence` metadata.
- Made final `UnitStats.move_speed_units_per_ms` the source for spawned `UnitBody.move_speed`.
- Made runtime `ApplyModifier(MoveSpeedUnitsPerMs)` update `UnitBody.move_speed` for the next movement tick.

### Removed Legacy

- Implicit hard CC replacement in timeline validation.
- Stackable `Silence` test fixture.
- Hard CC action-lock duplication.

### Fixed Contracts

- `BuffExpired` carries an explicit lifecycle reason.
- Unity/replay can observe hard CC replacement and death cleanup instead of inferring hidden lifecycle changes.
- Active hard CC buff expiry is the source of truth for hard CC action restriction.
- Movement speed stat changes are gameplay-effective, not display-only.

### Validation

- `cargo test hard_cc_replacement_records_explicit_buff_expired_reason --lib`
- `cargo test unit_death_clears_target_and_caster_active_buffs_with_reasons --lib`
- `cargo test move_speed_modifier_updates_runtime_body_speed_for_next_movement_tick --lib`
- `cargo test control_status_metadata_rejects_stackable_hard_cc --lib`
- `cargo test hard_cc --lib`
- `cargo test buff --lib`
- `cargo test move_speed --lib`
- `cargo test death_clears --lib`
- `cargo test --test ron_loading`
- `cargo check -p game_server`
- `cargo test --lib -- --test-threads=1`
- `git diff --check`

### Remaining Risk

- `BuffExpired.reason` is a Unity-facing timeline DTO addition. This is covered by the confirmed policy, but downstream client code must read the new reason field when consuming fresh timelines.
