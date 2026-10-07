# Phase 22 CRA-P03-C03 Experiment Notes

Status: complete
Implementation Complete: true
Candidate: CRA-P03-C03
Master Phase: 22 of 44
Date: 2026-08-10 (`Asia/Seoul`)

## Locked Decisions

- CRA-POL-006: both current public/serialized ledgers and all public/V3/RON/DTO/wire behavior remain exact.
- F-C01 remains imported graph admission authority; invalid state is rejected, never repaired or rebound.
- Consolidate only paired employee-slot and owned-equipment reverse-binding mutation.
- Preserve identity, slot/type/bound rules, enhancement, outcomes/errors/order and outer transaction behavior.
- Move/replace/unequip/starter/death/dismantle require exact full two-ledger characterization first.
- Phase 23 grant aggregation and every later refactor/cleanup are prohibited from this Child.

## Open Evidence Slots

- N02: activation expanded status, prospective target fingerprints and scoped dirty attribution.
- N03: current two-ledger mutation/caller/staging graph and exact transition table.
- N04: public/V3/RON/DTO/server/Unity/checkpoint/battle consumer compatibility inventory.
- N05: pre-refactor valid/invalid transition characterization and F-C01 post-state proof.
- N06: implemented typed transition authority, transferred callers and obsolete direct writes.
- N07: final focused/broad/static/canonical evidence and independent-review handoff.

## 2026-08-10 — N01 Policy, Dependency And Activation Boundary

- Source/evidence: CRA-POL-006; original/re-audited P03-C03 evidence; Final order/test/gates; F-C01 and Phase 21
  completion audits; Master readiness transition.
- Observation: employee slots and inventory `equipped_to` persist one association in two ledgers. Existing normal
  paths pass, but several lifecycle/staging owners historically synchronize the pair manually.
- Observation: private typed encapsulation is approved only with exact transition proof. Removing either field,
  narrowing public compatibility, repairing invalid input or changing gameplay is not approved.
- Impact: inspect no production/test/data/canonical content until all Child documents and the final Phase 21 audit
  are fully read and the expanded dirty baseline plus prospective hashes are captured.
- Disposition: activate Phase 22 only; keep Phase 23 and all later work pending.

## 2026-08-10 — N02 Activation Dirty Baseline

- Source/evidence: expanded status, scoped status and SHA-256 fingerprints after mandatory reads and before
  production/test/data/canonical content inspection.
- Observation: HEAD is `f318414fb0156a67e04c5479000c2a2683fc0c09`; expanded status is 479 entries at
  normalized SHA-256 `6accb3ce4b7b47d14c323e8b87e551c3b2743084d341c315d2f0a331ed3a17f8`.
- Core domain hashes: employee `8d3940dc...`, inventory `2b32a864...`, item slot `c20b7a68...`, resources module
  `f9c7bb3e...`, resource state `1a6efb5b...`, active run `bea18fa6...`, helpers `51b104af...`, maintenance
  `9455eaa7...`, combat `0f9d6af0...`, world state `1e5f9177...`, snapshot `6e512c4d...` and admin fixtures
  `c319893c...`.
- Test hashes: equipment `1ed57f19...`, snapshots/start `7885a26f...`, combat `933df538...` and checkpoint
  validation `6c269025...`.
- Canonical hashes: runtime `6e20d6a6...`, rulebook `6c53ecd5...`, server bridge `4d734af0...`, external Unity
  `a3224e24...`.
- Observation: employee, inventory, item slot, active run, helpers, maintenance, combat, world state, snapshot,
  admin fixtures, all four prospective tests except no separate resource test, and three workspace canonicals are
  tracked-modified; checkpoint validation is untracked. Resource module/state files are currently clean. Existing
  overlapping content predates Phase 22 until scoped attribution proves otherwise.
- Impact: edits must be minimal and hash-tracked. No reset, restore, clean, stash, broad format, schema/API change or
  adjacent grant cleanup is permitted.
- Disposition: baseline gate complete; current production/test/data/canonical inspection may begin.

## 2026-08-10 — N03 Current Mutation, Consumer And Transition Table

- Safezone equip: context, unbound item, target/source eligibility, current reverse/slot pairing, target same-type
  displacement, slot layout/duplicate rule and projected active-fragment compatibility are validated. Success moves
  the source, clears the displaced item, installs the incoming item and returns updated UUIDs `[incoming,
  displaced?]` before one outer commit.
- Safezone unequip: exact reverse owner, unbound metadata, target eligibility and slot membership are required.
  Success clears the target slot and reverse binding and returns exactly the requested item as updated.
- Starter: candidate placeholder slots validate authored layout before grants. Ordered granted owned UUIDs are paired
  to the rebuilt roster in assignment order; definition/instance/employee identities remain distinct.
- Death: only newly persistent-dead employees are processed in staged combat-result roots. Item-slot iteration order
  determines the exact updated-item order; paired equipment is returned, shared fragment/modifier cleanup follows,
  and live state stays untouched until the enclosing result commit.
- Dismantle: maintenance context, unbound metadata, recipe and grant preview precede optional paired unequip plus
  owned-instance removal. Existing material grant/result construction remains separate and exact.
- Import/consumer: F-C01 validates the full two-ledger graph before Phase 20 restore. Snapshot, battle setup, event
  checks, maintenance preview, server and Unity read the unchanged projections and own no mutation.
- Disposition: these five operation rows are the complete production mutation set. Fixtures in embedded test
  modules and data-validation scratch slots are not production lifecycle alternatives.

## 2026-08-10 — N04 Compatibility And Consumer Inventory

- `EmployeeLoadout.item_slot`, `OwnedEquipment.equipped_to`, `Inventory.equipments`, the public equipment inventory
  mutable methods and public `ItemSlot` mutation methods retain exact names, visibility, types and serde behavior.
- Strict V3 continues storing both ledgers. F-C01 rejects missing, duplicate, wrong-owner and identity/type mismatch
  edges before restore; no transition repairs imported state or selects a preferred ledger.
- Equip/unequip/dismantle request/result DTOs, `InventoryDiffDto`, snapshot `equipped_items`/`equipped_to`, update
  ordering and existing errors are unchanged. Enhancement remains on the same owned instance.
- Server source has no internal equipment-ledger consumer; it transports Core results/snapshots. The external Unity
  canonical consumes Core inventory/roster projections and never receives the private transition type.
- Existing canonicals already state atomic final-state equip/move/replace, bound and fragment rules, dismantle
  eligibility, death roster retention and strict V3. Private owner movement requires no wording change.
- Disposition: CRA-POL-006 is satisfied without migration, fallback, public narrowing or coordinated rollout.

## 2026-08-10 — N05 Pre-Refactor Characterization

- Baselines: equipment 30/30, snapshots/start 23/23, inventory 5/5, item slot 4/4 and checkpoint validation 6/6.
- Empty type-slot equip proves exact single updated UUID, final slot/reverse pairing and displaced starter remaining
  unbound. Replacement and cross-employee move prove exact `[incoming, displaced]` order and both source/target
  final slots.
- Same-target, bound unequip and active-fragment-incompatible replacement compare full serialized roster/inventory
  before/after and preserve the exact existing error with no mutation.
- Unequip, starter, staged death recovery and equipped dismantle prove exact final slots/reverse bindings, result
  ordering/removal/materials and F-C01 graph validity.
- All strengthened behavior assertions passed on the old implementation before production mutation. The added
  post-boundary mismatched death reverse-binding case further proves aggregate preflight leaves both staged roots
  unchanged.
- Disposition: P22-C03 complete; tests protect public behavior and the cross-ledger invariant, not helper layout.

## 2026-08-10 — N06 Implemented Authority And Caller Transfer

- `resources/equipment_loadout.rs` contains the sole crate-private `EquipmentLoadoutTransition`, typed equip plan/
  outcome and starter assignment. Each operation works on both roots and installs cloned next values only after all
  fallible paired edits succeed.
- `helpers.rs` delegates ledger admission planning and starter batch assignment; projected active-fragment
  validation stays in GameCore because it also needs the fragment catalog/inventory.
- `maintenance.rs` delegates staged equip/unequip and dismantle removal. `combat.rs` delegates per-dead-employee
  forced clear while retaining shared-fragment/modifier and outer result orchestration.
- Production search finds all reverse-binding writes in the new authority. No paired slot/reverse write remains in
  those callers. Test-only fixture construction and static local layout validation remain intentionally separate.
- Existing public low-level mutation APIs remain for compatibility, but no current production lifecycle uses them
  to synchronize only one ledger.
- Grant application, starter grant effects and dismantle material grant remain outside this two-ledger aggregate;
  no Phase 23 five-ledger transaction type or placeholder was introduced.
- Disposition: P22-C04–P22-C08 complete.

## 2026-08-10 — N07 Final Validation And Review Handoff

- Focused totals: equipment 31, snapshots/start 23, facility 11, inventory 5, item slot 4 and checkpoint validation
  6; 80/80 passed with zero failed/ignored.
- Broad totals: Core debug/release each 999 unit + 51 integrations; server debug/release each 59. Four debug/release
  checks, formatter and repository diff check pass.
- Clippy remains diagnostic-only due unchanged global debt: Core lib 83/lib-test 136 and server lib 3/lib-test 7.
  No diagnostic names the new transition source or Phase 22 caller/test edits.
- Final hashes: transition `e76eff46...`, resource module `846a2da2...`, helpers `e48b02d9...`, maintenance
  `faa07e01...`, combat `ef896bfa...`, equipment tests `550706db...`, snapshots/start `2144e86e...`.
- Inventory `2b32a864...`, item slot `c20b7a68...`, employee `8d3940dc...`, checkpoint test `6c269025...` and all
  four canonicals remain byte-identical to activation.
- Activation status was 479 entries at `6accb3ce...`; pre-audit is 481 at `d605fca5...`. Only clean resource
  registration plus the new transition file add rows; all pre-existing dirty work remains present.
- Disposition: P22-C01–P22-C09 complete; perform the independent phase-exit audit for P22-C10.

## 2026-08-10 — N08 Completion Audit Result

- Independent review found no material runtime, data, persistence, public API, consumer, canonical, dirty-scope or
  validation issue. Both ledgers and every compatibility surface remain exact.
- Audit-focused reruns are 80/80. Broad current evidence remains Core 999 + 51 twice and server 59 twice, with all
  required check/fmt/diff gates green.
- `COMPLETION_AUDIT.md` is 299 lines after final status-hash insertion and returns `Complete / High`, improvement
  class `none`.
- Audit-inclusive dirty status is 482 entries at normalized SHA-256 `70a9c09f...`. P22-C01–P22-C10 are complete.
