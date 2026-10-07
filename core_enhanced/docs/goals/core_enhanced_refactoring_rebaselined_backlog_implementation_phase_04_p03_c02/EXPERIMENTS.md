# Phase 04 CRA-P03-C02 Experiments

Status: complete
Implementation Complete: true

## 2026-08-09 — E00 Activation And Required-Document Initialization

### Authorization And Dependency

- The user's Master execution request remains active and authorizes the single active Child.
- Phase 03 CRA-P02-C01 has a current `Complete / High` audit with improvement class `none`.
- Phase 04 is next in the exact registry; Phases 05~44 remain pending and unauthorized.
- No candidate-specific policy decision applies; global CRA-POL-006 remains binding.

### Evidence Carried Into Initialization

- CRA-P03-C02 remains a correctness correction with priority `1/3/1/2/3/3/unknown = 13/18`.
- Historical defect: repeated material rows pass row-wise admission against the same balance, then sequential live
  consumption partially mutates before later failure.
- Required first proof: duplicate insufficient aggregate leaves full state/diff unchanged; exact aggregate consumes
  the total and enhances exactly once.
- Preservation boundary: current single-row live cost/result/errors, bound/max gates, recipe schema, V3/wire/public
  shape and balance content.

### Initialization Result

- Required Child documents initialized: `PLAN.md`, `EXPERIMENTS.md`, `EXPERIMENT_NOTES.md`.
- Master Phase 04 coordination changes from `ready` to `active`; no other Child is active.
- Phase 04 production status/source/data investigation: not started.
- Test executions: none.
- Production/test/data/canonical mutation before initialization: none.

### Next Experiment

Read all three Child documents completely, capture expanded dirty status and prospective target hashes, then discover
the current recipe publication/admission/execution/diff path and establish a nonzero enhancement baseline.

## 2026-08-09 — E01 Current Authority, Dirty Baseline And Focused Baseline

### Current Authority And Defect

- `data/equipment_data.rs` owns recipe types/publication validation. It rejects empty/zero/missing references but
  retains repeated material rows and has no checked aggregate helper.
- `world/helpers.rs::validate_enhance_equipment_payload` checks each row against the same pre-command balance.
- `world/maintenance.rs::handle_enhance_equipment` resolves metadata and consumes authoritative material rows in
  order, then mutates `OwnedEquipment::enhancement_level` and constructs `InventoryDiffDto`. It has no local staging
  or generic transaction wrapper.
- `EquipmentMaterialInventory` is the authoritative stack owner; `consume` removes a stack at zero and errors on an
  insufficient later row. Action capability/maintenance preview delegates back to the same payload validator.
- The current defect therefore exactly matches the historical disposition.

### Live Data And Canonical

- Live file is `game_resources/data/equipments/base.ron`, hash
  `322d7455a256082f03804c0dcc7a77a4caa3a06919d5e34b667042b411bcd1a6`, pre-existing modified 207+/0- for weapon
  tactical profiles/training sword. The sole enhancement recipe remains `standard_armor` with one
  `equipment_dust: 2` row; this Phase will not edit it.
- The game rulebook already promises failed Maintenance commands leave no partial mutation and states the sole live
  recipe coverage. Its pre-Phase hash is
  `9ad37a2095d34043aaf4d52513477316996bec3c9aa86a1ec1d905ac1dc02fd3`.

### Dirty Baseline And Overlap

- Commit remains `f318414fb0156a67e04c5479000c2a2683fc0c09`.
- Expanded post-initialization status: 388 entries; SHA-256
  `0b0c458e5dc704a998af6011e803e16bdabcd7644a336b79bb4b89e33cbd5f93`.
- Pre-existing modified targets:
  - equipment data 113+/2-, hash `f07e6cef9d380710cb50ae0c68f9b6e46499276c485024fbce74fe2615b38fcd`
    for tactical weapon profiles/tests;
  - world helpers 9+/7-, hash `ac2ff3569fc90775f1af94da895f20be220cf9a18e9f1484d5539bee9f9f5c7c`
    for skill-fragment compatibility profile routing;
  - equipment tests 32+/8-, hash `0d0decf15dcda924d4245c76c15c5c2ed1ab13678ae5a5c46ba9f0b5d34e20e6`
    for armed-starter/weapon behavior;
  - live RON and `ron_loading.rs` contain broad user-owned tactical/Boss changes.
- `world/maintenance.rs` and `world/facility.rs` are clean with hashes
  `85058ffcfee794c2c9e905f6181ed1dd44ac74b2d58bccbd6e4d20675c562776` and
  `47b95a0a9a9f1a23b84464d5fab115e6f3c54e57d78a57d2af4707648a2737f9`.
- The required enhancement seams are separable additions; do not alter the identified tactical/weapon hunks.

### Baseline Command And Result

```text
cargo test -p game_core enhance_equipment --lib -- --nocapture
```

Exactly 1 test executed and passed with 0 failed/ignored and 933 filtered. It proves Maintenance-only single-row
success and instance update, but not duplicate aggregate failure/success or overflow atomicity.

### Next Experiment

Add isolated duplicate insufficient, exact and overflow cases to the existing equipment test module. Record the
pre-implementation state/result behavior before adding a shared checked aggregate/staged commit.

## 2026-08-09 — E02 Test-First Duplicate And Overflow Proof

### Added Proofs

- Duplicate insufficient: two `dust: 2` rows with balance 2, complete serialized inventory equality, typed rejection,
  level 0 and balance 2 after failure.
- Duplicate exact: the same rows with balance 4, one enhancement, one updated item, one logical material-stack diff
  and zero remaining balance.
- Overflow publication: `u32::MAX` plus `1` for the same material must fail the equipment contract at the recipe's
  aggregate cost path with exact object/path/cause.

### Pre-Implementation Results

```text
cargo test -p game_core duplicate_material_rows --lib -- --nocapture
cargo test -p game_core duplicate_enhancement_material_total_overflow_is_rejected_at_publication --lib -- --nocapture
```

- The two runtime tests both failed, 0 passed/ignored and 935 filtered. Insufficient aggregate consumed the first
  row and removed the material stack before returning failure. Exact aggregate enhanced once but emitted two
  material-stack diffs rather than one logical aggregate.
- The overflow test failed 0/1 with 936 filtered because current publication returned `Ok(())`.
- Initial assertion-type compile corrections changed only the new test (`Option<String>` field access) and occurred
  before behavior evidence; the final pre-implementation executions compiled and failed for the intended causes.

### Implementation Decision

Add one `pub(crate)` checked aggregate method on enhancement recipe metadata that preserves first material order.
Use it in publication validation, runtime payload validation and execution. Execute all aggregate consumes and the
level increment against a cloned inventory, build the existing result/diff from that staged state, and assign the
inventory root only after every step succeeds. This preserves single-row shape and prevents both known and future
late failures from partially committing.

## 2026-08-09 — E03 Checked Aggregate, Staged Commit And Preview Closure

### Implementation

- `EquipmentEnhancementRecipeMetadata::aggregated_costs` now preserves first material order and combines repeated
  IDs with `checked_add`.
- Recipe publication rejects duplicate-total overflow at `costs_per_level` with the recipe object ID and exact
  `aggregate cost amount overflow` cause.
- Runtime payload validation uses the same aggregate, and execution resolves one metadata row per aggregate before
  consuming and enhancing a cloned inventory. The authoritative inventory root is assigned only after all steps
  succeed.
- Maintenance preview now exposes aggregate costs and evaluates eligibility against the aggregate. This closes the
  capability gate as well as the handler: an insufficient duplicate total is rejected as the existing
  `NoEligiblePayload` action contract, not admitted by a stale row-wise preview.
- The game rulebook Maintenance contract now states checked first-order aggregation, pre-mutation rejection and the
  one-stack/one-level exact-total result. Live RON/schema/content remain unchanged.

### Focused Results

```text
cargo test -p game_core duplicate_material_rows --lib -- --nocapture
cargo test -p game_core duplicate_enhancement_material_total_overflow_is_rejected_at_publication --lib -- --nocapture
```

- Initial post-implementation run: exact aggregate passed, while insufficient aggregate preserved inventory but
  returned target-level `InvalidAction`. Investigation proved the Maintenance preview still checked duplicate rows
  independently, keeping the root action enabled. This was a production contract seam, not a test-only mismatch.
- After routing preview cost display and eligibility through the same aggregate, duplicate runtime proof passed 2/2
  with 935 filtered and overflow publication proof passed 1/1 with 936 filtered; 0 failed/ignored.
- Formatting check passed before the final focused executions.

### Next Experiment

Run the entire equipment world/data modules, live RON integration and owning Core package/check. Then inspect scoped
diffs and hashes, compare single-row/output compatibility, and perform the required completion review.

## 2026-08-09 — E04 Broad Validation And Phase Exit

- Equipment world module passed 30/30; equipment data module passed 7/7; production RON loading passed 25/25.
- Full Core package passed 986/986 physical tests across nonzero binaries, with 0 failed/ignored. Core and server
  all-target checks, formatting and `git diff --check` pass.
- Static inventory found no remaining row-wise runtime consumer outside aggregate construction, row-level publication
  validation and fixtures. The production RON hash remains unchanged.
- Expanded status is 390 entries with SHA-256
  `c57615f22f8beb4665827dbd3c6f0f8c3251385001f4dd606686e913b46ae98b`; the two-entry increase is exactly the
  formerly clean Maintenance and facility source files.
- Required completion review reread current code/data/consumer/canonical/diff evidence and returned
  `Complete / High`, improvement class `none`. Phase 04 is complete and may release Phase 05 to readiness.
