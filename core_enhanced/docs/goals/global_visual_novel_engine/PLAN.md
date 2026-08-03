# Global Visual Novel Engine

## Objective

Implement Visual Novel (VN) presentation as a route-independent global overlay owned by core runtime state, not as an Event-node-specific screen under `selected_event.type == "event"`.

Unity should open VN UI whenever the current core projection contains an active `visual_novel`, regardless of whether the player is on HQ briefing, map, loadout/maintenance, battle, event node, tutorial, radio, or cutscene UI.

This goal is intentionally larger than moving the existing Event scene snapshot to a new field. The long-term direction is a global VN engine:

- Core owns the active VN session, current scene, choice availability, trigger conditions, blocking policy, and progression.
- Unity renders an overlay from core-provided IDs and does not infer scene progression or choice availability.
- VN scene graph data remains core-owned RON data.
- VN presentation uses Unity-local resource/localization IDs only; core must not send localized dialogue text or image payloads.
- VN can include multiple visible speakers/characters in a scene.
- Blocking VN can suppress background UI commands, including battle commands where applicable.
- Battle VN delivery must work with the existing `battle_update` transport rather than relying only on full `state_snapshot`.

## Source Of Truth Order

1. Runtime code and focused tests.
2. Live RON/data.
3. Unity-facing snapshot/command/server-message contracts.
4. Live server/probe behavior.
5. Current policy documents.

Do not trust documents alone. Existing Unity-facing docs still route Event scene presentation through `selected_event.type == "event"` and describe battle runtime as `battle_update` based. Both must be reconciled against code before editing.

## Current Evidence To Re-Read

- `src/game/data/event_data.rs`
  - `EventDefinition` already provides a scene graph with `entry_scene_id`, scenes, `next`, choices, previews, and effects.
  - `EventScenePresentation` currently has singular `speaker_id` and `portrait_id`. This is not enough for authored multi-character scenes.
- `../game_resources/data/events/story/base.ron`
  - Live Event scene graph currently uses singular `speaker_id`/`portrait_id`.
- `src/game/resources/selection.rs`
  - `EventSessionState` stores event-node-specific progression: `node_id`, `event_id`, `current_scene_id`, committed choice, and started combat marker.
  - `ActiveNodeContent::Event(EventSessionState)` ties Event progression to active node content.
- `src/game/world/event_node.rs`
  - `handle_advance_event_scene` and `handle_select_event_choice` require the current active node content to be an Event.
  - `event_scene_snapshot()` projects current Event scene data for Unity but is Event-session based.
- `src/game/behavior.rs`
  - `EventSceneSnapshotDto` and `EventChoiceSnapshotDto` are currently Event-node DTOs.
  - `SelectedEventSnapshotDto::Event { node_id, event }` exposes the VN-like scene payload under `selected_event`.
  - `RunSnapshotDto` owns the root state snapshot shape and is the correct place for a top-level `visual_novel` projection.
  - `PlayerBehavior::AdvanceEventScene` and `PlayerBehavior::SelectEventChoice` are Event-node commands, not global VN commands.
  - `LiveBattleUpdateDto` is the battle runtime delivery shape; it currently has no VN cue/projection.
- `src/game/managers/action_scheduler.rs`
  - Allowed actions are mostly `GameState` based. Event scene actions are allowed only during `InNode`.
  - Blocking global VN requires action gating that depends on VN runtime state, not only game state.
- `src/game/world/state.rs`
  - `GameCoreState` owns global run/runtime state. A global VN runtime belongs here, not inside `ActiveNodeContent`.
  - `ActiveBattleSession::simulation_delta_for_tick()` skips progression only when playback is paused. Blocking battle VN needs an explicit policy so battle time does not continue invisibly behind a modal VN.
- `../game_server/src/game/player_game_actor/state.rs` and `handlers.rs`
  - Non-battle commands send `command_result` plus a trailing `state_snapshot`.
  - Battle commands and live ticks send side messages such as `battle_update` and intentionally do not send full state snapshots.
- External Unity canonical docs:
  - `/mnt/f/unity projects/ark/docs/unity_core_contract.md` currently describes `selected_event.type == "event"` as Event scene UI source.
  - `/mnt/f/unity projects/ark/docs/core_unity_battle_transport_contract.md` fixes live battle runtime around `battle_setup_snapshot`, `battle_update.events_delta`, and `battle_update.checkpoint`.

## Confirmed Policy

- `visual_novel` is the Unity-facing source of truth for VN overlay presentation.
- Unity must not use `selected_event.type == "event"` to decide whether to show VN UI.
- Event nodes and Visual Novel scenes are separate concepts. An Event node is a node-level choice/result interaction such as the Limbus Company-style "select a choice and receive outcome" screen. A VN scene is dialogue/presentation overlay content that may appear over any route or screen. Do not model every Event node as a VN scene, and do not require every VN scene to come from an Event node.
- `selected_event` may remain for active node content and Event-node-specific choice/result state, but VN scene/dialogue payload should be removed from it once the new contract lands.
- Avoid compatibility layer, fallback path, or dual schema by default.
- If temporary dual exposure is unavoidable for a live migration, record the reason and removal condition here and ask the user before implementing it.
- Core sends resource IDs, localization IDs, and authored layout slots only. Core does not send dialogue text or image bytes.
- Multi-character scenes are a first-class requirement. Do not preserve the current singular `speaker_id`/`portrait_id` model as the long-term schema.
- Existing singular `speaker_id`/`portrait_id` live data should be migrated to `speakers[]` in the same implementation slice that changes the authored schema. A single-speaker scene is represented as a one-element `speakers` array. Do not keep singular authoring aliases without explicit approval.
- VN data is expected to be produced and validated by an authoring/management program, not hand-edited as raw RON. Prefer explicit enum/data shapes and strong validation over compact or implicit schemas.
- VN graph data must use a dedicated `VisualNovelDefinition` catalog. Do not reuse `EventDefinition` as the VN graph schema.
- VN trigger data must use a dedicated `VisualNovelTrigger` table. Do not add an Event-scene `vn_on_enter` shortcut or a presentation effect as the long-term Event-to-VN bridge.
- Event nodes may trigger VN overlays through the trigger system, but Event node scenes themselves are not VN scenes.
- VN commands are global VN commands, not Event-node commands.
- The root snapshot should serialize `visual_novel` explicitly. When no VN overlay is active, the contract should expose `visual_novel: null` unless a concrete serializer constraint is found during implementation.
- Blocking VN suppresses background input. Non-blocking/radio VN may allow background commands if explicitly represented by policy.
- Battle VN must be deliverable without reintroducing per-tick full `state_snapshot` as a battle state source.
- Battle VN transport is not finalized here. Re-read the live battle update/server message code before choosing between `battle_update.visual_novel` and a separate presentation side message.
- Battle blocking VN must define whether server simulation pauses. The preferred first policy is: blocking cinematic VN pauses battle simulation until advanced/dismissed.

## Confirmed Event Node Policy

Event nodes are scene-chain choice/result interactions, not global VN overlays.

Desired Event node flow:

1. Enter the current Event scene.
2. Unity presents that scene's authored text/script pages. Player click/Enter/Space advances local text presentation; core only needs to provide the ordered dialogue/script identifiers and scene state.
3. If the scene has choices, Unity presents them and sends the selected choice to core.
4. If the scene has no choices and has a next scene, Unity can send an Event-scene advance command after local text presentation is complete.
5. When the scene is completed, core stages that scene's reward table at most once for the current Event session.
6. Choice effects may be either immediate or staged, according to authored effect timing.
7. The selected choice, or a choice-less scene transition, determines the next scene.
8. If a next scene exists, repeat the same flow.
9. If no next scene exists, the Event node ends and staged Event rewards/effects are committed.
10. If the Event flow starts combat, staged Event rewards are committed when that combat resolves, not at combat start.

Additional Event node policy:

- Each Event scene may have its own reward table.
- Event scene reward tables are authored directly as `reward_effects: Vec<RewardEffect>`.
- Scene reward tables are staged on scene completion, not on scene entry.
- The same scene reward table is staged at most once per Event session, even if the graph can revisit the same scene.
- Event choice effects are authored as separate `immediate_effects` and `staged_effects` arrays.
- `StartCombat` is always an immediate effect.
- Event-started combat commits staged Event rewards/effects on victory by default, with authored override support for other policies.
- Event node local text is authored as ordered `script_ids[]`.
- Event node text/dialogue is still node UI content unless it explicitly starts a global VN overlay.
- Event node choices/results remain under Event-node state, while global VN dialogue/presentation remains under `visual_novel`.

## Confirmed Visual Novel Flow Policy

Visual Novel is a global overlay session, not a node screen.

Desired VN flow:

1. Core evaluates VN triggers from the current runtime context.
2. A matching trigger starts or queues a `VisualNovelSession`.
3. The active session owns `session_id`, `source`, `vn_id`, `current_scene_id`, choice/progression state, `blocking`, and `presentation_mode`.
4. Core projects the active session to top-level `visual_novel` in run snapshots, or to the chosen battle/global presentation transport during live battle.
5. If no VN session is active, the root contract exposes `visual_novel: null`.
6. Unity opens the VN overlay whenever `visual_novel` is non-null, regardless of route/screen.
7. Unity resolves IDs locally: background, script, speaker, portrait, choice label, reward hint, penalty hint, and other presentation resources.
8. Unity handles local text presentation details such as typewriter, page reveal, and Click/Enter/Space advancing within a script.
9. When Unity finishes local presentation for a VN scene with no choices and a next scene, it sends `advance_visual_novel`.
10. When a VN scene has choices, Unity displays core-provided choices and sends `select_visual_novel_choice` for the selected choice.
11. Core validates `session_id`, source/session state, current scene, and choice availability before advancing.
12. Core advances to the next scene, applies authored VN-side consequences according to policy, or ends the active session.
13. When the active session ends, the next projection becomes `visual_novel: null` unless another queued VN session becomes active.

Additional VN policy:

- VN can originate from HQ, map, loadout, battle, tutorial, radio, cutscene, or an Event node, but Unity must not use that origin to decide whether to show the overlay.
- `source` is routing/debug/command context, not the display source of truth.
- Unity must not infer scene progression, choice availability, or trigger completion.
- `speakers[]` is the public VN speaker shape, including single-speaker scenes.
- Blocking VN gates background actions in core, not only in Unity UI.
- Battle-time VN delivery must preserve battle update ordering while keeping VN overlay state separate from battle checkpoint/reconcile state.
- Active VN session and queued VN sessions are global runtime state. Do not store them inside Event-node session state.
- VN scenes must carry an explicit text kind. The initial text kinds are `Dialogue`, `Narration`, and `System`.
- VN speaker positions are authored as an enum, not free strings. The initial slots are `FarLeft`, `Left`, `Center`, `Right`, and `FarRight`.
- VN background authoring uses an explicit enum, not `background_id: Option<String>`. The initial variants are `CurrentScreenDimmed` and `Resource { background_id }`.
- Active and queued VN sessions are run runtime state and participate in run checkpoint rewind. Trigger seen/consumed history is separate narrative state and is designed with trigger persistence/repeat policy.
- Event-node-local `speaker_id` and `portrait_id` are legacy presentation fields and should be removed when VN authoring lands. Event node imagery should use Event-node-specific fields such as background/illustration ids, not VN speaker fields.

## Proposed Public DTO Shape

Add a top-level optional field to `RunSnapshotDto`:

```json
"visual_novel": null
```

When active:

```json
"visual_novel": {
  "source": {
    "type": "event_node",
    "node_id": "..."
  },
  "session_id": "...",
  "vn_id": "white_night_confession_01",
  "current_scene_id": "start",
  "background_id": "white_night_chapel",
  "script_id": "white_night_confession_intro",
  "speakers": [
    {
      "speaker_id": "white_night",
      "portrait_id": "white_night_veil",
      "position": "center",
      "active": true
    }
  ],
  "choices": [
    {
      "choice_id": "listen",
      "label_id": "white_night_choice_listen",
      "preview": {
        "risk": "High",
        "starts_combat": false,
        "reward_hint_id": null,
        "penalty_hint_id": "white_night_mark_hint"
      },
      "selectable": true
    }
  ],
  "blocking": true,
  "presentation_mode": "cinematic"
}
```

Open details to verify during implementation:

- Whether `session_id` should be required for idempotent command routing. Prefer required.
- Whether `source.type` needs more first-wave variants than `event_node`, `hq_briefing`, `map`, `loadout`, `battle`, `tutorial`, `radio`, and `cutscene`.
- Whether battle delivery should add `visual_novel` to `LiveBattleUpdateDto` or introduce a separate global presentation side message. Decide only after re-reading the live battle transport and server actor code. In either case, the VN overlay projection must remain separate from battle checkpoint/reconcile state.

## Proposed Runtime Shape

Introduce a global VN module rather than extending Event node code in place:

- `src/game/visual_novel.rs` or `src/game/world/visual_novel.rs`
- `VisualNovelRuntimeState`
  - active session;
  - pending queue;
  - seen/consumed trigger ids;
  - optional battle pause ownership marker;
- `VisualNovelSession`
  - session id;
  - source;
  - vn/event graph id;
  - current scene id;
  - committed choice id;
  - blocking policy;
  - presentation mode;
- `VisualNovelSource`
  - event node source;
  - HQ/source screens;
  - battle source with battle uuid and optional event seq/time;
  - tutorial/radio/cutscene sources.

The existing Event scene graph can be used as the first data source, but the runtime state must not require `ActiveNodeContent::Event`.
Event-node choice/result screens may still exist without an active VN session. Entering an Event node should start a VN session only when authored content actually contains VN/dialogue presentation.

## Proposed Data Shape

Introduce a dedicated VN authoring catalog:

```ron
VisualNovelDefinition(
    id: "white_night_confession_intro",
    entry_scene_id: "start",
    default_blocking: true,
    default_presentation_mode: Cinematic,
    scenes: [
        VisualNovelScene(
            id: "start",
            text_kind: Dialogue,
            background: Resource(background_id: "white_night_chapel"),
            script_id: "white_night_confession_intro",
            speakers: [
                VisualNovelSpeaker(
                    speaker_id: "white_night",
                    portrait_id: Some("white_night_veil"),
                    position: Center,
                    active: true,
                ),
            ],
            next: Scene(scene_id: "choice"),
        ),
    ],
)
```

Narration and system scenes are explicit:

```ron
VisualNovelScene(
    id: "dark_room_intro",
    text_kind: Narration,
    background: CurrentScreenDimmed,
    script_id: "dark_room_intro_narration",
    speakers: [],
    next: Scene(scene_id: "first_voice"),
)
```

Event nodes start VN overlays through trigger data, not inline scene shortcuts:

```ron
VisualNovelTrigger(
    id: "white_night_event_intro",
    source: EventSceneEntered(
        event_id: "white_night_confession_01",
        scene_id: "start",
    ),
    vn_id: "white_night_confession_intro",
    repeat: OncePerRun,
    priority: 100,
    blocking: true,
)
```

Policy direction:

- Add `VisualNovelDefinition` and `VisualNovelTrigger` as dedicated catalogs.
- Do not reuse `EventDefinition` for VN authoring.
- Do not add `vn_on_enter` or similar Event-scene shortcut fields.
- VN scene `text_kind` is required.
- Initial text kinds:
  - `Dialogue`: character dialogue; requires at least one speaker and exactly one active speaker.
  - `Narration`: description/atmosphere; speakers may be empty, and active speakers are optional with at most one active speaker.
  - `System`: warning/tutorial/system presentation; speakers must be empty in the first implementation.
- VN scene `background` is an enum:
  - `CurrentScreenDimmed`: keep the current route/screen behind a dim/blur overlay according to Unity presentation policy.
  - `Resource { background_id }`: display a Unity-local authored background resource.
- VN speaker `position` is an enum with initial values `FarLeft`, `Left`, `Center`, `Right`, and `FarRight`.
- `script_id`, `speaker_id`, `portrait_id`, choice label ids, and hint ids are Unity-local catalog/localization/resource ids. Core must not send localized body text or image bytes.
- Event-node-local `speaker_id`/`portrait_id` should be removed when VN authoring lands. Event-node non-VN imagery should use Event-specific fields such as `illustration_id`.

Legacy example to remove during Phase 3:

```ron
presentation: EventScenePresentation(
    background_id: "white_night_chapel",
    script_id: "white_night_confession_intro",
    speaker_id: Some("white_night"),
    portrait_id: Some("white_night_veil"),
),
```

That legacy Event presentation shape is Event-node-local and must not become the VN authoring shape.

## Phase Plan

This goal is intentionally split into phases. Do not implement the full engine as one large patch. Each phase should update this plan, `EXPERIMENTS.md`, and `EXPERIMENT_NOTES.md`; add focused behavior/contract tests; run focused validation; and stop for policy questions when needed.

### Phase 0: Re-Read And Contract Baseline

Scope:

- Re-read Event data and validation.
- Re-read Event node session/progression.
- Re-read run snapshot DTOs and server snapshot wrapping.
- Re-read action scheduler and validator.
- Re-read battle update transport and server message mapping.
- Re-read current Unity canonical docs.
- Add failing or pending tests only when the desired behavior is already policy-confirmed.

Completion conditions:

- Runtime code, live RON, Unity-facing contracts, and server transport have been rechecked.
- The current Event-node and VN-like payload boundaries are documented in `EXPERIMENTS.md`.
- Any newly discovered policy decisions are reported before implementation.

### Phase 1: Public Contract Skeleton

Scope:

- Add wire-specific `VisualNovelSnapshotDto`.
- Add source, speaker, choice, preview, presentation-mode, and blocking DTOs.
- Add top-level `RunSnapshotDto.visual_novel`.
- Serialize inactive VN as explicit `visual_novel: null` unless a concrete serializer constraint is found.
- Preserve Event-node choice/result state while making `visual_novel` the only VN overlay display source.
- Update server root-key preservation tests.

Completion conditions:

- `RunSnapshotDto` exposes top-level `visual_novel`.
- Unity-facing VN display source is `visual_novel`, not `selected_event.type == "event"`.
- No dual Event/VN presentation source is introduced.
- Server snapshot wrapping preserves `visual_novel`.
- Focused snapshot/JSON contract tests pass.

### Phase 2: Event Node Flow Refactor

Scope:

- Refactor Event node data/runtime toward scene-chain choice/result flow.
- Support choice-less scenes.
- Add per-scene reward tables.
- Add ordered Event-node local `script_ids[]`.
- Stage scene rewards on scene completion, not scene entry.
- Stage the same scene reward at most once per Event session.
- Add explicit `immediate_effects` versus `staged_effects` choice effect timing.
- Treat `StartCombat` as immediate-only.
- Commit staged Event rewards/effects when the Event node ends.
- Commit staged Event rewards/effects when Event-started combat resolves according to authored combat reward policy, defaulting to victory-only.
- Keep Event node text/page ids as node UI content, not global VN overlay content.

Completion conditions:

- Event-node choice/result UI remains a separate concept from VN overlay UI.
- Event scene reward tables are staged and committed according to confirmed policy.
- Revisited Event scenes do not duplicate scene reward staging.
- Choice effects can declare and honor `immediate_effects` versus `staged_effects` timing.
- Event-started combat honors the staged reward/effect commit policy and defaults to victory-only.
- Choice-less Event scenes can advance after Unity local text presentation completes.
- Event snapshots expose ordered Event-node local `script_ids[]`.
- Live RON loading and focused Event-node flow tests pass.

Implementation status as of 2026-07-08:

- Implemented for Event-node data/runtime.
- Event scene data now authors ordered `script_ids[]`.
- Event scenes can author `reward_effects`.
- Event choices now author `immediate_effects`, `staged_effects`, and `combat_reward_commit`.
- Runtime stages scene rewards once per Event session when a scene completes.
- Runtime stages choice staged effects and commits staged Event rewards/effects at Event completion.
- Event-started combat defaults to `OnVictory` staged Event reward/effect commit and supports `OnAnyResolution` / `Never` policy.
- `StartCombat` is immediate-only by schema shape.
- Live Event RON has been migrated away from `script_id` / `effects`.
- Focused Event-node, Event-started combat, live RON loading, snapshot, and server root-field tests pass.

Still intentionally deferred to later phases:

- Global VN runtime/session/trigger engine.
- VN `speakers[]` authoring migration.
- VN commands and blocking action gating.
- Battle-time VN transport.

### Phase 3: VN Authoring And Global Runtime

Scope:

- Add a dedicated `VisualNovelDefinition` catalog.
- Add required VN scene `text_kind`.
- Add `VisualNovelBackground` enum with `CurrentScreenDimmed` and `Resource { background_id }`.
- Add `VisualNovelSpeakerPosition` enum with `FarLeft`, `Left`, `Center`, `Right`, and `FarRight`.
- Replace singular VN speaker authoring with `speakers[]`; do not keep singular aliases without explicit approval.
- Add validation for speaker ids, portrait ids, position enum, active speaker constraints, `text_kind` rules, and background resource ids.
- Introduce global `VisualNovelRuntimeState`.
- Introduce `VisualNovelSession`, queue state, source enum, and projection helper.
- Ensure active/queued VN state lives in global runtime state, not Event session state.
- Store active/queued VN session state in run state and run checkpoints.
- Remove Event-node-local `speaker_id`/`portrait_id` presentation fields, or replace Event-node imagery needs with Event-specific fields such as `illustration_id`.

Completion conditions:

- Public VN DTO uses `speakers[]`.
- Live RON/data supports authored multiple speakers per VN scene through dedicated `VisualNovelDefinition`.
- Live RON/data no longer uses singular `speaker_id`/`portrait_id` authoring for VN presentation.
- VN scenes validate `text_kind`, background enum, speaker position enum, and active speaker rules.
- Core owns active VN session state globally, not through `ActiveNodeContent::Event`.
- Active/queued VN session state rewinds with run checkpoint load.
- Data validation catches missing VN graph, missing scene, invalid choice graph, invalid speaker entry, invalid position slot, and missing trigger references.

Implementation status as of 2026-07-08:

- Implemented the dedicated `VisualNovelDefinition` catalog and live `visual_novels/base.ron` data.
- Implemented required VN `text_kind`, typed `VisualNovelBackground`, typed `VisualNovelSpeakerPosition`, `speakers[]`, choices, previews, and graph validation.
- Implemented global `VisualNovelRuntimeState`, `VisualNovelSession`, source enum, queue state, and root snapshot projection helper.
- Stored active/queued VN session state in run state and run checkpoints.
- Projected an active VN session to top-level `RunSnapshotDto.visual_novel`; inactive sessions serialize as `visual_novel: null`.
- Removed Event-node-local `speaker_id` / `portrait_id` presentation fields from Event data, live RON, snapshot DTOs, and tests.
- Added focused tests for active VN snapshot projection and checkpoint rewind of active VN state.

Still intentionally deferred to later phases:

- `VisualNovelTrigger` data schema, trigger reference validation, repeat policy, and trigger activation belong to Phase 6.
- Global VN progression commands and blocking action gating belong to Phase 4.
- Battle-time VN transport and battle pause behavior belong to Phase 5.
- Unity-facing external docs and probes are finalized in Phase 7 after command and battle transport behavior exists.

### Phase 4: VN Commands And Blocking

Scope:

- Add `AdvanceVisualNovel`.
- Add `SelectVisualNovelChoice`.
- Route VN progression through global VN runtime.
- Remove `AdvanceEventScene`/`SelectEventChoice` as VN commands when the new contract lands. Add a temporary bridge only with explicit user approval and documented removal conditions.
- Add action gating based on active VN state.
- If active VN is blocking, expose only VN commands and explicitly safe system commands.
- If active VN is non-blocking, merge VN commands with normal allowed actions.

Completion conditions:

- Global VN commands advance scenes and select choices independently of active Event node content.
- Core validates session id, source/session state, current scene, and choice availability.
- Blocking VN gates background actions in core.
- Non-blocking VN policy is represented explicitly if implemented.
- Focused command/action-scheduler tests pass.

Implementation status as of 2026-07-08:

- Implemented `PlayerBehavior::AdvanceVisualNovel` and `PlayerBehavior::SelectVisualNovelChoice`.
- Implemented `ActionKind::AdvanceVisualNovel` and `ActionKind::SelectVisualNovelChoice`.
- Implemented global VN progression handlers independent of Event-node active content.
- VN commands validate active session existence, `session_id`, `current_scene_id`, current scene shape, and selected choice id.
- Advancing a VN scene moves to the next scene or closes the active session; closing promotes the next queued session if one exists.
- Selecting a VN choice moves to the authored next scene or closes the active session.
- Blocking active VN sessions gate core allowed actions to VN progression commands only.
- Non-blocking active VN sessions keep background allowed actions and add VN progression commands.
- Command results can return `VisualNovelState { visual_novel }`, while normal non-battle command handling still emits a trailing root snapshot through the server path.

Still intentionally deferred to later phases:

- VN trigger activation and repeat/priority/queue policy remain Phase 6.
- Battle VN transport and battle simulation pause behavior remain Phase 5.
- External Unity docs and live probes remain Phase 7 after battle transport is implemented.

### Phase 5: Battle VN Delivery

Scope:

- Re-read battle update/server actor ordering before choosing transport.
- Decide between `LiveBattleUpdateDto.visual_novel` and a separate global presentation side message.
- Preserve battle update ordering and avoid per-tick full `state_snapshot` battle updates.
- Keep VN overlay projection separate from battle checkpoint/reconcile state.
- Add battle blocking pause policy.
- Add focused battle transport/probe coverage.

Completion conditions:

- Battle-compatible VN delivery exists without reintroducing per-tick full `state_snapshot` battle updates.
- Blocking battle VN pauses or otherwise explicitly controls battle simulation according to documented policy.
- Battle VN projection/cue ordering is tested.
- Unity-facing battle transport docs and probes are updated.

Implementation status as of 2026-07-08:

- Chose `LiveBattleUpdateDto.visual_novel` instead of a separate `visual_novel_update` side message.
- `battle_update.visual_novel` carries the same public VN projection used by root `state_snapshot.visual_novel`.
- Inactive battle VN serializes as `visual_novel: null` on both top-level `battle_update` and nested `battle_resync.update`.
- Active battle VN is attached to battle updates generated by live ticks, battle commands, resync/state requests, and VN progression commands during active battle.
- `advance_visual_novel` / `select_visual_novel_choice` during active battle now return live battle transport: `CommandAccepted` plus a `battle_update` side message, not a trailing full `state_snapshot`.
- Blocking active VN pauses live battle simulation ticks while active. The server does not emit empty tick updates just because the VN is blocking.
- Updated external Unity battle transport docs, Unity core contract excerpts, smoke probe contract, and smoke probe script to include `battle_update.visual_novel`.

Still intentionally deferred to later phases:

- Data-driven battle VN trigger activation remains Phase 6.
- Live WebSocket probe of non-null battle VN requires a trigger/admin path and remains a follow-up after Phase 6.

### Phase 6: Data-Driven Trigger Engine

Scope:

- Define dedicated `VisualNovelTrigger` data schema.
- Add data loading and validation.
- Evaluate non-battle triggers after commands and state transitions.
- Evaluate battle triggers during battle update generation without polluting `BattleLogEvent` with VN-only presentation state.
- Implement trigger repeat, seen/consumed, priority, and queue behavior after policy confirmation.
- Add at least one HQ/map/loadout-compatible trigger entry point.
- Add at least one Event-node trigger source such as `EventSceneEntered` so Event nodes can start VN overlays without inline Event-scene shortcut fields.

Completion conditions:

- HQ/map/loadout-compatible trigger entry points exist, even if only one non-event trigger is live-authored initially.
- Event-node-originated VN flow works through `VisualNovelTrigger`, not `vn_on_enter` or Event effects.
- Trigger repeat/seen/consumed state is tested.
- Queue/priority behavior is tested.
- Data validation catches missing trigger references.
- Unity-facing docs and relevant probes are updated.

Policy stop as of 2026-07-08:

Do not implement Phase 6 until the user confirms the trigger policies below.

- Trigger repeat policy set and semantics: first implementation values such as `OncePerRun`, `OncePerCheckpointedRun`, `OncePerProfile`, `Always`, or a smaller set.
- Trigger consumed timing: mark consumed when matched, when enqueued, when activated, or when the VN session completes.
- Checkpoint behavior for trigger history: which trigger history rewinds with run checkpoints and which survives checkpoint load.
- Multiple matching triggers: priority sort direction, deterministic tie-breaker, and whether lower-priority matches are queued or discarded.
- Queue behavior: whether active VN sessions always queue later triggers, whether a higher-priority trigger can interrupt, and whether blocking/non-blocking sessions can coexist.
- Battle trigger timing: whether battle triggers are evaluated before or after building the current `battle_update`, and whether a newly triggered blocking VN pauses the same tick or the next tick.

### Phase 7: Final Docs And Broad Validation

Scope:

- Update external `unity_core_contract.md`.
- Update external `core_unity_battle_transport_contract.md`.
- Update `docs/game_rulebook.md`.
- Update `docs/core_runtime_contract.md`.
- Add exact VN-focused test names to this plan.
- Run focused validation after each small edit, then broad validation.

Completion conditions:

- Live RON loading passes.
- Focused VN, Event-node, server snapshot, and battle transport tests pass.
- Broad `game_core` validation passes, or failures are documented with cause and follow-up.
- Unity-facing docs and probes match the implemented contract.
- No compatibility alias, fallback schema, ignored legacy test, or dual Event/VN presentation source is introduced without explicit user approval and recorded removal conditions.

## Policy Completion Rule

If implementation reveals a user-facing policy decision not already covered, complete the goal with a policy-decision report instead of guessing. Do this especially for:

- exact VN trigger priority and queue behavior;
- whether multiple VN sessions can stack or only queue;
- whether non-blocking/radio VN can coexist with another blocking VN;
- whether battle blocking VN always pauses simulation;
- whether old `advance_event_scene` / `select_event_choice` commands need a temporary migration bridge despite the default removal policy;
- save data migration policy if persistent saves already exist.

When this rule triggers, mark the goal complete with the policy question list. Do not leave it half-implemented.

## Validation Commands

Start with focused commands and expand as the implementation grows:

```bash
cargo test -p game_core event --lib -- --test-threads=1
cargo test -p game_core game::world::tests::map_flow --lib -- --test-threads=1
cargo test -p game_core game::world::tests::snapshots_and_start --lib -- --test-threads=1
cargo test -p game_core --test ron_loading -- --test-threads=1
cargo test -p game_server player_state_snapshot_preserves_core_snapshot_root_fields -- --test-threads=1
cargo check -p game_server
cargo test -p game_core
```

Introduced VN-focused tests:

- `game::world::tests::snapshots_and_start::run_snapshot_exposes_inactive_visual_novel_as_explicit_null`
