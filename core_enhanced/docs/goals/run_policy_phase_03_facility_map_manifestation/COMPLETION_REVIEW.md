# P03 Facility Map And Manifestation Completion Review

Review date: 2026-07-27

Verdict: **Complete / High**

Implementation complete: true

## Audit Table

| Item | Source of truth | Verdict | Fit |
| --- | --- | --- | --- |
| Authored facility schema/live content | `facility_templates.ron`, `node_assignment_profiles.ron`, `facility.rs` | Complete | High |
| Deterministic pool and constrained assignment | `facility_generator.rs`, checkpoint-owned `FacilityRunState` | Complete | High |
| Authored navigation and typed terminals | live template connections, `progression.rs`, `RunMap` strict validation | Complete | High |
| Conditional manifestation | precommitted current map state + template manifestation rule | Complete | High |
| Generic BossOmen capstone | `FacilityCapstoneRequest`, authored `boss_omen_capstone` | Complete | High |
| Unity exact facility projection | `NodeMapTemplateCatalog`, composer/view model/UGUI renderer | Complete | High |
| Persistence and wire version contract | `RunState`, checkpoint/active-run, `MapViewDto.facility_template_version` | Complete | High |
| Legacy/canonical convergence | removed procedural schema/code/data; canonical docs updated | Complete | High |

## 1. Authored Schema And Live Content

Expected behavior:

Facility template RON owns semantic room slots, allowed categories, tags, typed terminals and directed
connections. Assignment profile RON owns Standard/Endless schedules and complete min/max/exact matching.
Invalid content fails before publication without repair or fallback.

Evidence:

- Runtime: `src/game/map/facility.rs` validates duplicate/unknown tags, slot/connection references, directed
  terminal reachability, manifestation dependencies, schedules and guardian dominators.
- Data: `game_resources/data/map/facility_templates.ron` authors Standard 3, Endless 3 and BossOmen capstone;
  `node_assignment_profiles.ron` authors exact Standard floors and open-ended Endless stages.
- Tests: `template_accepts_authored_semantic_slots_and_typed_terminal`,
  `directed_dead_end_before_terminal_is_rejected`, `manifested_mandatory_category_is_rejected` and live RON
  publication tests pass.
- Legacy audit: `generation_policy.ron`, `MapGenerator`, depth/lane slots and node-definition depth ranges are
  absent from production code/data/tests.

Long-term direction: High. Topology and assignment policy are separate live authorities with no compatibility
reader or procedural fallback.

## 2. Deterministic Generation And Persistence

Expected behavior:

The selected pool order/cycle and complete slot assignment are deterministic and checkpoint-owned. Reload and
reconnect restore the committed map instead of rerolling current weights.

Evidence:

- Runtime: `src/game/map/facility_generator.rs` performs seeded pool shuffle/no-repeat cycle and backtracking
  assignment; `FacilityRunState` is part of `RunState` and checkpoint/active-run export.
- Tests: `live_standard_floors_use_authored_increasing_templates`,
  `endless_pool_does_not_repeat_before_exhaustion`,
  `same_seed_and_state_reproduce_template_assignment_and_ids`, checkpoint and active-run round trips pass.
- Import: `RunMap::validate_facility_template_contract` rejects template version drift, missing/extra/duplicate
  slots, disallowed categories, wrong starts and typed-terminal drift.

Long-term direction: High. Saved decisions, not a reconstruction recipe, are the durable current-map source.

## 3. Navigation, Terminals And Manifestation

Expected behavior:

Core traverses authored connection directions through completed rooms only. Typed terminals determine boundary
resolution. Optional manifested rooms reveal monotonically after every prerequisite success and never reroll.

Evidence:

- Runtime: `RunMap` has no edges/depth/lane; `MapProgression` resolves live
  `FacilityTemplate.connections`. World node flows pass the resolved template into progression operations.
- Tests: completed transit, unavailable/concealed exclusion, ForwardOnly reverse rejection, typed terminal closure,
  semantic slot contract and missing-terminal rejection all pass.
- Manifestation: `manifestation_reveals_preassigned_bonus_only_after_all_prerequisites_complete` plus event/combat
  rollback/reconnect coverage confirm success-only, idempotent reveal with stable node identity/content.

Long-term direction: High. Runtime graph duplication is removed and the same template mechanism covers ordinary,
puzzle and terminal maps.

## 4. Generic Capstone

Expected behavior:

BossOmen completion atomically replaces exploration with the authored two-room capstone. Boss victory reveals
the already allocated Exit Gate; it does not append nodes or advance the Floor itself.

Evidence:

- Runtime: `FacilityCapstoneRequest` validates template kind and exact Entry inputs before building a complete
  staged map. `force_boss_node_if_ready` switches to `boss_omen_capstone`; victory only reveals its typed
  `BossOmenExitGate`.
- Tests: `authored_capstone_precommits_concealed_terminal_and_applies_entry_payload`, missing input rejection,
  `completed_boss_omen_chain_forces_single_boss_node_ahead_of_current_room`, and
  `boss_omen_event_step_consumption_forces_authored_capstone` pass.

Long-term direction: High. P05 can supply typed capstone/requirement inputs without mutating topology or adding
content-specific eligibility to the facility engine.

## 5. Unity Exact Resource And Wire Contract

Expected behavior:

Unity has an exact local v1 visual resource for every live Core template, draws static authored corridors and
ForwardOnly direction, gates concealed corridors from Core visibility, and never computes selection or fallback
coordinates.

Evidence:

- External resource: `NodeMapTemplateCatalog.cs` contains all 7 template versions, exact ordered semantic slots,
  normalized anchors, connections, directions and reveal slots.
- Wire: Core `MapViewDto` publishes `map_template_id`, required `facility_template_version` and node slot/visibility;
  Unity rejects missing/mismatched versions.
- Presentation: `NodeMapScreenComposer` consumes Core visibility/selectability; `UguiGameScreenViewAdapter`
  renders corridor lines and ForwardOnly arrows behind markers.
- Tests: exact version/slot/connection contract, version mismatch rejection, capstone reveal, marker states and
  largest-template 11-corridor/3-arrow render pass.

Long-term direction: High. Unity duplicates only exact presentation metadata; gameplay topology and eligibility
remain Core-owned.

## Cross-Component Findings

- No competing runtime topology authority remains: Core uses live template connections, snapshots omit edges,
  and Unity local connections are presentation-only.
- Template version now crosses checkpoint, Core public map DTO, Unity reader and local resource validation.
- Structural Start is synthesized from the Start role and intentionally excluded from node-definition pools.
- P04 difficulty references and P05 monolith eligibility/content were not authored early; only typed generic seams
  exist.

No immediate correction or policy question remains.

## Validation

- `cargo test -p game_core --quiet`: 761 unit; integrations 3/4/23/13/4; **808 passed, 0 failed**.
- `cargo check -p game_server --quiet`: passed.
- `rustfmt --check` on touched Rust sources: passed.
- Core/Unity scoped `git diff --check`: passed.
- Production legacy searches for generator/depth/lane/default template/placeholder identifiers: 0 findings.
- Unity MCP script validation: 7 files, 0 errors; Editor compile ready; Console errors 0.
- Unity focused MCP jobs: EditMode `ebb50f95c6f34a72b98f388adf3be049` 6/6; PlayMode
  `8ab8c03d17a04df191f47198a103eb52` 1/1.
- Unity broad MCP jobs: EditMode `d9b594cfbc5c4a0db41b22f9eff83558` 254/254; non-live PlayMode
  `db8af03403a24fa8a4fbf4354143c6d6` 62/62.
- Full PlayMode assembly also attempted four `[Explicit][Category("LiveServer")]` tests without an external
  server and those four reported connection failure; the complete non-live set was then run explicitly and is
  green. This is an external opt-in boundary, not a P03 runtime regression.

## Final Verdict

- Primary verdict: **Complete**
- Long-term fit: **High**
- Improvement class: **none**
- Remaining risk: final facility art/sound/VFX and P04/P05 policy content remain intentionally out of scope.
