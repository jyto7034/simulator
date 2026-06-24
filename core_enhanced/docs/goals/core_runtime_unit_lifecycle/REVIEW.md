# Core Runtime Unit Lifecycle Review

Date: 2026-06-23

Guide: `docs/goal_completion_review_guide.md`

## Verdict Summary

| Item | Verdict | Long-term fit | Notes |
|---|---|---|---|
| `RuntimeUnitLifecycle` source of truth | Complete | High | `RuntimeUnit` now carries `Active`, `Withdrawn`, `Dead`; active gameplay gates use lifecycle instead of HP/action state. |
| Death lifecycle invariants | Complete | High | Death sets lifecycle `Dead`, HP 0, and `ActionState::Dead`; tests that manually fixture death now set lifecycle explicitly. |
| Withdrawn runtime registry behavior | Complete | High | Withdrawal keeps the old runtime unit in `BattleCore.units`, preserves final body position, and excludes it from active gameplay. |
| Withdrawal cleanup events | Complete | High | Buff and cast cleanup emit typed withdrawal reasons/events and validators understand them. |
| Official checkpoint Active-only units | Complete | High | Live checkpoint unit list and live deployment reconciliation filter by `unit.is_active()`. |
| Static layout / occupant residue during lifecycle pass | Complete | High | Remaining battle/test `units_at` fixture was removed; battle layout has no dynamic occupant query dependency. |

## Runtime Evidence

- `src/game/battle/core/types.rs`
  - `RuntimeUnitLifecycle { Active, Withdrawn, Dead }` exists.
  - `RuntimeUnit::is_dead()` is lifecycle-based.
  - `RuntimeUnit::is_active()` gates combat participation helpers.
- `src/game/battle/core/build.rs`
  - runtime unit spawn initializes lifecycle as `Active`.
  - `withdraw_unit` sets lifecycle to `Withdrawn`, keeps the unit in `BattleCore.units`, clears active runtime state, records `UnitWithdrawn`, expires withdrawal-related buffs, and cancels pending/active skill runtime with `SkillCastCancelled`.
- `src/game/battle/core/commands.rs`
  - death finalization sets lifecycle `Dead`, HP 0, and `ActionState::Dead`.
  - active candidate paths now reject inactive units.
- `src/game/battle/core/mod.rs`, `targeting.rs`, `movement/*`, `skill_runtime/*`, `target_usefulness.rs`
  - movement, targeting, skill, damage/heal/modifier, resonance, and live projection paths use active lifecycle gates where they select current gameplay participants.
- `src/game/world/state.rs`
  - live deployment reconciliation removes deployed entries whose runtime unit is not active.
  - official live checkpoint `units` includes Active units only.

## Contract Evidence

- `src/game/battle/event_log.rs`
  - added `BuffExpireReason::TargetWithdrawn` and `BuffExpireReason::CasterWithdrawn`.
  - added `SkillCastCancelReason::Withdrawn`.
  - added `BattleLogEvent::SkillCastCancelled`.
- `src/game/battle/validation/{autocast,deaths,parent,spawns,validator}.rs`
  - `SkillCastCancelled` is validated for spawned references, dead-unit operation rules, autocast pairing, and cast start relations.
  - pending cancellation references `AutoCastStart`/`ManualCastStart`; active cancellation references `AbilityCast`.

## Test Evidence

- Runtime tests:
  - `withdraw_clears_runtime_buffs_with_withdrawn_reasons`
  - `withdraw_cancels_pending_skill_cast_with_cancelled_event`
  - `withdraw_cancels_active_skill_cast_with_cancelled_event`
  - `apply_live_command_deploys_and_withdraws_player_unit`
  - `live_defense_deploy_and_withdraw_manage_cost_and_redeploy_lock`
- Validator test:
  - `validator_accepts_cancelled_active_skill_cast_referencing_ability_cast`
- Fixture cleanup:
  - `tests/skill_test/common/run.rs` now finds runtime units from active `BattleCore.units` and `RuntimeUnit.body.projected_tile()`, not layout occupant APIs.

## Legacy / Fallback Audit

- Search evidence:
  - `rg -n "units_at\\(|occupant\\(|position_of\\(|battlefield\\.remove\\(|battlefield\\.place\\(" src tests`
  - Only non-battle roster board `occupant` remains.
- Remaining `is_dead()` search evidence:
  - only win/loss checks, participant result survival, and movement DTO dead flags remain in battle core.
- No compatibility layer, fallback schema, dual lifecycle source, or ignored legacy test was added.

## Long-Term Direction Review

Fit: High.

Improvement class: none for this subgoal; follow-up subgoals remain for redeploy, inactive effects, and event position contracts.

Reason:
- Lifecycle is now an explicit runtime source of truth rather than inferred from HP or action state.
- Static battlefield layout remains terrain-only; runtime unit body owns dynamic position.
- Withdrawal cleanup is event-log-visible and typed, avoiding silent state disappearance or false death/interruption semantics.
- Tests now pin gameplay-visible behavior and DTO/checkpoint participation rather than private helper shape.

## Remaining Risk

- Redeploy HP/new instance behavior is intentionally out of scope and must be handled by `core_redeploy_lifecycle_policy`.
- Projectile/delayed-effect behavior for inactive units is intentionally out of scope and must be handled by `core_inactive_unit_effect_resolution`.
- `UnitWithdrawn` / `UnitDied` position fields and debug/admin inactive visibility are intentionally out of scope and must be handled by later lifecycle event/snapshot subgoals.

## Validation

- `cargo check` - passed.
- `cargo test --no-run` - passed.
- `cargo test withdraw_` - passed.
- `cargo test validator_accepts_cancelled_active_skill_cast_referencing_ability_cast` - passed.
- `cargo test --lib` - initially failed because old tests used `stats.current_health = 0` as death SoT; fixed tests to set `RuntimeUnitLifecycle::Dead`; rerun passed.
- `cargo test` - passed.
