# Policy Decision Report: Locked Effects Against Withdrawn Units

Status: 정책 확정

## Blocking Area

`core_inactive_unit_effect_resolution` cannot be implemented cleanly from the current confirmed policy alone.

Confirmed policies say:

- `Withdrawn` units are excluded from new movement, targeting, skill, and new damage candidates.
- Projectile / delayed effects that were launched, locked, or scheduled before withdrawal may resolve against a withdrawn target.
- `Dead` targets are excluded from damage/effect resolution to prevent duplicate damage.
- Withdraw redeploy HP is fixed from the HP at withdrawal:

```text
redeploy_hp = min(max_hp, withdrawn_hp + floor(max_hp * 0.30))
```

The unresolved part was what "resolve against a withdrawn target" means for:

- damage after withdrawal,
- non-damage effects such as buff/debuff/heal/stat modifier/resonance,
- whether post-withdraw locked damage should update the redeploy HP lock,
- whether post-withdraw locked damage can turn the old withdrawn runtime unit into `Dead`.

## Code Evidence

- `src/game/battle/core/commands.rs`
  - `advance_basic_attack_projectile` currently records a miss if the locked target is not `is_active()`.
  - `apply_basic_attack_projectile_hit_at` records hit first, but skips damage if the target is not `is_active()`.
  - `apply_damage_result_and_record` also returns early for non-active targets.
  - `process_commands` skips `ApplyDamage`, `ApplyModifier`, and other command families when the target is not active.
- `src/game/battle/core/skill_runtime/projectile.rs`
  - homing skill projectile advancement currently requires the locked target to be `is_active()`.
  - `apply_skill_projectile_impact` filters `first_hit_unit_id` through `unit.is_active()`.
- `src/game/battle/core/build.rs`
  - withdrawal removes battle runtime buffs and records withdrawal-specific cleanup events.
- `src/game/world/combat.rs` and `src/game/world/state.rs`
  - withdraw redeploy HP policy is stored in `LiveBattleRedeployState` at withdrawal time.
  - later damage to the old withdrawn runtime unit would not currently affect the redeploy lock's HP.

Therefore, simply changing `is_active()` to "not dead" in projectile resolution would create a new gameplay meaning that is not settled by the current policy.

## Concrete Options

### Option 1: Damage-only old-runtime resolution, redeploy HP unchanged

Already locked damage/projectile impacts can hit and damage the old withdrawn `RuntimeUnit`; `Dead` is still excluded. Non-damage effects such as buffs, debuffs, heals, stat modifiers, resonance, and aura-like effects remain skipped for withdrawn targets.

Redeploy HP remains the value captured at withdrawal time.

Pros:
- Preserves the already confirmed withdraw redeploy HP formula.
- Does not reintroduce battle buffs/debuffs on withdrawn inactive units.
- Keeps withdrawal cleanup policy intact.

Cons:
- Superseded assumption: this option treats withdrawal as not being an evasion button for event/debug/old runtime HP, while still preserving the next redeployed unit's HP.
- A post-withdraw hit could kill the old withdrawn runtime entity while the employee still has a withdraw redeploy lock with fixed HP.

### Option 2: Locked damage updates the redeploy HP lock

Already locked damage/projectile impacts can damage the withdrawn old runtime unit, and the live redeploy lock's carried HP is updated from that post-withdraw result. If the old withdrawn unit is killed by already locked damage before redeploy, the lock converts to the death redeploy policy.

Pros:
- Superseded assumption: strongest gameplay reading if withdrawal were not meant to evade already committed incoming damage.
- Incoming committed damage still matters to the employee's next deployment.

Cons:
- Changes the meaning of the confirmed withdraw HP formula from "HP at withdrawal" to "HP after pre-withdraw locked damage resolves".
- Requires `BattleCore` damage resolution to inform world-level `LiveBattleRedeployState`, or introduces a new live signal.
- Larger blast radius and timing complexity.

### Option 3: Locked effects miss/skip withdrawn targets

Keep current behavior for most projectile/effect paths: withdrawal makes already locked effects miss or skip.

Pros:
- Smallest implementation.
- Avoids redeploy HP and old-runtime death ambiguity.

Cons:
- Superseded concern: this would contradict a policy where withdrawal was not an evasion action. User later confirmed withdrawal should strongly evade opponent hostile projectiles, so this concern no longer applies to hostile projectile resolution.

### Option 4: Full locked effect resolution against withdrawn units

Damage, heal, buff/debuff, stat modifier, resonance, and other effects all resolve against withdrawn targets if they were already locked/scheduled.

Pros:
- Broadest literal interpretation of "effect resolution".

Cons:
- Conflicts with the confirmed withdrawal cleanup policy that withdrawn units are no longer battle buff/debuff/stat/aura targets.
- Can reintroduce inactive runtime state that redeploy does not carry, making effects visually/logically confusing.

## Decision

Use Option 3 for projectile/effect targets that become `Withdrawn`.

Confirmed policy:

- Withdrawal is intentionally a strong defensive/evasion action against incoming hostile projectiles fired by the opponent.
- A hostile projectile already launched or locked onto the unit being withdrawn is cancelled/missed if the target becomes `Withdrawn` before impact.
- A projectile aimed at a withdrawn unit does not damage the old withdrawn runtime unit.
- It does not update the redeploy HP lock.
- It does not convert the old withdrawn runtime unit into `Dead`.
- Withdraw redeploy HP remains the value captured at withdrawal time:

```text
redeploy_hp = min(max_hp, withdrawn_hp + floor(max_hp * 0.30))
```

- Non-projectile delayed effects also do not apply new damage/buff/debuff/heal/stat modifier/resonance to `Withdrawn` targets unless a future effect family explicitly opts into a new policy.
- `Dead` targets remain excluded from damage/effect resolution.
- Withdrawal should not be implemented by clearing all projectiles globally. Keep projectile records/events deterministic and cancel/miss them when their target is observed as `Withdrawn` during advance/impact resolution.

## Superseded Recommendation

Earlier recommendation was Option 1 unless the intended gameplay meaning was that withdrawal should strongly evade incoming projectiles.

That recommendation is superseded. User confirmed that the attacker in this case is the opponent and that withdrawal should strongly function as avoidance: hostile projectiles flying toward the withdrawing unit should cancel.

## Implementation Direction

- Candidate selection remains Active-only.
- Basic attack projectile advancement should treat `Withdrawn` target as miss/cancel, while keeping existing `Dead` miss/no-damage behavior.
- Skill homing projectile advancement and projectile impact should treat `Withdrawn` target as no-hit/cancel.
- Command processing should keep skipping non-active targets.
- Tests should pin that a projectile locked before withdrawal does not damage the withdrawn unit and does not change redeploy HP.

## Goal Status

Policy is confirmed. Implementation can resume.
