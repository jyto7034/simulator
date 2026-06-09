# Maintenance Independent Node Experiments

Record implementation attempts, failures, fixes, and validation results for the Maintenance independent node goal.

## 2026-06-08 - Goal Created

Goal: split Maintenance from `SupportNodeType::Maintenance` into an independent map node.

Initial policy:

- Maintenance is a workbench-style node, not a one-shot Support effect.
- Existing Maintenance operations should remain, but their availability should key off the independent Maintenance node.
- Support should focus on Medical/Rest and no longer expose Maintenance as a support choice.

Next:

- Read runtime code before making assumptions.
- Decide whether `ActiveNodeContent::Maintenance` needs a dedicated session type.
- Update external Unity contracts only after runtime shape is confirmed.

## 2026-06-08 - Runtime Split Implementation

Attempt:

- Added `MapNodeCategory::Maintenance`, `MapNodePayload::Maintenance`, and `NodeSessionKind::Maintenance`.
- Removed `Maintenance` from `SupportNodeType`.
- Added `MaintenanceSessionState` and `ActiveNodeContent::Maintenance`.
- Moved Maintenance entry through `world/map_content.rs` independent of Support.
- Changed Maintenance action gating to check `ActiveNodeContent::Maintenance`.
- Changed snapshot `selected_event` to emit `type: "maintenance"` with `maintenance_options`.
- Changed `BehaviorResult` to return `MaintenanceState` instead of embedding `maintenance_options` in `SupportState`.
- Migrated live `node_definitions.ron` so `support_maintenance` is now a `Maintenance` category/payload node.

Result:

- `cargo check -p game_core` passed after removing the stale helper that built old support-bound Maintenance options.

## 2026-06-08 - Tests Updated

Attempt:

- Replaced old Support-Maintenance fixtures in world tests with `MapNodeCategory::Maintenance` and `MapNodePayload::Maintenance`.
- Updated preview assertions from `SupportState { maintenance_options: Some(...) }` to `MaintenanceState { maintenance_options, .. }`.
- Added/kept tests that Support choices do not expose Maintenance actions and independent Maintenance nodes do.
- Updated game_server admin command tests to deserialize `admin_enter_maintenance`.

Validation:

```text
cargo test -p game_core maintenance -- --nocapture
cargo test -p game_core support -- --nocapture
cargo test -p game_server maintenance -- --nocapture
cargo test -p game_server admin -- --nocapture
cargo test -p game_core --test ron_loading -- --nocapture
```

Result:

- All listed focused tests passed.
- `ron_loading` confirmed live RON map content pools resolve with the independent Maintenance node.

## 2026-06-08 - Unity Contract And Probe Update

Attempt:

- Updated external canonical Unity docs under `/mnt/f/unity projects/ark/docs`.
- Replaced `admin_enter_support { support_type: "Maintenance" }` with `admin_enter_maintenance`.
- Replaced `selected_event.type == "support"` Maintenance routing with `selected_event.type == "maintenance"`.
- Removed Maintenance from Support choices in documented snapshots.
- Updated the admin debug probe to assert a `maintenance` selected event.

Result:

- Documentation now matches the runtime shape.
- Live probe still needs a final execution after formatting/final checks.

## 2026-06-08 - Live Probe

Failed attempts:

- Running `cargo run -p game_server` from the workspace root failed because `game_server/config/development` was not found.
- Running from `game_server` on the default port failed because `0.0.0.0:8081` was already in use.
- `APP_SERVER__PORT` did not override the port because the server config uses `Environment::with_prefix("APP").separator("__")`; the correct key is `APP__SERVER__PORT`.
- Running the Python probe inside the sandbox failed with `PermissionError` while creating a local socket.

Fix:

- Started the server from `/mnt/f/work/simulator/game_server` with:

```text
APP__SERVER__BIND_ADDRESS=127.0.0.1 APP__SERVER__PORT=18083 ENABLE_ADMIN_COMMANDS=true ADMIN_COMMAND_TOKEN=dev cargo run -p game_server
```

- Ran the probe with approved local socket access:

```text
ADMIN_COMMAND_TOKEN=dev WS_PORT=18083 python3 "/mnt/f/unity projects/ark/docs/unity_admin_debug_command_probe.py"
```

Result:

- Probe passed.
- Probe observed `AdminEnteredMaintenance`.
- Probe verified `maintenance_options` keys `items` and `materials`.
- Probe continued through Medical, Shop, Reward, Headquarters Contact, and inventory dump admin flows.

## 2026-06-08 - Broad Test Catch

Command:

```text
cargo test -p game_core
```

Failure:

- `examples/map_dump.rs` did not cover the new `MapNodeCategory::Maintenance` in its category symbol/name matches.

Fix:

- Added `Maintenance => "M"` and `Maintenance => "Maintenance"` in `examples/map_dump.rs`.

Result:

- Rerun passed.
- `cargo test -p game_core` passed with 446 lib tests plus integration/doc test suites.

## 2026-06-08 - Final Server Test

Command:

```text
cargo test -p game_server
```

Result:

- Passed 16 server tests.
- Confirms admin command message routing and `BehaviorResult::MaintenanceState` server mapping compile through the full server test target.
