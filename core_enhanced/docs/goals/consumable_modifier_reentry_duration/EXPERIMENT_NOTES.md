# Consumable Modifier Re-entry Duration Experiment Notes

## Findings

- `UseConsumableItem` already removes the owned item before applying the modifier and returns `inventory_diff.removed`.
- Overwrite already returns the replaced modifier without refunding either item.
- Duration storage is employee-local through `ActiveConsumableModifier.remaining_combat_nodes`.
- `handle_complete_combat_result` already decrements duration for non-retreat victory/failure node resolution.
- `RetreatBattle` must distinguish remaining-attempt retreat from exhausted-attempt retreat because only the latter resolves the node.
