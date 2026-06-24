# Policy Decision Report: Death Redeploy HP Policy

Status: 정책 확정

## Blocking Area

`core_redeploy_lifecycle_policy` cannot be implemented cleanly from the current confirmed policy alone.

The confirmed policy says:

- withdraw redeploy HP is `min(max_hp, withdrawn_hp + floor(max_hp * 0.30))`;
- death redeploy HP was previously described as recalculated from latest run/employee state, equipment, level, and updated trauma;
- redeploy creates a new `RuntimeUnit` and `UnitInstanceId`;
- old inactive units remain in `BattleCore.units`.

The runtime code showed that an updated-trauma death redeploy policy would require a live-battle trauma timing decision. The policy is now simplified: death redeploy does not require immediate trauma mutation.

## Code Evidence

- `src/game/world/state.rs`
  - `ActiveBattleSession::reconcile_live_deployment_with_battle_state` notices when a deployed unit is no longer active and creates a redeploy lock.
  - This method only has `ActiveBattleSession` state and cannot mutate `EmployeeRoster`, trust reactions, injury state, or run HP.
- `src/game/world/combat.rs`
  - `handle_deploy_unit` builds a fresh `BattleUnitDraft` from the current roster via `battle_unit_draft_for_employee`.
  - `apply_post_battle_resolution` applies `EmployeeTrustResolver::modify_trauma` and `employee.apply_incapacitation(...)` after the battle, based on `ParticipantBattleResult`.
- `src/game/employee.rs`
  - `Employee::combat_profile_for_battle` uses current employee health/trauma to calculate battle start HP.
- `src/game/battle/core/sim.rs`
  - participant results mark `became_incapacitated` from final battle runtime HP.

Therefore, if a unit dies and becomes redeployable during the same live battle, the redeployed unit should not try to infer or preview post-battle trauma. It uses the new runtime unit's max HP and starts at 60%.

## Decision

Death redeploy HP is fixed to 60% of the new runtime unit's max HP.

Official formula:

```text
death_redeploy_hp = max(1, floor(new_runtime_max_hp * 0.60))
```

The value is clamped to `new_runtime_max_hp`.

Death redeploy still creates a new runtime unit through the normal deployment path, so max HP continues to reflect the current roster/equipment/level/profile state. Only current HP is overridden to the 60% death redeploy value.

Live-battle death does not immediately apply employee trauma/injury for the purpose of redeploy HP. Existing post-battle resolution remains responsible for persistent incapacitation consequences.

## Rejected Options

1. Apply live incapacitation immediately on deployed unit death.
   - When live deployment reconciliation detects an Active -> Dead unit for an employee, mutate the persistent employee state immediately:
     - apply trust-modified incapacitation trauma,
     - apply run HP loss/injury,
     - record that this battle already applied defeat consequences for that employee.
   - Death redeploy then uses the normal `battle_unit_draft_for_employee` path, so HP is recalculated from the updated employee state.
   - Post-battle resolution skips duplicate incapacitation for employees already resolved during this battle.
   - Pros: true source of truth; redeploy HP comes from actual roster state; no hidden one-off HP override.
   - Cons: changes injury/trust/trauma timing from post-battle-only to live-battle death time; needs idempotence tracking.
   - Rejected because the new fixed 60% HP policy avoids this timing complexity.

2. Store a death redeploy HP override without mutating the roster until post-battle.
   - On death, compute the HP that would result after trauma, store it in redeploy lock, and apply it to the next draft.
   - Pros: smaller local change.
   - Cons: creates a second source of truth; Unity/player roster state still shows old trauma while redeploy HP reflects hypothetical trauma; post-battle later mutates the roster.
   - Rejected because it keeps the old updated-trauma premise.

3. Keep death redeploy locked until battle end.
   - Death still creates a redeploy lock visually, but actual redeploy is impossible until post-battle consequences are applied.
   - Pros: avoids in-battle trauma timing.
   - Cons: contradicts the confirmed redeploy policy and likely player expectation for live redeploy.
   - Rejected because death redeploy should remain available during live battle after cooldown.

## Recommendation

Use the confirmed fixed 60% death redeploy HP policy.

Reason:

- It is simpler than live trauma timing and idempotent post-battle bookkeeping.
- It keeps persistent trauma/injury timing in the existing post-battle resolution path.
- It still creates a fresh runtime unit and uses that new unit's max HP as the source for the 60% value.
- It clearly distinguishes death redeploy from withdraw redeploy:
  - withdraw redeploy preserves old runtime HP plus 30% max HP recovery;
  - death redeploy discards old runtime HP and starts at 60% of the new runtime max HP.

## Suggested Follow-Up Policy Wording

When an employee's active live battle unit enters `RuntimeUnitLifecycle::Dead`, live deployment reconciliation creates a death redeploy lock without mutating persistent employee trauma/injury state. Persistent incapacitation consequences remain in post-battle resolution.

Death redeploy creates a new runtime unit and new `UnitInstanceId` through the normal deploy draft path. After the new unit's max HP is calculated, current HP is set to:

```text
max(1, floor(max_hp * 0.60))
```

Withdraw redeploy remains different: it does not mutate employee trauma and uses the confirmed withdraw HP formula based on the old runtime unit's HP at withdrawal.

## Goal Status

Policy is confirmed. Implementation can resume without adding live-battle trauma mutation or post-battle duplicate-incapacitation tracking.
