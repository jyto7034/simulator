# Global Visual Novel Engine Notes

## Initial Judgment

The long-term fix is a global VN runtime, not a selected-event DTO reshuffle.

Moving `EventSceneSnapshotDto` from `selected_event.event` to `visual_novel` would improve Unity's read path, but it would not solve:

- HQ briefing VN;
- map/loadout overlay VN;
- battle-time VN triggers;
- blocking overlay action gating;
- battle simulation pause policy;
- multi-speaker scene authoring;
- trigger priority/repeat policy.

The engine should separate:

1. VN graph data;
2. VN runtime/session state;
3. VN trigger evaluation;
4. Unity-facing VN projection;
5. transport delivery for snapshot and battle update contexts.

## Phased Implementation

User policy confirmation: this goal is large enough that implementation should proceed in phases.

Operational guidance:

- Do not implement Event node refactor, global VN runtime, battle delivery, and trigger engine in one patch.
- Each phase should have its own focused tests and validation.
- Finish the public contract skeleton before deeper runtime behavior.
- Refactor Event node flow separately from global VN runtime.
- Add battle VN delivery only after re-reading live battle transport and server actor ordering.
- Add the data-driven trigger engine after VN session/runtime semantics are stable.
- Stop and report policy questions when a phase exposes user-facing behavior not already decided.

## Event Node Is Not VN

User policy clarification: an Event node and a Visual Novel scene are different concepts.

The reference interaction is a Limbus Company-style Event node: the player sees a node-specific UI, reviews situational text/options, selects a choice, and receives a result. That is a choice/result node, not automatically dialogue presentation.

Long-term implications:

- Do not model every Event node as a VN scene.
- Do not require VN to originate from an Event node.
- `selected_event` may still represent node-specific interaction state such as available choices, committed choice, rewards, penalties, or node result flow.
- `visual_novel` represents overlay dialogue/presentation and is route-independent.
- An Event node can start a VN session only when its authored content actually includes VN/dialogue presentation.
- A plain choice/result Event node should remain renderable without an active `visual_novel`.

## Event Node Flow Policy

User policy clarification: Event nodes should be scene-chain interactions.

Expected flow:

- enter Scene 1;
- present that scene's authored text/script pages through Unity-local presentation;
- offer choices if the scene has choices;
- selected choice determines the next scene;
- if the completed scene has a reward table, stage that scene reward;
- end the scene;
- enter the next scene if one exists;
- if no next scene exists, end the Event node and commit staged Event rewards/effects.

Choice-less scenes are valid. After local text presentation completes, Unity may ask core to advance to the next scene or end the Event node.

Reward policy:

- each Event scene may own a reward table;
- scene rewards are staged when the scene is completed, not when entered;
- the same scene reward is staged at most once per Event session;
- staged Event rewards commit when the Event node ends;
- if the Event flow starts combat, staged Event rewards commit when that combat resolves, not when combat starts.

Effect timing policy:

- authored choice effects may be immediate or staged;
- immediate effects apply when the choice is accepted;
- staged effects accumulate with scene rewards and commit at Event completion or Event-started combat resolution;
- the schema should make timing explicit so designers and tests can see which effects are immediate.

Text/page policy:

- Event node text can be shown as multiple click/Enter/Space-advanced script pages;
- Unity implements the page-by-page presentation behavior;
- core should expose ordered ids/state cleanly and should not send localized body text;
- this local Event-node text presentation is separate from global `visual_novel`.

## Phase 2 Resolved Event Schema Policy

User approved the recommended Phase 2 Event-node schema direction.

Resolved policy:

- Scene reward table shape: each scene owns `reward_effects: Vec<RewardEffect>` directly.
- Choice effect timing shape: choices use separate `immediate_effects` and `staged_effects` arrays.
- Combat-start timing: `StartCombat` is always immediate.
- Combat resolution payout condition: default staged Event reward/effect commit is victory-only, with authored override support.
- Event text page shape: Event-node local text uses ordered `script_ids[]`.

Implementation implications:

- Do not add a separate Event reward table catalog in Phase 2.
- Do not model timing as a per-effect `timing` field.
- Do not allow staged `StartCombat`.
- Add an authored combat reward commit policy rather than hard-coding every Event-started combat to the same payout behavior.
- Remove singular `script_id` from Event-node local text authoring in the Phase 2 schema migration.

Implemented runtime notes:

- Event-node `script_ids[]` is local Event UI text/page identity, not global VN overlay script state.
- Event scenes stage `reward_effects` when they are completed through `advance_event_scene` or `select_event_choice`.
- Event sessions remember completed scene ids so a revisited scene cannot stage its scene reward twice.
- `immediate_effects` are planned on cloned runtime state before commit; if combat start validation fails, immediate grants are not committed.
- `staged_effects` accumulate with scene rewards and are granted only at Event-node completion or Event-started combat resolution.
- Event-started combat uses `EventCombatRewardCommitPolicy`, defaulting to `OnVictory`; `OnAnyResolution` commits staged Event rewards during non-victory post-battle resolution, and `Never` suppresses staged Event reward commit through combat resolution.
- Duplicate immediate `StartCombat` is now rejected as authored data invariant rather than handled as a runtime partial-failure case.

## Visual Novel Flow Policy

User policy confirmation: VN is a global overlay session that can appear over any screen.

Expected flow:

- core evaluates VN triggers from runtime state;
- matching trigger starts or queues a `VisualNovelSession`;
- active session projects as top-level `visual_novel` for normal run snapshots, or through the selected battle/global presentation transport during battle;
- Unity shows the overlay whenever `visual_novel` is non-null;
- Unity resolves all presentation ids locally and does not receive dialogue body text or image payloads;
- Unity handles local text effects and Click/Enter/Space advancement inside the current script;
- Unity sends `advance_visual_novel` only after local presentation has completed and the current VN scene has no choices;
- Unity sends `select_visual_novel_choice` when the user selects a core-provided selectable choice;
- core validates session/source/current scene/choice availability before changing VN state;
- core moves to the next scene or ends the session;
- when the session ends, the projection becomes `visual_novel: null` unless a queued VN becomes active.

Long-term implications:

- VN session state is global runtime state, not Event session state.
- `source` explains where the VN came from but is not the display source of truth.
- route/screen does not determine whether Unity shows VN; `visual_novel` does.
- blocking VN must gate background commands in core.
- battle VN transport must preserve ordering without turning battle checkpoint state into VN state.

## Current Speaker Model

Current runtime/data is single-speaker:

- `EventScenePresentation.background_id: String`
- `EventScenePresentation.script_id: String`
- `EventScenePresentation.speaker_id: Option<String>`
- `EventScenePresentation.portrait_id: Option<String>`
- `EventSceneSnapshotDto.speaker_id: Option<String>`
- `EventSceneSnapshotDto.portrait_id: Option<String>`

This is not enough for VN scenes with several visible characters.

Preferred direction:

- replace singular speaker fields with `speakers: Vec<VisualNovelSpeakerDef>`;
- each speaker has `speaker_id`, optional `portrait_id`, `position`, and `active`;
- validate position slots;
- validate active speaker policy;
- allow empty speaker array only if narration/system text is explicitly supported;
- project the same shape in `VisualNovelSnapshotDto.speakers`;
- migrate existing single-speaker live RON to one-element `speakers` arrays when the schema changes.

Avoid:

- keeping singular and array fields together as a long-term dual schema;
- keeping singular authoring aliases for `speaker_id` / `portrait_id` without explicit approval;
- silently converting multiple authored speakers into only one active speaker;
- making Unity infer active speaker from script id.

## Runtime Boundary

`EventSessionState` should not become the global VN session type by accretion. It contains `node_id` and Event-started combat details, which are Event-node concerns.

Prefer a new `VisualNovelSession` with a source enum:

- Event node source can carry `node_id`.
- Battle source can carry `battle_uuid`, optional seq/time, and trigger id.
- HQ/map/loadout/tutorial/radio/cutscene sources can carry their own minimal routing identifiers.

Event node can use the global VN session, but the global VN session should not depend on Event node.

## Phase 3 Resolved Authoring Policy

User policy update: VN data will be produced and validated by an authoring/management program, not hand-authored directly in raw RON. Prefer explicit schemas, enums, and strict validation even if the data shape is more verbose.

Resolved policy:

- VN graph/catalog shape: add a dedicated `VisualNovelDefinition` catalog. Do not reuse `EventDefinition` as the VN graph schema.
- VN trigger shape: add a dedicated `VisualNovelTrigger` table. Event nodes, HQ, map, loadout, battle, tutorial, radio, and cutscene sources should all enter VN through trigger data.
- Event-node VN bridge: do not add an Event-scene `vn_on_enter` shortcut and do not model VN start as an Event gameplay effect. Event nodes can trigger VN overlays through `VisualNovelTrigger`, for example `EventSceneEntered(event_id, scene_id)`.
- VN scene text kind: every VN scene must have `text_kind`.
- Initial `VisualNovelTextKind` values:
  - `Dialogue`: character dialogue; requires at least one speaker and exactly one active speaker.
  - `Narration`: situation/atmosphere narration; permits empty speakers, and permits at most one active speaker if visible characters are authored.
  - `System`: warning/tutorial/system presentation; requires empty speakers in the first implementation.
- Speaker position slots: use a typed enum, not free strings. Initial values are `FarLeft`, `Left`, `Center`, `Right`, and `FarRight`.
- Background policy: use a typed `VisualNovelBackground` enum, not `background_id: Option<String>`.
  - `CurrentScreenDimmed`: keep current route/screen behind Unity dim/blur presentation.
  - `Resource { background_id }`: display an authored Unity-local background resource.
- Active VN checkpoint policy: active and queued VN sessions are run runtime state and must be included in run checkpoints. Loading a checkpoint rewinds active/queued VN session state with the run.
- Trigger history persistence: seen/consumed trigger history is separate narrative state. Repeat policy can distinguish run-local/checkpoint-rewound state from longer-lived profile/story state in the trigger phase.
- Event-node local presentation cleanup: remove Event-node-local `speaker_id`/`portrait_id` when VN authoring lands. Event node imagery should use Event-specific fields such as `illustration_id` rather than VN speaker fields.

Implementation implications:

- Phase 3 should create VN authoring/runtime primitives without adding a shortcut Event field.
- Phase 3 can introduce runtime state and definition projection first, but Event-node-originated VN activation should wait for `VisualNovelTrigger` rather than hard-coding Event flow.
- Phase 6 should own trigger repeat/seen/consumed policy and non-Event trigger sources.
- Management tooling can expose typed dropdowns and validation for text kind, speaker position, background variant, and trigger source.

Implemented runtime notes:

- `VisualNovelDefinition` is now a dedicated live data catalog loaded by `GameDataBase`.
- VN authoring uses required `text_kind`, typed background, typed speaker position, and `speakers[]`; singular VN speaker aliases were not introduced.
- `CurrentScreenDimmed` remains explicit in authored data and projects to public `background_id: null`, matching the current Unity-facing root DTO shape.
- Active and queued VN sessions live in global run runtime state, not Event session state.
- Run checkpoint save/load rewinds active and queued VN session state.
- Root snapshot projection reads only the active global VN session. Event-node scene payload is not copied into `visual_novel`.
- Event-origin activation is intentionally not hard-coded. It waits for the Phase 6 `VisualNovelTrigger` engine.
- Event-node-local `speaker_id` / `portrait_id` fields have been removed; future Event-node imagery should use Event-specific fields rather than VN speaker fields.

## Trigger Boundary

Do not put VN-only presentation triggers into `BattleLogEvent` unless they are also meaningful battle replay/gameplay events.

Better options:

- battle trigger evaluator observes battle update generation, event log seqs, battle time, and checkpoint state;
- it starts/enqueues a VN session;
- the resulting projection is delivered beside battle update data.

This preserves the battle event log as gameplay/presentation timeline source for battle actions, while VN remains global presentation/story state.

## Action Gating

Blocking VN must affect allowed actions. Otherwise Unity can hide inputs locally, but stale or custom clients could still submit background commands while core believes the overlay should be modal.

Preferred policy:

- blocking cinematic VN: only VN advance/choice commands plus explicitly safe system commands;
- non-blocking/radio VN: normal actions plus VN advance/dismiss commands;
- battle blocking VN: battle simulation pauses while active unless later policy says otherwise.

This belongs in core allowed-action calculation, not only Unity UI.

Implemented command/gating notes:

- First-wave VN commands are `advance_visual_novel` and `select_visual_novel_choice`.
- Both commands require `session_id` and `current_scene_id`; choice selection also requires `choice_id`.
- Core rejects VN commands if there is no active session, the session id does not match, the current scene id does not match, the scene shape is wrong for the command, or the choice id is not authored for the current scene.
- Blocking VN currently exposes only VN progression commands as allowed actions. No extra "safe system command" is included yet because none has been explicitly classified as safe for every blocking overlay context.
- Non-blocking VN keeps the background state's allowed actions and adds VN progression commands.
- Ending an active VN session promotes the next queued session if one exists, otherwise `visual_novel` becomes null.
- Battle-time blocking still needs Phase 5 transport and simulation pause implementation; Phase 4 only gates command capability from the global run action path.

## Battle Transport

Live battle intentionally avoids per-tick full state snapshots. A battle VN solution must not undo that.

Two possible approaches:

1. Add optional `visual_novel` to `LiveBattleUpdateDto`.
2. Add a separate top-level `visual_novel_update` or global presentation side message.

No final choice yet. Re-check server message ordering and live battle update code before implementing. In either approach, keep VN overlay projection separate from checkpoint/reconcile state.

Implemented transport notes:

- Phase 5 chose option 1: `LiveBattleUpdateDto.visual_novel`.
- Rationale: existing battle commands, ticks, and resync already share `battle_update`; adding the projection there preserves ordering without introducing a second presentation side-channel.
- `visual_novel` is outside `checkpoint` because VN overlay state is global presentation/session state, not durable battle actor/HUD state.
- `battle_update.visual_novel` is explicit `null` when inactive.
- `battle_resync.update.visual_novel` follows the same shape because `battle_resync.update` is a nested `LiveBattleUpdateDto`.
- Battle-time `advance_visual_novel` / `select_visual_novel_choice` are converted to battle transport results while the game state is `InBattle`: command ack plus side `battle_update`, no trailing full `state_snapshot`.
- Blocking active VN pauses live battle tick simulation. The pause is not represented by mutating `checkpoint.playback.paused`; playback remains the player-visible speed/pause state, while blocking VN is a modal overlay gate.
- Live non-null battle VN smoke probing is deferred until the trigger/admin path can start a VN session in an actual WebSocket flow.

## Checkpoint And Persistence Policy

Story seen/unlock state may need different persistence from current active VN state.

Potential split:

- active VN session: transient current runtime state, possibly checkpoint-rewound if it belongs to run progress;
- seen trigger ids / story unlocks: player-facing narrative history, likely preserved across checkpoint load;
- trigger repeat state: may be per-run or permanent depending on trigger kind.

This needs user confirmation if implementation reaches persistent save/checkpoint behavior.

## Phase 6 Policy Questions

The trigger engine should not be implemented until these are confirmed:

- Which repeat policies are valid in the first schema?
- Is a trigger consumed when it matches, when it is enqueued, when it becomes active, or when its VN session completes?
- Which trigger history rewinds on run checkpoint load?
- Which trigger history is longer-lived story/profile state?
- If several triggers match in one evaluation pass, is higher numeric `priority` first?
- What is the deterministic tie-breaker for equal priority: trigger id, author order, or validation error?
- Are lower-priority matches queued after a higher-priority active session, or discarded until the next evaluation point?
- Can a higher-priority trigger interrupt an active VN session?
- Can a non-blocking/radio VN coexist with a blocking cinematic VN, or must all sessions use one queue?
- For battle triggers, does a newly matched blocking VN pause the same server tick before simulation, or does it appear in the current `battle_update` and pause following ticks?

## Things To Avoid

- Adding `visual_novel` while leaving Unity to also read `selected_event.event`.
- Keeping `AdvanceEventScene` as the only progression command for global VN.
- Hard-coding HQ or battle VN starts directly in UI-specific code paths when data-driven trigger rules would work.
- Adding fallback parsing for both singular speaker fields and speaker arrays without a removal plan.
- Sending dialogue body text or image data from core.
- Reintroducing battle full snapshots as the live battle VN delivery mechanism.
- Using ignored tests to preserve legacy Event UI behavior.

## Out Of Scope For First Implementation Slice

- Unity overlay implementation.
- Full script/localization catalog authoring in Unity.
- Save-file migration unless persistent saves already need it.
- Advanced VN scripting language features beyond scene graph, choices, triggers, blocking, presentation mode, and speakers.
- Branch-local variables or expression scripting unless trigger/choice conditions require them.
- Voice/audio sequencing unless existing resource IDs already cover it.

## Follow-Up Candidates

- Dedicated VN authoring documentation for designers.
- A VN graph linter that reports unreachable scenes, missing script ids, invalid speaker ids, and ambiguous active speaker settings.
- Debug/admin command to inspect active VN session and trigger history.
- WebSocket probe specifically for battle-time VN trigger delivery.
- Unity-side contract examples for cinematic, radio, compact, and non-blocking modes.
