# Experiments

| Date | Trial | Result | Notes |
| --- | --- | --- | --- |
| 2026-07-06 | Goal document creation from audit review. | Planned. | Covers P-011 and P-013: combat setup namespace clarity and shop raw schema cleanup. |
| 2026-07-06 | Inspect live shop data and raw schema. | Success. | Live shop RON uses `stock_items`; `ShopMetadataRaw` now rejects unknown fields, so raw authored `hidden_items` is no longer accepted. Runtime `ShopMetadata.hidden_items` remains internal reserve state. |
| 2026-07-06 | Replace flat `combat_*` setup modules with a `combat_setup` namespace. | Success. | `src/game/mod.rs` exports `pub mod combat_setup`; call sites use paths such as `game::combat_setup::enemy_spawns` and `game::combat_setup::rewards`. No compatibility aliases were kept. |
| 2026-07-06 | Check canonical docs for shop authoring schema or old flat combat module paths. | No update needed. | `docs/README.md` marks audit/goal docs as non-canonical. Current top-level canonical docs did not contain a shop authoring schema or old flat `combat_*` setup API that needed updating for this goal. |

## Validation Log

- 2026-07-06: `cargo test -p game_core shop --lib -- --test-threads=1` passed.
- 2026-07-06: `cargo test -p game_core --test ron_loading -- --test-threads=1` passed.
- 2026-07-06: `cargo test -p game_core combat_setup --lib -- --test-threads=1` passed.
- 2026-07-06: `cargo check -p game_core` passed.
- 2026-07-06: `cargo fmt --check` passed.
