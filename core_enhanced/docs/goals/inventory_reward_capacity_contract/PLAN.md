# Inventory Reward Capacity Contract

Goal: G24

Finding: DCA-M-30

Status: complete

Implementation complete: true

Updated: 2026-07-14

## Objective

Replace fixed ordinary-item admission caps and per-copy consumable identity with one authoritative,
logically unbounded ordinary inventory contract shared by grants, checkpoints, snapshots, and Unity.

## Fixed Boundary

- Equipment remains instance-owned and keyed by its owned instance UUID.
- Identical consumables form one stack keyed by `definition_id`; quantity is explicit and has no
  stack cap.
- A consumable's static metadata UUID remains its wire selector. It identifies the definition-backed
  stack and is never an owned-copy UUID.
- Ordinary occupied slots equal equipment instance count plus distinct consumable stack count.
- Persisted logical `reserved_slot_count` starts at 50, grows by the minimum number of 10-slot
  increments needed after a successful new-slot addition, and never shrinks.
- Quantity-only stack changes do not grow the reserve.
- The reserve is presentation high-water state, not collection capacity, overflow storage, or a
  grant eligibility rule.
- Equipment materials, skill fragments, fragment dust, and currencies remain non-slot resources.
- Artifact remains the isolated legacy subsystem. G24 does not count, stack, migrate, or otherwise
  adapt it; G27 removes it after G28 removes Shop dependencies.

## Transaction Contract

- Ordinary additions cannot return capacity-based `InventoryFull`.
- Duplicate equipment instance UUID and zero/overflowing consumable quantity remain invalid
  mutations and must fail before inventory or reserve changes.
- A first consumable definition grant produces an added stack; later grants produce an updated stack
  with its full post-commit quantity.
- Consuming one unit produces either an updated positive stack or an explicit removed definition
  identity when the quantity reaches zero.
- Reward, Event, admin, combat-result, and Shop staging clone the same inventory state. Reserve and
  quantities commit only with their containing transaction.
- Run checkpoint and battle-entry snapshots retain the cloned reserve and stack quantity exactly.

## Cross-Layer Projection

- Root inventory snapshot exposes `reserved_slot_count`, `occupied_slot_count`, equipment instances,
  and definition-backed consumable stacks with `quantity`.
- Inventory diffs use the same consumable stack DTO. A removed consumable uses its static definition
  UUID while a removed equipment uses its owned instance UUID; cross-registry UUID uniqueness keeps
  those selectors unambiguous.
- Unity renders the Core-projected reserve rather than a local fixed slot count, displays stack
  quantity, and sends the definition's static item UUID for item use.
- No legacy `count` fallback, per-copy consumable UUID reader, fixed 20/30 admission limit, or Unity
  fixed 40-slot authority remains.

## Validation

- Focused inventory tests for 50/10 growth, non-shrink, quantity-only stability, unbounded equipment,
  stack overflow atomicity, and Artifact isolation.
- Focused reward/Event/admin/checkpoint/item-use tests for shared quantities and rollback.
- Core snapshot/serde assertions and exact Rust-to-Unity fixture/read-model tests.
- Full Core, game_server, Unity EditMode, and Unity PlayMode suites plus strict legacy searches,
  formatting, whitespace, and console audit.
- Independent completion review after implementation validation.

## Exit Gate

Complete / High only when ordinary grants cannot fail because of occupied slots, every mutation path
uses the same definition-stack/equipment-instance authority, reserve high-water semantics survive
rollback/checkpoint projection, Unity has no fixed local slot authority, and Artifact is unchanged and
excluded from ordinary occupancy.

Exit result: Complete / High on 2026-07-14. Runtime, snapshot, reward/Event/admin/Shop adapters,
checkpoint and battle-entry rollback, Rust-to-Unity fixture, Unity read model/presentation, canonical
contracts, focused tests, and full Core/server/Unity suites satisfy the fixed boundary. Artifact remains
isolated and unchanged for G27.
