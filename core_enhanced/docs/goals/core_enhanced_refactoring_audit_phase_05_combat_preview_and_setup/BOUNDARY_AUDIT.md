# Core Enhanced Refactoring Audit Phase 05 — Boundary Audit

Audit Date: 2026-08-07
Status: complete; phase-exit audit `Complete / High`

## Exact Boundary

Phase 5 owns the current manifested-node to `CombatPreview` to battle-start setup seam. It reads persistence,
economy and battle runtime only far enough to determine input authority, plan validation and the first successful
handoff into `ActiveBattleSession`.

### Primary runtime

- all five files under `src/game/combat_preview/**`
- all nine files under `src/game/combat_setup/**`
- `src/game/wave_resolution.rs`
- `src/game/combat_power.rs`
- `src/game/events/combat.rs` at preview-to-scenario construction only
- `src/game/world/combat.rs` at preview cache, start planning and initial commit only

### Direct carriers and consumers

- `src/game/world/{state,node_flow,active_run}.rs` for preview-cache and checkpoint carriage
- `src/game/data/{pve_data,run_policy_data,equipment_data,validation}.rs`
- `src/game/pve_bonus_objectives.rs` and `src/game/reward_policy.rs` only at setup/result policy seams
- `src/game/battle/{scenario,core/sim}.rs` only to identify the final setup validator and first execution boundary
- published encounter, corroded-wave, battlefield, run-policy and equipment RON consumed by setup

### Tests and canonical

- preview/setup inline tests, `events/combat.rs` setup tests, selected world combat/map-flow tests and
  `tests/ron_loading.rs`
- `docs/{game_rulebook,core_runtime_contract,data_loading_contract}.md`

Active battle clock/events/results remain Phase 6; live deployment/spatial mutation remains Phase 7; formulas,
effects and death remain Phases 8~9; DTO/server/Unity refactor discovery remains Phase 10.

## Authority Matrix

| Domain | Input/current authority | Derived plan or cache | First consumer | Finding |
| --- | --- | --- | --- | --- |
| Node combat identity | manifested map node category/payload | node id/category/encounter input | preview resolver | authoritative for normal Combat/Boss |
| Event-authored combat identity | committed Event action/session | explicit Combat category plus encounter | same preview generator | Phase 4 lifecycle carrier; no manifested difficulty |
| Difficulty | node `MapNodeCombatDifficulty` plus live run policy | stat scale, generated-wave budget, extra waves, reward multiplier | preview enemy drafts and rewards | exact gameplay authority; Floor index is not authority |
| Battlefield | live archetype/template policy plus encounter override | `BattlefieldInstance`, then `CombatPreview` | scenario battlefield | deterministic seed selection and strict topology validation |
| Waves | authored `PveWaveData`, preset database and difficulty tuning | resolved `SpawnWave.enemy_entries` | scenario enemy spawn groups | preview result is fixed; battle does not regenerate waves |
| Mission | encounter node type/variant, explicit fallback policy | tactical plan, objective and win condition | scenario validation | current live pairs are Defense/Defense and Boss/Boss |
| Player eligibility | persistent employee condition plus equipped weapon | certified power and on-command employee draft | live deployment | armed eligibility agrees at command boundary |
| Enemy identity | resolved entry plus live abnormality/corroded profile | deterministic draft UUID and scenario unit ref | battle spawn event | stable for one preview/order; FacilityEntity remains publication-rejected |
| Rewards | encounter reward UUIDs plus mission policy and node difficulty | scaled reward options | active battle result state | quantity-only scaling; Event combat stays 100% |
| Preview cache | `RunState.combat_previews` | node-confirm/retry projection | preview and setup resolver | currently mixes serialized presentation with internal setup scale |

## Preview Authority Trace

1. Selecting a node calls `combat_preview_for_node`. A normal Combat/Boss node must carry manifested difficulty;
   the resolver validates Monolith stage authority, derives the namespaced `PREV` seed and generates a preview.
2. `BattlefieldGenerator` resolves encounter override or deterministic archetype/size/template fallback, parses the
   ASCII battlefield, applies authored obstacles, resolves difficulty tuning, fixes waves, briefing and warnings,
   then validates battlefield topology.
3. `CombatPreview` copies the complete `BattlefieldInstance` field set and is cached by node id. Event combat uses
   the same generator with an explicit Combat category and no manifested difficulty.
4. Node cancel clears only the active `NodeSession`; it intentionally leaves the preview cache so a subsequent
   selection reuses the deterministic projection.
5. Both normal and explicit resolvers return a cached value before revalidating/rederiving difficulty. This is safe
   only while the cache retains its internal fields.

The node's manifested difficulty, not current roster or Floor number, is the tuning authority. The
`BattlefieldGenerationRequest.floor_index` field and `*_at_floor` names are still threaded through callers but no
generation expression reads the field. The current difficulty test deliberately proves equal output across Floor
values. This is stale API authority, not a missing Floor-scaling bug.

## Durable Cache Correctness Gap

`CombatPreview.enemy_stat_scale` is marked `serde(skip)` and its own unit test proves that JSON round trip restores
the default 100% scale. That is correct for a Unity-facing preview and matches canonical. The same type is also
stored in `RunCheckpointPayload.combat_previews`, however:

1. previewing then cancelling an available Combat node leaves its cache entry;
2. completing a selectable sibling SavePoint clones all cache entries into the checkpoint;
3. active-run export serializes that payload and Continue deserializes/restores the cache;
4. selecting or starting the cached node returns it before manifested difficulty regeneration; and
5. `enemy_spawn_groups_from_preview` copies the restored default scale into every enemy draft.

This is a production-reachable violation of `core_runtime_contract.md`: a player can preview/cancel a Combat node,
take a sibling SavePoint, continue the durable run and later receive 100% enemies for a non-100% manifested node.
The checkpoint validator validates node difficulty but never correlates or regenerates cached preview internals.
The current serialization unit test documents the lossy property but no world/active-run test covers this sequence.

## Wave And Battlefield Setup Trace

- Manual waves are copied exactly. Generated-corroded waves use a namespaced deterministic seed, preset/override
  budget, difficulty multiplier, min/max/weight policy and resolve to explicit preview entries.
- Corroded visual `appearance_seeds` are deterministic preview data; runtime owned UUIDs are separately derived
  from base UUID, wave index and global spawn index. Runtime does not infer spawn identity from briefing.
- Preview validation checks dimensions, exact canonical/valid/obstacle tile sets, zone/route identities, references,
  route contiguity/shared defense endpoint and one deployment-to-spawn path.
- Setup sorts waves by time, resolves unique cells, builds enemy drafts, rejects insufficient cell capacity, builds
  default/authored tactical policy and defense object, then creates a `BattleScenario`.
- `BattleScenario::validate()` runs only when `start_battle_execution()` is called during world start planning. It
  adds checks for group/event/unit identity, spawn capacity/result positions, tactical points, field-work cells,
  objective/win references and scenario topology.

### Publication gap

Official bundle validation generates baseline previews for five seeds and checks threat-warning coverage. It does
not execute the setup-only feasibility checks. Accepted data can therefore generate a valid preview and still fail
at combat entry because a wave needs more unique spawn cells than its zones provide, an authored tactical/field-work
position is invalid for the selected template, or the assembled objective/unit references are inconsistent.

Phase 1 C02 already owns the separate fact that publication preview validation uses only baseline difficulty. The
Phase 5 correction is narrower and downstream: for the eventual Phase 1 seed/stage matrix, validate that every
generated preview can assemble and validate the actual empty-player live scenario without committing runtime state.

## Player, Power And Reward Trace

- Certified power filters `is_battle_deployable`, so condition eligibility and weapon requirement agree with live
  deploy/range-preview admission. It snapshots persistent level/stats/equipment enhancement/fragment inputs and
  intentionally excludes current HP, eligible condition magnitude and active consumables.
- Live battle starts with an empty player scenario group. Each later deployment rechecks the typed weapon/condition
  reason, zone affinity and Core tactical destination before creating the employee draft.
- Encounter rewards are resolved against node type/variant, validated by `CombatRewardPolicy`, then quantity effects
  are scaled once with fixed rounding. Singular equipment/pool/fragment identity grants remain singular.
- `BattleStartPlan::from_preview`, reward resolution and scenario assembly each resolve the same encounter in
  separate functions. `world/combat.rs` manually joins their results into `ActiveBattleSession`. Current behavior is
  coherent, but ownership and failure characterization are spread across four modules.

## Plan, Commit And Failure Semantics

`plan_explicit_combat_node_with_state` performs preview resolution, reward validation/scaling, battle construction
and `start_battle_execution()` before returning `PlannedLiveBattleStart`. Normal map confirmation is inside the full
non-battle state transaction. Commit then certifies current armed roster power, captures the entry snapshot, consumes
an attempt, records run peak, stores the preview, allocates the battle UUID, installs `ActiveBattleSession`,
transitions to `InBattle` and performs the initial zero-delta advance.

- All setup/static-data failures occur before normal map-node commit and the outer transaction protects the later
  transition/initial-advance tail.
- Event choice combat is planned before its session commit. The remaining paired session/start atomicity seam is
  already Phase 4 C03 ownership and is not duplicated here.
- No additional current partial-start defect was proven for valid published data. Moving active lifecycle or first
  tick semantics into Phase 5 would cross the Phase 6 boundary.

## Determinism And Retry

- Preview seed is full node UUID plus the fixed namespace; same request is deterministic.
- Template variant, generated wave role picks, rumor warning, corroded appearance seed and enemy owned UUID use
  explicit deterministic sources.
- Retry reuses the exact in-memory preview and battle-entry snapshot; Monolith conversion explicitly removes the
  old preview and attempts before regenerating.
- Durable serialized retry/cache is the one confirmed divergence because internal scale is intentionally omitted.

## Canonical And Live-Data Alignment

- Current live encounters, templates, routes, profiles, rewards and run policy pass official RON/reference and
  representative full DefenseRoute setup/playability tests.
- Current canonical requires manifested difficulty to own stat/wave/reward tuning, preview waves to be fixed before
  combat, `battle_setup_snapshot` to own scene construction, armed membership parity and serialized preview never to
  become setup authority. Runtime agrees except for the durable cache gap.
- FacilityEntity remains a future/wire-reserved variant rejected by live validation. Its unreachable preview/setup
  branches are not a current FacilityEntity implementation defect.

## Consumer Classification

- Confirmed authoritative consumers: Core node selection, world battle-start planning, active-run checkpoint
  restore, scenario validation and enemy draft creation.
- Confirmed derived consumers: Core node preview, battle setup snapshot and threat/briefing projections.
- Supporting server behavior: transports Core active-run bytes and Core DTOs; no server-owned setup policy was
  found or claimed.
- Unity and unknown external Rust callers are unverified/out-of-scope. Public helper or wire-field removal therefore
  requires compatibility inventory even when a workspace call search finds none.

## Boundary Verdict

The live happy path is deterministic and strongly tested, but one high-confidence durable-cache correctness defect
and one publication-time setup-validation gap remain. Three consolidation/cleanup targets are separately executable.
Phase 5 has enough evidence to close without changing production files.
