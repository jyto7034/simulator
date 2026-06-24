# Defense Route Path Continuity Repair Notes

Created: 2026-06-17

## Current Diagnosis

The remaining movement issue is not primarily Unity rendering and not primarily battle update transport.

Current evidence points to generated route topology:

- Authored routes from template overlays produce ordered route cells.
- Fallback/generated routes currently produce only `[start, end]`.
- Enemy movement consumes route cells as world-space waypoint centers.
- Ground movement collides with void/static obstacle cells.
- In non-rectangular maps, direct endpoint movement can hit void boundaries immediately after spawn.

This means the project currently has two route qualities:

- Authored overlay route: likely usable as path.
- Generated fallback route: endpoint pair, not guaranteed to be path.

The goal should remove that split by making generated fallback routes real paths or by failing generation when a real path cannot be found.

## Important Runtime Boundaries

- `combat_preview` is the earliest place that has battlefield valid tiles, obstacles, spawn zones, deployment zones, and route DTOs together.
- `mission_policy` and `enemy_spawns` already copy `route.cells`; they should not need a second route-generation policy.
- `battle_setup_snapshot` should expose the same route cells that core movement uses.
- Movement planner should not be responsible for inventing a route from invalid route data.

## Policy Leaning

Use cardinal adjacency for generated DefenseRoute paths unless code/data proves diagonal tile adjacency is already official.

Reasoning:

- Battlefield templates are ASCII grid based.
- Existing route overlays use arrows such as `v` and `>`, implying cardinal path steps.
- Collision/void behavior is tile based; diagonal corner cutting through void would be surprising for DefenseRoute enemies.

If diagonal adjacency is desired as a gameplay policy, stop and ask the user before implementing.

## Implementation Decisions

- Generated fallback route pathfinding uses cardinal BFS.
  - This matches existing authored route overlays, which use `>`, `<`, `^`, and `v`.
  - It avoids diagonal corner cutting through void tiles on non-rectangular ASCII battlefields.
- Generated fallback route endpoints now resolve to an actual deployment-zone cell nearest to the deployment-zone centroid.
  - The previous raw centroid happened to be valid for current live templates, but choosing a real authored deployment cell is a stronger long-term contract.
  - This did not require a RON schema or Unity-facing DTO shape change.
- `validate_instance` rejects non-cardinal-adjacent route cells.
  - This makes route continuity a runtime/data contract and prevents future generated or authored route jumps from silently passing.
- Live checkpoints skip dead units.
  - Dead units remain in `BattleCore.units` for combat history/state purposes, but their battlefield placement is removed and their death snapshot is kept in the graveyard.
  - Checkpoint DTOs represent current live battlefield state, so emitting only non-dead units is the correct boundary.
  - The invariant remains strict for a non-dead unit that has no battlefield position.

## Likely Tests To Add

- Generated route for `corridor_medium_hook_01` is not `[start, end]`.
- Generated route cells are all valid, non-obstacle, and adjacent.
- `validate_instance` rejects a route with a non-adjacent jump.
- Scenario conversion preserves generated route cells in `TacticalPlan.enemy_plan` and wave group `enemy_movement_plan`.
- Live battle movement on a fallback-generated route advances route progress for at least several ticks and does not remain at `y ~= 0.65`.
- Setup snapshot route cells equal the scenario route cells for the same battle.

## Follow-Up Candidates

- Client-facing `MovementStopped` event for `StaticObstacleBlocked`.
  - Not required to fix route continuity.
  - Useful for diagnostics if a route or obstacle bug appears again.
- A debug assertion or validation mode that checks official DefenseRoute encounters have at least one complete route from spawn to defense objective.
- Richer route authoring for templates that currently rely on generated fallback routes.

## Open Questions

None blocking at goal creation time.

Potential future policy question:

- Should generated route pathfinding prefer shortest path, route toward deployment centroid, or route toward explicit defense objective/tactical point when those differ?

Current answer for this goal:

- Use deterministic shortest valid cardinal path toward the selected defense endpoint.
- Keep richer route authoring or explicit defense-object endpoint policy as a follow-up if future encounters need more authored control.
