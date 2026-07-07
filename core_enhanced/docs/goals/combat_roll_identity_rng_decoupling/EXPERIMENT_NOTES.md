# Combat Roll Identity RNG Decoupling Notes

## Initial Finding

The current source snapshot contract freezes critical roll data, which is good. The remaining issue is the identity used to create the frozen roll:

```text
DamageSourceSnapshot.crit_roll_event_log_seq
damage_roll_percent_with_event_log_seq(...)
```

This means event log sequence is part of gameplay RNG. A harmless log event can change critical outcomes after it.

## Chosen Policy Direction

Use explicit combat roll identity.

```text
battle seed + CombatRollIdentity -> roll percent
```

Do not use event log sequence or RNG draw order.

## Important Design Note

Avoid using `committed_at_ms` as the primary identity key.

Time is useful context, but multiple hits can happen at the same battle time from the same source and target. The identity should be anchored on a source instance:

- basic attack instance;
- skill cast / skill step instance;
- projectile / delivery instance;
- buff tick instance;
- environment pulse instance.

Then use `hit_index` for multi-target or repeated-hit cases.

## Current Open Questions For Implementation

- Resolved: damage source instances are now allocated through `BattleCore.damage_source_seq`. This keeps the identity in the battle domain and independent from event log sequence.
- Resolved: `CombatRollIdentity` lives in `damage.rs` with the damage snapshot contract because it is currently consumed by damage critical roll materialization.
- Resolved: delayed/multi-target source templates use `materialize_damage_source_snapshot_for_target_hit(...)` to bind target id and hit index.
- Follow-up: proc rolls in `sim.rs` are not migrated in this goal because the documented scope is damage critical rolls and event-log coupling.

## Policy Boundaries

- Do not change crit chance/damage math.
- Do not change serialized battle events.
- Do not change live RON.
- Do not change Unity-facing DTOs.
- Do not preserve `event_log_seq` under a renamed field.

## Follow-Up Candidates

- Canonical documentation after implementation:
  - event log is observation;
  - combat roll identity is gameplay;
  - logging changes must not change combat outcomes.
- Consider migrating non-damage proc rolls to the same identity pattern if future audits find call-order sensitivity there.
