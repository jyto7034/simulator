# Gameplay RON Strictness And Bundle Validation Experiments

Status: active

Implementation complete: false

Created: 2026-07-13

## 2026-07-13 — Goal initialization and document gate

Hypothesis:

G12 can be bounded around one production gameplay-bundle acceptance/publication authority while
closing DCA-H-04, CPR-H-08, CPR-M-01's remaining early-cardinality portion, and CPR-M-02 without
absorbing later domain gameplay policy.

Documents read before G12 code/data exploration:

- Master PD-09, G12 ownership, Phase 1 order/exit conditions, findings, and current progress.
- Master EXPERIMENTS/EXPERIMENT_NOTES G12 handoffs.
- G04 and R00 completion boundaries for the RunStart fixture/cardinality split.
- G01, G06, and G07 completion reviews for the current Phase 1 authority boundary.
- `docs/goal_completion_review_guide.md`.

Result:

- Accepted policy is sufficient to initialize a bounded child.
- G04 closed only the server fixture regression; supported-mode RunStart cardinality remains G12.
- G12 owns exact generated-RON parsing, shared final bundle validation, deterministic diagnostics,
  and all-or-nothing publication, but not new Event/content behavior or the authoring application's UX.
- No G12 runtime code, live RON, or test search was performed before these three documents were
  created.

Decision and next step:

Begin read-only current-source discovery in the PLAN evidence order. Record the first ownership matrix,
dirty-worktree overlap, and baseline before modifying parser/schema/data.

## Experiment Record Template

For every meaningful trial, append:

- hypothesis,
- smallest attempted experiment,
- changed files or explicit read-only scope,
- exact commands,
- actual nonzero test/file/domain counts,
- success or failure,
- failure cause,
- correction/revert,
- next judgment.

Never record a zero-match filtered test as success.

## 2026-07-13 — Production loader and dirty-worktree discovery

Hypothesis:

The current repository already has a central immutable bundle construction seam that can become the
strict authority, even if parsing and diagnostics around it remain permissive/panic-based.

Read-only scope and commands:

- Scoped `git status --short` across Core and sibling game_server/resources.
- `rg --files` and bounded `rg -n` for RON/data/loader/database/bundle entry points.
- Direct reads of `src/game/data/mod.rs`, `src/game/data/validation.rs`,
  `tests/ron_loading.rs`, and game_server startup.

Observed result:

- Production server startup calls `GameDataBase::load_live_embedded()`.
- The loader parses 18 embedded domain files separately, builds one `GameDataBase`, and returns only
  after `GameDataBase::new()` index/reference validation plus generated preview validation.
- `GameDataBase::new()` is already an atomic local construction boundary; no partially constructed
  database is returned on panic. The missing contract is typed exact-version parsing, complete
  validation coverage, and source-rich error return rather than a competing publication object.
- No schema-version field is present at the loader boundary.
- Parse errors are per-file `expect` panics. Semantic validation uses panics/assertions and usually
  lacks file/field-path context.
- Dirty overlap includes the central data modules and live Event file. No edits were made.

Decision and next step:

Retain `GameDataBase::new()` as the candidate final validation seam. Inventory all typed serde
strictness/default/alias behavior and every database duplicate/index implementation before choosing
the exact version envelope and typed diagnostic API.

## 2026-07-13 — Live file, serde, duplicate, and late-reference inventory slice

Hypothesis:

The current loader's strongest gaps are bounded and reproducible: independently lazy map catalogs,
legacy aliases/defaulted generated fields, first-match pool IDs, incomplete RunStart cardinality,
and Event effect validation that does not reuse reward validation.

Read-only commands and baseline tests:

- Counted and sampled every `game_resources/data/**/*.ron` file: 23 files.
- Enumerated every gameplay `include_str!` loader and domain builtin.
- Searched gameplay serde `alias`, `default`, `deny_unknown_fields`, database indexes, `find`, and
  pool lookup implementations.
- Traced Event effect enums, runtime handlers, reward validation, RunStart validation, and live uses.
- `cargo test -q -p game_core --test ron_loading -- --test-threads=1`: 18 executed, 18 passed.
- `cargo test -q -p game_core game::data --lib -- --test-threads=1`: 78 executed, 78 passed, 551
  filtered out.

Observed result:

- Nineteen files load through the main production data path; four map/battlefield files remain lazy
  domain builtins. A server can therefore finish `GameDataBase` startup without parsing all authored
  gameplay RON.
- Explicit legacy serde aliases exist on `DamageType`; current live data does not use them.
- Many generated structs use serde defaults. Classification remains pending; they cannot all be
  removed mechanically because some encode intentionally optional gameplay meaning.
- Shop pool and reward pool lookup uses first match and does not reject duplicate pool IDs.
- Event-embedded RewardEffects bypass the existing reference/value validator.
- Both Event `ApplyBossOmenStepResult` variants are schema-accepted/runtime-panic and absent from live
  data.
- RunStart checks uniqueness only. It does not prove exactly one Standard and one Endless trigger or
  their required policy.
- Current positive baselines are green but do not cover these negative contracts.

Decision and next step:

Design small negative tests around the existing production/bundle constructors before changing code.
First complete the extra-wave/map/stage/mission reference trace and classify which defaults are
required canonical output versus intentionally optional meaning.

## 2026-07-13 — Exact-version and source-aware parse diagnostic trial

Hypothesis:

One strict bundle manifest plus a shared RON parser can enforce the exact supported version and
preserve source/domain/nested field-path diagnostics without adding a version field to every domain
database.

Minimal change:

- Added `serde_path_to_error` and `src/game/data/live_ron.rs` with `LiveRonError`, shared
  `parse_live_ron`, schema version 1, and strict manifest validation.
- Added `game_resources/data/bundle.ron` containing only required `schema_version: 1`.
- Did not connect the trial parser or manifest to production loading yet.

Command and actual result:

- `cargo test -q -p game_core game::data::live_ron::tests --lib -- --test-threads=1`:
  3 executed, 3 passed, 629 filtered out.
- Tests prove current version acceptance, wrong version rejection, missing field rejection, unknown
  field rejection, and `data/test.ron [test_domain] at entry.count` path capture.
- First run emitted dead-code warnings for deserialize-only nested test fields. Added local
  `#[allow(dead_code)]` to those test fixtures; production code was not weakened.
- Warning-free revalidation: `cargo fmt --all -- --check` passed and the same focused group passed
  3/3 with no warnings.

Decision and next step:

The parser/manifest shape is viable. Next rerun warning-free, then connect a testable 23-source set to
the same parser before replacing production `expect` calls.

## 2026-07-13 — First 23-source parsing trial failure

Hypothesis:

A named `GameplayRonSourceSet` can parse all 23 production files through the same path-aware parser
before publication.

Minimal experiment:

- Added a 23-field embedded source set and typed `ParsedGameplayRon` result.
- Ran the four focused live-RON parser/source-set tests.

Actual result:

- 3 passed, 1 failed.
- `data/map/battlefield_templates.ron [battlefield_templates]: 1:1 Expected struct`.
- Cause: the live file intentionally authors a top-level `Vec<BattlefieldTemplateDefinition>`, while
  the runtime `BattlefieldTemplateDatabase` is an internal wrapper constructed after parsing. The
  trial incorrectly tried to deserialize the wrapper directly.
- The run also reported temporary dead-field warnings because the parsed result is not connected to
  production yet.

Correction:

- Preserve the actual live schema: parse the top-level vector, then construct the internal database
  wrapper explicitly.
- Make the internal template type/wrapper field crate-visible only for this shared source loader.
- Consume all parsed fields in the focused test so an intermediate source-set trial is warning-free;
  do not suppress production warnings globally.

Next step:

Rerun the exact 4-test group. This failed attempt is not success evidence.

Revalidation:

- `cargo fmt --all` succeeded.
- Exact focused group: 4 executed, 4 passed, 0 failed, no warnings.
- The shared source set now parses the actual shapes of all 23 production gameplay RON files only
  after accepting the exact bundle manifest version.

Decision:

Use this source set as the sole production parse input. Next make local/cross-bundle validation
fallible and connect production publication to the parsed result; do not retain the old independent
`from_str(...).expect` assembly beside it.

## 2026-07-13 — First production source-set cutover

Hypothesis:

The production loader can consume the parsed 23-source result without changing current live gameplay
behavior, while immediately bringing all four domain-owned files under startup parse/local-validation.

Changes:

- Replaced the 18 direct `ron::de::from_str(...).expect` calls and production
  `RunPolicyData::builtin()` call in `GameDataBase::load_live_embedded()` with
  `GameplayRonSourceSet::embedded() -> parse_gameplay_ron_sources()`.
- Added fallible local validation for run policy, map generation policy, map node definitions,
  battlefield archetypes, and battlefield templates with source/domain diagnostic context.
- Refactored battlefield-template validation to accept the parsed archetype policy rather than
  consulting its global builtin during validation.

Commands and results:

- `cargo fmt --all`: passed.
- Focused live-RON parser/source-set tests: 4 executed, 4 passed.
- `cargo test -q -p game_core --test ron_loading -- --test-threads=1`: 18 executed, 18 passed.

Current intermediate boundary:

- Server production now cannot finish `GameDataBase::load_live_embedded()` without parsing and locally
  validating all 23 files under exact manifest version 1.
- Existing `GameDataBase::new()` semantic validators still panic/assert; production loader still
  converts `LiveRonError` to startup panic. Typed `Result` publication is not complete.
- The four map/battlefield values are validated then dropped; runtime domain builtins still parse the
  same embedded files later. This residual dual parse is explicitly not completion state.

Decision and next step:

Migrate runtime map/battlefield consumption to the parsed validated objects or an equivalent single
publication object, then remove the old production builtin parse path. Continue typed validation
conversion and add negative bundle tests before changing server startup signature.

## 2026-07-13 — Domain-owned map/battlefield runtime authority cutover

Hypothesis:

The four domain-owned catalogs can remain typed in their owning modules while their parsed values are
published inside `GameDataBase` and consumed by map generation/combat preview, eliminating lazy
reparse without creating a monolithic RON file.

Changes:

- Added `GameplayMapCatalogs` containing the parsed map generation policy, node definitions,
  battlefield archetype policy, and battlefield template database.
- Added the catalog set to `GameDataBaseParts`, `GameDataBuilder`, and `GameDataBase`.
- Production loader moves the exact source-set values into `GameDataBase`.
- `MapGenerator` production path now requires the database catalogs; its no-argument live builtin
  path exists only as a unit-test convenience backed by the same source parser.
- `BattlefieldGenerator` uses `game_data.map_catalogs` for archetype, size, template selection, and
  template-policy validation.
- Removed all four old `builtin()` embedded readers and all `include_str!` calls from map/combat
  preview runtime modules.
- Updated live integration tests to read map catalogs through the loaded `GameDataBase`.

Commands and actual results:

- `cargo fmt --all`: passed.
- `cargo check -q -p game_core`: passed.
- Core map focused group: 23 executed, 23 passed.
- Core combat-preview focused group: 25 executed, 25 passed.
- Live RON integration: 18 executed, 18 passed.
- Legacy search for the four old `::builtin()` calls and map/combat-preview `include_str!`: no
  matches.

Observed result:

All 23 source files are parsed once by the official production source set, validated before return,
and the four formerly lazy values are the exact objects used by runtime generation. Domain type
ownership remains in map/combat-preview; publication/parse authority is unified.

Decision and next step:

Proceed to strict negative contracts and typed semantic validation. Production still panics on
`GameDataBase::new()` semantic failures and must become a typed `Result` before G12 completion.

## 2026-07-13 — First strict unknown-field negative test compile failure

Hypothesis:

The shared source-set parser reports an unknown nested live field with exact source/domain/path, and
removed `DamageType` aliases reject while canonical variants load.

Minimal change:

- Removed the seven legacy `DamageType` aliases.
- Added `deny_unknown_fields` to confirmed live-input gaps across Artifact, Consumable, Buff,
  employee candidate/loadout, Equipment, Reward, Skill raw root/range preset, SkillFragment,
  BossOmen DB, map-node DB, and battlefield policy/template authored structs.
- Added focused alias and corrupted-consumable source-set tests.

First command result:

- Test compilation failed before execution because `Result::expect_err` requires the successful
  `ParsedGameplayRon` type to implement `Debug`.
- This is a test harness issue, not schema behavior evidence. No test executed.

Correction:

- Match the `Result` explicitly and panic only on unexpected success. Do not add broad `Debug`
  derivations to the complete parsed bundle merely for one negative assertion.

Next step:

Rerun the focused group with a nonzero count. The compile failure is not success evidence.

Second command result:

- 6 tests executed: 5 passed, 1 failed.
- The corrupted consumable was correctly rejected with source/domain and a more precise path than
  expected: actual `items[0].legacy_power`, expected `items[0]`.
- Alias tests and all other source-set tests passed.

Correction:

- Fix the assertion to require the exact field-qualified path. Do not weaken the diagnostic to the
  containing object path.

Next step:

Rerun 6/6, then rerun the real live bundle. The 5/6 run is retained as failed evidence.

Final revalidation:

- Focused live-RON group: 6 executed, 6 passed.
- Live RON integration: 18 executed, 18 passed.
- Legacy `DamageType` spellings reject; canonical `Physical` loads.
- Corrupted live Consumable input rejects with
  `data/consumables/base.ron [consumables] at items[0].legacy_power`.

Decision and next step:

Continue strict authored-type coverage with table-driven root/nested unknown-field negative tests.
Then close duplicate pool, RunStart, Event reward/unsupported effect, map payload, and extra-wave
semantic gaps through one typed validator boundary.

## 2026-07-13 — Unsupported Event BossOmen variant removal

Hypothesis:

Because neither live Event data nor approved gameplay policy uses `ApplyBossOmenStepResult`, removing
it from both authored Event enums is safer than retaining a schema variant that panics during play.

Changes:

- Removed `ApplyBossOmenStepResult` from immediate and staged Event effect enums.
- Removed both runtime panic match arms.
- Replaced the runtime panic-preservation test with parse-negative tests for both removed variants.

Commands and actual results:

- `cargo check -q -p game_core`: passed.
- Focused live-RON group: 7 executed, 7 passed.
- First attempted event atomicity filter matched 0 tests and is not success evidence.
- Corrected filter
  `event_choice_effects_are_not_committed_when_combat_start_fails`: 1 executed, 1 passed.
- Source/live search finds no remaining `ApplyBossOmenStepResult` occurrence outside historical goal/
  audit records.

Decision and next step:

CPR-H-08's runtime-panic variant is structurally removed. The remaining CPR-H-08 work is shared
RewardEffect reference/value validation for all Event positions before publication.

## 2026-07-13 — Supported-mode RunStart contract validation

Hypothesis:

The exact generated bundle can reject missing or policy-mismatched pre-run VN triggers at the final
catalog boundary instead of leaving supported-mode availability to a later runtime lookup.

Changes:

- Added typed final validation requiring exactly one `RunStart` trigger for each supported
  `GameMode::{Standard, Endless}`.
- Required both triggers to be blocking, `OncePerRun`, and `PersistAcrossCheckpointLoad`.
- Added negative fixtures for a missing supported mode and a mismatched trigger policy with
  source/object/path diagnostics.

Command and actual result:

- `cargo test -q -p game_core game::data::live_ron::tests --lib -- --test-threads=1`:
  8 executed, 8 passed.
- Existing live-bundle positive loading remained green in the same focused authority.

Decision and next step:

CPR-M-01's G12-owned supported-mode cardinality/policy portion is closed. Continue with duplicate
pool and cross-bundle Event/map/extra-wave reference validation.

## 2026-07-13 — First-match pool ID rejection

Hypothesis:

Validating canonical pool IDs before publication removes the Shop/Reward first-match ambiguity
without changing their runtime selection APIs.

Changes:

- Added one typed pool-ID validator at the final parsed-bundle boundary.
- Applied it to Shop general pools, Reward general pools, and equipment reward pools.
- Added negative fixtures that duplicate a live Shop pool ID and a live Reward pool ID.

Command and actual result:

- `cargo test -q -p game_core game::data::live_ron::tests --lib -- --test-threads=1`:
  9 executed, 9 passed.
- Both duplicates reject before lookup and report source/domain/object/path plus
  `duplicate pool id` cause.

Decision and next step:

The confirmed pool first-match gaps are closed at publication. Reuse one fallible RewardEffect
validator for Reward records and every Event-embedded effect instead of copying domain logic.

## 2026-07-13 — Shared Reward and Event effect validation

Hypothesis:

The existing Reward reference/value rules can become one fallible pure function and validate every
authored RewardEffect position before publication without an Event-specific policy copy.

Changes:

- Extracted the existing RewardEffect match into one fallible validator and kept the legacy
  `GameDataBase` assertion path as a caller of that same authority during the staged conversion.
- Applied the validator at the final parsed-bundle boundary to Reward records, Event scene rewards,
  immediate `Grant`, and staged `Grant` effects.
- Added one negative contract test injecting an unknown Consumable into all four positions and
  asserting source/object/exact nested path diagnostics.

Commands and actual result:

- `cargo fmt --all`: passed.
- `cargo check -q -p game_core`: passed.
- `cargo test -q -p game_core game::data::live_ron::tests --lib -- --test-threads=1`:
  10 executed, 10 passed, 628 filtered out.

Observed result:

All supported RewardEffect positions reject the same invalid reference before publication. Event
paths distinguish `reward_effects`, `immediate_effects[..].Grant.effects`, and
`staged_effects[..].Grant.effects`; no copied Event-only effect policy was introduced.

Decision and next step:

CPR-H-08's shared reward/effect validation portion is closed. Continue CPR-M-02 with typed map
payload and complete extra-wave/stage/mission reference validation.

## 2026-07-13 — First map payload negative test compile failure

Hypothesis:

Every explicitly authored map Event/encounter/Shop/pool/Reward reference can be resolved at the final
bundle boundary while preserving `None` as the existing random-selection meaning.

Minimal change:

- Added typed cross-bundle validation for all explicit map payload IDs.
- Added negative Event and Shop-pool mutations to the live source-set test.

Command and actual result:

- `cargo fmt --all && cargo check -q -p game_core && cargo test -q -p game_core
  game::data::live_ron::tests --lib -- --test-threads=1` failed during test compilation; 0 tests
  executed.
- Cause: the test attempted to mutate `GameplayMapCatalogs.map_node_definitions` through its runtime
  `Arc`, which correctly has no `DerefMut`. Two accompanying `unused_mut` warnings were consequences
  of the failed borrow.

Correction and next step:

Use `Arc::make_mut` to clone-on-write only the parsed test catalog, preserving production immutable
sharing. Rerun with a nonzero focused count; this compile failure is not success evidence.

Revalidation:

- Replaced the invalid mutable `Arc` access with test-only clone-on-write via `Arc::make_mut`.
- `cargo fmt --all`: passed.
- `cargo check -q -p game_core`: passed.
- `cargo test -q -p game_core game::data::live_ron::tests --lib -- --test-threads=1`:
  11 executed, 11 passed, 628 filtered out.
- Missing explicit Event and Shop-pool IDs reject with map source, node kind ID, payload path, and
  missing target cause.

Decision and next step:

The map payload portion of CPR-M-02 is closed without changing the meaning of optional random
selection. Extract and apply one wave reference validator to encounter waves and every run-policy
extra wave.

## 2026-07-13 — Shared encounter and stage extra-wave validation

Hypothesis:

Run-policy `extra_waves` can share the exact authored-wave and target-reference rules used by PVE
encounters, eliminating the floor-0-only acceptance gap without a stage-specific schema.

Changes:

- Extracted the existing encounter-wave shape/value rules into one fallible helper still used by
  encounter database validation.
- Extracted abnormality/corroded-preset/profile and unsupported FacilityEntity checks into one
  fallible reference helper still used by encounter validation.
- Applied both helpers to every scaling stage extra wave, required IDs unique within a stage and
  non-colliding with waves they are combined with, and required defense-compatible routes because
  stage extras apply to the Defense encounter set too.
- Added negative stage-1-index fixtures for a missing abnormality and zero enemy count.

Commands and actual result:

- `cargo fmt --all`: passed.
- `cargo check -q -p game_core`: passed.
- `cargo test -q -p game_core game::data::pve_data::tests --lib -- --test-threads=1`:
  11 executed, 11 passed, 629 filtered out.
- `cargo test -q -p game_core game::data::live_ron::tests --lib -- --test-threads=1`:
  12 executed, 12 passed, 628 filtered out.

Observed result:

Every extra wave is validated even though current live stages author empty lists. Diagnostics name
`data/run/policy.ron`, the wave ID, and the exact scaling-stage extra-wave path.

Decision and next step:

Extend generated combat-preview startup validation from floor 0 to a representative floor for every
scaling stage so actual template route/spawn-zone and mission composition are also proven.

## 2026-07-13 — Every-stage generated combat-preview validation

Hypothesis:

Generating the real combat preview for every encounter, deterministic template seed, and scaling
stage representative floor will expose route/spawn-zone/mission composition failures that direct ID
validation cannot prove.

Changes:

- Replaced the floor-0-only startup preview validation with
  `try_generate_for_node_at_floor` at each stage's `min_floor - 1`.
- Preserved the existing five deterministic seeds and briefing-warning checks for every combination.
- Enriched failure context with encounter ID, authored stage range, and seed.

Commands and actual result:

- `cargo fmt --all`: passed.
- `cargo check -q -p game_core`: passed.
- `cargo test -q -p game_core --test ron_loading -- --test-threads=1`:
  18 executed, 18 passed, 0 filtered out.

Observed result:

The production live loader successfully validates every current encounter across all four authored
scaling stages and five seeds per stage through actual battlefield construction.

Decision and next step:

CPR-M-02's extra-wave/stage/mission generation gap is closed behaviorally. Audit and finish the
typed publication error boundary, remaining strict defaults/unknown variants, and broad evidence.

## 2026-07-13 — Production loader structured startup failure

Hypothesis:

The official loader can return strict parse/catalog errors directly and let game_server abort startup
without retaining a panic-only or fallback production API.

Changes:

- Changed `GameDataBase::load_live_embedded()` to return
  `Result<Arc<GameDataBase>, LiveRonError>`.
- Propagated parse and final catalog validation with `?`.
- Changed game_server startup to map the structured error to `std::io::ErrorKind::InvalidData`
  before the server binds.
- Updated test-only live-data helpers to use explicit expectations; no production fallback or dual
  schema was added.

Commands and actual result:

- `cargo fmt --all`: passed.
- `cargo check -q -p game_core`: passed.
- `cargo check -q -p game_server`: passed.
- `cargo test -q -p game_core --test ron_loading -- --test-threads=1`:
  18 executed, 18 passed, 0 filtered out.

Current limitation and next step:

Parse/new final validators now propagate typed diagnostics and server startup aborts cleanly, but
legacy domain/index and generated-preview validators inside `GameDataBase::new()` still assert or
panic. Convert those authorities rather than catching panics before claiming the publication error
boundary complete.

## 2026-07-13 — Top-level catalog and cross-item key validation

Hypothesis:

Validating every top-level authored ID/UUID directly on the parsed bundle can guarantee index
construction has no duplicate winner or duplicate-item panic without building a parallel lookup.

Changes:

- Added source-aware uniqueness/empty-ID checks for every top-level domain catalog and its indexed
  UUIDs, including range presets and equipment materials.
- Added one cross-registry UUID check across Equipment, Artifact, and Consumable items before
  `ItemRegistry` construction.
- Added negative mutations for duplicate Consumable ID and Equipment→Consumable UUID collision.

Commands and actual result:

- `cargo fmt --all`: passed.
- `cargo check -q -p game_core`: passed.
- `cargo test -q -p game_core game::data::live_ron::tests --lib -- --test-threads=1`:
  13 executed, 13 passed, 628 filtered out.

Observed result:

Both duplicate classes reject at the final parsed-bundle boundary with the concrete source file,
object ID, exact key path, and prior-domain cause. No first/last winner or panic capture is used.

Decision and next step:

Top-level index ambiguity is closed. Continue converting nested/domain semantic assertions and
classify/remove generated-field defaults that still permit omitted authoring output.

## 2026-07-13 — First generated-field default removal

Hypothesis:

Consumable `rarity` and `live_pool` are already explicitly generated in live data, so their serde
defaults only conceal missing exporter output and can be removed without gameplay change.

Changes:

- Removed the ZAYIN fallback for missing `rarity` and the false fallback for missing `live_pool`.
- Removed the now-unused default function.
- Added a source-set mutation deleting the first Consumable `live_pool` and asserted source/domain/
  object path plus missing-field cause.

Commands and actual result:

- `cargo fmt --all`: passed.
- `cargo check -q -p game_core`: passed.
- Consumable focused tests: 4 executed, 4 passed, 638 filtered out.
- Live-RON focused tests: 14 executed, 14 passed, 628 filtered out.

Decision and next step:

These two defaults were permissive legacy authoring behavior, not optional runtime meaning. Continue
the inventory in small domain batches; shared runtime/Unity DTO defaults require a raw authored
schema decision rather than mechanical removal.

## 2026-07-13 — First Core-only strict-default batch fixture failures

Hypothesis:

Shop stock/pools, Reward pools/filter/entries, run-policy extra waves, and BossOmen optional
references are Core-only generated fields already explicit in live RON and can become required
without gameplay change.

Changes before the first run:

- Removed serde defaults from those authored fields; no live data edit was required.

Command and actual result:

- The chained `cargo fmt`, Core check, and focused domain/live tests reached the Shop group.
- Shop focused: 6 executed, 4 passed, 2 failed. Later focused groups did not execute.
- Failure 1: a positive rerollable Shop fixture omitted database `pools` and now rejected with
  `missing field pools`.
- Failure 2: the old negative fixture expected a semantic reroll-reserve error after omitting
  `stock_items`; strict parsing now correctly rejects earlier with `missing field stock_items`.

Cause and correction:

The fixtures encoded the removed lenient schema, not current live behavior. Add explicit `pools: []`
to the positive fixture and replace/update the negative assertion so missing generator output is the
contract under test. Rerun all groups with nonzero counts; 4/6 is not success evidence.

Revalidation after fixture correction:

- Positive Shop RON now explicitly authors `pools: []`; the missing-stock negative test now asserts
  strict parse rejection rather than the later reserve-stock semantic error.
- Shop: 6/6 passed; Reward: 3/3 passed; RunPolicy: 5/5 passed; live-RON: 14/14 passed.
- The BossOmen filtered command executed 0 tests and is not success evidence.

Next step:

Add direct strict-schema negative tests for each removed default, including BossOmen, then rerun a
nonzero group. Do not treat the zero-test BossOmen command as proof.

First direct negative run:

- Live-RON focused: 15 executed, 14 passed, 1 failed.
- Shop `pools`, Reward `equipment_pools`, and stage `extra_waves` rejected when missing.
- BossOmen `encounter_id: Option<String>` unexpectedly deserialized as `None` when omitted even after
  removing `#[serde(default)]`.

Cause and next judgment:

Serde treats missing `Option<T>` fields as `None` intrinsically. Annotation removal alone cannot make
generated optional-valued fields presence-required. Add one shared `deserialize_required_option`
helper and apply it to Core-only authored Option fields; do not add sentinel values or wrapper schema.
Rerun 15/15 before recording the batch as success.

Correction and final revalidation:

- Added one generic `deserialize_required_option` helper at the data boundary.
- Applied it to both BossOmen authored Option fields and removed `skip_serializing_if`, so typed
  writer output also preserves explicit `None`.
- `cargo fmt --all`: passed.
- `cargo check -q -p game_core`: passed.
- Live-RON focused: 15 executed, 15 passed, 628 filtered out.

Decision and next step:

The first Core-only strict-default batch is complete. Extend required-field handling to the remaining
Core-only authored schemas, while keeping shared runtime/Unity DTO deserialization unchanged unless a
separate strict raw authored type is introduced.

## 2026-07-13 — First fallible domain-validator test migration failure

Hypothesis:

Consumable and Buff semantic assertions can return a shared `{object_id, field_path, cause}` error,
with legacy fixture builders only unwrapping that same validator.

Changes before the first run:

- Added `DataValidationError` and fallible Consumable/Buff validators.
- Connected them to the parsed live-bundle validation boundary.
- Added typed negative bundle mutations.

Command and actual result:

- The chained focused run reached Consumable tests: 4 executed, 1 passed, 3 failed. Buff and live-RON
  groups did not execute.
- All three invalid fixtures were still rejected, but their tests expected historical panic text
  (`above 100%`, old tier-cap sentence, `uses CombatNodes(0)`). The new thin unwrap emitted the typed
  error debug representation instead.

Cause and correction:

The tests fixed an internal panic-string shape, not user-visible data acceptance. Replace them with
direct `validate_contract()` error assertions for object/path/cause, then rerun every nonzero group.
Do not preserve old panic wording as a compatibility contract.

First correction run:

- Consumable: 4/4 passed after direct typed-error assertions.
- Buff: 3 executed, 2 passed, 1 failed; live-RON did not execute.
- The remaining Buff failure likewise expected the historical full panic sentence for stackable hard
  crowd control, while the same invalid object now returned `buffs[0].max_stacks` plus a typed cause.

Next correction:

Migrate the Buff negative test to `validate_contract()` field assertions and rerun Buff plus live-RON.

Final revalidation:

- Buff panic-string test was replaced by direct object/path/cause assertions.
- `cargo fmt --all`: passed.
- `cargo check -q -p game_core`: passed.
- Consumable: 4/4 passed in the prior correction run.
- Buff: 3/3 passed.
- Live-RON: 16/16 passed, 628 filtered out.

Observed result and next step:

Invalid Consumable and Buff semantics now reach the official bundle error with source file, domain,
object ID, exact authored field path, and cause. Continue the same single-validator pattern across
the remaining domain/index and cross-reference assertions; do not add panic capture.

## 2026-07-13 — Employee candidate domain validation conversion

Hypothesis:

Starter/recruitment candidate cardinality and required text fields can use the shared typed domain
error without changing fixture-builder ergonomics or candidate policy.

Changes:

- Converted candidate field validation and starter count validation to fallible contracts.
- Kept starter empty database valid for minimal fixtures, matching existing semantics.
- Connected starter and recruitment contracts to their distinct production source files.
- Added live-bundle negative mutations for starter role and recruitment background.

Commands and actual result:

- `cargo fmt --all`: passed.
- `cargo check -q -p game_core`: passed.
- Employee focused: 2/2 passed, 643 filtered out.
- Live-RON focused: 17/17 passed, 628 filtered out.

Decision and next step:

Employee candidate semantic panics are no longer reachable from a production bundle. Continue with
CorrodedWave and other bounded/nested domain validators using the same authority.

## 2026-07-13 — Corroded-wave domain validation conversion

Hypothesis:

Preset and nested role bounds can be converted without changing generated-wave selection semantics.

Changes:

- Converted preset ID/budget/count range/role-list and nested profile/weight/cost/min-max checks to
  `DataValidationError`.
- Connected the validator to `data/enemies/corroded_wave_presets.ron` publication.
- Added a zero role-weight mutation asserting preset ID and exact nested path.

Commands and actual result:

- `cargo fmt --all`: passed.
- `cargo check -q -p game_core`: passed.
- CorrodedWave focused: 2/2 passed, 644 filtered out.
- Live-RON focused: 18/18 passed, 628 filtered out.

Decision and next step:

CorrodedWave semantic assertions are no longer reachable from production publication. Continue the
remaining domain validators and then convert cross-domain constructor assertions.

## 2026-07-13 — Shop and Reward domain validation conversion

Hypothesis:

Same-domain pool membership and nonempty Reward semantics can validate against parsed collections
without constructing first-match indexes.

Changes:

- Converted Shop pool ID/member references and Reward effects/pool ID/member references to fallible
  domain contracts.
- Used direct collection membership so validation itself does not invoke an index that could panic.
- Added missing Shop and Reward pool-member mutations with exact member paths.

Commands and actual result:

- `cargo fmt --all`: passed.
- `cargo check -q -p game_core`: passed.
- Shop: 6/6 passed; Reward: 3/3 passed; live-RON: 19/19 passed.

Decision and next step:

Same-domain Shop/Reward lookup failures are closed at publication. Continue remaining local domain
contracts before the larger cross-domain validator conversion.

Auxiliary search note:

The read-only Warp Grep follow-up hit a 429 retry limit and produced no evidence; no code or document
judgment relies on it.

## 2026-07-13 — BossOmen local domain validation conversion

Hypothesis:

BossOmen chain/step structure and source-kind combinations can become fallible without deciding new
omen gameplay or target-reference policy.

Changes:

- Converted chain IDs/bounds/required step count, nested step IDs/presentation IDs, and Event/Combat
  field combinations to one DataValidationError authority.
- Kept actual Event/PVE/abnormality target existence in the later cross-domain conversion.
- Added an Event step with forbidden encounter_id mutation and exact nested path assertion.

Commands and actual result:

- cargo fmt --all: passed.
- cargo check -q -p game_core: passed.
- Live-RON focused: 20/20 passed, 628 filtered out.

Decision and next step:

BossOmen local semantic panics are no longer reachable from production publication. Continue local
Event/PVE/equipment/skill validators, then convert the shared cross-domain reference authority.

## 2026-07-13 — Event graph domain validation conversion

Hypothesis:

The current Event graph's structural safety can become fallible without implementing or constraining
G03's future illustrated employee-check/d20 policy.

Changes:

- Converted Event/scene/choice identity, presentation fields, next references, cycle/reachability,
  terminal-path, and StartCombat cardinality validation to DataValidationError.
- Corrected the old choice-next control flow so StartCombat cardinality is validated even when the
  choice ends rather than links to another scene.
- Added an empty background mutation with exact Event scene path.

Commands and actual result:

- cargo fmt --all: passed.
- cargo check -q -p game_core: passed.
- Event focused: 1/1 passed, 648 filtered out.
- Live-RON focused: 21/21 passed, 628 filtered out.

Decision and next step:

Current Event structural panics are no longer reachable from production publication. Continue PVE and
other domain validators; preserve G03 ownership of the replacement Event gameplay/DTO schema.

## 2026-07-13 — PVE local authoring validation conversion

Hypothesis:

Existing PVE authoring invariants can return the shared typed domain error without changing encounter,
mission, reward, suppression, or combat-preview policy.

Changes:

- Converted wave presence/identity/shape, encounter-class primary-target rules, suppression bonus
  objective fields, tactical-point identity, and tactical objective/win-condition consistency from
  assertions to one `DataValidationError` contract.
- Kept the existing shared fallible wave-shape helper and attached it to the exact encounter/wave
  path instead of copying its rules.
- Connected the PVE contract to `data/pve/encounters.ron` at the parsed-bundle publication boundary.
- Replaced four panic-string tests with direct object/path/cause assertions and added a live-bundle
  empty-wave mutation.

Commands and actual result:

- `cargo fmt --all`: passed.
- `cargo check -q -p game_core`: passed.
- PVE focused: 11/11 passed, 639 filtered out.
- Live-RON focused: 22/22 passed, 628 filtered out.

Decision and next step:

PVE local authoring assertions are no longer reachable from production publication. The distinct
cross-domain PVE references and mission/reward compatibility assertions in `validation.rs` remain for
the shared cross-domain fallible conversion; do not duplicate them in the local domain contract.

## 2026-07-13 — Visual Novel trigger local validation conversion

Hypothesis:

Trigger-local identity and source payload rules can become fallible without changing the completed
pre-run/VN trigger policy or its separate cross-domain reference rules.

Changes:

- Converted trigger ID/VN ID, Event scene IDs, map kind ID, battle encounter ID, and positive battle
  time validation to `DataValidationError`.
- Kept RunStart cardinality/policy and VN/Event/map/PVE target existence in their existing final
  bundle/cross-domain ownership.
- Connected the local trigger validator to `data/visual_novel_triggers/base.ron` and added an empty
  `vn_id` live-bundle mutation.
- Replaced the local panic-capture test with direct object/path/cause assertions.

Commands and actual result:

- `cargo fmt --all`: passed.
- `cargo check -q -p game_core`: passed.
- Visual Novel trigger focused: 2/2 passed, 649 filtered out.
- Live-RON focused: 23/23 passed, 628 filtered out.

Decision and next step:

Local trigger assertions are no longer reachable from production publication. Cross-domain trigger
references remain part of the later shared `validation.rs` conversion, while full VN graph structure
is the next local domain candidate.

## 2026-07-13 — Visual Novel graph validation conversion

Hypothesis:

The completed VN schema's existing local graph and presentation invariants can become fallible without
changing any scene progression, blocking, trigger, or Unity-facing policy.

Changes:

- Converted VN/scene/speaker/choice identity and presentation constraints, target existence,
  cycle/reachability detection, and terminal-path validation to `DataValidationError`.
- Removed the DFS internal `expect` by returning a typed missing-scene error even though target
  existence is checked before traversal.
- Connected the VN contract to `data/visual_novels/base.ron`, added an empty-script live-bundle
  mutation, and replaced the dialogue panic-capture test with direct object/path/cause assertions.

Commands and actual result:

- `cargo fmt --all`: passed.
- `cargo check -q -p game_core`: passed.
- Visual Novel focused: 2/2 passed, 650 filtered out.
- Live-RON focused: 24/24 passed, 628 filtered out.

Decision and next step:

Visual Novel graph assertions are no longer reachable from production publication. Continue the
remaining local gameplay databases, then convert their cross-domain reference consumers through one
fallible bundle validator.

## 2026-07-13 — Skill Fragment compatibility validation conversion

Hypothesis:

Static compatibility requirements can expose their exact invalid field without changing runtime
equipment/fragment compatibility evaluation.

Changes:

- Replaced free-form `GameError` construction for duplicate compatibility axes, zero block minimum,
  empty/duplicate tags, and required/incompatible tag conflicts with a small field-aware local error.
- Mapped that local error to `DataValidationError` at the fragment database boundary and kept fixture
  index validation as a thin unwrap of the same contract.
- Connected the contract to `data/skill_fragments/base.ron`, added a duplicate damage-type axis live
  mutation, and updated three focused static-contract tests to assert field/cause directly.

Commands and actual result:

- `cargo fmt --all`: passed.
- `cargo check -q -p game_core`: passed.
- Skill Fragment focused: 6/6 passed, 647 filtered out.
- Live-RON focused: 25/25 passed, 628 filtered out.

Decision and next step:

Skill Fragment local compatibility panics are no longer reachable from production publication. Skill
existence and fragment effect references remain cross-domain work for the shared validator conversion.

## 2026-07-13 — Equipment live-boundary patch context failure

Hypothesis:

The new equipment contract can be connected immediately after the Skill Fragment contract using the
previously read formatting context.

Minimal attempt and result:

- Applied the equipment domain validator conversion successfully.
- The combined follow-up patch for `live_ron.rs` failed before changing that file because the expected
  formatted Skill Fragment block no longer matched the exact current text.
- No compile or test command was run, so this attempt is not validation evidence.

Correction and next step:

Read the exact current `validate_fallible_domain_contracts` and focused-test insertion context, then
apply smaller patches. Do not broaden or rewrite the file to work around a context mismatch.

Follow-up compile result:

- The smaller live-boundary/test patches applied.
- `cargo fmt --all` passed, then `cargo check -q -p game_core` failed; 0 tests executed.
- Cause: `WeaponCombatProfile::validate_runtime_contract` returns `GameError`, which intentionally
  has no `Display` implementation, so the new adapter could not call `to_string()`.

Correction:

Extract the existing `GameError::InvalidStaticData` payload explicitly at this internal boundary and
retain a debug cause only for any future unexpected variant. Rerun check and both nonzero focused
groups; the failed compile is not success evidence.

Correction and final revalidation:

- Explicitly extracted `GameError::InvalidStaticData` from weapon-profile validation.
- Converted weapon-profile, combination-recipe, dismantle, and enhancement rules to exact-path
  `DataValidationError` values and connected them to `data/equipments/base.ron`.
- Changed equipment constructor indexes from eager to lazy construction so the shared validator can
  reject invalid recipes before `build_recipe_index` is reachable.
- Added a focused duplicate unordered recipe-key test plus a live weapon-without-profile mutation.
- `cargo fmt --all`: passed.
- `cargo check -q -p game_core`: passed.
- Equipment focused: 2/2 passed, 653 filtered out.
- Live-RON focused: 26/26 passed, 629 filtered out.

Decision and next step:

Equipment semantic/reference and recipe-index panics are no longer reachable from production
publication. Cross-domain equipment uses in rewards, starters, Shops, and skills remain owned by the
shared bundle validator.

## 2026-07-13 — Visual Novel trigger cross-reference conversion

Hypothesis:

The existing trigger-to-VN reference rule can become one fallible authority shared by live bundle
publication and later `GameDataBase` fixture construction.

Changes:

- Converted `validate_visual_novel_trigger_references` from assertion to `DataValidationError` with
  trigger index/ID/VN-field context.
- Connected the same function to parsed-bundle publication and retained the legacy constructor as a
  thin expect caller during the larger cross-domain staged conversion.
- Replaced the panic-capture reference test and added a missing live VN target mutation.

Commands and actual result:

- `cargo fmt --all`: passed.
- `cargo check -q -p game_core`: passed.
- Visual Novel trigger focused: 2/2 passed, 654 filtered out.
- Live-RON focused: 27/27 passed, 629 filtered out.

Decision and next step:

Missing trigger VN targets no longer reach production constructor assertions. Continue converting
cross-domain validators by source-owning catalog so every failure can retain an exact live file/path.

## 2026-07-13 — Event combat cross-reference conversion

Hypothesis:

Current Event `StartCombat` target existence/primary-target consistency can become fallible without
changing G03's deferred Event gameplay or effect execution semantics.

Changes:

- Converted Event combat lookup and optional primary-abnormality consistency assertions to one
  source-owning `DataValidationError` function.
- Connected the same function to parsed-bundle publication; the staged `GameDataBase` constructor
  remains a thin expect caller until all cross-domain validators return one result.
- Added a missing encounter mutation at an actual live Event choice/effect location.

Commands and actual result:

- `cargo fmt --all`: passed.
- `cargo check -q -p game_core`: passed.
- Event focused: 1/1 passed, 656 filtered out.
- Live-RON focused: 28/28 passed, 629 filtered out.

Decision and next step:

Invalid Event combat targets no longer reach production runtime assertions. Continue BossOmen, PVE,
skill, reward, Shop, and starter-loadout cross-reference conversion.

## 2026-07-13 — Shop item cross-reference patch context failure

Hypothesis and attempt:

After converting Shop item UUID lookup from `ItemRegistry` panic to a direct fallible catalog check,
connect it after the equipment contract using previously read `live_ron.rs` context.

Result:

- The validator and `GameDataBase` call-order changes applied.
- The live-boundary patch failed before changing `live_ron.rs` because formatting/context differed.
- No compile or test command ran; this is not success evidence.

Correction and next step:

Read the exact current tail of `validate_fallible_domain_contracts`, apply a smaller insertion, then
add the missing-item live mutation and run nonzero Shop/live groups.

Correction and final revalidation:

- Connected the direct Artifact/Consumable/Equipment UUID validator with the exact current context.
- Moved Shop reference validation before `ItemRegistry` derivation in `GameDataBase::new()` and kept
  the constructor as a thin expect caller during staged cross-domain conversion.
- Added a missing visible-item UUID live mutation with exact stock index path.
- `cargo fmt --all`: passed.
- `cargo check -q -p game_core`: passed.
- Shop focused: 6/6 passed, 652 filtered out.
- Live-RON focused: 29/29 passed, 629 filtered out.

Decision and next step:

Shop item reference panic is no longer reachable from production publication, and item registry is
now purely derived after canonical reference validation.

## 2026-07-13 — Starter employee loadout cross-reference conversion

Hypothesis:

Starter loadout cardinality, Skill Fragment/equipment existence, and equipment-slot compatibility can
be validated fallibly against parsed catalogs before candidate publication.

Changes:

- Converted empty baseline/equipment lists, missing Fragment/equipment targets, and `ItemSlot` layout
  failure to exact-path `DataValidationError` values.
- Connected the same function to live bundle validation and retained the staged `GameDataBase`
  constructor as a thin expect caller.
- Added a missing baseline Skill Fragment live mutation.

Commands and actual result:

- `cargo fmt --all`: passed.
- `cargo check -q -p game_core`: passed.
- Employee focused: 2/2 passed, 657 filtered out.
- Live-RON focused: 30/30 passed, 629 filtered out.

Decision and next step:

Starter loadout assertions no longer remain on the production publication path. Continue remaining
PVE/BossOmen/skill/reward cross-catalog rules and local Skill/Abnormality/CorrodedEmployee conversion.

## 2026-07-13 — BossOmen cross-reference first compile failure

Hypothesis and changes:

Split abnormality-owned omen-chain references from chain-owned boss/Event/PVE references and return
exact-path typed errors from both.

First command result:

- `cargo fmt --all` passed; `cargo check -q -p game_core` failed; 0 tests executed.
- Cause: the new diagnostic formatted `BossOmenChainId` with `Display`, which that typed ID does not
  implement.

Correction and next step:

Use its existing canonical `as_str()` accessor in the cause, then rerun check and nonzero BossOmen/
live groups. The compile failure is not success evidence.

Correction and final revalidation:

- Used `BossOmenChainId::as_str()` and completed the split source-owning validators.
- Added a missing Event target mutation at an actual live BossOmen Event step.
- `cargo fmt --all`: passed.
- `cargo check -q -p game_core`: passed.
- The first `game::data::boss_omen_data::tests` filter executed 0 tests and is not evidence.
- Corrected `boss_omen --lib` filter: 11/11 passed, 649 filtered out.
- Live-RON focused: 31/31 passed, 629 filtered out.

Decision and next step:

BossOmen cross-catalog assertions and local-validation `expect` assumptions are no longer reachable
from production publication. Continue the remaining PVE, reward, Skill/Fragment, and unit-skill
references.

## 2026-07-13 — PVE cross-contract validation ordering failure

Hypothesis and changes:

Convert PVE primary target/threat class, wave/reward references, survive timer, mission compatibility,
and reward-policy assertions to one exact-path typed validator.

First command result:

- `cargo fmt --all` and `cargo check -q -p game_core`: passed.
- PVE focused: 11/11 passed, 650 filtered out.
- Live-RON: 31/32 passed; 1 failed.
- Failure: the existing invalid RewardEffect test was reported as a PVE reward-policy error because
  the new PVE cross validator ran inside local-domain validation before the source-owning RewardEffect
  validator. Expected Reward source, actual PVE source.

Cause and correction:

Cross-domain consumers must not preempt a source catalog's own effect validity. Move PVE cross
validation after shared RewardEffect validation in the final bundle sequence, then rerun PVE and all
live-RON tests. Do not weaken the existing Reward diagnostic expectation.

Correction and final revalidation:

- Moved the PVE cross-contract call out of local-domain validation and after the shared RewardEffect
  validator in the final bundle sequence.
- The existing invalid RewardEffect test again reports `data/events/rewards/base.ron`; the new zero
  survive-timer mutation reports `data/pve/encounters.ron` and its exact field.
- `cargo fmt --all`: passed.
- `cargo check -q -p game_core`: passed.
- PVE focused: 11/11 passed, 650 filtered out.
- Live-RON focused: 32/32 passed, 629 filtered out.

Decision and next step:

PVE cross-catalog and mission/reward-policy assertions no longer remain on production publication.
Source-local validation must continue to precede downstream cross-domain consumer validation.

## 2026-07-13 — Unity MCP readiness discovery

Read-only result:

- Unity MCP reported one instance: `ark@b49d32f6f2f70bbe`, Unity 6000.3.4f1, project
  `F:/unity projects/ark`, with the project test runner available.
- The editor was idle/not compiling/not playing, but its canonical state was stale and explicitly
  reported `ready_for_tools: false` with `stale_status`.
- No Unity test was started and no Unity state was mutated. This is not test evidence.

Decision:

Re-read the Unity editor state at G12 broad verification and use the MCP test runner only when ready.
Current G12 changes are Core/live-RON-only and introduce no Unity-facing DTO or reader change.

## 2026-07-13 — Equipment Reward pool cross-reference fixture failure

Hypothesis and changes:

Move equipment reward-pool weight/reference/candidate assertions to one fallible validator executed
after RewardEffect validation and before PVE reward consumption.

First command result:

- `cargo fmt --all` and `cargo check -q -p game_core`: passed.
- Reward focused: 3/3 passed, 659 filtered out.
- Live-RON: 32/33 passed; 1 failed.
- Cause: the live first equipment pool exists but has zero explicit entries, so the negative test
  indexed `entries[0]` and panicked before invoking the validator.

Correction and next step:

Append a complete invalid `EquipmentRewardPoolEntry` fixture instead of assuming a live entry, then
rerun Reward and all live-RON tests. The 32/33 run is not success evidence.

Second run:

- Reward focused remained 3/3.
- Live-RON remained 32/33; the validator rejected the bundle, but the earlier RewardEffect consumer
  reported object `ego_gift_reward` instead of the defining pool `ego_gift_reward_pool` because the
  invalid explicit entry made the pool produce no candidates.

Correction:

Validate equipment-pool definitions before RewardEffects that consume those pools, preserving the
final dependency order `pool definition -> RewardEffect consumer -> PVE reward consumer`. Rerun the
same nonzero groups; do not change the exact pool diagnostic expectation.

Final revalidation:

- Moved the equipment-pool definition validator before RewardEffect validation.
- `cargo fmt --all`: passed.
- `cargo check -q -p game_core`: passed.
- Reward focused: 3/3 passed, 659 filtered out.
- Live-RON focused: 33/33 passed, 629 filtered out.

Decision and next step:

Equipment Reward pool assertions no longer remain on the production publication path. The final
definition/consumer ordering now gives malformed pool data one deterministic source-object-path
diagnostic before any dependent Reward or PVE policy evaluates it.

## 2026-07-13 — Abnormality negative-test patch context failure

Hypothesis and applied change:

Add a fallible `BasicAttackDef` contract, use it for Abnormality local validation, and connect that
source-local contract before BossOmen/PVE consumers.

Intermediate result:

- Runtime and live-boundary patches applied.
- The negative-test insertion patch failed before changing tests because the expected combined
  Consumable/Buff test function name did not match current source.
- No compile or test command has run; this is not validation evidence.

Correction and next step:

Read the exact current test function context, insert the Abnormality mutation separately, then run
nonzero Abnormality and live-RON groups.

Correction and final revalidation:

- Added `BasicAttackDef::try_validate_runtime_contract()` and retained the old panic wrapper only as
  a staged non-production caller for remaining CorrodedEmployee/test paths.
- Added `AbnormalityDatabase::validate_contract()` and connected it before BossOmen/PVE consumers.
- Added a zero basic-attack interval live mutation with exact Abnormality path.
- `cargo fmt --all`: passed.
- `cargo check -q -p game_core`: passed.
- Abnormality focused: 4/4 passed, 659 filtered out.
- Live-RON focused: 34/34 passed, 629 filtered out.

Decision and next step:

Abnormality local authoring assertions no longer remain on production publication. Reuse the same
fallible basic-attack primitive when converting CorrodedEmployee raw preset resolution and local
validation; do not keep its current panic wrapper as a second production authority.

## 2026-07-13 — CorrodedEmployee raw conversion and local validation

Hypothesis:

The custom raw-to-runtime preset resolution and the post-resolution profile contract can become one
fully fallible path without changing CorrodedEmployee range or combat policy.

Changes:

- Replaced eager constructor indexes with lazy indexes.
- Replaced raw preset duplicate/range, unknown preset, and inline-plus-preset panics with Serde custom
  errors returned from `try_into_database`/`try_into_profile`.
- Added a typed local validator for preset/profile/basic-attack invariants and connected it to the
  live bundle before CorrodedWave/PVE consumers.
- Added a zero-health live mutation with exact profile path.

First validation result:

- `cargo fmt --all` and `cargo check -q -p game_core`: passed functionally but emitted one dead-code
  warning for the now-unused staged `BasicAttackDef::validate_runtime_contract` panic wrapper.
- CorrodedEmployee focused: 6/6 passed, 658 filtered out.
- Live-RON focused: 35/35 passed, 629 filtered out.

Correction and next step:

Remove the obsolete panic wrapper and migrate its final unit-test caller to the fallible primitive,
then rerun warning-free. The green counts above are intermediate evidence, not final closure.

Wrapper-removal run:

- CorrodedEmployee remained 6/6.
- Abnormality ran 4 tests: 3 passed, 1 failed; live-RON did not execute.
- The remaining test retained a historical `#[should_panic(expected = "basic_attack ...")]`
  assertion while the fallible primitive correctly returned the TileArea rejection as `Err`.

Correction:

Replace the panic-string test with a direct error-cause assertion and rerun all three nonzero groups.
Do not restore the obsolete wrapper merely to satisfy an internal panic wording test.

Final revalidation:

- Removed the obsolete BasicAttack panic wrapper.
- Replaced its final panic-string test with direct fallible-cause validation.
- `cargo fmt --all`: passed.
- `cargo check -q -p game_core`: passed without warnings.
- CorrodedEmployee focused: 6/6 passed, 658 filtered out.
- Abnormality focused: 4/4 passed, 660 filtered out.
- Live-RON focused: 35/35 passed, 629 filtered out.

Decision and next step:

CorrodedEmployee raw parsing, preset resolution, local semantics, and index construction no longer
expose a production panic boundary. Continue with the structurally similar but larger Skill raw
range-resolution/runtime contract as one coherent conversion.

## 2026-07-13 — Skill raw/local validation first focused failure

Hypothesis and changes:

Convert Skill range-preset raw resolution, resolved skill/step/projectile/TileArea semantics, and Buff
references to fallible validators before connecting later Skill consumers.

First command result:

- `cargo fmt --all` and `cargo check -q -p game_core`: passed.
- Skill focused: 10/11 passed; 1 failed. Live-RON did not execute.
- The unknown-Buff test ignored the new `Result`, emitted an `unused_must_use` warning, and still
  expected `catch_unwind` to observe the historical panic.

Correction and next step:

Replace that panic-capture test with direct `DataValidationError` object/path/cause assertions, then
rerun Skill and live-RON groups warning-free. Do not restore panic behavior.

Correction and final revalidation:

- Migrated the unknown-Buff test to direct typed error assertions.
- Skill raw preset resolution now returns Serde errors for duplicate/invalid/unknown/dual range
  sources; resolved local validation returns exact-path `DataValidationError`.
- Added fallible projectile collision/homing and persistent TileArea primitives while retaining their
  runtime assertion wrappers for already validated runtime objects.
- Connected Skill local and Buff-reference validation after Buff source-local validation.
- Added an empty-step live mutation.
- `cargo fmt --all`: passed.
- `cargo check -q -p game_core`: passed without warnings.
- Skill focused: 11/11 passed, 654 filtered out.
- Live-RON focused: 36/36 passed, 629 filtered out.

Decision and next step:

Skill raw parsing, local semantics, Buff references, and index construction no longer expose a
production publication panic. The remaining Skill Fragment and unit-skill cross references may now
safely consume this validated Skill catalog.

## 2026-07-13 — Skill Fragment and unit Skill cross-reference conversion

Hypothesis:

After Skill/Abnormality/CorrodedEmployee local validation, Fragment origin/source/dependency/active-
Skill and unit Skill references can share one fallible DefenseRoute contract without masking source
errors.

Changes:

- Converted Skill Fragment Abnormality and imitation/upgrade/awakened Skill references to exact-path
  `DataValidationError`.
- Split unit consumers by authored source: Abnormality threat/response-Fragment/Skill fields and
  CorrodedEmployee Skill fields return errors mapped to their own live files.
- Converted the shared DefenseRoute range check to a pure fallible helper.
- Added a missing imitation Skill mutation at an actual live ActiveSkill Fragment.

Commands and actual result:

- `cargo fmt --all`: passed.
- `cargo check -q -p game_core`: passed.
- Skill Fragment focused: 6/6 passed, 660 filtered out.
- Live-RON focused: 37/37 passed, 629 filtered out.

Decision and next step:

Skill Fragment and unit Skill cross-reference assertions no longer remain on production publication.
The remaining constructor-level cross-reference duplication can now be collapsed into one fallible
bundle call after the last Reward/preview boundaries are converted.

## 2026-07-13 — Generated combat-preview typed loader boundary

Hypothesis:

The existing every-encounter/every-stage/five-seed real preview proof can preserve its behavioral
coverage while returning source-aware loader errors instead of panicking after database construction.

Changes:

- Converted preview generation and required briefing-warning assertions to
  `Result<(), DataValidationError>` with encounter ID/path and stage/seed cause.
- Changed the official loader to map that result to `LiveRonError` before returning the database.
- Removed the redundant public post-load assertion API; the integration test now proves the same
  behavior by loading through the official production path.

Commands and actual result:

- `cargo fmt --all`: passed.
- `cargo check -q -p game_core`: passed.
- Live RON integration: 18/18 passed, 0 filtered out.
- Live-RON focused negative/authority group: 37/37 passed, 629 filtered out.

Decision and next step:

Generated preview validation no longer leaves a panic-only tail after typed bundle validation. Audit
the remaining `GameDataBase::new()` assertion wrappers and collapse production construction so the
official loader never re-enters fixture-oriented panic validation.

## 2026-07-13 — Validated production publication cutover

Hypothesis:

Once the parsed bundle has passed every migrated local/cross/generated validator, production should
derive indexes and publish the immutable database without re-running fixture-oriented panic wrappers.

Changes:

- Added a private `GameDataBase::from_validated_parts` constructor that only derives the item index
  and assembles the already validated catalogs.
- Changed `load_live_embedded()` to construct its exact parsed parts directly through that private
  seam rather than round-tripping through `GameDataBuilder::build_arc()`/`GameDataBase::new()`.
- Kept public fixture builder behavior unchanged; tests still fail fast on invalid hand-built parts.

Commands and actual result:

- `cargo fmt --all`: passed.
- `cargo check -q -p game_core`: passed.
- `cargo check -q -p game_server`: passed.
- Live RON integration: 18/18 passed, 0 filtered out.

Decision and next step:

The official production loader no longer has a duplicate panic-validation publication boundary.
Continue strict-default inventory and legacy/panic searches; fixture-only unwraps are not production
acceptance authority and may remain only where they call the same validators.

## 2026-07-13 — Required generated Skill field fixture failure

Hypothesis and changes:

Live Skill authoring explicitly emits every Skill `name/kind/focus_time_ms/steps` and every step
`id/delay_ms/delivery/effects`, so their raw-schema defaults conceal missing generator output.

First command result:

- `cargo fmt --all` and `cargo check -q -p game_core`: passed.
- Skill focused: 10/11 passed; 1 failed. Live-RON did not execute.
- The range-preset positive fixture omitted step `delay_ms` and therefore encoded the removed lenient
  schema; production live data already authors the field on all 51 steps.

Correction and next step:

Make that fixture explicit and rerun Skill plus live-RON, including the new missing-Skill-name parse
negative. Do not restore defaults for fixture convenience.

Correction and final revalidation:

- Updated the one lenient range-preset fixture to explicitly author the required Skill/step fields.
- Added a source-set negative proving missing generated Skill `name` rejects during parse with Skill
  source/domain/object path.
- `cargo fmt --all`: passed.
- `cargo check -q -p game_core`: passed.
- Skill focused: 11/11 passed, 656 filtered out.
- Live-RON focused: 38/38 passed, 629 filtered out.

Decision and next step:

The required generated Skill field batch is complete. Retain defaults for fields whose omission has
intentional authored meaning (`Pattern`, no inline range, no repeat/condition/presentation, etc.) and
continue the same evidence-based classification for other Core-only raw schemas.

## 2026-07-13 — Required Abnormality/CorrodedEmployee combat-field fixture failure

Hypothesis and changes:

All live Abnormality and CorrodedEmployee records explicitly author magic resistance, threat/profile
class, movement, and resonance, while empty traits and optional references retain intentional defaults.
Also remove Abnormality custom-deserialize validation panic by building lazy resolved indexes.

First command result:

- `cargo fmt --all` and `cargo check -q -p game_core`: passed.
- Abnormality focused: 4/4 passed, 663 filtered out.
- CorrodedEmployee focused: 5/6 passed; 1 failed. Live-RON did not execute.
- The range-preset positive fixture omitted required `magic_resist` (and would next omit movement/
  resonance), while production live profiles explicitly author all three.

Correction and next step:

Make that fixture canonical and explicit, then rerun both domain groups and live-RON. Do not restore
raw defaults for fixture convenience.

Correction and final revalidation:

- Updated the one CorrodedEmployee range-preset fixture with explicit magic resistance, movement,
  and resonance.
- Abnormality custom deserialization now constructs lazy resolved data and cannot invoke semantic/
  duplicate index panics before the final bundle validator.
- Added a source-set negative proving missing generated Abnormality `magic_resist` rejects during
  parse.
- `cargo fmt --all`: passed.
- `cargo check -q -p game_core`: passed.
- Abnormality focused: 4/4 passed, 663 filtered out.
- CorrodedEmployee focused: 6/6 passed, 661 filtered out.
- Live-RON focused after the direct parse negative: 39/39 passed, 629 filtered out.

Decision and next step:

This required combat-field batch is complete. Optional trait/reference fields remain intentionally
optional; continue audit without mechanically removing defaults from runtime/Unity-shared types.

## 2026-07-13 — Required CorrodedWave balance fields

Hypothesis:

Every live CorrodedWave preset explicitly authors preferred node/risk lists and each role's weight,
cost, min/max count, and tier; defaults on those Core-only balance fields conceal generator output.

Changes:

- Removed defaults from all confirmed fields and required explicit `Some`/`None` for `max_count`.
- Added a source-set negative that removes the first role weight and asserts exact source/domain/path.

Commands and actual result:

- `cargo fmt --all`: passed.
- `cargo check -q -p game_core`: passed.
- CorrodedWave focused: 2/2 passed, 667 filtered out.
- Live-RON focused: 40/40 passed, 629 filtered out.

Decision and next step:

CorrodedWave balance defaults are removed without changing live values. Continue only with fields
proven generator-required; do not remove optional runtime meaning mechanically.

## 2026-07-13 — Required PVE enemy tier/count and wave time fields

### Hypothesis

The live generator already emits every manual enemy `tier`/`count` and every wave `time_ms`.
Allowing those authored combat values to disappear and silently become Tier I, count 1, or time 0
violates the exact-version generated schema, while optional reference/presentation fields can retain
their explicit absence semantics.

### Minimal experiment

- Removed Serde defaults from manual enemy `tier`/`count` and wave `time_ms`.
- Added a source-set negative test that removes one live manual enemy `tier`.

### First result — failed before tests ran

Command:

`cargo fmt --all && cargo test -q -p game_core pve_data::tests -- --test-threads=1 && cargo test -q -p game_core game::data::live_ron::tests -- --test-threads=1`

Actual tests executed: 0. Compilation failed because the new test used `Result::expect_err`, which
requires the successful `ParsedGameplayRon` type to implement `Debug`. This was a test-harness
mistake, not a runtime contract failure. Replace it with the existing explicit `match` pattern and
rerun the same focused checks.

### Second result — fixture migration required

After replacing `expect_err`, the PVE focused command executed 11 tests: 5 passed and 6 failed;
659 were filtered out. All six failures were authored-RON fixtures that omitted manual enemy
`tier`/`count` and therefore failed at the new parse boundary with `missing field 'tier'`. The live
PVE source already emits both fields. Migrate these fixtures to the canonical generated shape, then
rerun PVE and live-RON focused tests.

### Correction and final result

- Migrated the six authored-RON fixtures to explicit manual enemy `tier`/`count` and wave `time_ms`.
- `cargo fmt --all`: passed.
- PVE focused: 11/11 passed, 659 filtered out.
- Live-RON focused: 41/41 passed, 629 filtered out.

Decision: the three generated combat values are now exact-version required fields without changing
live gameplay values. Retain only separately classified optional/neutral PVE omission semantics.

## 2026-07-13 — Broad Core and Unity MCP verification

### First Core result — stale panic assertion

Command: `cargo test -q -p game_core --all-targets -- --test-threads=1`

Actual result: 670 library tests executed; 669 passed and 1 failed. The remaining targets did not
run because the library target failed. `game_data_validation_rejects_attack_interval_zero` still
used `#[should_panic(expected = "basic_attack interval_ms must be greater than zero")]`, while the
new shared Abnormality validator correctly rejected with structured object/path/cause inside the
fixture constructor panic wrapper. Replace the obsolete panic-string test with direct typed contract
assertions and rerun the broad command.

Correction: removed this redundant legacy panic-string test rather than changing its expected text.
The current live-RON negative test already mutates the same Abnormality interval and asserts the
typed source/object/path failure through the production bundle validator.

### Final Core/server/static result

- `cargo fmt --all && cargo test -q -p game_core --all-targets -- --test-threads=1`: passed.
  Actual tests: 669 library + 3 + 3 + 18 + 13 + 4 integration = 710 passed; 0 failed; 0 ignored.
- `cargo test -q -p game_server --all-targets -- --test-threads=1`: 27/27 passed; 0 failed;
  0 ignored.
- `cargo fmt --all -- --check`: passed.
- `cargo check -q -p game_core --all-targets`: passed.
- `cargo check -q -p game_server --all-targets`: passed.
- Residual searches found no Serde alias and no ignored test in the production data/map/preview,
  Core integration, or server source scopes. The only `legacy_fallback` hit is the negative
  run-policy unknown-field test. Production gameplay `include_str!` calls are centralized in
  `GameplayRonSourceSet`; the remaining direct CorrodedWave/Buff/RunPolicy readers are test/fixture
  helpers, not server startup paths.

### Unity MCP start

- Instance: `ark@b49d32f6f2f70bbe`, Unity 6000.3.4f1, project `F:/unity projects/ark`.
- Pre-run editor state: ready, idle, not compiling, not playing, no test job running.
- Started the full EditMode suite through Unity MCP as job
  `b9128241e23c48a6b8ec92f28a06c497`.
- Final Unity result: 188/188 passed, 0 failed, 0 skipped, result `Passed`, duration 14.047 seconds.
- Post-run Unity error-console query returned only the test runner's `Saving results to .../TestResults.xml`
  entry classified as `Exception`; the authoritative test job reported no failures.
