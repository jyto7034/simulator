# Experiments

| Date | Attempt | Result | Follow-up |
|---|---|---|---|
| 2026-06-23 | Goal setup | Created the subgoal document for redeploy lifecycle policy. | Run after `core_runtime_unit_lifecycle`. |
| 2026-06-23 | Redeploy runtime inventory | Reading deploy/withdraw/death/redeploy paths showed that withdraw HP carry is implementable, while the old updated-trauma death HP wording conflicted with post-battle-only trauma timing. | Resolved by policy: death redeploy uses 60% of the new runtime max HP and does not require live trauma mutation. |
| 2026-06-23 | Draft-level HP override trial by inspection | Re-reading `effective_stats_for_draft` showed that changing `BattleUnitDraft` current HP would be ratio-scaled by equipment/artifact stat calculation, so it was not the correct place for an absolute runtime redeploy HP policy. | Apply redeploy HP after `spawn_scenario_group` creates the new `RuntimeUnit`, before spawn/deploy events are recorded. |
| 2026-06-23 | Redeploy HP implementation | Added `BattleDeployCurrentHpPolicy`, stored it in `LiveBattleRedeployState`, passed it through `BattleLiveCommand::DeployPlayerUnit`, and applied it to the newly spawned runtime unit before event logging. | Run focused deploy/withdraw/death tests. |
| 2026-06-23 | Focused test command syntax | `cargo test live_defense_deploy_and_withdraw_manage_cost_and_redeploy_lock live_deployment_reconciles_defeated_player_unit_before_state_dto` failed because Cargo accepts one test-name filter before `--`. | Re-ran the two focused tests as separate commands. |
| 2026-06-23 | Focused validation | `cargo test live_defense_deploy_and_withdraw_manage_cost_and_redeploy_lock` and `cargo test live_deployment_reconciles_defeated_player_unit_before_state_dto` both passed. | Run `cargo check` and broad `cargo test`. |
| 2026-06-23 | Broad validation | `cargo check` and full `cargo test` passed. | Complete review and update master status. |
