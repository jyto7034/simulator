# Tile Based Attack Delivery Contract Plan

## Objective

Make official combat attack judgment tile-based while keeping explicit projectile/collision mechanics for skills that need them.

The long-term policy is:

- Basic attack target eligibility is tile-based only.
- Official single-target skill cast/retarget eligibility is tile-based unless the skill delivery explicitly declares continuous collision.
- Non-player spawned units use movement direction as the basic attack facing source. Route-following enemies update facing from the current movement direction; when blocked or engaged, they keep their last movement facing.
- `WholeFieldValidTiles` is a shared range policy usable by both basic attacks and skills. It is not tied to route-less movement. Route-following units may use it only when their unit/profile/skill explicitly allows the policy. Bosses, special corroded employees, and special units such as snipers may use it. Regular `profile_role: Normal` corroded employees must not use it. This only widens candidate tiles; actual target choice still follows validation and targeting rules.
- Ordinary corroded employees have tile-based basic attack ranges and targeting profiles authored in RON. A corroded employee profile selects its basic attack range from a range preset pool; melee/ranged behavior is data-driven rather than inferred from `range_units`.
- Corroded employee profiles use explicit `profile_role` metadata. `Normal` is for regular wave enemies and cannot use `WholeFieldValidTiles`; `Special` and `LegacyEcho` can have authored special ranges, targeting, and skills.
- Route-following ranged enemies are forced-route attackers, not stationary turrets. They stop to attack, then move for an authored reposition window, then revalidate the previous target before searching again. Blockers have absolute priority during block engagement, even when a protected objective is inside the ranged attack tiles.
- Basic attack cadence remains data-authored per unit/profile/weapon. `interval_ms`, `windup_ms`, and ranged reposition timing must stay independently configurable so fast and slow attackers can coexist without special-case code.
- `DeliveryDef::Instant` is the hitscan/immediate application delivery.
- `DeliveryDef::Projectile` is split by explicit projectile hit policy:
  - target-locked projectile: target is chosen by tile rules, never retargets to bystanders, and uses only original-target impact validity. For basic attacks, `docs/goals/basic_attack_projectile_target_only_collision/PLAN.md` further refines this into continuous sweep against the original locked target body only.
  - directional collision projectile: skill-only exception that is fired toward a tile/direction without a required unit target, travels forward within its authored range/lifetime, and uses continuous sweep/collision, piercing, radius, and max hit rules.
- `DeliveryDef::TileArea` remains tile-based area delivery.
- `range_units` is not deleted. It remains available for future non-attack mechanics such as contagion, contact, aura, proximity, and special continuous-space skill mechanics. It must not be the source of truth for official basic attack eligibility.

Do not implement a short-term patch for the currently observed early-hit symptom only. This goal should cleanly separate target eligibility, delivery, and impact resolution so future skills can use the correct model without hidden fallbacks.

## Source Of Truth Order

Use this order while implementing:

1. Runtime battle code:
   - `src/game/battle/core/targeting.rs`
   - `src/game/battle/core/basic_attack.rs`
   - `src/game/battle/core/commands.rs`
   - `src/game/battle/core/sim.rs`
   - `src/game/battle/core/skill_runtime/cast.rs`
   - `src/game/battle/core/skill_runtime/projectile.rs`
   - `src/game/battle/core/skill_runtime/area.rs`
   - `src/game/battle/core/movement/planner.rs`
   - `src/game/battle/core/movement/engine.rs`
   - `src/game/range_preview.rs`
   - `src/game/ability.rs`
   - `src/game/data/skill_data.rs`
   - `src/game/data/equipment_data.rs`
2. Live RON/data:
   - weapons/equipment data that defines basic attack delivery and tile ranges.
   - abnormality and corroded-employee basic attack data that may currently define `range_units` without a RON-authored tile range or range preset.
   - skills data that defines `DeliveryDef::{Instant, Projectile, TileArea}`.
   - any skill currently relying on `SkillProjectileCollisionDef`.
3. Unity-facing snapshot/command/timeline DTOs and battle records.
4. Latest policy docs:
   - `docs/skill_target_contract.md`
   - `docs/game_rulebook.md`

Do not trust older docs over runtime code and live data. If runtime data reveals that a live skill needs a policy not covered here, stop and ask the user before deciding.

## Current Evidence

Current runtime has mixed judgment models:

- `BattleCore::is_basic_attack_target_in_range()` uses `defense_tile_range` only for fixed-defense player units, but falls back to `UnitBody::can_reach(range_units)` for other cases.
- `BattleCore::airborne_enemy_basic_attack_target_in_range()` still depends on the same basic attack policy and continuous distance sorting.
- `BattleCore::choose_enemy_target_in_range_units()` is continuous-distance based and is used by non-fixed-defense skill target selection.
- `DeliveryDef::Projectile` already exists and is used by both basic attacks and skills.
- `SkillProjectileCollisionDef` already exists and supports collision radius, hit filter, piercing, despawn behavior, and max hits.
- Skill projectile runtime already has continuous sweep collision code.
- Skill `TileArea` runtime already resolves affected tiles by tile membership.
- Basic attack projectile runtime currently behaves like a physical projectile with continuous collision. Under the new policy, basic attack projectile should not use path collision to discover hits. It should be target-locked after tile eligibility succeeds.
- `DeliveryDef::Projectile` currently maps to runtime/timeline `ProjectileGuidance::{Homing, Fixed}`. The new `TargetLocked` / `DirectionalCollision` policy must replace or clearly map this axis rather than layering a second ambiguous projectile classification on top.
- `src/game/battle/core/sim.rs` has additional target validation and delivery execution paths. It must be refactored together with the live runtime paths so tests and simulation do not preserve the old continuous eligibility model.
- `src/game/range_preview.rs` still has a fixed fallback of self tile plus one forward tile. This must be narrowed to the employee no-weapon fallback and not reused as a corroded employee range source; deployed and pending range previews must match combat eligibility.
- Movement planning and continuous movement input still use `basic_attack.range_units` as approach/steering distance. That is not the same as attack eligibility and must not be removed accidentally during this goal.
- Some live enemy/basic attack RON data may use `range_units > 1` or `DeliveryDef::Projectile` without an authored tile range. These entries should be migrated by policy instead of inferred ad hoc: ordinary corroded employee profiles must reference an explicit range preset from the corroded employee basic attack range pool, while boss/abnormality/special long-range attacks must receive an explicit authored `defense_tile_range`.
- The user approved movement direction as the facing source for non-player route-following tile-based basic attacks. Special units can use explicit `WholeFieldValidTiles` range policy whether or not they have a route. Elite, easter-egg employee, boss, and other special units must keep authored attack/skill ranges and targeting rules; ordinary fallback must not flatten their unique skill fragment or attack range identity.

Current docs already point toward tile-based official DefenseRoute ranges, but still mention `range_units` in some skill retargeting text. This goal must reconcile code and docs rather than preserving contradictory behavior.

## Target Policy

### Eligibility

Attack eligibility means "may this attack or skill choose this target at this battle time?"

- Basic attack eligibility is tile-based for all units in official combat.
- A target is eligible for a basic attack only if:
  - target is alive and hostile,
  - target satisfies air capability,
  - attacker has a valid facing where required, or the attack/skill explicitly uses `WholeFieldValidTiles`,
  - attacker and target sampled positions project to tiles,
  - target tile is inside the attacker's final basic attack tile range.
- Player deployed units use deployment facing. Non-player route units use their last movement direction as facing. Attacks/skills using `WholeFieldValidTiles` do not require facing for range projection.
- `WholeFieldValidTiles` can be used by both basic attacks and skills, but it only defines candidate tiles. Basic attacks still use their unit/basic-attack targeting profile, while skills must define an explicit targeting rule. A `WholeFieldValidTiles` skill without an explicit targeting rule is invalid data.
- `WholeFieldValidTiles` plus `TileArea` is allowed for explicitly authored whole-field area skills. This must be explicit in skill data and validation must fail if targeting/effect policy is omitted.
- Airborne targets are not an exception. They must still be inside the attack tile range.
- `range_units`, hitbox radius, center distance, and continuous body reach are not valid basic attack eligibility sources.
- Official single-target skill eligibility follows the same tile-range principle unless the skill explicitly uses a continuous collision projectile delivery.
- `TileArea` uses final affected tiles as the hit source of truth.

### Enemy Ranged Route Attack Cadence

Route-following ranged enemies use this priority order:

1. If currently blocked, attack the blocker when the blocker is a valid target.
2. Blocker priority wins over the previous ranged target, normal target-search candidates, route movement, and protected/defense objectives, even if those objectives are inside the ranged attack tiles.
3. If not blocked and inside `ranged_reposition_ms`, continue route movement and do not search for a new ranged target unless a stronger interrupt state occurs.
4. After reposition ends, revalidate the previous target first.
5. Reuse the previous target when it is still present, alive, targetable, useful by hostile target validation, and inside the attack tiles.
6. If the previous target is invalid, run normal target selection using the unit's authored targeting profile/rule ordering.
7. Protected/defense objectives are target candidates only through the same authored targeting/fallback policy; they do not override blocker priority.

Attack cadence is authored:

- `interval_ms` controls how often the unit can start its next basic attack.
- `windup_ms` controls release timing.
- `ranged_reposition_ms` controls post-release route movement before previous-target revalidation.
- These fields are independent. Windup time does not count as reposition time, and reposition time must not silently replace attack interval.
- `interval_ms` must remain per-profile/per-weapon/per-unit configurable and greater than zero.
- `ranged_reposition_ms` may be zero to mean immediate previous-target revalidation after release.

### Delivery And Impact

Delivery answers "how does a valid attack apply its effects?"

- `DeliveryDef::Instant`
  - Hitscan/immediate delivery.
  - After eligibility succeeds, effects are applied immediately at the event's battle time.

- `DeliveryDef::Projectile` target-locked mode
  - Suggested schema mode name: `TargetLocked`.
  - The target is chosen by tile eligibility.
  - A projectile timeline is emitted for presentation.
  - Damage/effects apply only to the original target if it is still alive, still hostile to the original attacker side, still present in the battle, and still passes non-range validity filters such as air capability and untargetable/dead checks.
  - For basic attacks, impact timing is refined by `docs/goals/basic_attack_projectile_target_only_collision/PLAN.md`: `expected_impact_time_ms` is an initial presentation estimate, and gameplay impact is the first continuous sweep contact against the original locked target body within the projectile's finite travel window.
  - For basic attacks, tile eligibility is checked when the attack resolves/releases and the projectile is launched. The later projectile arrival uses only target-locked impact validity and must not perform another range check.
  - Do not re-check attack range at arrival.
  - Do not re-check target usefulness at arrival.
  - Do not collide with different units along the path.

- `DeliveryDef::Projectile` collision mode
  - Suggested schema mode name: `DirectionalCollision`.
  - Skill-only exception.
  - Represents a directional projectile fired in the caster's current facing direction rather than locked to a unit target.
  - User input does not choose the direction at cast time. Unit facing, normally chosen at deployment time, is the direction source of truth.
  - Does not require an initial unit target.
  - May be cast without a currently valid unit target only when the skill explicitly allows targetless single-shot casting.
  - Suggested targetless option name: `allow_targetless_cast`.
  - Travels within its authored tile range, lifetime, endpoint, pierce-count, kill-count, hit-count, or equivalent stop policy.
  - Suggested stop policy field names:
    - `max_range_tiles`
    - `max_lifetime_ms`
    - `max_hits`
    - `max_kills`
    - `max_pierces`
  - `max_range_tiles` / authored range tiles define the authoritative forward path and endpoint. The projectile stops at the end of that authored range tile path.
  - If multiple stop policies are present, the projectile ends when the first stop policy is satisfied.
  - Uses continuous sweep/collision during flight.
  - May use collision radius, hit target filter, piercing, max hits, max kills, max pierces, and despawn-on-hit.
  - Default hit target filter is `Enemies`.
  - Allies can be hit only when `hit_targets` explicitly allows `Allies` or `Any`.
  - This mode is not allowed for basic attacks unless the user approves a new policy.

- `DeliveryDef::TileArea`
  - Skill area delivery only.
  - Uses `defense_tile_range`/final affected tile cells to decide hit targets.
  - Does not use continuous geometric AoE for official DefenseRoute skills.

### `range_units`

`range_units` remains in the data model only where it has a legitimate non-basic-attack purpose.

Allowed long-term uses:

- contagion/proximity/contact mechanics,
- aura or proximity checks,
- special skill delivery explicitly declared as continuous-space,
- internal movement steering if the policy requires approach distance,
- legacy data while it is being migrated, only if PLAN records why and how it will be removed from attack eligibility.

Movement note:

- `range_units` may remain as an approach/stop-distance hint for continuous movement until a separate movement policy replaces it.
- Keeping `range_units` for movement does not permit using it for basic attack eligibility, target selection, range previews, or official DefenseRoute skill range checks.

Disallowed uses:

- basic attack eligibility,
- official basic attack target selection,
- airborne basic attack exception targeting,
- default official DefenseRoute skill range,
- Unity range preview reconstruction.

## Implementation Plan

1. Audit live data and runtime call sites.
   - List every use of `basic_attack.range_units`.
   - List every use of `SkillStepDef.range_units`.
   - List every use of `UnitBody::can_reach()` in battle combat code.
   - Include both live runtime and simulation paths, especially `src/game/battle/core/sim.rs`.
   - Include movement/planner call sites, but classify them separately from attack eligibility.
   - Classify each call site as basic attack, skill eligibility, skill collision, movement, or future/special mechanic.
   - Audit live RON entries where basic attacks have `range_units > 1`, `DeliveryDef::Projectile`, or missing `defense_tile_range`.
   - Record findings in `EXPERIMENTS.md`.

2. Define the domain model before code changes.
   - Reuse `DeliveryDef::Instant` for hitscan.
   - Keep `DeliveryDef::TileArea` for tile area skill delivery.
   - Decide the smallest clean representation for projectile mode:
     - preferred direction: make `DeliveryDef::Projectile` explicitly distinguish `TargetLocked` vs `DirectionalCollision`;
     - do not infer collision mode merely because `collision` has non-default fields unless live data migration proves this is unavoidable.
   - Audit existing `ProjectileGuidance::{Homing, Fixed}` and `TimelineProjectileGuidance::{Homing, Fixed}` before adding fields:
     - `TargetLocked` may replace the current basic/homing projectile impact semantics, but must not imply continuous collision;
     - `DirectionalCollision` may reuse fixed projectile motion internally, but must be explicitly named at the data/runtime/timeline boundary where Unity or validation needs to distinguish it.
   - Do not leave two independent public concepts that can disagree, such as `guidance: Homing` plus `mode: DirectionalCollision`.
   - Prefer explicit field names for directional projectiles:
     - `allow_targetless_cast`
     - `max_range_tiles`
     - `max_lifetime_ms`
     - `max_hits`
     - `max_kills`
     - `max_pierces`
   - If this requires a live RON schema migration, record the migration plan in `PLAN.md` before editing data.

3. Refactor basic attack tile range source.
   - Ensure every basic attack profile has an effective tile range:
     - authored weapon/basic attack `defense_tile_range`,
     - corroded employee profile basic attack range preset resolved to `TileRangePattern`,
     - otherwise employee/test fallback based on explicit policy:
       - no weapon/no skill employee fallback: self tile plus one tile forward.
   - Migrate live RON basic attack data by this policy:
     - existing authored `defense_tile_range` stays authoritative,
     - live corroded employee profiles must select a named range preset from the basic attack range pool,
     - initial corroded employee range presets are `melee_front_1`, `ranged_center_3x3`, and `ranged_center_5x5`,
     - `melee_front_1` means self tile plus one facing-forward tile,
     - `ranged_center_3x3` means a 3x3 square centered on the unit's current tile and does not use facing,
     - `ranged_center_5x5` means a 5x5 square centered on the unit's current tile and does not use facing,
     - ranged center presets use only battlefield valid tiles and do not remove obstacle/blocked tiles from attack candidates,
     - the basic attack range preset pool is a shared authoring pool; corroded employee profiles reference entries from it,
     - live ranged basic attacks identified only by `range_units > 1` or projectile delivery should be made explicit as a profile-selected range preset,
     - boss, abnormality, elite, or special attacks that intentionally need nonstandard range must receive a specific authored `defense_tile_range`.
   - Treat fallback as the default for genuinely unspecified employee/test attacks, not as a silent replacement for live corroded employee or special long-range balance.
   - Remove basic attack eligibility dependence on `range_units`.
   - Apply this to player units, enemy units, airborne units, blocked target checks, persisted target checks, and retargeting.
   - Update route-following non-player facing from movement direction and keep the last movement facing while blocked/engaged.
   - Add explicit `WholeFieldValidTiles` range policy for special units/attacks/skills. It should be available to both basic attacks and skills and must not be tied to route-less movement.
   - Attacks/skills using `WholeFieldValidTiles` should not fail range projection merely because the unit lacks facing. Candidate range is all battlefield valid tiles, followed by normal target validation and targeting profile/rule ordering.
   - Require skills using `WholeFieldValidTiles` to declare explicit targeting rules.
   - Add `profile_role` metadata to corroded employee profiles:
     - `Normal`: regular wave profile; `WholeFieldValidTiles` is invalid.
     - `Special`: special or sniper-like profile; `WholeFieldValidTiles` may be valid if authored, regardless of route presence.
     - `LegacyEcho`: dead employee echo/easter-egg elite profile; authored special ranges/skills are allowed.
   - Treat `WholeFieldValidTiles` on `Normal` corroded employee profiles as invalid data.
   - Do not use ordinary corroded employee presets or employee fallback to replace authored elite, easter-egg employee, boss, abnormality, or special-unit attack ranges. If such units lack intended authored range data, stop and ask instead of collapsing them to default range.
   - Preserve targeting profile ordering after candidate filtering.
   - Implement enemy ranged route attack cadence:
     - route-following ranged enemies move along their route by default,
     - when a valid target is found, stop movement and perform the attack,
     - after attack release/damage/projectile spawn, start `ranged_reposition_ms`; windup time does not count toward reposition,
     - attempt route movement for explicit `ranged_reposition_ms`, defaulting to 1000ms,
     - `ranged_reposition_ms` is profile/RON-overridable, must be >= 0, and 0 means immediate previous-target revalidation,
     - keep `interval_ms`, `windup_ms`, and `ranged_reposition_ms` as separate authored timing controls; do not replace attack interval with reposition timing,
     - keep `interval_ms` per-profile/per-weapon/per-unit configurable and validate it remains greater than zero,
     - do not search for new ranged targets during reposition unless a stronger interrupt state occurs,
     - after reposition, revalidate the previous target before searching for a new target,
     - if the previous target is alive, targetable, still inside attack tiles, and useful by hostile target validation, attack it again without running a fresh best-target search,
     - if the previous target is invalid, run normal targeting,
     - blocked enemies do not reposition; their blocker takes priority over previous ranged targets, normal target-search candidates, defense/protection objectives, and route movement,
     - blocker priority applies even when a protected/defense objective is inside the ranged attack tiles,
     - protected/defense objectives are target candidates only through the same authored targeting/fallback policy after blocker and previous-target rules are resolved,
     - reposition can end early on block, death, route end, battle end, or stronger movement locks.
   - Basic attack tile eligibility is sampled from core runtime tiles, not Unity presentation position:
     - `AttackStart` checks whether an attack can begin,
     - `AttackResolve`/release-time eligibility is the authoritative final check for damage application or projectile spawn.
   - Refactor `src/game/range_preview.rs` to use the same effective basic attack tile range helper as combat eligibility. Pending placement, deployed checkpoint previews, and combat targeting must not each own separate fallback logic.

4. Refactor basic attack projectile impact.
   - Basic attack projectile must be target-locked.
   - Launch is allowed only after tile eligibility succeeds.
   - For windup-based basic attacks, the authoritative launch eligibility check is the `AttackResolve`/release-time tile check. If the target moved out of tile range before release, the attack should fail/miss before projectile launch.
   - Arrival applies effects to the original target if it is still alive, still hostile to the original attacker side, still present in the battle, and still targetable by non-range filters.
   - Arrival must not re-run range or usefulness checks.
   - Arrival does not use path collision to discover a different hit target.
   - Superseding refinement: `docs/goals/basic_attack_projectile_target_only_collision/PLAN.md` allows continuous sweep only against the original locked target body, while preserving all tile eligibility and no-bystander rules from this completed goal.
   - Arrival does not hit bystanders along the path.
   - Timeline should still include projectile launch/impact events for Unity presentation.

5. Refactor skill eligibility and projectile modes.
   - Official skill `EnemySingle` and `RetargetOnStep` eligibility should use tile range when authored as official DefenseRoute target selection.
   - Directional collision projectile skills must explicitly opt into continuous collision.
   - Directional collision projectiles use the caster's current facing as their direction source.
   - Directional collision projectiles may be launched without a unit target only when the skill explicitly permits targetless single-shot casting.
   - Use `allow_targetless_cast` as the preferred explicit option name.
   - Directional collision projectiles travel forward within authored tile range/lifetime/endpoint/hit-count/kill-count/pierce-count limits and use continuous collision only for hit detection during flight.
   - Directional collision projectiles stop at the end of their authored forward tile range path. Do not add separate invalid-tile boundary truncation unless a later policy explicitly needs it.
   - Directional collision projectile hit filtering defaults to `Enemies`; ally hits require explicit `Allies` or `Any`.
   - Target-locked projectile skills should not use continuous collision.
   - `TileArea` remains tile-based.
   - Keep `SkillProjectileCollisionDef` only for directional collision projectile mode.

6. Update validation.
   - Reject basic attacks that try to use `TileArea`.
   - Reject basic attacks that opt into collision projectile mode unless a later policy allows it.
   - Require tile range or fallback availability for basic attacks.
   - Require explicit directional collision projectile mode for skills using piercing/max_hits/collision radius behavior.
   - Require directional projectile skills to declare enough travel policy to stop deterministically: authored tile range, lifetime, endpoint, hit count, kill count, pierce count, or equivalent data.
   - Require targetless directional projectile skills to explicitly opt into targetless casting.
   - Ensure official DefenseRoute skills that need `EnemySingle`, `RetargetOnStep`, or `TileArea` have `defense_tile_range`.

7. Update Unity-facing DTO/contracts only if the shape changes.
   - Timeline delivery names must remain clear enough for Unity to animate:
     - instant/hitscan,
     - target-locked projectile,
     - directional collision projectile,
     - tile area.
   - Range preview DTO remains final cells; Unity must not reconstruct ranges from RON fields.
   - If new enum values are exposed, update contract docs and probes.
   - If projectile mode is exposed in timeline events, update both basic attack projectile and skill projectile events so Unity never has to infer target-locked vs collision behavior from `guidance`, speed, collision radius, or event names.
   - Do not add enemy attack range previews to routine checkpoint/update DTOs. Enemy ranges are not player-operated previews.
   - If an enemy attack, boss pattern, or skill needs visual warning, send only the actual affected/telegraphed tiles for that event rather than all candidate cells.
   - Update `docs/skill_target_contract.md` and `docs/game_rulebook.md` if implementation changes default fallback wording, `RetargetOnStep` source-of-truth wording, or projectile delivery terminology.

8. Update tests.
   - Replace tests that assert basic attack `range_units` behavior with tile-based equivalents.
   - Add tests that prove high `range_units` does not allow a basic attack outside tile range.
   - Add airborne target tests proving air-capable still requires tile range.
   - Add target-locked projectile tests proving bystanders are not hit by path collision.
   - Add directional collision projectile skill tests proving piercing/max_hits/radius behavior still works.
   - Add directional collision projectile skill tests proving a projectile can be fired toward a tile/direction without a unit target and stops at its authored limit.
   - Add live RON validation tests for projectile mode and tile range requirements.

9. Run validation in small steps.
   - Run focused tests after each refactor.
   - Run `cargo check` after schema/type changes.
   - Run wider battle/skill/data validation tests at the end.

## Test Plan

Focused tests:

- `basic_attack_uses_tile_range_not_range_units`
  - Give a unit very large `range_units`.
  - Place target outside final tile range.
  - Assert no target is selected and no attack starts.

- `employee_basic_attack_fallback_is_front_1_only`
  - Build an employee/test basic attack profile with no authored tile range.
  - Assert fallback allows self tile plus one forward tile.
  - Assert side/back/outside targets are not eligible.
  - Assert `range_previews.basic_attack.cells` uses the same fallback cells as combat targeting.

- `corroded_employee_profile_range_preset_is_authoritative`
  - Build corroded employee profiles that reference `melee_front_1`, `ranged_center_3x3`, and `ranged_center_5x5`.
  - Assert `melee_front_1` allows self tile plus one forward tile.
  - Assert `ranged_center_3x3` allows a facing-independent 3x3 square centered on the unit's current tile.
  - Assert `ranged_center_5x5` allows a facing-independent 5x5 square centered on the unit's current tile.
  - Assert ranged center presets exclude invalid/void/out-of-battle tiles but do not exclude obstacle/blocked tiles.
  - Assert `range_units` or projectile delivery alone does not change the tile candidates.

- `ranged_enemy_repositions_before_reusing_target`
  - Spawn a route-following ranged enemy and an eligible target inside its centered range.
  - Assert the enemy stops to attack.
  - Assert the enemy attempts route movement for its `ranged_reposition_ms` before the next attack cycle.
  - Assert the previous target is reused after reposition when it remains alive, targetable, useful, and inside attack tiles.

- `blocked_ranged_enemy_prioritizes_blocker`
  - Block a route-following ranged enemy while it has a previous ranged target.
  - Assert the enemy does not perform reposition movement while blocked.
  - Assert the blocker is prioritized over the previous ranged target.

- `blocked_ranged_enemy_prioritizes_blocker_over_protected_object`
  - Put a route-following ranged enemy in range of both a blocker and the protected/defense objective.
  - Assert the enemy attacks the blocker.
  - Assert the protected/defense objective is not selected until the blocker no longer blocks or is no longer valid.

- `ranged_enemy_attack_timing_keeps_interval_and_reposition_separate`
  - Give two ranged enemy profiles different `interval_ms` values and the same `ranged_reposition_ms`.
  - Assert each unit's next attack respects its authored `interval_ms`.
  - Assert post-release route movement still uses `ranged_reposition_ms` and does not overwrite attack interval.

- `airborne_basic_attack_target_still_requires_tile_range`
  - Air-capable attacker.
  - Airborne target outside tile range.
  - Assert no attack.
  - Move target into tile range.
  - Assert attack can start.

- `basic_projectile_is_target_locked_not_path_collision`
  - Valid tile target is selected.
  - Put another hostile unit along projectile path.
  - Assert only original target can be impacted.

- `basic_projectile_does_not_recheck_range_at_arrival`
  - Target is valid at launch.
  - Target moves outside range before arrival.
  - Assert impact can still apply if target remains alive and targetable.

- `basic_projectile_does_not_launch_if_target_leaves_tile_range_before_release`
  - Target is valid when windup starts.
  - Target moves outside tile range before `AttackResolve`.
  - Assert projectile is not launched and no damage is applied.

- `basic_projectile_misses_dead_or_untargetable_target`
  - Target dies or becomes invalid before arrival.
  - Assert projectile records miss/no damage.

- `skill_directional_collision_projectile_uses_continuous_sweep`
  - Explicit directional collision projectile skill.
  - Moving projectile intersects a target between ticks.
  - Assert impact occurs.

- `skill_directional_collision_projectile_supports_piercing_and_max_hits`
  - Explicit directional collision projectile skill.
  - Multiple targets along path.
  - Assert piercing/max_hits policy is honored.

- `skill_directional_collision_projectile_can_launch_without_unit_target`
  - Explicit directional collision projectile skill.
  - Skill is fired in the caster's current facing direction rather than at a unit.
  - Assert projectile travels forward and stops at its authored range/lifetime/endpoint.

- `skill_directional_collision_projectile_stops_at_range_tile_endpoint`
  - Explicit directional collision projectile skill.
  - Projectile travels in the caster's facing direction.
  - Assert core projectile terminates at the end of the authored forward tile range path.

- `skill_directional_collision_projectile_hit_targets_default_to_enemies`
  - Explicit directional collision projectile skill without hit target override.
  - Ally and enemy are both in the path.
  - Assert only enemy targets are hit.

- `skill_directional_collision_projectile_can_hit_allies_when_explicit`
  - Explicit directional collision projectile skill with `hit_targets: Any` or `Allies`.
  - Ally is in the path.
  - Assert ally can be hit.

- `skill_target_locked_projectile_does_not_hit_bystander`
  - Target-locked projectile skill.
  - Bystander on path.
  - Assert bystander is not impacted by continuous collision.

- `retarget_on_step_uses_tile_range_for_official_targeting`
  - Step retargets at execution time.
  - Only targets in step tile range are eligible.

Live/data tests:

- Existing live weapons load with effective tile ranges or employee fallback where appropriate.
- Live ordinary corroded-employee basic attacks reference explicit RON range presets from the corroded employee range pool.
- Live corroded employee profiles declare `profile_role`, and `Normal` profiles cannot use `WholeFieldValidTiles`.
- `WholeFieldValidTiles` skills without explicit targeting rules fail validation.
- Explicit whole-field `TileArea` skills can load only when targeting/effect policy is fully authored.
- Special boss/abnormality/elite long-range basic attacks have authored `defense_tile_range` instead of relying on `range_units`.
- Existing live skills declare projectile mode consistently.
- Official DefenseRoute skill data does not silently depend on `range_units` for target eligibility.
- `docs/skill_target_contract.md` no longer describes `RetargetOnStep` or default basic attack fallback in a way that conflicts with this goal.

## Completion Conditions

- Basic attack eligibility no longer uses continuous distance, hitbox radius, or `range_units`.
- Airborne basic attack targets still require tile range membership.
- Basic attack projectile impact is target-locked and no longer path-collision based.
- Collision projectile remains available for explicitly declared skill deliveries.
- `range_units` remains available for non-basic-attack mechanics and explicitly continuous skill mechanics.
- Official DefenseRoute skill eligibility is tile-based unless the skill explicitly declares collision projectile behavior.
- Route-following ranged enemy cadence honors blocker priority, previous-target revalidation, authored `interval_ms`, and authored `ranged_reposition_ms`.
- Unity-facing range preview remains final cell DTO based.
- Routine Unity-facing updates do not include enemy range previews; enemy telegraph/debug events include actual affected tiles only when needed.
- Deployed and pending range preview cells use the same effective tile range helper as combat eligibility.
- Docs are updated to match the implemented policy.
- Tests cover tile-only basic attacks, target-locked projectiles, collision projectile skills, and live data validation.
- No compatibility layer, fallback path, or dual schema is introduced unless explicitly recorded with removal conditions and approved by the user.

## Stop Conditions

Stop and ask the user before continuing if any of these appear:

- A live basic attack intentionally needs collision projectile behavior.
- A live skill needs both target-locked and collision projectile behavior in one delivery step.
- A RON schema migration would delete or reinterpret existing authored content in a way that changes intended balance.
- Unity-facing timeline DTO shape must change and the required presentation semantics are not already covered by docs.
- Movement planner cannot function without `basic_attack.range_units` and replacing it requires a separate movement policy.
- Existing official skills rely on `range_units` for intended non-projectile target eligibility.
- Implementing this safely requires a compatibility layer or long-lived dual schema.

## Verification Commands

Use exact focused names after tests are added. Expected categories:

```text
cargo test basic_attack_uses_tile_range_not_range_units
cargo test airborne_basic_attack_target_still_requires_tile_range
cargo test basic_projectile_is_target_locked_not_path_collision
cargo test skill_collision_projectile_uses_continuous_sweep
cargo test retarget_on_step_uses_tile_range_for_official_targeting
cargo test -p game_core
cargo check -p game_core
```

Adjust package names to the actual workspace package names after confirming `Cargo.toml`.
