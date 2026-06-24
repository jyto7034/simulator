# Core Battlefield Layout Extraction Review

Review guide: `docs/goal_completion_review_guide.md`

## Audit Table

| Item | Verdict | Long-term fit | Notes |
|---|---|---|---|
| Rename `Battlefield` to static layout object | Complete | High | Runtime layout type is now `BattlefieldLayout`; it owns only dimensions, valid tiles, and static obstacles. |
| Remove dynamic unit position ownership | Complete | High | `unit_pos`, tile `occupant`, `position_of`, `place`, `remove`, and `units_at` were removed with no compatibility wrappers. |
| Derive tile positions from `RuntimeUnit.body` | Complete | High | live deployment/checkpoint, death snapshots, spawn/deploy event positions, skills, projectiles, and tests now use `projected_tile` helpers. |
| Valid/walkable/void tile API naming | Complete | High | `is_valid_tile`, `position_in_bounds`, `ensure_valid_tile`, `ensure_walkable_tile`, and `void_tiles` match the confirmed policy. |

## Battlefield Static Layout

Master item: `core_battlefield_layout_extraction`

Subgoal: `docs/goals/core_battlefield_layout_extraction`

Policy source: `docs/goals/core_component_refactor_master/POLICY_DECISIONS.md` / `Battlefield static layout extraction`

Expected behavior:

`BattlefieldLayout` is a static authored-layout object. It must not store runtime unit position, occupant state, deployment state, death state, or withdrawn state. Runtime unit position is sourced from `RuntimeUnit.body`.

Runtime evidence:

- `src/game/battle/battlefield/mod.rs`: `BattlefieldLayout` fields are `width`, `height`, `valid_tiles`, and `static_obstacles`.
- `src/game/battle/battlefield/field.rs`: layout APIs are valid/walkable/static-obstacle/void-tile queries only.
- `src/game/battle/core/mod.rs`: `BattleCore` owns `pub battlefield: BattlefieldLayout` and exposes body-derived `unit_projected_tile` helpers.
- `src/game/battle/core/build.rs`: spawn/deploy validate walkable tiles and deployed event position comes from `unit_projected_tile`.
- `src/game/battle/core/commands.rs`: death snapshot uses `target.body.projected_tile`; projectile launch uses live body-derived projected tiles.
- `src/game/battle/core/sim.rs`: skill cast target anchoring and manual caster position use body-derived projected tiles; explicit tile targets use `is_valid_tile`.
- `src/game/world/state.rs`: live deployment and checkpoint DTO `position` fields derive from `unit_projected_tile` / `unit.body.projected_tile`.
- `src/game/range_preview.rs`: whole-field previews use `valid_positions`; tile-pattern previews clip with `is_valid_tile`.

Data evidence:

- No live RON schema change was required. This subgoal changes runtime ownership and API names, not authored data shape.

External contract evidence:

- Unity-facing DTO `position` fields remain present.
- Their source is now body projection rather than layout-owned tile storage.
- No new external DTO field, transport envelope, or save migration was introduced.

Test evidence:

- `cargo check`
- `cargo check -p game_server`
- `cargo test battlefield --lib -- --test-threads=1`
- `cargo test static_obstacles_block_placement --lib -- --test-threads=1`
- `cargo test advance_basic_attack_projectile_does_not_recheck_tile_range_at_arrival --lib -- --test-threads=1`
- `cargo test instant_tile_area --lib -- --test-threads=1`
- `cargo test live_deployment_reconciles_defeated_player_unit_before_state_dto --lib -- --test-threads=1`

Legacy/fallback audit:

- `rg -n "battlefield\\.(position_of|place|remove|units_at|occupant)|\\bunit_pos\\b|pub struct Battlefield\\b|impl Battlefield\\b|struct Tile\\b|\\.in_bounds\\(" src tests -g '*.rs'` returned no matches.
- No deprecated wrapper APIs were kept.
- No ignored tests or dual position source were added.

Long-term direction review:

- Fit: high
- Improvement class: follow-up refactor
- Reason: unit position SoT is simplified around `RuntimeUnit.body`, and static layout responsibility is clearer. The only notable follow-up is that `build_runtime_field` currently validates all materialized units against walkable projected tiles. That is correct before lifecycle retention lands, but the lifecycle subgoal must revisit it when withdrawn/dead units remain in `BattleCore.units`.

Verdict:

- Complete

Remaining risk:

- `core_runtime_unit_lifecycle` must ensure inactive units retained in `BattleCore.units` are not accidentally treated as active placement participants by `build_runtime_field`, targeting, movement, or checkpoint filters.

## Cross-Component Findings

- No new policy-decision requirement was found.
- The implementation aligns with the later lifecycle policy by removing layout-owned unit position first.
- The next lifecycle/redeploy subgoals must update any code that currently removes units from `BattleCore.units`; that is outside this subgoal and already part of the master sequence.
