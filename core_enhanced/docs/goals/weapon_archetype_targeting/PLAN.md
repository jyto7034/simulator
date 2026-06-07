# Weapon Archetype Targeting Plan

## Objective

Finish the weapon-profile source of truth so equipped weapons determine current combat role, basic attack profile, damage type, target priority, and Unity-facing display.

## Completion Conditions

- Weapon equipment always has a valid `weapon_profile`; armor/accessory never has one.
- Live equipment RON and test fixtures no longer bypass weapon profile validation.
- Employee effective combat profile applies the equipped weapon profile exactly once.
- Basic attack damage type, range role, target profile, air capability, range pattern, interval, windup, and delivery come from the weapon profile.
- Targeting profiles are implemented from actual runtime data:
  - `DefaultForward`
  - `AirFirst`
  - `LowDefenseFirst`
  - `LowMagicResistFirst`
  - `SplashClusterFirst`
- Melee units prefer blocked enemies before weapon targeting profile fallback.
- Unity-facing snapshots/contracts expose the current weapon combat profile.
- Full live RON loading passes unless a new unrelated blocker is found and recorded.
- If code review exposes a policy that must be decided with the user, stop the goal.

## Current Findings

- `WeaponCombatProfile`, `WeaponRangeRole`, `WeaponArchetype`, and `TargetingProfile` already exist in `equipment_data.rs`.
- `EquipmentDatabase::validate_weapon_profiles` already rejects weapon equipment without `weapon_profile`.
- `BattleUnitDraft::combat_profile_from_abnormality` already applies an equipped weapon profile into `UnitCombatProfile`.
- Live/test weapon metadata has been migrated to explicit `weapon_profile`; validation stays strict.
- Player basic attack target selection now applies weapon `TargetingProfile` after melee blocked-target priority.
- Unity-facing snapshots expose `effective_weapon_profile` and equipment `weapon_profile`.

## Initial Plan

1. Read equipment live RON, employee profile calculation, basic attack command path, and targeting path.
2. Add shared default/profile constructors only if they represent real policy, not compatibility fallback.
3. Update test helpers to create valid weapon metadata by default.
4. Update live equipment RON with explicit profiles for existing representative weapons.
5. Implement or complete targeting profile runtime behavior.
6. Expose current weapon profile in Unity-facing employee/loadout/deployment snapshots if missing.
7. Update canonical Unity docs under `F:\unity projects\ark\docs`.
8. Run focused equipment/targeting/battle/ron tests and then broad checks.

## Status

- Complete.

## Completion Summary

- Added a `Default` implementation for `WeaponCombatProfile` only as an explicit Rust fixture constructor; RON weapons still require authored profiles.
- Migrated test helpers and live equipment RON to valid weapon profiles.
- Implemented runtime `DefaultForward`, `AirFirst`, `LowDefenseFirst`, `LowMagicResistFirst`, and `SplashClusterFirst` selection.
- Added route remaining-distance support for forward targeting.
- Replaced old hint/current-target-first player basic attack expectations with weapon-profile-first behavior.
- Updated canonical Unity docs under `F:\unity projects\ark\docs`.
