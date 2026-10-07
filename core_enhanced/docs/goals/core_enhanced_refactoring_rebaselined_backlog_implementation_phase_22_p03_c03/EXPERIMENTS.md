# Phase 22 CRA-P03-C03 Experiments

Status: complete
Implementation Complete: true
Candidate: CRA-P03-C03
Master Phase: 22 of 44
Date: 2026-08-10 (`Asia/Seoul`)

## Experiment Protocol

- Execute only while Phase 22 is the sole active Master row.
- Preserve both ledgers, public/V3/RON/DTO/wire shape and current behavior under CRA-POL-006.
- Keep F-C01 as strict imported graph admission; do not repair or choose a ledger.
- Read all Child documents and the complete final Phase 21 audit before production inspection.
- Record expanded dirty status and prospective target fingerprints before source/test/data/canonical inspection or
  edits.
- Add the exact two-ledger transition matrix before behavior-preserving changes.
- Do not prebuild Phase 23 grant transaction work.
- Complete only after an independent `Complete / High` phase-exit audit.

## 2026-08-10 — E01 Activation And Policy Boundary

### Inputs Reviewed Before Child Initialization

- Phase 21's final 312-line audit is `Complete / High`; Master made Phase 22 ready only after that exit.
- F-C01 already rejects contradictory roster/inventory/loadout graphs before restore.
- CRA-POL-006 preserves strict public/V3/RON/DTO/wire compatibility and excludes field deletion, API narrowing,
  repair and migration.
- Original and Boss-era evidence classifies separate slot/reverse-binding mutation as encapsulation debt, not a
  confirmed current half-commit defect.
- Final order requires one exact both-ledger transition matrix covering move, replace, unequip, starter, death and
  dismantle before consolidation.

### Activation Boundary

- Required Child documents were created and Master Phase 22 was registered as the sole active row.
- Phase 23 and later work remain pending and unauthorized.
- No Phase 22 production source, test, data or canonical was inspected or mutated during activation.

### Next Experiment

Fully read `PLAN.md`, this file and `EXPERIMENT_NOTES.md`, then fully reread the final 312-line Phase 21 audit.
Only afterward capture dirty baseline/fingerprints and inspect the current two-ledger mutation graph.

## 2026-08-10 — E02 Mandatory Reads And Dirty Baseline

### Read Gate

- `PLAN.md` 195/195 lines, `EXPERIMENTS.md` 43/43 activation lines and `EXPERIMENT_NOTES.md` 37/37 activation
  lines were fully read after Master registered Phase 22 as the sole active Child.
- Phase 21's final `COMPLETION_AUDIT.md` was then reread in full, 312/312 lines. Its exact public/persistence and
  strict linear handoff is a required boundary, not authorization to revisit Phase 21.

### Activation Baseline

- HEAD is `f318414fb0156a67e04c5479000c2a2683fc0c09`.
- Expanded status contains 479 entries with normalized SHA-256
  `6accb3ce4b7b47d14c323e8b87e551c3b2743084d341c315d2f0a331ed3a17f8`.
- Prospective target hashes and dirty classifications are recorded in N02. Most likely production/test/canonical
  paths already overlap the dirty campaign; resource module/state files are currently clean but fingerprinted.
- No Phase 22 production/test/data/canonical content was read before this baseline was captured.

### Next Experiment

Read the complete current roster/inventory/loadout mutation and consumer graph, build the exact transition/staging
table and run focused nonzero baselines before adding characterization.

## 2026-08-10 — E03 Current Graph, Compatibility And Test-First Characterization

### Current Mutation Graph

- `handle_equip_item` validated the live two-ledger graph, cloned roster/inventory, then manually removed the source
  slot, cleared a displaced target reverse binding, installed the target slot and wrote the incoming reverse binding.
- `handle_unequip_item` validated first but then directly cleared the live employee slot and inventory binding.
- Starter setup validated placeholder slots, granted equipment, replaced the roster and then paired each real owned
  UUID with one employee through repeated direct slot/reverse writes.
- Combat-result death recovery already received staged roster/inventory roots, but cleared a dead employee's slots
  before iterating and clearing each reverse binding. Equipped dismantle similarly cleared the slot and removed the
  owned instance before applying the existing material grant.
- F-C01 calls `Inventory::validate_checkpoint_state` before restore and already proves exact instance/base/type/
  employee ownership, uniqueness and reverse-binding agreement. Snapshots and battle/event consumers read both
  ledgers but own no mutation.

### Baseline And Characterization

- Pre-edit baselines were green with nonzero counts: equipment 30, snapshots/start 23, inventory resources 5, item
  slots 4 and checkpoint validation 6.
- Before production changes, strengthened integration assertions froze empty-type-slot equip, replacement,
  cross-employee move, same-target rejection, unequip, bound rejection, active-fragment rejection, starter, death
  and dismantle over both ledgers. They also froze exact incoming-then-displaced `updated` ordering, removed UUIDs,
  full invalid roster/inventory no-mutation and the F-C01 graph predicate.
- The strengthened old implementation remained green at equipment 30 and snapshots/start 23, as required for a
  behavior-preserving refactor. No intended gameplay Red was introduced.

### Next Experiment

Introduce the smallest crate-private transition aggregate, transfer only the four production lifecycle families and
then rerun the complete focused matrix plus direct-write/public compatibility searches.

## 2026-08-10 — E04 Typed Paired Transition Boundary

### Implementation

- Added crate-private `EquipmentLoadoutTransition`. Its typed equip plan validates identity, bound/eligibility,
  source, target, displacement, slot rules and projected active-fragment compatibility before the aggregate commits
  cloned roster/inventory together. Existing result construction retains incoming-then-displaced update order.
- Typed unequip, starter batch assignment, dead-employee forced clear and dismantle removal now preflight and commit
  both ledgers together. Starter grants, fragment/modifier death cleanup and dismantle material grants remain at
  their existing orchestration owners.
- Safezone equip/unequip, starter setup, staged combat-result death recovery and maintenance dismantle now delegate
  paired mutation. Direct production `equipped_to` writes outside the transition module are gone.
- `Inventory`, `EquipmentInventory`, `OwnedEquipment`, `EmployeeLoadout`, `ItemSlot` and all public mutation/
  serialization APIs remain unchanged. F-C01 remains the only imported graph admission authority.

### Immediate Result

- The full focused matrix passes: equipment 31 (including an invalid staged death reverse-binding atomicity case),
  snapshots/start 23, facility 11, inventory 5, item slot 4 and checkpoint validation 6.
- Production reverse-binding writes occur only in the new transition module. Remaining embedded writes in
  `event_node.rs` and `combat_power.rs` are below their `#[cfg(test)]` modules; data validation equips only a local
  scratch slot.

## 2026-08-10 — E05 Focused, Broad And Compatibility Validation

### Focused And Broad

- Focused current-tree total is 80 tests: equipment 31, snapshots/start 23, facility 11, inventory 5, item slot 4
  and checkpoint validation 6; all passed with zero failed/ignored.
- Complete Core debug/release each passed 999 unit + 51 official integration tests.
- Complete server debug/release each passed 59 tests. Core/server debug/release checks, formatter and repository
  diff check all pass.

### Static, Compatibility And Dirty Scope

- Public/serde fields and mutable compatibility APIs are unchanged. Runtime/rulebook/server/Unity canonical hashes
  remain `6e20d6a6...`, `6c53ecd5...`, `4d734af0...`, `a3224e24...`; no durable wording edit is warranted.
- Non-strict Clippy retains established debt: Core lib 83/lib-test 136, server lib 3/lib-test 7. No diagnostic points
  at the transition module or Phase 22 caller/test edits.
- Activation status was 479 entries at `6accb3ce...`; pre-audit status is 481 entries at `d605fca5...`. The two new
  rows are exactly the previously clean resource module registration and new transition source. All other target
  rows were already dirty and remain in place.
- No RON, V3, DTO, wire, server, Unity, public visibility, migration, repair, grant aggregation or Phase 23 source
  changed.

### Review Handoff

P22-C01–P22-C09 claim complete. P22-C10 remains `review_pending` until the full completion guide and current Child
are independently reread and a `Complete / High` phase-exit audit is written.

## 2026-08-10 — E06 Independent Phase-Exit Audit

- The completion guide was reread 567/567, the current Child and policy/order/F-C01 evidence were independently
  reread, and the new transition/callers/tests/consumers were inspected rather than accepting prior notes.
- Audit reruns passed equipment 31, snapshots/start 23, facility 11, inventory 5, item slot 4 and checkpoint 6:
  80/80 with zero failed/ignored. Formatter, diff and direct-write/public/canonical searches also pass.
- `COMPLETION_AUDIT.md` records all ten evidence rows pass, primary verdict `Complete`, fit `High`, improvement class
  `none`. Its artifact brings status to 482 entries at `70a9c09f...`.
- P22-C10 is complete. Phase 23 may become ready only through the Master transition; it remains uninspected here.
