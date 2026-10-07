# Core Enhanced Refactoring Audit Phase 03 — Boundary Audit

Audit Date: 2026-08-07
Status: complete; phase-exit audit `Complete / High`
Execution Mode: investigation_only
Production Mutation: none authorized or performed

## Scope Reconciliation

| Surface | Current evidence | Phase disposition |
| --- | --- | --- |
| Root persistent economy | `world/state.rs` `GameCoreState`: Enkephalin, inventory, roster, fragment inventory/policy | primary |
| Employee and roster | `employee.rs`, `recruitment.rs`, employee published data and live definitions | primary |
| Item ownership and loadout | `resources/{inventory,item_slot}.rs`, equipment/consumable published data | primary |
| Fragment and research | `skill_fragment.rs`, `abnormality_research.rs`, live fragment catalog | primary |
| Direct asset commands | world helpers, reward, shop, headquarters, maintenance, admin grants | primary |
| Cross-domain grant handoffs | event and combat staged grant calls only | supporting boundary evidence |
| Checkpoint carrier | `RunCheckpointPayload` validation/restore and focused checkpoint tests | supporting persistent seam |
| Focused safety net | 201 retained passing executions across 18 filters; 291 physical executions including one uncaptured duplicate batch | primary/supporting safety net |
| Canonical | `game_rulebook.md` and routed Core runtime/data contracts | current durable contract |
| Event/VN session and battle internals | only direct persistent economy commit is followed | assigned to later Phases |
| DTO/Unity/other external Rust users | snapshots inspected only as contradictory-ledger consumer; external users unverified | out-of-scope/unverified |

The physical `world` module is not the Phase boundary. The Phase follows an employee or asset only while its
definition identity, run-local ownership, quantity, loadout binding, cost/grant transaction, pending research or
checkpoint persistence is authoritative.

## Root Ownership And Identity Trace

1. `GameCoreState` owns the current Enkephalin, `Inventory`, `EmployeeRoster`, `SkillFragmentInventory`,
   `SkillFragmentPolicy` and UUID manager. `RunState` separately owns recruitment and abnormality-research state.
2. An employee definition ID identifies authored employee data; `Employee::uuid` identifies one run-local roster
   member. `EmployeeRoster` is a UUID-keyed map, while recruitment accounts for definition IDs across queue,
   active session and roster.
3. Equipment has a run-local owned instance UUID and an authored equipment definition ID. `EquipmentInventory`
   is keyed by instance UUID, and `OwnedEquipment` embeds the same UUID, full metadata, an `equipped_to` employee
   UUID and instance-local enhancement level.
4. An employee `ItemSlot` repeats the equipment instance UUID, base definition UUID and item type in an
   `EquippedRef`. The slot and `OwnedEquipment::equipped_to` therefore form one bidirectional invariant across two
   independently serialized ledgers.
5. Consumables are definition-ID-keyed stacks containing full metadata and quantity. Their static metadata UUID is
   the selector for generic inventory operations; it is not an owned instance UUID. Equipment materials are
   string-ID-keyed quantities.
6. Fragment ownership, research progress, pending delivery and fragment dust share `SkillFragmentInventory`.
   Each employee loadout stores baseline fragment IDs and at most one active fragment ID.
7. Enkephalin is a root-owned checked balance. Shop session state owns offers, refresh history and purchase
   history, not the currency itself.

These identities are intentionally distinct. Replacing them with one generic UUID or treating authored metadata
as the player ledger would change the current contract.

## Roster, Equipment, And Loadout Mutation Trace

- Starter selection creates employee UUIDs and owned equipment UUIDs, inserts employees/equipment, and writes
  both the employee slot and `OwnedEquipment::equipped_to` association.
- Normal equip first validates the employee, source item and both ledgers, then clones roster and inventory. It
  handles the source employee, displaced item and target slot in the clones, updates both sides and commits both
  roots together.
- Unequip validates the pair before clearing the item slot and `equipped_to`. Current post-validation writes are
  effectively infallible, but the operation still has two mutation owners.
- Consumable use clones inventory and roster, removes one definition stack unit, applies the persistent modifier
  and commits only after the complete operation succeeds.
- Battle-death recovery edits a staged combat-result state, clears every equipment slot on the dead employee and
  clears the matching inventory bindings before the staged result is installed.
- Dismantle validates bound/equipped state, automatically clears an equipped association when allowed, removes
  the owned instance and grants materials through a staged path.
- `Inventory`, `EquipmentInventory`, `OwnedEquipment`, `EmployeeLoadout` and roster mutation methods expose
  enough independent mutation surfaces to construct contradictory bindings. Production commands currently
  maintain them manually; fixtures frequently reproduce the same two-sided setup.

No normal command path was found that currently commits only one side of an equip transition. The duplication is
therefore a change-amplification and invariant-encapsulation candidate, while contradictory V3 import is a
separate correctness candidate.

## Fragment, Growth, Research, And Currency Trace

- Fragment grant updates ownership count and research progress in the same inventory. Pending research delivery
  applies to a clone and installs it only after the whole delivery validates; invalid pending entries leave the
  prior planning/node state unchanged.
- Active-fragment equip validates current ownership, live metadata kind and policy equip limit. Battle profile
  construction later trusts the stored active ID and reads its progress, but does not re-check that the inventory
  still owns the active fragment.
- Baseline fragment IDs are authored/starter capability and do not require ordinary ownership. Active fragment
  ownership is a different invariant and must not be weakened by baseline behavior.
- `GrowthStack` is carried in each employee combat profile, but current canonical explicitly records no gameplay
  producer. Its presence alone is not dead-code or consolidation proof.
- Reward, shop, event, combat and admin paths call the same `GrantExecutor`. It applies effects in declared order,
  derives each effect seed from the supplied seed/index and mutates five supplied ledgers: inventory, fragments,
  roster, UUID manager and Enkephalin.
- Every current Core production caller supplies clones or a broader staged state and commits only after the grant
  succeeds. The public executor itself is not atomic if called with live ledgers, but no direct workspace caller
  does that.

## Reward, Shop, Headquarters, And Maintenance Atomicity

### Reward and direct grants

- Reward claim/apply and preview create the same five cloned grant ledgers before invoking `GrantExecutor`.
- Event grants use `PlannedEventGrant`; combat completion uses `StagedCombatResultState`; admin grants also clone
  all five values. Failures do not install the staged values.
- Current tests demonstrate reward-claim and combat-result failure do not partially commit earlier effects.

### Shop

- Purchase validates session/offer/currency, clones all five grant ledgers, subtracts currency in the clone,
  applies the grant, removes the offer, then installs currency/assets/session state together.
- Sale validates identity, quantity, bound/equipped rules and checked currency addition, removes from a cloned
  inventory and commits the inventory plus balance. The later plain addition is safe because the checked result is
  established inside the same method before mutation.
- Refresh plans the next full page and checked currency/use counters before replacing the session and balance.
  Exhaustion and insufficient funds leave both unchanged.

### Headquarters

- Recruitment and emergency-supply completion are wrapped by a headquarters-specific rollback helper. That helper
  manually snapshots ten state components and restores them on error.
- The same `GameCore` already provides `with_state_transaction()`, which clones the full non-battle state and
  preserves the encounter-compendium monotonic union. Current headquarters actions do not mutate the extra
  excluded/preserved domains, so the bespoke rollback is current-behavior duplication, not a proven bug.

### Maintenance and the confirmed partial-mutation seam

- Dismantle and related grant paths stage inventory/grants before commit.
- Enhancement publication validates a nonempty recipe, positive individual costs and known material IDs, but
  does not reject or aggregate duplicate material IDs in one level's `costs_per_level`.
- Command validation compares each row independently with the current quantity. The handler then consumes each
  row sequentially from the live inventory and increments enhancement only afterward.
- For two rows of the same material costing 2 each and a balance of 2, both prechecks see `2 >= 2`; the first
  consume commits, the second returns an error, and the equipment level remains unchanged. This is a source-proven
  partial asset loss accepted by the typed schema.
- Current live data has one enhancement recipe (`standard_armor`, maximum level 5) and one cost row
  (`equipment_dust`, amount 2), so the published live instance does not trigger the defect. The schema and loader
  nevertheless authorize the duplicate-row shape.

## Checkpoint And Consumer Integrity Trace

`RunCheckpointPayload` strictly denies unknown root fields and carries Enkephalin, inventory, roster, fragments
and UUID state. Before restore, the current validator checks power/map/facility/progression references,
recruitment exact definition ownership and monolith state. It does not validate the internal economy graph.

Missing pre-restore checks include:

- roster map key equals `Employee::uuid`;
- equipment map key equals `OwnedEquipment::instance_uuid`;
- every item slot resolves to one owned instance and agrees with its stored base identity/type and `equipped_to`;
- no instance is placed in multiple employee slots and no inventory binding points at a different/missing employee;
- consumable map key/metadata identity, nonzero quantity and inventory reserve high-water consistency;
- material and fragment stack/progress/pending self-consistency;
- active fragment is currently owned and valid under the stored policy/loadout invariant.

Restore assigns all four economy roots directly. Battle setup then resolves authored equipment from the slot's
base UUID and uses the owned instance only for enhancement; it does not prove `OwnedEquipment::meta` or
`equipped_to` agrees with the slot. Fragment profile construction similarly trusts the active loadout ID after
looking up its metadata/progress. Snapshot projection exposes both inventory bindings and employee slots and can
surface contradictory state rather than resolving authority.

Normal commands preserve these invariants, so a correction should reject inconsistent current V3 input before
mutation. It must not repair/rebind data or silently select one ledger. Whether embedded owned metadata must equal
the latest complete live record is a separate save/content-lifecycle policy; the confirmed correction can first
enforce payload self-consistency and required resolvable identities.

This finding extends Phase 2 C02's context-aware checkpoint validation boundary with economy-specific predicates.
Final consolidation may share one import validation entry point, but the work types and domain invariants remain
explicit.

## Published Live Data And Canonical Alignment

The audited live catalog currently contains 12 employee definitions, 88 equipment records, eight consumables,
13 rewards, 31 skill fragments, three shop contexts, one equipment material, one dismantle recipe and one
enhancement recipe. Current run-policy values observed at direct seams include starter Enkephalin 500, survival
experience 10, one paid refresh use and headquarters emergency Enkephalin 120.

Canonical and code agree that:

- Safezone loadout replacement is atomic and enhancement belongs to an owned equipment instance;
- equipment material and fragment dust are different ledgers;
- `GrantConsumable` consumers share the grant executor and cloned inventory stage;
- SavePoint/battle staging carries inventory quantity and reserve state;
- pending research is delivered atomically only on authoritative planning return;
- shop purchase stages currency, offer removal and all grants before commit;
- maintenance failure must leave no partial mutation.

The duplicate enhancement-cost shape contradicts the last invariant. The live recipe's single cost row explains
why current content tests do not expose it; it does not erase the valid-schema runtime defect.

## Consumer And Compatibility Classification

| Consumer or seam | Classification | Evidence |
| --- | --- | --- |
| Core world commands | confirmed authoritative consumers | direct command dispatch and mutation calls |
| Core event/combat flows | confirmed staged grant consumers only | typed staged state around shared grant executor |
| Core active-run V3 | confirmed persistent carrier | strict payload and direct restore assignments |
| published RON catalogs | confirmed typed inputs | strict loader and 24 live RON tests |
| snapshots | confirmed derived consumer | projects employee slots and inventory bindings separately |
| admin grants | confirmed development consumer | same five-ledger cloned stage |
| server | opaque checkpoint/result transport for this boundary | Phase 2 evidence; no server economy authority found |
| Unity and other external Rust APIs | unverified | excluded deployment/consumer inventory |

Serialized V3 fields and current public Core outputs remain preserved by refactor candidates. Removing one side of
the equipment binding, rebinding embedded metadata, changing catalog values or changing public grant APIs without
consumer evidence requires a separate compatibility/migration decision.

## Focused Safety Net And Gaps

The retained evidentiary set is 18 runner invocations and 201 tests, all passing with zero failures and zero
ignored tests. An earlier eight-filter parallel batch physically executed the same 90 tests, but its aggregated
output was truncated before the per-filter results could be retained; all eight filters were immediately rerun.
Accordingly, the physical total is 26 invocations/291 test executions, while only the captured 201/201 set is used
as pass evidence. Coverage includes inventory/item slots/employees/fragments/research, equipment commands,
recruitment, rewards, shop,
reward and checkpoint node sessions, maintenance/facility, starter/snapshots, live RON catalogs, item skills, map
research delivery and selected combat reward/rollback paths. Runner-reported filtered counts are nonmatching
tests, not skips.

Missing characterization evidence:

- current V3 import rejects every contradictory roster/inventory/loadout/fragment graph without partial restore;
- duplicate same-material enhancement costs cannot pass validation then partially consume;
- an aggregate equip transition covers move, replacement, bound item, death, dismantle, starter grant and restore;
- one typed grant transaction preserves effect/diff ordering, UUID/seed continuation and every current caller's
  no-partial-commit behavior;
- headquarters generic transaction replacement preserves exact rollback and compendium behavior;
- public Core grant/inventory APIs have no external Rust consumers beyond this workspace.

No focused test file contains `#[ignore]`. A synthetic duplicate-cost runtime probe was not added because
production/test mutation is forbidden; the validator/handler branch proof is recorded as the required first
negative test for implementation.
