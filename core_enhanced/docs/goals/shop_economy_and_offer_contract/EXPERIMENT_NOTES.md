# Shop Economy And Offer Contract Notes

Goal: G28

Status: complete

## Accepted Policy Sources

- Master PD-18 and the 2026-07-12 Shop decision series.
- G24's completed equipment-instance/definition-stack/reserve transaction contract.
- PD-17: Artifact is removed without replacement; G28 removes only its Shop/content dependencies and
  G27 deletes the remaining subsystem.

## Technical Decisions

- Use typed merchandise identity variants rather than raw cross-registry UUID stock. Initial variants
  are Equipment and Consumable; future categories extend the enum and behavior deliberately.
- Give each generated offer a deterministic session/page/slot-derived UUID separate from equipment
  owned identity and consumable definition identity.
- Store complete current offers in the active session. Never regenerate them during snapshot/read.
- Weighted sampling is without replacement after repeatability and eligibility filtering. Sort the
  flattened candidates by typed identity before applying deterministic RNG so RON/index iteration
  order is not an accidental authority.
- Quote an offer's total authored quantity. Equipment quantity must be one. Consumable quantity is a
  checked positive value granted into the G24 stack.
- Rename Reroll to Refresh across action kind, player behavior, command result, server/Unity command,
  mocks, docs, and tests. Do not retain aliases.
- Keep `shop_pool_id` only as the existing encounter-level Shop selection profile seam unless source
  work proves a rename materially improves ownership. It is distinct from merchandise presets.
- Sale tags belong to merchandise entries for filtered Shops; `accepted_tags: None` still accepts any
  intrinsically removable equipment/consumable even if it has no Shop entry.
- Context capability answers whether any payload is executable. Offer and sale DTO eligibility owns
  exact payload reason and quote; commit reuses those evaluators.

## Guardrails

- No `ShopType`, `DiscountShop`, fixed `visible_items`/`stock_items`, hidden swap, `can_reroll`, free
  one-shot reroll, raw base-price purchase, global 50% sale, Artifact offer, or placeholder offer.
- No client-side price, percentage, refresh, sellability, or pool calculation.
- No compatibility RON schema, serde alias, dual command name, UUID-only offer DTO, or synthetic
  fallback offer.
- No hard-coded permanent merchandise-category allowlist or high-value-implies-no-refresh rule.
- No partial refresh page, silent duplicate merge, price clamp, zero-value transaction, or mutation
  before every checked stage succeeds.

## Current Trace

- Static schema/index: `src/game/data/shop_data.rs`.
- Cross-database validation: `src/game/data/validation.rs` and live bundle tests.
- Entry/session creation: `src/game/world/map_content.rs`, `world/headquarters.rs`, and
  `resources/selection.rs`.
- Transaction/capability: `src/game/world/shop.rs` and `world/action_capability.rs`.
- DTO/result projection: `src/game/behavior.rs` and `world/snapshot.rs`.
- Checkpoint state: `src/game/world/state.rs` and `world/node_flow.rs`.
- Server: typed Core payload pass-through plus bridge documentation; no parallel economy mapper.
- Unity: `SelectedEventModel.cs`, `ShopRewardScreenComposer.cs`, selection state, command dispatch,
  mocks, and contract/render tests.

## Completion Boundary

- The final Shop contract is Core-authoritative from composed live content through deterministic
  session state, checked quotes, payload eligibility, atomic transactions, checkpoint persistence,
  server pass-through, and Unity presentation/dispatch.
- Completion review found and corrected one obsolete `ShopRerollNotAllowed` production error mapping
  and added startup validation for quantity-adjusted total quote overflow. The corrected full boundary
  is green.
- Artifact has no Shop merchandise, validation, runtime, DTO, server, fixture, or Unity dependency.
  G27 is now unblocked and owns deletion of the remaining isolated Artifact subsystem without
  replacement or compatibility.
- Verdict: Complete / High; no follow-up correction goal is required.
