# Experiments

## Log

| Date | Scope | Attempt | Result | Follow-up |
| --- | --- | --- | --- | --- |
| 2026-06-22 | Goal setup | Defined static data/map/scenario implementation scope. | Not started. | Begin with loader and live RON inventory. |
| 2026-06-22 | Map node tags | Removed `MapNodeDefinition.tags` and deleted `tags` entries from live `map/node_definitions.ron`. Added `deny_unknown_fields` to reject legacy tag-bearing node definitions instead of silently accepting them. | Success. `cargo test builtin_node_definitions_validate_contract --lib` passed. | Continue with the remaining static data/map/scenario policies. |
| 2026-06-22 | NodeSessionKind | Removed derived `NodeSessionKind`, removed `session_kind` from `NodeSession`, and switched headquarters/session tests to use canonical `MapNodeCategory`. | Success. Focused session/executor tests passed. | Continue checking broader compile/test surfaces before completing this subgoal. |
| 2026-06-22 | Static data/map compile surface | Ran full `cargo check` after map tag and `NodeSessionKind` removal. | Success. Core compiled with only the pre-existing workspace manifest warning. | Continue with remaining static data/map/scenario policies. |
| 2026-06-22 | RUN_SYSTEM_POLICY data source | Added integrated live `game_resources/data/run/policy.ron`, `RunPolicyData`, validation, and `GameDataBase.run_policy`. Removed the runtime `RUN_SYSTEM_POLICY` code constant and switched world setup, support, headquarters, post-battle, and live deployment flows to `GameCore::run_policy()`. | Success. Focused run policy, live RON loader, world start, headquarters, support, post-battle, combat start, deployment-cost tests, and `cargo check` passed. | Continue with remaining static data/map/scenario policies. |
| 2026-06-22 | MapViewDto progression projection | Read DTO producer/consumer usage before editing. | Policy-decision completion. The confirmed policy requires reducing duplicated id-list projection, but the exact Unity/server-facing DTO shape had to be chosen before implementation. | User later confirmed option 1. |
| 2026-06-22 | MapViewDto id-list removal | Removed `available_node_ids` and `completed_node_ids` from `MapViewDto`, kept internal `MapProgression` lists, and updated core/server `MapViewDto` consumers to derive availability/completion from `MapNodeDto.state`. | Success. Focused map flow/snapshot tests, RON/map validation tests, `cargo check`, and `cargo check -p game_server` passed. | Continue remaining static data/map/scenario policies. |
| 2026-06-22 | Authored defense routes | Removed generated Defense route fallback and combat-preview empty-wave fallback. Added authored `defense_main` routes to live battlefield templates, required Defense waves to declare `route_id`, and updated live PVE encounters/tests to reference authored routes. | Success. Combat preview/event tests, live RON PVE tests, `cargo test --lib`, `cargo check`, and `cargo check -p game_server` passed. | Continue remaining static data/map/scenario policies. |
| 2026-06-22 | MapNode state SoT follow-up | Broad lib tests exposed stale sibling `MapNodeState::Available` after selecting one fork choice. Updated `MapProgression::enter_node` to mark unselected previously available siblings as `Revealed`, making `MapNodeDto.state` match selectable availability. | Success. Focused map progression tests and full `cargo test --lib` passed. | Keep DTO state as the Unity/server-facing availability source. |
| 2026-06-22 | Starter employee loadout source | Added `StarterEmployeeLoadout` to starter candidate live data, moved the starter basic attack fragment into live `skill_fragments/base.ron`, gave all live starter candidates explicit `standard_armor` plus baseline fragment ids, and made start-game roster initialization create owned/equipped starter equipment from candidate data. Production live loaders now use RON skill fragments directly instead of `with_builtin_starter`; the old helper is `cfg(test)` fixture-only. | Success. Focused starter/snapshot/equipment tests, live RON PVE tests, single-thread full lib tests, and `cargo check -p game_server` passed. | Continue remaining static data/map/scenario policies. |
| 2026-06-22 | Remove legacy random event RON | Confirmed official runtime/test/server loaders do not reference legacy random event files, then deleted legacy event pools, random events, random-event reward/shop files, and the legacy random-event abnormality file from live data. | Success. Remaining references are historical analysis docs only; live RON PVE tests and `cargo check -p game_server` passed. | Continue remaining static data/map/scenario policies. |
| 2026-06-22 | FacilityEntity future schema | Checked current PVE validation and live RON audit paths for `FacilityEntity`. | Already implemented. `GameDataBase` reference validation panics on any manual `FacilityEntity` PVE wave, and `tests/ron_loading.rs` also rejects it in live RON audit. | Record as satisfied unless a future facility schema policy is opened. |
| 2026-06-22 | Battlefield archetype random selection source / SplitRoom | Added `enabled` and `weight` to live `map/battlefield_archetypes.ron`. Replaced hard-coded `seed % 6` fallback with weighted enabled-archetype selection from RON. Validation rejects enabled `SplitRoom`, enabled random archetypes with zero weight, and a policy with no enabled non-boss random weight. | Success. Focused battlefield policy/generation tests and live RON scenario preview test passed. | Continue static subgoal completion audit. |
| 2026-06-22 | Server startup live preview validation | Connected `game_data.validate_generated_combat_preview_contracts()` to the official `game_server` RON loader after `GameDataBuilder::build_arc()`. | Success. `cargo check -p game_server` passed. | Continue static subgoal completion audit. |
| 2026-06-22 | battle_records export | Read `RunState::record_battle`, `battle_record_root_dir`, `battle_record_path`, and runtime accessors. | Already implemented. Battle records are always written under `battle_records/run_<seed>/*.json`, kept in memory for debug/test access, and no runtime/server/live-data path reads them back as gameplay SoT. | Record as satisfied; keep final validation focused on affected map/static data code. |
| 2026-06-22 | Map authoring/generation policy source | Added live `map/generation_policy.ron` and `MapGenerationPolicyData`. Moved early-depth category weights, pre-boss category weights, safe replacement categories, max combat per row divisor, and pre-boss support repair from generator constants to RON. | Success. Focused generation policy, generated row, generated node, and live map content RON audit tests passed; `cargo check --lib` and `cargo check -p game_server` passed. | Run final broad validation and mark the static subgoal complete if no regressions appear. |
| 2026-06-22 | Static data/map/scenario final validation | Ran broad single-thread lib tests and full RON loading audit after all static subgoal changes. | Success. `cargo test --lib -- --test-threads=1` passed 498 tests, and `cargo test --test ron_loading` passed 17 tests. | Static data/map/scenario subgoal can be treated as complete for the master sequence. |

## Failed Approaches

| Date | Scope | Attempt | Failure | Correction |
| --- | --- | --- | --- | --- |
| 2026-06-22 | Map node tags | Ran `cargo test map_node_definition --lib` as an initial focused check. | The command compiled successfully but matched 0 tests, so it did not prove live node definition loading. | Re-ran the concrete builtin node definition validation test. |
| 2026-06-22 | NodeSessionKind | Ran `cargo test node_session_preserves_identity_routing_category_and_payload enter_creates_a_session_without_interpreting_map_progression --lib`. | `cargo test` accepts one test-name filter before `--`, so the second test name was rejected as an unexpected argument. | Ran both focused tests as separate commands. |
| 2026-06-22 | MapViewDto id-list removal | Ran `cargo test completing_entered_node_returns_to_map_and_advances_progression --lib`. | The command compiled successfully but matched 0 tests because the test name was stale. | Re-ran `cargo test map_progression_selects_completes_and_unlocks_next_nodes --lib`. |
| 2026-06-22 | Authored defense routes | Ran `cargo test load_game_data_from_ron_generates_battlefield_preview --test ron_loading`. | The command compiled successfully but matched 0 tests because the test name was stale. | Re-ran `cargo test live_pve_scenario_authoring_contracts_drive_preview_data --test ron_loading`. |
| 2026-06-22 | Authored defense routes | Ran `cargo test every_authored_battlefield_template_satisfies_instance_contract --lib` after removing generated routes. | Failed because the old test forced `BossArena` through the Defense route contract. | Updated the test to skip `BossArena` as non-Defense template coverage. |
| 2026-06-22 | Authored defense routes | Ran focused combat preview/event tests after requiring `route_id`. | Several test fixtures still relied on implicit Defense route fallback or encounter-less preview generation. | Updated fixtures to use authored encounters and explicit `defense_main` route ids. |
| 2026-06-22 | MapNode state SoT follow-up | Ran `cargo test --lib`. | Failed at `boss_completion_advances_acts_until_run_complete` because stale unselected fork siblings still had `MapNodeState::Available` after selection. | Updated `MapProgression::enter_node` to mark unselected previously available nodes as `Revealed`, then reran focused tests and full lib tests. |
| 2026-06-22 | Starter employee loadout source | Ran `cargo test starter --lib` after adding starter loadout validation. | Failed because world test starter candidates still had empty loadouts and later because the test builder lacked starter equipment/fragment data. | Updated starter test candidates and default test GameData fixtures to include explicit starter armor and starter basic attack fragment metadata. |
| 2026-06-22 | Starter employee loadout source | Ran `cargo test --lib` after moving starter loadout to data. | Failed in stat/equipment tests that assumed `Employee::new` or start-game employees implicitly had no data-driven baseline equipment/fragment. | Made stat tests explicitly attach baseline fragments, and updated equipment/snapshot tests to expect `standard_armor` to remain equipped from starter live data. |
| 2026-06-22 | Starter employee loadout source | Ran parallel full lib test after fixes. | One battle-record JSON test failed with EOF while reading a shared output file, but the same test passed alone. | Re-ran full lib tests with `--test-threads=1`; all tests passed, so this was recorded as a parallel file-output collision rather than a policy regression. |
| 2026-06-22 | Remove legacy random event RON | Ran `rg` for deleted legacy file names after deletion. | Only historical documentation/audit references remained; no runtime, test loader, server, or live data references remained. | Kept historical notes intact and updated the active policy decision target list to include the deleted legacy abnormality file. |
| 2026-06-22 | Battlefield archetype random selection source | Replaced the fixed `seed % 6` table with weighted deterministic selection. | The old `generator_covers_all_combat_archetypes_with_valid_instances` test depended on exactly six seeds covering six archetypes. That assertion was too coupled to the old table. | Updated the test to sample a wider deterministic seed range while still proving all six enabled random archetypes appear and `SplitRoom`/`BossArena` do not. |

## Validation Commands

- `cargo test map_node_definition --lib` - compiled successfully, but 0 tests matched.
- `cargo test builtin_node_definitions_validate_contract --lib` - passed, 1 test.
- `cargo test node_session_preserves_identity_routing_category_and_payload --lib` - passed, 1 test.
- `cargo test enter_creates_a_session_without_interpreting_map_progression --lib` - passed, 1 test.
- `cargo check` - passed.
- `cargo test builtin_run_policy_validates_contract --lib` - passed, 1 test.
- `cargo test run_policy_rejects_legacy_or_unknown_fields --lib` - passed, 1 test.
- `cargo test load_game_data_from_ron_reads_run_policy --test ron_loading` - passed, 1 test.
- `cargo test start_new_game_creates_starter_employee_roster_and_order --lib` - passed, 1 test.
- `cargo test headquarters_emergency_supplies_completes_node_without_shop_or_recruitment --lib` - passed, 1 test.
- `cargo test medical_support_node_emergency_care_restores_hp --lib` - passed, 1 test.
- `cargo test post_battle_resolution_applies_employee_incapacitation_trauma --lib` - passed, 1 test.
- `cargo test combat_node_confirm_starts_live_battle --lib` - passed, 1 test.
- `cargo test deploy_cost_reduction_consumable_reduces_live_deployment_cost_for_employee --lib` - passed, 1 test.
- `cargo check` - passed after RUN_SYSTEM_POLICY migration.
- `cargo test start_new_game_exposes_initial_map_without_entering_a_node_session --lib` - passed, 1 test.
- `cargo test completing_entered_node_returns_to_map_and_advances_progression --lib` - compiled successfully, but 0 tests matched.
- `cargo test map_progression_selects_completes_and_unlocks_next_nodes --lib` - passed, 1 test.
- `cargo test run_snapshot_exposes_current_flow_after_start --lib` - passed, 1 test.
- `cargo test load_game_data_from_ron_reads_run_policy --test ron_loading` - passed after MapViewDto removal, 1 test.
- `cargo test builtin_node_definitions_validate_contract --lib` - passed after MapViewDto removal, 1 test.
- `cargo check` - passed after MapViewDto removal.
- `cargo check -p game_server` - passed after MapViewDto removal.
- `cargo test battlefield_template_database_selects_seeded_variants --lib` - passed, 1 test.
- `cargo test every_authored_battlefield_template_satisfies_instance_contract --lib` - initially failed for `BossArena` Defense coverage; passed after test correction.
- `cargo test authored_defense_route_for_non_rectangular_corridor_is_contiguous --lib` - passed, 1 test.
- `cargo test load_game_data_from_ron_generates_battlefield_preview --test ron_loading` - compiled successfully, but 0 tests matched.
- `cargo test live_pve_scenario_authoring_contracts_drive_preview_data --test ron_loading` - passed, 1 test.
- `cargo test live_pve_references_resolve --test ron_loading` - passed, 1 test.
- `cargo test generated_corroded_wave_source_resolves_during_preview_generation --lib` - initially failed on missing Defense `route_id`; passed after fixture update.
- `cargo test combat_preview_serializes_typed_threat_warnings --lib` - passed, 1 test.
- `cargo test authored_defense_route_enemy_progresses_past_spawn_boundary --lib` - initially failed on missing Defense `route_id` and then route/object placement overlap; passed after fixture and placement update.
- `cargo test combat_preview_drives_actual_map_battle_start_layout --lib` - initially failed on missing fallback Defense `route_id`; passed after explicit route update.
- `cargo test combat_preview --lib` - initially failed on encounter-less preview tests; passed after authored encounter fixture updates, 27 tests.
- `cargo test game::events::combat --lib` - initially failed on two route-less fallback Defense fixtures; passed after explicit route updates, 13 tests.
- `cargo test pve --test ron_loading` - passed, 3 tests.
- `cargo check` - passed after authored Defense route implementation.
- `cargo check -p game_server` - passed after authored Defense route implementation.
- `cargo test boss_completion_advances_acts_until_run_complete --lib` - initially failed on stale `MapNodeState::Available`; passed after `MapProgression::enter_node` state SoT fix.
- `cargo test entering_and_completing_node_reveals_only_next_choices --lib` - passed, 1 test.
- `cargo test map_progression_selects_completes_and_unlocks_next_nodes --lib` - passed after state SoT fix, 1 test.
- `cargo test --lib` - passed, 497 tests.
- `cargo check -p game_server` - passed after final state SoT fix.
- `cargo check --lib` - passed after starter loadout schema/runtime implementation.
- `cargo test pve --test ron_loading` - passed after starter live RON changes, 3 tests.
- `cargo test starter --lib` - initially failed on incomplete starter test fixture data; passed after fixture updates, 3 tests.
- `cargo test employee_roster_snapshot --lib` - passed after snapshot expectations were updated for starter equipment, 2 tests.
- `cargo check -p game_server` - passed after removing production `with_builtin_starter` loader usage.
- `cargo test game::world::tests::equipment:: --lib` - initially failed on old empty-loadout assumptions; passed after updating equipment expectations, 20 tests.
- `cargo test retreat_from_live_defense_battle_reenters_until_attempts_are_exhausted --lib` - passed alone after parallel full-lib EOF failure.
- `cargo test --lib -- --test-threads=1` - passed, 497 tests.
- `cargo test pve --test ron_loading` - passed after final starter loadout implementation, 3 tests.
- `cargo check -p game_server` - passed after final starter loadout implementation.
- `rg -n "legacy_random_event_abnormalities|legacy_random_events|legacy_event_pools|legacy_random_event_rewards|legacy_random_event_shops" src tests ../game_server ../game_resources/data docs -S` - no runtime/test/server/live-data references remained after deletion; historical docs still mention the old files.
- `cargo test pve --test ron_loading` - passed after legacy random event RON deletion, 3 tests.
- `cargo check -p game_server` - passed after legacy random event RON deletion.
- `cargo test battlefield_generation_policy_is_loaded_from_shared_ron --lib` - passed after adding `enabled`/`weight` policy fields, 1 test.
- `cargo test generator_covers_all_combat_archetypes_with_valid_instances --lib` - passed after weighted RON selection, 1 test.
- `cargo test live_pve_scenario_authoring_contracts_drive_preview_data --test ron_loading` - passed after weighted RON selection, 1 test.
- `cargo check -p game_server` - passed after server startup preview validation hookup.
- `cargo test builtin_generation_policy_validate_contract --lib` - passed after adding live map generation policy, 1 test.
- `cargo test live_map_content_pools_are_safe_and_resolve --test ron_loading` - passed after adding generation policy/live node definition cross-checks, 1 test.
- `cargo test generated_rows_are_not_combat_only_corridors --lib` - passed after row repair policy moved to RON, 1 test.
- `cargo test generated_nodes_use_ron_definitions --lib` - passed after map generation policy moved to RON, 1 test.
- `cargo check --lib` - passed after map generation policy migration.
- `cargo check -p game_server` - passed after map generation policy migration.
- `cargo test --lib -- --test-threads=1` - passed final static subgoal validation, 498 tests.
- `cargo test --test ron_loading` - passed final static subgoal validation, 17 tests.
