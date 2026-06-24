# Experiments

## Log

| Date | Scope | Attempt | Result | Follow-up |
| --- | --- | --- | --- | --- |
| 2026-06-22 | Goal setup | Defined Unity/server contract implementation scope. | Not started. | Inventory DTO definitions and WebSocket mappings. |
| 2026-06-22 | Battle resync contract | Removed dormant `BattleResync.setup` from server message DTO and serialization tests. | Success. `BattleResync` now carries only `battle_uuid` and nested `update`; tests no longer expect a null `setup` field. | Keep setup recovery represented by `BattleSetupSnapshot` plus battle updates. |
| 2026-06-22 | Server test fixtures | Updated player-game actor fixtures to provide policy-valid starter equipment and baseline skill fragments. | Success. Server tests now satisfy starter loadout data validation instead of relying on empty legacy defaults. | Reuse helper for future actor tests needing `GameDataBuilder::empty()`. |
| 2026-06-22 | Shared command DTO | Made core `PlayerBehavior` the tagged snake_case command payload and removed server-local `PlayerBehaviorRequest`. Moved `battle_resync` response selection into `battle_response` transport metadata on the WebSocket command envelope. | Success. Gameplay payload now has one shared typed source; `request_battle_resync` and `need_setup` were removed from normal command payloads. | Continue with typed run snapshot DTO migration. |
| 2026-06-22 | Typed run snapshot DTO | Added `RunSnapshotDto` and `GameStateContextDto`; `GameCore::get_run_snapshot_json` now serializes `get_run_snapshot_dto()`. | Success. Root snapshot and game-state context are typed while preserving existing JSON shape. | Migrate nested sections next. |
| 2026-06-22 | Nested run snapshot DTOs | Converted `selected_event`, `inventory`, `roster`, and `roster_order` from raw JSON builders into typed snapshot DTOs while retaining JSON wrapper methods for existing admin/test callers. | Success. `RunSnapshotDto` no longer contains `serde_json::Value`; public JSON methods serialize typed DTOs. | Continue with BehaviorResult boundary and combat result timeline attachment ownership. |
| 2026-06-22 | Combat result timeline attachment | Added core `CombatResultTimelineAttachmentDto` and changed `GameCore` to expose typed attachment meaning while the server keeps gzip/base64 compression and `compressed_timeline` transport serialization. | Success. Existing combat result snapshot JSON contract is preserved, but core now owns the attachment meaning instead of exposing a raw `(winner, Timeline)` tuple. | Continue with BehaviorResult boundary. |
| 2026-06-22 | BehaviorResult boundary | Added core domain result contracts (`RunCommandResult`, `NodeCommandResult`, `InventoryCommandResult`, `RewardCommandResult`, `BattleCommandResult`, etc.) and changed the server to consume `BehaviorCommandResultContract` rather than hand-assembling every domain payload. | Success. Unity-facing `result_type`/payload JSON remains stable; battle results still map to `CommandAccepted` plus side battle messages, and read-only deployment preview still suppresses trailing state snapshot. | Check battle record export policy and then run broader validation. |
| 2026-06-22 | Battle records export | Verified `RunState::record_battle` writes `battle_records/run_<seed>/*.json` without an environment gate and focused combat tests assert file creation plus JSON validity. | Success. Policy is already implemented as always-on debugging artifact and not promoted to gameplay/replay SoT. | Run final focused/broad validation for the subgoal. |

## Failed Approaches

- `perl -pi` mechanical replacement against `../game_server/src/game/player_game_actor/messages.rs` failed because the sandbox could not create an in-place temp file outside this workspace root. Used `apply_patch` instead.

## Validation Commands

- `cargo check -p game_server` - passed after `BattleResync.setup` removal and starter fixture repair.
- `cargo test -p game_server battle_resync -- --test-threads=1` - initially failed because the server fixture used empty starter loadouts; passed after adding policy-valid starter equipment/fragments.
- `cargo check --lib` - passed after shared command DTO migration.
- `cargo check -p game_server` - passed after shared command DTO migration.
- `cargo test -p game_server player_game_actor::messages -- --test-threads=1` - passed, 8 tests.
- `cargo test -p game_server battle_resync -- --test-threads=1` - passed, 2 tests.
- `cargo test --lib behavior -- --test-threads=1` - passed, 1 filtered test.
- `cargo test -p game_server player_game_actor -- --test-threads=1` - passed, 22 tests.
- `cargo test --lib snapshots_and_start -- --test-threads=1` - passed, 8 tests.
- `cargo test --lib combat:: -- --test-threads=1` - passed, 40 tests.
- `cargo test --lib equipment:: -- --test-threads=1` - passed, 23 tests.
- `cargo check --lib` - passed after nested run snapshot DTO migration.
- `cargo check -p game_server` - passed after nested run snapshot DTO migration.
- `cargo check --lib` - passed after combat result timeline attachment ownership migration.
- `cargo check -p game_server` - passed after combat result timeline attachment ownership migration.
- `cargo test -p game_server player_game_actor::handlers::tests::finished_live_battle_tick_pushes_combat_result_snapshot_after_final_update -- --test-threads=1` - passed, 1 test.
- `cargo test --lib combat:: -- --test-threads=1` - passed, 40 tests.
- `cargo check --lib` - passed after BehaviorResult domain contract migration.
- `cargo check -p game_server` - passed without new warnings after BehaviorResult domain contract migration.
- `cargo test -p game_server player_game_actor::state -- --test-threads=1` - passed, 3 tests.
- `cargo test -p game_server player_game_actor::handlers::tests::finished_live_battle_tick_pushes_combat_result_snapshot_after_final_update -- --test-threads=1` - passed, 1 test.
- `cargo test --lib snapshots_and_start -- --test-threads=1` - passed, 8 tests after final subgoal changes.
- `cargo test --lib combat:: -- --test-threads=1` - passed, 40 tests after final subgoal changes.
- `cargo test -p game_server player_game_actor -- --test-threads=1` - passed, 22 tests after final subgoal changes.
