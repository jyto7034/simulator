# G12 Gameplay RON Strictness And Bundle Validation Completion Review

Reviewed: 2026-07-13

Guide: `docs/goal_completion_review_guide.md`

Primary verdict: Complete

Long-term fit: High

Improvement class: none

Immediate correction recommended: no

## Review Inputs

- Master item: G12 gameplay RON strictness and bundle validation.
- Subgoal: `docs/goals/gameplay_ron_strictness_and_bundle_validation/`.
- Policy source: PD-09 and this goal's accepted contract.
- Findings: DCA-H-04, CPR-H-08, CPR-M-01 supported-mode cardinality portion, CPR-M-02.
- Runtime owners: `src/game/data/live_ron.rs`, `src/game/data/mod.rs`, domain data modules,
  `src/game/data/validation.rs`, map/combat-preview consumers, and `game_server/src/main.rs`.
- Live source: `/mnt/f/work/simulator/game_resources/data/bundle.ron` plus 23 gameplay RON files.
- External contract impact: no Unity-facing DTO or transport shape change.

## Item Audit Table

| Item | Source of truth after G12 | Verdict | Fit | Immediate correction |
| --- | --- | --- | --- | --- |
| Exact schema and complete source set | Core manifest/source-set parser | Complete | High | none |
| Canonical strict domain parsing | Typed RON schemas and fallible domain validators | Complete | High | none |
| Duplicate and cross-catalog validity | Final parsed-bundle validator | Complete | High | none |
| RunStart/Event/extra-wave/map contracts | Final parsed-bundle validator | Complete | High | none |
| All-or-nothing startup publication | `GameDataBase::load_live_embedded()` return boundary | Complete | High | none |
| Map/battlefield/generated preview validity | Stored map catalogs and generated validation gate | Complete | High | none |
| Server and Unity regression boundary | Core startup result; unchanged Unity contract | Complete | High | none |

## Exact Schema And Complete Source Set

Master item: establish one exact-version canonical bundle rather than independent lenient readers.

Expected behavior: schema version mismatch or any source parse failure prevents gameplay startup.

Runtime evidence:

- `src/game/data/live_ron.rs` owns `LIVE_RON_SCHEMA_VERSION = 1`, the manifest parser,
  `GameplayRonSourceSet`, source-aware parse diagnostics, and all 23 gameplay source paths.
- `src/game/data/mod.rs` exposes `GameDataBase::load_live_embedded()` as the production entry point.
- Map generation policy, node definitions, battlefield archetypes, and battlefield templates are
  parsed into the same source set and stored in the resulting database rather than reread through a
  second production builtin.

Data evidence:

- `/mnt/f/work/simulator/game_resources/data/bundle.ron` declares the exact current version.
- The current embedded source set parses and validates through the production entry point.

Test evidence:

- Live-RON focused: 41/41 passed, including missing/wrong/unknown manifest fields and missing
  generated fields.
- Core all-targets: 710/710 passed.

Legacy/fallback audit:

- No Serde alias remains in the reviewed production source scopes.
- Direct CorrodedWave, Buff, and RunPolicy readers are test/fixture helpers, not competing server
  startup loaders.

Long-term direction review:

- Fit: high.
- Improvement class: none.
- Reason: one version/source-set authority replaces per-domain production reads without introducing
  a compatibility schema.

Verdict: Complete.

Remaining risk: the external authoring application must adopt or reproduce this exact schema; it is
not present in this workspace and remains an integration follow-up.

## Canonical Strict Domain Parsing

Master item: reject unknown, removed, missing-required, unsupported, and invalid authored data before
play, with defaults retained only for classified optional meaning.

Expected behavior: malformed generated data returns a typed source/object/path/cause failure instead
of being silently repaired or reaching a runtime panic.

Runtime evidence:

- Domain types use unknown-field denial and fallible local validators for Consumable, Buff, employee,
  CorrodedWave, Shop, Reward, BossOmen, Event, PVE, Visual Novel, trigger, Skill, Skill Fragment,
  Equipment, Abnormality, and CorrodedEmployee data.
- Custom Abnormality, CorrodedEmployee, and Skill deserializers resolve raw authoring input without
  invoking panic constructors.
- Generated Skill identity/timing/step fields, unit combat fields, CorrodedWave balance fields, and
  PVE manual enemy tier/count and wave time are explicit where live data already authors gameplay
  meaning.

Data evidence:

- Live RON uses canonical enum spellings and required field shapes; removed aliases and the
  unsupported Event `ApplyBossOmenStepResult` variant have no live dependency.

Test evidence:

- Negative tests cover unknown fields, missing required fields, invalid bounds, unsupported variants,
  raw preset resolution, and typed paths.
- PVE focused 11/11, CorrodedWave 2/2, Skill 11/11, Abnormality 4/4, and CorrodedEmployee 6/6 passed
  in the recorded focused runs.

Legacy/fallback audit:

- No ignored test preserves a legacy live schema.
- The old redundant panic-string Abnormality interval test was removed after the production-boundary
  typed negative superseded it.

Long-term direction review:

- Fit: high.
- Improvement class: none.
- Reason: validators are reusable by the final bundle gate and fixture constructors unwrap those same
  contracts rather than maintaining separate policy tables.

Verdict: Complete.

Remaining risk: optional/default fields remain a deliberate review obligation when future domain
goals add gameplay meaning; G12 did not pre-decide G03 Event UX or other out-of-scope policies.

## Duplicate, Cross-Catalog, And Flow Contracts

Master item: close first-match, late-reference, supported-mode, Event, extra-wave, stage/mission, and
map payload gaps.

Expected behavior: every accepted key is unique, every required reference resolves, every supported
mode has exactly one valid RunStart trigger, and no accepted Event effect reaches an unsupported
runtime branch.

Runtime evidence:

- `ParsedGameplayRon::validate_domain_catalogs()` orders top-level key/UUID, domain, cross-item,
  pool, RewardEffect, PVE, RunStart, extra-wave, map-catalog, and map-payload validation before the
  candidate is exposed.
- Shop/Reward/equipment pool IDs reject duplicates before first-match lookup is observable.
- Reward/Event/PVE/Equipment, VN trigger, Skill/Fragment/unit, BossOmen, starter loadout, and map
  payload references use source-owning fallible validators.
- Standard and Endless each require one blocking, OncePerRun, PersistAcrossCheckpointLoad RunStart.

Data evidence:

- Current live Event data does not contain the removed unsupported omen-step-result effect.
- Current live RunStart triggers and map/extra-wave references satisfy the final validator.

Test evidence:

- Live-RON negatives cover duplicate pools and catalog keys, RunStart absence/policy, nested Reward/
  Event/PVE/Skill references, extra waves, map payloads, and source-aware paths.
- Core and server broad suites passed after focused validation.

Legacy/fallback audit:

- No duplicate winner, alias, dual schema, or ignored legacy flow remains in the reviewed scope.

Long-term direction review:

- Fit: high.
- Improvement class: none.
- Reason: domain validators compose into one ordered final authority, preserving the defining source's
  diagnostic instead of adding a second registry.

Verdict: Complete.

Remaining risk: every future catalog/reference-bearing domain must be added to this shared ordering;
that is an extension rule, not a current correctness gap.

## All-Or-Nothing Publication And Generated Validation

Master item: prevent partial bundle use and validate generated combat combinations before play.

Expected behavior: a failed parse/domain/reference/generated check returns an error, the server does
not bind, and no candidate database becomes observable.

Runtime evidence:

- `GameDataBase::load_live_embedded()` parses and validates the source set, assembles a private
  candidate through `from_validated_parts`, validates generated preview/briefing contracts against
  it, and returns the Arc only on success.
- Production does not re-enter fixture `GameDataBuilder`/`GameDataBase::new()` panic wrappers.
- `game_server/src/main.rs` maps `LiveRonError` to startup `InvalidData` before bind.
- Generated validation covers every encounter at the first floor of every scaling stage across five
  fixed seeds and validates required briefing warnings.

Data evidence:

- Runtime map and combat-preview consumers use the exact validated catalogs stored in the database.

Test evidence:

- `cargo test -q -p game_core --all-targets -- --test-threads=1`: 710 passed, 0 failed/ignored.
- `cargo test -q -p game_server --all-targets -- --test-threads=1`: 27 passed, 0 failed/ignored.
- `cargo fmt --all -- --check`, Core all-target check, and game_server all-target check passed.

Legacy/fallback audit:

- Candidate allocation precedes generated validation because preview generation requires the
  assembled database, but candidate publication does not; no candidate escapes on failure.
- No fallback or partial bundle path exists in server startup.

Long-term direction review:

- Fit: high.
- Improvement class: none.
- Reason: the return boundary is a single all-or-nothing authority and the private candidate avoids
  duplicating preview-generation inputs merely to encode a typestate marker.

Verdict: Complete.

Remaining risk: five deterministic seeds are representative rather than an exhaustive proof of every
random output; domain validation and every-stage coverage bound this risk, and broader exhaustive
generation would be a test-strengthening follow-up rather than an immediate correction.

## External Contract Evidence

G12 does not serialize gameplay RON diagnostics or schema to Unity and changes no Unity-facing DTO,
command, snapshot, or WebSocket message. The external player contract is therefore unchanged.

Evidence:

- Unity MCP instance `ark@b49d32f6f2f70bbe`, Unity 6000.3.4f1, project
  `F:/unity projects/ark` was ready and idle before the run.
- Full EditMode Unity MCP job `b9128241e23c48a6b8ec92f28a06c497`: 188/188 passed, 0 failed,
  0 skipped, result `Passed`.
- Post-run editor state was idle, not compiling, not playing, and ready for tools.
- The error-console filter returned only Unity Test Runner's results-file message; the job contained
  no failed test details.

Verdict: Complete.

Long-term fit: High. Core retains data-validity authority and Unity does not reproduce RON rules.

## Cross-Component Findings

- Core loader, map/preview consumers, and game_server startup agree on one bundle instance and one
  failure boundary.
- No Unity DTO or transport document needs a schema migration for G12.
- `docs/data_loading_contract.md` was corrected to describe the versioned source set, stored map
  catalogs, private candidate, generated validation, and return/publication boundary.
- Fixture panic wrappers remain intentionally outside production startup and unwrap the same
  validators; they are not compatibility or fallback paths.

## Immediate Correction Candidates

None.

## Follow-Up Refactor Candidates

- Integrate the external dedicated authoring program with the exact schema/version and validator
  contract when that program enters this workspace or exposes an export interface.
- Consider a larger deterministic seed corpus only if future generator complexity produces evidence
  that five seeds plus semantic validators are insufficient.

Neither candidate blocks G12 completion.

## Final Verdict

- Verdict: Complete.
- Long-term fit: High.
- Improvement class: none.
- Immediate correction recommended: no.
- Policy questions: none.
