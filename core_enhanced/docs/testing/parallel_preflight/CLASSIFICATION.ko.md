# 테스트 분류 기준과 등록표

자동 생성: `policy.json` + 검토된 `inventory.csv`. 직접 수정하지 않는다.

현재 작업 트리의 승인이나 병렬 실행 통과를 뜻하지 않는다. 실행 전 `check_inventory.py`가 통과해야 한다.

## 분류 기준

| 표시 분류 | 의미 |
| --- | --- |
| `parallel_candidate` | No known cross-test mutable resource in the reviewed source boundary; validate against a serial baseline before enabling. |
| `resource_serial` | Serialize members sharing each resource key across the applicable scope; unrelated candidates can run concurrently. |
| `isolated_process` | Exactly one selected test in a fresh test process; also obey all checkout resource keys. |
| `quiet_host` | Initial timing-sensitive validation without concurrent test/build load; still obey checkout resource keys. Conservative, not proven inherently serial. |

표시 분류와 별개로 새 프로세스·호스트 독점·모든 자원 제약을 함께 적용한다.
설정·플랫폼·완전한 시험 이름은 구성별 하네스 대조에서 확정한다. 빌드/fixture 선행 조건은 실행 묶음이 소유한다.

## 공유 자원

| 자원 키 | 적용 범위 | 근거 |
| --- | --- | --- |
| `checkout:core-debug-artifacts` | all test processes and feature configurations using the same checkout | World result transitions export deterministic run_seed/battle_uuid paths; diagnostic helpers also truncate fixed filenames. |
| `process:crossbeam-default-collector` | one test process | Local recorders still use the default crossbeam epoch collector; a pinned guard can postpone another test's reclamation gate. |
| `process:allocator-ledger` | all allocations in one test process | LIVE_BYTES/PROCESS_LEDGER and playback process windows observe unrelated allocations too. |
| `host:timer-validation` | host CPU and IO load during initial comparison | Actor tests use real 2-120ms sleeps and scheduler progress; keep their initial validation separate from throughput tests. |

## 등록표 — 소스와 조건별 묶음

개별 선언은 `inventory.csv`에서 확인한다. 새 테스트는 규칙을 상속하되 소스 변경 검토 전에는 실행 승격되지 않는다.

| 소스 | 수 | 표시 분류 | 새 프로세스 | 조용한 호스트 | 자원 키 |
| --- | ---: | --- | --- | --- | --- |
| `core_enhanced/src/application.rs` | 12 | resource_serial | False | False | checkout:core-debug-artifacts |
| `core_enhanced/src/application/fault.rs` | 1 | resource_serial | False | False | checkout:core-debug-artifacts |
| `core_enhanced/src/application/fault_tests.rs` | 19 | resource_serial | False | False | checkout:core-debug-artifacts |
| `core_enhanced/src/application/runtime_observation_tests.rs` | 11 | resource_serial | False | False | checkout:core-debug-artifacts |
| `core_enhanced/src/authoring/amount.rs` | 3 | parallel_candidate | False | False | — |
| `core_enhanced/src/authoring/clone_shape.rs` | 7 | parallel_candidate | False | False | — |
| `core_enhanced/src/authoring/control.rs` | 3 | parallel_candidate | False | False | — |
| `core_enhanced/src/authoring/diagnostic.rs` | 3 | parallel_candidate | False | False | — |
| `core_enhanced/src/authoring/output.rs` | 5 | parallel_candidate | False | False | — |
| `core_enhanced/src/authoring/projection.rs` | 1 | parallel_candidate | False | False | — |
| `core_enhanced/src/authoring/resolve.rs` | 11 | parallel_candidate | False | False | — |
| `core_enhanced/src/authoring/scenario.rs` | 4 | parallel_candidate | False | False | — |
| `core_enhanced/src/authoring/tests.rs` | 15 | resource_serial | False | False | checkout:core-debug-artifacts |
| `core_enhanced/src/authoring/trace.rs` | 3 | parallel_candidate | False | False | — |
| `core_enhanced/src/authoring/trial.rs` | 10 | resource_serial | False | False | checkout:core-debug-artifacts |
| `core_enhanced/src/authoring/trial/tests/amount_completion.rs` | 8 | resource_serial | False | False | checkout:core-debug-artifacts |
| `core_enhanced/src/authoring/trial/tests/amount_order.rs` | 1 | resource_serial | False | False | checkout:core-debug-artifacts |
| `core_enhanced/src/authoring/trial/tests/r07_copies.rs` | 2 | resource_serial | False | False | checkout:core-debug-artifacts |
| `core_enhanced/src/authoring/trial/tests/u07_objects.rs` | 1 | resource_serial | False | False | checkout:core-debug-artifacts |
| `core_enhanced/src/authoring/trial/tests/u08_relics.rs` | 4 | resource_serial | False | False | checkout:core-debug-artifacts |
| `core_enhanced/src/game/ability.rs` | 16 | parallel_candidate | False | False | — |
| `core_enhanced/src/game/ability/authoring/tests.rs` | 4 | parallel_candidate | False | False | — |
| `core_enhanced/src/game/battle/battlefield/field.rs` | 3 | parallel_candidate | False | False | — |
| `core_enhanced/src/game/battle/buffs.rs` | 4 | parallel_candidate | False | False | — |
| `core_enhanced/src/game/battle/core/boss_action_cancellation_tests.rs` | 7 | parallel_candidate | False | False | — |
| `core_enhanced/src/game/battle/core/boss_action_tests.rs` | 72 | parallel_candidate | False | False | — |
| `core_enhanced/src/game/battle/core/boss_runtime.rs` | 22 | parallel_candidate | False | False | — |
| `core_enhanced/src/game/battle/core/commands.rs` | 63 | parallel_candidate | False | False | — |
| `core_enhanced/src/game/battle/core/commands.rs` | 1 | resource_serial | False | False | checkout:core-debug-artifacts |
| `core_enhanced/src/game/battle/core/corrosion.rs` | 5 | parallel_candidate | False | False | — |
| `core_enhanced/src/game/battle/core/field_work.rs` | 10 | parallel_candidate | False | False | — |
| `core_enhanced/src/game/battle/core/ids.rs` | 3 | parallel_candidate | False | False | — |
| `core_enhanced/src/game/battle/core/independent_objects.rs` | 2 | parallel_candidate | False | False | — |
| `core_enhanced/src/game/battle/core/mod.rs` | 146 | parallel_candidate | False | False | — |
| `core_enhanced/src/game/battle/core/movement/engine.rs` | 7 | parallel_candidate | False | False | — |
| `core_enhanced/src/game/battle/core/movement/planner.rs` | 28 | parallel_candidate | False | False | — |
| `core_enhanced/src/game/battle/core/movement/rapier_backend.rs` | 21 | parallel_candidate | False | False | — |
| `core_enhanced/src/game/battle/core/movement/types.rs` | 3 | parallel_candidate | False | False | — |
| `core_enhanced/src/game/battle/core/object_creation.rs` | 18 | parallel_candidate | False | False | — |
| `core_enhanced/src/game/battle/core/projectile_math.rs` | 1 | parallel_candidate | False | False | — |
| `core_enhanced/src/game/battle/core/queue.rs` | 4 | parallel_candidate | False | False | — |
| `core_enhanced/src/game/battle/core/relic_structural_tests.rs` | 2 | parallel_candidate | False | False | — |
| `core_enhanced/src/game/battle/core/resonance_runtime/tests.rs` | 21 | parallel_candidate | False | False | — |
| `core_enhanced/src/game/battle/core/runtime_facts.rs` | 16 | parallel_candidate | False | False | — |
| `core_enhanced/src/game/battle/core/skill_runtime/projectile.rs` | 1 | parallel_candidate | False | False | — |
| `core_enhanced/src/game/battle/core/spatial.rs` | 4 | parallel_candidate | False | False | — |
| `core_enhanced/src/game/battle/core/spawn_kernel.rs` | 15 | parallel_candidate | False | False | — |
| `core_enhanced/src/game/battle/core/summon_runtime.rs` | 12 | parallel_candidate | False | False | — |
| `core_enhanced/src/game/battle/core/sustained.rs` | 1 | parallel_candidate | False | False | — |
| `core_enhanced/src/game/battle/core/tactical_stance.rs` | 14 | parallel_candidate | False | False | — |
| `core_enhanced/src/game/battle/core/targeting.rs` | 24 | parallel_candidate | False | False | — |
| `core_enhanced/src/game/battle/core/tests/amount_order.rs` | 5 | parallel_candidate | False | False | — |
| `core_enhanced/src/game/battle/core/tests/u06_starter_effects.inc.rs` | 4 | parallel_candidate | False | False | — |
| `core_enhanced/src/game/battle/core/tests/u07_block_lifecycle.inc.rs` | 12 | parallel_candidate | False | False | — |
| `core_enhanced/src/game/battle/core/tests/u07_edges.inc.rs` | 12 | parallel_candidate | False | False | — |
| `core_enhanced/src/game/battle/core/tests/u07_official.inc.rs` | 4 | parallel_candidate | False | False | — |
| `core_enhanced/src/game/battle/core/tests/u07_segmented.inc.rs` | 10 | parallel_candidate | False | False | — |
| `core_enhanced/src/game/battle/core/triggers.rs` | 1 | parallel_candidate | False | False | — |
| `core_enhanced/src/game/battle/core/types.rs` | 2 | parallel_candidate | False | False | — |
| `core_enhanced/src/game/battle/core/value_expr_runtime.rs` | 54 | parallel_candidate | False | False | — |
| `core_enhanced/src/game/battle/core/value_expr_runtime/tests/aggregate_replay.rs` | 3 | parallel_candidate | False | False | — |
| `core_enhanced/src/game/battle/core/value_expr_runtime/tests/amount_replay.rs` | 5 | parallel_candidate | False | False | — |
| `core_enhanced/src/game/battle/core/value_expr_runtime/tests/completion.rs` | 9 | parallel_candidate | False | False | — |
| `core_enhanced/src/game/battle/core/value_expr_runtime/tests/legacy_amount.rs` | 2 | parallel_candidate | False | False | — |
| `core_enhanced/src/game/battle/core/value_expr_runtime/tests/relic_amount.rs` | 12 | parallel_candidate | False | False | — |
| `core_enhanced/src/game/battle/core/wave_runtime.rs` | 10 | parallel_candidate | False | False | — |
| `core_enhanced/src/game/battle/damage.rs` | 22 | parallel_candidate | False | False | — |
| `core_enhanced/src/game/battle/enums.rs` | 6 | parallel_candidate | False | False | — |
| `core_enhanced/src/game/battle/event_log.rs` | 13 | parallel_candidate | False | False | — |
| `core_enhanced/src/game/battle/result_stats.rs` | 7 | parallel_candidate | False | False | — |
| `core_enhanced/src/game/battle/runtime_control.rs` | 8 | parallel_candidate | False | False | — |
| `core_enhanced/src/game/battle/runtime_facts.rs` | 4 | parallel_candidate | False | False | — |
| `core_enhanced/src/game/battle/scenario.rs` | 6 | parallel_candidate | False | False | — |
| `core_enhanced/src/game/battle/stat_pipeline.rs` | 6 | parallel_candidate | False | False | — |
| `core_enhanced/src/game/battle/stat_pipeline/tests/r06_stats.rs` | 4 | parallel_candidate | False | False | — |
| `core_enhanced/src/game/battle/tile_range.rs` | 4 | parallel_candidate | False | False | — |
| `core_enhanced/src/game/battle/types.rs` | 9 | parallel_candidate | False | False | — |
| `core_enhanced/src/game/behavior.rs` | 12 | parallel_candidate | False | False | — |
| `core_enhanced/src/game/boss_omen.rs` | 2 | parallel_candidate | False | False | — |
| `core_enhanced/src/game/combat_power.rs` | 5 | parallel_candidate | False | False | — |
| `core_enhanced/src/game/combat_preview/mod.rs` | 28 | parallel_candidate | False | False | — |
| `core_enhanced/src/game/combat_setup/battlefield_plan.rs` | 1 | parallel_candidate | False | False | — |
| `core_enhanced/src/game/combat_setup/boss_runtime.rs` | 3 | parallel_candidate | False | False | — |
| `core_enhanced/src/game/combat_setup/enemy_spawns.rs` | 1 | parallel_candidate | False | False | — |
| `core_enhanced/src/game/combat_setup/mission_policy.rs` | 3 | parallel_candidate | False | False | — |
| `core_enhanced/src/game/combat_setup/rewards.rs` | 5 | parallel_candidate | False | False | — |
| `core_enhanced/src/game/data/abnormality_data.rs` | 5 | parallel_candidate | False | False | — |
| `core_enhanced/src/game/data/boss_omen_data.rs` | 3 | parallel_candidate | False | False | — |
| `core_enhanced/src/game/data/boss_runtime_data.rs` | 11 | parallel_candidate | False | False | — |
| `core_enhanced/src/game/data/consumable_data.rs` | 6 | parallel_candidate | False | False | — |
| `core_enhanced/src/game/data/content_pack/canonical.rs` | 3 | parallel_candidate | False | False | — |
| `core_enhanced/src/game/data/content_pack/diagnostic_tests.rs` | 6 | parallel_candidate | False | False | — |
| `core_enhanced/src/game/data/content_pack/packaging.rs` | 3 | parallel_candidate | False | False | — |
| `core_enhanced/src/game/data/content_pack/raw/tests.rs` | 5 | parallel_candidate | False | False | — |
| `core_enhanced/src/game/data/content_pack/resolver/tests.rs` | 8 | parallel_candidate | False | False | — |
| `core_enhanced/src/game/data/content_pack/snapshot/tests.rs` | 11 | parallel_candidate | False | False | — |
| `core_enhanced/src/game/data/content_pack/tests.rs` | 23 | resource_serial | False | False | checkout:core-debug-artifacts |
| `core_enhanced/src/game/data/corroded_employee_data.rs` | 6 | parallel_candidate | False | False | — |
| `core_enhanced/src/game/data/corroded_wave_data.rs` | 2 | parallel_candidate | False | False | — |
| `core_enhanced/src/game/data/employee_data.rs` | 4 | parallel_candidate | False | False | — |
| `core_enhanced/src/game/data/equipment_data.rs` | 9 | parallel_candidate | False | False | — |
| `core_enhanced/src/game/data/event_data.rs` | 8 | parallel_candidate | False | False | — |
| `core_enhanced/src/game/data/independent_object_data.rs` | 5 | parallel_candidate | False | False | — |
| `core_enhanced/src/game/data/live_ron.rs` | 82 | parallel_candidate | False | False | — |
| `core_enhanced/src/game/data/mod.rs` | 14 | parallel_candidate | False | False | — |
| `core_enhanced/src/game/data/pve_data.rs` | 15 | parallel_candidate | False | False | — |
| `core_enhanced/src/game/data/reward_data.rs` | 2 | parallel_candidate | False | False | — |
| `core_enhanced/src/game/data/run_policy_data.rs` | 19 | parallel_candidate | False | False | — |
| `core_enhanced/src/game/data/shop_data.rs` | 4 | parallel_candidate | False | False | — |
| `core_enhanced/src/game/data/skill_data.rs` | 14 | parallel_candidate | False | False | — |
| `core_enhanced/src/game/data/skill_fragment_data.rs` | 6 | parallel_candidate | False | False | — |
| `core_enhanced/src/game/data/summoned_unit_data.rs` | 3 | parallel_candidate | False | False | — |
| `core_enhanced/src/game/data/validation.rs` | 2 | parallel_candidate | False | False | — |
| `core_enhanced/src/game/data/visual_novel_data.rs` | 3 | parallel_candidate | False | False | — |
| `core_enhanced/src/game/data/visual_novel_trigger_data.rs` | 4 | parallel_candidate | False | False | — |
| `core_enhanced/src/game/determinism.rs` | 3 | parallel_candidate | False | False | — |
| `core_enhanced/src/game/employee.rs` | 19 | parallel_candidate | False | False | — |
| `core_enhanced/src/game/employee_psychology.rs` | 1 | parallel_candidate | False | False | — |
| `core_enhanced/src/game/encounter_compendium.rs` | 5 | parallel_candidate | False | False | — |
| `core_enhanced/src/game/encounter_progression.rs` | 5 | parallel_candidate | False | False | — |
| `core_enhanced/src/game/events/combat.rs` | 14 | parallel_candidate | False | False | — |
| `core_enhanced/src/game/managers/uuid_manager.rs` | 2 | parallel_candidate | False | False | — |
| `core_enhanced/src/game/map/executor.rs` | 1 | parallel_candidate | False | False | — |
| `core_enhanced/src/game/map/facility.rs` | 3 | parallel_candidate | False | False | — |
| `core_enhanced/src/game/map/facility_generator.rs` | 8 | parallel_candidate | False | False | — |
| `core_enhanced/src/game/map/progression.rs` | 11 | parallel_candidate | False | False | — |
| `core_enhanced/src/game/map/session.rs` | 1 | parallel_candidate | False | False | — |
| `core_enhanced/src/game/map/types.rs` | 2 | parallel_candidate | False | False | — |
| `core_enhanced/src/game/monolith.rs` | 7 | parallel_candidate | False | False | — |
| `core_enhanced/src/game/range_preview.rs` | 13 | parallel_candidate | False | False | — |
| `core_enhanced/src/game/recruitment.rs` | 3 | parallel_candidate | False | False | — |
| `core_enhanced/src/game/relic/allocation.rs` | 2 | parallel_candidate | False | False | — |
| `core_enhanced/src/game/relic/numeric.rs` | 1 | parallel_candidate | False | False | — |
| `core_enhanced/src/game/relic/structural/tests.rs` | 3 | parallel_candidate | False | False | — |
| `core_enhanced/src/game/relic/structural/tests/compiler_cases.rs` | 9 | parallel_candidate | False | False | — |
| `core_enhanced/src/game/relic/structural/tests/storage_cases.rs` | 2 | parallel_candidate | False | False | — |
| `core_enhanced/src/game/relic/tests.rs` | 6 | parallel_candidate | False | False | — |
| `core_enhanced/src/game/resource/mod.rs` | 3 | parallel_candidate | False | False | — |
| `core_enhanced/src/game/resource/store/tests.rs` | 24 | parallel_candidate | False | False | — |
| `core_enhanced/src/game/resources/inventory.rs` | 6 | parallel_candidate | False | False | — |
| `core_enhanced/src/game/resources/item_slot.rs` | 6 | parallel_candidate | False | False | — |
| `core_enhanced/src/game/reward.rs` | 2 | parallel_candidate | False | False | — |
| `core_enhanced/src/game/reward_policy.rs` | 3 | parallel_candidate | False | False | — |
| `core_enhanced/src/game/stats.rs` | 4 | parallel_candidate | False | False | — |
| `core_enhanced/src/game/value_expr/catalog.rs` | 8 | parallel_candidate | False | False | — |
| `core_enhanced/src/game/value_expr/parse.rs` | 8 | parallel_candidate | False | False | — |
| `core_enhanced/src/game/value_expr/tests.rs` | 27 | parallel_candidate | False | False | — |
| `core_enhanced/src/game/wave_resolution.rs` | 2 | parallel_candidate | False | False | — |
| `core_enhanced/src/game/world/active_run/continuation.rs` | 1 | resource_serial | False | False | checkout:core-debug-artifacts |
| `core_enhanced/src/game/world/admin/tests.rs` | 17 | resource_serial | False | False | checkout:core-debug-artifacts |
| `core_enhanced/src/game/world/application.rs` | 3 | resource_serial | False | False | checkout:core-debug-artifacts |
| `core_enhanced/src/game/world/event_node.rs` | 5 | resource_serial | False | False | checkout:core-debug-artifacts |
| `core_enhanced/src/game/world/map_encounters.rs` | 2 | resource_serial | False | False | checkout:core-debug-artifacts |
| `core_enhanced/src/game/world/node_flow.rs` | 1 | resource_serial | False | False | checkout:core-debug-artifacts |
| `core_enhanced/src/game/world/shop.rs` | 7 | resource_serial | False | False | checkout:core-debug-artifacts |
| `core_enhanced/src/game/world/snapshot.rs` | 2 | resource_serial | False | False | checkout:core-debug-artifacts |
| `core_enhanced/src/game/world/state.rs` | 7 | resource_serial | False | False | checkout:core-debug-artifacts |
| `core_enhanced/src/game/world/tests/checkpoint_validation.rs` | 23 | resource_serial | False | False | checkout:core-debug-artifacts |
| `core_enhanced/src/game/world/tests/combat.rs` | 117 | resource_serial | False | False | checkout:core-debug-artifacts |
| `core_enhanced/src/game/world/tests/content_generation.rs` | 3 | resource_serial | False | False | checkout:core-debug-artifacts |
| `core_enhanced/src/game/world/tests/equipment.rs` | 31 | resource_serial | False | False | checkout:core-debug-artifacts |
| `core_enhanced/src/game/world/tests/facility.rs` | 12 | resource_serial | False | False | checkout:core-debug-artifacts |
| `core_enhanced/src/game/world/tests/map_flow.rs` | 38 | resource_serial | False | False | checkout:core-debug-artifacts |
| `core_enhanced/src/game/world/tests/mastered_elite_sweep.rs` | 7 | resource_serial | False | False | checkout:core-debug-artifacts |
| `core_enhanced/src/game/world/tests/node_sessions.rs` | 32 | resource_serial | False | False | checkout:core-debug-artifacts |
| `core_enhanced/src/game/world/tests/r07_copies.rs` | 13 | resource_serial | False | False | checkout:core-debug-artifacts |
| `core_enhanced/src/game/world/tests/r09_combat.inc.rs` | 11 | resource_serial | False | False | checkout:core-debug-artifacts |
| `core_enhanced/src/game/world/tests/resource_lifetimes.rs` | 19 | resource_serial | False | False | checkout:core-debug-artifacts |
| `core_enhanced/src/game/world/tests/snapshots_and_start.rs` | 24 | resource_serial | False | False | checkout:core-debug-artifacts |
| `core_enhanced/src/game/world/tests/u04_continuation.inc.rs` | 13 | resource_serial | False | False | checkout:core-debug-artifacts |
| `core_enhanced/src/game/world/tests/u06_interaction.inc.rs` | 4 | resource_serial | False | False | checkout:core-debug-artifacts |
| `core_enhanced/src/game/world/tests/u06_starter.rs` | 6 | resource_serial | False | False | checkout:core-debug-artifacts |
| `core_enhanced/src/game/world/tests/u07_objects.rs` | 1 | resource_serial | False | False | checkout:core-debug-artifacts |
| `core_enhanced/src/game/world/tests/u08_carriers.rs` | 12 | resource_serial | False | False | checkout:core-debug-artifacts |
| `core_enhanced/src/game/world/tests/u08_continue_owner.rs` | 2 | resource_serial | False | False | checkout:core-debug-artifacts |
| `core_enhanced/src/game/world/tests/u08_event_owner.rs` | 4 | resource_serial | False | False | checkout:core-debug-artifacts |
| `core_enhanced/src/game/world/tests/u08_lifecycle.rs` | 4 | resource_serial | False | False | checkout:core-debug-artifacts |
| `core_enhanced/src/game/world/tests/u08_query_owner.rs` | 3 | resource_serial | False | False | checkout:core-debug-artifacts |
| `core_enhanced/src/game/world/tests/u08_relics.rs` | 15 | resource_serial | False | False | checkout:core-debug-artifacts |
| `core_enhanced/src/game/world/tests/u09_structural.rs` | 1 | resource_serial | False | False | checkout:core-debug-artifacts |
| `core_enhanced/src/game/world/tests/u09_structural/flow_cases.rs` | 3 | resource_serial | False | False | checkout:core-debug-artifacts |
| `core_enhanced/src/game/world/tests/u09_structural/flow_cases/cancellation_cases.rs` | 4 | resource_serial | False | False | checkout:core-debug-artifacts |
| `core_enhanced/src/game/world/tests/u09_structural/flow_cases/footprint_cases.rs` | 2 | resource_serial | False | False | checkout:core-debug-artifacts |
| `core_enhanced/src/game/world/tests/u09_structural/flow_cases/isolation_cases.rs` | 2 | resource_serial | False | False | checkout:core-debug-artifacts |
| `core_enhanced/src/game/world/tests/u09_structural/flow_cases/lifetime_cases.rs` | 2 | resource_serial | False | False | checkout:core-debug-artifacts |
| `core_enhanced/src/game/world/tests/u09_structural/flow_cases/personal_cases.rs` | 9 | resource_serial | False | False | checkout:core-debug-artifacts |
| `core_enhanced/src/game/world/tests/u09_structural/flow_cases/resource_cases.rs` | 1 | resource_serial | False | False | checkout:core-debug-artifacts |
| `core_enhanced/src/game/world/tests/u09_structural/flow_cases/selected_cases.rs` | 3 | resource_serial | False | False | checkout:core-debug-artifacts |
| `core_enhanced/src/game/world/tests/u09_structural/flow_cases/support_cases.rs` | 4 | resource_serial | False | False | checkout:core-debug-artifacts |
| `core_enhanced/src/game/world/tests/u09_structural/publication_cases.rs` | 4 | resource_serial | False | False | checkout:core-debug-artifacts |
| `core_enhanced/src/game/world/tests/u09_structural/world_state_cases.rs` | 2 | resource_serial | False | False | checkout:core-debug-artifacts |
| `core_enhanced/src/owned_collections.rs` | 5 | parallel_candidate | False | False | — |
| `core_enhanced/src/runtime_observation/identity.rs` | 2 | parallel_candidate | False | False | — |
| `core_enhanced/src/runtime_safety/tests.rs` | 15 | resource_serial | False | False | checkout:core-debug-artifacts |
| `core_enhanced/tests/author_content_cli_arguments.rs` | 4 | parallel_candidate | False | False | — |
| `core_enhanced/tests/battle_showcase.rs` | 1 | resource_serial | False | False | checkout:core-debug-artifacts |
| `core_enhanced/tests/live_item_skill_activation.rs` | 4 | parallel_candidate | False | False | — |
| `core_enhanced/tests/live_skill_catalog_audit.rs` | 4 | parallel_candidate | False | False | — |
| `core_enhanced/tests/ron_loading.rs` | 25 | parallel_candidate | False | False | — |
| `core_enhanced/tests/skill_runtime_contract.rs` | 15 | parallel_candidate | False | False | — |
| `core_enhanced/tests/skill_test/little_red.rs` | 1 | resource_serial | False | False | checkout:core-debug-artifacts |
| `core_enhanced/tests/skill_test/melting_love.rs` | 1 | resource_serial | False | False | checkout:core-debug-artifacts |
| `core_enhanced/tests/skill_test/nothing_there.rs` | 1 | resource_serial | False | False | checkout:core-debug-artifacts |
| `core_enhanced/tests/skill_test/white_night.rs` | 1 | resource_serial | False | False | checkout:core-debug-artifacts |
| `core_enhanced/tests/u08_relic_public/event.rs` | 3 | resource_serial | False | False | checkout:core-debug-artifacts |
| `core_enhanced/tests/u08_relic_public/lifecycle.rs` | 3 | resource_serial | False | False | checkout:core-debug-artifacts |
| `core_enhanced/tests/u08_relic_public/prospective.rs` | 7 | resource_serial | False | False | checkout:core-debug-artifacts |
| `core_enhanced/tests/u08_relic_public/sell.rs` | 3 | resource_serial | False | False | checkout:core-debug-artifacts |
| `game_server/src/bootstrap.rs` | 2 | resource_serial | False | False | checkout:core-debug-artifacts |
| `game_server/src/env.rs` | 2 | parallel_candidate | False | False | — |
| `game_server/src/game/load_balance_actor/handlers.rs` | 2 | resource_serial | False | False | checkout:core-debug-artifacts |
| `game_server/src/game/player_game_actor/handlers.rs` | 25 | quiet_host | False | True | checkout:core-debug-artifacts; host:timer-validation |
| `game_server/src/game/player_game_actor/handlers/tests/durable.rs` | 8 | quiet_host | False | True | checkout:core-debug-artifacts; host:timer-validation |
| `game_server/src/game/player_game_actor/handlers/tests/fault.rs` | 4 | quiet_host | False | True | checkout:core-debug-artifacts; host:timer-validation |
| `game_server/src/game/player_game_actor/handlers/tests/u06.rs` | 1 | resource_serial | False | False | checkout:core-debug-artifacts |
| `game_server/src/game/player_game_actor/messages.rs` | 9 | resource_serial | False | False | checkout:core-debug-artifacts |
| `game_server/src/game/player_game_actor/session.rs` | 3 | resource_serial | False | False | checkout:core-debug-artifacts |
| `game_server/src/game/player_game_actor/state.rs` | 12 | resource_serial | False | False | checkout:core-debug-artifacts |
| `game_server/src/observability/tests.rs` | 1 | isolated_process | True | False | checkout:core-debug-artifacts; process:allocator-ledger; process:crossbeam-default-collector |
| `game_server/src/observability/tests.rs` | 9 | resource_serial | False | False | checkout:core-debug-artifacts; process:crossbeam-default-collector |
| `game_server/src/persistence/active_run_store.rs` | 13 | resource_serial | False | False | checkout:core-debug-artifacts |
| `game_server/src/persistence/player_profile_store.rs` | 11 | resource_serial | False | False | checkout:core-debug-artifacts |
| `game_server/src/persistence/player_profile_store/application_intent.rs` | 4 | resource_serial | False | False | checkout:core-debug-artifacts |
| `game_server/src/runtime_measurement/allocator.rs` | 8 | parallel_candidate | False | False | — |
| `game_server/src/runtime_measurement/playback.rs` | 3 | isolated_process | True | False | checkout:core-debug-artifacts; process:allocator-ledger; process:crossbeam-default-collector |
| `game_server/src/runtime_measurement/playback.rs` | 5 | resource_serial | False | False | checkout:core-debug-artifacts; process:crossbeam-default-collector |
| `game_server/src/runtime_measurement/playback/script.rs` | 2 | parallel_candidate | False | False | — |
| `game_server/src/runtime_measurement/rss.rs` | 1 | parallel_candidate | False | False | — |

## 하네스 밖의 등록된 측정 작업

| 소스 | 분류 | 자원 | 근거 |
| --- | --- | --- | --- |
| `game_server/src/bin/measure_runtime.rs` | quiet_host | process:allocator-ledger; checkout:core-debug-artifacts | Performance measurement requires a fresh process, stable inputs and no simultaneous build/test load; not included in declaration counts. |
| `game_server/examples/observe_runtime.rs` | quiet_host | checkout:core-debug-artifacts | Observation scenario measures elapsed wall time and sleeps; reserve an independent performance/observation validation interval. |
| `game_server/examples/fault_runtime.rs` | quiet_host | checkout:core-debug-artifacts | Fault scenario depends on scheduler progress and real sleeps; preserve its existing execution conditions. |
