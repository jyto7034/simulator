# Core Unity Battle Update Contract Plan

## Objective

Implement the next core/server -> Unity live battle transport contract described by `/mnt/f/unity projects/ark/docs/core_unity_battle_update_contract.md`, with event delta as the presentation timing authority and checkpoint as the current-state/recovery authority.

## Goal Mode Working Method

Maintain these working files throughout the goal:

- `docs/goals/core_unity_battle_update_contract/PLAN.md`
- `docs/goals/core_unity_battle_update_contract/EXPERIMENTS.md`
- `docs/goals/core_unity_battle_update_contract/EXPERIMENT_NOTES.md`

`PLAN.md` records plan, scope, completion conditions, stop conditions, and verification commands.
`EXPERIMENTS.md` records attempts, failures, fixes, and verification results.
`EXPERIMENT_NOTES.md` records source-of-truth findings, policy questions, and follow-up candidates.

## Engineering Principles

- Prefer long-term contract clarity over small compatibility patches.
- Do not preserve legacy `battle_delta` / `timeline_delta` behavior through a long-lived dual schema.
- If a short bridge is unavoidable, document the reason, priority field, removal condition, and verification path before implementing it.
- Do not make Unity calculate current gameplay state from event history.
- Do not make Unity create presentation timing from checkpoint arrival.
- Preserve deterministic event ordering and gap detection.
- Tests must lock Unity-facing DTO shape, command/result mapping, battle event stream behavior, and real gameplay flow rather than internal helper shape.

## Source Of Truth

Confirm facts in this order before changing code:

1. Runtime code:
   - `src/game/battle/timeline.rs`
   - `src/game/battle/recording.rs`
   - `src/game/battle/core/mod.rs`
   - `src/game/battle/core/sim.rs`
   - `src/game/battle/core/commands.rs`
   - `src/game/behavior.rs`
   - `src/game/world/state.rs`
   - `src/game/world/combat.rs`
   - `src/game/world/snapshot.rs`
   - `../game_server/src/game/player_game_actor/messages.rs`
   - `../game_server/src/game/player_game_actor/state.rs`
   - `../game_server/src/game/player_game_actor/handlers.rs`
2. Live data and flow:
   - live DefenseRoute encounters in `../game_resources/data/pve/encounters.ron`
   - battle scenario generation and live RON loading tests
3. Unity-facing contract:
   - `/mnt/f/unity projects/ark/docs/core_unity_battle_update_contract.md`
   - `/mnt/f/unity projects/ark/docs/unity_core_contract.md`
4. Policy docs:
   - `docs/game_rulebook.md`
   - `docs/refactor_preparation_plan.md`
   - `docs/code_documentation_sync_guidelines.md`

## In Scope

- Add or refactor server/core DTOs for `battle_update`, `events_delta`, and `checkpoint`.
- Establish a contiguous Unity-facing presentation event stream.
- Decide from code evidence whether existing timeline seq can serve as presentation seq or a separate `presentation_seq` / `stream_seq` is required.
- Add a player deployment presentation event such as `UnitDeployed`.
- Ensure `UnitSpawned` and `UnitDeployed` have distinct meanings.
- Move live battle update transport away from command-result-like `BattleAdvanced` payloads toward the target `battle_update` shape.
- Keep command result as accepted/rejected acknowledgement, with real battle changes arriving through event/checkpoint.
- Add resync request/response behavior or, if not fully implemented in this goal, define the minimal server-side contract and tests needed before Unity migration.
- Update external canonical Unity contract docs if implementation changes the DTO shape or command envelope.
- Add focused tests for DTO shape, seq continuity, gap/resync behavior where implemented, deployment event emission, and checkpoint `at_seq`.

## Out Of Scope

- Unity client implementation, except for contract-driven notes needed in external docs.
- Live RON schema changes unless runtime code proves they are required.
- Gameplay balance changes.
- New battle mechanics unrelated to transport.
- Rewriting the entire battle timeline/event enum solely for naming polish.
- Long-lived compatibility layers for old and new battle transport contracts.
- Full replay tooling beyond preserving full battle event log semantics.

## Policy Decisions To Confirm Before Implementation

Stop and ask the user if code reading shows any of these are not obvious:

- Whether Unity-facing event names should preserve current timeline enum names or use a new presentation event enum.
- Whether rejected battle commands should remain top-level `error` messages or move to `command_result { ok: false, result_type: "CommandRejected" }`.
- Whether `battle_update` is a top-level WebSocket message or a short-transition payload inside the existing `notification` envelope.
- Whether a separate `presentation_seq` / `stream_seq` is required.
- How much resync behavior must ship in the first implementation slice.
- Any Unity-facing DTO shape change beyond what the target contract already specifies.

## Implementation Plan

1. Inspect current timeline recording and server push path.
   - Record current event seq source, event filtering, and payload mapping in `EXPERIMENT_NOTES.md`.
2. Inventory current live battle events.
   - Identify which current events are presentation events, current-state-only data, internal/debug events, or result-log-only entries.
3. Decide the first implementation slice.
   - Prefer a small vertical slice: `UnitDeployed` event + contiguous event delta DTO + checkpoint DTO + tests.
   - Stop if the slice requires a policy decision listed above.
4. Add/adjust core event representation.
   - Add `UnitDeployed` or equivalent presentation event.
   - Keep `UnitSpawned` for scenario/wave/object spawn semantics.
5. Add presentation stream sequencing.
   - Ensure Unity-facing deltas are contiguous after filtering.
   - Add tests for `(after_seq, to_seq]` and gap detection assumptions.
6. Add checkpoint DTO.
   - Include `at_seq`, `battle_time_ms`, `playback`, `units`, and `deployment`.
   - Initial policy: `checkpoint.at_seq == events_delta.to_seq`.
7. Add server transport mapping.
   - Emit `battle_update` from live battle ticks or define the exact transition envelope if one short bridge is approved.
   - Keep command result from being the state-change source.
8. Add resync contract support.
   - Implement `request_battle_resync` if feasible in this goal.
   - Otherwise, document why it is deferred and what tests block Unity migration.
9. Update tests.
   - Focused game_core tests for event emission and checkpoint seq.
   - game_server mapping tests for WebSocket DTO shape.
   - Existing live battle flow tests updated away from legacy payload assumptions.
10. Update docs.
   - Keep `/mnt/f/unity projects/ark/docs/core_unity_battle_update_contract.md` in sync with final implementation.
   - Update `/mnt/f/unity projects/ark/docs/unity_core_contract.md` only when the runtime/server contract actually changes.
11. Run verification.
   - Run focused tests after each slice.
   - Finish with relevant `cargo check` and package tests.

## Current Progress

- Completed code reading for timeline seq assignment, live deploy flow, and current `BattleAdvanced` / `BattleState` payload construction.
- Added distinct `TimelineEvent::UnitDeployed` for player deployment.
- Added target-shaped `LiveBattleUpdateDto`, `LiveBattleEventDeltaDto`, and `LiveBattleStateCheckpointDto`.
- Routed current `BattleAdvanced` and requested `BattleState` legacy fields through the new internal battle update builder.
- Set new/reset timeline seq start to 1 so initial Unity stream requests can use `after_seq: 0`.
- Added focused tests for first event seq, empty delta semantics, `checkpoint.at_seq == events_delta.to_seq`, and deployment event/checkpoint separation.
- Added top-level server `battle_update` push messages for live battle ticks.
- Changed server battle command responses to `CommandAccepted` plus side `battle_update` instead of state-changing command payloads.
- Added server serialization coverage proving the Unity-facing message shape uses top-level `type: "battle_update"`.
- Added explicit `request_battle_resync` catch-up request mapping to `CommandAccepted` plus side `battle_resync` containing nested `battle_update`.
- Hard resync with setup snapshot is explicitly rejected as `hard_battle_resync_not_supported` because `battle_setup_snapshot` transport is not implemented yet.
- Removed the server legacy battle payload serializer path; battle `BehaviorResult` values now require battle update transport.
- Added optional `TimelineEntry.source_command_id` and server -> core request-id propagation so command-created presentation events can be correlated with the accepted command.
- Removed duplicate legacy battle fields from core battle `BehaviorResult` variants; current state and event deltas now flow through `battle_update`.

Still pending:

- Hard resync with `battle_setup_snapshot` remains a follow-up because the runtime/server currently has no setup snapshot transport to return.
- No implementation blockers remain for the live `battle_update` transport slice.

## Test Requirements

Tests should cover:

- `UnitDeployed` appears in the Unity-facing event stream when a player deploys a unit.
- Command-created presentation events expose `source_command_id` when the server request id is available.
- `CommandResult` for deployment is acknowledgement-only and does not need a large deployment snapshot as state source.
- `events_delta.events` is contiguous and covers `(after_seq, to_seq]`.
- Empty event delta is only valid when `to_seq == after_seq`.
- `checkpoint.at_seq == events_delta.to_seq` in the initial implementation.
- `checkpoint.deployment` contains current deployed unit state, including position/facing/skill readiness as applicable.
- `UnitSpawned` remains distinct from `UnitDeployed`.
- Server WebSocket mapping exposes the target shape consistently.
- Legacy tests that treat timeline events as current-state reconstruction are removed or rewritten to checkpoint-based assertions.

## Verification Commands

Start focused and expand as the goal progresses:

```text
cargo test -p game_core world::tests::combat -- --nocapture
cargo test -p game_core battle -- --nocapture
cargo test -p game_server battle -- --nocapture
cargo check -p game_core
cargo check -p game_server
cargo test -p game_core
cargo test -p game_server
```

If live RON or scenario loading is touched:

```text
cargo test -p game_core --test ron_loading -- --nocapture
```

If WebSocket probe contract is updated:

```text
APP_SERVER__BIND_ADDRESS=127.0.0.1 APP_SERVER__PORT=18082 cargo run -p game_server
WS_PORT=18082 python3 "/mnt/f/unity projects/ark/docs/probe/<updated_probe>.py"
```

## Final Verification Run

Executed on 2026-06-16:

```text
cargo check -p game_core
cargo check -p game_server
cargo test -p game_core defense_combat_node_smoke_writes_debug_event_log_export -- --nocapture
cargo test -p game_server player_game_actor -- --nocapture
cargo test -p game_core
cargo test -p game_server
```

Results:

- All commands passed.
- Full `game_core` run passed 444 lib tests plus integration suites: `live_item_skill_activation` 3, `live_skill_catalog_audit` 3, `ron_loading` 16, `skill_refactor_validation` 10, `skill_test_suite` 14.
- Full `game_server` run passed 17 tests.

## Completion Conditions

- Runtime/server emits or can serve the target `battle_update` contract for live DefenseRoute battle updates.
- Unity-facing event stream has contiguous presentation seq semantics.
- `UnitDeployed` exists in the presentation event stream for player deployment.
- `checkpoint.at_seq == events_delta.to_seq` is implemented and tested for the initial contract.
- Unity-facing checkpoint includes current battle UI/recovery state needed by the contract.
- Command results are no longer the authoritative source for battle state changes in the updated path.
- Command-created presentation events can be correlated to accepted commands through `source_command_id`.
- Legacy battle update assumptions are removed, not hidden behind a long-lived compatibility layer.
- External canonical docs match the implemented DTO/command shape.
- Focused and broader verification commands pass or any failures are recorded with cause and next action.
- 사용자와 의논하여 정해야 할 정책이 있을 경우 goal을 종료한다.

## Stop Conditions

- Unity-facing DTO shape must change beyond the target contract.
- Live RON schema change is required.
- Save data migration is required.
- UX meaning changes are required.
- Event naming requires a policy decision that cannot be derived from code.
- A long-lived dual schema appears necessary.
- Existing tests protect behavior that conflicts with the new contract but the user-visible replacement behavior is unclear.
- Server cannot provide contiguous presentation events without a larger timeline redesign.
- 사용자와 의논하여 정해야 할 정책이 있을 경우 goal을 종료한다.
