# Live Movement Event Immutability Experiments

Created: 2026-06-17

Status: Implemented and verified on 2026-06-17.

This file records code reading, implementation attempts, failures, fixes, and verification results for the live movement event immutability goal.

## 2026-06-17 - Initial Code Reading

Result: issue confirmed as plausible from code and wire-log behavior.

Evidence read:

- `DEFAULT_MOVEMENT_TICK_MS` is `50`.
- `ContinuousMovementTick` runs movement at fixed 50ms simulation steps.
- `record_or_extend_continuous_movement_segment` mutates an existing `MovementSegmentStarted` timeline entry if the next movement output has the same quantized velocity and begins at the previous segment end.
- Live `battle_update.events_delta` is seq-cursor based. A client that already received a seq will not necessarily receive a later mutation of that same seq.

Observed wire-log pattern:

```text
seq 6: 0ms   -> 50ms
seq 7: 450ms -> 550ms
seq 8: 850ms -> 950ms
```

The later segments start from positions that imply the core simulation moved through the missing intervals. That points to live event mutation/coalescing being unsafe, not just Unity receive-time interpolation.

Decision:

- Treat Unity-facing presentation events as immutable after seq assignment.
- Prefer simple 50ms immutable live movement events first.
- Defer stored/archive coalescing until event volume is proven to be a real storage problem.

## Planned Experiments

### Experiment A - Remove Timeline Mutation

Hypothesis:

Recording a new immutable `MovementSegmentStarted` per actual movement tick will make live deltas complete and easier for Unity to sample.

Procedure:

1. Replace extension/mutation logic with new-event recording.
2. Keep stop handling intact.
3. Run focused movement tests.

Expected result:

- Constant movement over two ticks emits two events, not a mutated first event.
- The first event remains unchanged.

Actual result:

- Success.
- `record_or_extend_continuous_movement_segment` was replaced by immutable `record_continuous_movement_segment`.
- Same-velocity adjacent ticks now record separate movement events instead of mutating the first event.
- Focused test `battle_core_records_immutable_movement_segments_per_tick` pins:
  - two events are emitted for `0..50` and `50..100`,
  - the first event remains unchanged after the second tick,
  - the first target equals the second start.

### Experiment B - Live DTO Continuity

Hypothesis:

After immutable movement events, live battle update deltas expose contiguous movement time ranges for continuously moving enemies.

Procedure:

1. Use a DefenseRoute scenario with an enemy that moves without immediate block/target stop.
2. Advance the live battle.
3. Inspect `battle_update.events_delta.events` for movement segment continuity.

Expected result:

- For a continuously moving unit, adjacent segments have:

```text
previous.ends_at_ms == next.started_at_ms
previous.target == next.start
```

Actual result:

- Success.
- Added `live_battle_update_exposes_adjacent_immutable_movement_segments`.
- The test starts a live DefenseRoute battle, advances 50ms server ticks, collects Unity-facing `battle_update.events_delta` movement entries, and verifies adjacent 50ms movement coverage is exposed in the live DTO stream.

### Experiment C - No Movement Does Not Spam Events

Hypothesis:

Removing extension should not cause locked/dead/no-goal units to emit fake movement events.

Procedure:

1. Run existing movement lock/death/no-goal tests.
2. Add focused tests if existing coverage is too implicit.

Expected result:

- No bogus `MovementSegmentStarted` for unchanged positions.
- Existing stop/death/lock semantics remain intact.

Actual result:

- Success.
- Added `battle_core_does_not_record_movement_segment_without_body_motion`.
- Existing movement lock/death/no-goal related tests remained green under `cargo test -p game_core movement:: -- --nocapture` and full `cargo test -p game_core`.

## Verification Log

### Failed Command Attempt

```text
Command:
cargo test -p game_core battle_core_records_immutable_movement_segments_per_tick battle_core_does_not_record_movement_segment_without_body_motion -- --nocapture

Result:
Failed before running tests.

Failure cause:
cargo test accepts one test-name filter before `--`; the second test name was parsed as an unexpected argument.

Fix:
Ran the two focused tests separately.
```

### Focused Verification

```text
Command:
cargo test -p game_core battle_core_records_immutable_movement_segments_per_tick -- --nocapture

Result:
Passed. 1 passed.
```

```text
Command:
cargo test -p game_core battle_core_does_not_record_movement_segment_without_body_motion -- --nocapture

Result:
Passed. 1 passed.
```

```text
Command:
cargo test -p game_core live_battle_update_exposes_adjacent_immutable_movement_segments -- --nocapture

Result:
Passed. 1 passed.
```

### Plan Verification Commands

```text
Command:
cargo fmt -p game_core

Result:
Passed.
```

```text
Command:
cargo test -p game_core movement:: -- --nocapture

Result:
Passed. 58 passed.
```

```text
Command:
cargo test -p game_core live_defense_ -- --nocapture

Result:
Passed. 8 passed.
```

```text
Command:
cargo test -p game_core generated_defense_route_enemy_progresses_past_spawn_boundary -- --nocapture

Result:
Passed. 1 passed.
```

```text
Command:
cargo check -p game_core

Result:
Passed.
```

```text
Command:
cargo test -p game_core --test ron_loading -- --nocapture

Result:
Passed. 16 passed.
```

```text
Command:
cargo test -p game_core

Result:
Passed. 467 unit tests, 3 live item tests, 3 live skill catalog tests, 16 RON loading tests, 10 skill refactor validation tests, 14 skill suite tests, and doc tests passed.
```
