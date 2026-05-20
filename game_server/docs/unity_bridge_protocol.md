# Unity Bridge Protocol

`/game` is the active Unity single-player WebSocket route.

The route is intentionally a thin bridge:

```text
Unity JSON command
-> PlayerBehaviorRequest
-> game_core::game::behavior::PlayerBehavior
-> GameCore::execute(...)
-> CommandResult
-> StateSnapshot from GameCore::get_run_snapshot_json()
```

## Request Naming

Command behavior payloads use `type` with `snake_case` values.

Examples:

```json
{ "type": "start_new_game" }
{ "type": "select_starter_employees", "candidate_ids": ["field_medic", "breach_guard", "records_clerk"] }
{ "type": "request_map_data" }
{ "type": "select_map_node", "node_id": "..." }
{ "type": "confirm_enter_node" }
{ "type": "finish_combat_replay" }
```

Removed phase/suppression-era request names are not part of the active bridge:

- `request_phase_data`
- `select_event`
- `start_suppression`
- `finish_suppression_replay`
- `claim_combat_reward`
- `exit_combat_reward`

Use the current node-map and combat replay requests instead.

## Response Shape

The server keeps the existing two-message flow after command execution:

1. `command_result`
2. `state_snapshot`

`command_result.payload` follows the current `core_enhanced` `BehaviorResult` contract. Combat timelines are returned as `gzip+base64` payloads to keep WebSocket messages smaller.

`state_snapshot.state` is produced from `GameCore::get_run_snapshot_json()`. It is the source of truth for current state, allowed actions, map progression, selected node/session, roster, inventory, and resources.

## Deprecated Legacy Paths

The old multiplayer/matchmaking paths remain buildable for possible future reuse, but they are not the active single-player game bridge:

- `/ws/`
- `/events/stream`
- `matchmaking::*`
- `MatchCoordinator`
- Redis match/pubsub glue
