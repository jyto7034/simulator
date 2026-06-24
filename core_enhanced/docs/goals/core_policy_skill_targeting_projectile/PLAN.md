# Skill Targeting and Projectile Policy Implementation

## Objective

Make skill range/targeting explicit, tile-based, and runtime-preview driven while removing duplicated projectile and miss semantics.

## Policies Covered

- `explicit cast_targeting`
- `remove range_units from skill targeting`
- `TileArea affected_tiles clipping`
- `long line and piercing skill representation`
- `runtime range preview source`
- `projectile launch owner snapshot`
- `projectile miss event source`
- `remove SplashClusterFirst targeting profile`

## Plan

1. [x] Read skill RON schema, skill runtime execution, targeting profiles, tile area calculations, range preview commands, projectile scheduling, and battle event DTOs.
2. [x] Require explicit `cast_targeting` in live RON; remove inference from first step target in the live loader and public `SkillDef` RON.
3. [x] Remove `range_units` as a generic catalog display or live skill targeting source.
4. [x] Clip `TileArea.affected_tiles` to valid tiles like previews.
5. [x] Represent long line and piercing skills with tile patterns plus valid-tile clipping, not arbitrary projectile range hacks.
6. [x] Use runtime preview commands as the source for basic attack and selected skill ranges.
7. [x] Keep projectile launch owner snapshot for damage context after attacker death.
8. [x] Canonicalize miss events as `BasicAttackProjectileImpacted(hit=false)` and remove `ProjectileMiss`.
9. [x] Remove `SplashClusterFirst`.
10. [x] Update tests for live skill loading, preview DTOs, tile clipping, projectile hit/miss events, and removed targeting profiles.

## Completion Conditions

- Live skills cannot omit explicit cast targeting.
- Static catalog range display does not act as targeting source.
- Skill areas and previews agree on valid tile clipping.
- Long line/piercing skills are expressed through tile patterns.
- Projectile misses have one canonical event shape.
- Removed targeting profiles fail validation if referenced.

## Completion Report

- Live skill RON now declares explicit `cast_targeting` for every skill and no longer contains skill `range_units`.
- The live skill loader rejects omitted `cast_targeting` and legacy `range_units` fields; public `SkillDef` RON also no longer defaults missing `cast_targeting`.
- Unity-facing skill catalog DTOs no longer expose generic cast/step `range_units`; catalog continues to point to `range_previews.final_cells` as range source of truth.
- Runtime and preview tile-area calculations now clip pattern cells through the battlefield valid-tile set.
- Long line/piercing live skills remain represented by tile range presets/patterns and valid-tile clipping; no projectile range hack was introduced.
- Basic attack projectile miss now emits only `BasicAttackProjectileImpacted { hit: false }`; `ProjectileMiss` was removed from the timeline and validators.
- `SplashClusterFirst` targeting profile and its cluster-scoring hard-code were removed.
- Projectile launch owner snapshot behavior was preserved and revalidated by existing projectile tests.
- Validation passed: `cargo check --lib`; `cargo test game::data::skill_data --lib`; `cargo test skill --lib`; `cargo test projectile --lib`; `cargo test range_preview --lib`; `cargo test --test ron_loading`; `cargo check -p game_server`; `cargo test --lib -- --test-threads=1`; `git diff --check`.
