# Experiments

| Date | Attempt | Result | Next Step |
| --- | --- | --- | --- |
| 2026-06-26 | Goal setup and policy/code inventory | Created the goal directory and read the newly fixed node map policy in `docs/game_rulebook.md`, the external Unity map contract, and current map/progression/retreat code. Found that attempts are already 3, internal edges exist, but runtime lacks `GameMode`, `Gate`, `map_navigation`, and final-boss retreat semantics. | Implement in small slices, starting with DTO/navigation inventory and focused tests. |
| 2026-06-26 | Replaced the public run snapshot map progress projection with `map_navigation` and removed `map.edges` from the map DTO test expectation. | `cargo test -p game_core run_snapshot_exposes_current_flow_after_start -- --test-threads=1` passed. Snapshot now proves `map_navigation.current_node_id` and `selectable_node_ids`, and absence of public `map.edges` / legacy id lists for the focused start snapshot. | Add command validation and navigation reachability tests, then continue toward `GameMode`/Gate only if no stop condition appears. |
| 2026-06-26 | Added `SelectMapNode` validation against core-computed `MapProgression::selectable_node_ids()` and a user-visible rejection test. | One command attempt failed because `cargo test` accepts only one positional test filter. Re-ran the full `game::world::tests::map_flow` module successfully: 12 passed. | Continue with stronger navigation DTO/test coverage, then inventory Gate/GameMode stop conditions. |
| 2026-06-26 | Added explicit runtime `GameMode` to `RunProgression`, exposed it in `run_progression.game_mode`, and extended checkpoint load coverage. | Snapshot and checkpoint focused tests passed. Existing checkpoint is in-memory clone/load, so no external save migration decision appeared in code reviewed so far. | Continue to Gate node/policy field inventory and stop if command semantics are unclear. |
| 2026-06-26 | Added first-class `Gate` category, live `gate_stairs` node definition, RON policy `support.gate_transition_trauma_recovery_percent`, and a `ConfirmEnterNode` Gate transition path. | Focused Gate transition test passed: Gate advances Act, recovers living employee trauma by 10%, does not heal HP, and does not save a checkpoint. `cargo check -p game_core` passed before wiring transition; focused Gate test compiled the new path. | Resolve generated live map structure: current `RunMap.boss_node_id`/public `MapViewDto.boss_node_id` terminal model conflicts with Gate as a non-boss independent endpoint. |
| 2026-06-26 | Re-ran the map flow module, cargo check, and live RON integration after Gate wiring and RON edits. | `game::world::tests::map_flow` passed 13 tests, `cargo check -p game_core` passed, and `cargo test -p game_core --test ron_loading -- --test-threads=1` passed 18 tests. A first `cargo test -p game_core ron_loading` attempt was too weak because it filtered out every test. | Stop before generated Gate placement until terminal endpoint vs boss identity is resolved, or refactor `RunMap` terminal identity if user approves. |
| 2026-06-26 | Ran formatter and revalidated the most touched Gate slice. | `cargo fmt` completed, then Gate focused test and `cargo check -p game_core` both passed. | Report remaining generated-map terminal identity blocker before continuing into a broader map schema refactor. |
| 2026-06-26 | Checked server compilation after Unity-facing DTO shape changes. | `cargo check -p game_server` passed. | Keep goal active; generated Gate placement and final-boss/attempt policy are not complete. |
| 2026-06-26 | Split map terminal identity from boss identity by replacing `boss_node_id` with internal `terminal_node_id`, then drove live map generation from `RunProgression::terminal_node_category()`. | `game::world::tests::map_flow` passed. Standard Act 1-2 now generate a Gate terminal, while the final Standard Act generates a Boss terminal. Public `MapViewDto` does not expose a boss/terminal id. | Continue combat/final-boss attempt policy and external contract sync. |
| 2026-06-26 | Updated combat attempts so Boss nodes also consume attempts, final boss is detected from Standard final-act terminal Boss, Boss/Boss missions start as live DefenseRoute-derived battles, and failure/retreat retries while attempts remain. | Focused tests passed for failed defense retry, final boss third retreat run failure, final boss defeat immediate run failure, and non-final boss defeat retry. | Run broader checks and update external docs/probe if DTO examples lag. |
| 2026-06-26 | Strengthened Gate terminal validation and synced external Unity/probe docs with the current `map_navigation`/`game_mode` DTO contract. | Focused map flow, combat attempt, final boss, live RON, `cargo check -p game_core`, and `cargo check -p game_server` all passed. Probe Python syntax check passed via bytecode-free compile after `py_compile` hit an external `__pycache__` write-permission issue. Later audit also refreshed the external retreat and Boss/Boss live battle wording. | Start the actual server and run the Unity smoke probe if the environment supports it. |
| 2026-06-26 | Started the actual `game_server` and ran the Unity WebSocket smoke probe against it. | Passed after rerunning the probe with local socket permission. The live snapshot root keys include `map_navigation` and omit `map_progression`; live map nodes were Revealed and included a Gate terminal in Act 1. | Perform final completion audit against PLAN requirements. |
| 2026-06-26 | Added explicit Gate preview/cancel coverage after the final audit found the requirement was only indirectly proven. | `gate_preview_does_not_advance_act_or_apply_reward_without_confirm` passed as part of the map flow module. It proves selecting a Gate only enters `NodeConfirm`; act index, trauma, HP, and checkpoint state remain unchanged until `ConfirmEnterNode`. | Final completion audit. |
| 2026-06-26 | Ran and then strengthened the map progression unit module during final audit to directly cover transit and edge-direction rules. | Passed 10 tests after adding `completed_nodes_are_transit_only_not_selectable_destinations`. Coverage includes selectability requires available/not-concealed, locked children contract rejection, missing edge target rejection, ForwardOnly reverse transit rejection, and Completed transit-only/non-selectable behavior. | Final completion audit. |

## Validation Log

Add every command here while implementing. Include failed commands and the fix before re-running.

```text
# Example format:
# cargo test -p game_core <test_name> -- --test-threads=1
# result:
# fix:
# revalidation:

cargo test -p game_core run_snapshot_exposes_current_flow_after_start -- --test-threads=1
result: passed
fix: n/a
revalidation: n/a

cargo test -p game_core select_map_node_rejects_nodes_outside_core_navigation node_confirm_allows_reselecting_another_available_node_before_entering -- --test-threads=1
result: failed before tests ran; cargo test accepts only one positional test filter
fix: reran the containing module filter instead
revalidation: cargo test -p game_core game::world::tests::map_flow -- --test-threads=1

cargo test -p game_core game::world::tests::map_flow -- --test-threads=1
result: passed, 12 passed
fix: n/a
revalidation: n/a

cargo test -p game_core run_snapshot_exposes_current_flow_after_start load_run_checkpoint_restores_visible_run_state_but_not_load_count -- --test-threads=1
result: failed before tests ran; cargo test accepts only one positional test filter
fix: reran each focused test separately
revalidation: see next two commands

cargo test -p game_core game::world::tests::snapshots_and_start::run_snapshot_exposes_current_flow_after_start game::world::tests::support::load_run_checkpoint_restores_visible_run_state_but_not_load_count -- --test-threads=1
result: failed before tests ran; cargo test accepts only one positional test filter
fix: reran each focused test separately
revalidation: see next two commands

cargo test -p game_core run_snapshot_exposes_current_flow_after_start -- --test-threads=1
result: passed
fix: n/a
revalidation: n/a

cargo test -p game_core load_run_checkpoint_restores_visible_run_state_but_not_load_count -- --test-threads=1
result: passed
fix: n/a
revalidation: n/a

cargo check -p game_core
result: passed with a warning that `GateTransition` was not yet constructed
fix: wired `GateTransition` into confirmed Gate transition path
revalidation: cargo test -p game_core confirmed_gate_transition_advances_act_and_recovers_living_trauma_without_checkpoint -- --test-threads=1

cargo test -p game_core confirmed_gate_transition_advances_act_and_recovers_living_trauma_without_checkpoint -- --test-threads=1
result: passed
fix: n/a
revalidation: n/a

cargo test -p game_core game::world::tests::map_flow -- --test-threads=1
result: passed, 13 passed
fix: n/a
revalidation: n/a

cargo check -p game_core
result: passed
fix: n/a
revalidation: n/a

cargo test -p game_core ron_loading -- --test-threads=1
result: ran 0 tests because the filter did not match integration test names
fix: reran the integration test target directly
revalidation: cargo test -p game_core --test ron_loading -- --test-threads=1

cargo test -p game_core --test ron_loading -- --test-threads=1
result: passed, 18 passed
fix: n/a
revalidation: n/a

cargo fmt
result: passed
fix: n/a
revalidation: cargo test -p game_core confirmed_gate_transition_advances_act_and_recovers_living_trauma_without_checkpoint -- --test-threads=1; cargo check -p game_core

cargo test -p game_core confirmed_gate_transition_advances_act_and_recovers_living_trauma_without_checkpoint -- --test-threads=1
result: passed after cargo fmt
fix: n/a
revalidation: n/a

cargo check -p game_core
result: passed after cargo fmt
fix: n/a
revalidation: n/a

cargo check -p game_server
result: passed
fix: n/a
revalidation: n/a

cargo test -p game_core game::world::tests::map_flow -- --test-threads=1
result: failed once because an old test still expected a public boss terminal in Act 1; updated it to assert Standard Act 1 Gate terminal and final Act Boss terminal
fix: updated `generated_standard_maps_use_gate_before_final_act_and_boss_on_final_act`
revalidation: cargo test -p game_core game::world::tests::map_flow -- --test-threads=1

cargo test -p game_core game::world::tests::map_flow -- --test-threads=1
result: passed, 13 passed
fix: n/a
revalidation: n/a

cargo test -p game_core failed_live_defense_battle_reenters_until_attempts_are_exhausted -- --test-threads=1
result: passed
fix: n/a
revalidation: n/a

cargo test -p game_core final_boss_retreat_is_allowed_until_third_attempt_fails_run -- --test-threads=1
result: failed once because Boss/Boss missions were live-enabled but still had no DefenseRoute tactical plan, so the battle did not stay in `in_battle`
fix: made Boss/Boss use the default DefenseRoute tactical plan and compatible protect-unit win condition
revalidation: cargo test -p game_core final_boss_retreat_is_allowed_until_third_attempt_fails_run -- --test-threads=1

cargo test -p game_core final_boss_retreat_is_allowed_until_third_attempt_fails_run -- --test-threads=1
result: passed
fix: n/a
revalidation: n/a

cargo test -p game_core final_boss_defeat_immediately_fails_run -- --test-threads=1
result: passed
fix: n/a
revalidation: n/a

cargo test -p game_core non_final_boss_defeat_reenters_while_attempts_remain -- --test-threads=1
result: failed once because NodeConfirm snapshot still hid `abnormality_attempt` for Boss nodes
fix: exposed NodeConfirm abnormality attempts for Combat and Boss categories
revalidation: cargo test -p game_core non_final_boss_defeat_reenters_while_attempts_remain -- --test-threads=1

cargo test -p game_core non_final_boss_defeat_reenters_while_attempts_remain -- --test-threads=1
result: passed
fix: n/a
revalidation: n/a

cargo fmt
result: passed after Gate validation/probe doc sync patches
fix: n/a
revalidation: focused tests below

python3 -m py_compile "/mnt/f/unity projects/ark/docs/probe/unity_ws_smoke_probe.py"
result: failed because Python tried to write bytecode into the external docs/probe __pycache__ directory and hit Errno 30 read-only file system
fix: use bytecode-free `compile(path.read_text(), path, "exec")` syntax validation instead
revalidation: see next command

python3 - <<'PY'
from pathlib import Path
path = Path('/mnt/f/unity projects/ark/docs/probe/unity_ws_smoke_probe.py')
compile(path.read_text(), str(path), 'exec')
print('syntax ok')
PY
result: passed
fix: n/a
revalidation: n/a

cargo test -p game_core game::world::tests::map_flow -- --test-threads=1
result: passed, 13 passed
fix: n/a
revalidation: n/a

cargo test -p game_core failed_live_defense_battle_reenters_until_attempts_are_exhausted -- --test-threads=1
result: passed
fix: n/a
revalidation: n/a

cargo test -p game_core final_boss_retreat_is_allowed_until_third_attempt_fails_run -- --test-threads=1
result: passed
fix: n/a
revalidation: n/a

cargo test -p game_core final_boss_defeat_immediately_fails_run -- --test-threads=1
result: passed
fix: n/a
revalidation: n/a

cargo test -p game_core non_final_boss_defeat_reenters_while_attempts_remain -- --test-threads=1
result: passed
fix: n/a
revalidation: n/a

cargo test -p game_core --test ron_loading -- --test-threads=1
result: passed, 18 passed
fix: n/a
revalidation: n/a

cargo check -p game_core
result: passed
fix: n/a
revalidation: n/a

cargo check -p game_server
result: passed
fix: n/a
revalidation: n/a

ss -ltn sport = :18083 || true
result: produced a netlink permission warning but no listening entry was shown
fix: proceeded to start the local server on the documented port
revalidation: server startup log confirmed 127.0.0.1:18083

APP__SERVER__BIND_ADDRESS=127.0.0.1 APP__SERVER__PORT=18083 cargo run
result: server started successfully from `/mnt/f/work/simulator/game_server` and logged `Game Server is running on 127.0.0.1:18083`
fix: n/a
revalidation: live smoke probe below

WS_PORT=18083 python3 "/mnt/f/unity projects/ark/docs/probe/unity_ws_smoke_probe.py"
result: failed in sandbox with `PermissionError: [Errno 1] Operation not permitted` while creating the local socket
fix: reran with escalated local socket permission because live WebSocket probe validation requires connecting to 127.0.0.1:18083
revalidation: see next command

WS_PORT=18083 python3 "/mnt/f/unity projects/ark/docs/probe/unity_ws_smoke_probe.py"
result: passed with escalated local socket permission; summary included `snapshot_root_keys` with `map_navigation` and without `map_progression`, `map_contract` node_count=27, available_count=4, selectable_count=4, and an Act 1 `Gate` terminal in the map payload
fix: n/a
revalidation: n/a

rg -n "비보스 실시간|비보스|can_retreat|third retreat|세 번째 시도|map_progression|boss_node_id|available_node_ids|completed_node_ids" "/mnt/f/unity projects/ark/docs/unity_core_contract.md" "/mnt/f/unity projects/ark/docs/probe/unity_ws_smoke_test_contract.md" "/mnt/f/unity projects/ark/docs/probe/unity_ws_smoke_probe.py"
result: found expected negative assertions for removed map fields and two stale external Unity contract phrasings around retreat / Boss live battle
fix: updated `unity_core_contract.md` to document final-boss retreat exception and `Boss/Boss` as a live DefenseRoute-derived combination
revalidation: bytecode-free probe syntax check passed again

cargo fmt
result: passed after adding Gate preview/cancel coverage
fix: n/a
revalidation: cargo test -p game_core game::world::tests::map_flow -- --test-threads=1

python3 - <<'PY'
from pathlib import Path
path = Path('/mnt/f/unity projects/ark/docs/probe/unity_ws_smoke_probe.py')
compile(path.read_text(), str(path), 'exec')
print('syntax ok')
PY
result: passed after external contract wording updates
fix: n/a
revalidation: n/a

cargo test -p game_core game::world::tests::map_flow -- --test-threads=1
result: passed, 14 passed including `gate_preview_does_not_advance_act_or_apply_reward_without_confirm`
fix: n/a
revalidation: n/a

cargo check -p game_core && cargo check -p game_server
result: passed
fix: n/a
revalidation: n/a

cargo test -p game_core game::map::progression -- --test-threads=1
result: passed, 9 passed
fix: final audit found Completed transit-only was still indirect, so added `completed_nodes_are_transit_only_not_selectable_destinations`
revalidation: see next command

cargo fmt && cargo test -p game_core game::map::progression -- --test-threads=1 && cargo check -p game_core && cargo check -p game_server
result: passed; progression module passed 10 tests including Completed transit-only/non-selectable coverage; core and server checks passed
fix: n/a
revalidation: n/a
```

## Runtime/Data Findings Rechecked

- `RunProgression` now stores `GameMode` and exposes `run_progression.game_mode`.
- `MapNodeCategory` now includes `Gate`, and live RON includes `gate_stairs`.
- Unity-facing `MapViewDto` no longer exposes `edges`, boss id, terminal id, or public selection id lists.
- Public snapshot uses `map_navigation.current_node_id` and `map_navigation.selectable_node_ids`.
- `handle_retreat_battle()` now allows final boss retreat before defeat while attempts remain, and final boss exhaustion/defeat fails the run.
- Run policy RON now includes `support.gate_transition_trauma_recovery_percent: 10`.
- Core navigation uses `selectable_node_ids()` through Completed transit nodes and honors internal edge direction.
