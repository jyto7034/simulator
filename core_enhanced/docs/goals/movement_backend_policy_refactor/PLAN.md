# Movement Backend Policy Refactor Plan

## Objective

Make movement/collision policy explicit in the continuous movement input path so Rapier is only a ground static obstacle correction helper, not the source of truth for blocking, targetability, or future airborne terrain immunity.

## Completion Conditions

- `MovementUnitInput` carries a typed movement policy.
- Ground movement applies board bounds and static obstacle correction.
- Airborne movement can ignore static obstacle/void/walkability correction while still staying inside board bounds.
- Unit colliders remain ignored for movement correction and blocking.
- Direct and Rapier backends expose the same policy-visible behavior for Ground/Airborne movement.
- Local obstacle avoidance does not silently route units around authored static obstacles in the default policy path.
- If code review exposes a policy that must be decided with the user, stop the goal.

## Status

- Complete.

## Implementation Summary

- Added typed `MovementTerrainPolicy` to `MovementUnitInput`.
- Default BattleCore runtime movement input now emits `Ground` policy.
- Direct movement applies static obstacle correction only for `Ground`.
- Rapier movement applies static obstacle correction only for `Ground`.
- `Airborne` policy ignores static obstacles/void obstacle projection while still using core board clamp.
- Removed Direct unit separation from the movement engine.
- Removed Rapier local obstacle avoidance from the default runtime path.
- Rapier runtime no longer syncs board wall colliders during tick; board bounds are enforced by core clamp.
- Added `StaticObstacleBlocked` movement stop reason for debuggable blocked movement output.
- Updated stale test fixtures to match current weapon/profile/target trait fields so movement tests can compile.
