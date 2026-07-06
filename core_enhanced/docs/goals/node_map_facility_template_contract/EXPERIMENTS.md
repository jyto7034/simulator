# Node Map Facility Template Contract Experiments

## Experiment Log

- Created goal documents from the finalized policy in `docs/node_map_facility_template_contract.md`.
- No runtime code has been modified under this goal yet.

## Failed Attempts

- `cargo test -p game_core map::progression snapshots_and_start::run_snapshot_exposes_current_flow_after_start snapshots_and_start::run_snapshot_exposes_current_node_session_after_node_entry -- --test-threads=1`
  - Result: failed before running tests.
  - Cause: `cargo test` accepts one test filter; multiple positional filters were passed.
  - Fix: rerun focused tests with separate commands or a broader single filter.
  - Revalidation: passed via the separate focused test commands below.
- `cargo test -p game_core map::progression -- --test-threads=1`
  - Result: failed on first run after adding validation tests.
  - Cause: the `forked_map` test fixture had two nodes at the same depth/lane, so the new slot validation failed on duplicate slots before the missing-edge assertion could run.
  - Fix: made the fixture's right branch use lane 1 with matching slot id.
  - Revalidation: passed.
- `cargo test -p game_core -- --test-threads=1`
  - Result: failed after the first broad run.
  - Cause: setup-loss and retreat reentry flows return to `node_confirm` for the same already-current unresolved node. The new `enter_node` rule rejected it because the node was no longer `Available`.
  - Fix: allow idempotent reentry when `current_node_id == node_id` and the node is the unresolved current room (`Unavailable` + `Revealed`).
  - Revalidation: the three failing focused tests passed, then the broad `game_core` suite passed.

## Validation Runs

- `cargo check -p game_core`
  - Result: passed after adding facility DTO/domain fields and current-location progression changes.
- `cargo test -p game_core map_flow -- --test-threads=1`
  - Result: passed.
  - Coverage: existing map flow behavior after current-node/progression DTO changes.
- `cargo check -p game_server`
  - Result: passed.
  - Coverage: game server still compiles with the new map DTO shape.
- `cargo test -p game_core map::progression -- --test-threads=1`
  - Result: passed after fixture correction, explicit edge model addition, and current unresolved reentry test.
  - Coverage: selectability, slot validation, edge endpoint validation, locked leaf validation, and `ForwardOnly` reverse-transit behavior.
- `cargo test -p game_core run_snapshot_exposes_current_flow_after_start -- --test-threads=1`
  - Result: passed.
  - Coverage: `viewing_map` JSON includes `map_template_id`, `slot_id`, `visibility`, `from_node_id`, `to_node_id`, `direction`, and excludes public map selection projections.
- `cargo test -p game_core run_snapshot_exposes_current_node_session_after_node_entry -- --test-threads=1`
  - Result: passed.
  - Coverage: `node_confirm` keeps map DTO and current-node progression field.
- `cargo test -p game_core --test ron_loading -- --test-threads=1`
  - Result: passed.
  - Coverage: live RON data still loads with the map contract changes.
- `cargo test -p game_core live_defense_setup_loss_recovery_returns_to_node_confirm_and_restarts_event_log -- --test-threads=1`
  - Result: passed after idempotent current-node reentry fix.
- `cargo test -p game_core retreat_from_live_defense_battle_reenters_until_attempts_are_exhausted -- --test-threads=1`
  - Result: passed after idempotent current-node reentry fix.
- `cargo test -p game_core consumable_modifier_survives_retreat_reentry_and_expires_when_abnormality_is_resolved -- --test-threads=1`
  - Result: passed after idempotent current-node reentry fix.
- `cargo test -p game_core -- --test-threads=1`
  - Result: passed after the idempotent current-node reentry fix.
  - Coverage: broad `game_core` lib, integration, and doc tests after the Node Map DTO/progression changes.
- `cargo test -p game_server -- --test-threads=1`
  - Result: passed.
  - Coverage: WebSocket/server-facing behavior still compiles and passes tests with the new map DTO shape.
- `cargo fmt`
  - Result: passed.
  - Coverage: Rust formatting after map DTO/progression edits.
- `python3 -m py_compile '/mnt/f/unity projects/ark/docs/probe/unity_ws_smoke_probe.py'`
  - Result: failed for environment reasons before syntax checking.
  - Cause: the external Unity docs folder is read-only for bytecode cache output, so Python could not write `__pycache__`.
  - Fix: validate syntax with `ast.parse` instead of bytecode compilation.
  - Revalidation: `python3 - <<'PY' ... ast.parse(...) ... PY` passed.
- `APP__SERVER__BIND_ADDRESS=127.0.0.1 APP__SERVER__PORT=18083 cargo run -p game_server`
  - Result: failed when launched from `core_enhanced`.
  - Cause: `game_server` expects `config/development.toml` relative to the `game_server` working directory.
  - Fix: reran the same command from `/mnt/f/work/simulator/game_server`.
  - Revalidation: server started on `127.0.0.1:18083`.
- `WS_HOST=127.0.0.1 WS_PORT=18083 python3 '/mnt/f/unity projects/ark/docs/probe/unity_ws_smoke_probe.py'`
  - Result: first sandboxed run failed with `PermissionError: Operation not permitted` when creating a local socket.
  - Fix: reran with elevated local-network permission.
  - Revalidation: passed. The live smoke probe asserted the new Node Map contract for both `viewing_map` and `node_confirm` snapshots.
- `cargo test -p game_core -- --test-threads=1`
  - Result: passed after `cargo fmt`.
  - Coverage: final broad `game_core` validation for lib, integration, and doc tests.
- `cargo test -p game_server -- --test-threads=1`
  - Result: passed after `cargo fmt`.
  - Coverage: final broad `game_server` validation.
- Post-completion correction: remove duplicated graph source
  - Change: removed `MapNode.outgoing`, removed `RunMap::edges_from_outgoing()`, made map generation build `RunMap.edges` directly, and made progression read edge traversal instead of per-node outgoing links.
  - `cargo check -p game_core`: passed.
  - `cargo fmt`: passed.
  - `cargo test -p game_core map::progression -- --test-threads=1`: passed.
  - `cargo test -p game_core generated_map_has_visual_start_connected_to_first_playable_row -- --test-threads=1`: passed.
  - `cargo test -p game_core generated_map_has_single_boss_at_last_depth -- --test-threads=1`: passed.
  - `cargo check -p game_server`: passed.
  - `cargo test -p game_core -- --test-threads=1`: passed.
  - `cargo test -p game_server -- --test-threads=1`: passed.
  - `rg -n "outgoing|edges_from_outgoing" src ../game_server/src -g'*.rs'`: no map graph leftovers; remaining `outgoing` matches are battlefield route-template local variables only.
- Post-correction documentation sync
  - Change: updated external `/mnt/f/unity projects/ark/docs/unity_core_contract.md` Map section so it no longer describes `map_progression.available_node_ids` as a Unity-facing selection source, and so the example uses `id`, `slot_id`, `visibility`, and edge `direction`.
  - Change: updated `docs/README.md` so `docs/node_map_facility_template_contract.md` is listed as a top-level core document and Node Map selection/traversal guidance points to it.
  - `rg -n "map_progression\\.available_node_ids|map\\.available_node_ids|map_progression\\.completed_node_ids|map\\.completed_node_ids|MapNode\\.outgoing|edges_from_outgoing|node_id.*slot_id|state.*cleared|state.*available" '/mnt/f/unity projects/ark/docs/unity_core_contract.md' docs/node_map_facility_template_contract.md '/mnt/f/unity projects/ark/docs/probe/unity_ws_smoke_test_contract.md'`: only intentional "absent / must not expose" references remain in the new Node Map contract and smoke contract.
- Canonical core contract cleanup after Unity implementation review
  - Change: updated `docs/node_map_facility_template_contract.md` so the required DTO example uses current `depth_XX_lane_YY` placeholder slots instead of future authored semantic slot names.
  - Change: removed the permissive `node_id` transitional alias note; current map node DTO public fields are `id`, `slot_id`, `from_node_id`, `to_node_id`, and `direction`.
  - `rg -n "entrance_lab|archive_room|save_room|security_room|exit_elevator|transitional alias|node_id.*alias|template_id|map_slot_id|node_type.*alias|state: \"Hidden\"|\"Hidden\"|available_node_ids|completed_node_ids" docs/node_map_facility_template_contract.md docs/README.md '/mnt/f/unity projects/ark/docs/unity_core_contract.md' '/mnt/f/unity projects/ark/docs/probe/unity_ws_smoke_test_contract.md'`: passed for canonical docs; remaining `available_node_ids` / `completed_node_ids` references are explicit absence rules, and authored semantic slot names appear only as future non-current examples.
- Live JSON contract revalidation after documentation cleanup
  - `APP__SERVER__BIND_ADDRESS=127.0.0.1 APP__SERVER__PORT=18084 cargo run -p game_server` from `/mnt/f/work/simulator/game_server`: first normal sandbox run failed because Cargo could not write `/mnt/f/work/simulator/target/debug/.cargo-lock`; reran with local build/server permission and the server started on `127.0.0.1:18084`.
  - `WS_HOST=127.0.0.1 WS_PORT=18084 python3 '/mnt/f/unity projects/ark/docs/probe/unity_ws_smoke_probe.py'`: passed with exit code 0.
  - Coverage: live `viewing_map`/`node_confirm` JSON included `map_template_id`, `depth_XX_lane_YY` `slot_id`, node `visibility`, edge `from_node_id` / `to_node_id` / `direction`, and absence of public `map_progression.available_node_ids` / `completed_node_ids`.
  - Cleanup: stopped the local server with SIGINT after the probe completed.

During implementation, record every focused test, cargo check, broad test, live RON loading check, and WebSocket probe here with:

- command
- result
- failure cause if any
- fix applied
- revalidation result
