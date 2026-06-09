# Admin Debug Commands Experiments

Record implementation attempts, failures, fixes, and validation results for the admin/debug command goal.

## 2026-06-07 - Goal Created

Goal: define a safe long-term implementation plan for dev/test-only admin commands.

Result:

- Created goal memory before code changes.
- Admin commands are explicitly framed as fixture builders, not gameplay rules.
- 1차 scope excludes battle manipulation because it can conflict with game design and should be handled as a separate policy decision.

Next:

- Read server WebSocket message flow.
- Decide whether admin commands live as a top-level WebSocket message or a separate route.

## 2026-06-07 - Core Admin Executor

Attempt:

- Added a dedicated `AdminCommand` enum under `game_core::game::world`.
- Added `GameCore::execute_admin_command` instead of mixing admin behavior into normal player behavior requests.
- Added admin fixture map nodes for support/shop/reward/HQ states so the resulting state remains structurally valid.

Result:

- `admin_enter_support` can create Medical/Rest support sessions, and `admin_enter_maintenance` creates an independent Maintenance session.
- `admin_enter_shop`, `admin_enter_reward`, and `admin_enter_headquarters_contact` use existing map content resolvers/session states.
- Grant commands validate live data before mutation.
- Employee HP/trauma commands reject missing or dead employees.

Validation:

```text
cargo check -p game_core
cargo test -p game_core admin_ -- --nocapture
```

Both passed.

## 2026-06-07 - Server WebSocket Contract

Attempt:

- Added top-level `admin_command` to `PlayerGameClientMessage`.
- Added `admin_result` to `PlayerGameServerMessage`.
- Added `ExecuteAdminCommand` actor message.
- Added WebSocket session gate based on env vars and optional request token.

Result:

- Admin commands are not part of `PlayerBehaviorRequest`.
- Successful admin commands emit `admin_result` and then the latest snapshot.
- Failed admin commands emit the existing `error` message path.
- Production mode rejects admin commands even if env flag and token are present.

Validation:

```text
cargo check -p game_server
cargo test -p game_server admin -- --nocapture
```

Both passed.

## 2026-06-07 - External Unity Contract And Live Probe

Attempt:

- Added `unity_admin_debug_command_contract.md` to the external Unity docs directory.
- Added `unity_admin_debug_command_probe.py` to the external Unity docs directory.
- Linked the admin contract from `unity_ws_smoke_test_contract.md`.
- Ran a live `game_server` with `ENABLE_ADMIN_COMMANDS=true ADMIN_COMMAND_TOKEN=dev`.

Failure:

- First probe run expected Medical `target_candidates` to be objects with `employee_uuid`.
- Runtime snapshot uses `Vec<Uuid>`, so the JSON shape is an array of UUID strings.

Fix:

- Updated the probe to treat `target_candidates` as a string array.
- Updated the admin contract to document that Medical target candidates are employee UUID strings.

Validation:

```text
ENABLE_ADMIN_COMMANDS=true ADMIN_COMMAND_TOKEN=dev cargo run -p game_server
ADMIN_COMMAND_TOKEN=dev WS_PORT=8081 python3 "/mnt/f/unity projects/ark/docs/unity_admin_debug_command_probe.py"
```

Passed. The probe verified Maintenance, grants, Medical target setup, Shop, Reward,
HeadquartersContact, and inventory dump.

## 2026-06-07 - Completion Audit

Audit:

- Goal memory files exist and were updated throughout the goal.
- Admin namespace is top-level `admin_command`, separate from gameplay `command`.
- Admin commands are rejected without `ENABLE_ADMIN_COMMANDS`, rejected in production, and token-gated when `ADMIN_COMMAND_TOKEN` is set.
- Core fixture commands cover dump, support/shop/reward/headquarters entry, grants, HP/trauma, and enkephalin.
- Server sends `admin_result` followed by `state_snapshot` on success and `error` on failure.
- External Unity docs contain the admin contract and Python probe.
- The live probe verified non-combat fixture creation and snapshot shapes against a running server.

Final validation:

```text
cargo test -p game_core admin_ -- --nocapture
cargo test -p game_server admin -- --nocapture
cargo check -p game_core
cargo check -p game_server
```

All passed. No unresolved policy questions were found during the completion audit.
