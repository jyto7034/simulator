# Experiment Notes

## Initial Audit Basis

- `src/game/mod.rs` exposes `combat_setup/*` files as flat modules through `#[path = ...]`.
- Live shop RON uses `stock_items`.
- `ShopMetadataRaw` still accepts raw `hidden_items`, while current policy treats hidden stock as internal runtime state.

## Policy Notes

- `stock_items` should mean authored total shop stock.
- `visible_items` should mean initially visible subset.
- Internal hidden/reserve stock can still exist after loading, but that should be derived from `stock_items - visible_items`.
- Raw authored `hidden_items` is intentionally rejected instead of treated as a compatibility alias. This keeps one external shop stock source while preserving internal reroll reserve state.
- Runtime `ShopMetadata.hidden_items` and `ShopSessionState.hidden_items` are not legacy authoring fields in this goal; they represent derived/internal reserve stock and remain valid.
- Flat combat setup aliases were removed rather than retained. `combat_setup` ownership is now visible in import paths.
- Broad doc search still finds stale audit/goal references to old issues. `docs/README.md` classifies those as review/history material, not canonical policy, so this goal does not rewrite every historical audit note.

## Follow-Up Candidates Outside Scope

- Larger reward/shop/support domain refactor.
- Full public module hierarchy cleanup across `src/game`.
