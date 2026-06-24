# Core Redeploy Lifecycle Policy Review

Review guide: `docs/goal_completion_review_guide.md`

## Item: Withdraw / Death Redeploy Identity And HP

Master item:
- `core_followup_policy_implementation_master` item 4, `core_redeploy_lifecycle_policy`.

Subgoal:
- `docs/goals/core_redeploy_lifecycle_policy`

Policy source:
- `docs/goals/core_component_refactor_master/POLICY_DECISIONS.md`, Runtime lifecycle decisions 7, 8, 9, 11, 12, 13, 14, and 15.
- `docs/goals/core_redeploy_lifecycle_policy/POLICY_DECISION_REPORT.md`.

Expected behavior:
- Withdraw redeploy creates a new runtime unit and starts at `min(max_hp, withdrawn_hp + floor(max_hp * 0.30))`.
- Death redeploy creates a new runtime unit and starts at `max(1, floor(new_runtime_max_hp * 0.60))`, clamped by max HP.
- Old withdrawn/dead runtime units remain in `BattleCore.units`.
- `LiveBattleDeploymentState.deployed_units[employee_uuid]` points to the newest active unit instance.
- The new redeployed unit does not inherit battle runtime buffs, pending casts, movement intent, targets, or action locks.

Runtime evidence:
- `src/game/battle/core/sim.rs`
  - `BattleDeployCurrentHpPolicy` defines explicit fixed-current-HP and percent-of-max deploy policies.
  - `withdraw_redeploy` and `death_redeploy` are named constructors for the confirmed HP formulas.
  - `BattleLiveCommand::DeployPlayerUnit` carries `current_hp_policy: Option<BattleDeployCurrentHpPolicy>`.
- `src/game/battle/core/build.rs`
  - `deploy_player_unit` creates a new live deploy group through `instance_salt`.
  - After `spawn_scenario_group` creates the new `RuntimeUnit`, `current_hp_policy` is applied against the final runtime `max_health`.
  - HP policy is applied before `record_spawned_units`, so `UnitSpawned` event stats and checkpoint stats agree.
- `src/game/world/state.rs`
  - `LiveBattleRedeployState` stores the next-deploy HP policy.
  - `reconcile_live_deployment_with_battle_state` creates death redeploy locks with `BattleDeployCurrentHpPolicy::death_redeploy()`.
- `src/game/world/combat.rs`
  - `handle_withdraw_unit` creates withdraw redeploy locks with `BattleDeployCurrentHpPolicy::withdraw_redeploy(current_hp, max_hp)`.
  - `handle_deploy_unit` passes the lock's HP policy into `BattleLiveCommand::DeployPlayerUnit` and then removes the lock after successful deployment.

Data evidence:
- No live RON schema changed.
- The policy uses existing live deployment cooldown/cost values from run policy data.

External contract evidence:
- `checkpoint.deployment.redeploying_units` remains `employee_uuid`, `ready_at_ms`, and `deploy_cost`; HP policy is internal runtime state and is observed through the redeployed unit's checkpoint stats.
- Normal first deployment still sends no HP override.

Test evidence:
- `live_defense_deploy_and_withdraw_manage_cost_and_redeploy_lock`
  - asserts withdraw lock visibility,
  - redeploys the same employee,
  - asserts new `UnitInstanceId`,
  - asserts old withdrawn unit remains,
  - asserts redeployed current HP uses the withdraw formula,
  - asserts public runtime action state is reset.
- `live_deployment_reconciles_defeated_player_unit_before_state_dto`
  - asserts death lock visibility,
  - redeploys the defeated employee,
  - asserts new `UnitInstanceId`,
  - asserts old dead unit remains,
  - asserts redeployed current HP is 60% of the new runtime max HP,
  - asserts deployment DTO points to the new instance.

Legacy/fallback audit:
- No old runtime unit reactivation path was added.
- No compatibility layer or dual schema was added.
- No live trauma/injury mutation workaround was added.
- No projectile cleanup workaround was added.

Long-term direction review:
- Fit: high
- Improvement class: none
- Reason: redeploy state is the source for "why the next deployment has non-default HP", while BattleCore applies the policy only after the final runtime max HP exists. This avoids draft/profile HP rescaling and keeps event log/checkpoint stats consistent.

Verdict:
- Complete

Remaining risk:
- Projectile/delayed-effect interaction with inactive units is intentionally left for `core_inactive_unit_effect_resolution`.
- `UnitWithdrawn` / `UnitDied` event position contracts and debug/admin inactive visibility remain later subgoals.

## Validation

- `cargo fmt`: passed.
- `cargo test live_defense_deploy_and_withdraw_manage_cost_and_redeploy_lock`: passed.
- `cargo test live_deployment_reconciles_defeated_player_unit_before_state_dto`: passed.
- `cargo check`: passed.
- `cargo test`: passed.

## Cross-Component Findings

- No competing HP source was found between roster trauma/run HP and live redeploy HP. Death redeploy does not mutate persistent employee consequences; post-battle resolution remains the persistent consequence path.
- No Unity-facing DTO shape change was required.
- The remaining lifecycle policies are still correctly separated into later subgoals.
