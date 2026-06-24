# Battle Attack Movement Time Consistency Notes

## Policy Decisions Already Made

- Do not reduce the movement tick as the primary fix.
- Keep DefenseRoute player attack and skill range tile-based.
- Unity should not need to compensate for core attack events that are timestamped earlier than their sampled range-valid position.
- `AttackStart -> AttackResolve` already represents windup. Do not add `AttackWindUp` as part of this goal.

## Current Interpretation

The bug is best described as:

```text
attack eligibility sampled from tick-end position
AttackStart timestamped at tick-start time
Unity presents movement segment over tick-start..tick-end
```

This can make a valid attack look early.

The long-term fix should make the attack decision and the `AttackStart.time_ms` refer to the same battle-time position sample.

## Implementation Preference

Prefer one of these long-term shapes:

1. Evaluate attacks for `time_ms` before applying movement outputs for `time_ms..time_ms + dt_ms`.
2. Add explicit `*_at(time_ms)` target/range helpers and ensure every attack decision uses sampled positions matching the event time.

Avoid:

- special-casing only projectile attacks
- special-casing only Apocalypse Bird
- lowering tick duration to hide mismatch
- delaying Unity VFX locally
- allowing DefenseRoute player attacks to fall back to `range_units`

## Questions To Revisit During Implementation

- Does skill auto-cast target selection have the same tick-end/tick-start mismatch?
- Should attack scheduling from `pending_basic_attack` use the next movement tick boundary or a newly computed exact range-entry time?
- Are active movement segments currently sufficient to sample previous tick-start positions after outputs are applied, or do we need an explicit pre-movement body snapshot?
- Should battle record export include enough equipment/source metadata to make future attack traces easier?

## Follow-Up Candidates Outside This Goal

- Add optional debug tooling that summarizes every `HpChanged` as:
  - source unit
  - employee/profile identity
  - equipped weapon id
  - range pattern source
  - sampled attacker/target tile at `AttackStart`
- Consider battle record enrichment for attack source profile IDs if debugging remains painful.
- Review Unity actor/roster visual binding separately if the wrong visible unit appears to own a damage label.
