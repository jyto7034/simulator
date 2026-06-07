# Unit Overlap Blocking Plan

## Objective

Allow runtime unit overlap while keeping allied deployment overlap forbidden, and make blocking an explicit deterministic state instead of physics collision.

## Completion Conditions

- Unit-unit collision no longer blocks movement.
- Static obstacle and board-bound collision still block movement.
- Blocking assignment prioritizes route progress.
- Excess enemies pass when block capacity is full.
- Blocked enemies and blockers may overlap in world coordinates.
- If code review exposes a policy that must be decided with the user, stop the goal.

## Status

- Complete.
