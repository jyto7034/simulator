# Unit Overlap Blocking Experiments

## Log

- Read movement backend, blocking planner, Rapier tests, and policy docs.
- Found that Rapier unit colliders were still included in character-controller movement correction.
- Found Direct movement fallback still had a separation solver.
- Found blocking candidate order was deterministic unit id order, not route progress.
- Updated Rapier movement correction to exclude unit colliders while keeping board bounds/static obstacles.
- Disabled default unit avoidance/separation in Direct and Rapier movement paths.
- Added `RuntimeUnit.spawn_order` and used route progress, spawn order, then unit id for block candidate priority.
- Replaced legacy movement tests that expected unit avoidance with overlap-allowed tests.
- Added live deployment test coverage for allied occupied deployment cell rejection.
- Updated canonical Unity docs:
  - `/mnt/f/unity projects/ark/docs/unity_core_contract.md`
  - `/mnt/f/unity projects/ark/docs/unity_client_implementation_goal.md`

## Failed Experiment

- First Rapier unit-collider filter compile failed due closure lifetime/type requirements. Fixed by binding a typed predicate.

## Verification

- `cargo test -p game_core movement -- --nocapture`: passed.
- `cargo test -p game_core fixed_defense_block -- --nocapture`: passed.
- `cargo test -p game_core live_defense -- --nocapture`: passed.
- `cargo check -p game_server`: passed.
