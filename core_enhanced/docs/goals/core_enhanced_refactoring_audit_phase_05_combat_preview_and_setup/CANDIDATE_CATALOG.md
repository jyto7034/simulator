# Core Enhanced Refactoring Audit Phase 05 — Candidate Catalog

Catalog Date: 2026-08-07
Status: complete; dispositions accepted by phase-exit audit `Complete / High`

Priority axes are ordered as: ownership duplication, correctness/atomicity, lifecycle complexity, change
amplification, test fragility, failure blast radius, change frequency. Current accumulated dirty work makes Git
history non-representative, so change frequency is `unknown` and no automatic total-band verdict is used.

## Confirmed Candidates

### CRA-P05-C01 — Keep Serialized Preview Cache Out Of Enemy Setup Authority

- Work Type: `correctness_correction`
- Problem Evidence:
  - `CombatPreview.enemy_stat_scale` is intentionally `serde(skip)`; the focused test proves a JSON round trip
    restores `BattleUnitStatScale::default()`.
  - `RunCheckpointPayload` nevertheless serializes `HashMap<MapNodeId, CombatPreview>`, SavePoint capture clones the
    cache, durable active-run export/import round-trips it, and restore installs it without cache validation.
  - Node selection can populate a Combat preview then cancel without removing it. A sibling SavePoint can therefore
    persist an uncompleted node's cached preview.
  - Both preview resolvers return an existing entry before manifested difficulty/Monolith regeneration, and enemy
    setup consumes the cached `enemy_stat_scale` directly.
  - Thus a non-100% node can become 100% after durable Continue even though its stored difficulty remains exact.
- Current Authority And Consumers: node difficulty plus live run policy own tuning; the cache currently carries both
  presentation fields and internal setup scale; checkpoint, node preview and enemy draft setup consume it.
- Target Authority: make the cache explicitly presentation-only and regenerate or rehydrate an internal setup input
  from current node/encounter/difficulty before battle construction. A cached projection may be reused only after
  exact key/authority validation; serialized data never supplies stat scale.
- Preserved Invariants: `enemy_stat_scale` remains absent from Unity JSON; same in-memory preview fields, seeded
  wave/template choice, retry layout, Event-authored 100% policy, Monolith stage composition, active-run strictness,
  UUID continuity and current public result shapes.
- Removal Or Consolidation Scope: remove direct use of deserialized `CombatPreview` as the enemy-draft scale source
  and cached early return as implicit tuning validation. Do not expose raw scale, delete preview caching, recompute
  waves during active battle or change checkpoint load counts.
- Priority Axes And Total: `3 / 3 / 3 / 3 / 3 / 3 / unknown`; known subtotal `18/18`, no automatic band.
  Correctness risk `3` independently triggers priority investigation.
- Validation Safety Net: first add the exact production sequence: preview Combat → cancel → complete sibling
  SavePoint → active-run JSON export/Continue → select/start the original node. Assert public preview parity,
  manifested stat scale in spawned drafts, wave identity, reward multiplier, cache key and unchanged load/UUID
  semantics. Cover Event combat and Monolith cache invalidation separately.
- Policy/External Contract Decision: none for rederiving an intentionally internal value. Removing
  `combat_previews` from V3 or bumping the active-run payload is optional migration/schema work; the correction can
  preserve the current serialized field and version.
- Readiness: `test_first_then_implementation`.
- Disposition: `priority_candidate`.

### CRA-P05-C02 — Validate Full Battle-Setup Feasibility At Publication

- Work Type: `correctness_correction`
- Problem Evidence:
  - Official data validation generates five baseline previews and checks preview/threat topology only.
  - Preview validation does not know the number of unique spawn cells required by expanded enemy counts and does
    not assemble authored tactical points, field-work objects, defense-object references or final scenario groups.
  - Runtime `enemy_spawn_groups_from_preview` rejects `positions.len() < drafts.len()`, while final scenario
    reference/position validation runs only in `start_battle_execution()` during node entry planning.
  - The typed/live publication boundary can therefore accept content that fails only when the player enters combat.
  - Current live RON passes representative setup/playability tests, which masks rather than closes the accepted-
    schema gap.
- Current Authority And Consumers: data validation authorizes encounter/template/run-policy bundles; preview and
  setup modules own successive validation layers; world combat is the late failure consumer.
- Target Authority: add a pure publication-time setup-feasibility validator that assembles the empty-player live
  scenario for every preview in the Phase 1 C02 seed/difficulty-stage matrix and calls the same scenario validation
  predicates without committing a battle or consuming runtime UUID/state.
- Preserved Invariants: current live bundle, exact template/wave selection, Event/non-manifested baseline tuning,
  scenario error specificity, runtime defensive validation, no battle execution or gameplay balance change.
- Removal Or Consolidation Scope: remove publication acceptance of setup-infeasible bundles and duplicate test-only
  feasibility assumptions. Do not require a player roster, simulate battle time, add spawn cells or rebalance waves.
- Priority Axes And Total: `2 / 3 / 2 / 3 / 3 / 3 / unknown`; known subtotal `16/18`, no automatic band.
  Correctness risk `3` independently triggers priority investigation.
- Validation Safety Net: negative bundle cases for expanded enemy count over cell capacity, tactical point/work-cell
  outside every selected template, broken objective/unit reference and difficulty-added wave capacity; assert a
  structured source-aware load error. Retain all live RON and one full DefenseRoute start smoke.
- Policy/External Contract Decision: no new content rule is needed; target rejects only states the current runtime
  setup already rejects. Coordinate with Phase 1 C02 so stage/seed enumeration has one owner.
- Readiness: `test_first_after_phase_01_c02_matrix_is_defined`.
- Disposition: `priority_candidate`.

### CRA-P05-C03 — Consolidate Preview Resolution And Remove Ghost Floor Authority

- Work Type: `behavior_preserving_refactor`
- Problem Evidence:
  - `try_generate_for_node` and `try_generate_for_node_at_floor_with_difficulty` manually move the same twenty-plus
    `BattlefieldInstance` fields into `CombatPreview` in two separate blocks.
  - Normal and explicit world resolvers separately collect node/category/encounter/difficulty/seed and implement
    different cache/validation ordering.
  - `BattlefieldGenerationRequest.floor_index` is passed by public/internal APIs and tests but is never read by the
    generator. Current tests explicitly prove that one manifested difficulty ignores Floor index.
  - Phase 2 canonical made manifested node difficulty the sole scaling authority, leaving the API name/field as a
    misleading second-authority affordance.
- Current Authority And Consumers: map node/difficulty and Event explicit encounter own generation input;
  `BattlefieldInstance`, conversion blocks and two world resolvers duplicate assembly/cache decisions.
- Target Authority: one typed resolved-preview input distinguishes manifested node and Event-authored combat; one
  conversion constructs `CombatPreview` from validated instance; one resolver owns cache-key checks. Remove the
  unused Floor field/name after public-call compatibility inventory.
- Preserved Invariants: exact seed, node/category/encounter semantics, all preview fields/order, fallback mission and
  template selection, errors, cache lifetime, Event retry, Monolith invalidation and serde shape.
- Removal Or Consolidation Scope: remove duplicate field mapping, resolver boilerplate and unused Floor threading.
  Do not merge presentation and internal setup authority, change difficulty tuning or absorb C01 into an unreviewed
  broad rewrite.
- Priority Axes And Total: `3 / 1 / 2 / 3 / 3 / 2 / unknown`; known subtotal `14/18`, no automatic band.
- Validation Safety Net: golden equality for normal, Boss, Event and Monolith inputs; all invalid difficulty and
  encounter mismatch errors; deterministic multi-seed output; cache hit/miss and conversion field-completeness
  characterization. Compile-search external/public callers before signature removal.
- Policy/External Contract Decision: private consolidation needs none. Removing public `*_at_floor` APIs needs an
  external Rust compatibility decision; they may remain deprecated wrappers if consumers are unknown.
- Readiness: `after_C01_authority_split_and_public_caller_inventory`.
- Disposition: `candidate`.

### CRA-P05-C04 — Build One Typed Combat-Setup Plan Before World Commit

- Work Type: `behavior_preserving_refactor`
- Problem Evidence:
  - World planning resolves preview, mission rewards, reward scale and attempt policy separately, then calls
    `CombatExecutor` and manually fills `ActiveBattleSession`.
  - Reward resolution, `BattleStartPlan::from_preview` and scenario assembly each look up the same encounter.
  - Battlefield, player, defense-object, enemy-group and scenario-event assembly is split across sibling helpers;
    their final cross-field validation occurs only when world calls `start_battle_execution()`.
  - Current paths pass and no partial setup commit was established, but adding a setup-owned field or validation
    requires synchronized edits across several modules and the world constructor.
- Current Authority And Consumers: each data domain keeps its own authority; `world/combat.rs` currently owns the
  aggregate orchestration and `ActiveBattleSession` field mapping.
- Target Authority: one private `ResolvedCombatSetup`/builder resolves the encounter once, accepts an authoritative
  internal preview input, produces validated scenario/BattleCore/execution plus reward/mission fields, and hands one
  typed plan to the existing world commit. Active battle lifecycle remains outside the builder.
- Preserved Invariants: empty initial player group, live deployment policy, defense object/profile, authored tactical
  overrides, group/event order, spawn timing, reward order/rounding, exact errors, BattleCore seed and world entry
  snapshot/UUID/attempt/run-peak semantics.
- Removal Or Consolidation Scope: remove repeated encounter lookup and manual cross-module setup result joining.
  Do not create a trait framework, merge reward/economy ownership, move post-start simulation into setup or widen
  Event lifecycle authority.
- Priority Axes And Total: `3 / 1 / 2 / 3 / 2 / 3 / unknown`; known subtotal `14/18`, no automatic band.
- Validation Safety Net: before/after golden scenario, setup snapshot, first event checkpoint and reward plan for
  normal Defense, Boss, survival timer, authored tactical plan and Event combat; exact late validation failure and
  full world no-mutation assertions; deterministic enemy UUID/order repeat.
- Policy/External Contract Decision: none for a private builder. Public `CombatExecutor` compatibility and test
  harness APIs must remain or receive a separate consumer decision.
- Readiness: `after_C02_feasibility_predicate_and_setup_parity_characterization`.
- Disposition: `candidate`.

### CRA-P05-C05 — Remove No-Op Mission Policy Branches And Shape-Only Tests

- Work Type: `code_cleanup`
- Problem Evidence:
  - `preferred_node_types_for_map_node` checks whether `kind_id` contains `elite`, but both the elite and ordinary
    Combat branches return exactly `[Defense]`.
  - `fallback_mission_variant_for_archetype` ignores its archetype argument, returns
    `default_for_node_type`, and has no production caller; its only caller is its own unit test.
  - `elite_map_nodes_prefer_high_pressure_missions` does not compare elite with normal behavior, and
    `battlefield_archetype_does_not_change_mission_variant` protects only the unused wrapper.
  - `defense_node_type_builds_fixed_defense_objective` is a strict subset of the stronger default-defense setup test;
    two reward-policy currency-only failure tests also overlap and can be merged.
- Current Authority And Consumers: mission selection remains the explicit current policy table; the identified
  branches/helpers/tests add no distinct runtime contract.
- Target Authority: simplify Combat preference to category policy, delete or privatize the unused variant wrapper,
  and merge/remove only tests whose assertions are fully subsumed by named surviving cases.
- Preserved Invariants: supported node types, SplitRoom rejection, Boss mapping, live mission pairs, featured reward
  kinds, default defense object/route/win condition and all public results.
- Removal Or Consolidation Scope: remove the no-op branch, unused wrapper and four redundant test bodies after exact
  assertion transfer. Do not data-drive mission policy, add variants or simplify the explicit SplitRoom failure.
- Priority Axes And Total: `1 / 1 / 1 / 1 / 2 / 1 / unknown`; known subtotal `7/18`, no automatic band.
- Validation Safety Net: retain explicit category matrix, SplitRoom negative, valid/invalid mission-pair matrix,
  stronger default-defense scenario assertions and reward empty/featured/forbidden cases; run workspace/public
  caller search before deleting exported helpers.
- Policy/External Contract Decision: private/no-caller cleanup needs none. Public helper removal needs external Rust
  compatibility confirmation or a deprecated wrapper.
- Readiness: `ready_after_assertion_transfer_and_caller_inventory`.
- Disposition: `cleanup_candidate`.

## Rejected, Deferred, And Unverified Signals

| Signal ID | Signal | Evidence and reasoning | Disposition |
| --- | --- | --- | --- |
| CRA-P05-S01 | Duplicate Phase 1 C02 as a new baseline/all-stage preview candidate | Phase 1 already owns publication preview stage coverage. C02 is distinct setup feasibility and explicitly consumes that matrix. | `rejected` duplicate |
| CRA-P05-S02 | Treat FacilityEntity preview/threat/setup branches as a current broken feature | Live validation deliberately rejects every FacilityEntity wave and canonical labels it future-only. | `rejected`; feature/policy work only |
| CRA-P05-S03 | Scale manual wave counts with generated-wave multiplier | Canonical and focused tests say only generated wave budget changes; manual authored counts remain exact. | `rejected` balance/contract change |
| CRA-P05-S04 | Recompute preview waves during battle or retry | Preview-fixed enemy entries are canonical setup authority and deterministic retry evidence depends on them. | `rejected` |
| CRA-P05-S05 | Classify initial world start as generally non-atomic | Normal entry uses the full state transaction and all known setup failures occur during planning. Event paired commit is Phase 4 ownership; no distinct valid-data failure was proven here. | `rejected` standalone correction |
| CRA-P05-S06 | Make `EnemyBriefing` the spawn source | Canonical explicitly makes it presentation-only; `SpawnWave.enemy_entries` is the actual source. | `rejected` |
| CRA-P05-S07 | Treat different preview appearance seeds and runtime owned UUID seeds as duplicate authority | They identify different consumers: presentation variation versus runtime owned actor identity. No collision/divergence defect was found. | `rejected` |
| CRA-P05-S08 | Move defense-object constants to live data now | Current mission policy intentionally remains a small code table until real variants exist. This is content extensibility, not current duplication. | `deferred` feature/policy |
| CRA-P05-S09 | Let current roster or Floor index rescale an existing node | Canonical requires immutable manifested node difficulty. Current roster only changes relative-threat/readiness projection. | `rejected` |
| CRA-P05-S10 | Remove preview caching entirely to fix C01 | Cache has valid node-confirm/retry projection value. Separating serialized presentation from internal setup authority is sufficient and lower-risk. | `rejected` broad removal |
| CRA-P05-S11 | Treat armed certified membership as a canonical mismatch | Current `core_runtime_contract.md` explicitly requires deploy/range/certified membership to share armed eligibility. | `rejected` |
| CRA-P05-S12 | Delete public generation/executor helpers because workspace production has no caller | Integration tests and unknown external Rust consumers exist; workspace search is not compatibility proof. | `unverified`; C03/C05 wrapper decision |

## Cross-Boundary Handoff

- Final should sequence C01 before C03 and C02 before C04. C01 and C02 are independent priority corrections.
- Phase 1 C02 should provide the shared published seed/difficulty-stage enumeration; Phase 5 C02 owns downstream
  scenario feasibility within that matrix.
- Phase 2/10 need only review a checkpoint/DTO schema change if implementation chooses removal/versioning. C01 does
  not require such a change.
- Phase 3 C01 remains the persistent economy-graph prerequisite for safe employee/equipment setup input.
- Phase 4 C03 remains Event paired-session/start commit ownership.
- Phase 6 may rely on a successfully validated `ActiveBattleSession`; it must not reopen preview/data publication.
