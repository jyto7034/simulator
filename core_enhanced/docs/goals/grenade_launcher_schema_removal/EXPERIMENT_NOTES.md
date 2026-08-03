# Grenade Launcher Schema Removal Notes

Status: complete

## Starting Evidence

- Reaudit found the runtime schema still exposed `GrenadeLauncher` although targeting/content docs
  excluded grenade weapons as current and described them as future-only.
- User approved removal from current schema and retention only as documentation for later design.

## Resume Point

- Child complete. Resume from Master G04.

## Final Ownership Classification

- Current schema authority removed: Core `WeaponArchetype`.
- Exhaustive test fixture repaired: world test `weapon_equipment` range-role match.
- Current contract inventory repaired: external `unity_core_contract.md`.
- Current visual authoring inventories repaired: external English/Korean E.G.O visual briefs.
- Future concept retained: Core content catalog and explicit future-only visual brief sections.
- Historical goal/audit occurrences remain immutable evidence and are not current authority.

## Validation Summary

- Removed-value parse regression: 1 passed.
- `ron_loading`: 20 passed.
- Core library: 739 passed.
- Rust formatting and Core/external diff checks passed.
