# Run Floor Progression Contract Experiments

## Experiment Log

| Date | Experiment | Result | Next Action |
| --- | --- | --- | --- |
| 2026-06-26 | Initial code/document audit before creating the goal | Read the relevant runtime surfaces enough to scope the goal. Current code already has `GameMode`, Gate terminal selection for non-final Standard/Endless, Gate trauma recovery, map graph selectability, and Standard final boss semantics. The major remaining contract debt is Act-centric progression naming/state/DTO. | Begin implementation by auditing `RunProgression` and snapshot/result DTOs in detail. |
| 2026-06-26 | Replaced Act-centric runtime progression with mode-aware Floor state. | `RunProgression` now stores `RunProgressionModeState::{Standard, Endless}`; map generation, Gate transition, final Standard boss checks, snapshots, command result payloads, and encounter difficulty selection use Floor terminology. | Update tests/docs and run focused checks. |
| 2026-06-26 | Updated live run policy RON and loader schema. | `setup.default_max_acts` was replaced by `setup.standard_floor_count`; live RON loading passed after the schema and fixture test were updated. | Keep broad validation. |
| 2026-06-26 | Updated user-visible tests for Floor DTOs and Standard progression. | `map_flow`, snapshot, run policy, final boss, and live RON loading focused tests passed. | Run broad `cargo test --lib`. |
| 2026-06-26 | Ran broad library tests after Floor migration. | Passed: 543 tests. Two tests failed in the first broad run, then passed after correcting stale expectations and an invalid combat-result validation order. | Phase 1 runtime implementation is complete. Do not start Phase 2 without user direction. |

## Verification Commands

Record every focused and broad check here while implementing.

Initial planned focused checks:

```bash
cargo test generated_standard_maps_use_gate_before_final_act_and_boss_on_final_act --lib
cargo test confirmed_gate_transition_advances_act_and_recovers_living_trauma_without_checkpoint --lib
cargo test start_new_game_uses_requested_game_mode_when_run_is_created --lib
cargo test snapshots_and_start --lib
cargo test map_flow --lib
cargo test ron_loading --test ron_loading
```

Update command names as tests are renamed from Act to Floor.

Commands actually run:

```bash
cargo check
cargo test game::world::tests::map_flow --lib
cargo test run_snapshot_exposes_current_flow_after_start --lib
cargo test run_policy --lib
cargo test load_game_data_from_ron --test ron_loading
cargo test final_boss --lib
cargo test combat_result_without_node_session_is_rejected --lib
cargo test live_ron_defense_route_playable_path_runs_to_combat_result --lib
cargo test --lib
```

Final successful broad result:

```text
test result: ok. 543 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

## Failed Attempts

### 2026-06-26: Incorrect multi-filter cargo test command

- Command:
  - `cargo test run_snapshot_exposes_current_flow_after_start start_new_game_exposes_initial_map_without_entering_a_node_session confirmed_gate_transition --lib`
  - `cargo test final_boss_retreat_is_allowed_until_third_attempt_fails_run final_boss_defeat_immediately_fails_run --lib`
  - `cargo test game::world::tests::map_flow::confirmed_gate_transition_advances_floor_and_recovers_living_trauma_without_checkpoint game::map::progression::tests::completing_terminal_boss_reports_boundary_and_keeps_no_available_nodes --lib`
- Failure summary: `cargo test` accepts a single test filter argument; extra filters were rejected as unexpected arguments.
- Root cause: Command syntax mistake, not a code failure.
- Corrective action: Re-ran using module/single broader filters such as `game::world::tests::map_flow`, `run_snapshot_exposes_current_flow_after_start`, and `final_boss`.
- Re-verification result: Passed.

### 2026-06-26: Probe docs directory absent

- Command: `rg ... docs/probe`
- Failure summary: `docs/probe` does not exist in this repository.
- Root cause: Probe docs were referenced by earlier planning text, but the current workspace has no `docs/probe` directory.
- Corrective action: Updated canonical docs that exist in this workspace and verified non-goal docs no longer expose old Act DTO fields.
- Re-verification result: `rg -n "act_index|max_acts|current_act_seed|ActComplete|default_max_acts" docs --glob '!docs/goals/**' ../game_resources -S` returned no matches.

### 2026-06-26: Run policy unknown-field test failed before reaching unknown field

- Command: `cargo test run_policy --lib`
- Failure summary: `run_policy_rejects_legacy_or_unknown_fields` did not report `legacy_fallback`.
- Root cause: The test fixture omitted the required `support.gate_transition_trauma_recovery_percent`, so deserialization failed before the intended unknown field check.
- Corrective action: Added the required Gate trauma field to the fixture and asserted the builtin value.
- Re-verification result: `cargo test run_policy --lib` passed.

### 2026-06-26: Broad lib test exposed stale combat-result expectations

- Command: `cargo test --lib`
- Failure summary: Two tests failed:
  - `combat_result_without_node_session_is_rejected`
  - `live_ron_defense_route_playable_path_runs_to_combat_result`
- Root cause:
  - `handle_complete_combat_result` checked final-boss status through run state before validating that a combat node session existed.
  - The live RON smoke test assumed a completed map node result, but current policy can return retry `NodePreview` when a non-final combat node is lost with attempts remaining.
- Corrective action:
  - Moved node-session validation before final-boss checks and used the session node id as the combat node source.
  - Updated the smoke test to accept policy-valid retry, node completion, Floor advancement, or run completion outcomes.
- Re-verification result:
  - `cargo test combat_result_without_node_session_is_rejected --lib` passed.
  - `cargo test live_ron_defense_route_playable_path_runs_to_combat_result --lib` passed.
  - `cargo test --lib` passed.

When a test or trial fails, record:

- command;
- failure summary;
- root cause;
- code/data/doc change made;
- re-verification command and result.
