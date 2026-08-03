# Shop Economy And Offer Contract Experiments

Goal: G28

Updated: 2026-07-14

## Entry Evidence

- `ShopMetadata` currently owns `ShopType`, `can_reroll`, fixed `visible_items`, and private
  `hidden_items` derived from `stock_items`.
- Map and Headquarters entry deterministically select a whole Shop, then clone that fixed page into
  `ActiveNodeContent::Shop`.
- Purchase selects a merchandise definition UUID, charges raw base price, grants quantity one, and
  deletes that UUID from the current page.
- Sale selects an owned item/static consumable UUID, always uses floor 50% of base price, and has no
  authored availability, tag filter, quantity payload, or projected quote.
- Reroll is a one-time free visible/hidden vector swap whose page size changes. Context capability
  projection cannot express concrete offer, refresh, or sale reasons.
- The current root snapshot already resolves visible item metadata, but Unity discards it and reads
  only `visible_item_uuids`. Unity shows UUID labels, only unequipped equipment for sale, always shows
  reroll, calculates no quote, and does not reconcile stale selection.
- Run checkpoints restore deterministic run/floor/map state but currently contain no Shop session
  records. Active node content is retained in memory for reconnect projection.
- Live Shop data contains Artifact-only/mixed stock, unreachable hidden stock, an empty initial page,
  zero-price placeholder equipment, and no live acquisition source for seven enabled consumables.

## Dirty Baseline

- Core/server repository HEAD: `c2c5dc5ec54bd26530e46f0d7ee7909de3ed7a1f`.
- Core status fingerprint: `613d37620a766b989415ff95cc289969a06c27cf4b29dcd40a34bddc49e6c3e0`.
- game_server status fingerprint: `f774d4ffe67e7c82e978623fa00e65b10d2643cf24a702c6fb17e6b1293db4af`.
- Unity HEAD: `7cd8df1f5ae2e438e28bda2c761e7a56d21e59cf`.
- Unity status fingerprint: `af00e2aaa006fd32720ec811abac93fa737539e6b48290e97c44f3eaa285a164`.
- Entry executable baseline inherits G24's immediately preceding green boundary: Core 761,
  game_server 30, Unity EditMode 217, Unity PlayMode 52 plus 4 explicit external-server skips, and
  cleared console 0 errors.
- Preserve all pre-existing dirty changes. G28 owns only Shop schema/live data/runtime/session/
  projection/server-contract/Unity Shop seams and its goal/master evidence.

## Planned Experiments

| Date | Experiment | Expected Evidence | Status |
|---|---|---|---|
| 2026-07-14 | Flatten preset references plus local entries | Exact unique typed candidate set; duplicate merchandise rejected | passed |
| 2026-07-14 | Generate entry page twice from the same restored state | Same Shop, offer identities, merchandise, quantities, prices, and order | passed |
| 2026-07-14 | Refresh before/after non-repeatable purchase | Full stable-size page; purchased identity excluded; cost/use/page commit together | passed |
| 2026-07-14 | Exhaust eligible refresh pool | Typed disabled reason and direct rejection with zero mutation | passed |
| 2026-07-14 | Purchase equipment and multi-quantity consumable offers | Owned equipment instance plus exact definition-stack quantity through G24 | passed |
| 2026-07-14 | Sell equipment and a partial consumable quantity | Exact checked quotes; correct removed/updated inventory diffs | passed |
| 2026-07-14 | Reject stale, bound/equipped, filtered, insufficient, overflow, and arbitrage cases | Same evaluator reason and zero mutation | passed |
| 2026-07-14 | Parse Rust Shop fixture in Unity and mutate root snapshots | Full offer/quote rendering, authoritative commands, stale selection reconciliation | passed |
| 2026-07-14 | Search live/catalog/runtime/client for Artifact and prototype Shop schema | No Shop dependency remains for G27 | passed |

## Completion Evidence

- Core schema validation composes typed preset/local merchandise, rejects duplicates and missing or
  invalid entries, checks unit and total quotes, and enforces strict same/cross-Shop loss on resale.
- Deterministic runtime coverage proves repeatable exact entry, stable offer identity/order, atomic
  purchase/refresh/sale, exhausted-pool zero mutation, partial stack sale, and exact checkpoint
  restoration of the cached Shop session.
- Live Headquarters and Unknown profiles contain only current priced equipment and consumables. The
  Artifact, zero-price placeholder, and prototype fixed-page searches return no production hits.
- The Rust Shop fixture regenerates byte-exactly into Unity. Unity parses and renders typed offers,
  exact quotes and reasons, sends offer/owned identities with explicit quantities, and reconciles
  stale selections from each authoritative snapshot.
- Final executable boundary: Core 724 library tests plus 44 binary/integration tests (768 total),
  game_server 30, Unity EditMode 220, Unity PlayMode 52 passed plus 4 explicit external-server skips,
  zero failures, and cleared Unity console zero errors.
- `cargo fmt --all --check`, Core scoped `git diff --check`, Unity Shop-scope whitespace checks,
  fixture drift comparison, current-source legacy vocabulary audit, and live Shop Artifact/content
  audit all pass. The only `reroll_shop`/`can_reroll` hits are deliberate negative schema tests.
