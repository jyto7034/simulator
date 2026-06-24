# Tile Based Attack Delivery Contract Experiments

## Purpose

Record implementation experiments, failed approaches, test failures, and validation results for the tile-based attack delivery contract.

Do not erase failed attempts. Future work should be able to see why an approach was rejected.

## Experiment Log

### 2026-06-20 - Initial Goal Document

Status: planned.

Summary:

- User policy was clarified before implementation.
- Basic attacks should be tile-eligibility only.
- Basic attack tile range should be authored or resolved from explicit data. Employee no-weapon fallback is self plus one forward tile; corroded employees use profile-selected range presets from RON.
- `range_units` should remain for future proximity/contact/contagion and explicitly continuous skill mechanics.
- Directional collision projectiles must remain possible for skills that fire toward a tile/direction without requiring a unit target, and must support continuous sweep/piercing/max hits.
- Basic attack projectiles should become target-locked rather than path-collision based.

Evidence gathered:

- `DeliveryDef::{Instant, Projectile, TileArea}` already exists.
- `SkillProjectileCollisionDef` already exists and models continuous collision settings.
- Basic attacks currently still use continuous range in some paths.
- Skill projectile runtime already has continuous collision behavior that should be preserved for explicit collision projectile skills.

Result:

- No code was changed in this step.
- Implementation should begin with call-site classification before refactoring.

### 2026-06-20 - Plan Review Against Current Code

Status: reviewed.

Summary:

- Compared the plan against current targeting, basic attack, skill cast, skill projectile, data validation, and movement code.
- Confirmed the main plan is valid: basic attack and skill target eligibility currently mix tile range and continuous `range_units`/`UnitBody::can_reach`.
- Confirmed basic attack projectiles currently use active projectile sweep against the original target body rather than a target-locked impact contract.
- Confirmed skill projectile collision support already exists and should be preserved for explicit directional collision projectile skills.

Findings added back to `PLAN.md`:

- `src/game/battle/core/sim.rs` must be audited/refactored together with live runtime paths.
- Movement planner/engine `range_units` usage must be classified separately from attack eligibility.
- Live enemy/abnormality/corroded-employee basic attacks need policy-based migration before fallback is applied: corroded employee profiles must reference explicit RON range presets, and special long-range attacks need authored `defense_tile_range`.
- Target-locked projectile arrival must not re-run range or usefulness checks.

Additional user policy fixed after review:

- Employee no-weapon fallback range is self tile plus one forward tile.
- Corroded employee melee/ranged ranges are not inferred fallback. They come from profile-selected RON range presets.
- Existing authored `defense_tile_range` stays authoritative.
- Existing live corroded employee ranged basic attacks identified only by `range_units > 1` or projectile delivery should be made explicit as profile-selected range presets.
- Boss, abnormality, elite, or special attacks that intentionally need nonstandard range must receive a specific authored `defense_tile_range`.
- Target-locked projectile and directional collision projectile must be distinct policies.
- Directional collision projectiles are important future/current skill support: they move in the caster's current facing direction without a required target and hit by continuous collision during flight.
- Directional collision projectile direction is not chosen by cast-time user input; unit facing is the source of truth.
- Directional collision projectiles may support targetless single-shot casting only through an explicit skill option.
- Core simulation stops directional collision projectiles at the end of their authored forward tile range path.
- Directional collision projectile hit targets default to `Enemies`, but explicit `Allies` or `Any` can allow ally hits.
- Directional collision projectile stop policies should be extensible beyond max range/lifetime/endpoint, including hit count, kill count, and pierce count.
- Suggested projectile mode names are `TargetLocked` and `DirectionalCollision`.
- Suggested targetless cast option name is `allow_targetless_cast`.
- Suggested directional stop policy fields are `max_range_tiles`, `max_lifetime_ms`, `max_hits`, `max_kills`, and `max_pierces`.
- Authored tile range endpoint is the authoritative termination point; do not add separate invalid/out-of-battle truncation for this goal.

Result:

- No code was changed.
- `PLAN.md` was updated to include these implementation constraints.

### 2026-06-20 - Follow-up Plan Hardening

Status: reviewed.

Summary:

- Re-read the plan against concrete line-level code after the user requested follow-up corrections.
- Confirmed `src/game/range_preview.rs` owns fallback preview logic separately from combat targeting.
- Confirmed fallback preview is currently self tile plus one forward tile only, so it must be narrowed to the employee no-weapon fallback and not reused as the corroded employee range source.
- Confirmed basic attack projectile launch happens at `AttackResolve`, while current projectile impact is handled later by active projectile advancement.
- Confirmed current timeline/runtime projectile naming uses `ProjectileGuidance::{Homing, Fixed}` and `TimelineProjectileGuidance::{Homing, Fixed}`, which can conflict with the new `TargetLocked` / `DirectionalCollision` policy if both axes remain public and independent.
- Confirmed `docs/skill_target_contract.md` still contains older wording for `RetargetOnStep` using `range_units` and default basic attack fallback as only self plus one forward tile.

Plan changes:

- Added `src/game/range_preview.rs` to runtime source-of-truth audit.
- Added explicit guidance to reuse one effective basic attack tile range helper for combat eligibility and range previews.
- Added explicit `AttackResolve`/release-time launch eligibility wording for basic attack projectiles.
- Added a test case for target moving out of tile range before projectile launch.
- Added requirement to reconcile existing `ProjectileGuidance` / timeline guidance with the new projectile modes instead of introducing two disagreeing classifications.
- Added docs/probe update requirements for projectile mode and fallback wording.

Result:

- No code was changed.
- `PLAN.md` now better guards against reintroducing split range preview fallback, ambiguous projectile mode naming, or stale `skill_target_contract.md` text.

## Validation Results

### 2026-06-20 - Initial Code Refactor Checks

Status: passed after fixes.

Commands:

- `cargo check -p game_core`
- `cargo test -p game_core basic_attack_uses_tile_range_not_range_units`
- `cargo test -p game_core skill_enemy_single_uses_tile_range_not_range_units`
- `cargo test -p game_core advance_basic_attack_projectile_is_target_locked_not_path_collision`

Findings:

- The first `cargo check -p game_core` failed because one `RuntimeUnit` initializer missed the new `ranged_reposition_until_ms` runtime field and one `DeliveryDef::Projectile` pattern did not ignore the newly added projectile policy fields.
- Focused test builds initially failed because several test/fixture `DeliveryDef::Projectile` struct literals did not specify the new projectile policy fields.
- After fixing those initialization sites, `cargo check -p game_core` passed.
- `basic_attack_uses_tile_range_not_range_units` passed and proves large `range_units` does not allow a basic attack outside the authored tile range.
- `skill_enemy_single_uses_tile_range_not_range_units` passed and proves official `EnemySingle` skill target selection no longer falls back to continuous `range_units`.
- `advance_basic_attack_projectile_is_target_locked_not_path_collision` passed and proves a basic attack projectile hits only its original target rather than a bystander on the path.

Code changes verified by these commands:

- Basic attack projectiles now schedule one arrival event at expected impact time instead of 1ms path reevaluation ticks.
- Basic attack projectile arrival no longer uses continuous sweep collision.
- Basic attack projectile arrival checks only target-locked validity: attacker side still exists, target exists, target is alive, target is hostile to the original attacker side, and target passes non-range air targetability.
- Official `EnemySingle` skill cast/step target selection uses authored `defense_tile_range` instead of `range_units`.
- Removed the now-unused continuous `choose_enemy_target_in_range_units` helper and its old continuous-distance tests.

### 2026-06-20 - Projectile Policy Validation

Status: passed.

Commands:

- `cargo check -p game_core`
- `cargo test -p game_core skill_database_rejects_nondefault_homing_collision_contracts`

Findings:

- Added skill data validation that separates `TargetLocked` and `DirectionalCollision` projectile policies.
- `TargetLocked` projectile steps now reject targetless casting, directional stop policy fields, and custom continuous collision settings.
- `DirectionalCollision` projectile steps must declare at least one deterministic stop policy: authored max range, lifetime, kill count, pierce count, or collision max hits.
- Existing tests that represented piercing/collision skillshots were updated to explicitly use `DirectionalCollision` with `max_range_tiles`.

Result:

- `cargo check -p game_core` passed after the validation change.
- `skill_database_rejects_nondefault_homing_collision_contracts` passed, preserving the rule that target-locked/homing-style projectiles cannot silently use custom collision behavior.

### 2026-06-20 - Live RON Loading Smoke

Status: passed, but exposed a remaining policy decision.

Command:

- `cargo test -p game_core --test ron_loading`

Result:

- All 16 `ron_loading` tests passed.

Policy finding:

- `../game_resources/data/enemies/corroded_employees.ron` still has no corroded employee basic attack range preset pool and no `basic_attack_range_preset` selections.
- The code currently accepts this because validation only checks referenced presets when a profile declares one.
- `PLAN.md` requires live ordinary corroded employee profiles to select named range presets, but the exact live mapping for `corroded_marksman` and `corroded_medic` is not explicitly fixed in the plan.
- Current live data suggests:
  - melee profiles: `corroded_guard`, `corroded_rusher`, `corroded_bruiser`, `corroded_veteran`
  - ranged profiles: `corroded_marksman`, `corroded_medic`
- Need user confirmation before assigning `ranged_center_3x3` vs `ranged_center_5x5` to each ranged live profile, because that is a balance/data policy choice rather than a code-only repair.

### 2026-06-20 - Approved Corroded Employee Range Preset Migration

Status: passed.

User-approved live mapping:

- `corroded_guard`, `corroded_rusher`, `corroded_bruiser`, `corroded_veteran` -> `melee_front_1`
- `corroded_marksman` -> `ranged_center_5x5`
- `corroded_medic` -> `ranged_center_3x3`

Changes:

- Added `melee_front_1`, `ranged_center_3x3`, and `ranged_center_5x5` to `../game_resources/data/enemies/corroded_employees.ron`.
- Added `profile_role: normal` and `basic_attack_range_preset` to each live corroded employee profile.
- Strengthened `CorrodedEmployeeProfileDatabase::validate_indexes()` so `profile_role: Normal` requires `basic_attack_range_preset`.
- Added tests proving preset resolution and validation failure for normal profiles without a preset.

Commands:

- `cargo test -p game_core corroded_employee_database_resolves_basic_attack_range_presets`
- `cargo test -p game_core normal_corroded_employee_profile_requires_basic_attack_range_preset`
- `cargo test -p game_core --test ron_loading`

Result:

- All commands passed.

## Failed/Rejected Approaches

### 2026-06-20 - Test Fixture Compile Failure After Adding `range_policy`

Attempt:

- Added `range_policy` to skill/basic-attack runtime data and then ran focused WholeField tests.

Failure:

- `cargo test -p game_core whole_field_basic_attack_uses_valid_tiles_without_facing` initially failed at test compile time.
- Cause: many `cfg(test)` and integration-test `SkillStepDef` / `SkillCastTargetingDef::Explicit` literals did not include the new `range_policy` field.

Fix:

- Added `range_policy: Default::default()` to existing explicit test fixtures so they preserve the old `pattern` behavior.
- Updated explicit cast-target tuple expectations to include the default range policy.

Revalidation:

- The same focused command passed after fixture updates.

Result:

- Successful. Keep this as a reminder that `cargo check` does not compile all test-only fixtures.

## Successful WholeFieldValidTiles Implementation Check

Changes:

- Added `TileRangePolicy::{Pattern, WholeFieldValidTiles}`.
- Added `range_policy` to basic attacks, skill cast targeting, skill steps, and Unity-facing skill catalog DTOs.
- Kept existing authored pattern behavior as the default.
- Implemented whole-field valid-tile range membership without requiring facing.
- Added `Battlefield::valid_positions()` so whole-field preview uses the same non-rectangular valid-tile source as battlefield bounds.
- Rejected `WholeFieldValidTiles` plus `defense_tile_range` as a source-of-truth conflict.
- Rejected `WholeFieldValidTiles` on `profile_role: Normal` corroded employee profiles.

Commands:

- `cargo check -p game_core`
- `cargo test -p game_core whole_field_basic_attack_uses_valid_tiles_without_facing`
- `cargo test -p game_core whole_field_basic_attack_rejects_void_tiles`
- `cargo test -p game_core skill_enemy_single_whole_field_uses_valid_tiles_without_facing`
- `cargo test -p game_core skill_database_rejects_whole_field_with_authored_tile_pattern`
- `cargo test -p game_core normal_corroded_employee_profile_rejects_whole_field_basic_attack`
- `cargo test -p game_core basic_attack_uses_tile_range_not_range_units`
- `cargo test -p game_core skill_enemy_single_uses_tile_range_not_range_units`
- `cargo test -p game_core advance_basic_attack_projectile_is_target_locked_not_path_collision`
- `cargo test -p game_core --test ron_loading`

Result:

- All commands passed.

## Full Regression After WholeFieldValidTiles and Fixture Updates

Attempt:

- Ran `cargo test -p game_core` after the WholeField implementation and focused test passes.

Initial failure:

- 11 lib tests failed because older test fixtures still assumed one of these legacy shapes:
  - enemies without facing could still attack through `range_units`,
  - a player unit with no authored pattern had no fallback range,
  - triggered item proc skills could reuse a current target without explicit tile policy,
  - test-created corroded employee profiles could skip preset resolution.
- One integration test later failed because a miss-oriented collision projectile fixture still used `ProjectileHitPolicy::TargetLocked` while customizing collision hit filters.

Fix:

- Updated planner test fixtures to provide the expected route-facing direction.
- Updated the player basic attack fallback test to match the current default range policy.
- Marked current-target item proc fixture skills as explicit `WholeFieldValidTiles`.
- Updated the combat preview corroded employee test database to include and resolve a range preset.
- Updated the miss-oriented projectile fixture to use `DirectionalCollision` plus an explicit stop range.

Revalidation:

- `cargo test -p game_core generated_corroded_wave_source_resolves_during_preview_generation`
- `cargo test -p game_core airborne_enemy_prioritizes_protected_target_in_attack_range`
- `cargo test -p game_core fixed_defense_player_basic_attack_uses_default_pattern_but_requires_facing`
- `cargo test -p game_core targeted_execute_drops_locked_target_when_target_leaves_range`
- `cargo test -p game_core --test skill_refactor_validation untargeted_projectile_miss_finalizes_step_and_cleans_up_damage_gated_followup`
- `cargo test -p game_core`
- `cargo check -p game_core`

Result:

- All revalidation commands passed.

## Blocking / Policy Findings

## 2026-06-20 - Directional Collision Runtime Completion Audit

Hypothesis:

- The earlier schema changes for `ProjectileHitPolicy`, `allow_targetless_cast`, and directional stop fields may not be enough. The runtime launch path must actually consume those fields.

Trial:

- Audited `src/game/battle/core/skill_runtime/projectile.rs`, `src/game/battle/core/sim.rs`, and `src/game/battle/core/types.rs`.
- Found that `DeliveryDef::Projectile` fields were present in data/validation but launch still received only `speed_units_per_ms` and `collision`.
- Implemented explicit runtime branching:
  - `TargetLocked` requires a unit target and uses homing/locked impact against the original target.
  - `DirectionalCollision` uses caster facing, supports explicit targetless casting, and derives fixed projectile endpoint from `max_range_tiles` / `max_lifetime_ms`.
  - `max_pierces` is mapped into an effective max hit limit.
  - `max_kills` is tracked after projectile impact damage resolution and stops the active projectile when reached.
- Added automatic-cast usefulness support for explicitly targetless directional collision projectiles.

Failures:

- First `cargo test -p game_core --test skill_refactor_validation` failed because the new targetless test expected an ability cast without enqueueing an auto-cast start.
- The same run exposed an existing directional projectile fixture with no caster facing; since the new policy requires caster facing, that fixture dropped its follow-up step.

Fixes:

- Updated directional projectile fixtures to set caster facing explicitly.
- Added a targetless directional projectile test that enqueues `AutoCastStart`, confirms `AbilityCast.target_instance_id == None`, confirms `SkillProjectileLaunched.guidance == Fixed`, and confirms damage along caster facing.
- Added a kill-limit regression test that confirms a piercing directional projectile with `max_kills = 1` stops after the first killed unit.
- Updated automatic skill target usefulness so `DirectionalCollision + allow_targetless_cast` can start without a unit target when the caster has facing and an endpoint stop policy.

Revalidation:

- `cargo check -p game_core` passed before the runtime test additions.
- `cargo test -p game_core --test skill_refactor_validation` passed after the runtime wiring fixes.
- `cargo test -p game_core --test skill_refactor_validation` passed again after adding the `max_kills` stop-policy regression.
- `cargo check -p game_core` passed after the final runtime/test updates.
- `cargo test -p game_core` passed after the final runtime/test updates.

Result:

- Directional collision projectile mode is now runtime-wired instead of schema-only.
- Targetless directional collision is explicitly tested.
- Existing projectile follow-up and piercing tests still pass under the stricter facing/source policy.

### 2026-06-20 - Non-player Facing Source Is Not Defined

Status: resolved by user policy.

Evidence:

- `src/game/battle/core/build.rs` initializes spawned scenario units with `facing_direction: Some(Right)` only for player-side groups.
- The same spawn path leaves opponent units with `facing_direction: None`.
- `src/game/battle/core/build.rs::deploy_player_unit` later writes the user-selected facing for deployed player units.
- `src/game/battle/core/targeting.rs::is_target_in_defense_tile_range` returns false if the attacker has no facing.

Why this matters:

- `PLAN.md` requires basic attack target eligibility to be tile-based for all units.
- Tile patterns need a facing direction unless a separate non-facing policy is defined.
- Current code does not define the source of truth for opponent/facility/auto-spawned unit facing.
- Choosing a default such as route direction, target direction, spawn direction, or static `Down` would be a gameplay policy not currently fixed by this plan.

Result:

- User approved movement direction as the facing source.
- Route-following non-player units should update facing from movement direction and keep the last movement facing while blocked/engaged.
- User later revised route-less policy into explicit `WholeFieldValidTiles` range policy, and then decoupled that policy from route-less movement.
- `WholeFieldValidTiles` is shared by basic attacks and skills.
- Route-following units may use `WholeFieldValidTiles` only when their unit/profile/skill explicitly allows it.
- Bosses, special corroded employees, and special units such as snipers may use this policy.
- Regular `profile_role: Normal` corroded employees must not use `WholeFieldValidTiles`.
- Whole-field range only widens candidate tiles; actual target choice still follows alive/hostile/air_capable/untargetable/usefulness validation and targeting profile/rule ordering.
- Skills using `WholeFieldValidTiles` must declare explicit targeting rules; otherwise they are invalid data.
- Enemy attack range previews should not be sent in routine Unity-facing DTOs. Visual warnings/telegraphs should send actual affected tiles only.
- `WholeFieldValidTiles` + `TileArea` is allowed for explicit whole-field area skills, but validation must fail if targeting/effect policy is omitted.
- Current code audit found no existing corroded employee profile role field on `CorrodedEmployeeProfileMetadata`; add explicit `profile_role` metadata rather than reusing wave preset `role_mix`.
- Proposed initial `profile_role` values: `Normal`, `Special`, `LegacyEcho`.
- User also confirmed elite, easter-egg employee, boss, and other special units have independent skill fragments and attack ranges. Ordinary fallback must not overwrite their authored ranges.

### 2026-06-20 - Corroded Employee Range Policy Clarification

Status: resolved by user policy.

Summary:

- Ordinary corroded employees do have basic attack ranges.
- Their basic attacks use tile/facing eligibility and then existing targeting profile ordering among eligible candidates.
- Corroded employee basic attack range is selected by the corroded employee profile from a RON basic attack range preset pool.
- Initial corroded employee presets are `melee_front_1`, `ranged_center_3x3`, and `ranged_center_5x5`.
- `melee_front_1` is self tile plus exactly one facing-forward tile.
- `ranged_center_3x3` is a 3x3 square centered on the unit's current tile and does not use facing.
- `ranged_center_5x5` is a 5x5 square centered on the unit's current tile and does not use facing.
- Ranged center presets intersect with battlefield valid tiles but do not exclude obstacle/blocked tiles from attack candidates.
- Elite and boss units have mana bars and keep their independently authored skills/ranges.
- Route-following ranged enemies are forced-forward attackers, not stationary turrets: attack, then attempt `ranged_reposition_ms` route movement, then revalidate the previous target before searching for a new target.
- Default `ranged_reposition_ms` is 1000ms and can be overridden by profile/RON.
- Blocked ranged enemies do not reposition and prioritize their blocker.

Result:

- `PLAN.md` and notes were corrected from the previous ranged fallback wording to RON profile-selected range presets.
