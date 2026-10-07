# Core Enhanced Refactoring Audit Phase 03 — Candidate Catalog

Catalog Date: 2026-08-07
Status: complete; dispositions accepted by phase-exit audit `Complete / High`

Priority axes are ordered as: ownership duplication, correctness/atomicity, lifecycle complexity, change
amplification, test fragility, failure blast radius, change frequency. Current accumulated dirty work makes Git
history non-representative, so change frequency is `unknown` and no automatic total-band verdict is used.

## Confirmed Candidates

### CRA-P03-C01 — Reject Internally Inconsistent Checkpoint Economy Graphs Before Restore

- Work Type: `correctness_correction`
- Problem Evidence:
  - Strict `RunCheckpointPayload` carries roster, inventory, skill fragments and Enkephalin and restore assigns
    all four roots directly.
  - The validator checks map/facility/progression references, recruitment definition ownership and monolith state,
    but has no economy-graph validator.
  - The payload can therefore disagree between map keys and embedded employee/equipment UUIDs, between employee
    `ItemSlot` and `OwnedEquipment::{meta,equipped_to}`, or between active fragments and current ownership.
  - Normal equip/death/dismantle commands maintain both equipment ledgers; battle setup and fragment profile
    construction later trust different portions of an imported graph. A checksummed current V3 document can be
    accepted and then produce a contradictory snapshot, altered equipment authority or a later failure.
  - Existing checkpoint negatives cover recruitment ownership and other Phase 2 domains, not persistent economy
    self-consistency or unchanged-state rejection.
- Current Authority And Consumers: the V3 import validator gates explicit load/Continue; roster, inventory and
  fragment state jointly own the invariant; battle setup and snapshot projection consume it.
- Target Authority: one economy-specific `validate_checkpoint_state` predicate is called from the existing
  context-aware checkpoint validator before mutation. It validates map-key/embedded identities, bidirectional and
  unique equipment binding, slot identity/type, positive/key-consistent stacks, reserve high-water and active
  fragment ownership/policy consistency.
- Preserved Invariants: exact valid V3 JSON shape/result, embedded owned metadata, instance enhancement, current
  item/fragment IDs, no repair or rebind, deterministic UUID continuation, load-count semantics and unchanged
  state/export after rejection.
- Removal Or Consolidation Scope: remove acceptance of internally contradictory current-version economy graphs
  and centralize their import predicate. Do not remove either serialized binding or alter normal command flow.
- Priority Axes And Total: `3 / 3 / 2 / 2 / 3 / 3 / unknown`; known subtotal `16/18`, no automatic band.
  Correctness risk `3` independently triggers priority investigation.
- Validation Safety Net: first add mutated-V3 negative tests for every key/UUID mismatch, missing/duplicate/wrong
  equipment association, reserve underflow, invalid stack and unowned active fragment; assert load count, current
  state and re-export bytes remain unchanged. Retain a full valid round trip with starter/equipped/enhanced items,
  consumable quantity, materials and fragment progress.
- Policy/External Contract Decision: payload self-consistency and fail-closed current-version restore follow the
  existing strict contract. Rebinding complete embedded metadata to the latest live catalog or repairing old saves
  is explicitly deferred `policy_or_feature_migration` work.
- Readiness: `test_first_then_implementation`; exact fragment progress/pending predicates should be characterized
  from current mutation APIs before centralization.
- Disposition: `priority_candidate`.

### CRA-P03-C02 — Make Enhancement Material Validation And Consumption Atomic Per Material

- Work Type: `correctness_correction`
- Problem Evidence:
  - Published-data validation requires nonempty costs, positive amounts and known material IDs, but permits the
    same material ID more than once in one recipe level.
  - Command validation checks every cost row independently against the same current balance.
  - The handler consumes cost rows sequentially from live inventory and increments the equipment level only after
    all consumes.
  - With duplicate rows `(dust, 2), (dust, 2)` and current dust `2`, both prechecks pass; the first consume removes
    two, the second errors, and the failed command leaves dust lost with the old enhancement level.
  - Current live data has one `standard_armor` recipe with a single `equipment_dust: 2` row, and the existing
    enhancement success test uses one row. Current content therefore masks a schema-valid defect.
- Current Authority And Consumers: equipment published-data validation authorizes recipe shape; maintenance
  validation and handler own cost admission/consumption; inventory owns material balance and equipment level.
- Target Authority: normalize or aggregate costs by material once for both admission and consumption, or execute
  the complete cost/level change on a cloned inventory and commit once. The same typed plan must feed result diffs.
- Preserved Invariants: current single-row live result, exact total cost, max level, bound-item rejection,
  metadata lookup errors, material-stack result semantics, enhancement instance identity and failure error class.
- Removal Or Consolidation Scope: remove the validate-then-consume disagreement for duplicate IDs. Do not add a
  recipe, rebalance cost, change maximum level or silently coalesce invalid published data without a declared
  validator decision.
- Priority Axes And Total: `1 / 3 / 1 / 2 / 3 / 3 / unknown`; known subtotal `13/18`, no automatic band.
  Correctness risk `3` independently triggers priority investigation.
- Validation Safety Net: first add a published-data duplicate-cost case and a command test with balance equal to
  one row but less than the aggregate; assert unchanged materials, level and inventory diff on failure. Add the
  exact aggregate-success case and retain all 28 equipment tests plus live RON loading.
- Policy/External Contract Decision: canonical already requires failed maintenance commands to leave no partial
  mutation. Whether duplicate rows are rejected at publication or accepted and aggregated should be selected in
  the implementation Goal, but either choice must prevent partial mutation and preserve current live behavior.
- Readiness: `test_first_then_implementation`.
- Disposition: `priority_candidate`.

### CRA-P03-C03 — Encapsulate Bidirectional Equipment/Employee Loadout Transitions

- Work Type: `behavior_preserving_refactor`
- Problem Evidence:
  - The same equipment association is stored as `EmployeeLoadout::ItemSlot::EquippedRef` and
    `OwnedEquipment::equipped_to` in separately owned maps.
  - Equip/move/replacement, unequip, starter setup, death recovery and dismantle manually coordinate the pair in
    different modules/staged states. Fixtures also construct both sides directly.
  - `EquipmentInventory::remove_item`, mutable item lookup and loadout fields permit independent mutation; no
    aggregate API mechanically prevents missing, duplicate or wrong-employee associations.
  - Current valid production paths and 201 selected tests pass, so this is an ownership/change-amplification risk,
    not evidence that a normal command presently commits half a transition.
- Current Authority And Consumers: employee roster owns slots; inventory owns item instances and reverse binding;
  starter/world/maintenance/combat-result paths are mutation consumers; checkpoint and snapshots carry/project
  both representations.
- Target Authority: a private roster-inventory transition aggregate exposes typed equip/move/replace/unequip and
  forced-clear operations. It validates once, edits both staged ledgers together and returns the existing outcome
  data; production callers stop assigning either side independently.
- Preserved Invariants: both serialized V3 fields, instance/base UUID distinction, item type/slot rules, bound
  semantics, displaced-item result order, enhancement, death/dismantle behavior, starter loadout and DTO shape.
- Removal Or Consolidation Scope: remove direct production synchronization outside the transition authority.
  Do not eliminate one persisted ledger, make equipment definition-singleton, or redesign safezone policy.
- Priority Axes And Total: `3 / 2 / 2 / 3 / 3 / 3 / unknown`; known subtotal `16/18`, no automatic band.
- Validation Safety Net: characterize empty equip, move between employees, same-target idempotence/error, slot
  replacement, bound rejection, unequip, death, dismantle and starter setup with assertions over both ledgers and
  exact existing outcomes. Keep V3 round trip and snapshot agreement tests.
- Policy/External Contract Decision: private mutation encapsulation needs none. Removing a serialized field or
  changing public inventory mutation compatibility is deferred migration/API work.
- Readiness: `ready_after_transition_characterization`; sequence after C01 so imported state enters the aggregate
  with a proven invariant.
- Disposition: `candidate`.

### CRA-P03-C04 — Consolidate The Five-Ledger Grant Transaction State

- Work Type: `behavior_preserving_refactor`
- Problem Evidence:
  - `GrantExecutor` accepts five separate mutable ledgers plus database, policy, context, effects and seed.
  - Reward apply/preview, shop purchase, event grant, combat result and admin grant repeatedly clone, pass and
    commit inventory, fragments, roster, UUID manager and Enkephalin as parallel local variables.
  - `GrantExecutor` applies effects in order and is not independently rollback-capable, but every current Core
    production caller stages those five values before invoking it. No current partial-commit defect was found.
  - Adding a new grant-owned ledger or changing commit order requires synchronized edits at every call site, and
    a missed field can compile while violating atomicity/result parity.
- Current Authority And Consumers: each root ledger remains asset authority; the grant executor owns ordered
  effect semantics; reward/shop/event/combat/admin callers independently own staging/commit boilerplate.
- Target Authority: one private typed `EconomyGrantState` (or equivalent capture/apply/commit transaction) holds
  the five values and their commit mapping. Current consumers pass the typed state while retaining domain-specific
  offer/session/combat outer staging.
- Preserved Invariants: effect order, seed derivation, UUID allocation, checked quantities, exact diff ordering,
  reward target context, no partial caller commit, shop offer/currency semantics and current public results.
- Removal Or Consolidation Scope: consolidate only the shared five-ledger capture/pass/commit family. Do not merge
  shop/event/combat session state, make the helper global policy authority or widen an outer transaction lifetime.
- Priority Axes And Total: `3 / 1 / 2 / 3 / 2 / 3 / unknown`; known subtotal `14/18`, no automatic band.
- Validation Safety Net: golden before/after and result-diff parity for each current caller; multi-effect late
  failure; duplicate equipment UUID/overflow; fragment research; employee experience target; exact seed/UUID
  continuation; preview no-mutation. Retain reward, shop, node-session, combat-result and admin filters.
- Policy/External Contract Decision: none for a private aggregate. `GrantExecutor` is public Rust API and other
  consumers are unknown, so signature removal must be avoided or separately inventoried.
- Readiness: `ready_after_caller_parity_characterization`.
- Disposition: `candidate`.

### CRA-P03-C05 — Replace Headquarters' Handwritten Rollback Field List With The Existing Transaction Boundary

- Work Type: `behavior_preserving_refactor`
- Problem Evidence:
  - `with_headquarters_transaction()` clones and restores game state, node content/session, run, UUID manager,
    Enkephalin, inventory, roster, fragments and checkpoint through ten handwritten assignments.
  - `with_state_transaction()` already captures the complete non-battle `GameCoreState` and restores it on error
    while preserving the encounter-compendium monotonic union.
  - Headquarters recruitment/emergency paths use only the state covered by the generic transaction; focused
    headquarters/node tests pass and no current rollback divergence was observed.
  - Future root fields can be omitted from the headquarters list without a compile-time failure, reproducing the
    same field-parity risk as other handwritten transaction clones.
- Current Authority And Consumers: headquarters owns its command-specific validation/completion; the generic
  world transaction owns non-battle rollback semantics; encounter compendium remains a separate monotonic profile
  aggregate.
- Target Authority: execute headquarters completion through the existing generic non-battle transaction or a
  thin domain wrapper that delegates to it. Remove the ten-field manual rollback list.
- Preserved Invariants: recruitment queue/session results, employee/UUID allocation, emergency balance, node
  completion, checkpoint replacement, original errors, active-battle rejection and compendium union behavior.
- Removal Or Consolidation Scope: remove only the duplicate rollback implementation. Do not merge headquarters
  session policy with reward/shop sessions or change what constitutes a completed node.
- Priority Axes And Total: `2 / 1 / 2 / 2 / 2 / 2 / unknown`; known subtotal `11/18`, no automatic band.
- Validation Safety Net: characterize success and a late failure for recruit, emergency supplies and non-recruit
  completion; compare the entire non-battle state plus compendium before/after. Retain all 18 node-session and 11
  facility tests.
- Policy/External Contract Decision: none if exact rollback/exclusion behavior is proven. Any decision to roll
  back profile knowledge or active battle state is excluded behavior change.
- Readiness: `ready_after_full_state_rollback_parity_characterization`.
- Disposition: `candidate`.

## Rejected, Deferred, And Unverified Signals

| Signal ID | Signal | Evidence and reasoning | Disposition |
| --- | --- | --- | --- |
| CRA-P03-S01 | Treat `EmployeeRoster::add()` silent same-UUID replacement as a standalone current defect | UUID manager allocation and recruitment exact-definition validation make collision unreachable in valid current production flow; no direct external caller was found. C01/C03 should validate/encapsulate the invariant without changing public error semantics on speculation. | `rejected` as standalone candidate; retain as C01/C03 defense case |
| CRA-P03-S02 | Rebind checkpoint-embedded employee/equipment metadata to the latest live catalog | Current payload intentionally embeds owned/runtime records and no canonical policy decides freeze-versus-rebind when content changes. Rebinding can change an active run and requires save/content compatibility policy. | `deferred`; `policy_or_feature_migration` only |
| CRA-P03-S03 | Classify `GrantExecutor` itself as a current atomicity bug | It mutates effects/ledgers incrementally, but every current Core production caller supplies cloned or broader staged state and commits only on success. Existing failure tests confirm the outer guarantee. | `rejected` correctness claim; C04 keeps the change-amplification refactor |
| CRA-P03-S04 | Merge shop, event, reward, combat and headquarters sessions because they can grant the same assets | Their admission, lifetime, retry and completion policies differ. Only the five persistent grant ledgers are shared. | `rejected`; C04 is deliberately narrower |
| CRA-P03-S05 | Starter selection needs a new global rollback because it mutates roster/inventory before run creation | With strictly loaded live data and clean pre-run state, no reachable post-grant failure that leaves durable partial state was established. General transaction expansion could alter run/profile lifetimes. | `unverified`; C03/C04 characterization seam, not standalone correction |
| CRA-P03-S06 | Delete `GrowthStack` because canonical reports no producer | It is serialized/runtime state and absence of a current producer is an intentional contract note, not deletion or migration authority. External/save consumers are unknown. | `rejected` |
| CRA-P03-S07 | Merge equipment material and fragment dust | Canonical, data and reward effects distinguish their ownership and consumers. Similar numeric operations are not shared authority. | `rejected` |
| CRA-P03-S08 | Add more enhancement recipes or rebalance the current dust cost | Current live coverage is narrow, but content quantity/value changes are feature/balance work and do not repair C02's atomicity seam. | `deferred` to content/balance Goal |
| CRA-P03-S09 | Restore removed artifact rewards, legacy shop vocabulary or old consumable fields as fallback compatibility | Current loaders/tests explicitly reject removed vocabulary and require canonical quantity/typed fields. A fallback would conceal invalid content. | `rejected` |
| CRA-P03-S10 | Promote snapshots or authored price/reward records to owned-asset authority | Snapshots are derived projections; catalogs authorize definitions/prices/effects. Root runtime ledgers own player quantities and associations. | `rejected` |
| CRA-P03-S11 | Audit event/VN and combat internals now because they invoke grants | Their direct grant handoff is confirmed staged; session lifecycle and battle calculation remain Phases 4–8. | `deferred` cross-boundary handoff |
| CRA-P03-S12 | Remove or change public inventory/grant APIs based on no workspace consumers | Workspace search cannot prove absence of external Rust/Unity consumers. Private encapsulation can proceed while public removal requires consumer inventory. | `unverified`; compatibility decision required |
| CRA-P03-S13 | Treat direct admin balance/condition mutation as competing gameplay authority | Admin handlers are explicit gated development commands and use staged grants where applicable. Their publication/persistence policy belongs to the admin/consumer boundary. | `deferred`; no gameplay consolidation claim |
| CRA-P03-S14 | Treat current live single-row enhancement recipe as proof duplicate costs are impossible | Typed validation permits duplicates, so the handler must be safe for every accepted publication. Live data establishes current impact frequency only, not correctness. | `rejected` assumption; C02 retained |

## Cross-Boundary Handoff

- Final should evaluate whether Phase 2 C02 and Phase 3 C01 share one context-aware checkpoint validation entry
  point while preserving explicit map/progression and economy predicates.
- Phase 4 owns event/shop/VN session lifecycle. It may rely on the proven staged persistent-asset commit but must
  audit offer/session/idempotence semantics separately.
- Phases 5–8 own battle setup/calculation/effects/death. C01/C03 hand off the finding that battle equipment and
  fragment setup trust the persistent graph; battle formula correctness was not audited here.
- Phase 10 owns DTO/projection design. It may treat separate roster/inventory projections as consumers but must not
  select either as mutation authority.
- C03 should follow C01; C04 and C05 can be planned independently after exact parity characterization; C02 is an
  isolated test-first correction.
