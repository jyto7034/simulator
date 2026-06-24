# Experiment Notes

## Master Notes

- `core_component_refactor_master` is the policy/audit source. This goal set is the implementation tracker.
- `docs/refactor_preparation_plan.md` remains the refactoring criteria baseline.
- Trust-system policies are intentionally deferred unless a later subgoal discovers a required compile/runtime conflict.
- Policy-sensitive implementation questions must be written as `사용자와 정책 논의 필요` in the relevant subgoal notes. If they are required for correct implementation, write a policy-decision report, close the active goal as `complete` through the available goal-status tool, and continue only after the user makes that policy decision.
- Subgoal 1 (`core_policy_static_data_map_scenario`) is complete by implementation/verification, not policy-decision completion. Its final validation commands were `cargo test --lib -- --test-threads=1` and `cargo test --test ron_loading`.
- Subgoal 2 (`core_policy_grant_economy_rewards`) is complete by implementation/verification, not policy-decision completion. Its final validation commands were `cargo test --test ron_loading`, `cargo check -p game_server`, and `cargo test --lib -- --test-threads=1`.
- Subgoal 3 (`core_policy_employee_item_growth`) is complete by implementation/verification, not policy-decision completion. Its final validation commands were `cargo test --test ron_loading`, `cargo check -p game_server`, and `cargo test --lib -- --test-threads=1`.
- Subgoal 4 (`core_policy_buff_status_stats`) is complete by implementation/verification, not policy-decision completion. Its final validation commands were `cargo test --test ron_loading`, `cargo check -p game_server`, and `cargo test --lib -- --test-threads=1`.
- Subgoal 5 (`core_policy_movement_battle_runtime`) is complete by implementation/verification, not policy-decision completion. Its final validation commands were `cargo test --test ron_loading`, `cargo check -p game_server`, and `cargo test --lib -- --test-threads=1`.
- Subgoal 6 (`core_policy_skill_targeting_projectile`) is complete by implementation/verification, not policy-decision completion. Its final validation commands were `cargo test --test ron_loading`, `cargo check -p game_server`, and `cargo test --lib -- --test-threads=1`.
- Subgoal 7 (`core_policy_unity_server_contract`) is complete by implementation/verification, not policy-decision completion. Its final validation commands were `cargo check --lib`, `cargo check -p game_server`, `cargo test --lib snapshots_and_start -- --test-threads=1`, `cargo test --lib combat:: -- --test-threads=1`, and `cargo test -p game_server player_game_actor -- --test-threads=1`.
- Subgoal 8 (`core_policy_validation_test_harness`) is complete by implementation/verification, not policy-decision completion. Its final validation commands were `cargo test --lib -- --test-threads=1`, `cargo test --test ron_loading -- --test-threads=1`, and `cargo check -p game_server`.
- Final cross-check artifact: `docs/goals/core_policy_validation_test_harness/POLICY_COVERAGE.md`.

## Follow-Up Candidates

- None yet.
