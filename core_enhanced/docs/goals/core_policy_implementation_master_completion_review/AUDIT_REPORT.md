# Audit Report

Status: complete.

This report audits `docs/goals/core_policy_implementation_master` using `docs/goal_completion_review_guide.md`.

## Executive Summary

Reviewed the master goal and all 8 completed subgoals against runtime code, live RON/data, server-facing DTO/transport code, policy docs, and recorded validation.

Most completed policies have credible implementation evidence. The strongest implementation areas are grant/reward execution, employee growth/loadout policies, buff/status timing, movement runtime policy, skill targeting live RON cleanup, shared server command DTOs, and validation harness migration.

Two material findings remain:

1. `MapViewDto` removed `available_node_ids` / `completed_node_ids`, but `RunSnapshotDto.map_progression` still exposes the same duplicate lists through `MapProgressionSnapshotDto`. This keeps a Unity-facing duplicate progression projection beside `MapNodeDto.state`.
2. `Timeline naming` is marked covered in the master sequence, but the confirmed policy says the type/JSON name should be renamed long-term. Current code explicitly keeps `Timeline` as serialized/client-facing name for compatibility and only documents its append-only event-log meaning.

One documentation drift finding remains:

- `core_policy_validation_test_harness/POLICY_COVERAGE.md` still says final broad validation needs to run, while `EXPERIMENTS.md` records that it already ran.

No code fixes were applied in this audit goal.

## Verdict Summary

| Subgoal | Item | Verdict | Long-Term Fit | Improvement Class | Evidence Status |
| --- | --- | --- | --- | --- | --- |
| `core_policy_static_data_map_scenario` | `RUN_SYSTEM_POLICY data source` | Complete | High | none | `RunPolicyData` owns integrated policy in `src/game/data/run_policy_data.rs`; live data is `../game_resources/data/run/policy.ron`; runtime uses `GameCore::run_policy()`. |
| `core_policy_static_data_map_scenario` | `battle_records export` | Complete | High | none | `RunState::record_battle` writes debug records and `GameCore::battle_records()` is read-only debug/test access; no live loader/server gameplay read path was found. |
| `core_policy_static_data_map_scenario` | `map authoring/generation policy source` | Complete | High | none | `map/generation_policy.ron`, `MapGenerationPolicyData`, and generator tests replace hard-coded generation policy. |
| `core_policy_static_data_map_scenario` | `NodeSessionKind` | Complete | High | none | Search found no `NodeSessionKind` runtime type; sessions use `MapNodeCategory`/payload. |
| `core_policy_static_data_map_scenario` | `MapViewDto progression projection` | Partially complete | Medium | immediate correction recommended | `MapViewDto` lists are gone and server fixtures derive from `MapNodeDto.state`, but `MapProgressionSnapshotDto` still serializes `available_node_ids` and `completed_node_ids`. |
| `core_policy_static_data_map_scenario` | `map node tags` | Complete | High | none | `MapNodeDefinition` is `deny_unknown_fields` and has no `tags`; live node definition tags are gone. |
| `core_policy_static_data_map_scenario` | encounter-less combat preview / empty wave fallback | Complete | High | none | Live PVE encounters use authored waves/routes; preview validation rejects missing route/empty fallback paths. |
| `core_policy_static_data_map_scenario` | battlefield archetype random source / `SplitRoom` | Complete | High | none | `battlefield_archetypes.ron` owns `enabled`/`weight`; validation rejects enabled `SplitRoom`. |
| `core_policy_static_data_map_scenario` | `FacilityEntity` future schema | Complete | Medium | none | Schema remains for future content, but `GameDataBase` validation and RON tests reject live `FacilityEntity` waves. |
| `core_policy_static_data_map_scenario` | authored defense route required | Complete | High | none | Live templates and encounters declare `defense_main`; preview/tests cover authored route requirements. |
| `core_policy_static_data_map_scenario` | starter employee loadout source | Complete | High | none | Live starter candidates declare `starter_loadout`; production starter fragment helper is `#[cfg(test)]`. |
| `core_policy_static_data_map_scenario` | remove legacy random event RON | Complete | High | none | Searches show only historical docs mention removed legacy random-event files. |
| `core_policy_static_data_map_scenario` | server startup live preview validation | Complete | High | none | Subgoal hooked generated preview validation into game server live data loading and `cargo check -p game_server` passed. |
| `core_policy_grant_economy_rewards` | canonical grant executor | Complete | High | none | `GrantExecutor` owns reward/effect grants; acquisition scans leave primitive APIs/tests and `GrantExecutor`. |
| `core_policy_grant_economy_rewards` | atomic reward claim | Complete | High | none | `GameCore::claim_reward` applies rewards to cloned state and commits after all effects succeed. |
| `core_policy_grant_economy_rewards` | reward fragment/research/XP diffs | Complete | High | none | `GrantExecutionResult`, `RewardGranted`, and `CombatRewardsGranted` carry fragment, research, and employee XP diffs. |
| `core_policy_grant_economy_rewards` | reward experience target policy | Complete | High | none | `GrantExperience` requires `ExperienceTargetPolicy`; live rewards declare explicit target. |
| `core_policy_grant_economy_rewards` | Enkephalin overflow policy | Complete | High | none | `Enkephalin::checked_add` is used by grant/start/headquarters/shop paths. |
| `core_policy_grant_economy_rewards` | remove reward tags | Complete | High | none | `RewardOption` has no generic tags; policy kind is derived from typed effects. |
| `core_policy_grant_economy_rewards` | explicit equipment reward pools | Complete | High | none | `GrantEquipmentFromPool` and `equipment_pools` are present in live reward data and validation. |
| `core_policy_grant_economy_rewards` | forbidden abnormality reward validation | Complete | High | none | `ForbiddenAbnormalityGrant` is absent from runtime/live data; invalid reward materialization is schema/validation failure. |
| `core_policy_grant_economy_rewards` | remove duplicate fragment auto-conversion | Complete | High | none | `ConvertAdditionalCopiesToResource` search found no runtime/live data references. |
| `core_policy_grant_economy_rewards` | fragment dust resource source | Complete | High | none | Fragment dust is granted through `RewardEffect::GrantFragmentDust` / skill-fragment resource state, not equipment material. |
| `core_policy_employee_item_growth` | remove unused growth ids | Complete | High | none | `PveWinStack` and `QuestRewardStack` searches found no runtime/live data references. |
| `core_policy_employee_item_growth` | trust feature surface | Not applicable | High | none | Trust policy was explicitly deferred; this master treats trust as the only allowed deferred policy family. |
| `core_policy_employee_item_growth` | growth curve and XP policy source | Complete | High | none | `RunPolicyData.growth` and live `run/policy.ron` own XP thresholds, survival XP, and battle-tier mapping. |
| `core_policy_employee_item_growth` | remove legacy employee grade | Complete | High | none | `EmployeeGrade` / `.grade` policy search found no runtime/live data references. |
| `core_policy_employee_item_growth` | trauma applies to final battle max HP | Complete | High | none | Battle/stat tests and subgoal validation cover final max HP after stat assembly. |
| `core_policy_employee_item_growth` | starter employee loadout source | Complete | High | none | Same evidence as static subgoal; live starter loadout is the source. |
| `core_policy_employee_item_growth` | skill fragment equip limit policy | Complete | High | none | `SkillFragmentEquipLimit::{OwnedCopies, GlobalExclusive}` exists in metadata and live RON; runtime equip checks owned copies/global exclusivity. |
| `core_policy_employee_item_growth` | skill fragment dismantle auto unequip | Complete | High | none | Dismantle tests cover deterministic loadout repair after copy reduction. |
| `core_policy_employee_item_growth` | bound equipment interaction lock | Complete | High | none | Runtime equip/unequip/dismantle/enhance/sell/combination paths reject bound equipment while DTO exposes warning state. |
| `core_policy_buff_status_stats` | hard CC action restriction source | Complete | High | none | Hard CC gates call active buff expiry helpers; search found no hard-CC duration copy into action locks. |
| `core_policy_buff_status_stats` | hard CC replacement event | Complete | High | none | `BuffExpired.reason` includes `Replaced`; replacement tests assert explicit expiry before new application. |
| `core_policy_buff_status_stats` | clear active buffs on death | Complete | High | none | Death cleanup emits `TargetDied`/`CasterDied` reasons. |
| `core_policy_buff_status_stats` | non-periodic control max stacks | Complete | High | none | `BuffDatabase` validates `Stun`, `Freeze`, `Silence` `max_stacks == 1`; live RON matches. |
| `core_policy_buff_status_stats` | move speed stat source / timing | Complete | High | none | `MoveSpeedUnitsPerMs` sets spawned body speed and runtime modifier changes affect the next movement tick. |
| `core_policy_movement_battle_runtime` | `allowed_actions` source | Complete | High | none | `allowed_actions` remains UI affordance projection; command handlers keep validation gates. |
| `core_policy_movement_battle_runtime` | action validation gates | Complete | High | none | Subgoal verified phase/action gate plus handler-specific validation; no broad bypass found. |
| `core_policy_movement_battle_runtime` | `RequestDeploymentRangePreview` gate | Complete | High | none | Preview remains an allowed action in battle and read-only result transport suppresses full state snapshot. |
| `core_policy_movement_battle_runtime` | forced movement clears engagement | Complete | Medium | follow-up refactor | Current movement/block state is recomputed from positions; `Battlefield` still has a single tile occupant compatibility projection. |
| `core_policy_movement_battle_runtime` | `MovementStopped(reason)` | Complete | High | none | Timeline event carries reason; movement engine emits `NoGoal` and `StaticObstacleBlocked`; tests cover stop events. |
| `core_policy_movement_battle_runtime` | movement tick policy source | Complete | High | none | `RunPolicyData.battle_runtime.movement_tick_ms` drives continuous movement scheduling. |
| `core_policy_movement_battle_runtime` | opponent spawn occupancy policy | Complete | High | none | Nearest-open fallback removed; overlap placement preserves authored spawn position. |
| `core_policy_movement_battle_runtime` | `UnitWithdrawn` timeline event | Complete | High | none | `TimelineEvent::UnitWithdrawn` exists and withdraw records it. |
| `core_policy_movement_battle_runtime` | `BehaviorResult` boundary | Complete | High | none | Final domain result contract migration is implemented in `core_policy_unity_server_contract`. |
| `core_policy_movement_battle_runtime` | battle runtime numeric policy | Complete | High | none | `movement_tick_ms` and `max_battle_time_ms` are live run policy data. |
| `core_policy_movement_battle_runtime` | `Timeline naming` | Document-only complete | Low | immediate correction recommended | Policy says rename type/JSON long-term; code keeps serialized/client-facing `Timeline` for compatibility and only documents event-log meaning. |
| `core_policy_movement_battle_runtime` | same timestamp attack resolve batch | Complete | High | none | Focused test proves same-time `AttackResolve` is recorded before same-time `BattleEnd`. |
| `core_policy_skill_targeting_projectile` | explicit `cast_targeting` | Complete | High | none | Live skill RON declares explicit cast targeting; raw loader requires `cast_targeting`. |
| `core_policy_skill_targeting_projectile` | remove `range_units` from skill targeting | Complete | Medium | follow-up refactor | Live skill RON and catalog DTO are clean; internal `SkillStepDef.range_units` and non-skill/basic/equipment range fields remain by scoped policy. |
| `core_policy_skill_targeting_projectile` | `TileArea.affected_tiles` clipping | Complete | High | none | Runtime area and preview both filter through `Battlefield::in_bounds`. |
| `core_policy_skill_targeting_projectile` | long line and piercing representation | Complete | High | none | Live skills use tile range presets/patterns and valid-tile clipping; no projectile range hack found. |
| `core_policy_skill_targeting_projectile` | runtime range preview source | Complete | High | none | Skill catalog points to `range_previews.final_cells`; preview command calculates live valid cells. |
| `core_policy_skill_targeting_projectile` | projectile launch owner snapshot | Complete | High | none | Existing projectile tests and source snapshots preserve launched damage context after attacker death. |
| `core_policy_skill_targeting_projectile` | projectile miss event source | Complete | High | none | `ProjectileMiss` is absent; miss records `BasicAttackProjectileImpacted { hit: false }`. |
| `core_policy_skill_targeting_projectile` | remove `SplashClusterFirst` | Complete | High | none | Search found no code/data references. |
| `core_policy_unity_server_contract` | shared gameplay command DTO | Complete | High | none | Server `PlayerGameClientMessage::Command` uses core `PlayerBehavior`; no `PlayerBehaviorRequest` remains. |
| `core_policy_unity_server_contract` | WebSocket envelope separation | Complete | High | none | `battle_response` is transport metadata and is validated separately from core `behavior`. |
| `core_policy_unity_server_contract` | `BehaviorResult` boundary | Complete | High | none | Server consumes `BehaviorCommandResultContract` from core instead of mapping all meaning ad hoc. |
| `core_policy_unity_server_contract` | typed run snapshot DTO | Complete | Medium | follow-up refactor | `RunSnapshotDto` and nested DTOs exist; public JSON wrappers still use `serde_json::Value` for server/admin serialization. |
| `core_policy_unity_server_contract` | combat result timeline attachment ownership | Complete | High | none | Core exposes `CombatResultTimelineAttachmentDto`; server owns gzip/base64 `compressed_timeline`. |
| `core_policy_unity_server_contract` | remove dormant `BattleResync.setup` | Complete | High | none | `BattleResync` carries `battle_uuid` and `update`; no `setup` field remains. |
| `core_policy_unity_server_contract` | `battle_records` export | Complete | High | none | Same evidence as static subgoal; always-on debug artifact remains. |
| `core_policy_validation_test_harness` | remove old timeline/log version 6/7 acceptance | Complete | High | none | Validator rejects any `timeline.version != TIMELINE_VERSION`; focused test covers versions 6/7. |
| `core_policy_validation_test_harness` | live content audit manifest | Complete | High | none | `tests/live_skill_catalog_audit.rs` loads `docs/audit/live_skill_catalog_manifest.ron`; hard-coded roster arrays are gone. |
| `core_policy_validation_test_harness` | final policy coverage cross-check | Partially complete | Medium | follow-up refactor | `POLICY_COVERAGE.md` exists, but it contains stale "Final broad validation still needs to run" wording after broad validation was recorded as passed. |

## Detailed Findings

### Map Progression DTO Still Exposes Duplicate Id Lists

Subgoal: `core_policy_static_data_map_scenario`

Master item: `MapViewDto progression projection`

Policy source: `docs/goals/core_component_refactor_master/POLICY_DECISIONS.md` says `MapViewDto.available_node_ids` / `completed_node_ids` should be removed and `MapNodeDto.state` should be the canonical Unity/server-facing source.

Expected behavior: Unity/server-facing map availability/completion should be read from per-node state. Internal `MapProgression` can keep lists for runtime calculation, but external projection should not expose a competing public source.

Runtime/contract evidence:

- `src/game/map/types.rs` defines `MapNode.state` and `MapNodeDefinition` without tags.
- `src/game/behavior.rs` still defines `MapProgressionSnapshotDto { current_node_id, available_node_ids, completed_node_ids }`.
- `src/game/world/snapshot.rs` copies `progression.available_node_ids` and `progression.completed_node_ids` into `RunSnapshotDto.map_progression`.
- `../game_server/src/game/player_game_actor/handlers.rs` no longer reads `map.available_node_ids`; it derives available ids from `map.nodes.iter().filter(|node| node.state == MapNodeState::Available)`.
- `src/game/world/tests/snapshots_and_start.rs` asserts `snapshot["map"].get("available_node_ids").is_none()`, but there is no equivalent assertion that `snapshot["map_progression"]` no longer exposes duplicate lists.

Legacy/fallback audit:

- No `MapViewDto.available_node_ids` field remains.
- The duplicate shape survives as a sibling snapshot DTO rather than as a `MapViewDto` field.

Long-term direction review:

- Fit: Medium.
- Improvement class: immediate correction recommended.
- Reason: the main DTO was cleaned, but a Unity-facing snapshot still exposes the same derived lists. This weakens the Source of Truth result and can let clients drift back to list-based progression.

Verdict: Partially complete.

Remaining risk:

- Removing or renaming `MapProgressionSnapshotDto.available_node_ids` / `completed_node_ids` is a Unity-facing DTO change. The policy already points toward removal, but implementation should be handled as a focused contract cleanup.

### Timeline Naming Was Not Implemented

Subgoal: `core_policy_movement_battle_runtime`, with deferred ownership expected in `core_policy_unity_server_contract`

Master item: `Timeline naming`

Policy source: `docs/goals/core_component_refactor_master/POLICY_DECISIONS.md` says `Timeline` type/JSON name should be renamed long-term to match battle event log meaning.

Expected behavior: Rust type and Unity-facing JSON contract should move toward an event-log name, or the implementation goal should explicitly produce a policy-decision/defer report.

Runtime/contract evidence:

- `src/game/battle/timeline.rs` still exposes `pub struct Timeline`.
- The code comment explicitly says "`Timeline` remains the serialized/client-facing name for compatibility with the current Unity contract."
- `docs/refactor_preparation_plan.md` has since softened the current baseline by saying the `Timeline` name can remain while responsibility is event-log oriented.
- `core_policy_validation_test_harness/POLICY_COVERAGE.md` states timeline naming remained current append-only event log wording in code/docs.

Legacy/fallback audit:

- This is not a hidden fallback; it is an explicit compatibility retention.
- It conflicts with the implementation master rule that confirmed policies should not be preserved through compatibility unless recorded and confirmed.

Long-term direction review:

- Fit: Low.
- Improvement class: immediate correction recommended.
- Reason: semantics were clarified, but the confirmed rename policy was not implemented. If the later refactor-preparation baseline supersedes the rename, the policy docs should be reconciled; otherwise a dedicated contract rename goal is needed.

Verdict: Document-only complete.

Remaining risk:

- A real rename is a Unity-facing contract change. This should become either a dedicated contract migration goal or an explicit policy update that replaces the older rename decision with "keep serialized `Timeline`, document event-log semantics."

### Validation Coverage Document Drift

Subgoal: `core_policy_validation_test_harness`

Master item: final cross-check and broad validation.

Expected behavior: final coverage artifact should reflect the actual validation state after the subgoal completed.

Evidence:

- `docs/goals/core_policy_validation_test_harness/EXPERIMENTS.md` records final broad validation as passed.
- `docs/goals/core_policy_validation_test_harness/POLICY_COVERAGE.md` still says "Final broad validation still needs to run after this checklist."

Long-term direction review:

- Fit: Medium.
- Improvement class: follow-up refactor.
- Reason: runtime implementation is not contradicted, but the final cross-check artifact contains stale status text.

Verdict: Partially complete.

Remaining risk:

- Future auditors may believe broad validation was not run unless they cross-read `EXPERIMENTS.md`.

## Cross-Component Findings

1. Map progression SoT is split across `MapNodeDto.state` and `RunSnapshotDto.map_progression.available_node_ids` / `completed_node_ids`. The server fixtures already use node state, but snapshot consumers can still read duplicate lists.
2. `Timeline naming` policy and current baseline docs disagree. The policy decision says rename; `refactor_preparation_plan.md` and runtime comments say the serialized/client-facing name may remain while semantics are event-log oriented.
3. Skill targeting cleanup intentionally scoped live skill RON/catalog surfaces, but internal `SkillStepDef.range_units` and `SkillCastTargetingDef::FirstStepTarget` remain for non-RON/test constructors. This is acceptable for current policy scope but should not be mistaken for full internal model cleanup.
4. Typed snapshot migration is meaningful, but JSON wrapper methods and server transport still use `serde_json::Value` at serialization boundaries. This is an acceptable transport/wrapper boundary, not a new gameplay source of truth.
5. `Battlefield` still has a single tile `occupant` compatibility projection while actual unit positions use `unit_pos`. Current overlap policy is implemented through `unit_pos`; future movement/blocking work should avoid reading tile occupant as authoritative unit position.

## Immediate Correction Candidates

1. Remove or debug-gate `MapProgressionSnapshotDto.available_node_ids` and `completed_node_ids`, and add a snapshot test proving external map progression no longer exposes duplicate availability/completion lists.
2. Resolve `Timeline naming` by either:
   - implementing the event-log type/JSON rename through a dedicated Unity/server contract migration, or
   - explicitly updating the confirmed policy to keep serialized `Timeline` and treat event-log semantics as the long-term contract.

## Follow-Up Refactor Candidates

1. Remove internal `SkillStepDef.range_units` and `SkillCastTargetingDef::FirstStepTarget` once all non-RON/test constructors can express skill targeting through explicit tile range policy.
2. Rename or wrap `Battlefield::in_bounds` so valid-tile clipping intent is not hidden behind an ordinary bounds name.
3. Clarify or remove the `Battlefield` tile occupant compatibility projection if future runtime code starts relying on it instead of `unit_pos`.
4. Update `docs/goals/core_policy_validation_test_harness/POLICY_COVERAGE.md` stale final-validation wording.
5. Consider reducing snapshot JSON wrapper surface after server/admin callers can consume typed DTOs directly.

## Validation Commands

Recorded by implementation subgoals:

- `cargo test --lib -- --test-threads=1` passed at the master final cross-check, 515 tests.
- `cargo test --test ron_loading -- --test-threads=1` passed at the master final cross-check, 18 tests.
- `cargo check -p game_server` passed at the master final cross-check.
- `cargo test --test live_skill_catalog_audit -- --test-threads=1` passed after manifest migration.
- `cargo test --lib game::battle::validation::validator::tests::normal_validation_rejects_legacy_timeline_versions -- --test-threads=1` passed after legacy timeline version validation.

Audit rerun:

- `cargo check --lib` - passed. Existing warning: `/mnt/f/work/simulator/auth_server/Cargo.toml: unused manifest key: env`.
- `cargo test --test live_skill_catalog_audit -- --test-threads=1` - passed, 3 tests.
- `cargo test --test ron_loading -- --test-threads=1` - passed, 18 tests.
- `cargo test --lib game::battle::validation::validator::tests::normal_validation_rejects_legacy_timeline_versions -- --test-threads=1` - passed, 1 test.
- `cargo check -p game_server` - passed. Existing warning: `/mnt/f/work/simulator/auth_server/Cargo.toml: unused manifest key: env`.
- `cargo test --lib -- --test-threads=1` - passed, 519 tests.
- `git diff --check` - passed.

## Policy Questions

사용자와 정책 논의 필요:

1. Does the confirmed `MapNodeDto.state` source-of-truth policy also require removing `RunSnapshotDto.map_progression.available_node_ids` / `completed_node_ids` from the external snapshot contract now?
   - Recommended long-term choice: yes, remove them or move them behind explicit debug-only naming if the client needs temporary diagnostics.
2. Should the older `Timeline naming` policy still require a Rust/JSON rename, or does the newer `refactor_preparation_plan.md` baseline supersede it by keeping serialized `Timeline` and documenting event-log semantics?
   - Recommended long-term choice: if Unity can accept a contract migration, rename to an event-log term; otherwise explicitly update the policy decision to keep `Timeline` as the serialized compatibility name.
