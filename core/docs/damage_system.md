# Damage System Design

## Goals

- Route every damage event through one damage pipeline.
- Support TFT-like damage categories: physical, magic, and true damage.
- Require authored damage data to specify its damage type explicitly.
- Preserve timeline/replay debuggability by recording raw and final damage.

## Damage Model

Damage has two separate concepts:

- `DamageSource`: where the damage came from, such as basic attack, ability, buff tick, or environment.
- `DamageType`: how the damage is mitigated, such as physical, magic, or true.

The long-term contract is:

- Basic attacks default to physical damage.
- Skill `Damage` effects must specify `damage_type`.
- Buff tick damage is typed by the buff definition.
- Trigger `BonusDamage` must specify `damage_type`.
- True damage ignores mitigation.

## Defensive Stats

The current `Defense` stat remains the physical resistance stat for backward compatibility. New code should treat it as armor. A new `MagicResist` stat handles magic damage mitigation. Both resistance stats are signed integers so debuffs and penetration can push them below zero.

Future cleanup can rename serialized `defense` to `armor`, but that should be a separate data migration. Until then:

- `UnitStats.defense: i32`: physical resistance / armor
- `UnitStats.magic_resist: i32`: magic resistance

## Mitigation Formula

The formula follows the common TFT/LoL resistance curve:

- Positive resistance: `final = raw * 100 / (100 + resistance)`
- Negative resistance: `final = raw * (2 - 100 / (100 - resistance))`
- True damage: `final = raw`

Rounding is integer floor after fixed-point calculation.

`DamageModifiers` is the source-specific second calculation layer:

- Flat armor penetration adjusts physical resistance before mitigation.
- Flat magic-resist penetration adjusts magic resistance before mitigation.
- Percentage armor penetration adjusts physical resistance before flat penetration.
- Percentage magic-resist penetration adjusts magic resistance before flat penetration.
- Critical hits multiply raw damage before mitigation.
- Damage amplification applies after mitigation.
- Type-specific amplification stacks with global amplification.
- Damage reduction applies after amplification.
- Type-specific reduction stacks with global reduction.

Minimum damage is a policy on the request and is enforced after critical hits, mitigation, and post-mitigation modifiers:

- Basic attacks require at least 1 final damage when raw damage is positive.
- Ability, buff, and environment damage can resolve to 0.

## Pipeline

All damage should flow through:

1. Build a `DamageRequest`.
2. Build a `DamageContext` from the source and target state.
3. Split raw damage into typed components.
4. Roll deterministic critical hit chance from the battle seed.
5. Apply critical raw-damage multiplier when the roll succeeds.
6. Apply resistance modifiers for each component.
7. Mitigate each component by its damage type.
8. Apply post-mitigation amplification/reduction.
9. Enforce request minimum damage.
10. Apply the resulting HP loss.
11. Record `HpChanged` with damage metadata.
12. Process triggered commands and deaths.

This replaces direct negative-heal damage for skills and buff ticks.

Mixed damage is represented as typed components in a single `DamageResult`.
For example, a physical basic attack can include a magic `BonusDamage` component.
Each component is mitigated by its own resistance before the final HP loss is summed.

## Timeline Contract

`HpChanged` keeps the existing fields and adds optional damage metadata:

- `damage_type`
- `damage_source`
- `raw_damage`
- `final_damage`
- `damage_breakdown`
- `critical`

The fields are optional so healing and legacy command-style HP changes remain representable.
`damage_type` is the primary/request type for quick UI use; `damage_breakdown` is the authoritative per-type component list when damage is mixed.
Damage `HpChanged` events emitted by the battle pipeline always include `damage_type`, `damage_source`, `raw_damage`, `final_damage`, `damage_breakdown`, and `critical`, so clients should not infer damage type from skill, source, or visuals.

## Data Compatibility

Damage RON must be explicit:

```ron
Damage(amount: 50, damage_type: Physical)
Damage(amount: 50, damage_type: Magic)
Damage(amount: 50, damage_type: True)
```

Trigger bonus damage is also explicit:

```ron
BonusDamage(flat: 12, percent: 0, damage_type: Magic)
```

Damage modifiers are optional and can be attached to skill steps or attack triggers:

```ron
ModifyDamage(modifiers:(
    armor_penetration_flat: 20,
    armor_penetration_percent: 30,
    magic_resist_penetration_flat: 0,
    damage_amp_percent: 10,
    physical_damage_amp_percent: 20,
    damage_reduction_percent: 0,
    crit_chance_percent: 25,
    crit_damage_percent: 50,
))
Damage(amount: 50, damage_type: Physical)
```

For skill steps, `ModifyDamage` applies to damage effects in the same step. For OnAttack item and artifact triggers, `ModifyDamage` applies as outgoing damage modification. For OnHit triggers, it applies as incoming damage modification, so defensive items can express received-damage reduction. In both cases it applies to the current damage request, including typed bonus damage components.

## Implementation Notes

- `DamageSource` and `DamageType` are intentionally separate.
- `BattleCommand::ApplyDamage` is the only command that should represent harmful HP changes.
- `BattleCommand::ApplyHeal` should only represent healing.
- Trigger `BonusDamage` adds a typed component to the active damage request.
- Trigger `ModifyDamage` and skill-step `ModifyDamage` feed the `DamageModifiers` second calculation layer.
- Future extensions can add percentage penetration, armor shred, magic-resist shred, lifesteal, and shields without adding alternate damage paths.
