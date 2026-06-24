# Unity/server-facing DTO/Contract Refactor Audit

## Scope

Canonical refactor 기준은 `docs/refactor_preparation_plan.md`다.

읽은 범위:

- Runtime code: `src/game/behavior.rs`, `src/game/world/snapshot.rs`, `src/game/world/state.rs`, `src/game/world/combat.rs`, `src/game/world/admin.rs`, `src/game/world/admin/catalog.rs`.
- Server transport code: `../game_server/src/game/player_game_actor/messages.rs`, `../game_server/src/game/player_game_actor/state.rs`, `../game_server/src/game/player_game_actor/handlers.rs`, `../game_server/src/game/player_game_actor/session.rs`.
- Unity-facing docs: `/mnt/f/unity projects/ark/docs/unity_core_contract.md`, `/mnt/f/unity projects/ark/docs/core_unity_battle_transport_contract.md`.
- Tests touched by contract surface: `../game_server/src/game/player_game_actor/messages.rs`, `../game_server/src/game/player_game_actor/state.rs`, `../game_server/src/game/player_game_actor/handlers.rs`, `src/game/world/tests/combat.rs`, `src/game/world/admin/tests.rs`, `tests/live_skill_catalog_audit.rs`.

## Current Structure

- Core player input source is `PlayerBehavior` in `src/game/behavior.rs:263`; server input envelope uses separate `PlayerBehaviorRequest` in `../game_server/src/game/player_game_actor/messages.rs:51`.
- Server converts `PlayerBehaviorRequest` to `PlayerBehavior` manually in `../game_server/src/game/player_game_actor/messages.rs:167` and adds transport-specific resync handling in `try_into_player_behavior()` at `../game_server/src/game/player_game_actor/messages.rs:311`.
- Core command output source is `BehaviorResult` in `src/game/behavior.rs:826`; server maps it to `command_result`/side messages in `../game_server/src/game/player_game_actor/state.rs:202`.
- Live battle setup/update/checkpoint DTOs are typed in `src/game/behavior.rs:534`, `src/game/behavior.rs:550`, `src/game/behavior.rs:635`, and `src/game/behavior.rs:642`.
- Live battle transport messages flatten or wrap those DTOs in `PlayerGameServerMessage` at `../game_server/src/game/player_game_actor/messages.rs:349`.
- Run snapshot is built as `serde_json::Value` in `src/game/world/snapshot.rs:75`; `skill_catalog` is an exception that serializes from typed `SkillCatalogDto` at `src/game/world/snapshot.rs:126` and `src/game/world/snapshot.rs:231`.
- Server adds `selected_event.compressed_timeline` after calling core snapshot in `../game_server/src/game/player_game_actor/handlers.rs:176`.
- Admin output is `AdminCommandOutput { result_type, payload: Value }` in `src/game/world/admin.rs:18`; grant catalog payload has typed internal DTOs in `src/game/world/admin/catalog.rs:87`.

## Source Of Truth

- Live battle runtime state source: `ActiveBattleState` and battle timeline in `src/game/world/state.rs`; Unity-facing transport source is `battle_setup_snapshot`, `battle_update.events_delta`, `battle_update.checkpoint`.
- Unity contract explicitly says battle `command_result` is not a state source; actual battle changes come from `battle_update` (`core_unity_battle_transport_contract.md:34`, `:60`, `:263`, `:470`; `unity_core_contract.md:784`, `:840`, `:2256`).
- Non-battle UI source is final `state_snapshot`: `game_state_context`, `allowed_actions`, `selected_event`, `inventory`, `roster` (`unity_core_contract.md:2256`).
- Skill catalog source is typed `SkillCatalogDto`; Unity docs explicitly say not to extend it through ad-hoc JSON key assembly (`unity_core_contract.md:2000`).

## Refactor Candidates

### 1. `PlayerBehaviorRequest` and `PlayerBehavior` are manually mirrored

Evidence:

- Core request enum is `PlayerBehavior` (`src/game/behavior.rs:263`).
- Server request enum is `PlayerBehaviorRequest` (`../game_server/src/game/player_game_actor/messages.rs:51`).
- Conversion is a large manual match (`../game_server/src/game/player_game_actor/messages.rs:167`).
- `request_battle_resync` is server-only and maps to either `RequestBattleState` or `RecoverBattleSetupLoss` (`../game_server/src/game/player_game_actor/messages.rs:263`, `:311`).

Why it matters:

- Most command shape data is duplicated across core and server. Adding/removing a gameplay command requires updating both enums, the conversion, deserialization tests, action gating, and `BehaviorResult` mapping.
- This is source-of-truth duplication, but the server enum also owns transport-specific casing and `request_battle_resync`, so blindly deleting it would mix transport policy into core.

Candidate:

- Keep a server transport envelope, but reduce duplicated command payload definitions by introducing a shared typed command DTO layer in core, or by deriving server request variants from core where transport-only commands are explicitly marked.
- Add a coverage test that every intended public `PlayerBehavior` has a server request mapping or an explicit "core-internal only" reason.

Policy:

- WebSocket request shape is Unity-facing. 사용자와 정책 논의 필요.

### 2. `BehaviorResult` to server `command_result` mapping is another manual contract table

Evidence:

- Core result enum lives at `src/game/behavior.rs:826`.
- Server maps every variant to a string and payload in `behavior_result_payload()` (`../game_server/src/game/player_game_actor/state.rs:280`).
- Battle results are rejected from legacy command payload transport (`../game_server/src/game/player_game_actor/state.rs:640`).
- Tests cover battle rejection and a few special mappings (`../game_server/src/game/player_game_actor/state.rs:758`, `:767`, `:801`), but not full variant coverage.

Why it matters:

- `BehaviorResult` is the source of command outcome data, but the server owns the public `result_type` strings and payload JSON assembly.
- A newly added result variant can compile only after matching, but it can still choose the wrong transport class, state snapshot behavior, or result type naming.

Candidate:

- Add an explicit transport classification helper close to `BehaviorResult`, for example `CommandPayload`, `BattleSideMessage`, `ReadOnlyPreview`, rather than spreading this policy across server functions.
- Add tests that all battle results use side-message transport, all read-only preview results suppress trailing full snapshots only when intended, and all non-battle results preserve documented `result_type`.

Policy:

- `result_type` strings and payload shapes are Unity-facing. 사용자와 정책 논의 필요.

### 3. Live battle side-message rule is correct but server-owned

Evidence:

- `behavior_result_to_command_result()` turns any battle update result into `CommandAccepted` plus `battle_update`/`battle_resync`, with no trailing state snapshot (`../game_server/src/game/player_game_actor/state.rs:202`).
- `battle_setup_snapshot_from_behavior_result()` only emits setup when `BattleAdvanced` carries `battle_setup_snapshot` (`../game_server/src/game/player_game_actor/state.rs:268`).
- Server live tick pushes final `battle_update` first, then `combat_result` state snapshot when finished (`../game_server/src/game/player_game_actor/handlers.rs:261`, `:275`).
- Unity contract requires this ordering (`core_unity_battle_transport_contract.md:68`, `:97`, `:263`, `:294`).

Why it matters:

- The important contract is implemented in server mapping rather than represented as first-class core output. Core returns `BehaviorResult::BattleAdvanced` with setup/update fields; server interprets it into transport messages.
- This is acceptable today, but future battle result variants can drift from the contract unless classification/coverage is explicit.

Candidate:

- Keep the transport split, but make battle transport extraction a small typed contract with exhaustive tests over all battle result variants.
- Consider moving "battle result must not be command payload" metadata next to the `BehaviorResult` variant definitions.

### 4. Run snapshot shape is mostly ad-hoc JSON while Unity treats it as contract

Evidence:

- `get_run_snapshot_json()` builds top-level snapshot fields with `json!` (`src/game/world/snapshot.rs:75`).
- `game_state_context` is assembled by hand (`src/game/world/snapshot.rs:137`).
- `inventory`, `roster`, `selected_event`, and display item snapshots are assembled by hand (`src/game/world/snapshot.rs:311`, `:456`, `:600`, `:652`).
- Unity docs treat `game_state_context`, `allowed_actions`, `selected_event`, `inventory`, and `roster` as official non-battle state sources (`unity_core_contract.md:2256`).

Why it matters:

- Ad-hoc JSON makes it easy to drift from docs/tests without compiler help.
- `skill_catalog` already moved in the better direction by using `SkillCatalogDto`; the rest of snapshot can be migrated incrementally to typed DTOs.

Candidate:

- Introduce typed snapshot DTOs by stable slices, starting with high-risk contract surfaces: `GameStateContextDto`, `SelectedEventSnapshotDto`, `InventorySnapshotDto`, `RosterSnapshotDto`.
- Preserve serialized shape while moving construction into typed structs.
- Add snapshot shape tests for representative states rather than only checking scattered fields.

Policy:

- Serialized state snapshot shape is Unity-facing. Internal DTO migration without shape change is safe; field rename/removal/addition needs 사용자와 정책 논의 필요.

### 5. Server mutates core snapshot to add combat result compressed timeline

Evidence:

- Core snapshot for `ActiveNodeContent::CombatBattle` includes `"has_timeline": true` but not `compressed_timeline` (`src/game/world/snapshot.rs:701`).
- Server `build_state_snapshot()` calls `game_core.get_run_snapshot_json()` and then inserts `selected_event.compressed_timeline` if `get_combat_result_event_log()` returns a log (`../game_server/src/game/player_game_actor/handlers.rs:176`).
- The battle transport contract requires final result snapshot to include `selected_event.compressed_timeline` (`core_unity_battle_transport_contract.md:101`).

Why it matters:

- Core snapshot and actual server snapshot are not the same source. A tool/test that calls core directly will miss a field the Unity runtime receives.
- The compression itself may be server-owned, but the presence/meaning of the field is a Unity-facing gameplay/result contract.

Candidate:

- Make the boundary explicit: either core exposes a typed "combat result snapshot attachment" contract and server only serializes/compresses bytes, or server wraps state snapshots in a typed server DTO with tested augmentation.
- Add a test proving final server snapshot has `game_state_context.type == "combat_result"` and `selected_event.compressed_timeline`, while core-only snapshot either documents absence or supplies a typed placeholder.

Policy:

- Whether `compressed_timeline` belongs to core snapshot or server wrapper is a contract ownership decision. 사용자와 정책 논의 필요.

### 6. `BattleResync.setup` exists but current constructor always sets `None`

Evidence:

- `PlayerGameServerMessage::BattleResync` has `setup: Option<Value>` (`../game_server/src/game/player_game_actor/messages.rs:404`).
- `PlayerGameServerMessage::battle_resync()` always sets `setup: None` (`../game_server/src/game/player_game_actor/messages.rs:441`).
- `request_battle_resync { need_setup: true }` maps to `RecoverBattleSetupLoss` and sends a normal setup/update flow, not a `battle_resync` with setup (`../game_server/src/game/player_game_actor/messages.rs:323`).

Why it matters:

- `setup` is a dormant dual-shape field. It suggests one resync envelope can optionally carry setup, but the implemented setup-loss path uses a different transport flow.
- This looks like compatibility/future fallback surface unless a near-term client path depends on it.

Candidate:

- Either remove `BattleResync.setup` and keep setup-loss recovery as `battle_setup_snapshot` + `battle_update`, or implement it as the official setup resync shape with tests.

Policy:

- Removing or repurposing the field changes Unity-facing transport JSON. 사용자와 정책 논의 필요.

### 7. `message_type` exists inside core live DTOs but server top-level messages flatten it away

Evidence:

- `LiveBattleUpdateDto` contains `#[serde(rename = "type")] message_type` (`src/game/behavior.rs:534`).
- `LiveBattleSetupSnapshotDto` also contains `message_type` (`src/game/behavior.rs:550`).
- `PlayerGameServerMessage::battle_update()` flattens fields into `BattleUpdate` and drops nested `message_type` (`../game_server/src/game/player_game_actor/messages.rs:413`).
- Tests assert top-level `battle_update` and `battle_setup_snapshot` do not include `message_type` (`../game_server/src/game/player_game_actor/messages.rs:550`, `:599`).

Why it matters:

- There are two serialized shapes for the same DTO family: core DTOs have a `type`, server top-level messages own the `type`.
- This is intentional enough to be tested, but it should remain documented as a boundary. Otherwise future code may serialize `LiveBattleUpdateDto` directly and produce a different envelope.

Candidate:

- Rename or split internal DTOs to make envelope ownership clear, for example `LiveBattleUpdatePayloadDto` for nested payload versus `PlayerGameServerMessage::BattleUpdate` for top-level.
- Alternatively keep as-is but add a source-of-truth comment and contract test that direct DTO serialization is not the WebSocket top-level shape.

### 8. Admin command output remains mostly `Value` payloads

Evidence:

- `AdminCommandOutput` carries `payload: Value` (`src/game/world/admin.rs:18`).
- Many admin commands assemble payloads with `json!` (`src/game/world/admin.rs:30`).
- Grant catalog is typed internally (`src/game/world/admin/catalog.rs:87`) but serialized into `Value` before returning (`src/game/world/admin/catalog.rs:108`).
- Server wraps admin results as `PlayerGameServerMessage::AdminResult { result_type, payload }` (`../game_server/src/game/player_game_actor/handlers.rs:104`).

Why it matters:

- Admin is likely dev-facing rather than ordinary gameplay-facing, so this is lower priority.
- Still, `admin_dump_grant_catalog` is useful Unity tooling data and already has a typed DTO; the final output boundary discards type information.

Candidate:

- Keep low-risk dump commands as `Value`, but preserve typed DTOs for stable admin tooling contracts such as grant catalog.
- Add focused tests around admin grant catalog schema if Unity tooling depends on it.

Policy:

- If admin UI/tooling consumes this as a stable contract, shape changes require 사용자와 정책 논의 필요.

## Existing Simplification Opportunities

- `skill_catalog` already demonstrates the desired direction: build with typed DTOs in core and serialize once (`src/game/world/snapshot.rs:231`, `unity_core_contract.md:2000`). The same pattern can be applied to `selected_event`, `game_state_context`, and inventory/roster slices without changing behavior.
- Live battle `command_result` is already simplified to `CommandAccepted`; avoid reintroducing per-command battle payloads because `battle_update` and checkpoint already provide the data.

## Legacy / Compatibility Candidates

- `BattleResync.setup` is the clearest dormant compatibility field.
- `selected_event.has_timeline` plus server-only `compressed_timeline` should be clarified. If `has_timeline` exists only to support older result UI, mark removal conditions or replace it with typed result-log availability.
- Core DTO `message_type` fields versus server top-level `type` should be documented as a deliberate boundary or split to avoid accidental dual schema.

## Do Not Change Yet

- Do not collapse `PlayerBehaviorRequest` directly into `PlayerBehavior` without preserving transport-only commands and snake_case request contract.
- Do not move live battle state back into full `state_snapshot`; current contract intentionally uses `battle_update.events_delta` plus `checkpoint`.
- Do not remove `CommandAccepted` simplification for battle commands.
- Do not change snapshot field names while only trying to improve internal typing.

## Validation Plan

Focused tests to add or run when implementing:

- `cargo test -p game_server game::player_game_actor::messages`
- `cargo test -p game_server game::player_game_actor::state`
- `cargo test -p game_server game::player_game_actor::handlers`
- `cargo test -p game_core world::tests::combat`
- `cargo test -p game_core live_skill_catalog_audit`
- Snapshot shape tests for `game_state_context`, `selected_event` variants, inventory, roster, and final combat result state snapshot.

No Rust tests were run for this audit-only document.
