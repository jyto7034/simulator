# Defense Route Path Continuity Repair Plan

Created: 2026-06-17

Status: Implemented and verified on 2026-06-17.

## Objective

Fix DefenseRoute fallback/generated route movement so enemies spawned on route start cells continue along a valid battlefield path instead of moving diagonally into void/static obstacles and stopping near the spawn row.

This goal must make route continuity a runtime/data contract, not a one-off fix for the currently observed Unity log.

## Source Of Truth Order

Use this order while implementing:

1. Runtime code.
2. Live RON/data.
3. Unity-facing setup/update DTO contract.
4. Current policy docs.

Do not trust docs over code. If code/data evidence shows a better long-term fix, record the evidence in `EXPERIMENT_NOTES.md` and use that direction unless a policy decision is required.

## Code Evidence Already Read

- `src/game/combat_preview/mod.rs`
  - `ensure_defense_route` creates `generated_defense_route` when a Defense template has no authored route.
  - `generated_route_cells(start, end)` currently returns only `[start, end]` for different endpoints.
  - This allows generated routes such as `(0,0) -> (3,6)` to become diagonal straight-line movement rather than adjacent cell pathing.
- `../game_resources/data/map/battlefield_templates.ron`
  - Some Corridor templates have non-rectangular valid tiles and no authored route.
  - Example shape has `NNN` in row 0 and void spaces below some spawn cells, so diagonal movement from spawn to deployment centroid crosses void.
- `src/game/combat_preview/validation.rs`
  - Validates route cells are non-empty, start/end aligned, inside valid tiles, and not obstacles.
  - It does not currently require adjacent route cells or prove a route follows a traversable cell path.
- `src/game/combat_setup/mission_policy.rs`
  - `default_defense_route_cells` copies the first preview route into the scenario tactical plan.
- `src/game/combat_setup/enemy_spawns.rs`
  - Wave `route_id` selects `route.cells` as each enemy group movement plan.
- `src/game/battle/core/movement/planner.rs`
  - Opponent movement consumes `EnemyMovementPlan::PathAlongCells`.
- `src/game/battle/core/movement/engine.rs`
  - Ground movement corrects against static obstacles and void tiles.
  - If movement correction produces no body movement, no `MovementSegmentStarted` is emitted.

## Problem Statement

Latest Unity/core wire logs showed:

- Immediate back-and-forth route retargeting was fixed in the previous movement planner repair.
- Enemies now move only briefly after spawn and then remain around `y ~= 0.649`.
- `MovementSegmentStarted` stops after the first short segments.
- Checkpoint positions remain fixed, so this is not just a Unity render interpretation issue.

The most likely cause is generated route shape:

```text
generated_route_cells(start, end)
  -> [start, end]
```

For a non-rectangular corridor, this makes the enemy attempt direct diagonal movement through void cells. With unit radius `0.35`, a void boundary at `y = 1.0` blocks the unit around `y = 0.65`, matching the observed checkpoint positions.

## Scope

In scope:

- Replace generated DefenseRoute fallback route construction with a valid adjacent path over existing valid, non-obstacle battlefield tiles.
- Add validation that route cells are contiguous enough for movement.
- Ensure generated routes used by `combat_preview`, battle scenario construction, `battle_setup_snapshot`, and movement runtime are the same route.
- Add focused tests for route generation and live movement progress on non-rectangular corridor templates.
- Add/adjust tests that pin Unity-facing route DTO shape when route cells are generated.
- Record any policy/data conflicts before changing live RON schema or authored content semantics.

Out of scope:

- Reworking the whole movement engine.
- Changing Unity consumer behavior.
- Changing battle transport DTO names or message order.
- Adding dual schema support for old route semantics.
- Balancing enemy speed, body radius, block radius, or wave timing unless required to prove movement flow.
- Editing live RON content as a workaround when runtime can generate a correct route from existing valid tiles.

## Long-Term Direction

DefenseRoute route cells must represent the path enemies are expected to follow.

Rules to lock:

- `route.cells[0] == route.start`.
- `route.cells.last() == route.end`.
- Every route cell is a valid battlefield tile.
- Route cells do not overlap obstacles.
- Consecutive route cells are adjacent by the movement/pathing policy chosen for DefenseRoute.
- Generated routes must be deterministic.
- Generated routes must use valid tile topology, not direct endpoint interpolation.
- Scenario enemy movement plans must consume the same route cells Unity sees in `battle_setup_snapshot.routes`.

Preferred implementation direction:

1. Add a small deterministic grid path helper in the combat preview/template layer. Done.
2. Use it in `ensure_defense_route` instead of `[start, end]`. Done.
3. Validate route contiguity in `validate_instance`. Done.
4. Let existing scenario handoff continue to copy `route.cells`; avoid adding a compatibility adapter. Done.

## Candidate Implementation Plan

1. Read runtime/data again before editing.
   - Confirm exact type fields in `BattlefieldRoute`, `BattlefieldInstance`, `SpawnWave`, `BattleScenario`, `EnemyMovementPlan`.
   - Confirm whether diagonal adjacency is allowed anywhere. If unclear, use cardinal adjacency as the conservative default and record the choice.

2. Add route path generation.
   - Implement deterministic pathfinding between fallback route start and end.
   - Inputs: `valid_tiles`, `obstacles`, `width`, `height`, `start`, `end`.
   - Output: ordered `Vec<Position>` from start to end.
   - Prefer BFS or another simple deterministic shortest path over valid non-obstacle tiles.
   - Neighbor order must be stable.

3. Replace fallback route construction.
   - Change `generated_route_cells` or replace it with a helper that can return `Result<Vec<Position>, GameError/String>`.
   - If no path exists, fail preview generation instead of silently creating an invalid two-point route.
   - Do not fall back to diagonal `[start, end]`.

4. Strengthen validation.
   - `validate_instance` should reject non-contiguous route cells.
   - Keep existing route start/end/valid/obstacle checks.
   - Add tests for rejection of route jumps.

5. Add gameplay/runtime tests.
   - Template-level test: generated route for non-authored Corridor/Hook template is adjacent and valid.
   - RON loading/data test: every Defense template route is valid, obstacle-free, and contiguous after fallback generation.
   - Scenario handoff test: `BattleScenario.tactical_plan.enemy_plan` and wave group movement plans use the same generated route cells.
   - Live movement test: enemy on generated corridor route has monotonically increasing route progress over multiple movement ticks and does not stop at the spawn boundary.

6. Unity-facing contract check.
   - Verify `battle_setup_snapshot.routes[*].cells` carries the repaired generated route.
   - If the DTO shape does not change, no Unity contract document update is required.
   - If route semantics are clarified in docs, update external Unity canonical docs and local docs index/guidelines as needed.

7. Verification.
   - Run focused route generation/validation tests.
   - Run movement tests.
   - Run live DefenseRoute tests.
   - Run `cargo check -p game_core`.
   - Run broader `cargo test -p game_core` if blast radius touches shared preview/data loading.

## Completion Conditions

- Generated DefenseRoute fallback routes are valid adjacent paths across valid non-obstacle tiles. Done.
- No generated route silently uses a two-point diagonal jump through void/obstacles. Done.
- `validate_instance` rejects route jumps. Done.
- Live route movement on a non-rectangular corridor progresses beyond the spawn-row boundary. Done.
- Unity-facing setup snapshot route cells and core scenario route cells agree. Done and pinned with `generated_defense_route_setup_snapshot_matches_runtime_route_cells`.
- Relevant focused tests and checks pass. Done.
- Any docs touched by DTO/schema/policy changes are updated. No DTO/schema shape changed; goal docs updated.
- `EXPERIMENTS.md` records failed attempts, test failures, fixes, and final verification. Done.
- `EXPERIMENT_NOTES.md` records policy questions and follow-up candidates. Done.

## Implementation Summary

- `ensure_defense_route` now generates `generated_defense_route.cells` with deterministic cardinal BFS over valid non-obstacle tiles.
- Generated route endpoint selection now chooses an actual deployment-zone cell nearest to the deployment centroid.
- `validate_instance` rejects non-cardinal-adjacent route cells.
- Scenario handoff tests now require preview route cells to equal tactical plan and wave movement plan route cells.
- A hook-corridor execution test verifies route progress beyond the spawn boundary.
- A live setup snapshot test verifies generated route cells equal runtime scenario route cells.
- Live battle checkpoint DTOs now skip dead units, while still panicking if a live unit has no battlefield position.

## Stop Conditions

Stop and report questions instead of deciding unilaterally if any of these are discovered:

- Live RON schema must change.
- Existing battlefield templates must be deleted or semantically reauthored.
- Diagonal movement/path adjacency is a design decision rather than a technical bug.
- Enemy radius, speed, spawn placement, or route endpoint policy needs balance/UX approval.
- Unity-facing DTO shape must change.
- A route cannot be generated from current valid tiles for an official live encounter.

## Verification Commands

Expected focused commands:

```text
cargo test -p game_core combat_preview -- --nocapture
cargo test -p game_core movement:: -- --nocapture
cargo test -p game_core live_defense_ -- --nocapture
cargo test -p game_core --test ron_loading -- --nocapture
cargo check -p game_core
```

If implementation touches server message mapping or setup snapshot serialization:

```text
cargo test -p game_server player_game_actor -- --nocapture
cargo check -p game_server
```
