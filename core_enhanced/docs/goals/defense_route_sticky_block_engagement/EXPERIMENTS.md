# DefenseRoute Sticky Block Engagement Experiments

This file records implementation attempts, failed approaches, fixes, and verification results for the sticky block engagement goal.

## Initial Investigation

Status: completed before implementation.

Commands/context used:

- Read the abnormal battle record:
  - `battle_records/run_af4bbe2e18145867/9fa8e909-e735-41c2-819a-d846194fb9a3.json`
- Read the normal comparison battle record:
  - `battle_records/run_7e0092b971f809d1/26c58ffc-fe9d-44e0-82d1-2586c3d91ee7.json`
- Read blocking/movement code:
  - `src/game/battle/core/movement/blocking.rs`
  - `src/game/battle/core/movement/planner.rs`
  - `src/game/battle/core/movement/engine.rs`
  - `src/game/battle/core/types.rs`
  - `src/game/battle/types.rs`

Findings:

- Both deployed player units in the compared records are `mobility_kind: ground`.
- The abnormal record's deployed player unit is not platform-only according to its Unity-facing spawn event.
- Default employee combat profile is `GroundOnly`, `block_capacity: 1`, `block_radius_units: 0.75`.
- Corroded employee enemies are `Ground`, `blockable: true`, and `block_capacity: 0`.
- The abnormal record has two enemies inside the player block radius while the player is alive.
- Both enemies continue to receive movement segments past the player.
- The normal record has one enemy, which stops near the player and does not pass through.

Interpretation:

The bug is unlikely to be caused by ground/platform classification or zero default block capacity. The likely root cause is that block assignment is recomputed from scratch every tick, allowing multiple enemies near one capacity-1 blocker to alternate movement instead of preserving a held engagement.

## Planned First Trial

Status: completed.

Create a focused unit test in the movement/blocking test area:

1. One fixed-defense player blocker at a stable tile center.
2. `block_capacity = 1`, `block_radius_units = 0.75`.
3. Two blockable ground route enemies inside or entering radius.
4. Run repeated continuous movement ticks.
5. Assert one enemy is sticky-blocked across ticks and the other is overflow.

Expected failure before implementation:

- Current code may switch the blocked enemy or let both enemies advance over repeated ticks.

Desired pass condition:

- Existing engagement remains stable.
- Only unblocked overflow enemy receives movement.

Result:

- Command: `cargo test fixed_defense_`
- Result: failed as expected before implementation.
- Passing: 18 fixed-defense related tests.
- Failing:
  - `fixed_defense_preserves_existing_block_before_new_route_priority`
  - `fixed_defense_keeps_block_when_enemy_position_drifts_outside_radius`

Interpretation:

- The current implementation replaces an existing held enemy when a later enemy has higher route progress.
- The current implementation also releases block when the enemy position is moved outside radius, which conflicts with the newly fixed policy that normal distance drift must not release block.

## Verification Log

### Sticky Engagement Implementation

Status: passed.

Implementation summary:

- Changed `refresh_block_state()` from full per-tick rebuild semantics to persistent engagement update semantics.
- Existing valid blocker/enemy pairs are preserved before any new capture is considered.
- Invalid pairs are pruned when the blocker/enemy dies, is removed, or no longer satisfies the block policy.
- Remaining block capacity is filled only by currently in-radius unblocked enemies.
- Released block target preferences are cleared when they still point at a released pair.

Focused tests added/updated:

- `fixed_defense_preserves_existing_block_before_new_route_priority`
- `fixed_defense_movement_tick_does_not_emit_segment_for_blocked_enemy`
- `fixed_defense_keeps_block_when_enemy_position_drifts_outside_radius`
- `fixed_defense_releases_block_when_enemy_dies_and_fills_capacity`
- `fixed_defense_releases_block_when_blocker_is_withdrawn`

Verification commands:

- `cargo test fixed_defense_`
  - Result: passed.
  - Coverage: 21 fixed-defense related tests.
- `cargo check -p game_core`
  - Result: passed.
- `cargo test -p game_core`
  - Result: passed.
  - Coverage: 485 unit tests, live item/skill tests, live skill catalog audit, RON loading tests, skill refactor validation tests, skill test suite, and doc tests.

Warnings:

- Cargo still reports an unrelated workspace warning: `/mnt/f/work/simulator/auth_server/Cargo.toml: unused manifest key: env`.
