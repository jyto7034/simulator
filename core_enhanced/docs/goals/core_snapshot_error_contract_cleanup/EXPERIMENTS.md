# Experiments

| Date | Attempt | Result | Follow-up |
|---|---|---|---|
| 2026-06-23 | Goal setup | Created the subgoal document for snapshot drift and blocked-position error cleanup. | Start by reading server snapshot wrapper and `GameError::PositionOccupied` call sites. |
| 2026-06-23 | Snapshot wrapper cleanup | Made `RunSnapshotDto` generic over its selected-event payload and changed `PlayerStateSnapshotDto` to serialize `#[serde(flatten)] core: RunSnapshotDto<PlayerSelectedEventSnapshotDto>`. | Success. Server no longer manually serializes the run snapshot root field list or mutates a `serde_json::Value`. | Add focused test and validate. |
| 2026-06-23 | Static obstacle error split | Added `GameError::StaticObstacleBlocked` and moved battlefield static-obstacle placement plus scenario obstacle/spawn overlap failures to it. Kept `PositionOccupied` for board slot occupancy. | Success. Static obstacle blocking no longer uses the unit/tile occupancy error name. | Validate core/server mappings. |
| 2026-06-23 | Focused validation | Ran `cargo test static_obstacles_block_placement --lib -- --test-threads=1`, `cargo test -p game_server player_state_snapshot_preserves_core_snapshot_root_fields -- --test-threads=1`, `cargo test -p game_server static_obstacle_blocked_uses_static_obstacle_error_code -- --test-threads=1`, and `cargo check -p game_server`. | Success. All passed. | Run subgoal completion review before moving to the next master subgoal. |
