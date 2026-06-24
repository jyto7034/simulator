# Core Test Contract Audit Experiments

This file records audit attempts, commands, findings, failed assumptions, and verification results.

## 2026-06-21 - Goal Document Creation

Action:

- Created `docs/goals/core_test_contract_audit/PLAN.md`.
- Created `docs/goals/core_test_contract_audit/EXPERIMENTS.md`.
- Created `docs/goals/core_test_contract_audit/EXPERIMENT_NOTES.md`.

Result:

- Audit workspace exists.
- No production code, tests, or live RON/data were changed.
- Test audit has not started yet.

Verification:

- Not run. This step only creates documentation.

## 2026-06-21 - Inventory And Contract Audit

Action:

- Read the goal documents and used `PLAN.md` as the audit scope.
- Inventoried Rust test files under `src` and `tests`.
- Inspected representative test bodies in:
  - `src/game/world/tests/combat.rs`
  - `src/game/battle/core/mod.rs`
  - `src/game/battle/core/movement/planner.rs`
  - `src/game/battle/core/movement/engine.rs`
  - `src/game/combat_preview/mod.rs`
  - `src/game/data/**`
  - `tests/ron_loading.rs`
  - `tests/live_skill_catalog_audit.rs`
  - `tests/live_item_skill_activation.rs`
  - `tests/skill_refactor_validation.rs`
  - `tests/skill_test_suite.rs`
  - `/mnt/f/work/simulator/game_server/src/game/player_game_actor/{messages,state,handlers,session}.rs`
- Listed available tests with `cargo test -- --list` and `cargo test -p game_server -- --list`.

Commands:

```text
sed -n '1,260p' docs/goals/core_test_contract_audit/PLAN.md
sed -n '1,220p' docs/goals/core_test_contract_audit/EXPERIMENTS.md
sed -n '1,220p' docs/goals/core_test_contract_audit/EXPERIMENT_NOTES.md
find src tests -path '*/target' -prune -o -type f -name '*.rs' -print | sort
rg -n "#\[(test|tokio::test|actix_rt::test|should_panic|ignore)|fn [a-zA-Z0-9_]+\(" src/game/world/tests src/game/battle src/game/combat_preview src/game/data tests /mnt/f/work/simulator/game_server/src/game/player_game_actor
rg -n "battle_setup_snapshot|battle_update|battle_resync|command_result|state_snapshot|range_previews|unit_source|hud|known_seq|combat_result" src tests /mnt/f/work/simulator/game_server/src/game/player_game_actor
rg -n "live_defense|DefenseRoute|redeploy|withdraw|retreat|battle_records|blocking|MovementSegmentStarted|BasicAttackProjectile|TileArea|WholeFieldValidTiles|range_units|InterruptCast|SkillCastInterrupted|target usefulness|usefulness|DirectionalCollision|TargetLocked" src tests
sed -n '360,560p' src/game/world/tests/combat.rs
sed -n '1000,1280p' src/game/world/tests/combat.rs
sed -n '1760,2070p' src/game/world/tests/combat.rs
sed -n '2560,2845p' src/game/world/tests/combat.rs
sed -n '1880,2340p' src/game/battle/core/mod.rs
sed -n '2680,3335p' src/game/battle/core/mod.rs
sed -n '540,920p' src/game/battle/core/movement/planner.rs
sed -n '520,900p' src/game/battle/core/movement/engine.rs
sed -n '1,220p' /mnt/f/work/simulator/game_server/src/game/player_game_actor/messages.rs
sed -n '1,260p' /mnt/f/work/simulator/game_server/src/game/player_game_actor/state.rs
sed -n '1,340p' /mnt/f/work/simulator/game_server/src/game/player_game_actor/handlers.rs
sed -n '1,220p' /mnt/f/work/simulator/game_server/src/game/player_game_actor/session.rs
rg -n "#\[test\]|#\[actix_rt::test\]|fn .*" /mnt/f/work/simulator/game_server/src/game/player_game_actor/messages.rs /mnt/f/work/simulator/game_server/src/game/player_game_actor/state.rs /mnt/f/work/simulator/game_server/src/game/player_game_actor/handlers.rs /mnt/f/work/simulator/game_server/src/game/player_game_actor/session.rs
sed -n '260,620p' /mnt/f/work/simulator/game_server/src/game/player_game_actor/handlers.rs
sed -n '220,520p' /mnt/f/work/simulator/game_server/src/game/player_game_actor/state.rs
sed -n '220,520p' /mnt/f/work/simulator/game_server/src/game/player_game_actor/messages.rs
cargo test -- --list
rg -n "#\[ignore\]|ignore" src tests /mnt/f/work/simulator/game_server/src/game/player_game_actor
rg -n "TODO|FIXME|legacy|Legacy|compat|fallback|smoke|snapshot|golden" src/game/world/tests src/game/battle src/game/combat_preview src/game/data tests /mnt/f/work/simulator/game_server/src/game/player_game_actor
cargo test -p game_server -- --list
rg -n "#\[test\]|fn .*" src/game/combat_preview src/game/data tests/ron_loading.rs tests/live_skill_catalog_audit.rs tests/live_item_skill_activation.rs tests/skill_refactor_validation.rs tests/skill_test_suite.rs
sed -n '560,760p' tests/ron_loading.rs
sed -n '1,220p' tests/live_skill_catalog_audit.rs
sed -n '1,220p' src/game/combat_preview/mod.rs
rg -n "corroded_guard|melee_front_1|ranged_center_5x5|ranged_center_3x3|range_preset|basic_attack_range" src tests
```

Result:

- `cargo test -- --list` listed 496 `game_core` unit tests plus integration tests:
  - `tests/ron_loading.rs`: 16 tests.
  - `tests/live_item_skill_activation.rs`: 3 tests.
  - `tests/live_skill_catalog_audit.rs`: 3 tests.
  - `tests/skill_refactor_validation.rs`: 12 tests.
  - `tests/skill_test_suite.rs`: 4 tests.
- `cargo test -p game_server -- --list` listed 23 `game_server` tests.
- No `#[ignore]` tests were found in the audited scope. Matches for the word `ignore` were ordinary code/test names, not ignored legacy tests.
- No production code, test code, or live RON/data were changed.

Classification:

| Area | Classification | Judgment |
| --- | --- | --- |
| `src/game/world/tests/combat.rs` live battle flow | `strong_contract` | Strongly covers player-visible and Unity-facing behavior: live Defense battle entry, setup/update DTOs, deploy/withdraw, redeploy lock, delayed waves, playback pause/speed, retreat/re-entry, setup loss recovery, future seq rejection, battle records, and combat result transition. |
| `src/game/world/tests/combat.rs::defense_combat_node_smoke_writes_debug_event_log_export` | `smoke_only` plus partial `strong_contract` | It verifies battle can finish and records expected event categories, but the debug file export side effect is smoke/debug support rather than a gameplay contract by itself. |
| `src/game/battle/core/mod.rs` attack/skill tests | `strong_contract` | Good coverage for tile-based basic attacks, `range_units` not being the eligibility source, WholeFieldValidTiles, target usefulness, zero-damage hostile effects, interrupt/epoch behavior, and stale cast-end handling. |
| `src/game/battle/core/commands.rs` projectile tests from list | `strong_contract` | The basic projectile target-only collision tests pin the recently settled policy: launch eligibility is tile-based, projectile travel is target-locked, arrival does not recheck tile range, death/movement/attacker-death cases are explicit. |
| `src/game/battle/core/movement/planner.rs` | `strong_contract` | Strongly protects DefenseRoute/Arknights-like blocking: blocked enemies stop route movement, capacity uses route progress/spawn order tie-breaks, held block persists, blocker death/withdraw releases, airborne rules differ. |
| `src/game/battle/core/movement/engine.rs` and `rapier_backend.rs` | mixed `useful_narrow` / `implementation_shape` | Tests protect important movement policies, but several also pin backend synchronization/collider implementation details. Keep them while movement backend is volatile, but they are less user-facing than planner/world tests. |
| `src/game/combat_preview/**` | `strong_contract` | Meaningful coverage for battlefield templates, non-rectangular valid tiles, generated route contiguity, route overlays, defense route requirements, spawn wave identity entries, preview data from authored encounters, boss arena policy, and threat warnings. |
| `src/game/data/**` inline tests | mixed `strong_contract` / `useful_narrow` | Strong for schema/data policy validation such as skill step contracts, WholeField conflicts, projectile collision validation, abnormality threat class, corroded range presets, PVE objective/win-condition consistency. Narrow utility/index tests are still useful but not player-visible. |
| `tests/ron_loading.rs` | `strong_contract` | Good live-data guard: live RON loads, references resolve, forbidden legacy rewards are rejected, PVE encounter/preview data is backed by live data, and spatial/instant skill delivery composition is audited. |
| `tests/live_skill_catalog_audit.rs` | `strong_contract` with maintenance cost | Strongly prevents live skill catalog drift without behavior coverage. It is intentionally fixed-list based, so adding real content requires updating the manifest and behavior tests. |
| `tests/live_item_skill_activation.rs` | `strong_contract` | Meaningful live item/artifact activation behavior checks: skill ids resolve and on-battle-start/on-attack skills actually apply effects. |
| `tests/skill_refactor_validation.rs` and `tests/skill_test_suite.rs` | `strong_contract` | High-value skill behavior scenarios; they check timeline categories, projectile and conditional follow-up behavior, explicit cast target separation, named abnormality skills, and live-ish event outcomes. |
| `game_server/player_game_actor/messages.rs` | `strong_contract` | Tests top-level `battle_update`, `battle_setup_snapshot`, nested `battle_resync`, snake-case command parsing, admin separation, live battle command parsing. |
| `game_server/player_game_actor/state.rs` | `strong_contract` | Tests that battle results do not serialize through legacy command payloads, battle commands use side messages, and range preview avoids trailing `state_snapshot`. |
| `game_server/player_game_actor/handlers.rs` | `strong_contract` | Strong transport-order coverage: battle start sends setup/update without full snapshot, catch-up resync avoids snapshot, final update precedes combat_result snapshot, future known_seq is rejected, active-battle attach recovers to node_confirm, paused battle does not push deltas. |
| `game_server/player_game_actor/session.rs` | `useful_narrow` | Admin access tests are useful security gates; less central to gameplay contracts. |

High-value tests to preserve during refactors:

- `live_defense_deploy_and_withdraw_manage_cost_and_redeploy_lock`
- `live_deployment_reconciles_defeated_player_unit_before_state_dto`
- `battle_setup_snapshot_live_defense_state_request_returns_battle_update_without_advancing_cursor`
- `live_battle_update_exposes_adjacent_immutable_movement_segments`
- `generated_defense_route_setup_snapshot_matches_runtime_route_cells`
- `battle_setup_snapshot_exposes_survival_timer_without_encirclement_variant`
- `live_defense_setup_loss_recovery_returns_to_node_confirm_and_restarts_timeline`
- `retreat_from_live_defense_battle_reenters_until_attempts_are_exhausted`
- `finished_live_battle_tick_pushes_combat_result_snapshot_after_final_update`
- `live_battle_tick_pushes_update_without_state_snapshot_after_confirm_enter`
- `behavior_result_to_command_result_uses_battle_update_side_message`
- `basic_attack_uses_tile_range_not_range_units`
- `fixed_defense_player_basic_attack_uses_facing_tile_pattern_not_continuous_radius`
- `skill_enemy_single_uses_tile_range_not_range_units`
- `manual_pending_cast_can_be_interrupted_and_stale_end_is_ignored`
- `advance_basic_attack_projectile_is_target_locked_not_path_collision`
- `advance_basic_attack_projectile_does_not_recheck_tile_range_at_arrival`
- `fixed_defense_blocks_near_blockable_enemy_and_prioritizes_attack`
- `fixed_defense_movement_tick_does_not_emit_segment_for_blocked_enemy`
- `live_skill_catalog_is_fully_covered_by_behavior_test_manifest`
- `live_pve_scenario_authoring_contracts_drive_preview_data`

Weak, narrow, or implementation-shaped areas:

- Movement backend synchronization tests in `rapier_backend.rs` are valuable while the backend is changing, but many assert collider/body synchronization details rather than player-visible movement contracts.
- Deterministic id helper tests in `src/game/battle/core/ids.rs` are useful for replay stability, but they are narrow and should remain subordinate to event-log/DTO tests.
- `defense_combat_node_smoke_writes_debug_event_log_export` should not be treated as the main battle correctness proof; its useful contract is broad “battle reaches event log with key event categories”, not the debug export itself.
- Fixed-list catalog tests in `tests/live_skill_catalog_audit.rs` are intentionally strict. They are meaningful, but they create a deliberate maintenance checkpoint when content changes.
- Legacy field names in compressed battle event log payload are intentional Unity JSON contract compatibility, not stale behavior. They should remain documented as external contract until Unity changes.

Missing or under-strength contract coverage:

- There is no single end-to-end test that ties movement presentation time and attack/damage time together from live battle update JSON. Unit tests cover movement segments and attack/projectile rules separately, but a regression could still make damage appear before Unity-visible movement catches up.
- Live RON currently validates that corroded employee profiles and range presets resolve, and inline data tests validate preset mechanics. It does not strongly pin the exact live mapping `corroded_guard/corroded_rusher/corroded_bruiser/corroded_veteran -> melee_front_1`, `corroded_marksman -> ranged_center_5x5`, `corroded_medic -> ranged_center_3x3`.
- Rust server tests cover transport mapping and actor behavior, but the actual WebSocket JSON probe remains outside the cargo test suite. This is acceptable for now, but the contract would be stronger if the probe were runnable in CI or mirrored by a Rust integration test.
- Enemy range-preview suppression / visual-warning-only policy is mostly a documentation/Unity consumption policy. The audited Rust tests do not clearly assert “enemy attack range DTOs are omitted unless explicitly needed for warning”.
- Attack/windup presentation event coverage is indirect. Current tests focus on attack resolution, projectile launch/arrival, and timeline categories; if Unity later needs a first-class `AttackWindUp` contract, tests must be added with that DTO/event shape.

Failed assumptions:

- I expected to find at least some ignored legacy tests because the codebase has undergone several contract transitions. The audited scope had no `#[ignore]` tests.
- I expected live RON to already pin every corroded employee profile-to-range preset mapping. It currently pins presence/resolution more than exact live mapping.

Verification:

- `cargo test -- --list` succeeded and changed nothing.
- `cargo test -p game_server -- --list` succeeded and changed nothing.
- Full test execution was not run because this goal is an audit-only pass and the list/read commands were sufficient to classify coverage.
