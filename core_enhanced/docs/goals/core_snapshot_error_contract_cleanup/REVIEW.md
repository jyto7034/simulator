# Completion Review

Review procedure: `docs/goal_completion_review_guide.md`

## Audit Table

| Item | Verdict | Long-term fit | Evidence |
|---|---|---|---|
| `PlayerStateSnapshotDto` / `RunSnapshotDto` root field drift | Complete | High | `RunSnapshotDto` is generic over selected-event payload; `PlayerStateSnapshotDto` flattens the core DTO and no longer manually serializes root fields. Focused server test compares root field sets. |
| `PositionOccupied` static obstacle meaning cleanup | Complete | High | Static obstacle placement and scenario obstacle/spawn overlap return `StaticObstacleBlocked`; `PositionOccupied` remains only for board slot occupancy. Core/server error mappings expose `static_obstacle_blocked`. |

## Player State Snapshot DTO Drift Risk

Master item:
`core_snapshot_error_contract_cleanup` should remove `PlayerStateSnapshotDto` / `RunSnapshotDto` top-level field drift risk.

Subgoal:
`docs/goals/core_snapshot_error_contract_cleanup`

Policy source:
`docs/goals/core_component_refactor_master/POLICY_DECISIONS.md` / `Player state snapshot DTO drift risk`

Expected behavior:
Server may enrich the selected event with transport compression data, but it must not own or duplicate the root run snapshot field list.

Runtime evidence:
- `src/game/behavior.rs`: `RunSnapshotDto<SelectedEvent = SelectedEventSnapshotDto>` owns the snapshot root shape.
- `src/game/behavior.rs`: `RunSnapshotDto::map_selected_event` changes only the selected-event payload type.
- `../game_server/src/game/player_game_actor/state.rs`: `PlayerStateSnapshotDto` has `#[serde(flatten)] core: RunSnapshotDto<PlayerSelectedEventSnapshotDto>`.
- `../game_server/src/game/player_game_actor/state.rs`: there is no custom `Serialize` implementation with `serialize_struct("PlayerStateSnapshotDto", 13)`.

Data evidence:
- Not applicable. This item is a typed DTO/server serialization boundary cleanup.

External contract evidence:
- The root JSON shape is still the serialized core run snapshot shape.
- Server transport enrichment remains on `selected_event.compressed_event_log`.

Test evidence:
- `cargo test -p game_server player_state_snapshot_preserves_core_snapshot_root_fields -- --test-threads=1` passed.
- `cargo check -p game_server` passed.

Legacy/fallback audit:
- No `serde_json::Value` mutation was reintroduced.
- No compatibility wrapper or dual snapshot root schema was added.

Long-term direction review:
- Fit: high
- Improvement class: none
- Reason: core owns the root snapshot shape; server only owns the selected-event transport attachment.

Verdict:
- Complete

Remaining risk:
- If future server-specific fields are needed outside `selected_event`, they should be added as typed composition deliberately, not by reintroducing a second root field list.

## PositionOccupied Error Meaning Cleanup

Master item:
`core_snapshot_error_contract_cleanup` should clean up `PositionOccupied` / `position_occupied` naming so static obstacle blocking does not imply unit tile occupancy.

Subgoal:
`docs/goals/core_snapshot_error_contract_cleanup`

Policy source:
`docs/goals/core_component_refactor_master/POLICY_DECISIONS.md` / `PositionOccupied error meaning cleanup`

Expected behavior:
Static obstacle blocked placement should have a static-obstacle/blocking error meaning. `PositionOccupied` should remain only where the domain really is an occupied position or slot.

Runtime evidence:
- `src/game/behavior.rs`: added `GameError::StaticObstacleBlocked`.
- `src/game/battle/battlefield/field.rs`: placing a unit onto a static obstacle returns `StaticObstacleBlocked`.
- `src/game/combat_setup/battlefield_plan.rs`: static obstacle overlap with authored spawn returns `StaticObstacleBlocked`.
- `src/game/resources/board.rs`: `PositionOccupied` remains for occupied board slots.

Data evidence:
- Not applicable. This item changes error classification, not live RON schema.

External contract evidence:
- `src/game/world/snapshot.rs`: maps `StaticObstacleBlocked` to `static_obstacle_blocked`.
- `../game_server/src/game/player_game_actor/state.rs`: maps `StaticObstacleBlocked` to `static_obstacle_blocked` with a static-obstacle message.
- `position_occupied` remains for actual occupied-slot errors.

Test evidence:
- `cargo test static_obstacles_block_placement --lib -- --test-threads=1` passed.
- `cargo test -p game_server static_obstacle_blocked_uses_static_obstacle_error_code -- --test-threads=1` passed.
- `cargo check -p game_server` passed.

Legacy/fallback audit:
- Static obstacle paths no longer use `PositionOccupied`.
- No compatibility alias from `static_obstacle_blocked` back to `position_occupied` was added.

Long-term direction review:
- Fit: high
- Improvement class: none
- Reason: error names now follow domain responsibility instead of preserving an obsolete unit/tile occupancy implication.

Verdict:
- Complete

Remaining risk:
- Unity clients that special-cased `position_occupied` for static obstacle placement must consume `static_obstacle_blocked`.

## Cross-Component Findings

- No competing source of truth found in this subgoal. Core owns the run snapshot root shape; server owns transport enrichment.
- `PositionOccupied` still exists, but only in board-slot occupancy paths where the name remains semantically accurate.

## Immediate Correction Candidates

- None.

## Follow-Up Refactor Candidates

- None inside this subgoal. `Battlefield` dynamic position ownership remains intentionally out of scope for `core_battlefield_layout_extraction`.

## Validation Commands

- `cargo test static_obstacles_block_placement --lib -- --test-threads=1` - passed.
- `cargo test -p game_server player_state_snapshot_preserves_core_snapshot_root_fields -- --test-threads=1` - passed.
- `cargo test -p game_server static_obstacle_blocked_uses_static_obstacle_error_code -- --test-threads=1` - passed.
- `cargo check -p game_server` - passed.
