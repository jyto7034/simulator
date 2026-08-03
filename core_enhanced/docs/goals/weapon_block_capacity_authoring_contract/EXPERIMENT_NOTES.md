# Weapon Block Capacity Authoring Contract Notes

Status: complete

## Starting Evidence

- Rulebook described weapon/armor block adjustment, but reaudit found no equipment block modifier in
  the current schema/stat pipeline.
- User rejected character-class authority and approved explicit weapon-only authoring.
- G06 fixed the current archetype set to Sword, Spear, Shield, Bow, Gun and Staff.

## Resume Point

- Child complete. Resume from Master G05.

## Final Authority Path

```text
equipments/base.ron WeaponCombatProfile.block_capacity
  -> loaded EquipmentMetadata
  -> BattleUnitDraft equipped weapon lookup
  -> UnitCombatProfile::apply_weapon_profile exact copy
  -> RuntimeUnit.block_capacity / block engagement
```

- Unarmed authority is `UnitCombatProfile::employee_default().block_capacity == 0`.
- Armor and `CombatAptitude` are outside the path.
- Existing roster/inventory snapshots serialize the same `WeaponCombatProfile` including the field;
  game_server remains a pass-through.

## Live Authoring Review

- 29 weapon profiles, 29 explicit `block_capacity` fields.
- Shield 3; Sword/Spear 1; Bow/Gun/Staff 0.
- All six current archetypes are represented; no exception required a balance decision.

## Validation Summary

- Missing required field regression: 1 passed.
- Six-archetype/unarmed/armor/replacement projection regression: 1 passed.
- Equipment module: 27 passed.
- `ron_loading`: 21 passed.
- Core library: 741 passed.
- game_server: 30 passed.
- Rust formatting and diff checks passed.
