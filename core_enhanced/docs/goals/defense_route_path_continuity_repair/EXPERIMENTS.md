# Defense Route Path Continuity Repair Experiments

Created: 2026-06-17

This file records implementation attempts, failed approaches, fixes, and verification results for the DefenseRoute path continuity repair goal.

## 2026-06-17 Goal Setup / Code Reading

### Intent

Create the implementation goal after investigating why enemies no longer oscillate but still stop near the spawn row.

### Sources Checked

- `src/game/combat_preview/mod.rs`
- `src/game/combat_preview/validation.rs`
- `src/game/combat_setup/mission_policy.rs`
- `src/game/combat_setup/enemy_spawns.rs`
- `src/game/battle/core/movement/planner.rs`
- `src/game/battle/core/movement/engine.rs`
- `src/game/events/combat.rs`
- `../game_resources/data/map/battlefield_templates.ron`
- `tests/ron_loading.rs`
- `src/game/world/tests/combat.rs`

### Findings

- `ensure_defense_route` creates a fallback route for Defense templates with no authored route.
- `generated_route_cells(start, end)` returns `[start, end]` when the endpoints differ.
- Some live battlefield templates have non-rectangular valid tiles and no authored route.
- A generated route such as `(0,0) -> (3,6)` can therefore become a direct diagonal movement goal through void.
- Movement correction treats void tiles as static obstacles for ground units.
- The observed stop position near `y ~= 0.649` matches a unit with radius `0.35` being blocked by the expanded boundary of a void tile at `y = 1.0`.

### Result

Goal documents created. No runtime code change made in this setup step.

### Verification

Not run. Documentation-only setup step after code inspection.

## 2026-06-17 Implementation / Route Continuity Contract

### Intent

Make generated DefenseRoute cells a real traversable path and pin the behavior with focused tests.

### Changes Tried

- Replaced generated two-point DefenseRoute fallback with deterministic cardinal BFS over valid non-obstacle battlefield tiles.
- Kept generated route construction in `combat_preview`, where valid tiles, obstacles, spawn zones, deployment zones, preview DTOs, and scenario handoff meet.
- Changed generated route endpoint selection to choose an actual deployment-zone cell nearest to the deployment centroid instead of returning the raw centroid coordinate.
- Added `validate_instance` rejection for non-cardinal-adjacent route cells.
- Strengthened battle scenario handoff tests so `TacticalPlan.enemy_plan` and wave `enemy_movement_plan` must equal `CombatPreview.routes[0].cells`, not just share the endpoint.
- Added a generated hook-corridor battle execution test that computes route progress independently and verifies the enemy progresses beyond the spawn boundary.
- Added a live setup snapshot test that verifies generated `battle_setup_snapshot.routes[0].cells` equals the runtime scenario `EnemyMovementPlan::PathAlongCells` cells.

### Failed Attempts / Test Failures

- Command mistake:
  - `cargo test -p game_core generated_defense_route_for_non_rectangular_corridor_is_contiguous validate_instance_rejects_non_contiguous_route_cells -- --nocapture`
  - Result: failed because Cargo accepts only one test filter argument before `--`.
  - Fix: reran the two filters separately.
- Command mistake repeated for failed full-test filters:
  - `cargo test -p game_core live_ron_defense_route_playable_path_runs_to_combat_result defense_combat_node_smoke_writes_debug_event_log_export -- --nocapture`
  - Result: failed for the same Cargo usage reason.
  - Fix: reran each filter separately.
- Broad test failure:
  - `cargo test -p game_core` initially failed in:
    - `game::world::tests::combat::defense_combat_node_smoke_writes_debug_event_log_export`
    - `game::world::tests::combat::live_ron_defense_route_playable_path_runs_to_combat_result`
  - Failure: live checkpoint panicked on a unit without battlefield position.
  - Cause: after the route fix, live DefenseRoute battles progressed far enough for units to die; dead units remain in `BattleCore.units` while their battlefield position is removed and their snapshot moves to the graveyard.
  - Fix: `live_checkpoint_dto` now emits only non-dead units while preserving the invariant panic for a live unit missing a battlefield position.

### Verification

Passed:

- `cargo fmt -p game_core`
- `cargo test -p game_core generated_defense_route_for_non_rectangular_corridor_is_contiguous -- --nocapture`
- `cargo test -p game_core validate_instance_rejects_non_contiguous_route_cells -- --nocapture`
- `cargo test -p game_core defense_node_type_builds_default_black_box_defense_objective -- --nocapture`
- `cargo test -p game_core generated_defense_route_enemy_progresses_past_spawn_boundary -- --nocapture`
- `cargo test -p game_core generated_defense_route_setup_snapshot_matches_runtime_route_cells -- --nocapture`
- `cargo test -p game_core combat_preview -- --nocapture`
- `cargo test -p game_core movement:: -- --nocapture`
- `cargo test -p game_core live_defense_ -- --nocapture`
- `cargo test -p game_core --test ron_loading -- --nocapture`
- `cargo check -p game_core`
- `cargo test -p game_core`

Observed warning on all Cargo commands:

- `/mnt/f/work/simulator/auth_server/Cargo.toml: unused manifest key: env`

### Result

Generated DefenseRoute fallback routes are now deterministic adjacent paths over valid non-obstacle tiles. The old `[start, end]` diagonal fallback is removed. Runtime movement, scenario handoff, setup snapshot route exposure, preview validation, live RON loading, and the full `game_core` test suite pass.
