# Experiments

This file records implementation attempts, validation commands, failures, fixes, and final verification for `core_tile_validity_typed_dto_cleanup`.

## Log

| Date | Scope | Attempt | Result | Follow-up |
| --- | --- | --- | --- | --- |
| 2026-06-23 | Goal setup | Created the subgoal document for tile validity audit and server/admin typed DTO boundary cleanup. | Pending implementation. | Start by reading policy docs and inventorying `in_bounds` and `serde_json::Value` shape assembly. |
| 2026-06-23 | `Battlefield::in_bounds` audit | Read `Battlefield::in_bounds`, range preview, skill runtime area clipping, explicit tile cast target validation, `WholeFieldValidTiles` target eligibility, and event-log validation raw rectangle checks. | Current callers consistently use `Battlefield::in_bounds` as authored valid-tile policy, while raw rectangle checks use `position_in_bounds` in event-log validation. No rename/wrapper needed. | Keep `Battlefield::in_bounds` unchanged; continue with typed DTO boundary cleanup. |
| 2026-06-23 | server snapshot typed DTO | Replaced `PlayerGameActor::build_state_snapshot` `serde_json::Value` object mutation with `PlayerStateSnapshotDto` and `PlayerSelectedEventSnapshotDto`; final transport still serializes to `Value`. | `selected_event.compressed_event_log` is now represented by a typed server snapshot DTO, and `as_object_mut`/string-key insertion is gone from the affected path. | Run focused server test and record validation. |

## Failed Approaches

Record failed approaches and why they were abandoned.

## Validation Commands

Record focused and broad validation commands here.

- `cargo check -p game_server` (pass)
- `cargo test -p game_server finished_live_battle_tick_pushes_combat_result_snapshot_after_final_update -- --test-threads=1` (pass)
- `cargo fmt -p game_server` (pass)
- `rg -n "as_object_mut|Value::Object|get_mut\\(\"selected_event\"|compressed_timeline|compressed_event_log" ../game_server/src/game/player_game_actor src/game/world -S` (pass: only typed DTO fields, final serialization, and tests mention `compressed_event_log`; no affected-path `Value` mutation remains)
