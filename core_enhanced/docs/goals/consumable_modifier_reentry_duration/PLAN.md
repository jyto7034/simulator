# Consumable Modifier Re-entry Duration Plan

## Objective

Make consumable modifiers persist across retreat/re-entry attempts and decrement only when the combat abnormality node is resolved.

## Completion Conditions

- `UseConsumableItem` consumes inventory immediately.
- Used items are not refunded by node cancel, retreat, re-entry, or modifier overwrite.
- Retreat with remaining abnormality attempts does not decrement duration.
- Success, non-retreat failure, and exhausted-attempt retreat decrement duration.
- Focused tests cover retreat/re-entry persistence and exhausted-attempt expiry.
- If code review exposes a policy that must be decided with the user, stop the goal.

## Status

- Complete.
