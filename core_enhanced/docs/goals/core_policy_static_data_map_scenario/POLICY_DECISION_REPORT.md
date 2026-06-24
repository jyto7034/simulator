# Policy Decision Report

## Area

`MapViewDto progression projection`

## Why This Requires Policy Completion

The confirmed policy says `MapViewDto` should use per-node state as the canonical projection and reduce or remove duplicated `available_node_ids` / `completed_node_ids`.

Implementation requires choosing the Unity/server-facing DTO shape:

- Remove the id-list fields from `MapViewDto`.
- Keep them under explicit debug/helper names.
- Keep them as regular public fields for compatibility.

That exact serialized contract was not fully decided, and the current code shows external server usage of `map.available_node_ids`.

## Code/Data Evidence

- `src/game/map/types.rs` exposes `MapViewDto.available_node_ids` and `MapViewDto.completed_node_ids`.
- `src/game/map/progression.rs` fills those fields from `MapProgression`.
- `src/game/world/snapshot.rs` separately serializes `map_progression.available_node_ids` and `map_progression.completed_node_ids`.
- `../game_server/src/game/player_game_actor/handlers.rs` reads `map.available_node_ids` directly.
- Core tests also read these fields in `src/game/world/tests/map_flow.rs`, `src/game/world/tests/snapshots_and_start.rs`, and `src/game/world/tests/combat.rs`.

## Options

1. Remove `available_node_ids` and `completed_node_ids` from `MapViewDto`.
   - Unity/server must derive availability/completion from each `MapNodeDto.state`.
   - This best matches the Source of Truth policy.
   - Requires updating server helper code and tests.

2. Keep debug/helper fields with explicit names such as `debug_available_node_ids` and `debug_completed_node_ids`.
   - Makes the duplicate nature visible.
   - Still preserves two projections in one Unity-facing DTO.
   - Useful for diagnostics but weaker as a SoT cleanup.

3. Keep the existing fields unchanged.
   - Lowest immediate churn.
   - Conflicts with the confirmed long-term direction because Unity/server can keep treating id lists as canonical.

## Recommendation

Choose option 1: remove the id-list fields from `MapViewDto` and make `MapNodeDto.state` the Unity/server-facing source for availability and completion.

If debug lists are still useful, expose them outside the gameplay DTO through a separate debug snapshot path rather than regular `MapViewDto`.

## User Decision

User confirmed option 1: remove `available_node_ids` and `completed_node_ids` from `MapViewDto`, and make `MapNodeDto.state` the canonical Unity/server-facing source.

## Goal Outcome

This policy-decision report is resolved and implemented with option 1.

Implementation removed the id-list fields from `MapViewDto`; internal `MapProgression` keeps its runtime lists for progression logic.
