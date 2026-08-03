# Inventory Reward Capacity Contract Notes

Goal: G24

Status: complete

## Accepted Policy

- Ordinary inventory is logically infinite.
- `reserved_slot_count` starts at 50, grows in sufficient 10-slot increments only for a successful
  new occupied slot, and never shrinks.
- Equipment is instance-owned; consumables are definition-ID quantity stacks.
- The consumable metadata UUID remains a stable definition selector on the wire. There is no owned
  consumable identity or per-copy UUID stream after the cutover.
- Artifact is not ordinary inventory and is not modified by this goal.

## Implementation Guardrails

- Do not use Rust collection capacity as gameplay or presentation state.
- Do not retain `with_max_slots`, fixed 20/30 limits, ordinary capacity preflight, deferred grants,
  overflow bags, replacement prompts, or a compatibility DTO.
- Do not use serde aliases/defaults or Unity `count` fallback to accept the removed consumable shape.
- Quantity arithmetic is checked; zero additions/removals and overflow fail without mutation.
- Diff entries carry full post-mutation stack state. Zero-quantity stacks do not exist.
- Root snapshot ordering is deterministic by equipment instance UUID and consumable definition ID.
- Artifact's existing cap/error/ownership behavior remains intact until G27.

## Current Trace

- Storage authority: `src/game/resources/inventory.rs`.
- Shared grant authority: `src/game/reward.rs`.
- Event/admin/combat/reward atomicity: cloned `Inventory` staging in their existing transaction paths.
- Checkpoint/retreat persistence: cloned `Inventory` in `RunCheckpointPayload` and
  `BattleEntrySnapshot`.
- Root projection: `src/game/world/snapshot.rs` and `InventorySnapshotDto` in
  `src/game/behavior.rs`.
- Existing consumable use: `world/helpers.rs` plus `world/maintenance.rs`.
- Legacy Shop adapter: `world/shop.rs`; G24 must consume the shared stack transaction but G28 owns
  final offer/sale identity, quantity, price, refresh, and session policy.
- Unity reader/presentation: `InventoryModel.cs`, `InventoryBagComposer.cs`, mock snapshots, and
  contract tests.

## Completion Boundary

- G24 is Complete / High on 2026-07-14.
- The ordinary inventory source of truth is the Core `Inventory`; no grant caller or Unity presenter
  owns a second capacity or consumable-stack model.
- G28 may now consume equipment-instance and definition-stack transactions for purchase, sale,
  quotes, refresh, and reconnect-stable Shop sessions. G28 owns the final Shop contract and must not
  reintroduce per-copy consumables, fixed slot admission, or a Shop-local inventory adapter.
- G27 remains responsible for deleting Artifact after G28 removes its Shop/content dependencies.
- No correction goal or follow-up refactor was identified by the G24 completion review.
