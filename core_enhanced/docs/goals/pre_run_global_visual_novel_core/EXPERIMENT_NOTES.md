# Pre-Run Global Visual Novel Core Experiment Notes

## Working Judgment

The best long-term architecture is a single top-level VN runtime:

```text
GameCoreState.visual_novel
```

not a dual-owner model:

```text
global_visual_novel else run.visual_novel
```

The dual-owner model would solve the immediate pre-run case but keep future
route-independent VN behavior split across owners. Since VN can originate from
HQ, map, loadout, battle, event node, tutorial, radio, and cutscene contexts, the
runtime should be global and the origin should live in `VisualNovelSource`.

## Why This Is Not A Compatibility Layer

This goal intentionally changes ownership instead of adding a fallback. The
current `RunState.visual_novel` design is already too narrow for the desired
flow because `RunState` does not exist after `StartNewGame` and before starter
selection. A fallback such as "if no run, check global VN; otherwise check
run-local VN" would preserve two sources of truth and force Unity/server tests to
reason about priority rather than a single public projection.

## StartNewGame Meaning

`StartNewGame(game_mode)` should mean "begin the run-start flow", not "create a
run immediately". This matches the desired product flow:

```text
mode selected
-> briefing/story
-> starter selection
-> actual run creation
```

The actual `RunState` should be created only after starter employees are
selected.

## PreRunBriefing State

Use an explicit `GameState::PreRunBriefing`. Hiding the state inside
`NotStarted` or `SelectingStarterEmployees` would make allowed actions, snapshot
context, command routing, and Unity UI interpretation harder to verify.

The snapshot context should carry at least `game_mode`, because at this point
there is no run/map but there is already a selected mode.

## Candidate Preparation Timing

Recommended policy: prepare starter candidates after HQ briefing VN completion,
when transitioning to `SelectingStarterEmployees`.

Reasoning:

- It keeps `StartNewGame` focused on starting the run-start flow.
- It avoids exposing starter candidates while a blocking pre-run VN is active.
- It makes the `SelectingStarterEmployees` context the source of truth for
  starter selection UI.

If implementation shows candidate validation must happen earlier to fail fast on
broken live data, it can still validate data availability at `StartNewGame`
without exposing candidates in the command result.

## HQ Briefing VN Ids

Recommended first ids:

```text
hq_briefing_standard_intro
hq_briefing_endless_intro
```

Even if both ids initially point to similar authored content, separate ids keep
future mode-specific story branching data-driven.

Current code review found that the live VN catalog only contains
`white_night_confession_intro`. The HQ briefing ids must be added before the
`StartNewGame -> PreRunBriefing` production path depends on them. Missing HQ VN
data should fail as invalid live data; core should not silently skip the
briefing or fall back to another VN id.

## VN Completion Hooks

VN runtime progression should report outcomes to `GameCore`; it should not own
game-state transitions directly.

Example:

```text
VN runtime finishes active session with source HqBriefing
-> GameCore prepares starter candidates
-> GameCore transitions to SelectingStarterEmployees
```

Future source-specific hooks can be added for battle, event-node, tutorial, or
map VN completion without teaching the pure VN progression helper about world
state internals.

The progression outcome should carry the completed session metadata before any
queued session is promoted. Relying on `visual_novel.active == None` or the
post-promotion active session would be too weak once queued VN sessions exist.
For HQ briefing, `GameCore` needs to know that the completed session's source was
`HqBriefing` even if another VN becomes active immediately after it.

## Checkpoint And Persistence Notes

Pre-run VN cannot be part of run checkpoint state because no `RunState` exists.
For this goal, in-memory server-session state is enough unless code review finds
an existing persistent save requirement.

Run checkpoint behavior for in-run VN should be reconsidered when moving
`RunState.visual_novel` out. The default recommendation is:

- Top-level active VN is global runtime state.
- Run checkpoint should not implicitly own VN runtime.
- If a future policy wants checkpoint-rewound VN sessions, model that as an
  explicit narrative/checkpoint policy rather than hidden `RunState` ownership.

Current focused tests include an expectation that checkpoint load restores
`RunState.visual_novel`. That test should be replaced under the new policy rather
than preserved as a compatibility requirement, unless implementation uncovers a
real product need for checkpoint-rewound VN sessions.

## Test Fixture Data Boundary

Implementation confirmed that any fixture which reconstructs `GameDataBase` from
`GameDataBuilder::empty()` must deliberately include `visual_novel_data` when it
expects `StartNewGame` to work. This is not a compatibility fallback; it reflects
the production invariant that mode-specific HQ briefing VN ids are required live
data.

Fixtures that only test unrelated not-started serialization may still use an
empty VN catalog as long as they do not execute `StartNewGame`.

Server-side live-battle fixtures should preserve embedded live data unless the
test is intentionally validating missing-data behavior. Rebuilding a narrow
fixture with only a defense encounter is brittle because starter selection and
map generation can legitimately touch normal combat, reward, equipment,
corroded wave, map, and VN catalogs before the test reaches the live battle.

For server survival-timer tests, prefer applying a targeted timer override to
live defense encounters over adding an isolated custom encounter that may not be
selected by deterministic map/encounter selection.

## Out-Of-Scope Follow-Up Candidates

- Full VN trigger engine with repeat, priority, consumption timing, and
  persistence rules.
- A VN debug/admin command to start a session over WebSocket for live non-null
  battle VN probing.
- Durable save/reconnect policy for pre-run VN after server process restart.
- Unity-local presentation script/cue authoring schema.
- More precise allowed actions that expose only `AdvanceVisualNovel` or only
  `SelectVisualNovelChoice` depending on current scene shape.

## Policy Questions To Ask Only If They Block Implementation

- Must the old `StartNewGame { candidates, required_count }` command result
  remain for an already-shipped Unity contract?
- Should `RunState` checkpoint load rewind active VN sessions, or should VN
  remain top-level non-checkpointed runtime?
- Should a failed or missing HQ briefing VN id block `StartNewGame`, or should
  core skip briefing? Current recommendation is to fail as invalid live data,
  not skip silently.
- Should VN checkpoint rewind exist as an explicit product feature? Current
  recommendation is no for this goal; keep VN top-level and non-checkpoint-owned.
