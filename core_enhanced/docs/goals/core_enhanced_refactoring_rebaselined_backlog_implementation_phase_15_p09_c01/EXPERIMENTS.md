# Phase 15 CRA-P09-C01 Experiments

Status: complete
Implementation Complete: true

## 2026-08-10 — E00 Activation And Required-Document Initialization

### Authorization And Dependency

- The user's Master execution request remains active and authorizes the single active Child.
- Phase 14 CRA-P09-C02 has a current `Complete / High` audit with improvement class `none`.
- Phase 01 CRA-P09-C08 supplies trustworthy exact-cause negative fixtures.
- CRA-POL-003 explicitly approves strict rejection of formerly constructible invalid projectile/basic values;
  CRA-POL-006 forbids an unapproved public/schema migration or compatibility shim.
- Phase 15 is next in the exact registry. Phases 16~44 remain pending and unauthorized.

### Evidence Carried Into Initialization

- CRA-P09-C01 remains correctness correction with `2/3/2/3/3/2/unknown = 15/18`.
- Historical defect: skill validation omits zero speed and programmatic optional zeros; basic/weapon validation accepts
  target-locked skill-only fields the runtime ignores.
- Target authority: one typed semantic publication contract shared by RON/programmatic candidates, with positive
  scalars, explicit delivery/consumer compatibility, field-specific provenance and atomic whole-candidate failure.
- Required first proof: RON/programmatic zero speed/limits plus exhaustive basic/weapon ignored-field exact errors;
  positive flight and every current live bundle remain valid.
- Terminal delivery, targeting queries/usefulness/preview, committed damage and motion extraction remain later or
  completed external seams.

### Initialization Result

- Required Child documents initialized: `PLAN.md`, `EXPERIMENTS.md`, `EXPERIMENT_NOTES.md`.
- Master Phase 15 coordination changes from `ready` to `active`; no other Child is active.
- Phase 15 production status/source/data investigation: not started.
- Test executions: none.
- Production/test/data/canonical mutation before initialization: none.

### Next Experiment

Read all three Child documents and the complete Phase 14 audit. Then capture expanded dirty status and prospective
target hashes before tracing current validators, tests, consumers or live data.

## 2026-08-10 — E01 Required Reads And Dirty Baseline

### Read And Dependency Gate

- All 277 lines across the Phase 15 Plan, Experiments and Notes were fully read before production/status/data
  inspection. The complete 246-line Phase 14 completion audit was then fully reread.
- Child and Master both record Phase 15 `active`; Phase 14 is `complete — Complete / High`; Phases 16~44 remain
  pending. CRA-POL-003/006 and Phase 01 exact-cause prerequisite are satisfied.

### Dirty Boundary

- HEAD remains `f318414fb0156a67e04c5479000c2a2683fc0c09`.
- Expanded post-initialization status: 443 entries; normalized SHA-256
  `31f2c5de78e8d2bf9698ba51aae95a9e3e728254914a5c65e63ff5a6424ffd71`.
- Prospective fingerprints before Phase 15 content mutation include:
  - ability/delivery types `6e3f14a91945fd7554521e1113811d1948c2350d6a0593b995c2542c035840e8`
  - skill validation `b70a9c87c56058027af82daace21b7d2090ed90fa1f99b281512a2218063418b`
  - abnormality data `d0bf4bdbea205ff68fb7ea55a50b573a4505185447fea95ff345a052f394f148`
  - equipment data `8e96dd1b531e66a5c8eec9e16f44e55b7c1b0934b2dc3699909f26d503263662`
  - live RON loader `343cf23fbfa427748af7b947bee6be147b7cf76924b99ac2d4c1275d07ce3897`
  - data builder `70f6fbf61cb9e5e26e1aea51df94656998a7c3d4530c4362a1ab3a3b8b535e64`
  - battle types `5d714af9d66bbdadfb0a130238210826512e72a04ae9e7baedffe5d3816f01e3`
  - projectile math `e96d9f1d84daa94cae7640c6606dba029510d3e353f9944ee61b0a08ba8b1a4c`
  - commands/runtime `08c773999dac25f4e5bea3c35971b2c5aa24017b6a69d98d335cb9fd865c465a`
  - RON integration `75558ff28be83cafcfe535252f552903b2077249c2cd81ef00b41d63195854b3`
  - live skill catalog `bf475aa3fb2e6fb81fc28a291cfaab8b81b937fb492a8ed8af27480a7f358d8a`
  - skill runtime integration `afd09b3f6392e8c0175a10d5be3781638e6221c956a55e8c4d129f80c1b49c58`
  - data canonical `4c30e36c12b4e1025e8c5c928ad373cc5b8356e5bacc500076f3514c33987786`
  - target canonical `631a36443745301231af6e93899b1e34a07846d7156dacf54fc3148b633d6e43`
  - rulebook `6c53ecd5d0588aace1a2b043b31434d004327d60938ecf61066ac19acfd31951`
  - live Skill RON `4087df054a663e20ddab14f184a206d532f8df7b472c656bbd708835298f1770`
  - live Abnormality RON `3d7ffdb9cf4bbe6fadb09661dc4c4658cafb699885f35f0e3efc656d59a45de2`
  - live Equipment RON `322d7455a256082f03804c0dcc7a77a4caa3a06919d5e34b667042b411bcd1a6`.
- Abnormality data, projectile math, live skill catalog and Abnormality RON are clean. The other prospective tracked
  paths are already modified by user/campaign work and remain user-owned until exact scoped diffs are reconciled.

### Next Experiment

Inspect existing scoped diffs first, then trace the actual semantic validation/publication and runtime-consumer graph
and run nonzero baselines before adding any Phase 15 test.

## 2026-08-10 — E02 Publication Graph And Green Baseline

### Current Graph

- `DeliveryDef::Projectile` exposes speed plus target policy, targetless, range/lifetime, kill/pierce and collision
  fields. Optional scalar serde helpers reject some RON zeros before database semantic validation; speed has no
  positive gate and programmatic optional zeros bypass those serde helpers.
- `SkillDatabase` resolves raw RON and invokes `validate_contract` from its custom `Deserialize`; official bundle
  parsing then invokes domain validation again. The early semantic failure is stringified as a parse error, so an
  invalid RON skill can lose typed object/path provenance before the official domain gate.
- Skill validation checks target-locked versus directional policy and collision combinations, but does not validate
  speed or programmatic zero range/lifetime/kill/pierce values.
- `BasicAttackDef` rejects `TileArea` and `DirectionalCollision` but accepts targetless, stop-policy and nondefault
  collision fields. `WeaponCombatProfile` rejects only `TileArea`. Their runtime projectile path destructures only
  `speed_units_per_ms`; all other projectile policy fields are ignored.
- Official abnormalities/equipment/skills are parsed into one candidate and `validate_domain_catalogs` maps
  `DataValidationError` into typed `LiveRonError` before publication. Programmatic `GameData` construction also runs
  the domain contracts. Current valid live basic/weapon projectiles author only positive speed and default
  target-locked policy fields.

### Pre-Red Baseline

- `cargo test -p game_core game::data::skill_data::tests --lib`: 13 passed, 0 failed.
- `cargo test -p game_core game::data::abnormality_data::tests --lib`: 4 passed, 0 failed.
- `cargo test -p game_core game::data::equipment_data::tests --lib`: 8 passed, 0 failed.
- `cargo test -p game_core --test ron_loading`: 25 passed, 0 failed.

### Decision

The defect is current and the approved strict cutover requires implementation. The smallest authority is a shared
typed `DeliveryDef` semantic validator for positive projectile scalars and basic-consumer compatibility, mapped by
skill/basic/weapon database validators into their exact owner paths. Official Skill RON must reach the outer typed
domain gate instead of stringifying semantic validation inside raw deserialization; direct `SkillDatabase` serde
will retain its current fail-closed behavior through the same validator.

### Next Experiment

Add exact otherwise-valid RON/programmatic zero scalar proofs and basic/weapon compatibility matrices, then capture
the historical red results before production mutation.

## 2026-08-10 — E03 Exact Historical Red

### Tests Added Before Production Mutation

- Programmatic official-candidate table for zero speed, range, lifetime, kills, pierces and collision hits.
- RON official-candidate table for the same six zero scalar cases.
- Programmatic official-candidate matrix for basic and weapon directional, targetless, range/lifetime, kill/pierce,
  every nondefault collision field and TileArea compatibility.
- RON parity proof for an ignored `max_kills` policy on a basic attack and projectile weapon.
- Every assertion pins source file, domain, object ID, full field path and exact cause.

### Red Results

- `programmatic_skill_publication_rejects_every_zero_projectile_scalar_with_exact_provenance`: failed because the
  speed-zero whole candidate returned `Ok(())`.
- `ron_skill_publication_rejects_every_zero_projectile_scalar_with_exact_provenance`: failed because RON speed zero
  parsed and domain validation returned `Ok(())`.
- `programmatic_basic_and_weapon_publication_reject_every_ignored_delivery_field`: failed because directional basic
  delivery reported only `items[0].basic_attack`, not the exact
  `items[0].basic_attack.delivery.Projectile.hit_policy` path.
- `ron_basic_and_weapon_publication_share_exact_delivery_compatibility_errors`: failed because authored basic
  `max_kills: Some(1)` passed the whole candidate.
- All four filters selected exactly one test; failures are intended contract mismatches, not fixture setup,
  compilation, empty selection or an earlier unrelated validation cause.

### Next Experiment

Implement one typed scalar/basic-consumer delivery authority, map it through skill/basic/weapon owner paths, and let
official Skill RON semantic validation reach the typed domain gate while preserving direct serde fail-closed
behavior.

## 2026-08-10 — E04 Shared Authority And Focused Green

### Correction

- `DeliveryDef` now owns one typed semantic check for positive projectile speed and every present optional
  range/lifetime/kill/pierce/hit scalar. Optional RON zeros reach this gate instead of failing in field-local serde
  with lost owner provenance.
- The same authority owns the basic-attack consumer matrix: only Instant or a positive-speed default TargetLocked
  projectile is accepted; directional, targetless, every stop field, each nondefault collision field and TileArea
  return an exact field suffix and cause.
- Skill validation maps scalar failures into `skills[i].steps[j].delivery.*`. Basic/corroded and equipment
  validators map the shared errors into their exact object-owned paths. Weapon runtime application retains its
  defensive `GameError` wrapper over the same typed validator.
- Official Skill RON now parses/resolves an unvalidated raw candidate and reaches outer domain validation before
  publication, preserving `LiveRonError` object/path/cause. Direct `Deserialize<SkillDatabase>` still invokes the
  same semantic validation and remains fail-closed.
- No public Rust field, RON variant/field/default, checkpoint, DTO, wire or runtime event shape changed. Current live
  RON needed no rewrite.

### Green Results

- Four exact red filters: 4 passed, 0 failed, one selected test each.
- Skill data module: 13 passed; abnormality data: 4 passed; corroded employee data: 6 passed; equipment data: 8
  passed; live RON unit family: 70 passed; `ron_loading`: 25 passed.
- Projectile flight arithmetic including positive ceiling division: 3 passed; weapon interval ceiling/override: 1
  passed; skill runtime contract: 14 passed; live skill catalog audit: 4 passed; live item skill activation: 4 passed.
- Current live skill/basic/weapon catalogs therefore publish without content changes, and valid TargetLocked,
  DirectionalCollision, kill/hit/lifetime interaction and weapon override behavior remain green.

### Canonical Update

- `data_loading_contract.md` now states the atomic positive scalar gate, typed provenance and strict basic/weapon
  compatibility boundary.
- `skill_target_contract.md` now places the publication rule immediately before projectile runtime lifecycle.

### Next Experiment

Run the complete Core library and affected integration/release/server/static matrix, inspect scoped diffs and hashes,
then prepare the independent completion audit if no regression or compatibility migration appears.

## 2026-08-10 — E05 Broad Validation And Compatibility Audit

### Raw Candidate Test Rebaseline

- The first complete Core run selected 978 tests: 977 passed and the sole failure was the former
  `projectile_collision_ron_rejects_zero_max_hits` assertion that a partial raw collision struct must fail during
  serde.
- That assertion contradicted the corrected typed-publication boundary: raw syntax may form an unpublished
  candidate, while owner-aware semantic validation must reject it before publication. It was rebaselined to require
  raw parse success followed by the existing semantic `max_hits must be non-zero` failure. This is not a permissive
  publication branch; official Skill/basic/weapon candidates still fail with exact owner/path/cause.
- Focused rebaseline: 1 passed. Repeated complete Core lib: 978 passed, 0 failed.

### Release, Server And Static Results

- Release Core lib: 978 passed; release skill runtime contract: 14 passed; release RON loading: 25 passed.
- `cargo test -p game_server --all-targets`: 59 library tests passed and all binary/example targets selected zero
  tests without failure.
- `cargo check --workspace --all-targets`: passed for Core and server.
- `cargo fmt --check` and scoped `git diff --check`: passed.
- The RON zero table and RON basic/weapon parity proof were additionally run through the full
  `GameDataBase::load_live_sources` publication entrypoint: 2 filtered tests passed. Invalid candidates returned
  before `GameDataBase` construction.

### Static And Compatibility Audit

- Shared scalar authority has one definition and the skill validator is its direct publication consumer. Shared
  basic-consumer authority has one definition and exactly two domain consumers: `BasicAttackDef` and
  `WeaponCombatProfile`.
- Basic projectile runtime still destructures only `speed_units_per_ms`; the newly rejected fields therefore match
  the previously ignored set. Skill runtime policy/collision and defensive zero guards were not changed.
- The remaining `deserialize_nonzero_option_u32` belongs to positive TileArea tick interval authoring, not projectile
  range/kill/pierce fields. Removed u8/u64 projectile serde helpers have no remaining reference.
- `DeliveryDef`, `BasicAttackDef` and `WeaponCombatProfile` public field/variant/default and serialized shape are
  unchanged. They appear in employee snapshot DTOs, but this Phase changes only pre-publication validity and typed
  error provenance; no DTO, server message, checkpoint or event definition changed.
- Live Skill, Abnormality and Equipment RON hashes remain exactly their pre-Phase values; no content migration or
  rewrite occurred.

### Dirty Boundary Refresh

- Expanded status is 445 entries, normalized SHA-256
  `d12b9e27a48eb6c4bd2fa57ed97593fd9bd87f40a1d3c5dc1be219c61ed466c7`. The increase from 443 is exactly the
  previously clean abnormality and corroded-employee source files becoming scoped Phase 15 edits; no test artifact
  or unrelated untracked file was added.
- Current scoped fingerprints:
  - ability `fc4b44d47132e334c308fa9619f62d76dd473ac9b7a51fcb7bfe6358d7488e25`
  - skill data `843490305ef8c168db37f20d6bb2b5ab88202e7111248d1c79b63071e6a4517f`
  - abnormality data `1159cf58a6da531afac72b3fc8bfdde262491f76603a11fecc989f4ff79c999b`
  - corroded employee data `1197b2da1df08d89526949e209f3bf71261e1d4b5c4cf7b094cf39c4256d20da`
  - equipment data `7b6a60a538dd2897bc53399e4227a1cad002a09b000d52b6705ca7afe6719559`
  - live RON loader/tests `ed82df50a763dff27391e4d8fb7116fe24b0854ef75e4e4edc01c47c00948dab`
  - data canonical `27cff557c7a85942e4ae5ed4bcbc68a7266897934e1bc90f7f83330dc84e48e1`
  - target canonical `17e8f36d2ac72902e51fd522155217eec1948da7d7ada6a4ec4fbf1bca375f2d`.

### Next Experiment

Refresh final document-inclusive hashes, close the Child evidence matrix, then fully reread the completion-review
guide and perform the independent phase-exit audit before any Phase 16 activation.

## 2026-08-10 — E06 Independent Phase-Exit Audit

- All 567 lines of `core_enhanced/docs/goal_completion_review_guide.md` were reread after implementation and broad
  validation completed.
- The audit independently rechecked current validators, official loader order, runtime-consumed fields, exact
  negative matrices, live data hashes, serialized consumer boundary, canonical text, scoped dirty state and the
  Phase 14/29 seams.
- `COMPLETION_AUDIT.md` records a complete ten-row evidence matrix with every row passing, no material
  cross-component finding, long-term fit `High`, improvement class `none`, and primary verdict `Complete`.
- Audit-inclusive expanded status is 446 entries with normalized SHA-256
  `224ef55530ffddedf1711e83728253523561e429d2ca99beb2425e61dfd5ff49`; the only post-implementation entry is the
  required audit document.
- Phase 15 transitions to `complete`, `Implementation Complete: true`. Master may record
  `complete — Complete / High` and only then activate Phase 16.
