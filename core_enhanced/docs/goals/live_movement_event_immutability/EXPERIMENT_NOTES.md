# Live Movement Event Immutability Experiment Notes

Created: 2026-06-17

Status: Implemented notes.

## Policy Decisions

### Live Events Are Immutable

Unity-facing `battle_update.events_delta.events[*]` must be immutable once seq is assigned and the event may be sent.

Reason:

- Seq-cursor delta transport assumes events are append-only.
- Mutating a previously emitted seq can leave Unity with a stale short movement segment.
- This makes presentation gaps possible even when core simulation moved continuously.

### Live Movement Accuracy Comes Before Log Compactness

Use 50ms immutable live movement segments for now.

Reason:

- It matches current simulation tick.
- It is easy for Unity to sample by presentation battle time.
- It avoids predictive long segments and cancellation complexity.
- Local offline gameplay should tolerate the event volume for current unit counts.

### Archive Coalescing Is A Future Optimization

Stored battle logs may eventually compress movement events, but this goal should not implement it unless storage volume blocks live correctness.

Future coalescing should happen outside the live event stream and should not change live event immutability.

Preferred future direction if needed:

- Either coalesce only same-velocity adjacent segments;
- Or introduce an archive-only movement track/sample-list representation.

Do not add a second Unity-facing schema for this goal.

## Implementation Cautions

- `active_movement_segments` may still be used for runtime sampling or cleanup. Do not delete it blindly.
- Check tests around `sample_position_at` and movement interruption before removing fields.
- Keep `MovementStopped` semantics intact.
- Do not use checkpoint position as a substitute movement event.
- Avoid adding fallback or dual-schema handling for older mutable movement logs.

## Questions To Stop For

Stop and ask the user if implementation reveals:

- Saved replay/archive consumers require coalesced movement as the canonical format.
- Unity cannot consume 50ms immutable movement segments.
- Event volume is already too high for local gameplay after focused testing.
- We need a new DTO such as `MovementTrack`.
- Movement stutter is caused primarily by block/targeting/route behavior rather than mutable event emission.

## Follow-Up Candidates

- Add archive-only movement coalescing after measuring real log size.
- Add a debug/test helper that reports movement coverage gaps per unit.
- Add a Unity smoke probe that verifies actor position is sampled from event time, not receive time.
- Add documentation examples for 50ms adjacent movement segments and checkpoint reconcile.
- Consider batch compression at WebSocket transport level only if local event volume becomes noisy.

## Implementation Notes

- `active_movement_segments` remains in the runtime because projectile and position sampling paths still need an active segment to sample from.
- `timeline_index` and `velocity` were removed from `ActiveMovementSegment` because they existed to support mutable timeline extension.
- No `MovementSegmentStarted` DTO shape change was needed.
- No stored/archive coalescing was implemented in this goal.
