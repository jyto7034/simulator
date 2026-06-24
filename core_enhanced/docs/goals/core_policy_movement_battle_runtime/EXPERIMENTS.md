# Experiments

## Log

| Date | Scope | Attempt | Result | Follow-up |
| --- | --- | --- | --- | --- |
| 2026-06-22 | Goal setup | Defined movement/battle runtime implementation scope. | Not started. | Start with behavior gate and scheduler inventory. |
| 2026-06-22 | Runtime/code inventory | Read action scheduler, behavior command execution, movement lifecycle, battlefield placement, battle stepping, scenario spawn, live withdraw, run policy, and timeline validation. | Success. `allowed_actions`, preview gate, command validation gates, movement stop reasons, and forced engagement recompute were already aligned; spawn fallback, runtime numeric constants, and withdraw timeline event needed code changes. | Implement focused changes and tests. |
| 2026-06-22 | Runtime policy data | Added `battle_runtime` to `RunPolicyData`, live `run/policy.ron`, validation, and tests. | Success. Movement tick and max battle time now load from live run policy. | Use policy in battle stepper. |
| 2026-06-22 | Spawn occupancy | Removed opponent nearest-open fallback and added overlap-preserving placement for opponent authored spawns. | Success. Opponent units can share authored spawn position; position source remains authored data. | Keep static obstacle/out-of-bounds validation. |
| 2026-06-22 | Live withdraw timeline | Added `TimelineEvent::UnitWithdrawn`, wired `BattleLiveCommand::WithdrawUnit { time_ms }`, and updated timeline validators. | Success. Live withdraw now leaves an explicit timeline event. | DTO split remains later contract work. |
| 2026-06-22 | Same timestamp attack resolve | Drained same-time `AttackResolve` events before winner finalization and replaced the stale TODO with current policy wording. | Success. Focused regression test confirms `AttackResolve` is recorded before same-time `BattleEnd`. | Do not broaden into dead-attacker basic attack snapshot policy here. |
| 2026-06-22 | Final validation | Ran focused tests, movement/live deployment/run policy/attack resolve groups, live RON loading, server check, full lib tests, and diff check. | Success. All validation passed. | Continue with `core_policy_skill_targeting_projectile`. |

## Failed Approaches

- `cargo check --lib` initially failed after adding `UnitWithdrawn` because timeline death/spawn validators did not cover the new event. Added explicit match arms so withdraw references a spawned unit and is treated as an invalid operation after death.

## Validation Commands

- `cargo check --lib` - passed.
- `cargo test apply_live_command_deploys_and_withdraws_player_unit --lib` - passed.
- `cargo test opponent_spawn_uses_authored_position_even_when_occupied --lib` - passed.
- `cargo test battle_runtime_policy_controls_max_battle_time --lib` - passed.
- `cargo test movement_tick_schedule_uses_run_policy_interval --lib` - passed.
- `cargo test overlap_placement_preserves_each_unit_position --lib` - passed.
- `cargo test same_timestamp_attack_resolve_is_drained_before_battle_end --lib` - passed.
- `cargo test movement --lib` - passed, 67 tests.
- `cargo test live_deployment --lib` - passed, 2 tests.
- `cargo test run_policy --lib` - passed, 3 tests.
- `cargo test attack_resolve --lib` - passed, 1 test.
- `cargo test --test ron_loading` - passed, 18 tests.
- `cargo check -p game_server` - passed.
- `cargo test --lib -- --test-threads=1` - passed, 512 tests.
- `git diff --check` - passed.
