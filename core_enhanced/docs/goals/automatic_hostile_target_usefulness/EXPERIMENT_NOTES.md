# Automatic Hostile Target Usefulness Experiment Notes

This file records runtime findings, design judgments, policy questions, and follow-up candidates discovered while implementing automatic hostile target usefulness.

## Fixed Policy

- Automatic hostile targeting chooses only targets that can receive deterministic final damage greater than zero or at least one hostile target-applied effect.
- Hostile target-applied effects are enemy-applied state changes such as debuff, control/status ailment, forced movement, DoT, stat/defense/resistance reduction, vulnerability, mark, aggro/threat manipulation, or beneficial-effect removal.
- Self buffs, ally buffs/heals/shields, resource gain, cooldown reduction, visual/audio-only effects, projectile spawn itself, summon/spawn effect itself, global-only effects, and no-state-change triggers do not make an enemy useful.
- Already-applied, refreshable, stackable, resisted, or immune status is not checked at targeting time unless it is already represented by a skill target filter.
- Valid candidates continue to use existing targeting profile ordering. Do not add a new priority between damage-capable targets and effect-only targets.
- AoE automatic cast gating uses the same usefulness rule and requires at least one useful enemy in the affected area.

## Initial Runtime Questions To Answer

- Where is final damage currently calculated, and can targeting call a deterministic preview without mutating combat state?
- Does basic attack target selection already know enough about attack damage type and modifiers to estimate final damage?
- Does skill cast targeting know which step effects will apply to the selected target?
- How are target-applied hostile effects represented in `SkillStepDef` and runtime effect execution?
- Can `RetargetOnStep` reuse the same usefulness predicate as cast targeting?
- Does `TileArea` have a clean pre-execution path for listing affected enemies before deciding whether to cast?
- Which existing tests encode current behavior around zero-damage targets or immune feedback?

## Runtime Findings

- Basic attack targeting has several entry points: blocked target, hinted target, persisted target, and profile-based candidate search. The usefulness filter must sit inside the shared candidate predicates, not only the fallback search.
- Skill target selection uses both cast-time target resolution and step-time target resolution. `ReuseCastTarget` and `RetargetOnStep` need different checks:
  - `ReuseCastTarget` must revalidate that the stored unit target is still alive, targetable, and useful for the step.
  - `RetargetOnStep` must run the normal target rule again with the current step as the usefulness predicate.
- Automatic cast startup cannot rely on `skill.first_step()`. Self-setup skills can have the first useful hostile action in a later step.
- Tile-area skills need a pre-application usefulness gate. The gate should check affected units after tile geometry is resolved and before declaring/spawning the area.
- Persistent tile areas should not be registered if their initial affected area contains no useful hostile target.
- `ExtraAttack` is a hostile payload only through the basic attack it schedules. Its usefulness should therefore delegate to basic-attack usefulness for the hinted target.
- A skill whose enemy cast target has no hostile target-consuming step should not choose an enemy target. Falling back to "no usefulness filter" would reintroduce legacy no-op targeting.
- Live Punishing Bird data uses `front_3x2`. Focused tests must place the target in the live forward range; smoke tests that broaden all skill ranges are not sufficient to validate the focused fixture.

## Final Implementation Judgments

- Use deterministic damage preview with `crit_roll_percent: None`.
- Basic attack preview keeps the existing basic attack minimum damage behavior.
- Skill damage preview uses each `Damage` effect with `minimum_damage: 0` so immunity/mitigation can make the target useless.
- Hostile `ModifyStats` is currently inferred by modifier direction:
  - `MaxHealth`, `Attack`, `Defense`, `MagicResist` are hostile when value is negative.
  - `AttackIntervalMs` is hostile when value is positive.
  - `MoveSpeedUnitsPerMs` is hostile when value is negative.
- `ApplyBuff` is considered hostile when it is applied to a hostile target. The policy intentionally does not predict stack/refresh/resist/immunity state.
- `Heal`, `ModifyResonance`, `ModifyStabilization`, `ModifyDamage` alone, projectile spawn itself, and targetless delivery are not hostile target-applied effects.
- Automatic hostile skills wait when no useful hostile target exists. They do not start focus, emit a cast-start event, or spend resonance just to whiff.

## Stop-And-Ask Policy Questions

Stop the goal and ask the user if any of these are discovered:

- The implementation needs a new live RON field to mark hostile effects.
- The implementation needs a new target priority rule between damage and effect-only targets.
- The implementation needs special-case policy for The Silent Orchestra or another named abnormality.
- The implementation would change Unity-facing DTO shape or timeline event meaning.
- The implementation would change damage formula, damage type balance, or skill effect semantics.
- The implementation requires deciding whether a specific current skill should be a debuff/control/effect-only skill.

## Follow-Up Candidates Outside This Goal

- Add richer effect taxonomy if future skills need precise stack/refresh/immunity target prediction.
- Add boss/phase encounter target-role policies only if a future encounter needs behavior beyond the general usefulness filter.
- Add Unity warning/UI language for "no useful target" only if the core exposes an explicit reason in a future DTO.
