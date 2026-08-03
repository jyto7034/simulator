# Inventory Reward Capacity Contract Experiments

Goal: G24

Updated: 2026-07-14

## Entry Evidence

- `EquipmentInventory` stores owned instances in a UUID-keyed `HashMap` capped at 20.
- `ConsumableInventory` stores every copy as a separate owned UUID in a UUID-keyed `HashMap` capped
  at 30.
- `GrantExecutor::grant_inventory_item` preflights `Inventory::can_add_item`, allocates a new owned
  consumable UUID, and returns `InventoryFull` when either ordinary sub-inventory reaches its cap.
- `InventorySnapshotDto` has neither reserve/occupancy nor consumable quantity.
- Consumable use removes a whole owned instance and reports that UUID in `inventory_diff.removed`.
- Reward, Event, admin, combat-result, and checkpoint paths already clone/stage the `Inventory`, so
  placing reserve and stacks inside that authority preserves their atomic commit/rollback boundary.
- Unity's `ConsumableView` tolerates an optional `count` with fallback 1, while the current Core never
  sends it.
- `InventoryBagComposer` locally truncates/fills every category to 40 slots, independently of Core.
- Artifact owns a separate capped `ArtifactSlots` collection and is ignored by the normal Unity bag.

## Dirty Baseline

- Core/server repository HEAD: `c2c5dc5ec54bd26530e46f0d7ee7909de3ed7a1f`.
- Core status fingerprint: `ab69ece9d38301761471f03d5f8902f949bd9ff2bc48fb80220ca5d64ca70013`.
- game_server status fingerprint: `17622ec8e9ed5b853c21973ffb703135ce03941f662d6ef3302f467564baf6b7`.
- Unity HEAD: `7cd8df1f5ae2e438e28bda2c761e7a56d21e59cf`.
- Unity status fingerprint: `d4cbf7fe310ed5a6d2eb3337c344bc75b4963519f0caa944b70410bd9908daee`.
- Preserve all pre-existing dirty changes. G24 owns only the inventory/reward/checkpoint/Unity seams,
  its canonical documentation, tests, and goal/master evidence.

## Planned Experiments

| Date | Experiment | Expected Evidence | Status |
|---|---|---|---|
| 2026-07-14 | Add equipment through occupied slots 50, 51, and beyond 60 | Reserve is 50, then 60, then sufficient 10-slot high-water value; no capacity error | passed |
| 2026-07-14 | Add repeated copies of one consumable definition | One stack quantity changes; occupancy/reserve do not change after first copy | passed |
| 2026-07-14 | Remove ordinary items after reserve growth | Occupancy falls and reserve does not shrink | passed |
| 2026-07-14 | Fail duplicate UUID / quantity overflow in a staged transaction | Inventory, quantity, reserve, UUID continuation, and other staged effects remain unchanged | passed |
| 2026-07-14 | Save/load checkpoint and battle-entry rollback after growth | Exact quantity and high-water reserve restore from the owning snapshot | passed |
| 2026-07-14 | Parse Core-shaped inventory in Unity | Core reserve controls slot rendering and explicit `quantity` controls the badge | passed |
| 2026-07-14 | Add/remove Artifact alongside ordinary items | Ordinary occupancy/reserve remain unchanged | passed |

## Implementation Evidence

- `Inventory` now owns persisted `reserved_slot_count`; ordinary occupancy is equipment instance
  count plus distinct consumable definition-stack count. Successful new-slot mutations grow the
  reserve from 50 in sufficient 10-slot increments, and removals/quantity-only changes never shrink
  or grow it.
- Equipment retains owned instance UUID identity. Consumables use one checked positive `u32`
  quantity stack per definition ID and retain the metadata UUID as the stable wire selector. The
  removed owned-consumable UUID stream and fixed equipment/consumable limits have no compatibility
  path.
- The typed reward effect requires `quantity`. The shared staged grant executor emits the first
  stack in `added` and repeat grants in `updated` with full post-commit quantity. Event, admin,
  combat-result, and legacy Shop callers consume the same mutation authority.
- Item use decrements exactly one unit, producing an updated positive stack or a removed static
  definition UUID at zero. Snapshot ordering is deterministic and exposes reserve, occupancy, and
  explicit quantities.
- Run checkpoints and battle-entry snapshots clone the owning inventory state. Focused tests prove
  exact reserve/quantity restoration after checkpoint load and whole-stage abandonment.
- Unity reads required `quantity`, filters the removed `count` shape, renders the Core reserve in
  normal and Maintenance bags, and uses the metadata UUID selector. A Rust-generated fixture proves
  the exact external shape and 60-slot presentation.
- Artifact remains in its pre-G24 isolated capped collection, is excluded from ordinary occupancy
  and reserve, and is intentionally retained for G27 removal.

## Validation Results

- Focused Core inventory, reward diff, Event, admin, checkpoint, battle-abandonment, item-use, serde,
  and atomicity tests passed. The final added/repeat reward-stack diff test passed 1/1.
- `CARGO_TARGET_DIR=/tmp/core_g09_target cargo test --all-targets -q`: 761 passed, 0 failed
  (717 library plus 44 other target tests).
- game_server all-target suite: 30 passed, 0 failed.
- Unity EditMode: 217 passed, 0 failed. Unity PlayMode: 52 passed, 0 failed, with 4 explicit
  external-server skips. The cleared Unity console contained 0 errors.
- The Rust fixture generator output exactly matched
  `Assets/Tests/Fixtures/inventory_capacity_contract.json`.
- `cargo fmt --all -- --check`, scoped Core/server and Unity `git diff --check`, and legacy/fallback
  searches passed. Remaining `with_max_slots`/`InventoryFull` inventory matches are Artifact-only;
  board/roster capacity matches are unrelated domain limits.
