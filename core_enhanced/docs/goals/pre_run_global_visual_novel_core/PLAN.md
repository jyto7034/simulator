# Pre-Run Global Visual Novel Core

## Objective

Refactor core Visual Novel (VN) runtime ownership so VN is a top-level game
runtime system, not a `RunState`-owned subsystem, and support this product flow:

```text
Main Menu
-> StartNewGame(game_mode)
-> PreRunBriefing with root state.visual_novel active
-> advance_visual_novel / select_visual_novel_choice
-> SelectingStarterEmployees
-> SelectStarterEmployees
-> RunState creation
-> ViewingMap
```

The long-term goal is a route-independent VN overlay. HQ briefing, map dialogue,
loadout dialogue, battle dialogue, event-node-triggered VN, tutorial, radio, and
cutscene content should share the same canonical VN runtime. Their origin is
represented by `VisualNovelSource`, not by separate runtime owners.

## Source Of Truth Order

1. Runtime code and focused tests.
2. Live RON/data.
3. Unity-facing snapshot/command/server-message contracts.
4. Live server/probe behavior.
5. Current policy documents.

Do not rely on older goal documents alone. The existing
`global_visual_novel_engine` goal is useful background, but this goal supersedes
any wording that treats active/queued VN sessions as `RunState`-owned runtime
state.

## Confirmed Product Policy

- `StartNewGame(game_mode)` starts a run-start flow. It does not immediately
  mean the run object has been created.
- `pre-run` means before `RunState` creation, not before `StartNewGame`.
- HQ briefing VN starts after `StartNewGame(game_mode)` succeeds and before
  starter employee selection.
- `RunState` is created only after starter employees are selected.
- Unity opens VN from the public `visual_novel` projection, not from
  `selected_event` or route-specific UI state.
- Event nodes and VN overlays remain separate concepts. Event nodes are
  choice/result node interactions; VN is global presentation/story overlay.
- Core owns VN session id, source, VN id, current scene id, choice availability,
  blocking policy, and progression.
- Unity owns local script/cue presentation, text reveal, background changes,
  expression/portrait changes, audio hooks, fade/shake, and camera effects.
- `script_id` remains a Unity-local VN presentation script resource id. Core
  must not send localized dialogue body text, image data, or Unity cue payloads.

## Canonical Runtime Policy

Use a single canonical top-level VN runtime:

```text
GameCoreState
-> visual_novel: VisualNovelRuntimeState
-> run: Option<RunState>
```

`RunState.visual_novel` should be removed or made non-canonical during this goal.
Do not implement a long-term `global_visual_novel` plus `run.visual_novel`
priority fallback. That dual-owner model would preserve the current dependency
and make future route-independent VN behavior harder to reason about.

The public projection should be:

```text
state.visual_novel = GameCoreState.visual_novel.active else null
```

Battle transport should attach the same projection to `battle_update.visual_novel`
when battle transport is the active delivery channel.

## Target State Flow

Add an explicit game state:

```text
GameState::PreRunBriefing
```

Recommended state flow:

```text
NotStarted
-> StartNewGame(game_mode)
-> initialize player/session-level state
-> store pending_game_mode
-> clear old run/node/battle state
-> activate GameCoreState.visual_novel with source HqBriefing
-> PreRunBriefing
-> VN commands progress the HQ briefing
-> HQ briefing VN ends
-> prepare starter employee candidates
-> SelectingStarterEmployees
-> SelectStarterEmployees
-> create RunState from pending_game_mode
-> ViewingMap
```

`PreRunBriefing` should have a Unity-facing `GameStateContextDto` variant. The
first shape should include the selected `game_mode` so Unity can distinguish the
mode-specific run-start flow while there is no map/run yet:

```json
{
  "type": "pre_run_briefing",
  "game_mode": "Standard"
}
```

## Command Result Policy

Before this goal, `BehaviorResult::StartNewGame` returned starter candidates
immediately. That shape no longer matches the desired flow.

Preferred long-term contract:

```text
BehaviorResult::RunStartFlowStarted {
  game_mode,
  visual_novel,
}
```

or an equivalently renamed command result. Do not keep returning starter
candidates from `StartNewGame` while the state is `PreRunBriefing`; that would
blur the state contract. Starter candidates should be exposed when the core
actually transitions to `SelectingStarterEmployees`.

## HQ Briefing VN Id Policy

Use mode-specific VN ids, even if the initial authored content is shared:

```text
Standard -> hq_briefing_standard_intro
Endless -> hq_briefing_endless_intro
```

This keeps future mode-specific story changes data-driven and avoids another
contract migration.

## Live Data Readiness Policy

The current live VN catalog must contain the HQ briefing ids before
`StartNewGame(game_mode)` can activate the pre-run briefing in production code.
Do not silently skip the briefing when an HQ VN id is missing. Treat missing HQ
VN data as invalid live data so QA catches the broken authoring path.

Implementation may add the live HQ VN definitions immediately before or during
Phase 3. Phase 4 should then verify the final live catalog and transport shape
rather than discovering the missing ids late.

## Persistence Policy For This Goal

- Pre-run VN state is in memory as part of `GameCoreState`.
- Pre-run VN is not stored in run checkpoints because no `RunState` exists yet.
- Same server-session reconnect may observe the current in-memory pre-run VN
  state through the normal snapshot path.
- Durable save-file migration and process-restart recovery for pre-run VN are
  out of scope unless existing persistent save code requires an immediate
  decision.
- Existing tests currently expect run checkpoint load to restore active
  `RunState.visual_novel`. Under the new top-level ownership policy, replace
  those with tests that prove checkpoints do not hide a second VN owner unless a
  future explicit policy chooses checkpoint-rewound VN sessions.

## Implementation Plan

### Phase 0: Re-Read Current Runtime Boundary

Scope:

- Re-read `GameCoreState`, `RunState`, `RunCheckpointPayload`, `StartNewGame`,
  `SelectStarterEmployees`, VN flow handlers, snapshot projection, action
  scheduler context, battle VN attachment, and server command-result mapping.
- Re-read live `visual_novels/base.ron`.
- Re-read focused tests that currently mutate `run.visual_novel`.

Completion conditions:

- Current `RunState.visual_novel` dependencies are listed in `EXPERIMENTS.md`.
- Any policy issue that requires user input is recorded before code changes.

### Phase 1: Move VN Runtime Ownership

Scope:

- Add `GameCoreState.visual_novel: VisualNovelRuntimeState`.
- Initialize it in `GameCoreState::new`.
- Move snapshot projection to `GameCoreState.visual_novel`.
- Move allowed-action VN context to `GameCoreState.visual_novel`.
- Move battle tick pause checks and battle update attachment to the top-level
  projection.
- Remove or stop using `RunState.visual_novel`.
- Update run checkpoint payload/load paths so they no longer treat VN as
  run-local checkpoint state unless a required runtime invariant is found.

Completion conditions:

- Root snapshot `visual_novel` is driven by top-level VN runtime.
- Battle update `visual_novel` is driven by the same projection.
- VN action gating works before `RunState` exists.
- Focused snapshot, VN command, action scheduler, battle VN tests pass.

### Phase 2: Generalize VN Progression Runtime

Scope:

- Refactor VN advance/select logic to operate on `&mut VisualNovelRuntimeState`
  instead of `&mut RunState`.
- Return a small progression outcome that lets `GameCore` handle source-specific
  completion hooks.
- The progression outcome must include the completed session metadata
  (`session_id`, `source`, `vn_id`, and final scene/choice context as needed)
  before any queued session is promoted. Do not infer completion hooks from the
  post-advance active session alone; that becomes ambiguous once queueing is
  used.
- Keep VN graph validation and scene lookup data-driven through
  `VisualNovelDatabase`.
- Preserve command validation: session id, current scene id, scene shape, and
  choice id must match active session data.

Completion conditions:

- `advance_visual_novel` and `select_visual_novel_choice` work without
  `RunState`.
- Existing map/battle/event-source VN tests are updated to activate top-level
  VN state.
- No fallback path reads `run.visual_novel`.

### Phase 3: Add PreRunBriefing Flow

Scope:

- Add `GameState::PreRunBriefing`.
- Add `GameStateContextDto::PreRunBriefing { game_mode }`.
- Change `StartNewGame(game_mode)` to start the run-start flow:
  - initialize player;
  - clear previous run/node/battle state;
  - store `pending_game_mode`;
  - activate HQ briefing VN;
  - transition to `PreRunBriefing`.
- Require the mode-specific HQ briefing VN id to exist before activation; a
  missing id is invalid live data, not a signal to skip the briefing.
- Change `StartNewGame` command result contract to the new run-start-flow
  meaning.
- On HQ briefing VN completion, prepare starter candidates and transition to
  `SelectingStarterEmployees`.
- Ensure `SelectStarterEmployees` still creates `RunState` from the original
  `pending_game_mode`.

Completion conditions:

- `StartNewGame(Standard)` transitions to `PreRunBriefing`.
- `StartNewGame(Endless)` preserves `GameMode::Endless` through briefing and
  run creation.
- Snapshot during `PreRunBriefing` has root `visual_novel.source.type ==
  "hq_briefing"`.
- Snapshot during `PreRunBriefing` has no run/map.
- VN completion transitions to `SelectingStarterEmployees` with candidates.
- `SelectStarterEmployees` creates `RunState` and transitions to `ViewingMap`.

### Phase 4: Live Data, Contract, And Probe Verification

Scope:

- Verify live VN definitions for `hq_briefing_standard_intro` and
  `hq_briefing_endless_intro`.
- Update Unity-facing transport/core docs after code shape is confirmed.
- Run focused tests after each small change and broader tests at the end.
- If feasible, run the game server and a smoke/probe path that verifies
  `StartNewGame -> PreRunBriefing -> visual_novel`.

Completion conditions:

- Live RON loads successfully. Status: core live-data load path is covered by
  focused and broad lib tests; `../game_server cargo run` also loaded embedded
  live game data before starting the HTTP server.
- Unity-facing JSON shape is covered by contract tests. Status: core tests now
  cover root `visual_novel`, HQ source, no run/map before starter selection,
  null inactive projection, and battle update projection.
- Server command-result mapping reflects the new `StartNewGame` semantics.
  Status: core payload contract returns `RunStartFlowStarted`; server actor
  tests expect the generic payload mapping and `pre_run_briefing` snapshot.
- Final verification commands are recorded in `EXPERIMENTS.md`.

## Required Tests

- `StartNewGame(Standard)` returns the new run-start-flow result and enters
  `PreRunBriefing`.
- `StartNewGame(Standard)` snapshot contains `visual_novel` with
  `source.type == "hq_briefing"`.
- `StartNewGame` snapshot has no run/map before starter selection.
- `StartNewGame(Endless)` preserves mode through briefing and run creation.
- Blocking pre-run VN exposes only VN progression commands.
- `advance_visual_novel` works without `RunState`.
- `select_visual_novel_choice` works without `RunState` for choice scenes.
- Missing mode-specific HQ briefing VN id fails as invalid live data instead of
  silently skipping briefing.
- HQ briefing VN completion transitions to `SelectingStarterEmployees`.
- Starter candidates appear only when `SelectingStarterEmployees` is active.
- `SelectStarterEmployees` creates `RunState` from `pending_game_mode`.
- Inactive top-level VN serializes root `visual_novel: null`.
- Battle update VN projection uses the same top-level runtime.
- Run checkpoint load no longer restores or depends on `RunState.visual_novel`
  unless a documented policy change is approved.

## Out Of Scope

- Unity overlay implementation.
- Full VN trigger engine repeat/priority/consume policy.
- Persistent save migration for pre-run VN.
- New Unity-local script/cue language design.
- Rewriting Event-node reward/choice flow unless a direct dependency blocks this
  goal.
- Adding compatibility aliases for old VN runtime ownership.

## Stop And Ask

Stop before implementation if any of these are discovered:

- The public command result must preserve old `StartNewGame` candidate payload
  for a live Unity compatibility reason.
- Live data cannot safely add mode-specific HQ briefing VN ids.
- Save/persistence code requires pre-run VN durable recovery now.
- Removing `RunState.visual_novel` would break a policy that intentionally wants
  checkpoint-rewound VN sessions.
- A new Unity-facing DTO shape beyond `PreRunBriefing` context or run-start
  command result is required.
