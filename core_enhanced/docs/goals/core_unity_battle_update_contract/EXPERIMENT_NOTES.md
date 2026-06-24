# Core Unity Battle Update Contract Experiment Notes

This file records source-of-truth findings, implementation judgments, policy questions, and follow-up candidates.

## 2026-06-15 Initial Notes

Target contract:

- `/mnt/f/unity projects/ark/docs/core_unity_battle_update_contract.md`

Core policy to preserve:

- Unity does not calculate current gameplay state from events.
- Unity does not create presentation timing from checkpoints.
- Unity processes events through `checkpoint.at_seq`, then reconciles to checkpoint.
- `CommandResult` is not a battle state-change source.
- Unity-facing battle event seq must be contiguous in the presentation stream.

Important initial implementation constraints:

- Initial checkpoint policy should be `checkpoint.at_seq == events_delta.to_seq`.
- If current core timeline contains internal events not sent to Unity, implementation likely needs a separate Unity-facing `presentation_seq` / `stream_seq`.
- `UnitDeployed` should be distinct from `UnitSpawned`.
- Deployment command result should become acknowledgement-only in the updated path; visible deployment timing belongs in the event stream.
- Current server failure shape may remain top-level `error` unless the user approves a broader command rejection shape change.

## Source-Of-Truth Questions To Answer By Reading Code

- Where is the current timeline seq assigned, and is it already contiguous for the Unity-visible event subset?
- Which current timeline events are presentation-worthy and which are internal/debug/result-only?
- Does current `UnitSpawned` include player deployment, enemy/wave spawn, defense object spawn, or all of them?
- Can `BattleCore` emit `UnitDeployed` at the exact command acceptance battle time without duplicating spawn semantics?
- Where does `BattleAdvanced` mapping currently become WebSocket notification payload?
- Which tests currently treat command result payload as the state-change source?
- Which Unity-facing snapshots already contain enough checkpoint data, and which fields need new typed DTOs?

## 2026-06-15 Code Reading Findings

- Current event seq is assigned in `src/game/battle/recording.rs::record_timeline` from `BattleCore.timeline_seq`, then incremented by 1. The full timeline entries are contiguous.
- `Timeline::entries_after_seq(last_seen_seq)` returns all entries with `entry.seq > last_seen_seq`; current deltas are conceptually `(last_seen_seq, last_event_log_seq]`.
- Current server tick path is `ActiveBattleSession::drain_event_log_delta()` -> `BehaviorResult::BattleAdvanced` -> `game_server` `Notification { notification_type: "battle_delta", payload: { result_type: "BattleAdvanced", payload } }`.
- Current deploy path is `GameCore::handle_deploy_unit` -> `BattleCore::apply_live_command(DeployPlayerUnit)` -> `BattleCore::deploy_player_unit`.
- `BattleCore::deploy_player_unit` creates a live player scenario group and calls `spawn_scenario_group`, then records spawned units as `TimelineEvent::UnitSpawned`.
- Current `UnitSpawned` covers the live deployed player unit, so Unity cannot distinguish scenario/wave/object spawn from player command deployment by event type alone.
- Existing `TimelineEvent` already uses implementation-near names from the target contract (`ManualCastStart`, `SkillProjectileLaunched`, `BasicAttackProjectileImpacted`, etc.), so no naming policy question is needed for the first slice.
- `BattleUnitDraft.owned_uuid` is the employee UUID for player deployment drafts, and `handle_deploy_unit` already has employee UUID, position, facing, time, and resulting unit id.
- First safe slice: add a distinct `TimelineEvent::UnitDeployed` after live player deployment while leaving existing `UnitSpawned` intact for scenario/wave/object spawn semantics. This advances the target contract without deciding the final WebSocket envelope yet.
- Existing timeline seq started at 0, which conflicts with the target `(after_seq, to_seq]` stream contract for an initial Unity request of `after_seq: 0`. New/reset live battles should start timeline seq at 1 so the first event is `seq: 1`.
- `BattleAdvanced` and requested `BattleState` can share a single `LiveBattleUpdateDto` builder internally. This reduces schema drift even before the outer WebSocket envelope is switched to the target `battle_update` transport.
- Initial checkpoint policy is implemented as `checkpoint.at_seq == events_delta.to_seq`. Empty deltas from `battle_update_dto_after(Some(last_seq))` keep `to_seq == after_seq` and checkpoint at the same seq.
- `checkpoint.deployment` currently reuses `LiveBattleDeploymentDto`, including deployed unit `position`, facing, costs, redeploy locks, and skill readiness. `checkpoint.units` is a broader current unit-state list for recovery/UI use.
- Core battle `BehaviorResult` variants now carry `battle_update: LiveBattleUpdateDto` so server transport can emit the authoritative update without rebuilding checkpoint data from legacy fields.
- Server live tick transport now emits top-level `battle_update` messages instead of `notification_type: "battle_delta"`.
- Server command handling now maps battle-related behavior results to `command_result { ok: true, result_type: "CommandAccepted" }` plus a side `battle_update`. The command result no longer exposes deployment/timeline payload as the state-change source.
- Existing `RequestBattleState { since_seq }` can serve the current resync-like fetch because it returns a battle update side message without advancing the live push cursor. The contract still calls the target command `request_battle_resync`, so naming/aliasing remains a follow-up unless code evidence or user policy says to rename immediately.
- Added explicit server client request `request_battle_resync { known_seq, need_setup }` for catch-up resync. It maps to core `RequestBattleState { since_seq: Some(known_seq) }` and returns `CommandAccepted` plus side `battle_resync` with nested `battle_update`.
- `need_setup: true` is intentionally rejected with `hard_battle_resync_not_supported`; implementing it correctly requires `battle_setup_snapshot`, which is not currently exposed by runtime/server transport.
- Removed the server-side legacy battle command payload serializer. `behavior_result_payload` now rejects battle results if they bypass `behavior_result_to_command_result` and its `battle_update` / `battle_resync` side-message mapping.
- `source_command_id` belongs on the Unity-facing timeline entry, not inside the domain `PlayerBehavior`, because it is a transport/request correlation id rather than a gameplay command parameter.
- Server actor command handling now calls `GameCore::execute_with_source_command_id` with the WebSocket `request_id`; non-server callers can keep using `GameCore::execute` and produce entries without source ids.
- `BattleCore` records source command ids through a stack alongside cause context. This keeps command correlation on all events recorded during the live command execution without manually threading the id through every event constructor.
- Live deploy, withdraw, and manual skill activation are currently the source-command-aware live command paths. Passive simulation ticks and scheduled effects do not get a source id unless they are recorded during the originating command execution.
- Core battle `BehaviorResult` variants no longer duplicate `events_delta` or checkpoint fields outside `battle_update`. This removes the internal compatibility surface instead of keeping an old result shape beside the new transport contract.

## Policy Questions If Encountered

Stop the goal and ask the user if any of these become necessary:

- Rename current battle event enum variants vs introduce a new presentation event enum.
- Switch rejected commands from top-level `error` to `command_result { ok: false }`.
- Keep `battle_update` temporarily inside the existing `notification` envelope.
- Add live RON fields to support the transport.
- Keep a temporary dual schema beyond a narrow transition window.
- Whether hard resync + `battle_setup_snapshot` must ship in the same goal or in a later Unity setup contract goal.

## Follow-Up Candidates

These are not in scope unless required to finish the target contract safely:

- Unity client migration to event queue + checkpoint reconcile.
- Dedicated WebSocket smoke probe for battle update gap/resync behavior.
- `battle_setup_snapshot` DTO and hard resync transport.
- Full battle result log viewer changes.
