# Inventory Reward Capacity Contract Completion Review

Review date: 2026-07-14

Review procedure: `docs/goal_completion_review_guide.md`

Primary verdict: Complete

Long-term fit: High

Improvement class: none

Immediate correction: none

## Item Audit

| Finding | Integrated result | Verdict | Fit | Correction |
|---|---|---|---|---|
| DCA-M-30 fixed ordinary capacity can make grants impossible | ordinary equipment and consumable additions are logically unbounded; reserve is presentation high-water state | Complete | High | None |
| PD-15 consumable identity and quantity | equipment remains instance-owned; consumables are definition-ID quantity stacks with a static metadata UUID selector | Complete | High | None |
| G06 concrete inventory capability boundary | item-use identity and eligibility consume the authoritative inventory representation without a local capacity rule | Complete | High | None |

## Runtime And Transaction Review

- `src/game/resources/inventory.rs` owns equipment instances, definition-backed consumable stacks,
  ordinary occupancy, and persisted reserve high-water state. Checked mutations reject duplicate
  equipment identity, zero quantity, and quantity overflow before changing inventory or reserve.
- Fixed equipment/consumable admission caps and owned-consumable UUID allocation are removed.
  Ordinary additions do not return capacity-based `InventoryFull`; the retained Artifact-only error
  is outside the accepted ordinary boundary.
- `src/game/reward.rs` requires explicit positive consumable quantity and uses the same inventory
  mutation authority. First grant is an added stack; later grants are updated full post-state.
- Event, admin, combat result, item use, and the legacy Shop adapter compose the shared transaction.
  No caller maintains a second quantity or capacity source of truth.
- Checkpoint and battle-entry rollback clone the whole inventory, so quantity and reserve commit or
  restore with the containing gameplay transaction.

## External Contract Review

- Root snapshots expose `reserved_slot_count`, `occupied_slot_count`, equipment instances, and
  consumable stacks with required `quantity`. Deterministic sorting preserves stable projection.
- A consumed final unit reports the consumable metadata UUID in `removed`; a partial consumption
  reports the full positive stack in `updated`. Equipment removal continues to use owned instance
  UUID identity.
- Unity `InventoryModel` reads the Core reserve/occupancy and required quantity, filters the removed
  `count` shape, and retains the static consumable UUID for commands.
- Both general inventory and Maintenance presentation use the Core reserve. No fixed 40-slot client
  authority remains.
- The Rust generator and Unity fixture match exactly and exercise reserve 60, occupancy 2, one
  equipment instance, and one consumable stack at quantity 7.

## Canonical And Boundary Review

- The rulebook, Core runtime/loading contracts, server bridge contract, and Unity contract agree on
  logically unbounded ordinary inventory, 50/10 nonshrinking reserve semantics, explicit stack
  quantity, and equipment/static-consumable identity.
- Materials, fragments, dust, and currencies remain non-slot resources.
- Artifact was not normalized into ordinary inventory. Its legacy collection and behavior remain
  isolated, unchanged, and excluded from occupancy/reserve until G27 deletes the subsystem after
  G28 removes Shop dependencies.

## Validation Review

- Focused tests cover 50/60/70 reserve growth, non-shrink, quantity-only stability, more than former
  fixed caps, overflow/duplicate atomicity, added-versus-updated reward diffs, Event/admin grants,
  item use, checkpoint restore, battle-abandonment restore, and Artifact exclusion.
- Core all-targets: 761 passed, 0 failed (717 library plus 44 other targets).
- game_server all-targets: 30 passed, 0 failed.
- Unity EditMode: 217 passed, 0 failed. Unity PlayMode: 52 passed, 0 failed, 4 explicit
  external-server skips. Cleared Unity console: 0 errors.
- Formatting, generated-fixture drift, scoped whitespace, and legacy/fallback searches passed.

## Legacy And Fallback Audit

- No `OwnedConsumable`, owned-consumable UUID stream, ordinary `with_max_slots`, fixed 20/30
  sub-inventory cap, Unity default 40 slots, 4x10 assumption, or `count` fallback remains.
- Artifact-only `with_max_slots` and `InventoryFull` are intentionally retained for G27. Board and
  roster capacity fields are unrelated domain limits.
- No compatibility alias, dual DTO, overflow bag, replacement prompt, or deferred-grant mechanism
  was introduced.

## Cross-Component Findings

- None. Runtime storage, staged grants, persistence, root projection, server pass-through, Unity
  reading, and presentation share one identity/quantity/reserve contract.
- G28 must preserve this boundary while replacing the legacy Shop contract. G27 must then remove
  Artifact without adapting it into ordinary inventory.

## Long-Term Direction Review

- Fit: High.
- Improvement class: none.
- Reason: the change removes two hard caps and a per-copy UUID stream, centralizes mutations in one
  persisted authority, gives Unity explicit presentation state without making it gameplay capacity,
  and leaves a clean typed quantity boundary for future merchandise and reward content.

## Review Conclusion

G24 is Complete / High with no immediate correction. G28 may begin.
