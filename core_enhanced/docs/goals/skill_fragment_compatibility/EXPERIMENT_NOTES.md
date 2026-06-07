# Skill Fragment Compatibility Notes

## Findings

- Battle runtime and Safezone-facing views currently have different effective-profile paths.
- Actual battle deployment can apply equipped weapons because `BattleUnitDraft::combat_profile` receives equipped item refs.
- Employee snapshot/profile helpers can miss equipped weapon effects because they start from the employee's stored combat profile and skill fragment loadout without applying equipment.
- This divergence matters for skill fragment compatibility because requirements must be evaluated against the same current weapon profile that battle runtime will use.
- Equipment equip currently mutates `EmployeeLoadout.item_slot` in `world/maintenance.rs` without checking whether the resulting weapon profile still satisfies the employee's active skill fragment.
- Equipment equip handling contains substantial validation/mutation logic inline, including combination outcomes. If incompatible weapon replacement is blocked, this path should use a shared "projected loadout/effective profile" validation helper instead of duplicating one-off checks.

## Policy Gates

- Resolved: the user confirmed weapons are always default-provided, active fragment equip still fails closed if no effective weapon profile exists.
- Resolved: equipment combination/weapon replacement that invalidates an active fragment is rejected.
- Resolved: compatibility failure DTOs use stable core reason codes shared by snapshot and command failure.

## Follow-Up Candidate

- After this goal, consider consolidating all "employee effective battle profile" derivation into a single helper used by:
  - deployment validation
  - roster/safezone snapshot
  - skill fragment compatibility
  - any future equipment preview UI
