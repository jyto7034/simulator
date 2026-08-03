# Pre-Run Global Visual Novel Core Design Notes

Status: historical design input / goal implemented

This file preserves the design reasoning that fed the completed
[Pre-Run Global Visual Novel Core goal](../goals/pre_run_global_visual_novel_core/PLAN.md). It is not the current runtime,
gameplay, data, or Unity wire source of truth. Current policy lives in
[`core_runtime_contract.md`](../core_runtime_contract.md),
[`data_loading_contract.md`](../data_loading_contract.md), and the external
`/mnt/f/unity projects/ark/docs/unity_core_contract.md`. Implementation evidence
and corrections to the original boundary live in
[the goal experiments](../goals/pre_run_global_visual_novel_core/EXPERIMENTS.md).

The implemented disposition is a single top-level `GameCoreState.visual_novel`,
`PreRunBriefing`, `RunStartFlowStarted`, and live-authored `RunStart { game_mode }`
triggers. Recommendations and open questions below are retained as the
point-in-time design input and must not be read as unresolved current work.

This was a temporary design note for the long-term core work needed to support a main-menu
mode selection followed by an HQ briefing Visual Novel before starter employee
selection.

This note has been updated after the policy discussion that produced
`docs/goals/pre_run_global_visual_novel_core/`. The important policy change from
the first draft is that VN runtime should have one canonical top-level owner,
not a long-term `global_visual_novel` plus `run.visual_novel` priority fallback.

## Desired Product Flow

```text
Main Menu
-> StartNewGame(game_mode)
-> pre-run global state.visual_novel non-null HQ Briefing
-> advance_visual_novel after Unity-local script/cue exhaustion
-> core transitions to SelectingStarterEmployees
-> SelectStarterEmployees
-> RunState is created with the selected game_mode
-> ViewingMap
```

## Original Core Reality Before The Goal

- `StartNewGame { game_mode }` already existed.
- `GameMode` already supported `Standard` and `Endless`.
- `StartNewGame` immediately transitioned to `SelectingStarterEmployees`.
- `RunState` was not created until `SelectStarterEmployees`.
- VN runtime lived under `RunState.visual_novel`.
- Because there was no `RunState` immediately after `StartNewGame`, core could
  not naturally emit a non-null pre-run `state.visual_novel`.

## Implemented Direction

The active goal moved this design from proposal toward implementation:

- `GameCoreState.visual_novel` is the canonical VN runtime owner.
- `StartNewGame(game_mode)` starts `PreRunBriefing` and returns
  `RunStartFlowStarted { game_mode, visual_novel }`.
- HQ briefing completion prepares starter candidates and transitions to
  `SelectingStarterEmployees`.
- `RunState` is still created by `SelectStarterEmployees`.
- Run checkpoint state no longer restores or depends on `RunState.visual_novel`.

## Recommended Long-Term Core Shape

Move VN runtime out of `RunState` and make it a top-level core runtime system:

```text
GameCoreState
-> visual_novel: VisualNovelRuntimeState
-> run: Option<RunState>
```

`GameCoreState.visual_novel` is the canonical owner. `RunState.visual_novel`
should be removed or made non-canonical during the implementation goal.

Do not keep a long-term dual-owner projection such as:

```text
global_visual_novel.active else run.visual_novel.active
```

That fallback would solve the immediate pre-run case but preserve two sources of
truth. VN should be route-independent; HQ, map, loadout, battle, event-node,
tutorial, radio, and cutscene origins should be represented by
`VisualNovelSource`, not by separate runtime owners.

Root snapshot projection should use:

```text
state.visual_novel = GameCoreState.visual_novel.active else null
```

Battle transport can continue to expose VN through `battle_update.visual_novel`,
but it should attach the same top-level projection. VN overlay state remains
outside battle checkpoint/reconcile state.

## Recommended State Flow

Prefer adding an explicit state:

```text
GameState::PreRunBriefing
```

Recommended flow:

```text
NotStarted
-> StartNewGame(game_mode)
-> store pending_game_mode
-> activate GameCoreState.visual_novel session { source: HqBriefing, ... }
-> PreRunBriefing
-> advance_visual_novel
-> HQ briefing VN ends
-> SelectingStarterEmployees
-> SelectStarterEmployees
-> create RunState using pending_game_mode
-> ViewingMap
```

Avoid hiding this flow inside `NotStarted` or `SelectingStarterEmployees`; a
dedicated state keeps allowed actions, snapshots, tests, and Unity routing easier
to reason about.

## Required Core Changes From The Original Boundary

1. Add top-level `GameCoreState.visual_novel: VisualNovelRuntimeState`.
2. Remove or stop using `RunState.visual_novel` as a source of truth.
3. Update root `visual_novel` snapshot projection to read top-level VN runtime.
4. Update battle `battle_update.visual_novel` attachment to use the same top-level projection.
5. Add `GameState::PreRunBriefing`.
6. Add Unity-facing `GameStateContextDto::PreRunBriefing { game_mode }`.
7. Change `StartNewGame(game_mode)` so it starts the run-start flow:
   - initializes player/session-level state;
   - clears previous run/node/battle state;
   - stores `pending_game_mode`;
   - activates the HQ briefing VN in top-level VN runtime;
   - transitions to `PreRunBriefing`;
   - does not create `RunState` yet;
   - does not expose starter candidates yet.
8. Change `StartNewGame` command result shape so it reflects run-start flow start, not immediate starter selection.
9. Update `advance_visual_novel` handling so ending the HQ briefing VN transitions to `SelectingStarterEmployees`.
10. Prepare starter employee candidates when transitioning to `SelectingStarterEmployees`.
11. Ensure `SelectStarterEmployees` still creates `RunState` from `pending_game_mode`.
12. Update allowed actions so blocking pre-run VN allows only VN progression actions.
13. Ensure `AdvanceVisualNovel` and `SelectVisualNovelChoice` can run without `RunState`.
14. Keep Unity-local line/cue cursor out of core command payloads.

## Confirmed Policy

- `StartNewGame(game_mode)` means "begin run-start flow", not "create the run immediately".
- `pre-run` means before `RunState` creation, not before `StartNewGame`.
- HQ briefing VN starts after `StartNewGame(game_mode)` succeeds and before starter employee selection.
- `RunState` is created only after starter employees are selected.
- VN runtime has one canonical owner: `GameCoreState.visual_novel`.
- Unity opens VN from the public `visual_novel` projection, not from `selected_event` or route-specific UI state.
- Event nodes and VN overlays remain separate concepts.
- Core owns VN session id, source, VN id, current scene id, choice availability, blocking policy, and progression.
- Unity owns local script/cue presentation, text reveal, background changes, portrait/expression changes, audio hooks, fade, shake, and camera effects.
- `script_id` is a Unity-local VN presentation script resource id. Core must not send localized dialogue body text, image data, or Unity cue payloads.
- Starter candidates should be exposed when `SelectingStarterEmployees` is active, not while blocking pre-run VN is active.
- Pre-run VN state is in-memory core state for this goal. It is not run checkpoint state because no `RunState` exists yet.

## Command Result Policy

Before this goal, `BehaviorResult::StartNewGame` returned starter candidates
immediately. That no longer matches the desired state flow.

Preferred long-term shape:

```text
BehaviorResult::RunStartFlowStarted {
  game_mode,
  visual_novel,
}
```

or an equivalently renamed result. Do not keep returning starter candidates from
`StartNewGame` while the actual state is `PreRunBriefing`.

## HQ Briefing VN Id Policy

Use mode-specific VN ids:

```text
Standard -> hq_briefing_standard_intro
Endless -> hq_briefing_endless_intro
```

The two ids may initially point to similar authored content, but separate ids
make future mode-specific story changes data-driven.

## Completion Hook Policy

VN progression should report an outcome to `GameCore`; it should not directly
own world-state transitions. `GameCore` handles source-specific completion:

```text
source == HqBriefing and active session ends
-> prepare starter candidates
-> transition to SelectingStarterEmployees
```

Future hooks for battle, event node, tutorial, map, or loadout VN can follow the
same pattern.

## Trigger Policy Boundary

This pre-run core goal should not implement the full VN trigger engine. Its job
is to make VN runtime route-independent and capable of running before `RunState`
exists.

For this goal, `StartNewGame(game_mode)` may directly start the pre-run HQ
briefing VN as the first concrete run-start flow. That direct start is acceptable
only because it is the product entry flow being implemented here.

The next goal should generalize "when does a VN start?" into data-driven trigger
rules, for example:

```text
RunStart { game_mode } -> hq_briefing_standard_intro
RunStart { game_mode } -> hq_briefing_endless_intro
EventSceneEntered { event_id, scene_id } -> event-related VN
BattleStarted { encounter_id } -> battle intro VN
BattleTimeReached { battle_time_ms } -> timed battle VN
UnitHpBelow { unit, threshold } -> crisis VN
MapNodeEntered { node_id, kind } -> map/node VN
LoadoutOpened -> tutorial/loadout VN
```

Do not add ad hoc trigger shortcuts such as `vn_on_enter` fields to event scenes
as the long-term bridge. The long-term direction is a dedicated
`VisualNovelTrigger` catalog with explicit source, conditions, repeat policy,
priority, and queue/consume behavior.

Trigger engine policy questions are intentionally deferred:

- repeat policies: once per run, once per profile, always, once per node/session;
- consumed timing: matched, enqueued, activated, or completed;
- priority and deterministic tie-breakers;
- whether lower-priority triggers queue or wait for the next evaluation pass;
- whether high-priority triggers can interrupt an active VN;
- whether blocking and non-blocking VN sessions share one queue;
- battle trigger timing relative to the current server tick and `battle_update`.

Recommended next goal after this one:

```text
docs/goals/visual_novel_trigger_engine/
```

That goal should convert hard-coded VN starts, including pre-run HQ briefing, to
validated trigger data once the top-level VN runtime exists.

## Open Policy Questions For Implementation Only If Blocking

- Must an already-shipped Unity contract preserve the old `StartNewGame { candidates, required_count }` result?
- Should checkpoint load rewind active in-run VN sessions, or should all VN runtime remain top-level non-checkpointed state?
- Should a missing HQ briefing VN id fail `StartNewGame` as invalid live data? Current recommendation: fail rather than silently skip.

## No Longer Open

The following questions from the first draft now have recommended/default policy:

- Which live VN id starts after `StartNewGame(Standard)`?
  - `hq_briefing_standard_intro`.
- Which live VN id starts after `StartNewGame(Endless)`?
  - `hq_briefing_endless_intro`.
- Should Standard and Endless share an HQ briefing script at first?
  - They may share authored content, but keep separate ids.
- Should starter employee candidates be prepared at `StartNewGame`, or only after HQ briefing completes?
  - Prepare/expose them when transitioning to `SelectingStarterEmployees`.
- Should pre-run VN state survive reconnect before the run is created?
  - Same server-session reconnect can observe in-memory state through normal snapshot.
- Should global VN sessions be saved anywhere before a run checkpoint exists?
  - No, not in this goal.

## Suggested Tests

- `StartNewGame(Standard)` transitions to `PreRunBriefing`.
- `StartNewGame(Standard)` snapshot has non-null root `visual_novel` with `source.type == "hq_briefing"`.
- `StartNewGame(Standard)` snapshot has no run/map before starter selection.
- `StartNewGame(Endless)` preserves pending `GameMode::Endless` through the briefing.
- Blocking pre-run VN exposes only `AdvanceVisualNovel` / `SelectVisualNovelChoice` as allowed actions.
- `advance_visual_novel` works without `RunState`.
- `select_visual_novel_choice` works without `RunState`.
- Ending HQ briefing via `advance_visual_novel` transitions to `SelectingStarterEmployees`.
- Starter candidates appear only after `SelectingStarterEmployees`.
- After selecting starters, `RunState` is created with the original pending game mode.
- Inactive top-level VN serializes root `visual_novel: null`.
- Battle update VN projection uses the same top-level runtime.
- Run checkpoint load no longer restores or depends on `RunState.visual_novel` unless a later policy explicitly requires checkpoint-rewound VN sessions.

## Notes

This is not a request to move Unity-local presentation cue data into core.
`script_id` remains a Unity-local VN presentation script resource id. Core should
own session flow, scene ids, choices, and branching, while Unity owns line/cue
presentation such as background changes, portrait/expression changes, audio hooks,
fade, shake, and camera effects.
