# Low-Risk Schema And Namespace Cleanup

## Objective

Resolve the low-risk cleanup findings P-011 and P-013:

- `combat_setup/*` modules are exposed through flat names such as `combat_enemy_spawns`, which hides their ownership boundary.
- `ShopMetadataRaw` still accepts the older authoring field `hidden_items` even though live RON uses `stock_items` and hidden stock should be an internal runtime reserve state.

## Source Of Truth Order

1. Runtime code and live RON.
2. Data validation and live RON loading tests.
3. Unity-facing snapshot/command contracts.
4. Canonical docs.
5. Audit notes.

## Scope

- Move `combat_setup` exports toward a clear namespace if call sites allow it.
- Remove the external shop authoring ambiguity by rejecting/removing raw `hidden_items` from authored shop data.
- Keep runtime `ShopSessionState.hidden_items` or equivalent internal reserve-state naming where it actually means hidden stock after visible stock is selected.

## Non-Goals

- Do not change shop reroll behavior or visible shop DTO shape.
- Do not expose hidden shop stock to Unity.
- Do not refactor reward/shop economy logic beyond schema cleanup.
- Do not split all combat setup logic; this goal is namespace/API cleanup only.

## Plan

1. Re-read:
   - `src/game/mod.rs`
   - `src/game/combat_setup/*`
   - call sites for `combat_enemy_spawns`, `combat_rewards`, and related flat module names
   - `src/game/data/shop_data.rs`
   - live shop RON under `game_resources/data/events/shops`.
2. Shop schema cleanup:
   - Confirm live RON uses `stock_items`.
   - Remove `ShopMetadataRaw.hidden_items` as an accepted authoring field.
   - Keep validation that `visible_items` must be included in `stock_items`.
   - Update tests that still author raw `hidden_items`.
3. Combat setup namespace cleanup:
   - Prefer `crate::game::combat_setup::enemy_spawns` style if module visibility and call sites permit.
   - Avoid compatibility aliases unless a public API dependency is found.
   - If flat aliases must temporarily remain, record reason and removal condition.
4. Run focused shop and combat setup tests.
5. Update canonical docs if they mention shop authoring schema or combat setup module layout.

## Completion Conditions

- Authored shop RON has exactly one external total-stock field: `stock_items`.
- Runtime hidden reserve state remains internal and is not serialized to selected-event snapshots.
- `combat_setup` ownership is clearer from module paths or documented transition.
- No dual authoring schema remains unless explicitly justified with a removal condition.
- Focused live RON loading tests pass.

## Validation Commands

- `cargo test -p game_core shop --lib -- --test-threads=1`
- `cargo test -p game_core --test ron_loading -- --test-threads=1`
- `cargo test -p game_core combat_setup --lib -- --test-threads=1`
- `cargo check -p game_core`
- `cargo fmt --check`

## Stop Conditions

Complete the goal and report questions if:

- Any live or intended authored shop data still requires `hidden_items`.
- External crates or Unity-facing tooling depend on flat `combat_*` module paths.
- Shop reroll policy needs to change rather than only schema names.

## Implementation Summary

- `src/game/mod.rs` now exposes `pub mod combat_setup` instead of flat `combat_*` setup modules.
- `src/game/combat_setup/mod.rs` owns the setup namespace and re-exports `balance`, `battlefield_plan`, `defense_object`, `enemy_spawns`, `mission_policy`, `player_spawns`, `rewards`, and `scenario_groups`.
- Runtime call sites were updated to use namespaced paths such as `crate::game::combat_setup::enemy_spawns`.
- `ShopMetadataRaw` now accepts `stock_items` as the only authored total-stock field and rejects unknown fields, including raw `hidden_items`.
- Derived runtime hidden/reserve stock remains internal as `ShopMetadata.hidden_items`/`ShopSessionState.hidden_items`; selected-event snapshots still expose only visible shop stock.
- Canonical docs did not require a policy update for this cleanup. Historical audit/goal documents may still mention the old findings, but they are not source-of-truth documents.
