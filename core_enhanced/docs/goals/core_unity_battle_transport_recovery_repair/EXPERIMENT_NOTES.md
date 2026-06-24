# Core Unity Battle Transport Recovery Repair Experiment Notes

This file records source-of-truth findings, implementation judgments, policy questions, and follow-up candidates.

## 2026-06-16 Initial Findings

The following findings came from reading runtime/server code before creating this goal.

### Command Path Still Sends Full StateSnapshot

Source:

- `../game_server/src/game/player_game_actor/session.rs`
- `../game_server/src/game/player_game_actor/handlers.rs`
- `../game_server/src/game/player_game_actor/state.rs`
- `src/game/world/snapshot.rs`

Finding:

- Successful player commands currently send:
  - command response,
  - side messages,
  - full `state_snapshot`.
- Battle commands such as deploy, withdraw, activate skill, pause/resume/speed, and catch-up resync can therefore emit `battle_update` / `battle_resync` and then a full `state_snapshot`.
- The full snapshot includes `game_state_context.type == "in_battle"` with `deployment`, `playback`, and `last_pushed_timeline_seq`.

Judgment:

- This is a real transport ownership problem, not just duplicate data.
- Unity should not need to treat full snapshot as a battle actor/HUD state source after a battle command.
- Fix should be explicit in command execution result policy, not a client-side ignore rule.

### Attach / Reconnect During Active Battle Is Weak

Source:

- `../game_server/src/game/player_game_actor/session.rs`
- `../game_server/src/game/player_game_actor/handlers.rs`

Finding:

- Session attach sends `Authed`, then full `StateSnapshot`, then starts live battle ticking.
- Attach does not send `battle_setup_snapshot`.
- Attach does not send a same-battle `battle_resync`.
- Existing live tick cursor is stored in active battle state, not per session.

Judgment:

- Reconnecting into an active battle cannot reliably reconstruct the Unity scene from the current attach flow.
- The latest documented policy avoids same-battle hard resync: setup loss/reconnect should discard the active battle and return to node pre-entry.
- Implementation must verify whether attach is the right place to apply that policy or whether explicit `request_battle_resync { need_setup: true }` should be the first supported path.

### need_setup True Is Still Rejected In Code

Source:

- `../game_server/src/game/player_game_actor/messages.rs`

Finding:

- `request_battle_resync { need_setup: true }` currently returns `hard_battle_resync_not_supported`.

Judgment:

- The wording is now stale relative to the latest policy.
- The goal is not to implement same-battle hard resync.
- The correct behavior is setup-loss recovery: discard active battle, return to node-confirm/pre-entry state, and let Unity start a fresh battle by calling `confirm_enter_node`.

### known_seq / since_seq Is Not Validated

Source:

- `src/game/world/state.rs`
- `src/game/world/combat.rs`

Finding:

- `battle_update_dto_after(last_seen_seq, roster)` uses the client-provided seq as `after_seq`.
- If no events are found, `to_seq` becomes `after_seq`.
- Therefore `known_seq > latest actual seq` can produce `checkpoint.at_seq == known_seq`, even though that seq does not exist.

Judgment:

- This must be fixed before Unity relies on catch-up resync.
- `known_seq == latest_seq` should remain valid and produce an empty delta.
- `known_seq > latest_seq` should return a clear error or another documented recovery path, but must never mint a fake checkpoint seq.

### Actor-Wide Cursor Remains A Debt

Source:

- `src/game/world/state.rs`
- `src/game/world/snapshot.rs`

Finding:

- `ActiveBattleSession.last_pushed_timeline_seq` stores live pushed cursor inside core battle session state.
- `drain_battle_update_dto()` mutates that cursor.
- Full snapshots expose this cursor in `game_state_context.in_battle`.

Judgment:

- This is acceptable only for the current single Unity session happy path.
- It is structurally wrong for reconnect/session replacement/debug clients because delivery state is mixed into gameplay state.
- Preferred long-term direction is to move pushed cursor ownership toward transport/session.
- If that is too broad for this goal, the setup-loss recovery policy must prevent stale reconnect playback from depending on this cursor.

## Initial Implementation Hypotheses

- Add an explicit snapshot policy to `CommandExecutionResult`, rather than making `session.rs` infer from message variants.
- Treat battle side messages as authoritative for battle command state changes; no full snapshot should be sent after them.
- Keep `battle_setup_snapshot` only for fresh battle entry, not for setup-loss recovery of the same battle.
- Implement `need_setup: true` as a state recovery command that returns node-confirm snapshot, not as `battle_resync`.
- Add a core method for latest battle event seq or presentation seq and validate `since_seq` before building `LiveBattleUpdateDto`.
- Consider renaming code-level `hard_battle_resync_not_supported` terminology during implementation to avoid policy drift.

## Policy Questions If Encountered

Stop and ask the user if any of these become necessary:

- Should active-battle reconnect automatically abandon the current battle, or should only explicit `request_battle_resync { need_setup: true }` do it?
- If battle abandonment happens, should any node-entry costs or consumable duration changes be restored?
- If current node session data is missing, should recovery return to map view instead of node-confirm?
- Should invalid future `known_seq` be a top-level `error`, or should it force setup-loss recovery?
- Should battle command rejection shape change from top-level `error` to `command_result { ok: false }`?
- Is per-session cursor required now, or can it remain a bounded follow-up after reconnect policy is implemented?

## Follow-Up Candidates

These are not in scope unless required to finish the transport repair safely:

- Full per-session battle stream cursor support.
- Separate battle instance identity from map node id if Unity needs `battle_uuid` to change on same-node re-entry. Runtime currently uses map node id as `battle_uuid`, so setup-loss recovery starts a fresh setup/timeline while keeping the same UUID for the same node.
- WebSocket smoke probe for reconnect/setup-loss recovery.
- Battle abandonment UX copy and loading state on the Unity side.
- Dedicated telemetry/logging for invalid `known_seq` and setup-loss recovery.
- Removing `last_pushed_timeline_seq` from full state snapshots after transport cursor ownership is moved out of core.

## 2026-06-16 Implementation Findings

- `CommandExecutionResult` now carries `Option<Value>` for `state_snapshot`, and `CommandResultMapping` carries `send_state_snapshot`.
- The server no longer infers snapshot policy by inspecting message variants in `session.rs`; mapping owns the policy.
- Battle results mapped through `battle_update` / `battle_resync` set `send_state_snapshot: false`.
- Non-battle command results set `send_state_snapshot: true`.
- `request_battle_resync { need_setup: true }` now maps to `PlayerBehavior::RecoverBattleSetupLoss`.
- `RecoverBattleSetupLoss` deliberately does not reuse `RetreatBattle`; retreat has attempt/rumor/exhaustion semantics that should not be part of reconnect recovery.
- Setup-loss recovery uses the current active battle's node id, rebuilds the node preview/session from map data, clears `active_battle`, clears `active_node_content`, and transitions to `NodeConfirm`.
- Active-battle attach is treated as a reconnect/setup-loss boundary and performs the same recovery before returning the attach snapshot.
- `known_seq > latest_timeline_seq` now returns `GameError::InvalidBattleResyncSeq` and server error code `invalid_battle_resync_seq`.
- `known_seq == latest_timeline_seq` remains valid and returns an empty event delta at the latest seq.
- Existing `last_pushed_timeline_seq` remains actor/core-owned. This goal bounds reconnect risk by forcing reconnect/setup loss to node-confirm; it does not complete the broader per-session cursor refactor.
- Runtime code sets `ActiveBattleSession.battle_uuid` from the selected map node UUID. Changing that to a new per-attempt battle instance id would require separating node id from battle id across active battle, result, retreat, and snapshot flows. The current implementation keeps node-scoped `battle_uuid` and treats fresh setup/timeline as the re-entry boundary.
