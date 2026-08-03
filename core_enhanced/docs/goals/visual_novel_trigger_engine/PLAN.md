# Visual Novel Trigger Engine

## Objective

Implement a long-term, data-driven Visual Novel trigger engine so core can start
or queue global VN sessions from authored runtime conditions instead of
hard-coded call sites.

The product goal is:

```text
Game/runtime event occurs
-> core evaluates VisualNovelTrigger live data
-> matching trigger starts or queues a VisualNovelSession
-> Unity renders the existing state.visual_novel / battle_update.visual_novel projection
```

This goal must not turn VN into a route-specific UI feature. VN remains a global
overlay driven by the existing top-level `GameCoreState.visual_novel` runtime.
Unity should continue to decide overlay visibility from public `visual_novel`
projection, not from `selected_event`, route, or trigger details.

## Source Of Truth Order

Use this order during implementation:

1. Current runtime code and focused tests.
2. Live RON/data loaded by `GameDataBase::load_live_embedded()`.
3. Unity-facing snapshot/command/battle transport contract.
4. Latest policy documents, especially `docs/goals/pre_run_global_visual_novel_core/`.
5. Older goal documents only as historical context.

Do not trust older `global_visual_novel_engine` wording where it conflicts with
the current top-level VN runtime policy. In particular, current runtime ownership
is `GameCoreState.visual_novel`, not `RunState.visual_novel`.

## Current Runtime Baseline

Confirmed by code review before this goal:

- `GameCoreState.visual_novel` is the canonical VN runtime owner.
- `VisualNovelRuntimeState` already has `active` and `queue`.
- Root snapshot projects `state.visual_novel` from the top-level VN runtime.
- Battle transport attaches the same projection to `battle_update.visual_novel`.
- Blocking VN gates `allowed_actions` to VN progression commands.
- Blocking battle VN pauses later server battle ticks.
- `StartNewGame(game_mode)` currently starts HQ briefing through a hard-coded
  call to `activate_hq_briefing_visual_novel(game_mode)`.
- Live VN definitions exist in `game_resources/data/visual_novels/base.ron`.
- There is no dedicated trigger catalog yet.

## Confirmed Product Policy

- Trigger data is authored in RON, then later through a management/authoring
  program.
- Rust core is the condition evaluator. Unity does not evaluate trigger
  conditions.
- Core must not send trigger internals to Unity.
- Unity-facing DTO shape should remain stable. Trigger engine should continue
  using existing `visual_novel` projection.
- `script_id` remains a Unity-local VN script/cue resource id. Core does not
  send dialogue body text, images, sounds, or cue payloads.
- Event nodes and VN overlays remain separate concepts. Event nodes are
  choice/result gameplay nodes; VN is global presentation/story overlay.
- Do not add event-scene shortcut fields such as `vn_on_enter`. Event-originated
  VN must come through `VisualNovelTrigger`.
- Missing trigger references are invalid live data. Do not silently skip or
  fallback to another VN id.

## Trigger Data Model

Add a dedicated live-data catalog:

```text
src/game/data/visual_novel_trigger_data.rs
game_resources/data/visual_novel_triggers/base.ron
```

Initial shape:

```ron
VisualNovelTriggerDatabase(
    triggers: [
        VisualNovelTrigger(
            id: "run_start_standard_hq_briefing",
            source: RunStart(game_mode: Standard),
            vn_id: "hq_briefing_standard_intro",
            repeat: OncePerRun,
            checkpoint_policy: PersistAcrossCheckpointLoad,
            priority: 100,
            blocking: true,
        ),
    ],
)
```

Required fields:

- `id`: unique trigger id.
- `source`: runtime event/condition family.
- `vn_id`: referenced `VisualNovelDefinition`.
- `repeat`: how often this trigger may fire.
- `checkpoint_policy`: whether consumed history rewinds with run checkpoint load.
- `priority`: higher number evaluates first.
- `blocking`: runtime session blocking flag.

Optional future fields may include extra conditions, tags, authoring notes, or
debug labels, but do not add them until a concrete runtime need exists.

## Initial Trigger Sources

Phase 1 source set:

```text
RunStart { game_mode }
EventSceneEntered { event_id, scene_id }
MapNodeEntered { category, kind_id }
BattleStarted { encounter_id, node_type, mission_variant }
BattleTimeReached { battle_time_ms }
```

Out-of-scope for first implementation unless this plan is updated:

```text
UnitHpBelow
ObjectiveDiscovered
ObjectiveDestroyed
BossPhaseChanged
AbnormalityResearchReached
SkillFragmentAwakened
CommandFailed
OncePerProfile tutorial triggers
```

These are valid long-term sources, but they carry additional policy or runtime
state complexity.

## Repeat Policy

Initial repeat values:

```text
Always
OncePerRun
OncePerBattle
OncePerNode
```

Do not implement `OncePerProfile` in this goal unless durable profile storage
policy is explicitly added first.

Meaning:

- `Always`: may fire whenever the source matches.
- `OncePerRun`: fires once per run, subject to `checkpoint_policy`.
- `OncePerBattle`: fires once per battle UUID.
- `OncePerNode`: fires once per map node/session.

## Checkpoint Policy

Use explicit trigger-level checkpoint policy:

```text
PersistAcrossCheckpointLoad
RewindWithRunCheckpoint
```

Policy meaning:

- `PersistAcrossCheckpointLoad`: once consumed, the trigger remains consumed
  after run checkpoint load.
- `RewindWithRunCheckpoint`: consumed history rewinds to the checkpoint payload
  on run checkpoint load.

Long-term direction:

```text
Trigger repeat history is split by checkpoint_policy.
PersistAcrossCheckpointLoad history lives in top-level server-session runtime.
RewindWithRunCheckpoint history is included in run checkpoint payload.
```

This is the preferred long-term design. Do not collapse it to a single global
history as a shortcut.

## Consumed Timing

Mark a trigger consumed when its VN session is successfully activated or queued.

Do not mark consumed merely because a trigger matched. Do not wait until VN
completion to mark consumed, because the same trigger could be queued again
before the active session ends.

## Priority And Queue Policy

Ordering:

```text
higher priority first
same priority -> trigger id ascending
```

Queue behavior:

- One active VN session at a time.
- No interruption in this goal.
- If another trigger fires while a VN is active, enqueue it.
- Blocking and non-blocking sessions share the same active slot and queue in
  this goal.

Non-blocking concurrent radio/cinematic layering is a future feature and should
not be introduced as a hidden compatibility path.

## Battle Timing Policy

For battle triggers:

```text
advance battle / build battle_update
-> evaluate battle trigger events from the produced state/events
-> attach visual_novel to the same battle_update before sending
```

A newly triggered blocking VN pauses the next server battle tick. Do not attempt
to retroactively undo the battle simulation tick that produced the trigger.

For time thresholds such as `BattleTimeReached { battle_time_ms }`, evaluate
threshold crossing:

```text
previous_battle_time_ms < threshold <= current_battle_time_ms
```

This avoids repeated firing under pause, speed changes, and repeated state
requests.

## Unity-Facing Contract Policy

Do not expose trigger internals in public DTOs.

Keep using:

```text
state.visual_novel
battle_update.visual_novel
```

The following should not be added to Unity-facing DTOs in this goal:

- trigger id
- priority
- repeat policy
- checkpoint policy
- consumed history
- internal condition details

If a future source variant needs more origin metadata, extend
`VisualNovelSourceDto` narrowly and document it. Avoid broad DTO churn.

## Implementation Phases

## Implementation Status

Status as of 2026-07-09: Phases 0 through 7 are complete in core.

Implemented:

- Added strict `VisualNovelTrigger` live data and embedded loading.
- Replaced hard-coded HQ briefing activation with `RunStart { game_mode }`
  trigger evaluation.
- Added trigger runtime consumed history split by checkpoint policy.
- Added trigger evaluation for `RunStart`, `EventSceneEntered`,
  `MapNodeEntered`, `BattleStarted`, and `BattleTimeReached`.
- Kept Unity-facing DTO shape stable: no trigger internals are exposed.
- Added focused tests for live RON loading, missing RunStart trigger failure,
  checkpoint rewind/persist behavior, event/map/battle trigger hooks,
  priority/queue behavior, and battle-time threshold crossing.
- Updated core runtime/game-flow docs.

Not changed:

- No event-scene `vn_on_enter` shortcut was added.
- No compatibility layer, dual schema, or legacy hard-coded HQ briefing fallback
  was kept.
- No Unity-facing command or DTO shape was changed for trigger internals.
- No live server/probe update was required because observable transport shape
  did not change.

### Phase 0: Re-Read And Baseline

Scope:

- Re-read current VN runtime/projection/action gating/battle transport.
- Re-read `StartNewGame`, event node flow, map node flow, live battle advance,
  checkpoint save/load, and live RON loading.
- Re-read current Unity probe and contract docs.

Completion conditions:

- Current insertion points are recorded in `EXPERIMENTS.md`.
- Any policy conflict discovered in code is recorded before implementation.

### Phase 1: Trigger Data Catalog And Validation

Scope:

- Add `visual_novel_trigger_data` module.
- Add `VisualNovelTriggerDatabase`.
- Add live RON file.
- Add `GameDataBase` field, builder plumbing, embedded loading, and index
  validation.
- Add cross-reference validation from triggers to `visual_novel_data`.
- Add live RON load tests.

Completion conditions:

- Duplicate trigger ids fail validation.
- Missing `vn_id` fails validation.
- Empty ids and invalid fields fail fast.
- Live embedded data loads through the official path.

### Phase 2: Trigger Runtime State

Scope:

- Add `VisualNovelTriggerRuntimeState`.
- Track consumed history for both checkpoint policies.
- Add checkpoint payload support for `RewindWithRunCheckpoint` history.
- Ensure `PersistAcrossCheckpointLoad` history survives checkpoint load.
- Ensure `RewindWithRunCheckpoint` history rewinds with checkpoint load.

Completion conditions:

- Focused checkpoint tests cover both policies.
- Trigger history is not mixed with active VN session state.
- No `RunState.visual_novel` owner is reintroduced.

### Phase 3: Evaluation Engine

Scope:

- Add `VisualNovelTriggerEvent` runtime enum.
- Add `evaluate_visual_novel_triggers(event)` helper.
- Match source, apply repeat/history, sort by priority/id, create
  `VisualNovelSession`, and activate or queue.
- Mark consumed only after successful activation/queue.
- Preserve existing `VisualNovelSource` projection.

Completion conditions:

- Matching trigger opens `state.visual_novel`.
- Active VN causes later triggers to queue.
- Completing active VN promotes queued VN.
- Priority and tie-breaker behavior is tested.

### Phase 4: Replace Hard-Coded RunStart VN

Scope:

- Replace direct `activate_hq_briefing_visual_novel(game_mode)` call with
  `RunStart { game_mode }` trigger evaluation.
- Live trigger data must contain Standard and Endless HQ briefing triggers.
- `StartNewGame` should fail if required RunStart trigger data cannot open a VN.

Completion conditions:

- `StartNewGame(Standard)` opens HQ briefing through trigger data.
- `StartNewGame(Endless)` opens its mode-specific trigger.
- Missing RunStart trigger or missing VN id fails as invalid live data.
- Unity-facing `RunStartFlowStarted` and root `visual_novel` shape remain
  unchanged.

### Phase 5: Event And Map Hooks

Scope:

- Evaluate `EventSceneEntered` after event node scene/session changes.
- Evaluate `MapNodeEntered` after confirmed node entry.
- Ensure Event node gameplay snapshot and VN overlay stay separate.

Completion conditions:

- Event-originated VN starts through `VisualNovelTrigger`, not event scene
  shortcut fields.
- Map-node VN can appear over the current route.
- Existing Event choice/reward/combat behavior remains correct.

### Phase 6: Battle Hooks

Scope:

- Evaluate `BattleStarted` during live battle start transport.
- Evaluate `BattleTimeReached` from produced battle update time crossing.
- Attach newly triggered VN to the same `battle_update.visual_novel`.
- Ensure blocking VN pauses subsequent battle ticks, not the already-produced
  tick.

Completion conditions:

- Battle intro VN appears in battle update transport.
- Timed VN appears once when crossing the threshold.
- Repeated battle state requests do not refire timed triggers.
- Battle probe/docs are updated if observable transport examples change.

### Phase 7: Docs, Probe, And Wide Validation

Scope:

- Update core runtime docs.
- Update Unity-facing docs only if observable contract examples need refresh.
- Update probe scripts if a live-authored trigger changes smoke flow.
- Run focused tests after each phase and broad tests at the end.
- Run live server/probe if Unity-facing transport examples changed.

Completion conditions:

- `cargo test --lib -- --test-threads=1` passes in core.
- `cargo test --lib -- --test-threads=1` passes in game_server if server
  transport/tests are touched.
- Any live server probe changes are verified against a real `game_server`.

## Completion Criteria

The goal is complete when:

1. `VisualNovelTrigger` live data is loaded and validated.
2. RunStart HQ briefing is trigger-driven, not hard-coded.
3. Event scene and map node triggers can start VN overlays without route-specific
   shortcuts.
4. Battle started/time triggers can attach VN to battle transport.
5. Repeat, consumed timing, priority, queueing, and checkpoint policies are
   tested.
6. Unity-facing `visual_novel` DTO shape remains stable unless a documented,
   user-approved extension is required.
7. Missing trigger/VN references fail fast.
8. Legacy hard-coded HQ briefing activation is removed.
9. PLAN, EXPERIMENTS, and EXPERIMENT_NOTES are updated with final results.

## Stop Conditions

Stop and report policy questions instead of guessing if implementation requires:

- Adding trigger internals to Unity DTOs.
- Changing VN command payload shape.
- Changing `script_id` from Unity-local resource id to core-authored dialogue.
- Introducing concurrent active VN sessions.
- Supporting `OncePerProfile` without durable profile storage policy.
- Deciding a new story-critical checkpoint rewind rule not covered here.
- Reintroducing `RunState.visual_novel` or another dual-owner fallback.
- Adding event-scene `vn_on_enter` or other shortcut trigger fields.
