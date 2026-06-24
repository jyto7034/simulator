# Basic Attack Projectile Target-Only Collision Experiments

## 2026-06-21 - Initial Code Audit

Status: implemented.

Observed runtime hooks:

- `src/game/battle/core/commands.rs::spawn_basic_attack_projectile()` creates the runtime projectile and records `BasicAttackProjectileLaunched`.
- `src/game/battle/core/commands.rs::advance_basic_attack_projectile()` currently applies damage at the scheduled impact time when the original target is still valid.
- `src/game/battle/core/types.rs::ProjectileRecord` already stores the locked `target_instance_id`, current projectile position, aim, speed, and guidance.
- `src/game/battle/core/commands.rs::sample_unit_body_at()` can sample a target's interpolated body at a battle time.
- `src/game/battle/core/spatial.rs::moving_circle_sweep_hit_fraction()` can compute continuous contact between projectile motion and a moving target body.
- `src/game/battle/core/skill_runtime/projectile.rs` already contains target-only sweep logic for homing skill projectiles and fixed sweep logic for directional collision skill projectiles.

Initial conclusion:

- The feature is implementable without changing Unity-facing event shape.
- The likely runtime change is localized to basic attack projectile advancement plus tests.
- The implementation must be careful not to restore broad path collision against arbitrary units.
- Runtime evidence from target-locked skill projectiles uses `SKILL_PROJECTILE_REEVALUATION_TICK_MS = 1` and `max_travel_ms` based on launch-time travel distance/time. Basic attack projectiles reuse that cadence and finite launch-data travel window rather than inventing a new grace value.
- `ProjectileRecord` can store `attacker_owner_at_launch`, `air_capable_at_launch`, `max_travel_ms`, and event `cause` without changing Unity-facing DTO shape.

## Trials

- Implemented `ProjectileRecord` launch snapshots and finite runtime state in `src/game/battle/core/types.rs`.
- Changed `spawn_basic_attack_projectile()` to schedule advance from launch time and preserve launch side/air-capable/max-travel/cause.
- Changed `advance_basic_attack_projectile()` to:
  - advance in 1ms bounded windows,
  - sample only the original locked target body,
  - sweep projectile movement against that target body,
  - hit at first contact,
  - miss on invalid target or finite travel expiration,
  - never enumerate path units or re-run tile range/usefulness.
- Added/updated tests for:
  - path bystander not being hit,
  - moving locked target hit by sweep,
  - dead locked target miss,
  - no tile-range recheck at arrival,
  - launch-side hostility after attacker death,
  - duplicate projectile advance idempotence.

Failed attempt:

- `cargo test -p game_core advance_basic_attack_projectile -- --nocapture`
- Result: compile failure in test module.
- Cause: new test literals used `TimelineCause::default()` without importing `TimelineCause` into the test module.
- Fix: added `TimelineCause` to the test module timeline imports.

## Validation Log

- `cargo fmt` - passed.
- `cargo check -p game_core` - passed.
- `cargo test -p game_core advance_basic_attack_projectile -- --nocapture` - failed once due to missing test import, then passed after import fix.
- Latest focused result: 6 projectile tests passed.
- `cargo test -p game_core` - passed: 496 lib tests, 3 live item tests, 3 live skill catalog tests, 16 RON loading tests, 12 skill refactor validation tests, 4 skill test suite tests, and doc-tests all passed.
