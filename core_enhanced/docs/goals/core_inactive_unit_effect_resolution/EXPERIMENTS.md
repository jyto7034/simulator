# Experiments

| Date | Attempt | Result | Follow-up |
|---|---|---|---|
| 2026-06-23 | Goal setup | Created the subgoal document for inactive-unit projectile and delayed-effect resolution. | Run after lifecycle and redeploy policies are in place. |
| 2026-06-23 | Runtime inventory | Read basic attack projectile, skill projectile, command damage/effect processing, withdrawal cleanup, and redeploy lock code. Found that current code skips non-active targets, but changing that directly would leave unsettled semantics for non-damage effects and redeploy HP after withdrawal. | Created `POLICY_DECISION_REPORT.md` and completed the subgoal with policy discussion required. |
| 2026-06-23 | Withdrawn target projectile implementation | Confirmed policy is withdrawal-as-evasion. Hardened basic attack impact so missing/non-active targets record miss before hit, and added tests for locked basic attack projectile vs withdrawn target and skill projectile impact vs withdrawn target. | Run `cargo check`, broad tests, and completion review. |
| 2026-06-23 | Focused validation | `cargo fmt`, `cargo test advance_basic_attack_projectile_misses_when_locked_target_withdraws_before_contact`, and `cargo test skill_projectile_impact_does_not_hit_withdrawn_target` passed. | Continue with broad validation. |
| 2026-06-23 | Regression validation | `cargo check`, `cargo test advance_basic_attack_projectile_misses_when_locked_target_dies_before_contact`, `cargo test advance_basic_attack_projectile_uses_launch_side_after_attacker_death`, `cargo test skill_projectile_impact_uses_launch_source_snapshot_after_caster_death`, and full `cargo test` passed. | Record completion review. |
