# Phase 02 CRA-F-C01 Experiments

Status: complete
Implementation Complete: true

## 2026-08-09 — E00 Activation And Required-Document Initialization

### Authorization And Dependency

- The user explicitly requested execution of the Master PLAN on 2026-08-09; this continues to authorize the single
  active Child.
- Phase 01 CRA-P09-C08 has a current `Complete / High` completion review with improvement class `none`.
- Before Phase 02 production search, status inspection, tests or mutation, all three Master documents, the Phase 01
  completion review and directly relevant P02/P03/final evidence and CRA-POL-006 text were reviewed.
- Phase 02 merges the former CRA-P02-C02 seed/progression and CRA-P03-C01 asset/economy halves into CRA-F-C01; it
  does not authorize either later mapping refactor.

### Evidence Carried Into Initialization

- Candidate: CRA-F-C01, `correctness_correction`, priority `3/3/3/3/3/3/unknown = 18/18`.
- Historical P02 gap: validation references nodes but does not compose `run_progression.run_seed`, serialized UUID
  seed, live run seed and graph-derived progression/selectability before restore.
- Historical P03 gap: validation does not compose roster, inventory, equipment slots/reverse bindings, reserves,
  materials, fragment ownership/loadout/progress and economy roots before restore.
- Required first proof: one-edge-at-a-time current-V3 seed/progression/asset/economy negatives reject before
  mutation while a valid round trip remains unchanged.
- Required failure preservation: unchanged Core/export and host load count/metadata for Continue and explicit load.
- Policy: CRA-POL-006 requires strict current V3 rejection without repair/rebind, version migration or public-shape
  drift; private Boss/tactical state stays excluded.

### Initialization Result

- Required Child documents initialized: `PLAN.md`, `EXPERIMENTS.md`, `EXPERIMENT_NOTES.md`.
- Master Phase 02 coordination state changes from `ready` to `active`; no other Child is active.
- Production path investigation: not started.
- Repository/target dirty-state inspection: not started.
- Test executions: none.
- Production/test/canonical mutation before initialization: none.

### Next Experiment

Record the activation dirty baseline, discover the exact current writer/validator/restore/load paths and owning
targets, inspect every affected pre-existing diff, then establish nonzero current valid and negative baselines.

## 2026-08-09 — E01 Current Authority And Dirty Baseline

### Current Authority And Consumers

- `core_enhanced/src/game/world/state.rs` owns the strict `RunCheckpointPayload` field set.
- `core_enhanced/src/game/world/node_flow.rs` owns capture, explicit-load admission, restore and the current
  `validate_run_checkpoint_payload` predicate.
- `core_enhanced/src/game/world/active_run.rs` owns Core payload V3 serialization and Continue admission. Continue
  validates, then adopts `run_progression.run_seed`, initializes root/UUID state and restores.
- `MapProgression`/`RunMap`, `EmployeeRoster`, `Inventory`/`ItemSlot`, `SkillFragmentInventory` and `UuidManager`
  jointly own the missing graph predicates.
- `game_server/src/game/player_game_actor/handlers.rs` passes opaque store bytes and host monotonic metadata to
  Continue; `active_run_store.rs` owns only envelope/checksum/current-backup behavior.
- `game_server/docs/unity_bridge_protocol.md` is the nearest host canonical. Its Core payload-version sentence is
  stale (`2`) against the current source/tests' V3 and requires alignment if this Phase completes.

### Dirty Baseline

- Commit remains `f318414fb0156a67e04c5479000c2a2683fc0c09`.
- Post-test-initialization full status: 375 entries; SHA-256
  `67c2c3b4e0cf16c466856c1b0cf8945af0877cfc03e8a4dea5d0a9952a3a7432`.
- Pre-existing relevant modified paths include `active_run.rs` (18+/7- V3 work), `node_flow.rs` (1+/1- unrelated
  deployability), `employee.rs` (22+ weapon deployability), `item_slot.rs` (5+ equipment-type query),
  `tests/mod.rs` (105+/5- fixture/runtime work), server handlers (73+/13-) and the Unity bridge canonical (13+).
- Clean prospective domain targets at discovery: `uuid_manager.rs`, `map/progression.rs`, `inventory.rs` and
  `skill_fragment.rs`. Their pre-implementation SHA-256 values are respectively
  `f070f3da7d3d3184a1b4c06338bedb73a1c70d60396efa4392319630bb7eed9f`,
  `8c1e5f8582e272da2a93a7ec219c48cc472d089651c2b195a71c91abeffa2724`,
  `d6671321772b9b82717693c9cbfa323661ffdce2447863950edaf91033791907` and
  `97c260625ad85d56bd9a43e1926534d0c3e19ce74d4bd3f0cbff4d481f88e644`.
- Pre-implementation modified-target hashes: `node_flow.rs`
  `d57a26cc93978537d27f94c1239083b2628f3ef70bd2e178c6bd9711f0e28be2`, `active_run.rs`
  `fba27c0ac8cacdeedb5728c07b98ba0eca69401536ab3c655784dcb147914810`, `employee.rs`
  `167375edb575056499aeae2bea3050d609867a57df1ad116ffcf4d5dd051048b`, `item_slot.rs`
  `322d0b862b9f64423153886f65649300cf34029308ae48d84e95e1bf150ceb63`, and pre-module-addition
  `tests/mod.rs` `412ad89eb50dbafb4ab57c6f5bdf4d5f88eb621195f70c39523d2b681458c44a`.

### Overlap Decision

The user-owned changes are identifiable: V3 deserialization/version behavior, weapon deployability and unrelated
battle/server changes. This Phase can preserve them by adding private validation predicates and a separate
checkpoint test module. It must not rewrite the V3 reader, deployability methods or server mutation semantics.

## 2026-08-09 — E02 Valid Baseline And Failing One-Edge Matrix

### Baseline Commands And Result

```text
cargo test -p game_core active_run_checkpoint_round_trip_preserves_monotonic_loads_without_local_profile --lib -- --nocapture
cargo test -p game_core checkpoint_load_rejects_non_exact_recruitment_ownership_without_partial_restore --lib -- --nocapture
```

Both filters executed exactly 1 test and passed with 0 failed/ignored; 929 library tests were filtered in each run.
The valid V3/monotonic path and the existing recruitment ownership negative are therefore live, but neither covers
CRA-F-C01.

### Test-First Matrix

- Added isolated `world/tests/checkpoint_validation.rs` and one module declaration without changing production.
- The fixture is writer-produced V3 with completed SavePoint progression, six bidirectionally equipped starter
  items, positive consumable/material stacks, one owned/equipped active fragment, upgrade/dust state and one pending
  research delivery.
- Twenty one-edge cases cover run/UUID seed, current/completed/available progression, roster map identity,
  equipment map identity/missing/duplicate/reverse/base/type edges, reserve, consumable/material stacks, fragment
  stack/progress/pending and active ownership. A separate case covers owned-copy equip-limit consistency.
- The valid control continues and re-exports with equal JSON meaning and unchanged host monotonic values.

### Pre-Implementation Result

The two matrix tests compiled and failed as intended. Seed disagreements reached
`UuidManager::restore_checkpoint_preserving_counters` and panicked at its debug assertion. Most other one-edge
defects were accepted. Missing equipment and a wrong slot base reached post-restore map/snapshot construction and
returned unrelated `InventoryItemNotFound`/`MissingResource` errors instead of the required pre-mutation typed
checkpoint error. This directly reproduces both the incomplete graph and incorrect failure boundary.

### Decision

Implement one context-aware Core admission graph. Continue validates payload-internal seed equality then adopts the
seed; explicit load/export additionally require the active Core seed. Domain-owned private validators will check
progression, serialized map identities/stacks and cross-root equipment/fragment invariants. No repair, rebinding,
schema change or server-side duplicate rule is introduced.

## 2026-08-09 — E03 Shared Admission Graph And Focused Proof

### Implementation

- Added private/domain-scoped checkpoint predicates for `UuidManager`, `RunProgression`/`MapProgression`,
  `EmployeeRoster`, `Inventory`/`ItemSlot` and `SkillFragmentInventory`; no serialized field or public wire shape
  changed.
- `GameCore::validate_run_checkpoint_payload_with_seed` now composes those authorities before restore. Continue
  requires payload-internal run/UUID seed equality and may then adopt that seed; explicit load additionally checks
  the active Core run seed.
- The cross-root economy pass checks roster map identity, inventory stack/reserve rules, bidirectional equipment
  ownership, slot base/type identity, active-fragment ownership/compatibility and equip limits. Fragment profile
  defects are classified at the fragment boundary before equipment compatibility is evaluated.
- Continue now calls the Continue-specific admission entry point before initializing any destination roots.

### Fixture Correction

The first full library run exposed two existing tests whose shared `force_first_available_node` helper constructed a
state the production map transition cannot produce: an already-`Completed` predecessor was omitted from
`completed_node_ids`, and unrelated nodes remained `Available` while absent from `available_node_ids`. The helper
was corrected to rebuild the completed set from the map and make the synthetic available frontier exact. The two
affected filters then passed without weakening the production validator.

### Focused Result

```text
cargo fmt --all -- --check
cargo test -p game_core game::world::tests::checkpoint_validation --lib -- --nocapture
```

Formatting passed after applying rustfmt only to Phase-edited files. The checkpoint module executed 4 tests and
passed 4, with 0 failed/ignored and 930 filtered. Evidence includes the 20-case one-edge matrix on both Continue
and explicit load, valid Continue/re-export equality, owned-copy equip-limit rejection, and an explicit-load-only
case where the payload's two seed fields agree with each other but disagree with the live run. Every failure asserts
the relevant state, seed, UUID counter and load metadata remain unchanged; Continue failures also retain no export.

### Canonical Alignment

- Corrected the host canonical's stale Core payload version from 2 to 3.
- Documented strict pre-mutation V3 graph validation, explicit-load live-seed authority and the no
  repair/rebinding/migration policy in the Unity bridge protocol and game rulebook SavePoint section.

### Next Experiment

Run the full Core library/package, live RON and selected server active-run consumer/persistence suites; then perform
scoped static/diff/shape checks and the independent completion audit.

## 2026-08-09 — E04 Broad Validation And Static Audit

### Validation Results

- `cargo test -p game_core -q`: 983 physical tests passed, 0 failed/ignored. Breakdown: library 934 and integration
  binaries 3 + 4 + 25 + 13 + 4; zero-test binaries were not counted.
- `cargo test -p game_server active_run --lib -- --nocapture`: 13 passed, 0 failed/ignored, 43 filtered. This covers
  the handler's active-run boundaries and all nine active-run store tests selected by the filter.
- `cargo check -p game_core --all-targets` and `cargo check -p game_server --all-targets`: both passed.
- After the last production correction, the focused checkpoint module again passed 4/4. A final test-only audit
  addition directly compared export admission before and after failed explicit load and also passed 4/4.
- `cargo fmt --all -- --check` and affected-path `git diff --check`: passed.

### Static And Compatibility Result

- The current V3 payload field set, envelope, public command/result shape and private Boss/tactical exclusion remain
  unchanged by this Phase.
- Continue and explicit load both validate before their first mutation; the server returns a Core error before its
  persistence branch and continues to treat payload bytes as opaque.
- Scoped legacy search found no V2 Core writer, dual reader, repair/rebind path, migration alias, ignored focused
  test or panic-as-success assertion.
- Expanded final status has 379 entries versus 375 at E01. The Phase-added checkpoint test and Goal evidence account
  for the delta; all identified pre-existing relevant hunks remain present.
- Final affected production/test/canonical hashes:
  - UUID `f33bcbe273b05af15493c6dc0d823bd1717b01142aa0db8c894b61552525d386`
  - progression `135335b9c514d7bb5cb69fe38d7b4fdc6e0affef1a2c9e7ca91f6418a92234dc`
  - item slot `c20b7a68a8f5c88f2f506445a81989b8d27a764fd8890e97e15d45466294a381`
  - employee `b6adeef17585d008e4cd34dd8915dfbbe6ce4069a417c4fa57a864a869017984`
  - inventory `2b32a864fdae8958a0d57610d5193b688fe1699c4ae4f46a5cc43bbb50371de3`
  - fragment `5a0659e03ce5880085270857b00a2193dadcd81eaf59983ec3f9e162eedf94de`
  - node flow `d6924d02975a5a5fa2b3323ad21df44d6e48343bcd6591d2b3c741be11adb525`
  - active run `bea18fa60896df48fd46bc811c93f54773592ebf0dc12bb5b27a9e7855a3c3e4`
  - shared tests `5255416b8fd2a7733629c2421f079dea81a0836c2de57ebad79f1d6064de506f`
  - checkpoint tests `d54ebe7718587f535ac4ba3c0249f1c78d276ad27a5dac753b8d4bb558e7aad9`
  - rulebook `9ad37a2095d34043aaf4d52513477316996bec3c9aa86a1ec1d905ac1dc02fd3`
  - Unity bridge canonical `05cd588f42e098b43bc94a8f22ff77f6c93f0872c72f03deb0ce95f932ad1197`

## 2026-08-09 — E05 Completion Review And Exit

- The completion guide was reread in full before a separate phase-exit pass.
- `COMPLETION_AUDIT.md` traces all nine completion conditions through CRA-POL-006, current runtime, server
  persistence/consumer code, canonicals, dirty preservation and nonzero validation.
- Audit verdict: `Complete / High`; improvement class: `none`; no material finding or policy question remains.
- Child status changes to `complete` and `Implementation Complete: true`.
- Phase 03 may become `ready`, but no Phase 03 production work is authorized until its required documents are
  initialized and read completely.
