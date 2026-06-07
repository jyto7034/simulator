# Rapier Backend Quality Refactor Experiment Notes

## Initial Notes

- This goal starts from the post-`Movement Backend Policy Refactor` baseline.
- Rapier/backend physics must remain outside blocking, targetability, deploy occupancy, route progress, and airborne terrain immunity decisions.
- Runtime board bounds source of truth should remain core clamp.
- Unit collider exclusion must stay explicit and tested.
- If deeper review suggests Rapier should be removed entirely, stop and ask the user before implementation.

## Findings

- `RapierMovementWorld::tick` no longer syncs board wall colliders during runtime, but the old board wall helper still exists for tests/low-level inspection.
- The correction method name was too broad: it described generic corrected translation even though the long-term policy is ground static obstacle correction only.
- Rapier correction excluded unit colliders, but stale/test board wall colliders could still participate in the query if present.
- Rapier KCC `slide: true` could turn a blocked authored route into an implicit wall-slide. That conflicts with the policy that route/obstacle authoring errors should surface instead of being hidden by backend behavior.
- Direct and Rapier had individual movement tests, but backend equivalence coverage was too narrow.

## Decisions

- Keep Rapier, but narrow its runtime responsibility to ground static obstacle correction.
- Keep board wall collider helpers as test/low-level helpers only; runtime board bounds remain core clamp.
- Exclude unit colliders and board wall colliders from Rapier ground static obstacle correction.
- Disable Rapier KCC slide for default runtime movement. Ground static obstacles should stop/correct movement, not create untracked detours.
- Use focused Direct/Rapier equivalence tests as the guardrail for future backend edits.

## Follow-Up

- Airborne Enemy Mobility can now attach authored `mobility_kind` to the existing `MovementTerrainPolicy` path without adding Rapier-specific exceptions.
- If future gameplay wants slide around authored obstacles, add an explicit typed obstacle response policy and tests before re-enabling it.
