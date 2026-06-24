# Stat Pipeline And Inventory Contract Repair Notes

## Initial Findings

- `stat_pipeline.rs` centralized logic but still had two runtime functions whose relationship depended on call sites:
  - employee profile stage in employee/combat setup,
  - draft final-stat stage in battle build.
- The previous `final_employee_stats_apply_consumable_boost_before_equipment_effects` test did not strongly prove cross-stage order because it used a flat equipment modifier after a precomputed consumable profile.
- Artifact remains a valid retained concept, but `Inventory::remove_item` returning `None` for artifact UUIDs conflates "missing item" and "known but not removable".

## Policy

- Keep artifact.
- Do not make artifact sellable/removable through the generic remove path.
- Prefer explicit domain errors over silent `None` when the UUID is known but not removable.

## Final Decisions

- The stat pipeline remains split into employee-profile and draft-final-stat functions because that matches the runtime scenario assembly boundary.
- The module now documents those two stages as one battle-entry contract instead of two unrelated helper functions.
- The order-sensitive stat test uses consumable +50% followed by equipment +10% percent math, so reversing the effective order changes the expected value.
- Generic `Inventory::remove_item` now returns a domain `Result`; artifact UUIDs produce `InventoryRemoveError::NotRemovableArtifact`, which maps to `GameError::InventoryItemNotRemovable`.
- Shop sell rejects owned artifact UUIDs with `InventoryItemNotRemovable`, preserving artifact while making the Unity-facing error explicit.

## Follow-Up Candidates

- If artifact later gets a dedicated UX surface, document it as a global passive/run relic category distinct from equipment, skill fragments, consumables, and maintenance materials.
