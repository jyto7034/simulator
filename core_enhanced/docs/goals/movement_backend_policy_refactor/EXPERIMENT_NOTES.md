# Movement Backend Policy Refactor Experiment Notes

## Findings

- Rapier already excludes unit colliders from movement correction, so unit overlap policy is mostly preserved.
- `MovementUnitInput` has no typed ground/airborne terrain policy; every unit currently flows through the same static obstacle and board correction path.
- Direct backend applies static obstacle correction unconditionally.
- Rapier backend applies static obstacle/board Rapier correction unconditionally, then clamps to board.
- Rapier local avoidance can try perpendicular alternatives around static obstacles, which risks changing authored route semantics without an explicit policy.
- `RuntimeUnit` currently has no canonical mobility field; this goal should default runtime units to Ground and leave full enemy `mobility_kind` data/schema for the Airborne Enemy Mobility goal.

## Decisions

- `MovementTerrainPolicy` is the movement backend policy carrier for now.
- `Ground` means static obstacles and void-tile projections are respected.
- `Airborne` means static obstacles and void-tile projections are ignored, while board clamp still applies.
- Board bounds source of truth is core clamp, not Rapier wall colliders in runtime tick.
- Rapier `sync_board_bounds` remains as a low-level helper/test surface, but runtime movement tick no longer uses it.
- Rapier unit colliders may exist for backend bookkeeping, but corrected movement queries exclude them.
- Full authored enemy `mobility_kind` schema remains for `docs/airborne_enemy_mobility_goal.md`.
