# Current Handoff

This document is for the next AI or developer continuing the current game-flow refactor.

## One-Sentence Summary

The project is moving from phase-based progression and abnormality-owned combat units toward a Slay the Spire-like seeded node map where persistent employees explore a sealed facility, grow through combat, suffer trauma/injuries, and can permanently die.

## Current Game Flow

```text
StartNewGame
-> present starter employee candidates
-> select 3 starter employees
-> generate seeded act map
-> choose available node
-> preview selected node and confirm entry
-> enter node session/content
-> resolve node
-> complete node
-> unlock outgoing nodes
-> boss completes act
-> next act map or RunComplete
```

The player cannot backtrack. Only outgoing nodes from the completed node become available.

## Implemented Core Pieces

- Seeded run map generation.
- RON-backed map node definitions: `game_resources/data/map/node_definitions.ron`.
- Map combat encounter assignment now respects node intent: normal combat nodes prefer act/depth-appropriate non-boss encounters, elite combat nodes prefer higher difficulty non-boss encounters, and boss nodes prefer `CombatNodeType::Boss`.
- `CombatMissionPolicy` is the source of truth from map-node intent to combat mission purpose. Map encounter assignment uses it to prioritize `PveEncounter.node_type`, and `CombatPreview` uses it for fallback archetype-to-node-type inference when no authored encounter exists.
- Node categories:
  - `Combat`
  - `Boss`
  - `Event`
  - `Support`
  - `Shop`
  - `Reward`
- `NodeSession` for active map node content.
- `NodeConfirm` entry flow: `SelectMapNode` creates `NodePreview`, `ConfirmEnterNode` consumes/enters the node, and `CancelSelectedNode` returns to the map without consuming the node.
- Combat/Boss node routing into node combat.
- Combat/Boss preview policy now assumes basic tactical briefing before entry, with future limited precision scan resources for deeper enemy/terrain information.
- Shop/Reward/Event/Support minimum routing.
- Act progression and final `RunComplete`.
- Employee roster, starter employee candidates, and selected starter employees.
- Employee-based field/bench validation.
- Employee equipment loadout.
- Equipment materials, restoration recipes, dismantle recipes, and instance-level equipment enhancement recipes.
- `EnhanceEquipment` is Maintenance-gated, consumes equipment material stacks, updates `OwnedEquipment.enhancement_level`, and exposes the changed item/material stacks through inventory diffs.
- Battle entry copies equipped item enhancement levels into `BattleScenario` unit drafts so combat stats are calculated from immutable battle-start snapshots.
- Battle participant summaries.
- Post-battle trauma, injury, XP, and death resolution.
- Disabled-by-default employee trust foundation with employee-owned trust state, run-owned trust policy, snapshot exposure, and no-op hooks for medical recovery and post-battle trauma.
- Run failure checks cover no living employees, no deployable employees, and combat team unavailable. Combat entry requires explicit node-scoped deployment before battle start.
- Aggregate run snapshot via `GameCore::get_run_snapshot_json()`.

## Important Code Entry Points

```text
src/game/world.rs
- GameCore construction, execute dispatch, major flow handlers.

src/game/world/helpers.rs
- internal helpers: validation, state transitions, starter employees, reward/session helpers.

src/game/world/snapshot.rs
- JSON snapshots for run, roster, field, bench, inventory, selected event.

src/game/world/support.rs
- support node modes and minimum support effects.

src/game/world/tests.rs
- grouped GameCore tests.

src/game/map/
- map types, generation, progression, node executor, node session.

src/game/employee.rs
- employee roster and persistent employee state.

src/game/events/combat.rs
- node combat entry adapter that accepts `CombatPreview`, explicit deployment, roster, inventory, and skill fragment state.

src/game/battle/core/
- battle runtime. This is ECS-independent.

src/game/resources/
- domain state/resources split by responsibility:
- `state.rs`: `GameState`
- `board.rs`: `Position`, `Field`, `Bench`
- `economy.rs`: `Enkephalin`
- `progression.rs`: `Qliphoth`
- `selection.rs`: selected node content and reward/shop/support/combat sessions
- `action.rs`: `ActionValidator`
- `inventory.rs`, `item_slot.rs`: inventory DTO/storage and equipment slots
```

## Current Module Shape

`GameCore` stores runtime state in explicit `GameCoreState`; `bevy_ecs::World` has been removed from the core game flow. The recent split moved non-flow code out of `world.rs`:

```text
world.rs       -> main flow
helpers.rs     -> shared helpers
snapshot.rs    -> snapshot serialization
support.rs     -> support node logic
tests.rs       -> grouped tests
```

## ECS Removal Status

`bevy_ecs` has been removed from `Cargo.toml`, `src/ecs` has been deleted, and former resource types now live under `game::resources`.

Important distinction:

```text
GameCoreState
-> CombatExecutor
-> BattleScenario
-> BattleCore
```

`BattleCore::new(...)`, `PlayerDeckInfo`, and `BattleScenario::from_decks(...)` have been removed. Battle startup now enters through `BattleCore::new_from_scenario(...)`.
The skill-test common harness now builds `BattleScenario` directly, so skill tests cover the same scenario spawn path as node combat.
`skill_refactor_validation` also uses direct `BattleScenario` fixtures. Tests that mutate runtime units use `run_battle_with_post_spawn_setup(...)` so setup runs after scenario spawn events, not against pre-spawn empty state.
`battle_replay_validation` also uses direct `BattleScenario` fixtures and `TimelineExpectedCounts::from_scenario(...)`, so replay/validation coverage is no longer tied to legacy authoring.
`battle_ranged_attack` also builds direct `BattleScenario` fixtures and runs through `BattleCore::new_from_scenario(...)`, so ranged/projectile/windup behavior is no longer covered through legacy helpers.
`live_item_skill_activation` and `movement_timeline_exports` also use direct `BattleScenario` fixtures, including scenario-level artifacts and unit equipment loadouts where needed.
`battle_rapier_movement` and `unity_timeline_contract_exports` also use direct `BattleScenario` fixtures. Static obstacles in Rapier movement tests are authored through `BattleFieldSpec.obstacles`, not injected into the battlefield during setup.

## Current Limitations

- Employees now carry an employee-native combat profile with base stats, basic attack data, and growth stacks.
- Skill fragments have authored metadata in `SkillFragmentDatabase`, including origin, suppression/containment/rare reward sources, and future dependency concepts.
- Game-system direction: skill fragments are currently active-skill providers, not passive always-on modifiers. Employee active skill fragment equip is intended to be limited to one meaningful slot; unused fragments should feed decomposition/refinement, enhancement, research, or limited recombination loops.
- Live skill fragment RON data lives in `game_resources/data/skill_fragments/base.ron`.
- Active fragments grant employee imitation skills through `imitation_skill_id`, may define upgrade variants through `upgrade_skill_ids`, and may define an `awakened_skill_id`; the source abnormality keeps its own original `skill_id`.
- The run owns a `SkillFragmentInventory`; employees own separate `SkillFragmentLoadout`s.
- `SkillFragmentLoadout` now has baseline fragment ids plus one active fragment slot. A second active fragment is rejected at loadout mutation time.
- `SkillFragmentPolicy::default_run_policy()` groups explicit sub-policies for equip, stacking, composition, and dismantle rules. Timing is handled by action gates.
- `SkillFragmentInventory` now stores per-fragment counts and progress. Fragment rewards stack and same-rarity material composition consumes non-equipped, non-target, non-last-copy materials.
- `PlayerBehavior::UpgradeSkillFragment { target_fragment_id, material_fragment_id }` and `AwakenSkillFragment { target_fragment_id, material_fragment_ids }` update fragment progress/awakening state, but are gated behind `Maintenance` support nodes.
- `PlayerBehavior::DismantleSkillFragment { fragment_id }` is also Maintenance-gated. It converts extra, non-equipped fragment copies into `fragment_dust`; equipped fragments and last copies are protected.
- New runs now open `SelectingStarterEmployees` before map generation. `StartNewGame` returns starter candidates from `game_resources/data/employees/starter_candidates.ron`, `SelectStarterEmployees` must pick exactly 3 unique candidate ids, then the selected employees are created, benched, granted the starter basic-attack enhancement fragment, and the Act map/resources are initialized.
- Fragment equip/unequip player actions exist and update employee loadouts.
- `RewardEffect::GrantSkillFragment` exists and live reward data can grant early abnormality-derived fragments. Current A-rank live fragment rewards include `fragment_one_sin_penitence`, `fragment_scorched_spark`, `fragment_red_shoes_impulse`, `fragment_freischutz_black_round`, `fragment_spider_bud_red_eyes`, `fragment_funeral_butterfly_eulogy`, `fragment_helper_grinder_trace`, and `fragment_alriune_faint_aroma`.
- A-rank abnormality skill implementation has started from `docs/skills/lobotomy_a_rank_skill_goal_plan.md`. The first live content slice now covers 8 original/fragment pairs: One Sin, Scorched Girl, Spider Bud, Red Shoes, Der Freischutz, Funeral of the Dead Butterflies, All-Around Helper, and Alriune. `tests/live_skill_catalog_audit.rs` verifies original skill ids, fragment imitation skill ids, and original/fragment `SkillDef` independence for this slice.
- `RewardMetadata` now carries reward-purpose tags (`Currency`, `Experience`, `Equipment`, `Artifact`, `SkillFragment`, `ResearchProgress`, `Forbidden`, `Narrative`). Tags are explicit in live reward RON and inferred from effects for old/test fixtures. `RewardOption` and reward snapshots expose these tags so future node-type reward policy can route Defense/Recovery/Suppression outcomes without parsing effect lists.
- Live PVE encounter reward pools must not reference `Forbidden` rewards. Old "abnormality as player reward" placeholders remain representable for negative tests/legacy data, but `GameDataBase` validation rejects them from authored combat rewards.
- `CombatRewardPolicy` is the semantic bridge between `CombatNodeType` and `RewardTag`. It does not grant anything directly; it validates that a non-empty combat reward pool contains at least one featured tag for that mission purpose and rejects tags that are not allowed for combat rewards.
- `RewardEffect::GrantSkillFragmentResearch { fragment_id, amount }` now exists. It increases `SkillFragmentProgress.research_progress` without granting an owned fragment copy and is inferred/tagged as `ResearchProgress`. `research_progress` is intentionally separate from `awakening_progress`.
- Research completion now follows the agreed HQ-delivery policy. `SkillFragmentPolicy.research.completion_threshold` defaults to `100`; each threshold completion increments `SkillFragmentProgress.research_completion_count` and queues a `pending research delivery` instead of immediately granting a fragment. `ResearchDeliveryPolicy` owns the safe-node allowlist, defaulting to `Support`, `Shop`, `Reward`, and `Event`; `Combat` and `Boss` keep deliveries pending. `ConfirmEnterNode` automatically delivers pending research only when the entered node category is allowed. Inventory snapshots expose both all `skill_fragment_progress` entries and `pending_research_deliveries`, so unowned-but-researched fragments are visible to clients. Safe node entry results also carry `research_deliveries` directly on `NodeEntered`, `SupportState`, `ShopState`, and `RewardState`; Event nodes now resolve directly into Shop/Reward sessions instead of returning a phase-era `RandomEventState`.
- `EquipmentDatabase` now has `EquipmentMaterialMetadata` entries for stackable equipment materials such as fragments, residues, cores, and blueprints. `Inventory` stores these as `equipment_materials`, separate from owned/equipped equipment instances.
- `RewardEffect::GrantEquipmentMaterial { material_id, amount }` now exists. It increases an equipment material stack without consuming equipment inventory slots and is inferred/tagged as `Equipment`. Live data includes `damaged_weapon_fragment_reward` as the first material reward.
- Live PVE combat reward pools no longer reference generic `ego_gift_reward` or `double_ego_gift_reward`. They now use material/research rewards such as `field_equipment_salvage_reward`, `damaged_armor_fragment_reward`, `scorched_ego_residue_reward`, `early_abnormality_research_reward`, and `high_risk_abnormality_research_reward`.
- `CombatRewardPolicy` treats `ResearchProgress` as a featured reward identity for Suppression and Encirclement nodes, matching the current rulebook direction that suppression can advance HQ analysis even when a complete skill fragment does not drop.
- `EquipmentRestorationRecipeMetadata` now exists for Maintenance equipment restoration. `PlayerBehavior::RestoreEquipment { recipe_id }` is Maintenance-gated, consumes `equipment_materials`, creates an `OwnedEquipment` instance, and returns an `EquipmentRestored` result with inventory/material diffs. Live data includes `restore_standard_armor` and `restore_fourth_match`.
- `EquipmentDismantleRecipeMetadata` now exists for Maintenance equipment dismantling. `PlayerBehavior::DismantleEquipment { item_uuid }` is Maintenance-gated, rejects equipped items, removes the owned equipment instance, and returns material stack diffs. Live data includes dismantle recipes for `standard_armor` and `fourth_match`.
- Support node policy has been revised. Source of truth: `docs/game_rulebook.md`.
- Active support effects are Medical, Rest, and Maintenance only. Intel, Communications, Supply, Containment resurrection, and support-specific Random mode have been removed from active support policy.
- Employees have persistent `Run HP`. Battle start derives temporary `Battle HP` from the current Run HP ratio. Battle end does not write surviving participant Battle HP back to the employee.
- If an employee is incapacitated in battle, only that employee receives trauma and a tunable Run HP loss. Mission failure itself does not apply broad trauma to every participant.
- Combat/Boss node selection is blocked, not consumed, when no employee is deployable while an available Medical support route remains. If no deployable employee and no available Medical route remain, the run fails.
- Node selection itself is a preview. Combat deployment pruning, no-deployable run failure, and support-route blocking happen on `ConfirmEnterNode`, not on `SelectMapNode`.
- Combat preview now has a core data contract and a `BattlefieldGenerator` foundation. `NodePreview` can include `CombatPreview`, `UseReconScan` consumes `recon_charge`, repeated scans are allowed, zone-based enemy previews are exposed, scanned node information persists after cancel, and generated battlefield instances include archetype, size class, deployment zones, spawn zones, spawn waves, obstacles, and enemy briefing.
- Stable battlefield archetype metadata now lives in shared RON at `game_resources/data/map/battlefield_archetypes.ron`.
- Battlefield shape rules now live in shared RON at `game_resources/data/map/battlefield_templates.ron`. The templates use ASCII `rows`; spaces are void/outside-battlefield tiles, non-space tiles become `valid_tiles`, `#` becomes obstacles, `P` becomes deployment cells, and spawn markers such as `N/L/Q/A/B/W/R` become `SpawnZone`s. This supports non-rectangular fields such as L-shaped corridors without hardcoded geometry.
- The template database now supports multiple templates per archetype/size class and selects among candidates by seed. Current Medium variants cover `Corridor` corner/hook layouts, `Ambush` flank/corner layouts, `Surrounded` breach/ruin layouts, `SplitRoom` bridge/offset layouts, and an additional `ObstacleRoom` ruin layout.
- `CombatNodeType::Ambush` has been removed. Ambush remains only as battlefield/spawn/scenario texture through `BattlefieldArchetype::Ambush` and `SpawnZoneKind::Ambush`; authored encounters using ambush terrain should use a real mission purpose such as `Suppression`, `Frontline`, `Encirclement`, or `Recovery`.
- Void/outside-battlefield tiles are not only rejected by tile validation; `BattleCore::build_continuous_movement_input` now projects them as `MovementStaticObstacle::void_tile`, so Rapier continuous movement receives static colliders for ASCII spaces.
- `BattlefieldGenerator::try_generate` now validates generated battlefield instances on the production path and returns `InvalidStaticData` instead of relying on debug-only assertions. `GameCore` uses `CombatPreview::try_generate_for_node` when building node previews.
- Map combat entry now passes the selected node's `CombatPreview` into battle startup. Actual `BattleCore` field size, valid tiles, enemy spawn positions, and static obstacles are derived from the same preview data.
- Run-persistent TFT-style combat placement is no longer the official map combat source. Combat/Boss `NodeConfirm` creates a node-scoped `CombatDeployment`; `MoveUnit` in that state places employees only into the selected node's `DeploymentZone`, and `ConfirmEnterNode` requires at least one explicit node deployment. Legacy phase suppression and its fixed combined-field battle plan have been removed.
- `ConfirmEnterNode` revalidates node-scoped deployment before consuming the map node: every placed employee must still be available for combat and every placement must still be inside the selected node's deployment cells. Stale deployment state fails while staying in `NodeConfirm`, instead of partially entering combat.
- `BattleScenario` is the internal battle-start input. Legacy deck-based `BattleCore::new` and `PlayerDeckInfo` have been removed, and `BattleCore` no longer stores `player_info`/`opponent_info` directly.
- `ScenarioUnitSpawn.position`, `TacticalPoint.position`, `valid_tiles`, static obstacles, deployment-zone cells, and spawn-zone cells now share the same battlefield tile coordinate system. Runtime spawn world positions are `WorldVec2::from_tile_center(position)`; the old TFT/hex `PlacementBoard` conversion has been removed from battle startup.
- `BattleScenario::validate()` now runs before battle runtime state is built. It rejects invalid battlefield dimensions, out-of-bounds coordinates, duplicate ids, obstacle/spawn overlap, unknown tactical point or unit references, empty paths, invalid group radii, and required enemy groups that are never spawned.
- `BattleEvent::SpawnGroup` is now the runtime spawn path. `AtBattleStart` and `AtTimeMs` scenario events enqueue spawn events, spawned units emit `UnitSpawned` at the actual spawn time, and required enemy groups prevent early victory while later waves are still pending.
- Conditional scenario events are intentionally not implemented yet. Treat them as `BattleScenario` phase-2 work, not as the next immediate refactor. Current supported scenario triggers are battle start and absolute `time_ms`; current focus is stabilizing battle flow and expanding real encounter data.
- `CombatExecutor` now converts `CombatPreview.spawn_waves` into `BattleScenario` enemy groups/events and starts map/node battles through `BattleCore::new_from_scenario(...)`. Opponent deck construction has been removed from the node combat path.
- `BattleScenario` now includes a `tactical_plan`. The default preserves the current `SuppressAll + FreeEngage + AssaultPlayer` behavior. Spawned runtime units store a `tactical_anchor`, and the movement planner supports a first-pass `HoldDeployment` player policy that prevents unlimited far-target chasing and returns leashed units to their anchor. `CombatExecutor` now maps `CombatPreview.archetype` into the scenario tactical plan so map combat can start using archetype-driven movement policy without reintroducing deck-specific behavior.
- Tactical plans can now carry explicit `TacticalPoint`s. `EnemyMovementPlan::PathToPoint` and `PathAlongPath` have runtime implementations: enemies move toward tactical points or the first unreached waypoint instead of chasing far players, while still attacking player units already in range. Empty or invalid enemy paths do not fall back to unlimited free engage.
- `WinCondition::DefendPoint` and the `DefensePointBreached` timeline event have been removed from live/runtime/data contracts. Future leak-runner or gate-defense missions must be introduced as a new explicit contract instead of reusing deprecated point-leak defense.
- `WinCondition::ProtectUnitForDuration` is the default black-box/device defense objective. The protected unit is spawned by the scenario as `BattleUnitRole::DefenseObject`, cannot be moved by the player, cannot move, cannot attack, and losing it ends the battle in the opponent's favor. `Controlled` defense wins after the survival timer; `Unstable`/`Collapse` defense also requires required enemy cleanup.
- `TimelineEvent::UnitSpawned` now carries `role`. Clients should treat `DefenseObject` spawns as fixed objective markers, not deployable/controllable roster units.
- `CombatNodeType::Defense` now has a core default when the encounter does not author its own tactical plan or win condition: `CombatExecutor` creates a `black_box_recovery` tactical point near the primary deployment zone, injects the fixed `black_box_recovery_device` defense object, uses `HoldDeployment` for player movement, routes enemies with `PathToPoint`, and assigns `ProtectUnitForDuration(black_box_recovery_device)`. Authored tactical plans and win conditions still override these defaults.
- `WinCondition::RecoverHoldAndExtract` is the recovery-style objective. Required enemy groups must be cleared first. A live player entering `target_point_id` radius emits `RecoveryTargetSecured`; the group must hold that point for `hold_duration_ms`, then entering `extraction_point_id` radius emits `ExtractionCompleted` and wins the battle.
- `CombatNodeType::Recovery` now has a core default when the encounter does not author its own tactical plan or win condition: `CombatExecutor` creates `recovery_target` and `extraction_point`, assigns the player main group `AdvanceAlongPath([recovery_target, extraction_point])`, and uses `RecoverHoldAndExtract(target_radius: 0.75, extraction_radius: 0.75, hold_duration_ms)`. Authored tactical plans and win conditions still override these defaults.
- Live PVE data now includes `recover_black_box_archive`, the first `CombatNodeType::Recovery` encounter. It deliberately leaves `tactical_plan` and `win_condition` empty so the core default recovery contract is exercised by authored data. Its rewards are `Equipment`/`ResearchProgress` focused.
- `CombatNodeType::Frontline` now has a core default when the encounter does not author its own tactical plan or win condition: `CombatExecutor` keeps the objective as `SuppressAll`, uses `CautiousEngage(leash_radius: 5.0, chase_radius: 1.5)` for limited forward pressure, and keeps `AllRequiredEnemyGroupsDefeated` as the win condition. This is the TFT-like line-fight contract.
- `CombatNodeType::Encirclement` now has a core default when the encounter does not author its own tactical plan or win condition: `CombatExecutor` creates a `survival_anchor` near the primary deployment zone, uses `HoldDeployment` plus player-main `HoldArea`, and assigns `SurviveUntil(45_000ms)` as the win condition. This is the hold-formation survival contract.
- `CombatNodeType::SplitOperation` is intentionally deferred. Do not author live SplitOperation encounters or include it in map encounter selection until the game is stable enough to support split squads, multiple simultaneous objectives, and UI that explains divided deployment. `SplitRoom` remains a battlefield archetype and currently falls back to `Suppression` when no authored encounter exists.
- Live PVE data now gives `Frontline` encounters Corridor battlefield overrides and the `Encirclement` Blue Star encounter a Surrounded battlefield override, so node purpose and field shape are aligned while still allowing authored overrides later.
- Tactical plans now have a `TacticalGroupPlan` contract. Default plans include a `SideAll(Player)` `player_main` group for normal frontline/surround-style battles. Runtime spawns can resolve scenario spawn groups, explicit unit refs, or side-wide membership into `RuntimeUnit.tactical_group_id` and `ScenarioRuntimeState.tactical_groups`; explicit spawn/unit groups take priority over side-wide catch-all groups. `GroupObjective::AdvanceToPoint` advances the live group center toward the tactical point by a bounded cohesion step before assigning formation slots. `AdvanceAlongPath` skips reached waypoints and advances toward the first unreached waypoint without persistent path state. `HoldArea` uses the tactical point as the fixed hold center. Group objectives do not fall back to unlimited `FreeEngage`, use `engage_radius` for limited attacks, and keep approach points bounded by `cohesion_radius`. `ReconnectToGroup` moves members toward the live center of the target group and does not fallback to free engage when the target group is missing. Conditional paths, advanced formation reshaping, and group merge/split triggers are still future work.
- `EnemyBriefing` is now client-facing tactical information only. Actual wave composition is stored in `SpawnWave.enemy_entries`, so battle spawns no longer infer real enemy count/type from `EnemyBriefing.count_hint`.
- Wave enemy entries now carry `EnemyKind` through `PveWaveEnemyData.kind -> SpawnWaveEnemyEntry.kind`, so encounter authoring can distinguish `CorrodedEmployee`, `Abnormality`, and future `FacilityEntity` without changing battle startup.
- `CorrodedEmployeeProfileDatabase` exists and live data is loaded from `game_resources/data/enemies/corroded_employees.ron`. `BattleUnitSource::CorrodedEmployee` resolves combat stats from `profile_id`, while `BattleUnitSource::Abnormality` continues to resolve true abnormality bodies from `abnormality_id`.
- Live encounter data has started moving secondary/add waves away from mass abnormality swarms and into `CorrodedEmployee` profiles. Current profiles are guard, rusher, marksman, bruiser, medic, and veteran. Continue this direction for normal/elite waves; keep abnormalities as the main elite/boss/special threat.
- Early suppression rewards now include independent imitation skill fragments for Scorched Girl, Red Shoes, and Der Freischutz. These are separate `SkillDef`s, not direct reuse of the source abnormality skill.
- Corroded employee visual variety is seed-based and presentation-only. `SpawnWaveEnemyEntry.appearance_seeds` carries one seed per spawned corroded employee in that entry; non-corroded entries keep the list empty. Do not randomize combat stats from the appearance seed; same role/profile/difficulty should remain strategically learnable and reproducible.
- Corroded wave authoring now has a `PveWaveSource` split. Existing explicit `PveWaveData.enemies` remains the legacy/manual source, `PveWaveSource::Manual(Vec<PveWaveEnemyData>)` is the explicit manual form, and `PveWaveSource::GeneratedCorroded { preset_id, budget_override, seed_salt }` resolves preset-based corroded employee waves. Presets live in `game_resources/data/enemies/corroded_wave_presets.ron` as `CorrodedWavePresetDatabase` entries with difficulty/pressure/role_mix/budget/count_range. Generation resolves during `CombatPreview` creation into fixed `SpawnWave.enemy_entries`; `BattleScenario` and battle runtime never regenerate enemies.
- Live `recover_black_box_archive` now uses `GeneratedCorroded` for its archive sentry and breach response waves, so the live RON path exercises generated corroded wave authoring while keeping preview/battle input deterministic.
- Live `defend_black_box_relay` is the first authored `CombatNodeType::Defense` encounter. It uses a `ChokePoint` battlefield, leaves `tactical_plan`/`win_condition` empty to exercise the core default black-box defense contract, and spawns generated corroded employee waves from `black_box_breach_probe` and `black_box_breach_pressure`.
- `FacilityEntity` remains rejected by validation until its own profile source exists.
- `PveEncounter.waves` is the authored wave-composition contract. `BattlefieldGenerator` converts it into `CombatPreview.spawn_waves`; legacy `PveEncounter.units` and its compatibility fallback have been removed.
- PVE encounters must define at least one authored wave. Battle start no longer infers hidden enemy groups from `PveEncounter` if `CombatPreview.spawn_waves` is empty; an empty preview wave contract is invalid static data.
- `PveEncounter.node_type` is the authored combat mission purpose and is exposed as `CombatPreview.node_type`. It is intentionally separate from `BattlefieldArchetype`: archetype describes field shape, while node type describes why the battle exists.
- Live PVE RON encounters now explicitly declare `node_type`; RON loading tests reject live encounters that omit it. Default inference remains for generated/test-only encounters, not for authored live content.
- Live-supported combat node purposes are centralized in `CombatMissionPolicy::LIVE_SUPPORTED_NODE_TYPES`: `Suppression`, `Defense`, `Frontline`, `Encirclement`, `Recovery`, and `Boss`. `SplitOperation` remains an enum value for future split-squad design, but live PVE validation rejects it until split deployment, multi-objective UI, and reward framing exist.
- Battle startup now validates that the supplied `CombatPreview` was generated for the requested `encounter_id`, and that preview `node_type` matches authored `PveEncounter.node_type` when the encounter declares one. This keeps `PveEncounter -> CombatPreview -> BattleScenario` as a one-way contract instead of allowing stale or mismatched previews to start combat.
- Map combat copies `CombatPreview.node_type` into `CombatBattleState` and the selected event snapshot. Mission purpose therefore survives the transition from preview to active battle and can be used later by rewards, UI briefing, and post-battle scenario effects without re-reading encounter data.
- Node combat reward resolution now receives the active `CombatNodeType` and rejects authored encounter/node-type mismatches. Actual map combat therefore resolves rewards against the same mission purpose that was previewed and started, instead of using only `encounter_id`.
- `PveEncounter` now has scenario-authoring overrides for battlefield archetype/size/dimensions, tactical points/objective/enemy movement plan, win condition, per-wave spawn zones, and whether a wave is required for victory. These authoring fields feed `BattlefieldGenerator`, `CombatPreview`, and `CombatExecutor`; `BattleScenario` remains the runtime input, not the RON storage format.
- `PveEncounter.static_obstacles` is now part of the preview/battlefield authoring contract. Empty `static_obstacles` uses the archetype default obstacle pattern; non-empty `static_obstacles` replaces the archetype default obstacles with the authored obstacle list.
- Phase-era live flow has been removed. `WaitingPhaseRequest`, `SelectingEvent`, `RequestPhaseData`, `SelectEvent`, `AdvancePhase`, `RandomEventState`, `CurrentPhaseEvents`, and `GameProgression` are no longer part of the core runtime. Event nodes resolve directly to `InShop` or `InReward`; suppression-style random event targets are rejected until they are redesigned as real node content.
- Map content nodes now use the `shop_pool_id`, `reward_pool_id`, and `event_pool_id` declared in `node_definitions.ron`. These pools live on the corresponding Shop/Reward/RandomEvent databases, so map nodes no longer draw from the full database by accident. Live event pools must contain only Shop/Reward targets; Suppress random events may remain as legacy/authored data but are not included in live map event pools.
- Employee trust is now a core system, not an optional side feature. The default flow is narrative-first: memory/dialogue/recent reactions are on, while command refusal, combat modifiers, and trauma modifiers remain separate gated effects.
- There is a world-level smoke test from combat node preview through confirmed battle entry, automatic reward grant, and map progression.
- `map_combat_node_smoke_exports_timeline` is the world-flow timeline export. It runs `start game -> starter selection -> forced live combat node preview -> node-scoped deployment -> ConfirmEnterNode -> CombatResolved` and writes `timeline_exports/map_combat_node_smoke.json`. Use this as the representative run-based combat timeline; `unity_contract_*` remains client presentation showcase data and `skill_test/*` remains per-skill validation data.
- World combat tests now pin the agreed map-flow failure policy: failed Defense and Recovery replays consume the node, clear the active node session, return to `ViewingMap`, and apply incapacitation HP/trauma only to incapacitated employee participants. Boss loss is tested through party-wipe semantics and remains immediate `RunFailed(BossDefeated)`.
- Combat replay completion is now a map-node contract. A replay without an active `node_session` is rejected for both victory and defeat; the old non-node combat win -> extra reward path is removed.
- Map node definitions live in shared game resources at `game_resources/data/map/node_definitions.ron`.
- Newly agreed result-flow policy: combat/defense/recovery node failure is not immediate run failure. Node failure should apply heavy HP/trauma consequences and reduced/no rewards, then return to map flow if the run still has living/deployable recovery possibilities. Run failure remains based on no living employees or no deployable employees with no available Medical route.
- Newly agreed reward-flow policy: combat rewards are auto-granted for now. Optional reward selection is not part of the current core gameplay flow.
- Implemented reward-flow refactor: `InCombatReward`, `InCombatRewardClaimed`, `ClaimCombatReward`, and `ExitCombatReward` were removed. `FinishCombatReplay` now applies winning combat rewards immediately and, when the battle belongs to a map node, completes the node in the same flow.
- Non-combat Reward/Event nodes must not grant employee experience. Employee XP belongs to combat participation/post-battle progression, not safe-node rewards.
- Newly agreed Maintenance policy: Maintenance is an open work session. The player can repeat allowed Maintenance actions until they explicitly complete the node.

## Remaining Goals

The remaining work should preserve these goals:

- Combat preparation should matter more than mid-combat manual control.
- Normal waves should primarily use corroded former employees, not mass abnormality swarms.
- Abnormalities should remain rare core threats, elite/boss encounters, event subjects, and reward sources.
- Combat preview and actual battle input must not diverge.
- Maintenance should become a meaningful conversion/investment node, not a wasted support node.
- Employee trust should make employees feel remembered and reactive without frequently blocking normal combat control. Future trust work should expand event emission for treatment, rest, neglect, repeated investment, injured redeployment, and dangerous choices before adding stronger gameplay penalties.

## Recommended Next Work

1. Expand node combat data contracts.
   - Keep `CombatPreview`/`BattlefieldGenerator` as the only battle-start layout source.
   - Do not add fallback paths that reconstruct enemy waves from `PveEncounter` after preview generation; fix the preview/generator contract instead.
   - Move enemy wave composition and enemy kind selection toward data-driven contracts.
   - Keep test-only player abnormality harness isolated from normal player ownership rules.
   - Follow `docs/battle_scenario_core_refactor_plan.md` before changing `BattleCore`; `BattleScenario` should become the main combat input and runtime spawn source.
   - Next implementation step: stabilize battle flow and make normal/elite/boss wave authoring richer instead of reintroducing fixed enemy rosters.
   - Do not implement conditional scenario events yet unless a concrete boss/special encounter cannot be expressed with `AtBattleStart`, `AtTimeMs`, `WinCondition`, and `TacticalPlan`.
   - Core-internal empty/manual test fixtures, the skill-test common harness, skill refactor validation, replay validation tests, ranged attack tests, live item skill activation tests, movement timeline export tests, Rapier movement tests, and Unity timeline export tests now use `BattleScenario` directly.
   - Do not reintroduce `PlayerDeckInfo` or deck-oriented battle startup helpers.
2. Define enemy wave contracts in data.
   - Use `PveEncounter.node_type` for reward identity and mission framing; do not overload `BattlefieldArchetype` with narrative purpose.
   - Keep map node kind (`combat_monster`, `combat_elite`, `boss_abnormality`) aligned with encounter assignment policy; do not go back to drawing every combat node from the full encounter pool.
   - Build node-type reward policy on top of `RewardMetadata.tags`; keep reward tags as semantic categories, not direct grant logic.
   - Continue replacing generic combat rewards with node-purpose reward pools. `Forbidden` rewards must stay out of PVE encounter pools.
   - Continue adding node-purpose reward pools for Defense/Recovery/Frontline/Encirclement as live encounter types grow.
   - Keep SplitOperation reward pools empty until split-squad gameplay exists; current policy rejects non-empty SplitOperation rewards.
   - Ensure normal waves use `CorrodedEmployee` as the default enemy direction.
   - Keep `Abnormality` as elite/boss/special encounter direction.
   - Leave `FacilityEntity` and `BattlefieldHazard` as future extensions, not immediate implementation.
3. Expand `BattlefieldGenerator`.
   - Continue adding authored ASCII templates to `game_resources/data/map/battlefield_templates.ron`.
   - Add template selection tags or encounter-level template preferences before adding procedural variation.
   - Continue expanding production validation as authored battlefield parameters grow.
   - Do not reintroduce hardcoded deployment/spawn/obstacle geometry in `BattlefieldGenerator`.
   - Keep authored `PveEncounter.static_obstacles` as an explicit per-encounter layout override; add a separate obstacle-generation policy only if designers need additive/generated variants later.
4. Keep node-scoped deployment as the only official combat placement contract.
   - Old map-state `MoveUnit`/`TransferUnit` semantics have been removed.
   - `MoveUnit` is valid only in Combat/Boss `NodeConfirm` and writes to `CombatDeployment`.
   - `MoveBenchUnit` remains for roster bench ordering.
   - Do not reintroduce run-persistent Field placement as a player-facing flow.
5. Connect skill fragment progress to combat behavior.
   - Verify `upgrade_skill_ids` and `awakened_skill_id` selection in battle profiles.
   - Define `fragment_dust` sinks and replacement costs.
   - Research-completion delivery loop exists: threshold detection, pending HQ deliveries, `ResearchDeliveryPolicy` safe-node allowlist, auto-receive on safe node entry, snapshot exposure, and immediate `BehaviorResult.research_deliveries` exposure for safe node entry results.
6. Expand post-combat progression.
   - Strengthen employee XP/growth rewards.
   - Continue connecting suppression/containment outcomes to broader skill fragment rewards.
   - Emit richer trauma, injury, and trust memory events.
   - Preserve the agreed rule that node failure is not run failure; failed combat/defense/recovery should produce consequences and return to map flow unless the deployable roster/recovery-route run-failure condition is met.
7. Expand Maintenance.
   - Equipment enhancement/restoration/dismantle foundations exist; next Maintenance work should focus on repair/modification policies only after their gameplay purpose is clear.
   - Expose maintenance results clearly in behavior results and snapshots.
   - Keep Maintenance repeatable until explicit node completion; do not consume the node after a single Maintenance action.
8. Stabilize data and client contracts.
   - Map node definitions already live in shared game resources; keep future map authoring data there instead of reintroducing `core_enhanced/config` data files.
   - Decide whether battlefield archetype generation parameters should move to RON.
   - Client-facing reward session terminology has been renamed to Reward. Do not reintroduce the old reward-session names.

## Useful Verification Commands

Run these from `core_enhanced`:

```bash
cargo fmt
cargo check
cargo test run_snapshot
cargo test support_node
cargo test game::world::tests::map_flow::
cargo test game::world::tests::equipment::
cargo test game::world::tests::combat::
cargo test game::events::combat::tests::
cargo test game::world::tests::node_sessions::
cargo test game::world::tests::placement::
```

Avoid overly broad filters such as `cargo test equipment` when validating this refactor, because they may include older integration tests unrelated to the current GameCore world tests.

## Design Docs

The docs directory is intentionally small. Treat these as the active references:

- `docs/game_rulebook.md`: current Korean gameplay rules, including node preview, combat briefing, precision scan, support nodes, run failure, and skill fragment rules.
- `docs/gameplay_flow_example.md`: short playthrough-style example showing node choices, placement windows, and available actions.
- `docs/current_handoff.md`: current implementation handoff and recommended next work.
- `docs/core_system_goal_plan.md`: goal-mode style plan for remaining non-content core system gaps, with success criteria and quick verification commands.
- `docs/battlefield_archetypes.md`: battlefield archetype generation rules, enemy wave composition direction, and validation criteria for the future `BattlefieldGenerator`.
- `docs/battle_scenario_core_refactor_plan.md`: detailed plan for replacing deck-based battle startup with `BattleScenario`, runtime spawn events, and wave-aware win conditions.
- `docs/tactical_ai_objective_plan.md`: battle objective, side movement policy, defensive anchor movement, and future group/formation movement design.
- `docs/employee_trust_system.md`: employee trust, memory, trait, dialogue, and high-risk acceptance design.
- `docs/skills/skill_fragment_game_system.md`: active skill fragment and unused fragment sink design.
- `docs/skills/lobotomy_a_rank_skill_goal_plan.md`: goal-mode plan for implementing currently feasible A-rank abnormality original/fragment skills.
- `docs/skills/lobotomy_a_rank_skill_design.md`: A-rank abnormality original skill and employee fragment design notes.
- `docs/legacy_easter_egg_system.md`: one-time former-employee legacy encounter, `잔향 추적`, survival log, and archive design.
