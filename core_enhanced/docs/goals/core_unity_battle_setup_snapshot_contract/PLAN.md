# Core Unity Battle Setup Snapshot Contract Plan

## Objective

Implement the long-term core/server -> Unity `battle_setup_snapshot` contract described by `/mnt/f/unity projects/ark/docs/core_unity_battle_setup_snapshot_contract.md`.

The goal is to make Unity battle scene construction use an explicit setup snapshot instead of overloading `combat_preview`, `game_state_context`, or `battle_update.checkpoint`.

Responsibility split to preserve:

```text
combat_preview
  pre-battle UI and player-facing preview

battle_setup_snapshot
  battle scene construction

battle_update.events_delta
  presentation timeline

battle_update.checkpoint
  current state and recovery

command_result
  command acknowledgement
```

## Goal Working Method

Maintain these working files throughout the goal:

- `docs/goals/core_unity_battle_setup_snapshot_contract/PLAN.md`
- `docs/goals/core_unity_battle_setup_snapshot_contract/EXPERIMENTS.md`
- `docs/goals/core_unity_battle_setup_snapshot_contract/EXPERIMENT_NOTES.md`

`PLAN.md` records scope, implementation plan, completion conditions, stop conditions, and verification commands.
`EXPERIMENTS.md` records attempts, failures, fixes, and verification results.
`EXPERIMENT_NOTES.md` records source-of-truth findings, policy questions, and follow-up candidates.

Update these files as the implementation evolves. Do not leave them as a stale initial plan.

## Engineering Principles

- Prefer long-term contract clarity over compatibility patches.
- Do not make Unity infer battle setup from `combat_preview`, checkpoint, event replay, or live RON.
- Do not reintroduce command-result battle state payloads.
- Do not keep a long-lived dual schema for old and new battle start messages.
- If a short transition is unavoidable, document the exact reason, removal condition, owner, and tests before implementing it.
- Keep `combat_preview` as pre-battle UI data unless runtime code proves a narrower boundary is needed.
- Keep `battle_update.checkpoint` focused on current state/recovery.
- Keep `battle_update.events_delta` focused on presentation timing.
- Tests must lock Unity-facing DTO shape, message order, live RON/data consistency, and real gameplay flow.

## Source Of Truth

Confirm facts in this order before changing code:

1. Runtime code:
   - `src/game/events/combat.rs`
   - `src/game/combat_preview/mod.rs`
   - `src/game/combat_preview/types.rs`
   - `src/game/combat_preview/template.rs`
   - `src/game/world/combat.rs`
   - `src/game/world/state.rs`
   - `src/game/world/snapshot.rs`
   - `src/game/behavior.rs`
   - `src/game/battle/scenario.rs`
   - `src/game/battle/battlefield/`
   - `src/game/battle/timeline.rs`
   - `../game_server/src/game/player_game_actor/messages.rs`
   - `../game_server/src/game/player_game_actor/state.rs`
   - `../game_server/src/game/player_game_actor/handlers.rs`
   - `../game_server/src/game/player_game_actor/session.rs`
2. Live RON/data:
   - `../game_resources/data/pve/encounters.ron`
   - live battlefield/template data referenced by current encounters
   - live abnormality/unit/skill references used by setup metadata
3. Unity-facing contracts:
   - `/mnt/f/unity projects/ark/docs/core_unity_battle_setup_snapshot_contract.md`
   - `/mnt/f/unity projects/ark/docs/core_unity_battle_update_contract.md`
   - `/mnt/f/unity projects/ark/docs/unity_core_contract.md`
4. Policy docs:
   - `docs/code_documentation_sync_guidelines.md`
   - `docs/game_rulebook.md`
   - `docs/refactor_preparation_plan.md`

Do not trust the target document blindly. If runtime code shows a cleaner or safer contract shape, record the evidence in `EXPERIMENT_NOTES.md`. If the change is a Unity-facing DTO policy decision, stop and ask the user before implementing it.

## In Scope

- Inventory the runtime sources for battle setup data.
- Add typed core DTOs for `battle_setup_snapshot`.
- Build setup snapshot from runtime battle/session data, not from ad-hoc duplicated Unity-only calculations.
- Expose top-level server message `type: "battle_setup_snapshot"`.
- Send `battle_setup_snapshot` after `CommandAccepted` and before the first `battle_update` when `ConfirmEnterNode` starts a live battle.
- Ensure setup snapshot and first battle update share the same `battle_uuid`.
- Keep deployment/current mutable state in `battle_update.checkpoint`, not setup.
- Keep presentation timing in `battle_update.events_delta`, not setup.
- Preserve `BattleStart` event semantics; do not replace it with setup snapshot.
- Add tests for DTO shape, message order, live RON DefenseRoute setup data, and relationship with `battle_update`.
- Update external canonical Unity docs if implementation shape differs from the target contract.

## Out Of Scope

- Unity client implementation.
- Full hard resync implementation unless setup snapshot support proves small and policy-free after the main slice.
- Save data migration.
- Gameplay balance changes.
- Live RON schema changes unless runtime code proves setup snapshot cannot be correct without them.
- Removing `combat_preview`.
- Rewriting battle timeline event names.
- Embedding large display/catalog metadata unless the user approves that DTO policy.

## Policy Decisions To Confirm Before Implementation

Stop the goal and ask the user if any of these become necessary:

- Whether `initial_units` should be required, optional, or omitted in the first implementation.
- Whether setup snapshot should include catalog ids only or embed display metadata.
- Whether hard resync should return `{ setup, update }` or `{ setup, checkpoint }`.
- Whether `battle_setup_snapshot` should also be returned on `RequestBattleState { since_seq: None }` during Unity migration.
- Whether `setup_version` should be global or per nested section.
- Whether `combat_preview` should lose or rename any existing Unity-facing fields.
- Whether live RON schema/content must change.
- Whether a temporary dual schema is required for Unity migration.
- Whether setup should include data that can become mutable during battle, such as current HP, skill readiness, deployment cost, or playback.

## Implementation Plan

1. Runtime setup inventory
   - Read battle construction path from map node preview to active battle session.
   - Identify which fields come from `CombatPreview`, `BattleScenario`, `Battlefield`, tactical plans, deployment policy, and live data.
   - Record findings in `EXPERIMENT_NOTES.md`.

2. Contract shape audit
   - Compare runtime fields to the target `battle_setup_snapshot` DTO.
   - Decide which fields are safe for the first slice.
   - Stop if `initial_units`, catalog embedding, hard resync response shape, or live RON schema requires policy input.

3. Core DTO slice
   - Add `LiveBattleSetupSnapshotDto` and nested typed DTOs in or near `src/game/behavior.rs`, unless runtime structure suggests a clearer module.
   - Use `#[serde(rename = "type")]` and `snake_case`/existing enum casing consistently with current Unity-facing DTO conventions.
   - Keep the DTO focused on static/initial scene construction.

4. Runtime builder
   - Build setup snapshot from active battle session/runtime battle inputs.
   - Prefer reusing `CombatPreview`/battle scenario data over recomputing from live RON directly.
   - Avoid duplicating logic that already exists in battle construction.

5. Server transport
   - Add `PlayerGameServerMessage::BattleSetupSnapshot`.
   - On live battle enter, return:
     - `CommandAccepted`
     - side `BattleSetupSnapshot`
     - side `BattleUpdate`
   - Do not add setup snapshot to deploy/withdraw/skill commands.

6. Tests
   - Add focused core DTO tests.
   - Add live RON DefenseRoute setup test.
   - Add server actor message order test.
   - Add JSON shape test for `type: "battle_setup_snapshot"`.
   - Update old tests by replacing legacy assumptions, not by preserving ignored compatibility tests.

7. Documentation sync
   - Update `/mnt/f/unity projects/ark/docs/core_unity_battle_setup_snapshot_contract.md` if implementation differs.
   - Update `/mnt/f/unity projects/ark/docs/unity_core_contract.md` if the actual server contract changes.
   - Keep local goal docs current.

8. Verification
   - Run focused tests after each slice.
   - Finish with broad `cargo check` and relevant package tests.

## Implementation Status

Updated 2026-06-16:

- Runtime setup inventory completed.
- Core setup DTO slice completed with `LiveBattleSetupSnapshotDto`.
- Runtime builder completed through `ActiveBattleSession::battle_setup_snapshot_dto()`.
- Server transport completed for live battle enter side-message order:
  - `command_result`
  - `battle_setup_snapshot`
  - `battle_update`
- `request_battle_resync { need_setup: true }` remains explicitly unsupported; hard resync stays out of scope.
- `combat_preview` remains unchanged and continues to serve pre-battle UI.
- `initial_units` is currently an empty list because current start-time actors are timeline-spawned through `battle_update.events_delta`.
- No live RON schema or content changes were required.
- No compatibility layer, fallback path, or dual schema was added.
- External canonical Unity contract was updated with the implemented `battlefield.tiles`, ids-only `catalog_refs`, empty `initial_units`, actor-free `static_objects`, and hard-resync-not-supported notes.

Implemented setup snapshot fields:

- `type`
- `setup_version`
- `battle_uuid`
- `encounter_id`
- `node_type`
- `mission_variant`
- `battlefield`
- `routes`
- `deployment_zones`
- `spawn_zones`
- `tactical_points`
- `static_objects`
- `initial_units`
- `catalog_refs`

Focused verification completed:

- `cargo check -p game_core`
- `cargo check -p game_server`
- `cargo check -p game_core --tests`
- `cargo check -p game_server --tests`
- `cargo test -p game_core battle_setup_snapshot -- --nocapture`
- `cargo test -p game_server battle_setup_snapshot -- --nocapture`
- `cargo test -p game_server live_battle_tick_pushes_delta_and_snapshot_after_confirm_enter -- --nocapture`
- `cargo test -p game_server paused_live_battle_tick_does_not_push_delta_until_resumed -- --nocapture`
- `cargo test -p game_core`
- `cargo test -p game_server`

Remaining before completion:

- None for this implementation slice.

## Test Requirements

Core behavior tests should prove:

- Live DefenseRoute battle start can build `LiveBattleSetupSnapshotDto`.
- Setup snapshot includes battle identity, encounter id, node type, mission variant, battlefield dimensions, route data, deployment zones, spawn zones or explicit absence, and tactical/static objects where applicable.
- Setup snapshot excludes mutable current-state data such as deployment cost, skill readiness, playback, current HP, and projectile state.
- Setup snapshot and first `battle_update` share `battle_uuid`.
- `BattleStart` remains in `battle_update.events_delta`.
- `battle_update.checkpoint` remains the source for deployment/current unit state.

Server tests should prove:

- `ConfirmEnterNode` that starts live battle returns `CommandAccepted`.
- The side message order is `BattleSetupSnapshot` before `BattleUpdate`.
- Unity-facing JSON serializes top-level `type: "battle_setup_snapshot"`.
- Deploy/withdraw/skill commands do not send setup snapshot again.
- Existing `request_battle_resync { need_setup: true }` remains explicitly rejected until hard resync is implemented, or is updated with tests if hard resync is in scope.

Live data tests should prove:

- Current live RON DefenseRoute encounters have enough data to build setup snapshot.
- Route/deployment zone data used by runtime movement/deploy validation matches setup snapshot.
- No Unity setup field requires Unity to inspect live RON directly.

## Verification Commands

Start focused:

```text
cargo check -p game_core
cargo test -p game_core battle_setup_snapshot -- --nocapture
cargo test -p game_core live_ron_defense_route_playable_path_runs_to_combat_result -- --nocapture
cargo test -p game_server battle_setup_snapshot -- --nocapture
cargo test -p game_server player_game_actor -- --nocapture
```

If live RON/data is touched:

```text
cargo test -p game_core --test ron_loading -- --nocapture
```

Before completion:

```text
cargo check -p game_core
cargo check -p game_server
cargo test -p game_core
cargo test -p game_server
```

If the WebSocket probe is updated:

```text
APP_SERVER__BIND_ADDRESS=127.0.0.1 APP_SERVER__PORT=18082 cargo run -p game_server
WS_PORT=18082 python3 "/mnt/f/unity projects/ark/docs/probe/<updated_probe>.py"
```

## Completion Conditions

- Runtime/server can emit top-level `battle_setup_snapshot` for live battle start.
- `ConfirmEnterNode` live battle start sends `CommandAccepted`, then setup snapshot, then first battle update.
- Setup snapshot is typed, tested, and built from runtime sources.
- Setup snapshot does not contain mutable current battle state that belongs in checkpoint.
- `battle_update` remains the source for event timing and current-state checkpoint.
- No command result battle state payload or legacy `BattleAdvanced`-style state payload is reintroduced.
- `combat_preview` remains pre-battle UI/preview data; Unity runtime setup has an explicit setup source.
- Live RON DefenseRoute flow is covered by tests.
- External canonical Unity docs match the implemented DTO/message shape.
- Focused and broad verification commands pass, or failures are recorded with cause/fix/reverification in `EXPERIMENTS.md`.
- Any policy question listed above is either resolved from code evidence and documented, or the goal is stopped for user decision.

## Stop Conditions

Stop and report questions instead of guessing if:

- A Unity-facing DTO shape must differ materially from `core_unity_battle_setup_snapshot_contract.md`.
- live RON schema or content must change.
- current `combat_preview` fields must be removed, renamed, or semantically changed.
- `initial_units` cannot be implemented without deciding spawn presentation semantics.
- catalog references require choosing between ids-only and embedded display metadata.
- hard resync must be implemented before setup snapshot can be shipped.
- compatibility with old Unity consumers appears necessary.
- tests protect legacy behavior whose modern replacement is not clear.
- setup data would require Unity to calculate core gameplay rules.
