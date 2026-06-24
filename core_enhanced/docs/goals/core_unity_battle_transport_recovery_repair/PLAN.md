# Core Unity Battle Transport Recovery Repair Plan

## Objective

Repair the remaining core/server -> Unity live battle transport issues after the `battle_update` and `battle_setup_snapshot` contract work.

The target end state is:

- battle commands acknowledge acceptance without sending a full `state_snapshot` as a battle state source.
- live battle presentation changes flow through `battle_update` / `battle_resync` only.
- setup loss or reconnect during an active battle follows the documented offline-game policy: discard the active battle, return to node-confirm/pre-entry state, and start a new battle from `confirm_enter_node`.
- catch-up resync validates `known_seq` instead of accepting impossible future cursors.
- core battle timeline state is not permanently coupled to a single actor-wide transport cursor.

## Goal Working Method

Maintain these files throughout the goal:

- `docs/goals/core_unity_battle_transport_recovery_repair/PLAN.md`
- `docs/goals/core_unity_battle_transport_recovery_repair/EXPERIMENTS.md`
- `docs/goals/core_unity_battle_transport_recovery_repair/EXPERIMENT_NOTES.md`

`PLAN.md` records scope, implementation plan, completion conditions, stop conditions, and verification commands.
`EXPERIMENTS.md` records attempts, failures, fixes, and verification results.
`EXPERIMENT_NOTES.md` records source-of-truth findings, policy questions, and follow-up candidates.

Update these files as the implementation evolves. Do not leave them as a stale initial plan.

## Engineering Principles

- Prefer long-term transport ownership boundaries over patching individual tests.
- Do not reintroduce command-result battle state payloads.
- Do not let full `state_snapshot` compete with `battle_update.checkpoint` for live battle actor/HUD state.
- Do not implement a same-battle hard setup resync compatibility layer.
- Treat `need_setup: true` and active-battle reconnect/setup loss as battle abandonment and node pre-entry recovery.
- Keep catch-up resync read-only and cursor-safe.
- Separate gameplay timeline state from transport/session delivery state where practical.
- Tests must lock Unity-facing message order, DTO shape, recovery behavior, invalid cursor handling, and real gameplay flow.

## Source Of Truth

Confirm facts in this order before changing code:

1. Runtime/server code:
   - `../game_server/src/game/player_game_actor/session.rs`
   - `../game_server/src/game/player_game_actor/handlers.rs`
   - `../game_server/src/game/player_game_actor/messages.rs`
   - `../game_server/src/game/player_game_actor/state.rs`
   - `src/game/world.rs`
   - `src/game/world/combat.rs`
   - `src/game/world/state.rs`
   - `src/game/world/snapshot.rs`
   - `src/game/behavior.rs`
   - `src/game/resources.rs`
   - map/node progression code used by `confirm_enter_node`
2. Live data and flow:
   - live DefenseRoute encounters and node entry flow
   - live battle command flow for deploy, withdraw, activate skill, pause/resume/speed, retreat, request resync
3. Unity-facing contracts:
   - `/mnt/f/unity projects/ark/docs/core_unity_battle_update_contract.md`
   - `/mnt/f/unity projects/ark/docs/core_unity_battle_setup_snapshot_contract.md`
   - `/mnt/f/unity projects/ark/docs/unity_core_contract.md`
   - `/mnt/f/unity projects/ark/docs/unity_client_implementation_goal.md`
4. Policy docs:
   - `docs/game_rulebook.md`
   - `docs/code_documentation_sync_guidelines.md`
   - `docs/refactor_preparation_plan.md`

Do not trust documentation blindly. If runtime code shows a safer shape, record the evidence in `EXPERIMENT_NOTES.md`. If the change affects Unity-facing DTO shape, save data, node retry policy, rewards/consumption timing, or UX semantics beyond the documented reconnect policy, stop and ask the user.

## In Scope

- Stop sending full `state_snapshot` after successful battle commands that already emit `battle_update` or `battle_resync`.
- Keep non-battle commands and admin commands using `state_snapshot` where appropriate.
- Add explicit server/core behavior for `request_battle_resync { need_setup: true }` according to the latest policy:
  - discard current active battle,
  - return to the node-confirm/pre-entry state for the same node when safe,
  - send a `state_snapshot` whose `game_state_context.type` is `node_confirm`,
  - require Unity to call `confirm_enter_node` to start a new battle.
- Apply the same recovery policy when a new session attaches while the actor is already in an active battle, if runtime code confirms attach is the correct reconnect boundary.
- Validate catch-up `known_seq` / `since_seq`:
  - valid values produce `battle_resync` with nested `battle_update`;
  - `known_seq > latest presentation seq` must not produce a checkpoint at an impossible future seq.
- Reduce or remove actor-wide cursor coupling if feasible in this goal:
  - preferred direction: keep core timeline immutable and move pushed cursor ownership toward transport/session;
  - acceptable first slice: preserve existing live tick cursor for single-session delivery but document and test the recovery policy that prevents stale reconnect usage.
- Add focused tests for message order, no battle-command snapshot, setup-loss rollback, invalid cursor handling, and reconnect behavior.
- Update external canonical docs only if implementation differs from the documented policy.

## Out Of Scope

- Unity client implementation.
- Same-battle hard setup resync / scene reconstruction.
- Long-lived dual schema for old snapshot-based battle command handling.
- Save data migration unless runtime code proves node rollback cannot be represented without one.
- Gameplay balance changes.
- Live RON schema changes unless runtime code proves the recovery policy cannot be implemented without them.
- Rewriting timeline event names or battle event semantics unrelated to cursor/recovery repair.
- Multi-client spectator support beyond avoiding worse actor-wide cursor coupling.

## Policy Decisions To Confirm Before Implementation

Stop and ask the user if code reading shows any of these are required:

- Recovering from an active battle would refund or duplicate node-entry resource costs, rewards, abnormality attempts, consumable duration, or other progression effects.
- The current node cannot be restored to `NodeConfirm` without choosing between replaying or deleting gameplay state.
- Reconnect should keep battle progress instead of discarding it.
- `need_setup: true` should return a same-battle `{ setup, update }` payload after all.
- Invalid `known_seq` should be a top-level `error` vs setup-loss rollback. If no policy input is required, prefer a clear error for catch-up resync and rollback only for `need_setup: true` / attach setup loss.
- Battle command failures should change from top-level `error` to `command_result { ok: false }`.
- Per-session cursor support requires broad actor/session architecture changes beyond this transport repair.

## Implementation Plan

1. Re-read command and attach transport paths.
   - Confirm exactly which commands produce battle side messages.
   - Record current message order and snapshot sources in `EXPERIMENT_NOTES.md`.

2. Split command execution snapshot policy.
   - Extend `CommandExecutionResult` or its mapping with an explicit `state_snapshot_policy`.
   - For battle side-message commands, send:
     - `command_result`,
     - `battle_setup_snapshot` if this is battle start,
     - `battle_update` or `battle_resync`.
   - Do not send full `state_snapshot` after those battle side messages.
   - Preserve full `state_snapshot` for non-battle commands.

3. Add tests for command message order.
   - Deploy/withdraw/activate skill/pause/resume/speed/request catch-up resync should not be followed by `state_snapshot`.
   - `confirm_enter_node` should still produce `command_result`, `battle_setup_snapshot`, and `battle_update`.
   - Non-battle commands that rely on snapshot should still receive one.

4. Add latest seq validation.
   - Find or add a core method for latest presentation seq.
   - Validate `RequestBattleState { since_seq }` or the server `request_battle_resync` mapping before building a DTO.
   - Ensure `known_seq > latest_seq` returns a clear error and never creates `checkpoint.at_seq == known_seq`.
   - Keep `known_seq == latest_seq` valid with an empty delta and checkpoint at latest seq.

5. Implement setup-loss recovery.
   - Read node-confirm / confirm-enter-node transition code before editing.
   - Add a core behavior or server-side flow that discards `state.active_battle` and transitions from `InBattle` back to the correct `NodeConfirm`.
   - Ensure battle UUID changes on the next `confirm_enter_node`.
   - Do not preserve old event queue or old setup snapshot.

6. Wire `request_battle_resync { need_setup: true }`.
   - Replace `hard_battle_resync_not_supported` with the setup-loss recovery behavior.
   - Return a state snapshot showing node-confirm/pre-entry state.
   - Avoid sending `battle_resync` for this path.

7. Decide attach behavior from code evidence.
   - If attach during active battle is the reconnect boundary, apply setup-loss recovery before sending the initial `state_snapshot`.
   - If attach cannot safely mutate state, record why and keep explicit `need_setup: true` as the recovery command.

8. Reduce actor-wide cursor debt.
   - Prefer moving live push cursor out of `ActiveBattleSession` if the change is contained.
   - If not contained, document why the node-confirm recovery policy makes reconnect safe enough for this goal and leave per-session cursor as a follow-up with tests preventing stale attach playback.

9. Update tests and docs.
   - Replace legacy snapshot-after-battle-command expectations.
   - Add regression tests for invalid future `known_seq`.
   - Add recovery tests around reconnect/setup loss.
   - Update external docs only if runtime implementation changes the already documented contract.

10. Verification.
   - Run focused tests after each slice.
   - Finish with broader `cargo check` and package tests.

## Test Requirements

Tests should cover:

- Battle command responses are acknowledgement-only plus battle side messages; no full `state_snapshot` follows.
- `confirm_enter_node` live battle start still sends `battle_setup_snapshot` before first `battle_update`.
- Catch-up `request_battle_resync { need_setup: false, known_seq }` returns `battle_resync` for valid known seq.
- `known_seq == latest_seq` returns an empty delta with `to_seq == after_seq == latest_seq`.
- `known_seq > latest_seq` fails or triggers the chosen policy without producing an impossible checkpoint seq.
- `request_battle_resync { need_setup: true }` discards active battle and returns node-confirm/pre-entry state.
- Re-entering the node starts a fresh battle execution with fresh setup/update messages and a fresh presentation seq. Current runtime keeps `battle_uuid` tied to map node id, so same-node re-entry may reuse the same `battle_uuid`.
- Reconnect/attach during active battle follows the same documented setup-loss policy if implemented.
- Non-battle commands still send state snapshots where the client needs them.
- Existing live tick behavior still sends `battle_update` without full `state_snapshot`.

## Implementation Status

Updated 2026-06-16:

- Added `PlayerBehavior::RecoverBattleSetupLoss` and `ActionKind::RecoverBattleSetupLoss`.
- `request_battle_resync { need_setup: true }` now maps to setup-loss recovery instead of `hard_battle_resync_not_supported`.
- Setup-loss recovery discards `state.active_battle`, clears active node battle content, rebuilds the current node preview/session, transitions to `NodeConfirm`, and returns a normal `NodePreview` result plus state snapshot.
- Attach/reconnect while the actor is already in `InBattle` now stops live ticking, runs setup-loss recovery, and returns a `node_confirm` state snapshot.
- Battle command result mapping now carries an explicit `send_state_snapshot` policy.
- Battle side-message commands send `command_result` plus `battle_setup_snapshot` / `battle_update` / `battle_resync` as appropriate, without a trailing full `state_snapshot`.
- Non-battle commands and admin commands still send full state snapshots.
- Catch-up battle state requests validate `since_seq` / `known_seq`; future seq requests return `GameError::InvalidBattleResyncSeq` and server error code `invalid_battle_resync_seq`.
- Existing actor-wide `last_pushed_timeline_seq` remains in `ActiveBattleSession`; reconnect safety is bounded by setup-loss recovery instead of per-session cursor support in this slice.
- Runtime evidence showed `battle_uuid` is currently map-node scoped. External Unity docs were updated to say same-node re-entry starts a fresh setup/timeline but may reuse `battle_uuid`.

No same-battle hard setup resync, fallback path, or dual schema was added.

## Verification Commands

Start focused:

```text
cargo test -p game_server player_game_actor -- --nocapture
cargo test -p game_core battle_setup_snapshot -- --nocapture
cargo test -p game_core battle_update -- --nocapture
```

Then run broader checks:

```text
cargo check -p game_core
cargo check -p game_server
cargo test -p game_core
cargo test -p game_server
```

If live RON or node entry flow changes:

```text
cargo test -p game_core --test ron_loading -- --nocapture
```

If WebSocket probe contract is updated:

```text
APP_SERVER__BIND_ADDRESS=127.0.0.1 APP_SERVER__PORT=18082 cargo run -p game_server
WS_PORT=18082 python3 "/mnt/f/unity projects/ark/docs/unity_ws_smoke_probe.py"
```

## Completion Conditions

- Battle command path no longer sends full `state_snapshot` after `battle_update` / `battle_resync`.
- Non-battle command snapshot behavior remains intact.
- `known_seq` / `since_seq` validation prevents impossible future checkpoint seq.
- `request_battle_resync { need_setup: true }` implements the latest node pre-entry recovery policy or the goal stops with a concrete policy blocker.
- Active-battle attach/reconnect behavior is either repaired or explicitly documented with a safe user-visible fallback and tests.
- Actor-wide cursor debt is reduced or bounded by the recovery policy and recorded as a follow-up.
- No same-battle hard setup resync compatibility layer is added.
- Unity-facing message order is tested.
- External docs remain consistent with implementation.
- Focused and broad verification commands pass, or failures are recorded with cause and next action.
- 사용자와 의논하여 정해야 할 정책이 있을 경우 goal을 종료한다.

## Final Verification Run

Executed on 2026-06-16:

```text
cargo fmt -p game_core -p game_server
cargo check -p game_core
cargo check -p game_server
cargo test -p game_core live_defense_ -- --nocapture
cargo test -p game_server player_game_actor -- --nocapture
cargo test -p game_core
cargo test -p game_server
```

Results:

- All commands passed after updating the action scheduler test for the new setup-loss action.
- `cargo test -p game_core` passed 459 lib tests plus integration suites: `live_item_skill_activation` 3, `live_skill_catalog_audit` 3, `ron_loading` 16, `skill_refactor_validation` 10, `skill_test_suite` 14.
- `cargo test -p game_server` passed 20 tests.
- Commands continue to print the pre-existing workspace warning: `/mnt/f/work/simulator/auth_server/Cargo.toml: unused manifest key: env`.

## Stop Conditions

- Implementing setup-loss recovery would alter resource/reward/consumable/progression semantics not already covered by policy.
- Restoring node-confirm state requires guessing deleted or ambiguous node session data.
- Save migration becomes necessary.
- Unity-facing DTO shape must change beyond the current contract.
- A long-lived dual schema appears necessary.
- Per-session cursor repair requires a broad server actor redesign that would obscure this goal's transport fixes.
- Existing tests protect behavior that conflicts with the new contract but the user-visible replacement behavior is unclear.
- 사용자와 의논하여 정해야 할 정책이 있을 경우 goal을 종료한다.
