# Core Enhanced Refactoring Audit Phase 03 — Experiment Notes

Status: complete

## Boundary decision

- Authority family: employee roster/condition/loadout, inventory/equipment/consumable/material, skill fragments,
  growth/research, currency and direct grant/reward/shop/recruitment/maintenance transactions.
- Direct predecessor: Phase 2 `Complete / High`.
- Published typed data is a confirmed input; Phase 1 loading candidates remain separate.
- Run/checkpoint lifetime is a confirmed carrier; Phase 2 candidates remain separate.
- Event/VN sessions, battle internals and command/DTO projection retain later-Phase ownership.

## Source-of-truth distinctions

- A definition catalog authorizes types/values; run-local roster/item/fragment ledgers own player assets.
- Result/preview DTOs are derived consumers unless mutation code proves otherwise.
- Shared grant helpers are not automatically authority until actual cost/validation/commit callers are traced.
- Checkpoint round trip proves carriage, not economic correctness.
- Goal/history claims do not override current mutation and live data.

## Dirty worktree precautions

- Repository baseline remains start commit `f318414fb0156a67e04c5479000c2a2683fc0c09` plus current dirty tree.
- Employee/equipment/data/world/test/canonical/live-data files already contain user changes.
- Only Master/Phase audit artifacts are audit-owned.
- Modified production files require content hashes, not status-set comparison alone, to prove no mutation.

## External systems and consumers

- `game_resources` live economy catalogs are confirmed supporting inputs and may be read.
- Direct server transport may be read only where it consumes a Core result; server economy refactors are excluded.
- Unity economy/UI is explicitly out-of-scope/unverified.
- Other public Rust/save consumers remain unknown unless current workspace/canonical proves them.

## Initial rejected assumptions

- Do not call authored price/reward data the owned-asset ledger.
- Do not merge shop/reward/recruitment sessions solely because they grant the same item types.
- Do not classify all clones/staging as duplication before success/failure atomicity is traced.
- Do not rebalance or add recipes/rewards while auditing missing coverage.
- Do not treat successful checkpoint serialization as proof of exact ownership validation.

## Activation checklist — completed

1. Re-read this Phase's three required documents and Phase 2 completion/handoff.
2. Freeze scoped dirty/content baseline for roster/inventory/economy and direct live-data/test paths.
3. Inventory definition→instance→ledger→transaction→checkpoint/result paths.
4. Record complete candidate/disposition artifacts and independent completion audit.

## Investigation conclusions

- Root authority is split by asset family, not by command: roster, inventory, fragment inventory, UUID manager and
  Enkephalin remain the persistent grant ledgers; recruitment and abnormality research are run-owned lifecycle
  state.
- Definition IDs, employee UUIDs, equipment instance UUIDs and static consumable UUID selectors are intentionally
  different identities and must not be normalized into one generic key.
- Normal equip/death/dismantle paths currently preserve both employee slot and inventory `equipped_to`, but the
  two-sided mutation is repeated and current V3 restore does not validate their graph.
- Every current Core `GrantExecutor` caller stages all five ledgers. The executor is therefore not a current
  partial-commit defect, though its repeated caller boilerplate is a refactor candidate.
- Shop purchase/sale/refresh, reward claim, event grant, combat result and pending research use staged commits in
  the audited paths.
- Headquarters has a ten-field handwritten rollback helper despite an existing full non-battle transaction.
- Enhancement is the confirmed command atomicity defect: duplicate same-material recipe rows pass per-row
  validation and can consume the first row from live inventory before the second fails.
- Current live enhancement content uses only one material row, so impact frequency is content-dependent and not
  used to lower the correctness classification.

## Candidate decisions

| Candidate | Work type | Disposition | Readiness |
| --- | --- | --- | --- |
| CRA-P03-C01 checkpoint economy graph validation | `correctness_correction` | priority candidate | test first; characterize fragment predicates |
| CRA-P03-C02 atomic enhancement material cost | `correctness_correction` | priority candidate | test first |
| CRA-P03-C03 equipment/loadout transition authority | `behavior_preserving_refactor` | candidate | transition characterization first; after C01 |
| CRA-P03-C04 typed five-ledger grant transaction | `behavior_preserving_refactor` | candidate | caller parity characterization first |
| CRA-P03-C05 headquarters rollback consolidation | `behavior_preserving_refactor` | candidate | full-state rollback parity first |

Change frequency is `unknown` for all five candidates because the current dirty tree is not reliable history.
Priority and readiness remain separate.

## Policy and compatibility boundaries retained

- Checkpoint internal self-consistency can be rejected under the current strict V3 contract; rebinding embedded
  metadata to latest live content or repairing old saves cannot.
- C02 must prevent partial mutation. Rejecting duplicate cost rows versus aggregating them is an implementation
  choice only if both preserve current single-row live behavior and explicit data errors.
- Removing one serialized equipment binding, changing reward/shop/recipe values or deleting public grant/inventory
  APIs requires migration/consumer evidence.
- Unity and other external Rust consumers remain unverified.
- Event/VN session lifecycle, battle calculations/effects/death internals and DTO shape remain later Phases.

## Focused evidence summary

- Retained evidence: 18 runner invocations, 201 pass, 0 fail, 0 ignored.
- Physical executions: 26 invocations, 291 tests, because an eight-filter/90-test batch was rerun after its
  aggregate output was truncated.
- Live counts: employee 12, equipment 88, consumable 8, reward 13, skill fragment 31, shop context 3;
  material/dismantle/enhancement recipe 1 each.
- No focused `#[ignore]` was found. Filtered runner counts are nonmatching tests, not skips.

## Phase handoff

- Final should consider a shared checkpoint validation entry point for Phase 2 C02 and Phase 3 C01 without
  collapsing their separate domain predicates.
- Phase 4 may consume the established atomic asset-grant handoff, but must independently audit session identity,
  replay/idempotence and VN/Event/shop lifecycle.
- Battle Phases inherit only the finding that setup trusts the persistent equipment/fragment graph; formulas and
  active-battle mutation were not verified here.
- Phase 10 treats roster/inventory snapshots as derived consumers and does not promote either projection to
  authority.

## Completion posture

Boundary/candidate discovery, final production-preservation comparison and P3-01~P3-10 independent completion
audit are complete with verdict `Complete / High`. Parent registry reconciliation and sequential Phase 4
activation are now allowed.
