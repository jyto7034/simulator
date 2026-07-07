# Trigger Proc Roll Identity Decoupling Notes

## Initial Finding

This is not the same bug as damage crit rolls.

Damage crit rolls previously used event log sequence as gameplay RNG seed material. Triggered ability proc rolls do not currently use `event_log_seq` directly.

Current proc roll inputs are effectively:

```text
battle seed
activation source
ability id
binding index
current battle time
successful trigger count
```

The problem is subtler: successful trigger count is state, not candidate identity.

## Chosen Policy Direction

Use explicit proc occurrence identity for RNG, while preserving proc state for cooldown and max-trigger rules.

```text
ProcRollIdentity -> roll percent
AbilityProcState -> trigger_count / next_ready_ms
```

These two responsibilities should not be conflated.

## Important Design Notes

`trigger_count` should remain. It is still useful for:

- `max_triggers_per_battle`;
- internal cooldown state;
- debug/proc state visibility;
- tracking how many times a binding actually fired.

But RNG should answer a different question:

```text
Which proc candidate is being judged?
```

That candidate identity should come from trigger occurrence context.

## Open Questions For Implementation

- Resolved: `ProcRollIdentity` lives in `src/game/battle/damage.rs` because it is carried by `BattleCommand::TriggerAbility`.
- Resolved: `BattleCommand::TriggerAbility` carries `ProcRollIdentity` directly. The conversion boundary now takes `TriggerAbilityContext` so the identity is created before trigger occurrence context is lost.
- Resolved: `should_fire_triggered_ability(...)` derives the proc state key from `ProcRollIdentity` rather than accepting duplicate source/ability/binding context.
- Resolved: occurrence ids are assigned from stable existing runtime identities where possible:
  - instant basic attack: committed damage source instance id;
  - basic attack projectile impact: projectile id;
  - death/killer/ally-death triggers: deterministic death occurrence id;
  - battle start: deterministic per-unit battle-start occurrence id.
- Resolved for current runtime: current trigger activation commands are per proc binding command. The implementation preserves that unit of work and adds occurrence identity without changing trigger timing or target semantics.

## Policy Boundaries

- Do not change proc chance values.
- Do not change proc trigger timing.
- Do not change cooldown or max-trigger behavior.
- Do not remove `AbilityProcState.trigger_count`.
- Do not change Unity-facing DTOs.
- Do not change live RON.
- Do not introduce call-order-dependent RNG.

## Follow-Up Candidates

- After implementation, merge the general RNG identity rule into a canonical combat policy document.
- Consider a shared naming pattern for `CombatRollIdentity` and `ProcRollIdentity`.
- Add a replay-level stability test comparing battle outcomes with harmless extra log entries.
