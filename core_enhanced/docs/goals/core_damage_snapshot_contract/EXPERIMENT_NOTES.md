# Experiment Notes

## Design Notes

- Long-term direction: source-side damage context should be committed at the damage cause point, while target-side mitigation/HP context should remain live at apply/impact time.
- This direction aligns with the existing projectile-after-attacker-death policy, but generalizes it into an explicit contract instead of relying on live/graveyard fallback.
- The current `DamageContext` mixes source and target context. The implementation should split it rather than adding more optional fields to the same mixed struct.
- `BasicAttackDamageSnapshot` should either be renamed or replaced. Its current name is misleading because it is created at resolve time, not at attack-start/schedule time.
- `BattleCommand::ApplyDamage` is the likely central seam for skill/buff/environment damage. It should eventually carry a source snapshot or a source snapshot id, not just `source_id`.
- Target-side on-hit effects are currently evaluated at apply time. Keep that behavior for the initial long-term implementation unless the user explicitly changes policy.
- Source-side on-attack effects are more policy-sensitive. Freezing their effect list at source snapshot time is clearer for determinism, but executing source-triggered commands after the source dies may affect gameplay meaning.
- Crit roll timing is policy-sensitive. Freezing the crit roll at source snapshot time maximizes determinism; calculating it at apply time from stored source/target/time inputs preserves current style with less timeline payload.
- Implementation note: `DamageContext` was kept as the calculation input shape. The new source snapshot is converted into the existing source-side damage effect inputs, while target-side fields are still collected at apply/impact time.
- Implementation note: targetless or delayed skill projectiles store a launch source snapshot template with source identity, source attack, committed timestamp, and timeline sequence. Target-specific crit is materialized when the actual hit target is known, using the frozen source snapshot inputs.
- Implementation note: `damage_source_snapshot_template_for_unit(...)` can read `graveyard` only while creating a new committed snapshot at an event's commit point. The old apply/impact-time source fallback was removed from `ApplyDamage` and projectile impact handling.
- Implementation note: source-side damage modifiers and bonus damage are frozen as data. Source-side command/ability triggers are collected only when the source is still live, matching the confirmed death policy.

## Policy Questions To Stop For

If implementation reaches any of these points, record `사용자와 정책 논의 필요`, write a policy-decision report, and complete the active goal instead of deciding silently:

- Should timeline events expose damage source snapshot fields, or should snapshots remain internal runtime state only?
- Should source-side on-attack trigger effects execute if the source dies before impact?
- Should target-side on-hit effects be live at impact/apply time, or frozen at the original targeting/launch point?
- Should crit chance/critical result be frozen in the source snapshot, or calculated at apply time from stored seed inputs?
- Should skill multi-hit steps share one source snapshot per step, or create one source snapshot per target hit?
- Should delayed projectile skill damage use the skill step snapshot or a separate projectile launch snapshot when those moments differ?

## Policy Decision Report

Status: resolved by user confirmation on 2026-06-22.

Implementation had been paused before runtime code changes because the current code required decisions that this goal document explicitly marked as policy-sensitive.

Required decisions:

1. Crit roll timing
   - Current code calculates `DamageRequest.crit_roll_percent` at apply/impact time through `damage_roll_percent(...)`.
   - A source snapshot contract can either freeze the critical result/roll at source snapshot time, or store deterministic seed inputs and calculate at apply time.
   - This affects same-timestamp attacks and delayed projectiles, so it is not a private refactor detail.

2. Source-side on-attack trigger/effect behavior after source death
   - Current instant basic attack collects source `OnAttack` effects at resolve time.
   - Current basic attack projectile drops source-side effects if the attacker is dead at impact.
   - Freezing source-side effects at launch/commit time would preserve damage modifiers, but command-producing trigger effects raise the gameplay question of whether dead sources may still execute source-triggered commands.

3. Skill multi-hit source snapshot granularity
   - Current `build_skill_step_commands(...)` emits one `ApplyDamage` command per target.
   - A snapshot can be shared per step, or created per target hit.
   - Per-target snapshots preserve target-specific roll inputs more naturally; per-step snapshots make the skill step a single committed damage source.

4. Delayed skill projectile snapshot timing
   - Current projectile impact rebuilds skill damage commands at impact.
   - A source snapshot can be captured at skill step execution/dispatch, or at projectile launch.
   - The plan recommends projectile launch for skill projectiles, but this still needs confirmation because some skill steps may separate step execution, delivery launch, and final impact.

5. Internal event/runtime state shape
   - Implementing instant basic attack source snapshots likely requires carrying snapshot data through `BattleEvent::AttackResolve` or through an internal pending-snapshot store keyed by the resolve event.
   - This does not require changing serialized `TimelineEvent`, but it does change the internal battle scheduling contract.

Confirmed decisions:

- Freeze crit roll from source snapshot inputs.
- Preserve committed source-side damage modifiers/bonus damage after source death, but do not execute source-side command/ability triggers after source death.
- Create target-specific source snapshots for skill multi-hit.
- Use projectile launch source snapshots for delayed skill projectile damage.
- Carry the source snapshot directly on internal `BattleEvent::AttackResolve`.
- Keep serialized `TimelineEvent`, Unity DTO, and live RON schema unchanged in this goal.

Implementation result:

- Completed within the confirmed policy bounds.
- No serialized `TimelineEvent` shape, Unity-facing DTO shape, or live RON schema was changed.
- No further policy question was encountered during final implementation.
- Remaining possible follow-up: decide later whether debug/replay tools should expose source snapshots as an explicit typed trace. This is intentionally not part of this goal.

## Follow-Up Candidates

- If timeline snapshot payload is approved later, add validator rules proving `HpChanged` damage metadata matches the referenced source snapshot.
- Consider adding a debug-only damage trace view after the core contract is stable.
- Consider consolidating damage preview (`target_usefulness.rs`) with the same source/target context builders so preview and runtime cannot drift.
