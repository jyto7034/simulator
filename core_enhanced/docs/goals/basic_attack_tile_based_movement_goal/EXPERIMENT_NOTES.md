# Basic Attack Tile-Based Movement Goal Notes

## Policy Decisions

- Basic attack target selection, eligibility, and movement stop decisions are tile-based.
- `range_units` must not be used for basic attack stop distance or generic basic attack acquisition.
- Blocked enemies attack their blocker first. This is modeled as same-logical-tile engagement, not as a continuous distance exception.
- Melee route enemies that are not blocked keep moving along the route unless a tile-based policy explicitly gives them an attack target.
- Ranged route enemies stop only when a valid target is inside their tile attack range.
- `ranged_reposition_ms` remains the data-driven post-attack movement window for ranged route enemies.
- Airborne movement may ignore blocker movement constraints where existing policy allows it, but airborne basic attacks still require tile range.
- Route-less bosses and special units are out of scope. Their behavior must be handled by boss/special-unit-specific policy work, not by a shared fallback in this goal.
- `range_units` remains available for future explicitly continuous mechanics such as projectile collision, contact, aura, and contagion.

## Current Runtime Shape After Repair

- `choose_attack_target_in_range()` uses tile range policy and should remain the core basic attack target acquisition API.
- `is_basic_attack_target_in_range()` is the tile-based eligibility check.
- `MovementGoal::AttackUnit` is now attack intent only: `AttackUnit { target_id }`.
- Movement planning uses tile-range target acquisition for basic-attack stops. It no longer fills an attack stop distance from `basic_attack.range_units`.
- The old continuous-distance `closest_enemy_in_attack_range()` basic attack acquisition helper was removed from the movement flow.
- `MovementUnitInput` no longer carries `attack_range_units`; the direct movement engine does not receive basic attack range as a physical stop distance.
- Route-less boss/special behavior stayed out of scope. The inventory did not find an affected route-less branch in the repaired `EnemyMovementPlan` path.

## DTO Expectation

No Unity-facing DTO shape change is expected.

The transport already sends movement events and battle checkpoints. This goal changes runtime decision semantics, not JSON structure.

## Follow-Up Candidates

- Rename or split continuous-distance helpers so future aura/contact/contagion systems do not accidentally look like basic attack range.
- Add debugging output that reports why a unit stopped: blocked, tile-range target, route boundary, or reposition window.
- Consider a tile-local presentation offset policy for same-tile blocker combat so multiple units inside one logical tile are visually readable without changing gameplay coordinates.

## Open Questions

None at goal creation time. Stop conditions in `PLAN.md` cover the likely policy blockers.
