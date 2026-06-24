# Core Inactive Unit Effect Resolution

## Objective

Implement projectile and delayed-effect behavior around `Withdrawn` and `Dead` lifecycle states.

Confirmed policy source: `docs/goals/core_component_refactor_master/POLICY_DECISIONS.md`, decision 10 and related lifecycle decisions.

## Required Startup Protocol

At the beginning of this goal and every resumed run, read:

- `docs/goals/core_inactive_unit_effect_resolution/PLAN.md`
- `docs/goals/core_inactive_unit_effect_resolution/EXPERIMENTS.md`
- `docs/goals/core_inactive_unit_effect_resolution/EXPERIMENT_NOTES.md`
- `docs/goals/core_followup_policy_implementation_master/PLAN.md`
- `docs/goals/core_component_refactor_master/POLICY_DECISIONS.md`
- `docs/refactor_preparation_plan.md`

Do not start code search, edits, or validation before reading those documents.

## Scope

### In Scope

- Exclude `Withdrawn` units from new movement, targeting, skill, and new damage candidates.
- Cancel/miss projectile or delayed target effects launched, locked, or scheduled before withdrawal when the target becomes `Withdrawn`.
- Preserve attacker launch owner damage snapshot after death or withdrawal.
- Exclude `Dead` targets from damage/effect resolve to prevent double damage.
- Avoid deleting all projectiles on withdrawal.

### Out Of Scope

- Adding `RuntimeUnitLifecycle`.
- Redeploy identity and HP policy.
- Event DTO additions, except where tests need event evidence.

## Implementation Plan

1. Read projectile launch, target lock, delayed effect, damage snapshot, death, and withdrawal code.
2. Inventory where target liveness is checked at launch, lock, travel, impact, and damage application.
3. Separate new target candidate filtering from already-locked/scheduled effect resolution.
4. Preserve launch owner snapshots for attackers that later die or withdraw.
5. Ensure dead targets are excluded at resolve time.
6. Add tests for withdrawn target locked before withdrawal, withdrawn target not newly targetable, dead target not double-damaged, and attacker snapshot persistence.
7. Run focused projectile/skill/basic attack tests and `cargo check`.

## Policy Decision Completion Condition

If implementation reveals an unresolved timing or UX decision around when a target is considered locked/scheduled, or whether a specific effect family should ignore withdrawal, complete the goal with `사용자와 정책 논의 필요` and evidence.

## Completion Conditions

- Withdrawal is a strong defensive/evasion action for already locked incoming hostile projectiles fired by the opponent.
- Withdrawn target projectile/effect resolution cancels or misses without damaging old runtime HP or redeploy HP lock.
- Withdrawn units are not candidates for new offensive actions.
- Dead units do not receive duplicate damage/effects.
- Projectile cleanup is not used as a workaround for withdrawal.
- Focused and broad validation commands are recorded.

## Current Status

Status: complete; implementation and review passed.

Report: `docs/goals/core_inactive_unit_effect_resolution/POLICY_DECISION_REPORT.md`

Confirmed decision: withdrawal is a strong defensive/evasion action against incoming hostile projectiles fired by the opponent. Projectiles flying toward a target that becomes `Withdrawn` cancel/miss and do not damage old runtime HP, do not update redeploy HP locks, and do not convert old withdrawn units into `Dead`.

Implementation:

- `src/game/battle/core/commands.rs`
  - `advance_basic_attack_projectile` already resolves non-active locked targets as `BasicAttackProjectileImpacted { hit: false }`.
  - `apply_basic_attack_projectile_hit_at` now also guards missing/non-active targets before recording a hit, so direct impact resolution cannot log `hit: true` for `Withdrawn`/`Dead` targets.
  - Added `advance_basic_attack_projectile_misses_when_locked_target_withdraws_before_contact`.
  - Added `skill_projectile_impact_does_not_hit_withdrawn_target`.
- `src/game/battle/core/skill_runtime/projectile.rs`
  - Homing projectile advance and impact resolution keep `unit.is_active()` as the target gate, so `Withdrawn`/`Dead` targets resolve as no-hit instead of damage/effect targets.

Focused validation:

- `cargo fmt`
- `cargo test advance_basic_attack_projectile_misses_when_locked_target_withdraws_before_contact`
- `cargo test skill_projectile_impact_does_not_hit_withdrawn_target`
- `cargo check`
- `cargo test advance_basic_attack_projectile_misses_when_locked_target_dies_before_contact`
- `cargo test advance_basic_attack_projectile_uses_launch_side_after_attacker_death`
- `cargo test skill_projectile_impact_uses_launch_source_snapshot_after_caster_death`
- `cargo test`

Review: `docs/goals/core_inactive_unit_effect_resolution/REVIEW.md`
