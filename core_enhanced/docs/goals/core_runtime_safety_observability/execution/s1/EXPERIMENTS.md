# S1 실험 기록

GEX-00 인수: 관련 U01 Core source unchanged, audited baseline reused. Server before check exit101 preserved.
P2G40 passed; P2G50 first formatting check failed3fields, corrected before publication; second passed20dimensions.

- settings_focused_001: 5 PASS; DTO/default/hash/parser boundaries.
- generation_check_001: FAIL, packaging uses the shared publish_snapshot wrapper; restored its non-test availability (no gameplay change).
- settings_focused_002: 7 PASS including bounded history import/serde and public external generation identity.
- generation_check_002: Core all-target check PASS (preexisting warnings).
- settings_focused_003: test compile FAIL2 wrong field names in new probes; corrected exact EffectSite.action.definition_id and TypedInteger.value. Rerun pending.

- `settings_focused_004`: exit 101, 41.82s, source_unchanged=True; `../../evidence/settings_focused_004.json` / `.log`.

- `settings_focused_005`: exit 101, 40.21s, source_unchanged=True; `../../evidence/settings_focused_005.json` / `.log`.

- `settings_and_restore_006`: exit 101, 16.17s, source_unchanged=True; `../../evidence/settings_and_restore_006.json` / `.log`.

- `settings_and_restore_007`: exit 101, 43.07s, source_unchanged=True; `../../evidence/settings_and_restore_007.json` / `.log`.

- `settings_and_restore_008`: exit 101, 48.24s, source_unchanged=True; `../../evidence/settings_and_restore_008.json` / `.log`.

- `settings_and_restore_009`: exit 0, 48.21s, source_unchanged=True; `../../evidence/settings_and_restore_009.json` / `.log`.

- `persistence_rollback_001`: exit 0, 27.13s, source_unchanged=True; `../../evidence/persistence_rollback_001.json` / `.log`.

- `server_adapter_check_001`: exit 101, 29.63s, source_unchanged=True; `../../evidence/server_adapter_check_001.json` / `.log`.

- `server_adapter_check_002`: exit 101, 28.76s, source_unchanged=True; `../../evidence/server_adapter_check_002.json` / `.log`.

- `server_adapter_check_003`: exit 101, 13.65s, source_unchanged=True; `../../evidence/server_adapter_check_003.json` / `.log`.

- `server_adapter_check_004`: exit 101, 5.76s, source_unchanged=True; `../../evidence/server_adapter_check_004.json` / `.log`.

- `server_adapter_check_005`: exit 0, 5.65s, source_unchanged=True; `../../evidence/server_adapter_check_005.json` / `.log`.

- `server_tests_001`: exit 101, 81.70s, source_unchanged=True; `../../evidence/server_tests_001.json` / `.log`.

- `server_tests_002`: exit 0, 23.28s, source_unchanged=True; `../../evidence/server_tests_002.json` / `.log`.

- `settings_focused_011`: exit 101, 14.00s, source_unchanged=True; `../../evidence/settings_focused_011.json` / `.log`.

- `settings_focused_012`: exit 0, 38.42s, source_unchanged=True; `../../evidence/settings_focused_012.json` / `.log`.

- `server_tests_003`: exit 101, 45.91s, source_unchanged=True; `../../evidence/server_tests_003.json` / `.log`.

- `server_first_spawn_001`: exit 0, 19.63s, source_unchanged=True; `../../evidence/server_first_spawn_001.json` / `.log`.

- `history_mutation_001`: exit 0, 61.05s, source_unchanged=True; `../../evidence/history_mutation_001.json` / `.log`.

- `all_default_001`: exit 0, 171.15s, source_unchanged=True; `../../evidence/all_default_001.json` / `.log`.

- `all_feature_001`: exit 0, 183.02s, source_unchanged=True; `../../evidence/all_feature_001.json` / `.log`.

- `generation_check_001`: exit 101, 30.24s, source_unchanged=True; `../../evidence/generation_check_001.json` / `.log`.

- `generation_check_002`: exit 0, 24.36s, source_unchanged=True; `../../evidence/generation_check_002.json` / `.log`.

- `history_duration_001`: exit 0, 32.14s, source_unchanged=True; `../../evidence/history_duration_001.json` / `.log`.

- `probe_dependencies_001`: exit 0, 1.58s, source_unchanged=True; `../../evidence/probe_dependencies_001.json` / `.log`.

- `public_boundary_001`: exit 1, 1.87s, source_unchanged=True; `../../evidence/public_boundary_001.json` / `.log`.

- `public_cli_001`: exit 0, 23.93s, source_unchanged=True; `../../evidence/public_cli_001.json` / `.log`.

- `public_oracle_001`: exit 0, 5.70s, source_unchanged=True; `../../evidence/public_oracle_001.json` / `.log`.

- `public_profile_001`: exit 1, 8.34s, source_unchanged=True; `../../evidence/public_profile_001.json` / `.log`.

- `public_profile_002`: exit 0, 15.12s, source_unchanged=True; `../../evidence/public_profile_002.json` / `.log`.

- `public_profile_003`: exit 0, 16.35s, source_unchanged=True; `../../evidence/public_profile_003.json` / `.log`.

- `public_trace_001`: exit 0, 4.20s, source_unchanged=True; `../../evidence/public_trace_001.json` / `.log`.

- `release_001`: exit 0, 69.21s, source_unchanged=True; `../../evidence/release_001.json` / `.log`.

- `server_before_check_001`: exit 101, 37.93s, source_unchanged=True; `../../evidence/server_before_check_001.json` / `.log`.

- `settings_and_restore_010`: exit 0, 106.37s, source_unchanged=True; `../../evidence/settings_and_restore_010.json` / `.log`.

- `settings_focused_001`: exit 0, 60.03s, source_unchanged=True; `../../evidence/settings_focused_001.json` / `.log`.

- `settings_focused_002`: exit 0, 66.71s, source_unchanged=True; `../../evidence/settings_focused_002.json` / `.log`.

- `settings_focused_003`: exit 101, 15.36s, source_unchanged=True; `../../evidence/settings_focused_003.json` / `.log`.


Final evidence additions (raw failures retained):

- `all_default_002`: exit 0, 112.23s, source_unchanged=True; `../../evidence/all_default_002.json` / `.log`.

- `all_feature_002`: exit 0, 143.65s, source_unchanged=True; `../../evidence/all_feature_002.json` / `.log`.

- `check_001`: exit 0, 33.87s, source_unchanged=True; `../../evidence/check_001.json` / `.log`.

- `clippy_core_001`: exit 0, 41.06s, source_unchanged=True; `../../evidence/clippy_core_001.json` / `.log`.

- `clippy_server_001`: exit 0, 38.40s, source_unchanged=True; `../../evidence/clippy_server_001.json` / `.log`.

- `coverage_001`: exit 0, 3.04s, source_unchanged=True; `../../evidence/coverage_001.json` / `.log`.

- `default_parity_002`: exit 0, 1.61s, source_unchanged=True; `../../evidence/default_parity_002.json` / `.log`.

- `event_log_check_001`: exit 0, 31.03s, source_unchanged=True; `../../evidence/event_log_check_001.json` / `.log`.

- `fmt_001`: exit 0, 3.93s, source_unchanged=True; `../../evidence/fmt_001.json` / `.log`.

- `pc_compare_002`: exit 0, 1.55s, source_unchanged=True; `../../evidence/pc_compare_002.json` / `.log`.

- `pc_current_001`: exit 101, 22.63s, source_unchanged=True; `../../evidence/pc_current_001.json` / `.log`.

- `pc_current_002`: exit 0, 171.20s, source_unchanged=True; `../../evidence/pc_current_002.json` / `.log`.

- `pc_prepare_001`: exit 0, 6.64s, source_unchanged=True; `../../evidence/pc_prepare_001.json` / `.log`.

- `pc_prepare_002`: exit 0, 4.41s, source_unchanged=True; `../../evidence/pc_prepare_002.json` / `.log`.

- `public_api_cli_001`: exit 0, 74.14s, source_unchanged=True; `../../evidence/public_api_cli_001.json` / `.log`.

- `public_boundary_002`: exit 0, 109.75s, source_unchanged=True; `../../evidence/public_boundary_002.json` / `.log`.

- `r05_compare_002`: exit 0, 1.56s, source_unchanged=True; `../../evidence/r05_compare_002.json` / `.log`.

- `r05_current_002`: exit 101, 46.88s, source_unchanged=True; `../../evidence/r05_current_002.json` / `.log`.

- `server_all_targets_001`: exit 0, 67.72s, source_unchanged=True; `../../evidence/server_all_targets_001.json` / `.log`.
