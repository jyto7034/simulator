# Policy Decision Report: Runtime Unit Lifecycle Withdrawal Cleanup Events

Status: 정책 확정, 구현 대기

## Blocking Area

`core_runtime_unit_lifecycle` cannot be implemented cleanly with the currently confirmed policy because withdrawal cleanup requires Unity-facing event-log semantics that are not represented by the current contracts.

The confirmed policy says:

- `RuntimeUnitLifecycle { Active, Withdrawn, Dead }` is the canonical active/dead/withdrawn gate.
- Withdrawn units remain in `BattleCore.units`.
- Withdrawn units are excluded from active gameplay loops.
- Active gameplay state is cleared on withdrawal.
- Battle runtime buffs/debuffs are removed from withdrawn units.
- No `ActionState::Withdrawn` is added.

The code exposes a missing decision: when withdrawal clears buffs/casts/action state, the event log has no precise withdrawal-specific reason.

## Code Evidence

- `src/game/battle/event_log.rs`
  - `BattleLogEvent::UnitWithdrawn { unit_instance_id }` exists.
  - `BuffExpireReason` variants are only `Natural`, `Replaced`, `TargetDied`, and `CasterDied`.
  - `SkillCastInterrupted` requires `interrupter_instance_id`, `caster_instance_id`, `interrupted_skill_id`, and `interrupted_cast_seq`; it does not model self-withdrawal or lifecycle cleanup as a first-class reason.
- `src/game/battle/core/commands.rs`
  - death cleanup calls `clear_active_buffs_for_dead_unit`, which emits `TargetDied` / `CasterDied`.
  - reusing those variants for withdrawal would make withdrawn units look dead in the event log.
- `src/game/battle/core/build.rs`
  - current `withdraw_unit` removes the unit from `BattleCore.units`, which avoids the missing lifecycle/event semantics by deleting runtime identity. The confirmed lifecycle policy requires replacing this with `Withdrawn`, not deletion.

## Decision Needed

How should event-log contracts represent cleanup caused by withdrawal?

Decision: choose option 1. Withdrawal cleanup must be explicit in typed event-log contracts.

## Options

1. Add withdrawal-specific event reasons.
   - Add `BuffExpireReason::TargetWithdrawn` and `BuffExpireReason::CasterWithdrawn`.
   - Add a withdrawal/self-cancel representation for active casts, either:
     - `BattleLogEvent::SkillCastCancelled { caster_instance_id, interrupted_skill_id, interrupted_cast_seq, reason: SkillCastCancelReason::Withdrawn }`, or
     - extend `SkillCastInterrupted` with a reason and optional interrupter.
   - Pros: explicit, replay/debug/Unity can distinguish death from withdrawal, matches long-term SoT.
   - Cons: Unity-facing event enum contract changes and validators/tests must be updated.

2. Remove withdrawal-cleared buffs/casts silently.
   - Pros: smallest DTO change.
   - Cons: event log no longer explains why visible buffs/casts vanished; replay/debug can drift from runtime state.

3. Reuse death/interruption reasons.
   - Use `TargetDied` / `CasterDied` and `SkillCastInterrupted` with the withdrawing unit as interrupter.
   - Pros: minimal schema change.
   - Cons: semantically false; withdrawal becomes indistinguishable from death or external interruption.

4. Keep buffs/casts but ignore them while lifecycle is `Withdrawn`.
   - Pros: avoids event contract change.
   - Cons: violates the confirmed policy that battle runtime buffs/debuffs are removed from withdrawn units; preserves inactive state as hidden legacy.

## Recommendation

Chosen policy: option 1.

Reason:

- The project direction prefers explicit typed contracts over overloaded meanings.
- Withdrawal is neither death nor natural buff expiry.
- `BattleCore.units` retaining inactive units makes debug/replay contracts more important, not less.
- Adding explicit withdrawal reasons avoids a new source-of-truth split between runtime state and event log.

## Suggested Follow-Up Policy Wording

When a unit transitions to `RuntimeUnitLifecycle::Withdrawn`, runtime cleanup emits withdrawal-specific event reasons:

- buffs where the withdrawn unit is the target expire with `BuffExpireReason::TargetWithdrawn`;
- buffs where the withdrawn unit is the caster expire with `BuffExpireReason::CasterWithdrawn`;
- active or pending casts owned by the withdrawn unit are cancelled with `BattleLogEvent::SkillCastCancelled { caster_instance_id, interrupted_skill_id, interrupted_cast_seq, reason: SkillCastCancelReason::Withdrawn }`;
- `SkillCastInterrupted` remains for external interruption and must not be overloaded for self-withdrawal cleanup;
- these cleanup events are part of the official event log and must be covered by validator and Unity-facing DTO tests.

## Goal Status

Per the master goal policy-decision completion condition, this was a valid completed deliverable for the previous run. The policy is now confirmed and can be implemented by the next lifecycle goal/resume.
