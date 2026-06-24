# Pending Deployment Range Preview Contract Experiments

## Experiment Log

- Implemented core `RequestDeploymentRangePreview` / `DeploymentRangePreview` path using the existing one-facing `deployment_range_preview_dto(...)` helper for all four facings.
- Replaced range preview filtering from walkable-tile filtering to valid-tile filtering by using `Battlefield::in_bounds(...)` in `range_preview.rs`.
- Added server request/result wiring and a special command-result mapping that suppresses the trailing full `state_snapshot` for pending deployment preview.

## Failed Attempts

- `cargo test -p game_core pending_deployment_range_preview_returns_all_facings_without_mutating_battle`
  - Result: failed.
  - Cause: the test asserted that no `owner == Player` checkpoint unit existed after preview, but active battles can already contain a player-owned defense object.
  - Fix: narrowed the assertion to the actual contract: no employee `unit_source` appears, no deployed unit entry appears, no cost changes, and no timeline seq changes.
  - Revalidation: same test passed after the assertion was corrected.

## Validation Runs

- `cargo fmt`
  - Result: passed.
- `cargo test -p game_core range_preview`
  - Result: passed after the test assertion correction above.
  - Coverage: fallback range preview, facing rotation, out-of-bounds filtering, obstacle inclusion, active-skill separation, pending deployment preview command behavior.
- `cargo test -p game_core pending_deployment_range_preview_returns_all_facings_without_mutating_battle`
  - Result: passed.
- `cargo test -p game_server deployment_range_preview_result_does_not_request_state_snapshot`
  - Result: passed.
- `cargo test -p game_server deserializes_pending_deployment_range_preview_request`
  - Result: passed.
- `rg -n "request_deployment_range_preview|DeploymentRangePreview|full `state_snapshot`|pending_deployment_range_preview" ...`
  - Result: command produced useful matches but also shell noise because backticks in the pattern attempted command substitution.
  - Fix: reran with a shell-safe single-quoted pattern that omitted the backtick-wrapped phrase.
  - Revalidation: passed; external Unity docs and probe contain the new pending preview contract.
- `python3 -m py_compile '/mnt/f/unity projects/ark/docs/probe/unity_ws_smoke_probe.py'`
  - Result: failed with read-only `__pycache__` write under the external docs folder.
  - Fix: used `python3 -c "import ast, pathlib; ast.parse(...)"` to validate syntax without writing bytecode.
  - Revalidation: passed.
- `rg -n 'request_deployment_range_preview|DeploymentRangePreview|pending_deployment_range_preview' ...`
  - Result: passed.
  - Coverage: external `unity_core_contract.md`, `core_unity_battle_transport_contract.md`, and smoke probe all mention the new request/result.
- `python3 -c "import ast, pathlib; ast.parse(pathlib.Path('/mnt/f/unity projects/ark/docs/probe/unity_ws_smoke_probe.py').read_text())"`
  - Result: passed.
- `cargo check -p game_core`
  - Result: passed.
- `cargo check -p game_server`
  - Result: passed.
- `cargo test -p game_core`
  - Result: passed.
  - Coverage: full `game_core` unit/integration/doc test suite, including live RON loading tests and the new pending deployment preview behavior.
- `cargo test -p game_server`
  - Result: passed.
  - Coverage: full `game_server` unit/doc test suite, including new request deserialization and no trailing `state_snapshot` mapping.
- `APP__SERVER__BIND_ADDRESS=127.0.0.1 APP__SERVER__PORT=18083 cargo run -p game_server`
  - Result: server started successfully on `127.0.0.1:18083`.
- `WS_PORT=18083 python3 '/mnt/f/unity projects/ark/docs/probe/unity_ws_smoke_probe.py'`
  - Result: failed on first run.
  - Cause: the existing probe helper accepted any `battle_update` collected after sending a command, including live tick updates that could arrive before the matching `command_result`; deploy assertions then read an older checkpoint with no deployed unit.
  - Fix: changed the probe helper to ignore pre-command `battle_update` messages when waiting for a command's battle update.
  - Revalidation: same command passed.
  - Confirmed JSON:
    - pending preview response `result_type: "DeploymentRangePreview"`
    - `payload.facings` contains `down`, `left`, `right`, `up`
    - right-facing fallback cells include the deployment cell and one right cell
    - pending preview message types were `["command_result"]`, with no trailing `state_snapshot`
    - deploy after preview still produced `CommandAccepted` plus `battle_update`
- `python3 -c "import ast, pathlib; ast.parse(pathlib.Path('/mnt/f/unity projects/ark/docs/probe/unity_ws_smoke_probe.py').read_text())"`
  - Result: passed after the final probe edit.
- `rg -n 'request_deployment_range_preview|DeploymentRangePreview|pending_deployment_range_preview' ...`
  - Result: passed after the final probe/doc edit.

Record every focused test, cargo check, broad test, live RON loading test, and WebSocket probe here with:

- command
- result
- failure cause if any
- fix applied
- revalidation result
