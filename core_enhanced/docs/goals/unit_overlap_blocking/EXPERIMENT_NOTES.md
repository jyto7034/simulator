# Unit Overlap Blocking Experiment Notes

## Findings

- `RapierMovementWorld` stores board bounds, static obstacles, and unit colliders in the same collider set. The movement query must exclude unit colliders while preserving bounds/static obstacles.
- Steering defaults still create side-bias/congestion behavior around units. Runtime defaults need to stop treating other units as movement pressure.
- `RuntimeUnit` did not previously preserve explicit spawn order or instance salt. The goal added `spawn_order` as runtime-only state so tie-break does not collapse into unit id order.
- Steering still has non-default test coverage for side-bias/congestion helpers, but runtime default parameters no longer apply those pressures to unit movement.
