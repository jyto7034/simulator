# Audit Baseline B-001

- 감사 시작 시각: `2026-08-21T23:03:28+09:00 KST`
- timezone: `Asia/Seoul` (UTC+09:00)
- repository root: `/mnt/f/work/simulator`
- runtime working directory: `/mnt/f/work/simulator/core_enhanced`
- HEAD: `f318414fb0156a67e04c5479000c2a2683fc0c09`
- 적용 예정 baseline feature: `--all-features`
- Cargo/Rust 관련 환경 변수: 출력 없음
- 실행 환경: `Linux Acr0tic 6.18.33.2-microsoft-standard-WSL2 #1 SMP PREEMPT_DYNAMIC Thu Jun 18 21:54:43 UTC 2026 x86_64 x86_64 x86_64 GNU/Linux`

## Content Manifest Identifier

포함 표면은 `../Cargo.lock`, `Cargo.toml`, `src/lib.rs`, `src/game`, `tests`,
`../game_resources/data` 아래의 현재 regular file이다. Production runtime, runtime test, dependency lock와
C07에서 사용할 live-data 기준선을 함께 식별한다.

- file count: `210`
- aggregate SHA-256: `e30995cec4249a92c1d3981d4e7b5e64c153dac4b6e5df90d4f7d9fbb510fb61`
- 계산 명령:

```bash
find ../Cargo.lock Cargo.toml src/lib.rs src/game tests ../game_resources/data -type f -print0 |
  LC_ALL=C sort -z |
  xargs -0 sha256sum |
  sha256sum
```

집계 해시는 정렬된 각 파일의 `sha256sum` 출력(내용 해시와 상대 경로 포함)을 다시 SHA-256으로 계산한
값이다. 따라서 commit ID만이 아니라 dirty/untracked 파일 내용과 경로를 식별한다.

## Final contamination check

- evidence: E-022
- final HEAD: unchanged
- final manifest file count: `210` (unchanged)
- final aggregate SHA-256: `e30995cec4249a92c1d3981d4e7b5e64c153dac4b6e5df90d4f7d9fbb510fb61`
  (exact match)
- inspected adjacent snapshot hashes: exact match for all ADJ-PERSIST-001/ADJ-TRANSPORT-001 files
- conclusion: production/runtime/test/live-data baseline remained unchanged; only Goal documents and `/tmp` probe artifacts
  were created or updated by the audit.

## Toolchain

```text
rustc 1.91.1 (ed61e7d7e 2025-11-07)
binary: rustc
commit-hash: ed61e7d7e242494fb7057f2657300d9e77bb4fcb
commit-date: 2025-11-07
host: x86_64-unknown-linux-gnu
release: 1.91.1
LLVM version: 21.1.2
```

```text
cargo 1.91.1 (ea2d97820 2025-10-10)
release: 1.91.1
commit-hash: ea2d97820c16195b0ca3fadb4319fe512c199a43
commit-date: 2025-10-10
host: x86_64-unknown-linux-gnu
libgit2: 1.9.1 (sys:0.20.2 vendored)
libcurl: 8.15.0-DEV (sys:0.4.83+curl-8.15.0 vendored ssl:OpenSSL/3.5.2)
ssl: OpenSSL 3.5.2 5 Aug 2025
os: Ubuntu 24.4.0 (noble) [64-bit]
```

## Exact `git status --short`

다음은 B-001 고정 직전의 전체 출력이다. Goal 디렉터리가 이미 untracked이므로 이후 Goal 문서 내부 편집은
이 출력의 top-level shape를 바꾸지 않는다.

```text
 M ../Cargo.lock
 D .cargo/config.toml
 M Cargo.toml
 M docs/README.md
 M docs/core_runtime_contract.md
 M docs/data_loading_contract.md
 M docs/engineering_workflow.md
 M docs/game_rulebook.md
 D docs/goal_completion_review_guide.md
 M docs/skill_target_contract.md
 D docs/skills/abnormality_skill_design_notes.ko.md
 M docs/skills/lobotomy_content_catalog.md
 M src/bin/generate_encounter_compendium_contract_fixture.rs
 M src/bin/generate_inventory_contract_fixture.rs
 D src/bin/generate_research_delivery_contract_fixture.rs
 M src/bin/generate_shop_contract_fixture.rs
 M src/game/ability.rs
 D src/game/abnormality_research.rs
 M src/game/battle/battlefield/mod.rs
 M src/game/battle/buffs.rs
 M src/game/battle/core/basic_attack.rs
 M src/game/battle/core/basic_attack_projectile.rs
 M src/game/battle/core/build.rs
 M src/game/battle/core/commands.rs
 M src/game/battle/core/damage_runtime.rs
 M src/game/battle/core/death_runtime.rs
 M src/game/battle/core/mod.rs
 M src/game/battle/core/movement/blocking.rs
 M src/game/battle/core/movement/engine.rs
 M src/game/battle/core/movement/lifecycle.rs
 M src/game/battle/core/movement/mod.rs
 M src/game/battle/core/movement/planner.rs
 M src/game/battle/core/movement/rapier_backend.rs
 M src/game/battle/core/movement/steering.rs
 M src/game/battle/core/movement/types.rs
 M src/game/battle/core/projectile_math.rs
 M src/game/battle/core/sim.rs
 M src/game/battle/core/skill_runtime/area.rs
 M src/game/battle/core/skill_runtime/cast.rs
 M src/game/battle/core/skill_runtime/projectile.rs
 M src/game/battle/core/spatial.rs
 M src/game/battle/core/target_usefulness.rs
 M src/game/battle/core/targeting.rs
 M src/game/battle/core/triggers.rs
 M src/game/battle/core/types.rs
 M src/game/battle/damage.rs
 D src/game/battle/death.rs
 M src/game/battle/enums.rs
 M src/game/battle/event_log.rs
 M src/game/battle/mod.rs
 M src/game/battle/result_stats.rs
 M src/game/battle/scenario.rs
 M src/game/battle/stat_pipeline.rs
 M src/game/battle/tile_range.rs
 M src/game/battle/types.rs
 D src/game/battle/validation/attacks.rs
 D src/game/battle/validation/auto_attack.rs
 D src/game/battle/validation/autocast.rs
 D src/game/battle/validation/battlefield.rs
 D src/game/battle/validation/buffs.rs
 D src/game/battle/validation/deaths.rs
 D src/game/battle/validation/focus.rs
 D src/game/battle/validation/mod.rs
 D src/game/battle/validation/parent.rs
 D src/game/battle/validation/spawns.rs
 D src/game/battle/validation/state.rs
 D src/game/battle/validation/types.rs
 D src/game/battle/validation/unit_stats.rs
 D src/game/battle/validation/validator.rs
 M src/game/behavior.rs
 M src/game/boss_omen.rs
 M src/game/combat_power.rs
 M src/game/combat_preview/mod.rs
 M src/game/combat_preview/types.rs
 M src/game/combat_setup/battlefield_plan.rs
 M src/game/combat_setup/defense_object.rs
 M src/game/combat_setup/enemy_spawns.rs
 M src/game/combat_setup/mission_policy.rs
 M src/game/combat_setup/mod.rs
 M src/game/combat_setup/player_spawns.rs
 M src/game/combat_setup/rewards.rs
 M src/game/combat_setup/scenario_groups.rs
 M src/game/data/abnormality_data.rs
 M src/game/data/boss_omen_data.rs
 M src/game/data/corroded_employee_data.rs
 M src/game/data/equipment_data.rs
 M src/game/data/live_ron.rs
 M src/game/data/mod.rs
 M src/game/data/pve_data.rs
 M src/game/data/reward_data.rs
 M src/game/data/run_policy_data.rs
 M src/game/data/skill_data.rs
 M src/game/data/skill_fragment_data.rs
 M src/game/data/validation.rs
 M src/game/data/visual_novel_trigger_data.rs
 M src/game/employee.rs
 M src/game/encounter_compendium.rs
 M src/game/events/combat.rs
 M src/game/managers/uuid_manager.rs
 M src/game/map/facility.rs
 M src/game/map/facility_generator.rs
 M src/game/map/mod.rs
 M src/game/map/progression.rs
 M src/game/map/types.rs
 M src/game/mod.rs
 M src/game/monolith.rs
 D src/game/pve_bonus_objectives.rs
 M src/game/range_preview.rs
 M src/game/resources/inventory.rs
 M src/game/resources/item_slot.rs
 M src/game/resources/mod.rs
 M src/game/resources/selection.rs
 M src/game/resources/state.rs
 M src/game/reward.rs
 M src/game/reward_policy.rs
 M src/game/skill_fragment.rs
 M src/game/stats.rs
 M src/game/wave_resolution.rs
 M src/game/world.rs
 M src/game/world/action_capability.rs
 M src/game/world/active_run.rs
 M src/game/world/admin/fixtures.rs
 M src/game/world/admin/grants.rs
 M src/game/world/admin/tests.rs
 M src/game/world/battle_capability.rs
 M src/game/world/combat.rs
 M src/game/world/combat_trace.rs
 M src/game/world/event_node.rs
 M src/game/world/facility.rs
 M src/game/world/headquarters.rs
 M src/game/world/helpers.rs
 M src/game/world/maintenance.rs
 M src/game/world/map_content.rs
 M src/game/world/map_encounters.rs
 M src/game/world/node_flow.rs
 M src/game/world/reward.rs
 M src/game/world/shop.rs
 M src/game/world/snapshot.rs
 M src/game/world/state.rs
 M src/game/world/tests/combat.rs
 M src/game/world/tests/equipment.rs
 M src/game/world/tests/facility.rs
 M src/game/world/tests/map_flow.rs
 M src/game/world/tests/mod.rs
 M src/game/world/tests/node_sessions.rs
 M src/game/world/tests/snapshots_and_start.rs
 M src/game/world/visual_novel_flow.rs
 M src/game/world/visual_novel_triggers.rs
 M src/lib.rs
 D src/main.rs
 M tests/common/mod.rs
 M tests/live_item_skill_activation.rs
 M tests/live_skill_catalog_audit.rs
 M tests/ron_loading.rs
 M tests/skill_runtime_contract.rs
 M tests/skill_test/common/scenario.rs
 M tests/skill_test/common/types.rs
 D tests/test_actor/mod.rs
 M ../game_resources/data/abnormalities/base.ron
 M ../game_resources/data/boss_omen/chains.ron
 M ../game_resources/data/consumables/base.ron
 M ../game_resources/data/employees/definitions.ron
 M ../game_resources/data/enemies/corroded_employees.ron
 M ../game_resources/data/equipments/base.ron
 M ../game_resources/data/events/rewards/base.ron
 M ../game_resources/data/map/facility_templates.ron
 M ../game_resources/data/map/node_assignment_profiles.ron
 M ../game_resources/data/pve/encounters.ron
 M ../game_resources/data/run/policy.ron
 M ../game_resources/data/skill_fragments/base.ron
 M ../game_resources/data/skills/base.ron
 M ../game_server/config/development.toml
 M ../game_server/config/production.toml
 M ../game_server/docs/unity_bridge_protocol.md
 M ../game_server/examples/generate_battle_revision_fixture.rs
 M ../game_server/src/env.rs
 M ../game_server/src/game/load_balance_actor/handlers.rs
 M ../game_server/src/game/load_balance_actor/mod.rs
 M ../game_server/src/game/player_game_actor/handlers.rs
 M ../game_server/src/game/player_game_actor/messages.rs
 M ../game_server/src/game/player_game_actor/mod.rs
 M ../game_server/src/game/player_game_actor/state.rs
 M ../game_server/src/main.rs
 M ../game_server/src/persistence/active_run_store.rs
 D ../game_server/src/persistence/encounter_compendium_store.rs
 M ../game_server/src/persistence/mod.rs
?? docs/audit/current_runtime_structural_integrity_audit_criteria.ko.md
?? docs/decisions/defense_route_tactical_command_policy_2026-08-06.md
?? docs/goal_execution_pipeline.md
?? docs/goals/boss_runtime_implementation_readiness_audit/
?? docs/goals/boss_runtime_v1_implementation_master/
?? docs/goals/boss_runtime_v1_implementation_master_phase_00_execution_preflight/
?? docs/goals/boss_runtime_v1_implementation_master_phase_01_battle_lifecycle_correctness/
?? docs/goals/boss_runtime_v1_implementation_master_phase_02_catalog_and_typed_setup/
?? docs/goals/boss_runtime_v1_implementation_master_phase_03_runtime_kernel_and_target_movement/
?? docs/goals/boss_runtime_v1_implementation_master_phase_04_track_and_rejoin_vertical_slice/
?? docs/goals/boss_runtime_v1_implementation_master_phase_05_live_reference_boss/
?? docs/goals/boss_runtime_v1_implementation_master_phase_07_completion_audit/
?? docs/goals/conditional_wave_and_boss_summon_contract_alignment_audit/
?? docs/goals/conditional_wave_and_boss_summon_policy/
?? docs/goals/core_enhanced_refactoring_audit_final_catalog_consolidation/
?? docs/goals/core_enhanced_refactoring_audit_master/
?? docs/goals/core_enhanced_refactoring_audit_phase_00_surface_inventory/
?? docs/goals/core_enhanced_refactoring_audit_phase_01_live_data_loading_and_validation/
?? docs/goals/core_enhanced_refactoring_audit_phase_02_run_map_and_persistence/
?? docs/goals/core_enhanced_refactoring_audit_phase_03_roster_inventory_and_economy/
?? docs/goals/core_enhanced_refactoring_audit_phase_04_interactive_node_and_visual_novel/
?? docs/goals/core_enhanced_refactoring_audit_phase_05_combat_preview_and_setup/
?? docs/goals/core_enhanced_refactoring_audit_phase_06_battle_lifecycle_and_result/
?? docs/goals/core_enhanced_refactoring_audit_phase_07_battle_spatial_and_tactical/
?? docs/goals/core_enhanced_refactoring_audit_phase_08_battle_effects_and_stats/
?? docs/goals/core_enhanced_refactoring_audit_phase_09_battle_targeting_and_delivery/
?? docs/goals/core_enhanced_refactoring_audit_phase_10_command_projection_and_server_seam/
?? docs/goals/core_enhanced_refactoring_post_boss_runtime_reaudit_final_catalog_consolidation/
?? docs/goals/core_enhanced_refactoring_post_boss_runtime_reaudit_master/
?? docs/goals/core_enhanced_refactoring_post_boss_runtime_reaudit_phase_00_current_tree_delta_inventory/
?? docs/goals/core_enhanced_refactoring_post_boss_runtime_reaudit_phase_01_live_data_loading_and_publication/
?? docs/goals/core_enhanced_refactoring_post_boss_runtime_reaudit_phase_02_run_map_and_persistence/
?? docs/goals/core_enhanced_refactoring_post_boss_runtime_reaudit_phase_03_roster_inventory_and_economy/
?? docs/goals/core_enhanced_refactoring_post_boss_runtime_reaudit_phase_04_interactive_node_and_visual_novel/
?? docs/goals/core_enhanced_refactoring_post_boss_runtime_reaudit_phase_05_combat_preview_setup_and_boss_publication/
?? docs/goals/core_enhanced_refactoring_post_boss_runtime_reaudit_phase_06_battle_lifecycle_result_and_boss_scheduling/
?? docs/goals/core_enhanced_refactoring_post_boss_runtime_reaudit_phase_07_battle_spatial_tactical_and_boss_movement/
?? docs/goals/core_enhanced_refactoring_post_boss_runtime_reaudit_phase_08_battle_effects_stats_and_boss_leaf_seams/
?? docs/goals/core_enhanced_refactoring_post_boss_runtime_reaudit_phase_09_battle_targeting_delivery_and_projectiles/
?? docs/goals/core_enhanced_refactoring_post_boss_runtime_reaudit_phase_10_command_projection_server_and_boss_boundary/
?? docs/goals/core_enhanced_refactoring_rebaselined_backlog_implementation_master/
?? docs/goals/core_enhanced_refactoring_rebaselined_backlog_implementation_phase_01_p09_c08/
?? docs/goals/core_enhanced_refactoring_rebaselined_backlog_implementation_phase_02_f_c01/
?? docs/goals/core_enhanced_refactoring_rebaselined_backlog_implementation_phase_03_p02_c01/
?? docs/goals/core_enhanced_refactoring_rebaselined_backlog_implementation_phase_04_p03_c02/
?? docs/goals/core_enhanced_refactoring_rebaselined_backlog_implementation_phase_05_p04_c01/
?? docs/goals/core_enhanced_refactoring_rebaselined_backlog_implementation_phase_06_p05_c01/
?? docs/goals/core_enhanced_refactoring_rebaselined_backlog_implementation_phase_07_p06_c05/
?? docs/goals/core_enhanced_refactoring_rebaselined_backlog_implementation_phase_08_p07_c01/
?? docs/goals/core_enhanced_refactoring_rebaselined_backlog_implementation_phase_09_p07_c02/
?? docs/goals/core_enhanced_refactoring_rebaselined_backlog_implementation_phase_10_p08_c03/
?? docs/goals/core_enhanced_refactoring_rebaselined_backlog_implementation_phase_11_p08_c01/
?? docs/goals/core_enhanced_refactoring_rebaselined_backlog_implementation_phase_12_p08_c02/
?? docs/goals/core_enhanced_refactoring_rebaselined_backlog_implementation_phase_13_p08_c04/
?? docs/goals/core_enhanced_refactoring_rebaselined_backlog_implementation_phase_14_p09_c02/
?? docs/goals/core_enhanced_refactoring_rebaselined_backlog_implementation_phase_15_p09_c01/
?? docs/goals/core_enhanced_refactoring_rebaselined_backlog_implementation_phase_16_p09_c06/
?? docs/goals/core_enhanced_refactoring_rebaselined_backlog_implementation_phase_17_p09_c03/
?? docs/goals/core_enhanced_refactoring_rebaselined_backlog_implementation_phase_18_p09_c04/
?? docs/goals/core_enhanced_refactoring_rebaselined_backlog_implementation_phase_19_p01_c03/
?? docs/goals/core_enhanced_refactoring_rebaselined_backlog_implementation_phase_20_p02_c03/
?? docs/goals/core_enhanced_refactoring_rebaselined_backlog_implementation_phase_21_p02_c04/
?? docs/goals/core_enhanced_refactoring_rebaselined_backlog_implementation_phase_22_p03_c03/
?? docs/goals/core_enhanced_refactoring_rebaselined_backlog_implementation_phase_23_p03_c04/
?? docs/goals/core_enhanced_refactoring_rebaselined_backlog_implementation_phase_24_p04_c03/
?? docs/goals/core_enhanced_refactoring_rebaselined_backlog_implementation_phase_25_f_c02/
?? docs/goals/core_enhanced_refactoring_rebaselined_backlog_implementation_phase_26_p04_c04/
?? docs/goals/core_enhanced_refactoring_rebaselined_backlog_implementation_phase_27_p05_c03/
?? docs/goals/core_enhanced_refactoring_rebaselined_backlog_implementation_phase_28_p08_c05/
?? docs/goals/core_enhanced_refactoring_rebaselined_backlog_implementation_phase_29_p09_c05/
?? docs/goals/core_enhanced_refactoring_rebaselined_backlog_implementation_phase_30_p10_c01/
?? docs/goals/core_enhanced_refactoring_rebaselined_backlog_implementation_phase_31_p10_c04/
?? docs/goals/core_enhanced_refactoring_rebaselined_backlog_implementation_phase_32_p10_c05/
?? docs/goals/core_enhanced_refactoring_rebaselined_backlog_implementation_phase_33_p10_c02/
?? docs/goals/core_enhanced_refactoring_rebaselined_backlog_implementation_phase_34_p10_c03/
?? docs/goals/core_enhanced_refactoring_rebaselined_backlog_implementation_phase_35_p10_c07/
?? docs/goals/core_enhanced_refactoring_rebaselined_backlog_implementation_phase_36_p05_c05/
?? docs/goals/core_enhanced_refactoring_rebaselined_backlog_implementation_phase_37_p06_c04/
?? docs/goals/core_enhanced_refactoring_rebaselined_backlog_implementation_phase_38_p08_c06/
?? docs/goals/core_enhanced_refactoring_rebaselined_backlog_implementation_phase_39_p07_c03/
?? docs/goals/core_enhanced_refactoring_rebaselined_backlog_implementation_phase_40_p07_c04/
?? docs/goals/core_enhanced_refactoring_rebaselined_backlog_implementation_phase_41_p06_c06/
?? docs/goals/core_enhanced_refactoring_rebaselined_backlog_implementation_phase_42_p06_c07/
?? docs/goals/core_enhanced_refactoring_rebaselined_backlog_implementation_phase_43_p09_c07/
?? docs/goals/core_enhanced_refactoring_rebaselined_backlog_implementation_phase_44_p10_c06/
?? docs/goals/current_runtime_structural_integrity_audit/
?? docs/goals/defense_route_tactical_command_master/
?? docs/goals/defense_route_tactical_command_phase_00_policy_baseline/
?? docs/goals/defense_route_tactical_command_phase_01_withdraw_redeploy_lifecycle/
?? docs/goals/defense_route_tactical_command_phase_02_hard_cc_silence_capability/
?? docs/goals/defense_route_tactical_command_phase_03_tactical_resource_authority/
?? docs/goals/defense_route_tactical_command_phase_04_weapon_tactical_contract/
?? docs/goals/defense_route_tactical_command_phase_05_stance_tile_effects/
?? docs/goals/defense_route_tactical_command_phase_06_relocation_occupancy/
?? docs/goals/defense_route_tactical_command_phase_07_atomic_swap/
?? docs/goals/defense_route_tactical_command_phase_08_manual_skill_targeting/
?? docs/goals/defense_route_tactical_command_phase_09_field_work_integration/
?? docs/goals/game_rulebook_full_runtime_alignment_audit/
?? docs/goals/game_rulebook_policy_implementation_master/
?? docs/goals/game_rulebook_policy_implementation_master_phase_01_monolith_tuning/
?? docs/goals/game_rulebook_policy_implementation_master_phase_02_encounter_progression_v4/
?? docs/goals/game_rulebook_policy_implementation_master_phase_03_fixed_standard_and_profile/
?? docs/goals/game_rulebook_policy_implementation_master_phase_04_mastered_elite_sweep/
?? docs/goals/game_rulebook_policy_implementation_preflight/
?? docs/goals/game_rulebook_policy_implementation_readiness_audit/
?? docs/goals/game_rulebook_runtime_alignment_audit/
?? docs/goals/grant_experience_per_recipient_policy_implementation/
?? docs/goals/policy_to_implementation_manual_consolidation/
?? docs/goals/policy_to_implementation_manual_r4_correction/
?? docs/goals/policy_to_implementation_manual_state_machine_correction/
?? docs/policy_to_goal_pipeline.md
?? docs/skills/abnormality_design_notes.ko.md
?? docs/skills/abnormality_skill_fragments_design_notes.md
?? "docs/skills/\353\247\210\355\203\204\353\240\210\355\215\274\353\237\260\354\212\244.jpg"
?? docs/tmp/
?? docs/tools/
?? src/game/battle/core/boss_runtime.rs
?? src/game/battle/core/field_work.rs
?? src/game/battle/core/spawn_kernel.rs
?? src/game/battle/core/summon_runtime.rs
?? src/game/battle/core/tactical_relocation.rs
?? src/game/battle/core/tactical_stance.rs
?? src/game/battle/core/tactical_swap.rs
?? src/game/battle/core/wave_runtime.rs
?? src/game/combat_setup/boss_runtime.rs
?? src/game/data/boss_runtime_data.rs
?? src/game/data/summoned_unit_data.rs
?? src/game/encounter_progression.rs
?? src/game/resources/equipment_loadout.rs
?? src/game/world/grant_state.rs
?? src/game/world/mastered_elite_sweep.rs
?? src/game/world/tests/checkpoint_validation.rs
?? src/game/world/tests/mastered_elite_sweep.rs
?? ../game_resources/data/boss_runtime/
?? ../game_resources/data/summoned_units/
?? ../game_server/logs/app.log.2026-08-09
?? ../game_server/src/persistence/player_profile_store.rs
```
