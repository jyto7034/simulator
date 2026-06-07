# Delivery Area Removal Plan

## Objective

Remove legacy geometric skill AoE completely. Official DefenseRoute skills use only `SkillStepDef.defense_tile_range` and `DeliveryDef::TileArea`.

## Current Classification

- Live official RON: `../game_resources/data/skills/base.ron` has no `delivery: Area` or geometric shape hits.
- Legacy data: `../game_resources/data/skills/legacy_base.generated.ron` was an old generated `Area(...)` artifact and has been removed.
- Removed code residue: `DeliveryDef::Area`, `SkillAreaDeliveryDef`, `SkillAreaShapeDef`, geometric execution branch, and `skill_catalog` `legacy_area` branch.
- Removed test-only residue: core unit tests and spatial backend tests that directly validated geometric area semantics.

## Steps

1. Remove schema types and delivery variant.
2. Remove geometric runtime branch and helper functions.
3. Collapse timeline area shape to tile pattern only.
4. Remove or rewrite geometric tests.
5. Remove legacy generated RON.
6. Update docs and contracts.
7. Run focused tests, then full game_core and check commands.

## Completion Checklist

- [x] No `DeliveryDef::Area` in `src` or `tests`.
- [x] No `SkillAreaDeliveryDef` or `SkillAreaShapeDef` in `src` or `tests`.
- [x] No `legacy_area` catalog branch.
- [x] No geometric area timeline variants.
- [x] No `delivery: Area` in official or legacy skill RON.
- [x] Focused tests pass.
- [x] `cargo test -p game_core -- --nocapture` passes.
- [x] `cargo check -p game_core` passes.
- [x] `cargo check -p game_server` passes.
