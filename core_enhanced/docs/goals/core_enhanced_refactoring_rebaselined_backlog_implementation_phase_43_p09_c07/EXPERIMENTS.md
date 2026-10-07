# Phase 43 CRA-P09-C07 Experiments

Status: complete
Implementation Complete: true

## 2026-08-11 — E00 Goal Initialization

### Hypothesis

Historical evidence names serialized/copied positive `range_units`, a public no-caller range helper and a public
base-UUID delivery helper outside current production authorities. The now-complete P09 targeting/delivery chain and
user-approved cleanup may make some or all surfaces removable, but only a new current graph, data/fixture migration
inventory and retained-authority tests can establish the exact scope.

### Required preserved boundary

- `TileRangePolicy` and the typed tile membership/materialization authority remain eligibility truth.
- Current deterministic target choice, explicitly including `Nearest`, remains exact.
- Runtime resolved equipment/profile remains delivery truth; base metadata does not override it.
- Live RON, preview, DTO/wire, save and external shapes change only where the same candidate is proven ghost and the
  approved migration is complete.
- No compatibility shim, serde alias/default, world-distance fallback or Phase-44 work is introduced.

### Initialization evidence

- Phase 42's 225-line audit at `dc2abc41...` is `Complete / High`; Master Phase 43 is `ready`.
- Original P09 C07 catalog/boundary, post-Boss P09 reaudit and final catalog/order/wave/test records were reviewed.
- The user approved remaining serialized/public cleanup after inventory and separately fixed the retained
  tile-range/`Nearest` behavior.
- Pre-Child expanded state is 594 rows at `e44f1ce8...`; Phase 43 path was absent and no Child PLAN was active.
- Protected marker remains exact at 57 bytes and `614d2fbd...`.

### Decision

Initialize exactly PLAN/EXPERIMENTS/EXPERIMENT_NOTES with no current candidate-source inspection, test execution or
production mutation. Activate only after all three initialized documents and Master registry are synchronized.

## Next Experiment

After activation, fully reread all active documents and capture filename-only prospective fingerprints for range/
delivery definitions, publication adapters, runtime targeting, projections, live data, tests, external fixtures,
canonicals and manifests before opening their content.

## 2026-08-11 — E01 Sole Activation

- The initialized Child contains exactly the required three documents and explains the 594→597 expanded-status
  increase at canonical C-sort SHA-256 `d981d0ee...`.
- All three headers change from initialized to active; `Implementation Complete` remains false. Master Phase 43
  changes from ready to active as the sole Child; Phase 44 remains pending and absent.
- No current candidate production/data/test content was inspected and no test or production mutation ran during
  initialization/activation.
- Next gate: fully reread all active documents and capture filename-only prospective fingerprints before current
  source content discovery.

## 2026-08-11 — E02 Active Reread And Prospective Fingerprints

### Procedure

- Fully reread all three active Child documents.
- Used filename-only discovery and mechanical line/hash/status capture for prospective definition, publication,
  targeting, delivery, range projection, live-data, test, server, manifest and canonical carriers without opening
  current candidate content.

### Activation state

- Expanded status remains 597 rows at canonical C-sort SHA-256 `d981d0ee...`; Phase 43 is the sole active Child and
  Phase 44 is absent.
- Prospective Core definition/publication carriers are already dirty: `ability.rs` 1,251 `fc4b44d4...`,
  `equipment_data.rs` 1,012 `7b6a60a5...`, `abnormality_data.rs` 596 `1159cf58...`,
  `corroded_employee_data.rs` 607 `1197b2da...` and `skill_fragment_data.rs` 710 `c2a7d63c...`.
- Prospective targeting/delivery carriers are already dirty: `basic_attack.rs` 478 `071b9a45...`, `targeting.rs`
  704 `44e9af52...`, Core `mod.rs` 8,541 `a17dff27...`, runtime `types.rs` 541 `b4b608f0...`, `tile_range.rs`
  403 `73930102...` and `range_preview.rs` 1,172 `1a8fce57...`.
- `ron_loading.rs` is already dirty at 1,971 lines and `75558ff2...`. Official equipment RON is dirty at 1,988
  lines and `322d7455...`; prospective abnormality, corroded-employee and skill-fragment RON files are clean at
  `3d7ffdb9...`, `28300ee4...` and `b39c715e...`.
- Prospective server handlers/messages/state are already dirty at `431abf19...`, `8e3c5369...` and `85fe7208...`.
- Workspace/Core/server manifests, lockfile and Core root retain `98c9cf4d...`, `40d8a9ec...`, `19b41cd...`,
  `4223318b...` and `3fbfdca2...`. Runtime/gameplay/data/server canonicals retain `ab7d2f98...`, `6c53ecd5...`,
  `30ed5fd5...` and `c82000b1...`.
- Protected marker remains 57 bytes at `614d2fbd...` with exact recorded timestamps.

### Decision

Candidate carriers heavily overlap earlier completed-campaign dirty work; activation hashes are the attribution
boundary. Current symbol/value/serde/caller/data/external/authority discovery may now begin. No prospective path is
presumptively removable.

## 2026-08-11 — E03 Current Range, Delivery And Consumer Inventory

### Candidate surfaces

- Public serde `BasicAttackDef.range_units` has an additional required authored-input field, a default, conversion
  copy and no runtime reader. Official writers are 30 abnormality RON rows, 6 corroded-employee RON rows plus test/
  programmatic constructors.
- Public serde `WeaponCombatProfile.range_units` has a default, positive validator and copy into
  `BasicAttackDef`, but neither value is read after the copy. Official equipment RON has 30 writers; one integration
  test reads the value as catalog shape only.
- Public `BattleCore::basic_attack_delivery(base_uuid)` has zero caller and reads base abnormality metadata, so it
  can disagree with an employee's runtime equipped weapon. The retained Core-private
  `basic_attack_delivery_for_unit` reads the runtime unit profile and is used by both real attack-start/release and
  test resolution paths.
- Public `spatial::bodies_in_range` has only its own unit test. Its public `UnitBody::can_reach` leaf likewise has no
  caller except that wrapper and its own unit test. Both encode a scalar world-distance eligibility concept that the
  current battle targeting graph does not use.

### Retained authorities

- `basic_attack_range_policy_for_unit` captures runtime delivery, tile policy, effective tile pattern and air
  capability from the runtime unit.
- `is_basic_attack_target_in_range_with_policy` filters target capability and delegates solely to
  `TileReachQuery::contains`; `range_units` is absent from the path.
- Basic target selection preserves fixed-route blocker/current-target rules and profile-specific deterministic
  ordering. Skill `UnitTargetRule::Nearest` first retains a current eligible/useful attack target, then selects by
  world-distance squared and UUID tie-break among candidates admitted by the typed tile query.
- `UnitCombatProfile::apply_weapon_profile` supplies tile pattern, targeting, timing and delivery to the runtime
  profile. Release reads only that resolved runtime unit.

### Data and external consumers

- Exact Core/server/tool source search finds no DTO, save/checkpoint, server or Unity-facing read of the two scalar
  fields or removed-helper candidates. Phase-10 server sources contain no occurrence.
- Accessible `/mnt/f/work` Rust/C#/RON/config/script search, excluding build/VCS/docs, finds no consumer outside this
  repository. It reproduces only the classified definitions, official RON writers, tests and unrelated skill-
  legacy rejection/generic skill-test range helper.
- The user's approved serialized/public break therefore applies: remove both scalar fields and all their authored/
  copy/validation/test writers, the false base-UUID delivery helper, and the two disconnected continuous-reach
  helpers with no aliases/defaults/shims.
- Retain unrelated skill negative `range_units` rejection text and the `single_step_skill` local radius convenience;
  they are not BasicAttack/Weapon fields.

### Canonical disposition

- `data_loading_contract.md` currently lists `range_units` as a required authored basic-attack field and must be
  migrated with the strict schema.
- `skill_target_contract.md` already states tile pattern, not scalar world distance, owns basic eligibility; remove
  the obsolete field name from its retained explanation while preserving the policy.

## 2026-08-11 — E04 Pre-Cleanup Assertion Transfer And Baselines

### Retained controls added with the old surface present

- `equipped_weapon_profile_is_runtime_basic_attack_delivery_authority` proves applying a projectile weapon changes
  the resolved runtime basic-attack and stored weapon profiles from Instant to the exact projectile delivery.
- `basic_attack_release_delivery_reads_runtime_unit_profile` proves the release helper returns Projectile from the
  runtime unit profile independently of base-UUID metadata.
- Both select and pass 1/1 with the old public/base and scalar surfaces present. Core library count increases from
  1,009 to 1,011.

### Existing nonzero baselines

- all `range_not_range_units` controls: 3/0/0;
- lowest-UUID fixed-defense target inside tile range: 1/0/0;
- current `Nearest` retained-target/fallback controls: 2/0/0;
- weapon tile-pattern preview override: 1/0/0;
- typed tile-reach focused filter: 1/0/0;
- official live RON integration: 25/0/0.

### Diagnostic disposition

The first attempted focused batch failed to compile because the new battle-types test omitted its local
`DeliveryDef` import. This was a test-authoring error, not a runtime baseline failure; no behavioral test ran in
those failed binaries. The import was added and every listed control then selected nonzero and passed. The separate
RON integration had already passed 25/25.

### Decision

Target eligibility/ordering, current `Nearest`, runtime equipped delivery, preview and official authored data are
protected with the old surface present. Apply the approved strict field/data/helper cleanup, then rerun the same
controls with field-named tests retitled to retained tile-policy observations.

## 2026-08-11 — E05 Approved Strict Cleanup And Data Migration

### Removed candidate surface

- Removed public serde `BasicAttackDef.range_units` and its required authored-input field, default, conversion copy
  and every programmatic writer. `BasicAttackDef` now denies unknown fields, and a direct negative control proves
  the removed key fails closed.
- Removed public serde `WeaponCombatProfile.range_units`, its default, positive validator, runtime-profile copy and
  every programmatic writer. The existing strict profile schema plus a direct negative control reject the legacy
  key.
- Migrated exactly 66 official writers: 30 abnormality rows, 6 corroded-employee rows and 30 equipment rows. Exact
  executable/data search finds zero remaining `range_units:` writer. No alias, ignored key, permissive default or
  dual schema was introduced.
- Removed zero-caller public `BattleCore::basic_attack_delivery(base_uuid)`. The production
  `basic_attack_delivery_for_unit` remains Core-private and reads the resolved runtime unit for attack start and
  release.
- Removed zero-caller public `spatial::bodies_in_range` and its only leaf `UnitBody::can_reach`. The scalar reach
  assertions were retired while the same spatial test continues to protect world-position distance calculation.

### Retained behavior and canonicals

- `basic_attack_range_policy_for_unit`, `is_basic_attack_target_in_range_with_policy`, `TileReachQuery`, all tile
  pattern/facing/edge logic and the current `Nearest` candidate scoring/tie-break are unchanged.
- `UnitCombatProfile::apply_weapon_profile`, the private per-unit delivery resolver and instant/projectile mechanics
  remain the only runtime delivery path. The two pre-cleanup authority assertions remain green after cleanup.
- `data_loading_contract.md` drops the obsolete required authored field. `skill_target_contract.md` now states the
  retained tile-policy authority without naming the removed scalar field.
- Remaining executable `range_units` occurrences are deliberately unrelated: two skill-schema legacy rejection
  strings and the local `single_step_skill` radius parameter/use. The new BasicAttack/Weapon negative controls also
  contain the removed key only as rejection input.

### Scope and dirty attribution

- Manifests, lockfile, Core root, base attack runtime, Core runtime types, tile-range implementation, range preview,
  skill-fragment schema/data and all server carriers are unchanged from their activation fingerprints.
- Five formerly clean paths became dirty only for this Phase: `battle/core/spatial.rs`,
  `combat_setup/defense_object.rs`, `tests/skill_test/common/types.rs`, abnormality RON and corroded-employee RON.
  Their complete Git diffs are respectively the helper/test cleanup, one removed constructor field, one removed
  fixture field and exact official key deletion. All other edits overlap activation-dirty carriers and are bounded
  by their prospective hashes.

## 2026-08-11 — E06 Post-Cleanup Focused, Broad And Static Validation

### Focused parity

- Exact runtime weapon-authority and runtime-unit delivery controls pass 1/1 each.
- Exact fixed-defense tile target order and weapon tile-pattern preview controls pass 1/1 each; current `Nearest`
  controls pass 2/2; typed tile reach passes 1/1; the retained skill legacy scalar rejection passes 1/1.
- Targeting, movement-planner, range-preview, abnormality-data, equipment-data and tile-range modules pass
  12/12, 25/25, 13/13, 5/5, 9/9 and 3/3.
- Removed BasicAttack/Weapon schema controls pass 2/2. Official live RON passes 25/25; affected skill runtime and
  skill suite integrations pass 14/14 and 4/4.
- Two first post-cleanup guessed filters selected zero because their historical short names were stale. Those runs
  are rejected as evidence; the exact current names were obtained from the 1,012-name list and rerun 1/1 each.

### Broad profiles and static gates

- Core debug and release all-target suites each pass 1,063 tests: library 1,012 plus integrations
  `4 + 4 + 25 + 14 + 4`, with zero failure/ignore.
- Server debug and release all-target suites each pass 63/63. Workspace debug and release all-target checks pass.
- Normal Core and server Clippy exit 0 with the inherited warning inventory. No warning is introduced by a new
  Phase-43 assertion or removed production surface; the affected skill fixture's existing too-many-arguments
  warning remains while its arity decreases from ten to nine.
- `cargo fmt --all -- --check` and `git diff --check` pass.
- Sorted Core library list is 1,012 rows at
  `66f515f171b5b9bb9fcf2bf756c6fc534d1945c33041160073ad420beae84733`; server remains 63 rows at
  `beca9e7a9162c0a9a1511e30061703d965452e87a9d52f8c0a15cfe0923d4588`.

### Exact absence, fingerprints and status

- Candidate-symbol search finds no definition/caller/writer for `bodies_in_range`, exact
  `basic_attack_delivery`, `can_reach` or either removed public field. Official RON writer search selects zero.
- Post-cleanup definition/runtime hashes include abnormality `149c8ad4...`, equipment `a9a6b06f...`, battle types
  `bfe10a37...`, targeting `140d843b...`, spatial `cd7a4026...`, movement types `58411639...`, planner
  `20178529...` and Core mod `83ee9750...`.
- Official abnormality/corroded/equipment RON are `b608ad1c...`, `f4bf8aea...` and `5f6862bf...`;
  data-loading/skill-target canonicals are `e592fd8f...` and `a35a2024...`.
- Workspace/Core/server manifests, lockfile and Core root remain `98c9cf4d...`, `40d8a9ec...`, `19b41cd...`,
  `4223318b...` and `3fbfdca2...`. Runtime/gameplay/server canonicals remain `ab7d2f98...`, `6c53ecd5...` and
  `c82000b1...`.
- Current expanded status is 602 rows. The reproducible canonical command
  `git status --porcelain=v1 --untracked-files=all | LC_ALL=C sort | sha256sum` gives `a7edd0b7...`.
  Removing the five exact Phase-owned dirty additions gives 597 rows at `bdc1dbde...`, not the `d981d0ee...`
  recorded at activation. The row count, exact five path identities, prospective content hashes and complete
  candidate diffs remain sufficient attribution evidence, but the historical activation status hash is retained as
  a measurement anomaly rather than reused as proof.
- Protected marker remains 57 bytes at `614d2fbd...` with both recorded timestamps unchanged.

### Decision

P43-C01 through P43-C11 are satisfied. Keep the Child active and implementation-incomplete until a separate
completion audit rereads the review guide, reruns direct retained-authority/absence/broad controls and returns
`Complete / High`.

## 2026-08-11 — E07 Independent Completion Audit And Transition

- The separate 210-line `COMPLETION_AUDIT.md` at SHA-256
  `a9b674b4c52f58ad228f4214584f2d9f3c331e4054489ebaf238ee2a0f7a2acb` independently rereads runtime/data/
  canonical/consumer evidence and passes all 12 Child conditions.
- Its audit refresh reruns the two delivery-authority controls 1/1 each, tile target 1/1, current `Nearest` 2/2,
  preview 1/1, tile reach 1/1, strict removed-schema controls 2/2 and official RON 25/25.
- Audit broad debug reruns pass Core 1,063/0/0 and server 63/0/0. Current release/static evidence remains valid
  because only Goal documents changed after it.
- The audit artifact alone changes expanded status 602→603 rows at canonical C-sort SHA-256 `1510b29c...`;
  manifests, canonicals and the protected marker remain exact.
- Primary verdict is `Complete`, fit `High`, improvement class `none`, confidence High. Phase 43 transitions to
  complete and Phase 44 CRA-P10-C06 becomes ready only.
