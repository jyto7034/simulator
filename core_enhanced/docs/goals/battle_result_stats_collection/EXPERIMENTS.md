# Battle Result Stats Collection Experiments

Record implementation attempts, failures, fixes, and verification here.

| Date | Attempt | Result | Notes |
| --- | --- | --- | --- |
| 2026-06-23 | Goal draft | Created the initial implementation plan for battle-end event-log-derived result stats. | Implementation not started. First implementation step must re-read runtime event emitters and DTO boundaries. |
| 2026-06-23 | First focused collector test | Failed to compile because the test fixture used `Default::default()` for `FacingDirection`, which has no `Default` impl. | Fixed fixture to use an explicit `FacingDirection::Right`. |
| 2026-06-23 | Focused collector test after fixture fix | One test failed because the expected deployed time used battle duration instead of the interval from `UnitDeployed` at 100ms to `BattleEnd` at 1000ms. | Corrected expected `deployed_time_ms` to 900ms, matching the confirmed interval policy. |
| 2026-06-23 | Collector implementation | Added `battle::result_stats` DTOs and event-log collector. | Success. Focused collector tests pass. | Actual HP delta, participant merge, deployed time, basic attack count, kill count, and MVP eligibility are covered. |
| 2026-06-23 | Snapshot/server contract | Added `result_stats` to `CombatBattleState`, selected-event snapshot, battle record export, and server combat-result snapshot test. | Success. Focused core/server tests pass. | `compressed_event_log` remains transport/debug attachment; `result_stats` is the result UI source. |
| 2026-06-23 | RON/static MVP policy | Added `battle_result_stats.mvp` metric weights and tiebreakers to run policy RON/schema. | Success. Run policy and live RON loading tests pass. | MVP weights are data-driven; no hard-coded score table in the collector. |
| 2026-06-23 | Contract documentation | Updated internal rulebook and external Unity contract docs. | Success. | Docs now tell Unity to read `selected_event.result_stats` instead of recalculating result stats from raw event log. |
| 2026-06-23 | Completion-condition focused test audit | Expanded the collector fixture to assert damage taken, skill cast count, deploy count, withdraw count, and withdrawn deployment interval. | Success after one fixture fix. | Ability-sourced HP loss uses `HpChangeReason::Command` plus `DamageSource::Ability`; `HpChangeReason` itself has no `Ability` variant. |
| 2026-06-23 | Live Unity WS probe contract update | Updated `/mnt/f/unity projects/ark/docs/probe/unity_ws_smoke_probe.py` to assert `selected_event.result_stats.battle`, `employees`, `mvp`, and `compressed_event_log` when `has_event_log` is true. | Success after probe stabilization. | The old probe still checked legacy `has_timeline` / `compressed_timeline`; the live probe now checks the current JSON contract. |
| 2026-06-23 | Live result stat value audit | Added compact result-stat summaries to the Unity WS probe and inspected real values from a live server run. | Found and fixed a collector bug. | Player-side defense objects were being merged into `employees`; participant merge now requires `UnitSpawned.unit_source == Employee`. |

## Validation Log

- `cargo test -p game_core battle::result_stats -- --nocapture` failed before running tests due to the fixture compile error recorded above.
- `cargo test -p game_core battle::result_stats -- --nocapture` then ran 2 tests: 1 passed, 1 failed due to the incorrect 1000ms deployed-time expectation. Fixed to 900ms.
- `cargo test -p game_core battle::result_stats -- --nocapture` passed: 2 tests.
- `cargo test -p game_core battle::result_stats -- --nocapture` failed after expanding the fixture because it used non-existent `HpChangeReason::Ability`. Fixed the fixture to use `HpChangeReason::Command` with `DamageSource::Ability`.
- `cargo test -p game_core battle::result_stats -- --nocapture` passed after the focused test expansion: 2 tests.
- `cargo test -p game_core combat_result_snapshot_exposes_typed_result_stats -- --nocapture` passed.
- `cargo test -p game_core game::data::run_policy_data -- --nocapture` passed: 2 tests.
- `cargo check -p game_core` passed.
- `cargo check -p game_server` passed.
- `cargo test -p game_core --test ron_loading -- --nocapture` passed: 18 tests.
- `cargo test -p game_core game::world::tests::combat -- --nocapture` passed: 29 tests.
- `cargo test -p game_core game::world::tests::snapshots_and_start -- --nocapture` passed: 8 tests.
- `cargo test -p game_server finished_live_battle_tick_pushes_combat_result_snapshot_after_final_update -- --nocapture` passed.
- `cargo test -p game_core` passed: 529 lib tests, 3 live item tests, 3 live skill catalog tests, 18 RON loading tests, 12 skill refactor validation tests, 4 skill suite tests, and doc tests.
- `cargo test -p game_core` passed again after focused collector test expansion and goal document updates.
- `PYTHONPYCACHEPREFIX=/tmp python3 -m py_compile '/mnt/f/unity projects/ark/docs/probe/unity_ws_smoke_probe.py'` passed. Plain `python3 -m py_compile` failed in the sandbox because Python attempted to write `__pycache__` under the read-only external Unity docs directory.
- `APP__SERVER__BIND_ADDRESS=127.0.0.1 APP__SERVER__PORT=18083 cargo run` was run from `/mnt/f/work/simulator/game_server` to start a real local server.
- `PYTHONPYCACHEPREFIX=/tmp WS_PORT=18083 WS_CHECK_COMBAT_RESULT=1 WS_BATTLE_END_TIMEOUT=120 python3 '/mnt/f/unity projects/ark/docs/probe/unity_ws_smoke_probe.py'` first timed out because a one-employee deployment did not reliably resolve the selected battle before the timeout.
- The probe was stabilized to deploy one additional starter when possible and set battle speed to `X3`.
- `PYTHONPYCACHEPREFIX=/tmp WS_PORT=18083 WS_CHECK_COMBAT_RESULT=1 WS_BATTLE_END_TIMEOUT=180 python3 '/mnt/f/unity projects/ark/docs/probe/unity_ws_smoke_probe.py'` passed against the live server. The JSON summary reported `combat_result_game_state: "combat_result"`, `selected_event_type: "combat_battle"`, `has_event_log: true`, `result_stats_employee_count: 3`, and `result_stats_has_mvp: true`.
- A follow-up live value audit showed real employee stats were present (`damage_dealt`, `kill_count`, `basic_attack_count`, `deployed_time_ms`, and MVP), but also exposed an incorrect player-side defense object row in `result_stats.employees`.
- `cargo test -p game_core battle::result_stats -- --nocapture` passed after fixing participant merge to exclude non-employee unit sources: 3 tests.
- `PYTHONPYCACHEPREFIX=/tmp WS_PORT=18083 WS_CHECK_COMBAT_RESULT=1 WS_BATTLE_END_TIMEOUT=180 python3 '/mnt/f/unity projects/ark/docs/probe/unity_ws_smoke_probe.py'` passed again against the live server after the fix. The JSON summary reported `result_stats_employee_count: 2`, battle `winner: "Player"`, `killed_enemy_count: 2`, one employee with `damage_dealt: 180`, `kill_count: 2`, `basic_attack_count: 18`, and MVP title `"Most Kills"`.

## Failure Log

- Fixture compile failure: `tile_range::FacingDirection: Default` is not implemented. Fixed by using an explicit facing value.
- Incorrect test expectation: deployment interval starts at `UnitDeployed`, not battle start. Fixed expected `deployed_time_ms`.
- Fixture enum mismatch: `HpChangeReason` has `BasicAttack` and `Command`; ability source is represented through `DamageSource::Ability`. Fixed the test fixture accordingly.
- Probe contract drift: Unity smoke probe still checked `has_timeline` / `compressed_timeline`. Replaced with `result_stats` and `compressed_event_log` checks.
- Probe runtime instability: one deployed starter did not always finish the selected battle. Stabilized the live check by deploying one additional starter when possible and setting battle speed to `X3`.
- Result-stat source filtering bug: `ParticipantBattleResult` rows for player-side defense objects were merged into employee stats. Fixed by merging participant results only when the unit source maps directly to an employee.
