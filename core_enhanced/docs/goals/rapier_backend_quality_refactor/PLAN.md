# Rapier Backend Quality Refactor Plan

## Current State

- Status: complete.
- This goal is inserted after `docs/movement_backend_policy_refactor_goal.md` and before `docs/airborne_enemy_mobility_goal.md`.
- Implementation is complete.
- The goal exists to harden Rapier backend internals before full airborne enemy mobility is implemented.

## Checklist

1. Read current Rapier/Direct movement flow and record actual responsibilities - complete.
2. Add or update characterization tests for desired backend semantics - complete.
3. Split Rapier backend responsibilities into collider sync, static obstacle query, correction, and board clamp handoff - complete.
4. Remove stale local avoidance or legacy fallback behavior that can hide authored route errors - complete.
5. Keep unit collider exclusion explicit and tested - complete.
6. Keep runtime board bounds source of truth as core clamp - complete.
7. Ensure movement output is diagnosable enough for route/obstacle issues - complete.
8. Run focused movement/blocking/airborne/fixed defense checks - complete.
9. Update goal and master notes - complete.

## Completion Summary

- Renamed Rapier correction surface to ground static obstacle correction semantics.
- Rapier correction now ignores unit colliders and any test/stale board wall colliders.
- Rapier runtime still relies on core board clamp as the board-bound source of truth.
- Disabled Rapier KCC slide so authored static obstacles expose blocked routes instead of implicit wall-sliding detours.
- Added Direct/Rapier equivalence tests for ground obstacle stop policy, airborne static obstacle bypass, and board clamp behavior.
- Added Rapier-specific tests for board wall collider exclusion and no-slide static obstacle behavior.

## Completion Conditions

- Rapier backend responsibility is explicitly limited to ground static obstacle correction helper behavior.
- Direct/Rapier backend policy-visible semantics are locked by tests.
- Unit collider exclusion remains explicit and tested.
- Runtime board bounds remain core clamp's responsibility.
- Stale local avoidance/legacy fallback behavior that hides route errors is removed or explicitly justified.
- Movement diagnostics are sufficient to debug route authoring and obstacle issues.
- 사용자와 의논하여 정해야 할 정책이 있을 경우 goal을 종료한다.
