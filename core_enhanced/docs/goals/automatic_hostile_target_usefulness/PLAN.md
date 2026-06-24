# Automatic Hostile Target Usefulness Plan

## Objective

Implement the automatic hostile targeting policy documented in `docs/skill_target_contract.md`: an enemy target is useful only when the attack or skill can produce deterministic final damage greater than zero, or can apply at least one hostile target-applied effect to that enemy.

The implementation must be long-term targeting behavior, not a special case for The Silent Orchestra or a patch that only satisfies one test.

## Policy To Implement

Automatic hostile targeting must use this filter before normal target ordering:

1. A target is valid if deterministic expected final damage is greater than zero.
2. A target is valid if final damage is zero but the attack or skill has at least one hostile target-applied effect for the selected enemy or an enemy inside the AoE.
3. A target is invalid if both final damage and hostile target-applied effects are absent.
4. Multiple valid targets are still ordered by the existing targeting profile, route, distance, threat, and deterministic tie-breaker rules.
5. AoE skills are automatic cast candidates only when the affected area contains at least one valid enemy by the same rule.
6. If no valid target exists, the attack or attack skill is not used.

Hostile target-applied effects include debuff, control/status ailment, forced movement, damage-over-time, stat/defense/resistance reduction, vulnerability, mark/targeting modifier, aggro/threat manipulation, and beneficial-effect removal.

They do not include caster self buffs, ally buffs/heals/shields, resource gain, cooldown reduction, visual/audio-only effects, projectile spawn itself, summon/spawn effect itself, global battle state changes only, or triggers that leave no state change on the enemy target.

Targeting does not check whether the effect is already present, refreshable, stackable, resisted, or immune unless that limit is already represented by the skill target filter.

## Scope

In scope:

- Read current basic attack targeting, skill cast targeting, `RetargetOnStep`, `TileArea`, damage calculation, and effect definitions.
- Add a shared usefulness predicate or equivalent long-term abstraction used by automatic hostile target selection.
- Ensure basic attacks do not keep selecting enemies that deterministic damage resolves to zero.
- Ensure skill target selection and retargeting use the same usefulness rule.
- Ensure AoE automatic cast checks require at least one useful enemy in the affected area.
- Preserve existing targeting profiles and tie-breakers after usefulness filtering.
- Add behavior tests that pin user-visible targeting behavior.
- Update docs if runtime evidence shows the policy needs clearer wording.

Out of scope:

- Manual target selection, because current design does not expose direct manual enemy target selection.
- Damage formula redesign.
- Balance tuning for damage types, phase timings, or The Silent Orchestra waves.
- New targeting profiles such as `BossFirst`.
- New live RON schema unless runtime evidence proves the policy cannot be represented otherwise.
- Unity-facing DTO shape changes unless unavoidable.
- Compatibility layers for legacy targeting behavior.

## Source Of Truth

Verify in this order:

1. Runtime code:
   - `src/game/battle/core/targeting.rs`
   - `src/game/battle/core/basic_attack.rs`
   - `src/game/battle/core/sim.rs`
   - `src/game/battle/core/commands.rs`
   - `src/game/ability.rs`
   - damage/effect execution modules discovered from the code
2. Live RON/data:
   - `../game_resources/data/skills/base.ron`
   - live unit/equipment data that defines basic attack damage and skill effects
3. Unity-facing snapshot/command/timeline contracts:
   - current battle update/setup docs and DTO tests, only if this behavior changes visible events
4. Policy docs:
   - `docs/skill_target_contract.md`
   - `docs/game_rulebook.md`
   - `docs/skills/abnormality_skill_design_notes.ko.md`

Do not trust the documents blindly. If code and live data show a cleaner or safer long-term implementation, record the evidence in `EXPERIMENT_NOTES.md`. If a new gameplay policy decision is required, stop and ask the user.

## Plan

1. [x] Inventory current runtime flows.
   - Basic attack target candidate filtering.
   - Skill cast target selection.
   - `ReuseCastTarget` validation.
   - `RetargetOnStep` validation.
   - `TileArea` affected enemy selection and automatic cast gating.
   - Current damage and effect execution code.
2. [x] Identify the smallest shared runtime concept that can answer "is this hostile target useful for this attack/skill?"
   - Prefer a pure helper that uses existing damage/effect definitions.
   - Avoid duplicating damage formula logic in targeting.
   - Avoid making targeting depend on presentation/UI concepts.
3. [x] Add focused tests before or alongside implementation.
   - Pure damage against immune/zero-final-damage enemy is excluded.
   - Damage greater than zero remains targetable.
   - Zero-damage hostile effect skill remains targetable.
   - Self/ally/global/projectile/summon-only effects do not make an enemy target useful.
   - AoE automatic cast requires at least one useful enemy in the area.
   - Existing targeting profile ordering remains unchanged among valid candidates.
4. [x] Implement in small slices.
   - Basic attack usefulness filtering.
   - Skill cast target usefulness filtering.
   - Retarget step usefulness filtering.
   - AoE cast usefulness filtering.
5. [x] Run focused checks after each slice and record failures in `EXPERIMENTS.md`.
6. [x] Run broad package tests at the end.
7. [x] Update policy docs only if implementation evidence changes wording or reveals a missing edge case.

## Completion Conditions

- Automatic hostile targeting excludes targets that would receive zero deterministic final damage and no hostile target-applied effect.
- Basic attacks, skill cast targeting, `RetargetOnStep`, and AoE automatic cast gating use the same policy.
- Existing target ordering profiles still decide among valid candidates.
- No The Silent Orchestra-specific targeting code is introduced.
- No legacy compatibility layer, fallback path, or dual behavior is introduced.
- Tests cover user-visible behavior, not just helper internals.
- Live RON loading remains valid.
- Relevant docs remain synchronized with runtime behavior.
- Focused and broad verification commands pass.
- 사용자와 의논하여 정해야 할 정책이 발견되면 goal을 종료한다.

## Stop Conditions

- Runtime damage prediction cannot reuse deterministic damage logic without duplicating or changing the damage formula.
- Skill effect data does not expose enough information to distinguish hostile target-applied effects from self/ally/global/projectile/summon-only effects.
- Implementing the policy requires a live RON schema change.
- Implementing the policy changes Unity-facing DTO shape, timeline event names, or command contracts.
- The policy would require balance decisions for a specific boss, wave, item, skill fragment, or damage type.
- Existing tests encode a conflicting gameplay policy and the correct replacement behavior is not obvious from the canonical docs.
- 사용자와 의논하여 정해야 할 정책이 발견된다.

## Implementation Summary

- Added a shared battle-core usefulness helper for basic attacks, skill single-target steps, retargeted steps, instant/persistent tile areas, and automatic cast startup.
- Basic attacks now filter hostile candidates before existing target ordering.
- Skill target selection now filters hostile candidates before existing `CurrentTarget`, `Nearest`, `LowestHealthEnemy`, defense tile range, airborne, and deterministic tie-breaker rules.
- Automatic hostile skills now wait instead of consuming focus/resonance when no useful hostile target exists.
- `RetargetOnStep` can still find a useful target at execution time even when the cast target came from a self-targeted setup step.
- `ExtraAttack` counts as useful only when the scheduled basic attack would itself be useful against the target.
- Updated the Punishing Bird skill test fixture so the live `front_3x2` skill is tested against a target in front of the caster instead of relying on broad test-only targeting.

## Removed Legacy Behavior

- Removed implicit targeting of hostile units that would receive zero final damage and no hostile target-applied effect.
- Removed automatic skill start/whiff behavior that could spend focus/resonance without a useful hostile target.
- Removed no-op hostile skill fixtures from policy-sensitive tests by replacing them with explicit damage/effect payloads where the old no-op behavior was not the subject under test.

## Verification Commands

Run focused checks as the implementation evolves:

- `cargo check -p game_core --tests`
- `cargo test -p game_core targeting -- --nocapture`
- `cargo test -p game_core skill -- --nocapture`
- `cargo test -p game_core battle::core -- --nocapture`
- `cargo test -p game_core --test skill_refactor_validation -- --nocapture`
- `cargo test -p game_core --test ron_loading -- --nocapture`

Run broad checks before completion:

- `cargo fmt`
- `cargo test -p game_core`
- `cargo check -p game_server --tests`
