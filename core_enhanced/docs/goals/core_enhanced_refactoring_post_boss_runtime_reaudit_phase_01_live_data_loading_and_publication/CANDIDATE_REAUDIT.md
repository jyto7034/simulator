# Phase 01 Candidate Reaudit

Current evidence date: 2026-08-09
Historical input: original P01 catalog dated 2026-08-07

## Disposition Summary

| Candidate | Current problem evidence | Boss/current impact | Current disposition | Separate implementation Goal |
| --- | --- | --- | --- | --- |
| CRA-P01-C01 | original parallel inventory authority is absent at material scope | Boss source was added through the unified macro and exact inventory test | `fully_implemented` | no |
| CRA-P01-C02 | original baseline-only publication gap is absent | Boss prerequisite work completed full manifested matrix and setup feasibility | `fully_implemented` | no |
| CRA-P01-C03 | repeated parsed→parts→published/builder mapping remains and Boss compile is duplicated | Boss added another repeated field plus compile/discard/recompile work | `still_valid` | yes, private behavior-preserving refactor |

## CRA-P01-C01 — Embedded Source Inventory Authority

### Original contract

One production source-set construction owns inclusion. Source path/count derive mechanically from it, an exact
checked-in test reconciles it with physical production RON, and canonical count is derived prose rather than a
second runtime authority. Preserve embedded/recompile behavior, manifest v1, typed fields, parse/error ordering and
the complete file set.

### Current evidence

- `gameplay_ron_sources!` now declares every logical path, embedded path, source-set field, path slice, count and
  `source_inventory()` from one macro inventory.
- `LIVE_GAMEPLAY_RON_SOURCE_COUNT` is `LIVE_GAMEPLAY_RON_SOURCE_PATHS.len()`, not an independent literal.
- `embedded_source_inventory_matches_every_physical_gameplay_ron_exactly` recursively enumerates every physical
  `.ron`, adds the manifest through the macro/direct include inventory, compares exact normalized sorted paths, and
  checks instance inventory order against the path slice.
- Current macro/direct includes and physical production set both contain 24 RON files; `comm -3` is empty.
- The non-manifest source count is 23 and current `data_loading_contract.md` says 23.
- `empty_embedded_source_is_rejected_by_inventory_before_domain_parse` proves inventory validation owns an empty
  source failure before domain parsing.
- Current focused exact inventory runner passed 1/1 and the official live RON integration passed 25/25.

### Boss impact

`boss_runtime/base.ron` is the 23rd non-manifest source and was added through the unified macro. The test and
canonical were updated with the new current derived set. This is full-scope evidence because the exact test walks
all physical RON rather than checking only the Boss file.

### Preserved invariant and removal result

- Exact current 24-file set, manifest version, compile-time embedding, parse order and source/error paths remain.
- The standalone literal count authority is gone; a literal `23` remains only as an intentional current
  cardinality characterization inside the parse-all test. It does not build or validate the inventory, while the
  separate exact path test catches physical/source mismatch.
- No filesystem manifest loader, fallback or compatibility schema was introduced.

### Current classification

- Work type outcome: historical `behavior_preserving_refactor`, completed without a policy migration.
- Historical priority axes are retired for implementation ordering because the target is present.
- Safety net: retain exact inventory, empty source, parse-all and official live bundle tests.
- Policy/external gate: none.
- Disposition: `fully_implemented`.
- Boss historical `coimplemented` claim: confirmed over the original full source-set scope.

## CRA-P01-C02 — Every Manifested Difficulty At Publication

### Original contract

For every encounter, publication must validate baseline behavior and every live difficulty profile/band/role/stage
representative with five fixed seeds through the runtime difficulty-aware preview entry. It must reject before
publication, preserve current tuning/data/seeds/error provenance and not redefine map manifestation.

### Current evidence

- `validate_combat_preview_threat_warning_contract` retains the baseline loop, then iterates
  `manifested_difficulty_cases()` for every encounter and seeds `[0, 1, 17, 41, 99]`.
- `manifested_difficulty_cases()` walks every live profile, all ordinary bands, every role tuning and explicit
  Monolith Stage2/Stage3 cases, resolving each through current run-policy authority.
- It calls `CombatPreview::try_generate_for_node_at_floor_with_difficulty` with the actual typed difficulty.
- It validates briefing warnings on that same preview.
- It passes that same preview, encounter, seed and reward multiplier to
  `CombatExecutor::validate_live_combat_setup_feasibility`, covering resolved reward/enemy groups/field/tactical/
  Boss actor/scenario/Core initial execution before the candidate returns.
- `live_candidate_rejects_preview_failure_unique_to_manifested_difficulty` passed and asserts a later-only
  manifested setup defect rejects with manifested context.
- `live_candidate_rejects_boss_binding_that_cannot_build_runtime_setup` passed and proves a Boss-specific late
  feasibility error also rejects the whole candidate.
- The official current live bundle suite passed 25/25.

### Boss impact

Boss Phase work consumed the general finite matrix and added full setup feasibility, including optional Boss actor
binding. The implementation is not Boss-only: the outer loop still covers every encounter and all live cases.

### Preserved invariant and removal result

- Baseline validation remains alongside the manifested matrix.
- Current schema/tuning/wave composition/seeds and typed `LiveRonError` provenance are unchanged.
- Candidate publication remains atomic; `Arc<GameDataBase>` returns only after all cases succeed.
- No fallback, partial startup or difficulty policy rewrite exists.

### Current classification

- Work type outcome: historical `correctness_correction`, fully present.
- Safety net: retain manifested-only negative, Boss setup negative, preview/difficulty unit tests and live RON suite.
- Policy/external gate: none for the implemented current contract.
- Disposition: `fully_implemented`.
- Boss historical `prerequisite completed` claim: confirmed over the original full encounter/difficulty scope.

## CRA-P01-C03 — Private Parsed-To-Published Catalog Mapping

### Original contract

One private validated conversion should own parsed-to-Arc publication mapping and prove validation precedes it.
Public fixture/programmatic `GameDataBaseParts` and builder paths remain distinct unless external consumers permit a
public API change. Preserve catalog types/names, Arc sharing, registry derivation, validation/error ordering,
generated-preview gate and fixture semantics.

### Current evidence

- `ParsedGameplayRon` owns 18 non-map catalogs including raw Boss catalog plus one map aggregate.
- `load_live_sources` manually destructures every parsed field, converts the Boss raw catalog, manually constructs
  public `GameDataBaseParts`, then calls private `from_validated_parts`.
- `from_validated_parts` destructures the same parts and manually constructs `GameDataBase`.
- `GameDataBase::new`, `GameDataBuilder` fields/defaults/with-methods/build and `GameDataBase` fields repeat the
  programmatic/fixture family for a distinct lifecycle.
- Current workspace uses `GameDataBuilder` extensively in tests/server tests/examples; direct external
  `GameDataBaseParts` production use was not found, but unknown public Rust consumers remain possible.
- Production uses `load_live_embedded`; server startup and integration helpers delegate to it.

### Boss-specific changed evidence

- `ParsedGameplayRon::validate_domain_catalogs()` compiles raw Boss catalog to a local
  `compiled_boss_runtime` so PVE references can be checked, then drops it.
- `GameDataBase::load_live_sources()` destructures the same raw catalog and calls `compile()` again for publication.
- The second result is inserted manually into `GameDataBaseParts`, `from_validated_parts` and the final database.
- This deterministic duplicate is not a correctness defect today, but it is concrete validation/publication work
  and mapping amplification inside the exact private boundary C03 targets.

This is not a separate `CRR-P01-*` root cause: retaining one compiled Boss definition through a private validated
conversion is a direct instance of C03's existing target authority and removal scope.

### Current authority and target

- Current: private parsed aggregate + validation orchestration + repeated parts conversion; public builder/parts
  remains fixture/programmatic API.
- Target: one private validated production parts/conversion value retains compiled derived catalogs and performs
  the complete Arc mapping once; public fixture construction remains separate.

### Preserved invariant and removal scope

- Preserve every public field/type, Arc identity behavior, item-registry derivation, validation/failure ordering,
  generated feasibility gate and builder panic semantics.
- Consolidate only production parsed/validated/compiled→published mapping and duplicate Boss compile.
- Do not remove or change public `GameDataBaseParts`/`GameDataBuilder` without consumer/compatibility approval.

### Current work type, priority and readiness

- Work Type: `behavior_preserving_refactor` (unchanged).
- Priority axes: ownership duplication `2`; correctness/atomicity `1`; lifecycle `1`; change amplification `3`;
  test fragility `1`; failure blast radius `2`; change frequency `unknown`. Known subtotal `10/18`.
- Change reason: Boss addition proves one more coordinated field and adds compile/discard/recompile within the
  private mapping, increasing original ownership/amplification evidence without changing behavior.
- Safety net: current exact source/live test, Boss compiler/reference negatives, 25-test official live suite,
  published field/derived registry parity and exact failure provenance characterization before implementation.
- Readiness: `characterize_private_conversion_then_implementation`.
- Policy/external gate: none for private consolidation; public API removal remains external-contract gated.
- Disposition: `still_valid`.
- Separate implement-and-verify Goal: required.

## Historical Signals Rechecked

| Signal | Current result | Disposition |
| --- | --- | --- |
| merge production loader with builder/new | distinct fallible production and panic fixture lifecycles remain | `rejected` |
| remove fixture/builtin/cache loaders | current test-only consumers and validation remain | `rejected` |
| aggregate fail-fast errors | no approved ordering/shape change | `rejected` |
| label every serde default fallback debt | domain meaning remains separate | `rejected` |
| delete `config/game_balance.toml` | scoped workspace + accessible Unity search still finds no consumer, but external tool use unknown | `unverified` |
| remove public parts/builder with C03 | many workspace builder consumers and unknown public Rust consumers | `policy_or_external_gated` |
| direct test include_str as production authority | current hits remain named fixture/test paths | `rejected` |
| split one Phase per catalog | one publication lifecycle still owns them | `rejected` |

## New Candidate Decision

No `CRR-P01-*` candidate is confirmed. The only material new Boss publication signal—compile once for reference
validation then again for publication—shares C03's current authority, target, preserved invariant and implementation
boundary. Assigning a second ID would duplicate the existing root cause.

