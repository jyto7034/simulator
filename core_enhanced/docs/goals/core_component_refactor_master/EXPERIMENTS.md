# Core Component Refactor Master Experiments

## Experiment Log Policy

작은 trial and error를 통해 구조를 확인하고, 실패한 접근과 이유를 여기에 기록한다.

각 기록은 가능하면 아래 형식을 따른다.

- Context: 어떤 컴포넌트와 어떤 가설을 확인했는가.
- Action: 어떤 코드를 읽거나 바꾸거나 어떤 검증을 돌렸는가.
- Result: 성공, 실패, 보류 중 무엇인가.
- Failure cause: 실패했다면 원인은 무엇인가.
- Follow-up: 수정, 재검증, 문서화 결과는 무엇인가.

테스트 실패는 실패 원인, 수정 내용, 재검증 결과를 반드시 포함한다.

최종 재검토에서 refactor 문서와 실제 코드, live RON/data, Unity-facing 계약, 테스트 사이의 불일치가 발견되면 아래를 기록한다.

- Which document: 어떤 `<component_name>_refactor.md`였는가.
- Mismatch: 문서 판단과 실제 근거가 어떻게 달랐는가.
- Correction: 유지, 수정, 보류, 삭제 중 무엇으로 정정했는가.
- Verification: 어떤 파일, 데이터, 테스트로 다시 확인했는가.

## 2026-06-21 - Master goal recreated as component map

Action:

- Created `docs/goals/core_component_refactor_master/`.
- Wrote the project component breakdown and audit priority into `PLAN.md`.

Result:

- No sub goal was created.
- No code was changed.
- No Rust tests were run because this is planning documentation only.

Notes:

- The previous automatic-looking master/sub goal documents were removed at user request.
- This version is intentionally a component map and reading plan, not an auto-generated sub goal queue.

## 2026-06-21 - Master goal reframed as component refactor audit queue

Context:

- User clarified that the goal is to inspect every component in the component map and create one `<component_name>_refactor.md` document per component.

Action:

- Updated `PLAN.md` to define component audit documents as the primary output.
- Added progress tracking for all 16 components.
- Added source-of-truth, legacy removal, testing, stop condition, and reporting rules.

Result:

- Success. The goal now describes a full component-by-component refactor audit workflow.

Validation:

- No Rust tests were run because this was a documentation-only update.

## 2026-06-21 - Component 1 world/run progression audit documented

Context:

- Audited the first component, `월드/런 진행 시스템`, using `docs/refactor_preparation_plan.md` as the refactor criteria.

Action:

- Read `src/game/world.rs`, `src/game/world/state.rs`, `src/game/world/node_flow.rs`, `src/game/world/map_content.rs`, `src/game/world/combat.rs`, `src/game/world/support.rs`, `src/game/world/snapshot.rs`, `src/game/world/helpers.rs`, `src/game/managers/action_scheduler.rs`, `src/game/resources/state.rs`, and `src/game/resources/action.rs`.
- Checked live data touchpoints in `../game_resources/data/map/node_definitions.ron` and `../game_resources/data/employees/starter_candidates.ron`.
- Checked test coverage references in `src/game/world/tests/*` and `tests/ron_loading.rs`.
- Created `world_run_progression_refactor.md`.
- Updated `PLAN.md` progress for component 1.

Result:

- Success. Component 1 now has an initial refactor audit document.
- Main candidates recorded: `RUN_SYSTEM_POLICY` policy ownership, `GameState`/`ActiveNodeContent`/allowed action context parallel state, maintenance preview responsibility, reward policy follow-up, and battle record export source-of-truth clarification.

Validation:

- No Rust tests were run because this step only created planning/audit documentation.

## 2026-06-21 - Component 14 data/RON loading and validation audit documented

Context:

- Audited the fourteenth component, `데이터/RON 로딩 및 검증 시스템`, using `docs/refactor_preparation_plan.md` as the refactor criteria.

Action:

- Read `src/game/data/mod.rs`, `src/game/data/validation.rs`, major data modules, `src/game/map/types.rs`, `src/game/combat_preview/mod.rs`, `src/game/battle/buffs.rs`, `src/game/battle/validation/validator.rs`, and `../game_server/src/main.rs`.
- Checked live data paths under `../game_resources/data`, especially base RON files, map node definitions, battlefield archetypes/templates, PVE encounters, shop/reward data, and legacy random event files.
- Checked test loader and live RON assertions in `tests/common/mod.rs`, `src/game/world/tests/mod.rs`, `tests/ron_loading.rs`, `tests/live_skill_catalog_audit.rs`, and `tests/live_item_skill_activation.rs`.
- Checked current policy docs in `docs/refactor_preparation_plan.md`, `docs/game_rulebook.md`, and `docs/code_documentation_sync_guidelines.md`.
- Created `data_ron_validation_refactor.md`.
- Updated `PLAN.md` progress for component 14.

Result:

- Success. Component 14 now has an initial refactor audit document.
- Main candidates recorded: duplicated official live RON loaders, misleading `GameDataBuilder::live_defaults()` scope, code-injected starter skill fragment, map/battlefield RON validation outside `GameDataBase`, generated combat preview validation being test-strong but not server-startup-strong, reward explicit tag/effect drift, legacy random event RON cleanup, and possible typed loader error boundary.

Validation:

- No Rust tests were run because this step only created planning/audit documentation.

## 2026-06-21 - Component 13 reward/shop/support audit documented

Context:

- Audited the thirteenth component, `보상/상점/지원 노드 시스템`, using `docs/refactor_preparation_plan.md` as the refactor criteria.

Action:

- Read `src/game/reward.rs`, `src/game/reward_policy.rs`, `src/game/world/reward.rs`, `src/game/world/shop.rs`, `src/game/world/support.rs`, `src/game/world/headquarters.rs`, `src/game/world/map_content.rs`, `src/game/world/node_flow.rs`, `src/game/world/helpers.rs`, `src/game/resources/selection.rs`, `src/game/resources/inventory.rs`, and `src/game/skill_fragment.rs`.
- Checked data and validation touchpoints in `src/game/data/reward_data.rs`, `src/game/data/shop_data.rs`, `src/game/data/validation.rs`, `../game_resources/data/events/rewards/base.ron`, `../game_resources/data/events/rewards/legacy_random_event_rewards.ron`, `../game_resources/data/events/shops/base.ron`, `../game_resources/data/map/node_definitions.ron`, and `../game_resources/data/pve/encounters.ron`.
- Checked policy and Unity-facing contracts in `docs/game_rulebook.md`, `docs/refactor_preparation_plan.md`, and `/mnt/f/unity projects/ark/docs/unity_core_contract.md`.
- Checked related tests in `src/game/world/tests/map_flow.rs`, `src/game/world/tests/node_sessions.rs`, `src/game/world/tests/support.rs`, and `tests/ron_loading.rs`.
- Created `reward_shop_support_refactor.md`.
- Updated `PLAN.md` progress for component 13.

Result:

- Success. Component 13 now has an initial refactor audit document.
- Main candidates recorded: `GrantExperience` executor/combat source split, non-atomic reward application, missing skill fragment/research diff in `RewardGranted`, reward/shop item grant duplication, whole-DB random equipment reward, map reward pool runtime filtering versus validation, duplicated shop stock operations, Medical trust event semantics, and Enkephalin overflow policy divergence.

Validation:

- No Rust tests were run because this step only created planning/audit documentation.

## 2026-06-21 - Component 12 item/equipment/skill-fragment audit documented

Context:

- Audited the twelfth component, `아이템/장비/스킬 파편 경제 시스템`, using `docs/refactor_preparation_plan.md` as the refactor criteria.

Action:

- Read `src/game/resources/inventory.rs`, `src/game/resources/item_slot.rs`, `src/game/skill_fragment.rs`, `src/game/world/maintenance.rs`, `src/game/world/support.rs`, `src/game/world/helpers.rs`, `src/game/world/snapshot.rs`, `src/game/employee.rs`, `src/game/data/mod.rs`, `src/game/data/equipment_data.rs`, and `src/game/data/skill_fragment_data.rs`.
- Checked live equipment and skill fragment data in `../game_resources/data/equipments/base.ron` and `../game_resources/data/skill_fragments/base.ron`.
- Checked Unity inventory, consumable, skill fragment, and Maintenance contracts in `/mnt/f/unity projects/ark/docs/unity_core_contract.md`.
- Checked focused equipment/maintenance tests in `src/game/world/tests/equipment.rs` and live RON tests in `tests/ron_loading.rs`.
- Created `item_equipment_skill_fragment_refactor.md`.
- Updated `PLAN.md` progress for component 12.

Result:

- Success. Component 12 now has an initial refactor audit document.
- Main candidates recorded: equipment equipped-state duplication between `ItemSlot` and `OwnedEquipment.equipped_to`, ambiguous skill-fragment shared-unlock versus per-copy ownership semantics, Maintenance preview/validation/handler policy duplication, Maintenance projection living in `support.rs`, hard-coded `fragment_dust`/`equipment_dust` material IDs, dormant skill-fragment stacking policy surface, and bound equipment dismantle policy.

Validation:

- No Rust tests were run because this step only created planning/audit documentation.

## 2026-06-21 - Component 11 employee/growth/trust audit documented

Context:

- Audited the eleventh component, `직원/성장/신뢰도 시스템`, using `docs/refactor_preparation_plan.md` as the refactor criteria.

Action:

- Read `src/game/employee.rs`, `src/game/growth.rs`, `src/game/employee_trust.rs`, `src/game/world/combat.rs`, `src/game/world/support.rs`, `src/game/world/state.rs`, `src/game/world/snapshot.rs`, `src/game/combat_setup/player_spawns.rs`, `src/game/battle/stat_pipeline.rs`, and `src/game/data/employee_data.rs`.
- Checked live employee candidate data in `../game_resources/data/employees/starter_candidates.ron` and `../game_resources/data/employees/recruitment_candidates.ron`.
- Checked Unity roster contract in `/mnt/f/unity projects/ark/docs/unity_core_contract.md`.
- Checked focused world/employee tests in `src/game/world/tests/combat.rs`, `src/game/world/tests/support.rs`, `src/game/world/tests/equipment.rs`, and `src/game/world/tests/snapshots_and_start.rs`.
- Created `employee_growth_trust_refactor.md`.
- Updated `PLAN.md` progress for component 11.

Result:

- Success. Component 11 now has an initial refactor audit document.
- Main candidates recorded: `level` versus `combat_profile.grade` source-of-truth split, no-op `PveWinStack`/`QuestRewardStack`, trust dialogue cue loss in snapshot, battle incapacitation not producing trust memory under narrative policy, dormant trust feature flags, run HP versus battle max HP policy boundary, and hard-coded level/growth curve.

Validation:

- No Rust tests were run because this step only created planning/audit documentation.

## 2026-06-21 - Component 10 buff/status-effect audit documented

Context:

- Audited the tenth component, `버프/상태이상 시스템`, using `docs/refactor_preparation_plan.md` as the refactor criteria.

Action:

- Read `src/game/battle/buffs.rs`, `src/game/battle/core/sim.rs`, `src/game/battle/core/types.rs`, `src/game/battle/core/commands.rs`, `src/game/battle/enums.rs`, `src/game/battle/timeline.rs`, `src/game/battle/validation/buffs.rs`, and `src/game/battle/validation/validator.rs`.
- Checked live buff data in `../game_resources/data/buffs/base.ron`.
- Checked policy and Unity-facing contracts in `docs/refactor_preparation_plan.md`, `docs/skill_target_contract.md`, `docs/component_design_review.md`, `/mnt/f/unity projects/ark/docs/unity_core_contract.md`, and `/mnt/f/unity projects/ark/docs/core_unity_battle_transport_contract.md`.
- Created `buff_status_effect_refactor.md`.
- Updated `PLAN.md` progress for component 10.

Result:

- Success. Component 10 now has an initial refactor audit document.
- Main candidates recorded: hard CC active buff versus action lock source-of-truth split, hard CC replacement without `BuffExpired` presentation event, post-death active buff/tick lifecycle, runtime/validator buff policy duplication, `BuffDatabase::live_default()` usage boundary, generated `BuffId` collision validation, and status metadata invariants.

Validation:

- No Rust tests were run because this step only created planning/audit documentation.

## 2026-06-21 - Component 2 map/node audit documented

Context:

- Audited the second component, `맵/노드 시스템`, using `docs/refactor_preparation_plan.md` as the refactor criteria.

Action:

- Read `src/game/map/types.rs`, `src/game/map/generator.rs`, `src/game/map/progression.rs`, `src/game/map/executor.rs`, `src/game/map/session.rs`, `src/game/world/node_flow.rs`, `src/game/world/map_content.rs`, `src/game/world/map_encounters.rs`, `src/game/combat_setup/mission_policy.rs`, and related snapshot/test references.
- Checked live node authoring data in `../game_resources/data/map/node_definitions.ron`.
- Created `map_node_refactor.md`.
- Updated `PLAN.md` progress for component 2.

Result:

- Success. Component 2 now has an initial refactor audit document.
- Main candidates recorded: category/payload load-time validation, map generation policy split between RON and code, `MapNode.state`/`MapProgression` invariants, `NodeSessionKind` as derived projection, and `kind_id.contains("elite")` encounter assignment convention.

Validation:

- No Rust tests were run because this step only created planning/audit documentation.

## 2026-06-21 - Component 3 behavior/state gate audit documented

Context:

- Audited the third component, `행동/상태 게이트 시스템`, using `docs/refactor_preparation_plan.md` as the refactor criteria.

Action:

- Read `src/game/behavior.rs`, `src/game/managers/action_scheduler.rs`, `src/game/world.rs`, `src/game/world/helpers.rs`, `src/game/world/snapshot.rs`, `src/game/resources/action.rs`, and `src/game/resources/state.rs`.
- Checked Unity-facing command/snapshot contract references in `/mnt/f/unity projects/ark/docs/unity_core_contract.md`.
- Checked test coverage references in `src/game/managers/action_scheduler.rs` and `src/game/world/tests/*`.
- Created `behavior_state_gate_refactor.md`.
- Updated `PLAN.md` progress for component 3.

Result:

- Success. Component 3 now has an initial refactor audit document.
- Main candidates recorded: `ActionValidator.allowed_actions` derived-cache drift guard, pre-dispatch payload validation boundary, command surface coverage invariants, `AllowedActionContext` flow-context consolidation, and deferring `BehaviorResult` DTO decomposition to the Unity/server contract component.

Validation:

- No Rust tests were run because this step only created planning/audit documentation.

## 2026-06-21 - Component 4 battle runtime audit documented

Context:

- Audited the fourth component, `전투 런타임 시스템`, using `docs/refactor_preparation_plan.md` as the refactor criteria.

Action:

- Read `src/game/battle/core/mod.rs`, `src/game/battle/core/sim.rs`, `src/game/battle/core/build.rs`, `src/game/battle/core/commands.rs`, `src/game/battle/core/types.rs`, `src/game/battle/timeline.rs`, `src/game/battle/damage.rs`, `src/game/world/combat.rs`, `src/game/world/state.rs`, and `src/game/behavior.rs`.
- Checked live data touchpoints under `../game_resources/data/pve`, `../game_resources/data/map`, `../game_resources/data/skills`, `../game_resources/data/buffs`, `../game_resources/data/abnormalities`, and `../game_resources/data/enemies`.
- Checked Unity/server-facing battle transport contracts in `/mnt/f/unity projects/ark/docs/unity_core_contract.md`, `/mnt/f/unity projects/ark/docs/core_unity_battle_transport_contract.md`, and `../game_server/src/game/player_game_actor/state.rs`.
- Checked test coverage references in `src/game/world/tests/combat.rs`, `src/game/battle/core/*`, `tests/skill_refactor_validation.rs`, `tests/live_item_skill_activation.rs`, and `tests/ron_loading.rs`.
- Created `battle_runtime_refactor.md`.
- Updated `PLAN.md` progress for component 4.

Result:

- Success. Component 4 now has an initial refactor audit document.
- Main candidates recorded: `LiveBattleDeploymentState`/`BattleCore` sync transaction boundary, withdraw event contract mismatch, core battle `BehaviorResult` versus server `CommandAccepted` dual surface, battle update cursor invariants, and hard-coded battle runtime policy constants.

Validation:

- No Rust tests were run because this step only created planning/audit documentation.

## 2026-06-21 - Component 5 battlefield/scenario/wave audit documented

Context:

- Audited the fifth component, `전장/시나리오/웨이브 시스템`, using `docs/refactor_preparation_plan.md` as the refactor criteria.

Action:

- Read `src/game/combat_preview/mod.rs`, `src/game/combat_preview/types.rs`, `src/game/combat_preview/template.rs`, `src/game/combat_preview/validation.rs`, `src/game/combat_setup/battlefield_plan.rs`, `src/game/combat_setup/enemy_spawns.rs`, `src/game/combat_setup/defense_object.rs`, `src/game/combat_setup/mission_policy.rs`, `src/game/combat_setup/scenario_groups.rs`, `src/game/battle/scenario.rs`, `src/game/wave_resolution.rs`, `src/game/events/combat.rs`, `src/game/world/combat.rs`, `src/game/data/pve_data.rs`, `src/game/data/corroded_wave_data.rs`, and `src/game/data/validation.rs`.
- Checked live data touchpoints under `../game_resources/data/pve`, `../game_resources/data/map`, and `../game_resources/data/enemies`.
- Checked Unity-facing battlefield/preview/setup contract references in `docs/game_rulebook.md` and `/mnt/f/unity projects/ark/docs/unity_core_contract.md`.
- Checked test coverage references in `src/game/combat_preview/mod.rs`, `src/game/world/tests/combat.rs`, `src/game/world/tests/map_flow.rs`, and `tests/ron_loading.rs`.
- Created `battlefield_scenario_wave_refactor.md`.
- Updated `PLAN.md` progress for component 5.

Result:

- Success. Component 5 now has an initial refactor audit document.
- Main candidates recorded: live RON wave route/spawn-zone validation intent, empty fallback wave policy, archetype random selection ownership, generated defense route naming/policy boundary, FacilityEntity placeholder schema, and scenario handoff validation clarity.

Validation:

- No Rust tests were run because this step only created planning/audit documentation.

## 2026-06-21 - Component 6 movement/blocking audit documented

Context:

- Audited the sixth component, `이동/저지 시스템`, using `docs/refactor_preparation_plan.md` as the refactor criteria.

Action:

- Read `src/game/battle/core/movement/types.rs`, `src/game/battle/core/movement/engine.rs`, `src/game/battle/core/movement/rapier_backend.rs`, `src/game/battle/core/movement/steering.rs`, `src/game/battle/core/movement/path.rs`, `src/game/battle/core/movement/blocking.rs`, `src/game/battle/core/movement/planner.rs`, `src/game/battle/core/movement/lifecycle.rs`, `src/game/battle/core/mod.rs`, `src/game/battle/core/sim.rs`, `src/game/battle/core/build.rs`, `src/game/battle/core/basic_attack.rs`, `src/game/battle/core/commands.rs`, `src/game/battle/timeline.rs`, and `src/game/battle/scenario.rs`.
- Checked live data touchpoints in `../game_resources/data/abnormalities/base.ron`, `../game_resources/data/enemies/corroded_employees.ron`, `../game_resources/data/map/battlefield_templates.ron`, `../game_resources/data/pve/encounters.ron`, `src/game/data/abnormality_data.rs`, and `src/game/data/corroded_employee_data.rs`.
- Checked Unity-facing movement/blocking contract references in `docs/game_rulebook.md`, `/mnt/f/unity projects/ark/docs/unity_core_contract.md`, and `/mnt/f/unity projects/ark/docs/core_unity_battle_transport_contract.md`.
- Created `movement_blocking_refactor.md`.
- Updated `PLAN.md` progress for component 6.

Result:

- Success. Component 6 now has an initial refactor audit document.
- Main candidates recorded: `BlockRuntimeState`/`RuntimeUnit.current_target` target ownership boundary, internal movement stop versus timeline stop semantics, Direct/Rapier backend role clarity, movement policy constants ownership, static obstacle/void tile composition, and opponent spawn fallback policy boundary.

Validation:

- No Rust tests were run because this step only created planning/audit documentation.

## 2026-06-21 - Component 7 skill/targeting/range audit documented

Context:

- Audited the seventh component, `스킬/타겟팅/범위 시스템`, using `docs/refactor_preparation_plan.md` as the refactor criteria.

Action:

- Read `src/game/ability.rs`, `src/game/data/skill_data.rs`, `src/game/battle/tile_range.rs`, `src/game/range_preview.rs`, `src/game/battle/core/sim.rs`, `src/game/battle/core/skill_runtime/cast.rs`, `src/game/battle/core/skill_runtime/area.rs`, `src/game/battle/core/skill_runtime/projectile.rs`, `src/game/battle/core/targeting.rs`, `src/game/battle/core/target_usefulness.rs`, `src/game/battle/battlefield/field.rs`, `src/game/world/combat.rs`, and `src/game/world/snapshot.rs`.
- Checked live skill data in `../game_resources/data/skills/base.ron`, `../game_resources/data/skill_fragments/base.ron`, and `../game_resources/data/enemies/corroded_employees.ron`.
- Checked policy and Unity-facing range/skill catalog contracts in `docs/skill_target_contract.md`, `docs/game_rulebook.md`, `/mnt/f/unity projects/ark/docs/unity_core_contract.md`, and `/mnt/f/unity projects/ark/docs/core_unity_battle_transport_contract.md`.
- Checked test coverage references in `src/game/range_preview.rs`, `src/game/battle/core/mod.rs`, `src/game/battle/core/targeting.rs`, `tests/skill_refactor_validation.rs`, `tests/skill_test_suite.rs`, `tests/live_skill_catalog_audit.rs`, and `tests/ron_loading.rs`.
- Created `skill_targeting_range_refactor.md`.
- Updated `PLAN.md` progress for component 7.

Result:

- Success. Component 7 now has an initial refactor audit document.
- Main candidates recorded: `SkillCastTargetingDef::FirstStepTarget` compatibility/default policy, `range_units` eligibility isolation, shared range-valid-tile helper, `TileArea` affected tile clipping/presentation policy, directional projectile endpoint policy versus tile-range composition, and skill catalog/range preview contract guarding.

Validation:

- No Rust tests were run because this step only created planning/audit documentation.

## 2026-06-21 - Component 8 basic attack/projectile/judgement audit documented

Context:

- Audited the eighth component, `기본 공격/투사체/공격 판정 시스템`, using `docs/refactor_preparation_plan.md` as the refactor criteria.

Action:

- Read `src/game/battle/core/basic_attack.rs`, `src/game/battle/core/commands.rs`, `src/game/battle/core/targeting.rs`, `src/game/battle/core/target_usefulness.rs`, `src/game/battle/timeline.rs`, `src/game/battle/validation/attacks.rs`, `src/game/battle/validation/deaths.rs`, `src/game/data/abnormality_data.rs`, `src/game/data/equipment_data.rs`, `src/game/data/corroded_employee_data.rs`, and `src/game/ability.rs`.
- Checked live data touchpoints in `../game_resources/data/abnormalities/base.ron`, `../game_resources/data/equipments/base.ron`, and `../game_resources/data/enemies/corroded_employees.ron`.
- Checked policy and Unity-facing contracts in `docs/skill_target_contract.md`, `docs/refactor_preparation_plan.md`, `/mnt/f/unity projects/ark/docs/unity_core_contract.md`, and `/mnt/f/unity projects/ark/docs/core_unity_battle_transport_contract.md`.
- Checked historical goal context in `docs/goals/basic_attack_lifecycle_refactor`, `docs/goals/basic_attack_projectile_target_only_collision`, and `docs/goals/tile_based_attack_delivery_contract`.
- Created `basic_attack_projectile_judgement_refactor.md`.
- Updated `PLAN.md` progress for component 8.

Result:

- Success. Component 8 now has an initial refactor audit document.
- Main candidates recorded: same-time `AttackResolve` batch semantics, dead-attacker projectile damage context owner snapshot scope, projectile miss dual-event source-of-truth, projectile speed zero fallback, unused basic attack data helper surface, and `SplashClusterFirst` hard-coded policy.

Validation:

- No Rust tests were run because this step only created planning/audit documentation.

## 2026-06-21 - Component 9 damage/stats/battle-entry audit documented

Context:

- Audited the ninth component, `피해/스탯/전투 진입 스탯 시스템`, using `docs/refactor_preparation_plan.md` as the refactor criteria.

Action:

- Read `src/game/stats.rs`, `src/game/battle/damage.rs`, `src/game/battle/stat_pipeline.rs`, `src/game/employee.rs`, `src/game/battle/types.rs`, `src/game/battle/core/build.rs`, `src/game/battle/core/commands.rs`, `src/game/battle/core/triggers.rs`, `src/game/battle/core/movement/steering.rs`, `src/game/battle/core/movement/types.rs`, `src/game/battle/validation/unit_stats.rs`, and `src/game/battle/validation/state.rs`.
- Checked live stat modifier data in `../game_resources/data/equipments/base.ron`, `../game_resources/data/artifacts/base.ron`, `../game_resources/data/abnormalities/base.ron`, and `../game_resources/data/enemies/corroded_employees.ron`.
- Checked policy and Unity-facing movement/stat event contracts in `docs/game_rulebook.md`, `/mnt/f/unity projects/ark/docs/unity_core_contract.md`, and `/mnt/f/unity projects/ark/docs/core_unity_battle_transport_contract.md`.
- Checked historical goal context in `docs/goals/stats_pipeline_unification` and `docs/goals/stat_pipeline_inventory_contract_repair`.
- Created `damage_stats_entry_refactor.md`.
- Updated `PLAN.md` progress for component 9.

Result:

- Success. Component 9 now has an initial refactor audit document.
- Main candidates recorded: `MoveSpeedUnitsPerMs` final stat versus `UnitBody.move_speed` source-of-truth split, `spawn_scenario_group()` combat profile double calculation/final stat overwrite, `DamageResult` death command duplication, unused damage context fields, and move speed modifier presentation policy.

Validation:

- No Rust tests were run because this step only created planning/audit documentation.

## 2026-06-21 - Component 15 Unity/server contract audit documented

Context:

- Audited the fifteenth component, `Unity/server-facing DTO/계약 표면`, using `docs/refactor_preparation_plan.md` as the refactor criteria.

Action:

- Read `src/game/behavior.rs`, `src/game/world/snapshot.rs`, `src/game/world/state.rs`, `src/game/world/combat.rs`, `src/game/world/admin.rs`, and `src/game/world/admin/catalog.rs`.
- Read `../game_server/src/game/player_game_actor/messages.rs`, `../game_server/src/game/player_game_actor/state.rs`, `../game_server/src/game/player_game_actor/handlers.rs`, and contract-related test sections.
- Checked Unity-facing contract references in `/mnt/f/unity projects/ark/docs/unity_core_contract.md` and `/mnt/f/unity projects/ark/docs/core_unity_battle_transport_contract.md`.
- Created `unity_server_contract_refactor.md`.
- Updated `PLAN.md` progress for component 15.

Result:

- Success. Component 15 now has an initial refactor audit document.
- Main candidates recorded: `PlayerBehaviorRequest`/`PlayerBehavior` mirroring, `BehaviorResult` transport classification table, server-owned live battle side-message rules, ad-hoc snapshot DTO surface, server-side `compressed_timeline` snapshot augmentation, dormant `BattleResync.setup`, live DTO `message_type` versus server envelope `type`, and admin `Value` payload boundaries.

Validation:

- No Rust tests were run because this step only created planning/audit documentation.

## 2026-06-21 - Component 16 validation/test harness audit documented

Context:

- Audited the sixteenth component, `검증/테스트 하네스`, using `docs/refactor_preparation_plan.md` as the refactor criteria.

Action:

- Read `src/game/battle/validation/*`, especially `types.rs`, `validator.rs`, `state.rs`, `buffs.rs`, `parent.rs`, `auto_attack.rs`, and `spawns.rs`.
- Read `tests/ron_loading.rs`, `tests/common/mod.rs`, `tests/live_skill_catalog_audit.rs`, `tests/skill_test_suite.rs`, `tests/skill_test/common/*`, and representative world/server contract tests.
- Checked validator usage with `rg` to see whether `TimelineValidator` is integrated into high-level gameplay tests.
- Created `validation_test_harness_refactor.md`.
- Updated `PLAN.md` progress for component 16.

Result:

- Success. Component 16 now has an initial refactor audit document.
- Main candidates recorded: timeline validator not integrated into gameplay tests, validator config/actual-check mismatch, legacy timeline versions `6`/`7`, duplicated live RON loaders, hard-coded live content roster manifests, debug event log test side effects, `live_defaults()` naming/behavior mismatch, and scattered Unity contract test grouping.

Validation:

- No Rust tests were run because this step only created planning/audit documentation.

## 2026-06-21 - Final cross-check across component audit documents

Context:

- All 16 component audit documents had been created. The master goal required a final comparison between generated documents and actual code/live data/Unity contracts/tests.

Action:

- Listed the goal directory and verified all 16 `<component_name>_refactor.md` documents exist.
- Checked `PLAN.md` progress table for all components.
- Grepped all component documents for `docs/refactor_preparation_plan.md` references and policy markers.
- Re-checked repeated high-risk claims against code/data/docs with `rg`, including live RON loaders, starter fragment injection, `BattleResync`, `compressed_timeline`, `TimelineValidator`, timeline version acceptance, reward effects, `FirstStepTarget`, movement events, and debug event exports.
- Recorded cross-component repeated candidates in `PLAN.md`.

Result:

- Success. Final cross-check completed.
- No component document was found to be plainly contradicted by the current code/data/Unity contract evidence inspected in this pass.
- Cross-component clusters were recorded so later refactor goals can avoid treating the same root issue as unrelated work.

Validation:

- No Rust tests were run because this step only compared and updated audit documentation.
