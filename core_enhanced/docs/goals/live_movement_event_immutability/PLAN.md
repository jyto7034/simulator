# Live Movement Event Immutability Plan

Created: 2026-06-17

Status: Implemented and verified on 2026-06-17.

## Objective

Make Unity-facing live movement events immutable and precise enough for smooth presentation.

The core must stop depending on a movement timeline event being extended after it may already have been sent to Unity. Live `battle_update.events_delta` should expose movement as immutable 50ms simulation segments. Unity can then sample those segments by core battle time, while stored/archive logs may later coalesce movement for size and readability.

This is a long-term transport contract repair, not a one-off patch for the current GIF stutter.

## Source Of Truth Order

Use this order while implementing:

1. Runtime code.
2. Live wire logs and live RON/data.
3. Unity-facing battle update/setup DTO contract.
4. Current policy docs.

Do not trust docs over code. If runtime evidence shows a better long-term transport model, record it in `EXPERIMENT_NOTES.md`. Stop and report policy questions if the better model changes Unity-facing DTO shape, live RON schema, saved log format, or battle playback semantics.

## Code Evidence Already Read

- `src/game/battle/core/movement/types.rs`
  - `DEFAULT_MOVEMENT_TICK_MS` is `50`.
- `src/game/battle/core/sim.rs`
  - `ContinuousMovementTick` runs movement using `DEFAULT_MOVEMENT_TICK_MS`.
  - The next continuous movement tick is scheduled at `time_ms + 50`.
- `src/game/battle/core/movement/engine.rs`
  - Before this goal, `record_or_extend_continuous_movement_segment` mutated an existing `MovementSegmentStarted` timeline entry when:
    - the same unit has an active segment,
    - quantized velocity is equal,
    - and the active segment `ends_at_ms == time_ms`.
  - This is valid for a closed, replay-only timeline but unsafe for live delta transport if the original seq has already been sent.
  - This goal replaced it with immutable per-movement-tick event recording.
- `src/game/battle/timeline.rs`
  - `MovementSegmentStarted` has `start`, `target`, `started_at_ms`, `ends_at_ms`, and `end_kind`.
  - Once this event appears in `battle_update.events_delta.events`, Unity treats it as presentation source of truth.
- External `unity_core_contract.md` / `core_unity_battle_update_contract.md`
  - Unity must use event time, not receive time, for movement presentation.
  - `checkpoint.units[*].world_position` is a reconcile target, not immediate transform input.
  - Event stream seq is monotonic and gap-sensitive.

## Live Log Evidence

Recent wire logs showed movement events such as:

```text
seq 6: 0ms   -> 50ms
seq 7: 450ms -> 550ms
seq 8: 850ms -> 950ms
```

The later segment starts from a position that implies the core simulation continued moving during the gap. This suggests the internal timeline entry may have been extended after an earlier version had already been flushed, leaving Unity with only the old short event.

## Problem Statement

The current movement event extension logic makes a timeline event mutable:

```text
0ms movement tick
  record seq 6: 0..50

50ms movement tick with same velocity
  mutate seq 6: 0..100

100ms movement tick with same velocity
  mutate seq 6: 0..150
```

In live delta transport, Unity may already have received `seq 6: 0..50`. If core later mutates `seq 6` to `0..150`, the mutated payload is not necessarily resent because Unity requests new events by seq cursor. This can create presentation gaps even though core simulation moved continuously.

## Policy To Lock

### Live Movement Stream

- Any event that can be sent in `battle_update.events_delta` is immutable once seq is assigned.
- `MovementSegmentStarted` is not an exception.
- The live movement stream prioritizes correctness and presentation continuity over log compactness.
- Core movement simulation tick remains `50ms` unless a separate movement physics goal changes it.
- If a unit actually moves during a movement tick, emit an immutable movement segment for that tick:

```text
MovementSegmentStarted {
  started_at_ms: tick_time_ms,
  ends_at_ms: tick_time_ms + 50,
  start: position_before_tick,
  target: position_after_tick
}
```

- If the unit does not move, do not emit a movement segment for that unit.
- If movement stops because of block, target reached, movement lock, hard CC, death, no goal, or similar, emit/keep the appropriate stop/state event according to existing movement semantics.
- Unity stitches adjacent 50ms segments using presentation battle time. Unity does not use checkpoint position as a hidden movement source.

### Stored/Archive Battle Logs

- Do not implement stored log coalescing in this goal unless existing tests prove storage volume is already blocking.
- Stored/archive/replay logs may later coalesce movement events, but only after live stream correctness is fixed.
- Future coalescing must preserve presentation meaning and must be reversible enough for replay/debug.
- Conservative future coalescing candidates:
  - Same unit.
  - Adjacent times.
  - Previous `ends_at_ms == next.started_at_ms`.
  - Previous `target == next.start`.
  - No intervening `MovementStopped`, `UnitDied`, teleport, hard CC, forced position correction, route reset, or other movement-meaning event.
  - Either same quantized velocity, or a future `MovementTrack` sample-list format that preserves all positions.

## Scope

In scope:

- Remove or replace live timeline mutation of previously recorded `MovementSegmentStarted`.
- Ensure live movement events are immutable after recording.
- Preserve 50ms movement presentation accuracy for actual movement ticks.
- Add tests that prove an already recorded movement event is not mutated by later movement ticks.
- Add live battle/DTO-oriented tests that show adjacent movement ticks produce Unity-facing events with contiguous time coverage.
- Update Unity-facing docs if they still imply mutable/long live movement events.
- Record storage coalescing as a follow-up, not current implementation.

Out of scope:

- Implementing Unity segment queue/presentation clock.
- Changing `MovementSegmentStarted` DTO shape.
- Changing battle_update message order or seq rules.
- Changing movement simulation tick length.
- Adding stored battle log compression now.
- Tuning enemy speed, radius, route shape, targeting, block balance, or attack cadence unless a bug makes movement tests impossible.
- Reintroducing compatibility layers or dual movement schemas.

## Candidate Implementation Plan

1. Re-read movement code before editing.
   - Confirm all callers of `record_or_extend_continuous_movement_segment`.
   - Confirm whether replay/export paths rely on the current extension behavior.
   - Confirm `active_movement_segments` is used for any purpose other than extension and sampling.

2. Make live movement events immutable.
   - Preferred first implementation: replace `record_or_extend_continuous_movement_segment` with `record_continuous_movement_segment`.
   - Always record a new `MovementSegmentStarted` for each `BodyMoved` output with non-zero movement.
   - Keep active movement state only if another runtime feature still needs it.
   - Do not mutate an event already in `timeline.entries`.

3. Preserve stop semantics.
   - Keep `stop_continuous_movement_segment` behavior for `TargetReached` and `MovementStopped` if active runtime state remains.
   - Verify hard CC/death/movement lock routes still produce expected stop/state events.

4. Add focused movement timeline tests.
   - A unit moving with constant velocity across two 50ms ticks should produce two immutable movement events:
     - `0..50`
     - `50..100`
   - The first event must remain unchanged after the second tick.
   - Consecutive movement events for the same unit must have `previous.ends_at_ms == next.started_at_ms` when movement is continuous.

5. Add live DTO behavior tests.
   - Start a live DefenseRoute battle and advance enough ticks.
   - Assert the Unity-facing movement events are immutable, ordered by seq, and contain contiguous 50ms coverage for a continuously moving enemy.
   - Ensure checkpoint `world_position` still follows simulation state but is not required to infer missing movement events.

6. Regression tests for no movement.
   - A movement-locked/dead/no-goal unit must not spam `MovementSegmentStarted`.
   - If existing behavior emits `MovementStopped`, keep/pin it.

7. Docs.
   - Update external `core_unity_battle_update_contract.md` and `unity_core_contract.md` if needed:
     - live movement events are immutable;
     - current live policy emits 50ms movement segments for actual movement;
     - archive coalescing is allowed only outside live stream and is not part of the current implementation.
   - Update local docs only if the gameplay rulebook needs the transport policy summary.

8. Verification.
   - Run focused movement tests after each change.
   - Run live DefenseRoute tests.
   - Run `cargo check -p game_core`.
   - Run wider `cargo test -p game_core` before completion.

## Completion Conditions

- Unity-facing `MovementSegmentStarted` events are immutable after seq assignment. Done.
- Continuous movement no longer depends on mutating an earlier movement event after it may have been sent. Done.
- A continuously moving unit produces movement segments that cover adjacent 50ms intervals. Done.
- No-movement/locked/dead units do not emit bogus movement segments. Done for no-goal/no-body-motion; existing lock/death tests remain green.
- Existing movement, targeting, battle update, and live RON tests pass or are updated to the new policy. Done.
- Unity-facing contract docs describe live 50ms immutable movement events and defer archive coalescing. Done.
- `EXPERIMENTS.md` records implementation attempts, failures, fixes, and verification commands. Done.
- `EXPERIMENT_NOTES.md` records policy decisions and follow-up candidates. Done.

## Implementation Summary

- Replaced mutable movement timeline extension with immutable `record_continuous_movement_segment`.
- Each actual `BodyMoved` output now records a fresh `MovementSegmentStarted` for that movement tick.
- Removed `timeline_index` and `velocity` from `ActiveMovementSegment`; active movement state now keeps only data needed for runtime sampling.
- Kept `active_movement_segments` for projectile/position sampling and cleanup paths.
- Added focused tests:
  - `battle_core_records_immutable_movement_segments_per_tick`
  - `battle_core_does_not_record_movement_segment_without_body_motion`
- Added live DTO test:
  - `live_battle_update_exposes_adjacent_immutable_movement_segments`
- Updated external Unity-facing docs:
  - `/mnt/f/unity projects/ark/docs/core_unity_battle_update_contract.md`
  - `/mnt/f/unity projects/ark/docs/unity_core_contract.md`

## Stop Conditions

Stop and report questions instead of deciding unilaterally if any of these are discovered:

- A saved replay/archive format already depends on coalesced movement events.
- Removing movement event mutation changes a public DTO shape.
- Event volume becomes immediately unmanageable in official live gameplay tests.
- Unity requires a different movement segment contract than `MovementSegmentStarted`.
- Current movement code relies on `active_movement_segments.sample_position_at` for gameplay-critical logic in a way that conflicts with immutable events.
- Fixing this requires changing movement speed, enemy body radius, block behavior, route authoring, or combat balance.

## Expected Verification Commands

Focused:

```text
cargo test -p game_core movement:: -- --nocapture
cargo test -p game_core live_defense_ -- --nocapture
cargo test -p game_core generated_defense_route_enemy_progresses_past_spawn_boundary -- --nocapture
```

Contract/check:

```text
cargo check -p game_core
cargo test -p game_core --test ron_loading -- --nocapture
```

Final broad verification:

```text
cargo test -p game_core
```

If server message mapping changes unexpectedly:

```text
cargo check -p game_server
cargo test -p game_server player_game_actor -- --nocapture
```
