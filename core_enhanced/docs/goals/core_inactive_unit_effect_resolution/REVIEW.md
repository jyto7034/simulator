# Core Inactive Unit Effect Resolution Review

Review guide: `docs/goal_completion_review_guide.md`

Review date: 2026-06-23

## Item Verdict

| Item | Verdict | Long-term fit | Evidence |
|---|---|---|---|
| Withdrawn units excluded from new offensive candidates | Complete | High | `RuntimeUnit::is_active()` is the participation gate; targeting, skill targeting, command targets, and build snapshots use active-only filters. |
| Basic attack projectile locked before withdrawal | Complete | High | Withdrawal is a strong defensive/evasion action against opponent hostile projectiles; `advance_basic_attack_projectile` resolves non-active locked targets as `BasicAttackProjectileImpacted { hit: false }`; `apply_basic_attack_projectile_hit_at` now also guards missing/non-active targets before hit logging. |
| Skill projectile locked/impacting withdrawn target | Complete | High | Homing projectile advance requires `unit.is_active()`; impact filters `first_hit_unit_id` through active-only before event logging and command generation. |
| Dead target duplicate damage prevention | Complete | High | Same active-only gates cover `Dead`; existing dead-target focused test still passes. |
| Attacker launch snapshot preservation | Complete | High | Basic attack and skill projectile launch snapshot tests pass after inactive target changes. |
| Projectile cleanup strategy | Complete | High | Withdrawal does not clear the projectile registry globally; projectiles resolve at normal advance/impact boundaries. |

## Evidence

Runtime evidence:

- `src/game/battle/core/commands.rs`
  - `advance_basic_attack_projectile` checks `!target.is_active()` before sampling or hit resolution and records `BasicAttackProjectileImpacted { hit: false }`.
  - `apply_basic_attack_projectile_hit_at` now records miss and returns when the target is missing or non-active before any `hit: true` event can be emitted.
  - `process_commands` uses active-only target gates for damage/effect resolution paths.
- `src/game/battle/core/skill_runtime/projectile.rs`
  - `advance_homing_skill_projectile_runtime` treats missing/non-active locked targets as terminal no-hit impacts.
  - `apply_skill_projectile_impact` filters `first_hit_unit_id` by `unit.is_active()` before recording `SkillProjectileImpacted` and building commands.
- `src/game/battle/core/targeting.rs`
  - New candidate targeting remains active-only.

Data evidence:

- No live RON/data schema was changed for this subgoal.
- No projectile range, skill schema, reward, or map data fallback was introduced.

External contract evidence:

- Basic attack miss remains `BattleLogEvent::BasicAttackProjectileImpacted { hit: false }`.
- Skill projectile miss/no-hit remains `BattleLogEvent::SkillProjectileImpacted { first_hit_unit_id: None, ... }`.
- No new Unity-facing DTO/event variant was added in this subgoal.

Test evidence:

- Added `advance_basic_attack_projectile_misses_when_locked_target_withdraws_before_contact`.
- Added `skill_projectile_impact_does_not_hit_withdrawn_target`.
- Re-ran dead-target and attacker snapshot regression tests.

Legacy/fallback audit:

- No compatibility layer, dual schema, ignored test, or global projectile cleanup fallback was added.
- Search confirmed there is no canonical `ProjectileMiss` event path to reintroduce duplicate miss semantics.

Long-term direction review:

- Fit: high.
- Improvement class: none.
- Reason: the implementation uses `RuntimeUnitLifecycle`/`is_active()` as the single gameplay participation source, keeps projectile records as normal runtime objects, and resolves inactive targets at the same advance/impact boundaries that already own projectile outcomes.

## Validation

Passed:

- `cargo fmt`
- `cargo test advance_basic_attack_projectile_misses_when_locked_target_withdraws_before_contact`
- `cargo test skill_projectile_impact_does_not_hit_withdrawn_target`
- `cargo check`
- `cargo test advance_basic_attack_projectile_misses_when_locked_target_dies_before_contact`
- `cargo test advance_basic_attack_projectile_uses_launch_side_after_attacker_death`
- `cargo test skill_projectile_impact_uses_launch_source_snapshot_after_caster_death`
- `cargo test`

## Remaining Risk

- Non-projectile delayed effect families rely on existing `process_commands` active-only gates. This matches the confirmed policy, but future content that intentionally affects withdrawn units must introduce an explicit policy and schema instead of bypassing these gates.
- `UnitWithdrawn`/`UnitDied` position contracts and debug/admin/replay inactive-unit visibility remain for `core_lifecycle_event_snapshot_contract`.

## Verdict

Complete / High fit.
