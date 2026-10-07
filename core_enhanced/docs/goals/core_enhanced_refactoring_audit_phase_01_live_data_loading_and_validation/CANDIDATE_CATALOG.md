# Core Enhanced Refactoring Audit Phase 01 — Candidate Catalog

Catalog Date: 2026-08-07
Status: investigation complete; final Phase disposition pending completion audit

Priority axes are ordered as: ownership duplication, correctness/atomicity, lifecycle complexity, change
amplification, test fragility, failure blast radius, change frequency. Current accumulated dirty work makes Git
history non-representative, so change frequency is `unknown` and no automatic total-band verdict is used.

## Confirmed Candidates

### CRA-P01-C01 — Consolidate Embedded Source Inventory Authority

- Work Type: `behavior_preserving_refactor`
- Problem Evidence:
  - `GameplayRonSourceSet::embedded()` is the real source list: 1 manifest + 22 non-manifest files.
  - `LIVE_GAMEPLAY_RON_SOURCE_COUNT = 22` is a separate literal used only by a test that compares it with another
    literal `22`; it does not enumerate the source fields or live directory.
  - `data_loading_contract.md` independently says 21, proving actual drift between manual copies.
  - The current embedded/live sorted sets are both 23 and equal, but no checked-in exact-set test detects a newly
    authored, unreferenced file.
- Current Authority And Consumers: actual authority is `GameplayRonSourceSet::embedded()` plus
  `parse_gameplay_ron_sources()`; manual count constant/test/canonical prose are parallel derived claims. Server
  startup and `tests/common::load_game_data_from_ron()` consume the official loader.
- Target Authority: production source-set construction remains the single authority; source count and the exact
  file-set safety check derive from or mechanically enumerate that authority. Canonical records a derived fact,
  not an independent number.
- Preserved Invariants: exact current 23-file set, manifest version 1, compile-time embedding/recompile
  requirement, typed source fields, parse order, error source/domain paths, and no filesystem hot-load behavior.
- Removal Or Consolidation Scope: remove the standalone unverified count literal or make it mechanically derived;
  replace the tautological count assertion with an exact source/live reconciliation test; align stale canonical
  prose. Do not turn `bundle.ron` into a new runtime compatibility manifest without separate policy approval.
- Priority Axes And Total: `2 / 1 / 0 / 2 / 2 / 2 / unknown`; known subtotal `9/18`, no automatic band.
- Validation Safety Net: first add a characterization test proving the 23 exact normalized paths and a negative
  fixture proving an extra live file/reference mismatch is reported; rerun data unit and `ron_loading` suites.
- Policy/External Contract Decision: none for deriving existing inventory. A new deploy-time manifest/file loader
  would be a separate policy/feature migration and is not part of this candidate.
- Readiness: `ready_after_characterization_test`.
- Disposition: `candidate`.

### CRA-P01-C02 — Validate Every Authored Manifested Difficulty At Publication

- Work Type: `correctness_correction`
- Problem Evidence:
  - Current canonical requires every encounter at every run-policy scaling stage's first floor and five fixed
    seeds to pass generated preview/briefing validation.
  - `validate_combat_preview_threat_warning_contract()` loops every encounter and five seeds but always calls
    `try_generate_for_node_at_floor(..., floor_index: 0)` without `MapNodeCombatDifficulty`; its diagnostics name
    `baseline difficulty`.
  - Runtime paths in `world/combat.rs` call `try_generate_for_node_at_floor_with_difficulty()` with the manifested
    node difficulty.
  - Difficulty-specific tuning changes generated-wave budget and `extra_waves`, so a later-stage-only generated
    instance can differ materially from the startup baseline path.
  - The 24-test integration suite and generated-preview test repeat baseline/no-difficulty generation; no focused
    test mutates only a nonbaseline tuning and proves startup rejection.
- Current Authority And Consumers: startup publication gate owns baseline validation; run policy plus manifested
  `MapNodeCombatDifficulty` owns actual runtime tuning; `GameCore` is the direct consumer after publication.
- Target Authority: the publication gate enumerates the finite authored difficulty profile/band/role/stage
  representatives used by runtime and validates every encounter with the same difficulty-aware generation entry,
  five deterministic seeds, and first-floor representative. Run policy remains tuning authority.
- Preserved Invariants: current authored data and schema, five seed values, deterministic generation, typed
  `LiveRonError` provenance, immutable candidate publication, no fallback, and no partial server startup.
- Removal Or Consolidation Scope: remove the baseline-only gap and the duplicated baseline-only integration
  assumption. Do not redefine difficulty tuning, wave composition, reward policy, or map manifestation.
- Priority Axes And Total: `2 / 3 / 1 / 1 / 3 / 3 / unknown`; known subtotal `13/18`, no automatic band.
  Correctness risk `3` independently triggers priority investigation.
- Validation Safety Net: add a failing characterization case where baseline generation succeeds but one authored
  manifested tuning/extra-wave contract fails; prove loader rejection includes profile/stage context; rerun data,
  preview, and live RON integration suites. Measure startup cost for the finite matrix.
- Policy/External Contract Decision: none; current durable data contract already requires the broader gate. If the
  canonical requirement is intentionally to be weakened for startup-cost reasons, that would require an explicit
  policy decision instead of silently changing this correction.
- Readiness: `test_first_then_implementation`; current negative safety net is missing but the target contract is
  approved and runtime entrypoint exists.
- Disposition: `priority_candidate`.

### CRA-P01-C03 — Consolidate The Private Parsed-To-Published Catalog Mapping

- Work Type: `behavior_preserving_refactor`
- Problem Evidence:
  - Seventeen non-map catalog fields plus the map aggregate are repeated across `ParsedGameplayRon`, the loader
    destructure, `GameDataBaseParts`, and `GameDataBase::from_validated_parts()`; `GameDataBase` and
    `GameDataBuilder` repeat the published/fixture field family again.
  - Adding one catalog requires coordinated edits to source parsing, validation orchestration, conversion,
    publication fields, builder defaults, tests, and canonical index. C01 proves that manual synchronized loader
    metadata has already drifted in one adjacent dimension.
  - The current mapping is correct and all live tests pass; this is change-amplification/omission risk, not a
    current runtime defect.
- Current Authority And Consumers: `ParsedGameplayRon` owns private parsed values;
  `GameDataBase::load_live_embedded()` manually converts them through private `from_validated_parts()`;
  `GameDataBaseParts`/builder serve a distinct public fixture/programmatic seam.
- Target Authority: one private validated publication conversion owns the parsed-to-Arc field mapping and proves
  it can only be called after domain validation. The public fixture/programmatic path remains separate unless
  external consumer evidence later permits an API change.
- Preserved Invariants: catalog field names/types, `Arc` sharing, item-registry derivation, validation order and
  error paths, post-assembly preview gate, public `GameDataBase` behavior, and fixture panic semantics.
- Removal Or Consolidation Scope: consolidate only private production conversion boilerplate and its validation
  state marker. Do not merge production and fixture loaders or remove public `GameDataBaseParts`/builder APIs in
  the first implementation.
- Priority Axes And Total: `1 / 1 / 1 / 2 / 1 / 2 / unknown`; known subtotal `8/18`, no automatic band.
- Validation Safety Net: characterization covering every published catalog/derived registry from the official
  loader, typed error provenance before publication, and the existing 162 + 24 focused suites.
- Policy/External Contract Decision: private-only consolidation needs none. Removing/changing public construction
  APIs requires other Rust consumer evidence or a compatibility decision and is explicitly deferred.
- Readiness: `ready_if_private_scope_is_preserved`.
- Disposition: `candidate`.

## Rejected, Deferred, And Unverified Signals

| Signal ID | Signal | Evidence and reasoning | Disposition |
| --- | --- | --- | --- |
| CRA-P01-S01 | Merge production loader with `GameDataBuilder`/`GameDataBase::new()` | Canonical and code deliberately separate fallible source-aware production loading from panic-oriented minimal fixture construction; no production consumer uses the builder. Similar catalog shapes do not share lifecycle/failure semantics. | `rejected` |
| CRA-P01-S02 | Remove domain fixture loaders (`RunPolicyData::builtin`, cached map catalogs, live buff helper) as duplicate production readers | They are explicit test-only helpers, parse current schema, validate on construction/use, and never feed server startup. Removing them solely for structural similarity would merge independent test and production lifecycles. | `rejected` |
| CRA-P01-S03 | Replace fail-fast `LiveRonError` with aggregated errors | No approved contract requires aggregation; fail-fast source/domain/object/path/cause is deterministic, tested, and has no silent fallback. Changing failure ordering/shape is not behavior-preserving evidence. | `rejected` |
| CRA-P01-S04 | Treat all `serde(default)` as fallback debt | Current canonical permits neutral optional/default authoring; strict raw types and 162 focused tests reject gameplay-meaningful missing/unknown/legacy fields. Field-by-field domain policy belongs to the owning later Phase when consumption semantics matter. | `rejected` |
| CRA-P01-S05 | Delete `core_enhanced/config/game_balance.toml` | No workspace reference or generic Core loader was found, but external authoring/tool consumers are unknown. Absence search is insufficient deletion proof. | `unverified`; defer pending consumer inventory |
| CRA-P01-S06 | Remove public `GameDataBaseParts` and builder APIs while doing C03 | Workspace production does not use them, but other public Rust crate consumers are unknown. C03 is safe only while private. | `deferred` external-contract/API decision |
| CRA-P01-S07 | Treat direct test `include_str!` references as competing production authority | Occurrences outside the official source set are inside named fixture/unit-test helpers and do not publish server data. | `rejected` |
| CRA-P01-S08 | Split one Phase per catalog because files/types are large | All catalogs share one parse/validate/publication lifecycle; domain consumption is already separated into Phases 2–9. File size and physical module ownership are not source-of-truth boundaries. | `rejected` |

## Cross-Boundary Handoff

- Phase 2 and later may trust that all current 23 files are included and that the official loader publishes one
  immutable database only after the currently implemented gates pass.
- They must not assume every manifested difficulty variant has been startup-generated until C02 is implemented;
  current startup evidence is baseline-only.
- Domain Phases own gameplay interpretation of defaults and catalog values. Phase 1 has classified only their
  loading/validation/publication behavior.
- Final consolidation should compare C01 and C03 for a shared root cause (manual source/catalog metadata
  repetition) without merging C02's independent correctness gap.

