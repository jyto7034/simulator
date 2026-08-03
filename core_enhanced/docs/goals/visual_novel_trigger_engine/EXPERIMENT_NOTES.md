# Visual Novel Trigger Engine Experiment Notes

## Working Judgment

The trigger engine should be an internal core system that starts or queues
`VisualNovelSession`s. It should not be a Unity-facing contract expansion.

The existing public projection is already the right boundary:

```text
state.visual_novel
battle_update.visual_novel
```

Unity should not know why a VN started. Unity should only render the active VN
projection and send VN progression commands.

## Why Trigger Data Is Needed

Hard-coded VN activation works for one HQ briefing but does not scale to:

- boss intros;
- event scene overlays;
- map/node warnings;
- timed battle radio;
- HP crisis dialogue;
- tutorial prompts;
- objective discovery/destruction;
- abnormality research milestones.

Without a trigger catalog, those conditions would become scattered Rust
conditionals. That would make story content hard to author, hard to audit, and
hard to manage through a future content tool.

## DTO Impact

The trigger engine should not significantly change Unity-facing DTOs.

Keep:

- `VisualNovelSnapshotDto`;
- `VisualNovelSourceDto`;
- `VisualNovelSpeakerDto`;
- `VisualNovelChoiceSnapshotDto`;
- root `RunSnapshotDto.visual_novel`;
- `LiveBattleUpdateDto.visual_novel`.

Do not expose:

- trigger id;
- repeat policy;
- priority;
- checkpoint policy;
- consumed history;
- raw conditions.

If future content needs richer source metadata, extend `VisualNovelSourceDto`
narrowly after reviewing Unity impact.

## Checkpoint Policy Rationale

Long-term policy should be explicit per trigger:

```text
PersistAcrossCheckpointLoad
RewindWithRunCheckpoint
```

This is better than a single global behavior because different VN content has
different meaning:

- tutorial, warning, boss intro, and atmosphere VN usually should not repeat
  after checkpoint load;
- story consequence, choice result, and gameplay-coupled VN may need to rewind
  with checkpoint state.

The implementation should therefore split trigger consumed history by policy.
Do not collapse it into one global set as a narrow shortcut.

## Consumed Timing Rationale

Consumed timing should be activation/queue success.

Rejected alternatives:

- Matched: too early; a trigger could be consumed without any VN being shown or
  queued.
- Completed: too late; the same trigger can match again while the VN is active
  or queued.

Activation/queue success gives stable duplicate prevention while preserving the
meaning that the VN is now scheduled for presentation.

## Priority And Queue Rationale

Higher numeric priority first is easiest for authoring tools and designers.
Trigger id ascending as tie-breaker is deterministic and does not depend on RON
author order.

First implementation should not interrupt active VN sessions. Queueing is enough
to validate repeat/priority behavior without creating partial VN cancellation
semantics.

## Battle Timing Rationale

Battle trigger evaluation should use produced battle update data, then attach
VN before sending.

This avoids retroactive simulation changes while still letting Unity receive the
new overlay in the update caused by the trigger.

Blocking VN should pause the next tick, not undo the current tick.

Timed battle triggers should use threshold crossing:

```text
previous_time < threshold <= current_time
```

This prevents repeated firing from pause, speed changes, and repeated state
requests.

## Suggested First Live Triggers

Use live trigger data to replace hard-coded HQ briefing first:

```ron
VisualNovelTrigger(
    id: "run_start_standard_hq_briefing",
    source: RunStart(game_mode: Standard),
    vn_id: "hq_briefing_standard_intro",
    repeat: OncePerRun,
    checkpoint_policy: PersistAcrossCheckpointLoad,
    priority: 100,
    blocking: true,
)
```

```ron
VisualNovelTrigger(
    id: "run_start_endless_hq_briefing",
    source: RunStart(game_mode: Endless),
    vn_id: "hq_briefing_endless_intro",
    repeat: OncePerRun,
    checkpoint_policy: PersistAcrossCheckpointLoad,
    priority: 100,
    blocking: true,
)
```

Then add one EventSceneEntered fixture trigger and one BattleStarted or
BattleTimeReached fixture trigger to prove the engine is route-independent.

Implemented on 2026-07-09:

- Live RON contains `run_start_standard_hq_briefing` and
  `run_start_endless_hq_briefing`.
- Focused tests add fixture triggers for `EventSceneEntered`,
  `MapNodeEntered`, `BattleStarted`, and `BattleTimeReached`.
- No live event/map/battle story trigger was added beyond RunStart HQ briefing;
  broader story content remains an authoring task.

## Implementation Notes After Completion

- `VisualNovelTriggerDatabase` is part of `GameDataBase`; any test/helper that
  rebuilds `GameDataBase` from a base catalog must preserve
  `visual_novel_trigger_data` unless the test intentionally verifies missing
  trigger data.
- `StartNewGame` now treats "no matching RunStart trigger opened a VN" as
  invalid static data. This is stricter than merely checking whether a hard-coded
  VN id exists and matches the long-term data-driven policy.
- Trigger runtime state is separate from active/queued VN session state.
  Checkpoint load restores only checkpoint-scoped trigger history; it does not
  restore active VN sessions.
- Battle-time triggers are attached to the same produced `battle_update` after
  simulation/update construction. Blocking effects begin on the following tick.
- Existing `state.visual_novel` / `battle_update.visual_novel` DTO shape remains
  the only Unity-facing contract for VN overlay visibility.

## Out-Of-Scope Follow-Up Candidates

- `OncePerProfile` and durable account/profile storage.
- Multiple concurrent VN overlays.
- Interrupting active VN sessions.
- Non-blocking radio VN coexisting with blocking cinematic VN.
- HP/objective/boss phase triggers.
- Authoring-tool schema UX.
- Admin/debug command to force trigger evaluation or start VN by id.
- Live-authored non-HQ story triggers for actual event/map/battle content.
- Server/WebSocket smoke probe for a non-null battle VN once a gameplay/admin
  path can reliably trigger one in live server flow.

## Policy Questions To Reopen Only If Implementation Requires

- Should any first live trigger use `RewindWithRunCheckpoint`, or are the first
  live triggers all presentation-first?
- Does `OncePerBattle` identify battle by battle UUID, encounter id, or both?
  Current recommendation: battle UUID.
- Does `OncePerNode` identify node by map node UUID only, or event session id if
  event sessions later become independently addressable?
  Current recommendation: map node UUID.
- Should missing RunStart trigger data fail `StartNewGame`, or should only
  missing referenced VN fail?
  Current recommendation: fail `StartNewGame` if no required RunStart trigger
  opens a VN.
