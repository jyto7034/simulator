# Tile Based Attack Delivery Contract Experiment Notes

## Working Judgments

- Do not delete `range_units` globally.
  - It is likely needed for contagion, contact, proximity, aura, and special continuous-space mechanics.
  - It may remain useful for movement steering or explicitly continuous skill behavior.

- Do remove `range_units` from basic attack eligibility.
  - Basic attacks should feel like Arknights-style tile combat.
  - Airborne enemies are not a continuous-distance exception; they must be inside attack tiles.

- Do not remove continuous collision from skill projectiles.
  - Some future/current skills need physical projectile collision, piercing, and max-hit behavior.
  - This must be explicit in skill delivery, not an implicit default for all projectiles.
  - The important continuous projectile case is directional: a projectile can be fired in the caster's current facing direction without a unit target and should travel until its authored stop policy is satisfied.
  - Preferred mode names: `TargetLocked` and `DirectionalCollision`.
  - Preferred targetless option name: `allow_targetless_cast`.
  - Preferred stop policy fields: `max_range_tiles`, `max_lifetime_ms`, `max_hits`, `max_kills`, `max_pierces`.
  - Stop policies should be extensible: authored tile range, lifetime, endpoint, hit count, kill count, pierce count, or equivalent data.
  - For this goal, authored forward range tiles define the authoritative projectile path and endpoint. Do not add separate invalid/out-of-battle boundary truncation unless a later policy explicitly needs it.
  - Default hit target filter is `Enemies`; ally hits require explicit `Allies` or `Any`.

- Basic attack projectile should be target-locked.
  - Tile eligibility determines whether the shot can be fired.
  - For windup attacks, "can be fired" means the target is still tile-eligible at `AttackResolve` / release time.
  - Projectile timeline exists for presentation and delayed impact.
  - Arrival checks only target-locked validity: still present, alive, hostile to the original attacker side, and not invalidated by non-range targetability.
  - It should not discover a new hit by sweeping through bystanders.
  - It should not re-check tile range after launch.

- Basic attack tile range is data-first.
  - Employee no-weapon/no-skill fallback: self tile plus one forward tile.
  - Corroded employee basic attack range: profile-selected RON range preset.
  - Initial corroded employee presets:
    - `melee_front_1`: self tile plus one forward tile.
    - `ranged_center_3x3`: 3x3 square centered on the unit's current tile, facing-independent.
    - `ranged_center_5x5`: 5x5 square centered on the unit's current tile, facing-independent.
  - Ranged center presets use valid battlefield tiles only, but obstacle/blocked tiles remain attack candidates.
  - The basic attack range preset pool is a shared authoring pool; corroded employee profiles reference entries from it.
  - Existing authored `defense_tile_range` stays authoritative.
  - Existing live corroded employee attacks identified only by `range_units > 1` or projectile delivery should be migrated to explicit profile-selected range presets.
  - Boss, abnormality, elite, or special attacks that intentionally need nonstandard range must receive a specific authored `defense_tile_range`.
  - Employee fallback is for genuinely unspecified employee/test attacks, not for live corroded employee or special long-range balance.

- Prefer reusing `DeliveryDef` over inventing a parallel "attack type" model.
  - Existing concepts map well:
    - `Instant` => hitscan/immediate.
    - `Projectile` => projectile, with explicit target-locked vs collision mode.
    - `TileArea` => tile area skill delivery.

- Reconcile projectile mode with existing guidance naming before implementation.
  - Runtime and timeline currently expose `Homing` / `Fixed`.
  - New policy language is `TargetLocked` / `DirectionalCollision`.
  - These must not become two independent public axes that can disagree.
  - It is acceptable for `DirectionalCollision` to reuse fixed-motion internals, but data/runtime/timeline boundaries should expose the policy name that Unity and validation need.

- Range preview fallback must not be a second range system.
  - `src/game/range_preview.rs` currently owns fallback cells independently.
  - It should use the same effective basic attack tile range helper as combat eligibility.
  - Pending placement preview, deployed checkpoint preview, and targeting should agree for employee fallback and corroded employee profile presets.

- Non-player facing is an unresolved source-of-truth issue.
  - Player deployments have user-selected facing.
  - Current scenario-spawned opponent units usually start with `facing_direction: None`.
  - User approved movement direction as the facing source for non-player route units.
  - Route-following enemies update facing from movement direction and keep the last movement facing while blocked/engaged.
- User later revised ordinary ranged corroded employee range to be facing-independent:
  - ranged corroded employee basic attack presets are centered square ranges such as `ranged_center_3x3` and `ranged_center_5x5`,
  - route-following ranged enemies still move along their route,
  - after attack release/damage/projectile spawn they start explicit `ranged_reposition_ms` route movement, default 1000ms,
  - windup time does not count toward `ranged_reposition_ms`,
  - `ranged_reposition_ms` is profile/RON-overridable, must be >= 0, and 0 means immediate previous-target revalidation,
  - no new ranged target search occurs during reposition unless a stronger interrupt state occurs,
  - after reposition they revalidate the previous target before searching for a new target,
  - blocked enemies do not reposition and prioritize the blocker over previous ranged targets, normal target-search candidates, defense/protection objectives, and route movement,
  - blocker priority applies even when the protected/defense objective is inside the ranged attack tiles,
  - protected/defense objectives are target candidates only through the authored targeting/fallback policy after blocker and previous-target rules are resolved,
  - `AttackStart` checks whether an attack can begin, and `AttackResolve`/release-time tile eligibility is the authoritative final check for damage application or projectile spawn.
- User confirmed attack cadence must remain authored and customizable:
  - `interval_ms` controls the next basic attack timing,
  - `windup_ms` controls release timing,
  - `ranged_reposition_ms` controls post-release route movement before previous-target revalidation,
  - these fields are independent and must not be collapsed into one timing knob,
  - some units should be able to attack faster or slower through data without special-case code.
- User later revised route-less policy into explicit `WholeFieldValidTiles` range policy, and then decoupled that policy from route-less movement.
  - `WholeFieldValidTiles` can be used by both basic attacks and skills.
- Route-following units may use `WholeFieldValidTiles` only when their unit/profile/skill explicitly allows it.
- Bosses, special corroded employees, and special units such as snipers may use this policy.
- Regular `profile_role: Normal` corroded employees must not use `WholeFieldValidTiles`.
  - Whole-field range only widens candidate tiles; actual target choice still follows target validation and targeting rules.
  - Skills using `WholeFieldValidTiles` must declare explicit targeting rules; otherwise they are invalid data.
  - Enemy attack range previews should not be included in routine Unity-facing DTOs. Enemy telegraphs/debug warnings send actual affected tiles only when needed.
  - `WholeFieldValidTiles` + `TileArea` is allowed for explicit whole-field area skills, but skill data must clearly author the whole-field area and targeting/effect policy.
  - Current `CorrodedEmployeeProfileMetadata` has no profile-role field. Add explicit `profile_role` rather than reusing wave preset `role_mix`.
  - Initial `profile_role` values: `Normal`, `Special`, `LegacyEcho`.
  - Elite, easter-egg employee, boss, abnormality, and other special units keep authored attack/skill ranges. Do not let ordinary fallback erase those unique ranges.

## Policy Questions To Raise If Encountered

- Should a basic attack projectile be able to miss because the target moved, or only because the target died/became untargetable?
  - Current plan says no range recheck at arrival and no movement dodge unless target is dead/untargetable.

- Should collision projectile skills be allowed to choose no initial unit target and fly toward a tile/direction?
  - Yes. Directional collision projectile is an intended skill delivery policy.
  - Direction comes from the caster's current facing, not from cast-time user input.
  - Targetless single-shot casting must be explicit on the skill.
  - It still needs explicit travel stop policy data. Stop if new targeting UX or command DTO is needed.

- Should `range_units` remain serialized on basic attack data after it is no longer used for basic attack eligibility?
  - Current plan allows keeping it temporarily if live data migration or movement code still needs it, but attack eligibility must not use it.

- Should movement planner approach distance use tile range instead of `range_units`?
  - Current plan treats this as a possible separate policy if it becomes necessary.

- Should existing `ProjectileGuidance::{Homing, Fixed}` be renamed, wrapped, or kept as internal-only?
  - Decide during domain model work.
  - Stop if Unity-facing DTO compatibility requires keeping stale names that obscure target-locked vs collision semantics.

- If a special unit lacks authored attack range data, should it use ordinary fallback or should data be fixed?
  - Current policy says data should be fixed. Stop and ask if intended authored ranges are missing.

- If live basic attack data has `range_units > 1` or projectile delivery but no tile range, do not silently collapse it to melee range.
  - Ordinary corroded employee attacks must choose a profile range preset from RON.
  - Special long-range attacks require authored `defense_tile_range`; stop if the intended shape is unclear.

- When making basic attack projectiles target-locked, impact-time validity should be deliberately narrow.
  - Allowed checks: target still exists, alive, hostile to the original attacker side, and not invalidated by non-range targetability such as air capability/untargetable state.
  - Disallowed checks: attack range, target usefulness, continuous body reach, bystander collision.

- Implemented schema field name for whole-field range as `range_policy`.
  - Default is `pattern`; existing `defense_tile_range` projection remains the default behavior.
  - `whole_field_valid_tiles` is explicit data and conflicts with `defense_tile_range`.
  - Basic attacks and skills can both use the policy.
  - Unity-facing skill catalog DTO now includes `range_policy`, but range overlay remains driven by runtime `range_previews` final cell DTO.
  - Routine enemy range preview remains out of scope; enemy warning/telegraph events should send actual affected cells only when needed.

## Follow-Up Candidates Outside Current Goal

- Rename or document `Instant` as "hitscan" in user-facing docs if terminology confusion persists.
- Add a dedicated proximity mechanic module for contagion/contact/aura instead of reusing targeting helpers.
- Add richer projectile presentation hints for Unity now that target-locked vs directional collision projectile runtime behavior is implemented.
