# Experiment Notes

- Keep candidate selection and effect resolution separate. Candidate selection should be Active-only; already locked/scheduled resolution may use snapshots/references.
- Do not implement withdrawal by clearing all projectiles.
- Be careful not to reintroduce `Battlefield.position_of` as a fallback; use body-derived positions or existing snapshots.
- Runtime inventory found a policy gap: allowing locked damage to affect withdrawn targets is straightforward locally, but it is unclear whether that damage should update the existing redeploy HP lock or only affect the old withdrawn runtime entity.
- Runtime inventory also found an effect-family gap: full `process_commands` resolution against withdrawn targets could reapply buffs/debuffs/modifiers after withdrawal cleanup, which conflicts with the confirmed withdrawal cleanup policy.
- Because the subgoal explicitly says to complete with `사용자와 정책 논의 필요` if a specific effect family should ignore withdrawal, this goal produced a policy-decision report instead of implementation.
- After user decision, withdrawal is treated as a strong defensive/evasion action against incoming hostile projectiles fired by the opponent. The long-term implementation keeps projectile records until each projectile's normal advance/impact boundary and resolves withdrawn targets as cancel/miss/no-hit there; it does not globally clear all projectiles during withdrawal.
- `apply_basic_attack_projectile_hit_at` must not assume callers already filtered lifecycle. It now owns the invariant that a missing/non-active target cannot produce `BasicAttackProjectileImpacted { hit: true }`.
- Skill projectile impact already canonicalizes `first_hit_unit_id` through `unit.is_active()` before recording `SkillProjectileImpacted`, so a withdrawn target becomes `first_hit_unit_id: None` and produces no damage/effect command.
- Delayed command processing remains active-only through existing target lifecycle gates. If a future effect family intentionally affects withdrawn units, that must be a new explicit policy rather than a fallback.
