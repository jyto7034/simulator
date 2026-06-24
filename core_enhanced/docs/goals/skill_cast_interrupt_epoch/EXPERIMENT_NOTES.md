# Skill Cast Interrupt Epoch Experiment Notes

This file records runtime findings, design judgments, policy questions, and follow-up candidates discovered while implementing skill cast interruption with cast epoch/token protection.

## Fixed Policy

- `Silence` prevents new skill casts from starting. It does not cancel an already pending cast.
- Cast interruption is a separate skill effect.
- The cast-start timeline `seq` is the cast epoch/token.
- `PendingSkillCast` must store the start `seq`.
- `ManualCastEnd` and `AutoCastEnd` must only invoke when their parent `seq` matches the current pending cast token.
- A scheduled cast-end event whose token no longer matches is stale and must not invoke.
- Successful interruption removes the pending cast and emits a Unity-facing timeline event.
- Interruption does not cancel already invoked projectiles, areas, queued skill steps, ticks, triggered effects, or other runtime objects.
- Spent cast resources are not refunded by default.
- Interrupt-only hostile effects are target-useful only against a unit that currently has a pending skill cast.

## Initial Runtime Findings

- `PendingSkillCast` currently stores only `skill_id` and `cast_target`.
- Manual cast start records `TimelineEvent::ManualCastStart`, receives `start_seq`, then queues `BattleEvent::ManualCastEnd` with `cause: TimelineCause::Parent { seq: start_seq }`.
- Auto cast start records `TimelineEvent::AutoCastStart`, receives `start_seq`, then queues `BattleEvent::AutoCastEnd` with `cause: TimelineCause::Parent { seq: start_seq }`.
- Current cast-end handling takes `unit.pending_skill_cast` without checking that the queued end belongs to the same cast start.
- `BuffKind::Silence` exists and is already used as a start blocker for manual/automatic skill casts.
- `move_epoch` exists for movement interruption, but there is no skill cast epoch field.
- Timeline validators currently expect auto casts to end with `AutoCastEnd`; they will need to accept `SkillCastInterrupted` as a terminal outcome.
- `TimelineEvent` version is currently `TIMELINE_VERSION = 23`; adding a Unity-facing event likely requires bumping it.
- `SkillEffectDef` currently has `Damage`, `ModifyDamage`, `Heal`, `ModifyResonance`, `ModifyStabilization`, `ModifyStats`, `ApplyBuff`, and `ExtraAttack`; there is no interrupt effect variant.
- `target_usefulness` already centralizes hostile usefulness checks and should be extended rather than bypassed.

## Implementation Findings

- `BattleEvent::ManualCastEnd` and `BattleEvent::AutoCastEnd` already carry `TimelineCause::Parent { seq }` from the cast-start event, so no queue deletion or extra event id was needed.
- Token validation must happen before hard-CC delay rescheduling. Otherwise an already interrupted cast-end could be rescheduled again because `next_action_time` is still locked by another effect.
- `SkillCastInterrupted` should be a single event for both manual and auto casts. `interrupted_cast_seq` is enough to recover whether the canceled start was manual or auto.
- `ManualCastEnd`/`AutoCastEnd` should not be recorded for interrupted casts. In the presentation stream they mean successful cast completion, not "the old queue item arrived."
- `InterruptCast` can use the existing skill effect -> battle command -> process command path. This keeps Instant, Projectile, TileArea, and persistent TileArea ticks consistent.
- `start_manual_skill_cast` needed `pub(super)` visibility for battle-core tests to exercise the real lifecycle. This does not expose a new public gameplay API.
- `TIMELINE_VERSION` was bumped from 23 to 24 because the Unity-facing event enum changed.

## Initial Design Judgment

Use `CastStart` timeline `seq` as the cast token instead of introducing a separate `skill_cast_epoch` counter.

Reasons:

- The queued cast-end event already carries the start `seq` through `TimelineCause::Parent`.
- Unity presentation already uses the same event stream `seq` as its ordering and causality source.
- The interrupt event can identify the exact canceled `ManualCastStart` or `AutoCastStart` through `interrupted_cast_seq`.
- A separate counter would need new serialization/debug mapping without giving stronger ordering guarantees.

Expected event shape:

```json
{
  "type": "SkillCastInterrupted",
  "interrupter_instance_id": "...",
  "caster_instance_id": "...",
  "interrupted_skill_id": "...",
  "interrupted_cast_seq": 123
}
```

The exact Rust fields can be adjusted during implementation, but the event must let Unity map the interruption to a specific cast-start event.

## Runtime Questions To Answer

- Should `SkillCastInterrupted` cover both manual and auto starts with one event, or should manual/auto interruption be separate event variants?
- After interruption, should `next_action_time` remain at the original cast-end time, be set to `time_ms`, or be derived from action-lock policy?
- Should action locks be cleared on interruption or allowed to expire naturally?
- Should an interrupt effect count as applied when the target has no pending cast?
- Should the interrupt effect be allowed to hit allies in future support/control designs, or remain hostile-only for this goal?
- Where is the cleanest command boundary: `SkillEffectDef::InterruptCast` -> `BattleCommand::InterruptCast`, or direct runtime handling inside skill step execution?
- Which validators need explicit references to `SkillCastInterrupted`: autocast lifecycle, parent cause, deaths, spawns, battlefield positions, and transport DTO tests?

## Preferred Answers Unless Runtime Evidence Contradicts Them

- Use a single `SkillCastInterrupted` timeline event for manual and auto casts.
- Record `interrupted_cast_seq` and `interrupted_skill_id`; do not require Unity to infer the skill from historical lookup alone.
- Clear `pending_skill_cast` immediately on successful interrupt.
- Do not emit `ManualCastEnd` or `AutoCastEnd` for interrupted casts.
- Treat stale cast-end events as internal queue cleanup and do not emit a timeline event for them.
- Keep resource refund out of this goal.
- Keep action locks until their existing expiry after interruption. This avoids implicit refunds or immediate recovery. A future policy can shorten locks if gameplay needs it, but this goal does not grant that benefit.
- Implement through a battle command if that matches existing effect-to-command execution better than adding one-off skill runtime code.

## Stop-And-Ask Policy Questions

Stop the goal and ask the user if any of these are discovered:

- Whether interrupted casts should refund resonance, focus, cooldown, deployment cost, or other resources.
- Whether interruption should also cancel already spawned projectiles, declared areas, scheduled skill steps, or triggered effects.
- Whether interrupting a boss skill should use special boss-phase rules outside ordinary skill casting.
- Whether Unity wants separate event variants for `ManualCastInterrupted` and `AutoCastInterrupted`.
- Whether action locks after interruption should behave like stun, silence, cast completion, or immediate recovery.
- Whether interrupt effects should be allowed on allies or neutral units.
- Whether live content requires a richer RON schema than a simple interrupt effect variant.

## Follow-Up Candidates Outside This Goal

- Add The Silent Orchestra-specific skill fragments and special-node rewards after the generic interrupt runtime exists.
- Add UI-facing "interruptible" or "casting" indicators if Unity needs them beyond timeline events/checkpoint state.
- Add a future `CancelSkillRuntime` policy only if content explicitly needs projectile/area cancellation after invocation.
- Add richer control-effect taxonomy if future targeting needs to distinguish silence, interrupt, stun, freeze, and boss-only control immunity.
