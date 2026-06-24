# Experiments

## Log

| Date | Scope | Attempt | Result | Follow-up |
| --- | --- | --- | --- | --- |
| 2026-06-22 | Goal setup | Defined skill/targeting/projectile implementation scope. | Not started. | Begin with live skill schema and preview command contracts. |
| 2026-06-22 | Schema/code inventory | Read skill raw loader, public `SkillDef` serde, live skill RON, skill catalog DTOs, range preview, tile area runtime, projectile miss recording, timeline validators, and weapon targeting profiles. | Success. Found legacy implicit `cast_targeting`, live skill `range_units`, catalog `range_units`, raw tile-area affected tiles, duplicate `ProjectileMiss`, and `SplashClusterFirst`. | Implement focused policy removals. |
| 2026-06-22 | Explicit cast targeting | Migrated live skill RON to explicit `cast_targeting`; removed raw `FirstStepTarget` live-loader support; made public `SkillDef` RON require `cast_targeting`. | Success. Live RON and direct RON tests now require explicit cast targeting. | Keep test constructors free to use `Default` where they are not RON contracts. |
| 2026-06-22 | Skill range source | Removed live skill RON `range_units` and Unity catalog cast/step `range_units` fields. | Success. `range_previews.final_cells` remains the displayed range source. | Keep basic attack/equipment `range_units` because this policy targets skill targeting/catalog surfaces. |
| 2026-06-22 | Tile area clipping | Routed runtime tile-area affected tiles and active skill previews through battlefield valid-tile clipping. | Success. Added focused preview/runtime clipping tests. | None. |
| 2026-06-22 | Projectile miss event | Removed `TimelineEvent::ProjectileMiss` and stopped recording duplicate miss events. | Success. Existing projectile miss path now asserts `BasicAttackProjectileImpacted(hit=false)`. | Add `miss_reason` later only if policy requires it. |
| 2026-06-22 | Targeting profile removal | Removed `SplashClusterFirst` enum variant, scoring function, and test. | Success. No remaining references in code/data/tests. | None. |
| 2026-06-22 | Final validation | Ran focused skill/projectile/range-preview tests, live RON loading, server check, full lib tests, and diff check. | Success. All validation passed. | Continue with `core_policy_unity_server_contract`. |

## Failed Approaches

- Runtime tile-area clipping test initially failed because `BattleCore::new_from_scenario` does not build the scenario battlefield valid-tile set until execution setup. The test now explicitly installs a valid-tile `Battlefield` before calling the low-level area resolver.
- A patch accidentally made the tile-area clipping helper recursively call itself. Fixed the helper to call `TileRangePattern::affected_tiles` and then filter through `Battlefield::in_bounds`.

## Validation Commands

- `cargo check --lib` - passed.
- `cargo test game::ability::tests::skill_ --lib` - passed, 8 tests.
- `cargo test game::data::skill_data --lib` - passed, 11 tests.
- `cargo test active_skill_preview_clips_pattern_to_valid_tiles --lib` - passed.
- `cargo test instant_tile_area_affected_tiles_clip_to_valid_battlefield_tiles --lib` - passed.
- `cargo test advance_basic_attack_projectile_misses_when_locked_target_dies_before_contact --lib` - passed.
- `cargo test game::battle::core::targeting --lib` - passed, 6 tests.
- `cargo test skill --lib` - passed, 79 tests.
- `cargo test projectile --lib` - passed, 15 tests.
- `cargo test range_preview --lib` - passed, 8 tests.
- `cargo test --test ron_loading` - passed, 18 tests.
- `cargo check -p game_server` - passed.
- `cargo test --lib -- --test-threads=1` - passed, 514 tests.
- `git diff --check` - passed.
