# Core Redeploy Lifecycle Policy

## Objective

Implement confirmed withdraw/death redeploy identity and HP policies after `RuntimeUnitLifecycle` exists.

Confirmed policy source: `docs/goals/core_component_refactor_master/POLICY_DECISIONS.md`, decisions 7, 8, 9, 11, 12, 13, 14, and 15.

## Required Startup Protocol

At the beginning of this goal and every resumed run, read:

- `docs/goals/core_redeploy_lifecycle_policy/PLAN.md`
- `docs/goals/core_redeploy_lifecycle_policy/EXPERIMENTS.md`
- `docs/goals/core_redeploy_lifecycle_policy/EXPERIMENT_NOTES.md`
- `docs/goals/core_followup_policy_implementation_master/PLAN.md`
- `docs/goals/core_component_refactor_master/POLICY_DECISIONS.md`
- `docs/refactor_preparation_plan.md`

Do not start code search, edits, or validation before reading those documents.

## Scope

### In Scope

- Withdraw redeploy HP: `min(max_hp, withdrawn_hp + floor(max_hp * 0.30))`.
- Death redeploy HP: create a fresh runtime unit, then set current HP to `max(1, floor(max_hp * 0.60))`.
- Redeploy creates a new `RuntimeUnit` and new `UnitInstanceId`.
- `LiveBattleDeploymentState.deployed_units[employee_uuid]` points to the new unit instance.
- Old withdrawn/dead units remain in `BattleCore.units`.
- Redeploy unit does not inherit battle buffs, pending casts, movement goals, action locks, or skill runtime state.
- Redeploy cooldown/cost remains visible through `checkpoint.deployment.redeploying_units`.

### Out Of Scope

- Initial lifecycle introduction.
- Projectile/delayed-effect resolution rules.
- New debug/admin inactive unit query design.

## Implementation Plan

1. Read deployment, withdraw, death, redeploy, trauma, HP calculation, and live deployment state code.
2. Identify the current redeploy path and whether it reuses or replaces existing runtime units.
3. Implement new-instance redeploy for withdrawn units.
4. Implement death redeploy HP as 60% of the newly created runtime unit's max HP.
5. Ensure deployment state maps employees to the newest active unit instance.
6. Add tests for withdraw HP restore, death HP recalculation, old unit preservation, new instance identity, and runtime state reset.
7. Run focused deployment/battle tests and `cargo check`.

## Policy Decision Completion Condition

If implementation reveals unresolved balance/UX choices such as redeploy cooldown timing, deploy cost changes, trauma timing, or HP rounding beyond the confirmed formula, complete the goal with `사용자와 정책 논의 필요` and evidence.

## Completion Conditions

- Withdraw and death redeploy follow different confirmed HP policies.
- Redeploy never reactivates an old withdrawn/dead runtime unit.
- Old inactive units remain available for references inside the battle lifetime.
- Active deployment state points to the new unit instance.
- Focused and broad validation commands are recorded.

## Current Status

Status: implemented and reviewed.

Report: `docs/goals/core_redeploy_lifecycle_policy/POLICY_DECISION_REPORT.md`

Confirmed decision: death redeploy does not require live-battle trauma mutation. Death redeploy creates a fresh runtime unit and sets current HP to `max(1, floor(max_hp * 0.60))`, clamped to max HP. Persistent incapacitation consequences remain in post-battle resolution.

## Implementation Result

- `BattleDeployCurrentHpPolicy` applies redeploy HP after a new runtime unit's final max HP has been calculated and before `UnitSpawned` / `UnitDeployed` events are recorded.
- `LiveBattleRedeployState` stores the HP policy for the next deployment:
  - withdraw lock: fixed HP from `min(max_hp, withdrawn_hp + floor(max_hp * 0.30))`;
  - death lock: `PercentOfMax(60)`.
- Normal first deployment passes no HP override.
- Redeploy still creates a new `RuntimeUnit` through the existing `next_instance_salt` path.
- Old withdrawn/dead units remain in `BattleCore.units`; `deployed_units[employee_uuid]` points to the newly deployed instance.
- Focused tests now assert withdraw redeploy HP, death redeploy HP, old unit preservation, new instance identity, deployment state update, and active runtime state reset.

## Validation

- `cargo fmt`
- `cargo test live_defense_deploy_and_withdraw_manage_cost_and_redeploy_lock`
- `cargo test live_deployment_reconciles_defeated_player_unit_before_state_dto`
- `cargo check`
- `cargo test`
