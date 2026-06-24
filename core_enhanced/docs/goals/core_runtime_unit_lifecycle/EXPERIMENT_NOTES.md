# Experiment Notes

- This goal is intentionally separated from redeploy. First make runtime lifecycle and active filtering correct, then implement redeploy policy.
- Do not remove dead/withdrawn units from `BattleCore.units`; the registry must preserve references for events, debug, replay, and delayed effects.
- Do not encode withdrawn by moving the unit offboard or deleting its position.
- Future carryover effects must not be modeled by keeping battle buffs on withdrawn units.
- Policy-decision stop triggered before implementation: withdrawal cleanup needs event-log contract semantics for buff expiration and active/pending cast cancellation. Current `BuffExpireReason` only has death/natural/replaced variants, and `SkillCastInterrupted` does not naturally represent lifecycle withdrawal cleanup.
- Partial lifecycle edits that had been started during inventory were reverted before writing the report; `core_battlefield_layout_extraction` changes remain intact.
- Implementation resumed after policy confirmation. `RuntimeUnitLifecycle` is now the runtime source for active/dead/withdrawn participation, while `ActionState::Dead` and HP 0 remain invariants for dead units.
- Withdrawal now transitions the unit to `Withdrawn` instead of deleting it from `BattleCore.units`; the unit keeps its final `body.position`/projected tile for references, debug, replay, and later redeploy policy.
- Active gameplay loops should use `RuntimeUnit::is_active()` when selecting movement, targeting, skill, trigger, damage/heal/modifier, checkpoint, or deployment candidates. Remaining `is_dead()` uses are intentional death semantics such as win/loss checks, participant survival, and movement DTO dead flags.
- Active and pending skill cancellation have different parent seq sources: pending cancellation references `AutoCastStart`/`ManualCastStart`; active runtime cancellation references the `AbilityCast` seq. The validator now treats all three as valid cast starts for `SkillCastInterrupted`/`SkillCastCancelled` relation checks.
- `tests/skill_test/common/run.rs` had a stale single-tile occupant/layout query. It was corrected to derive unit lookup from active runtime units and their projected tile, matching the static-only `BattlefieldLayout` policy.
- Inactive projectile/delayed-effect resolution is still out of scope for this subgoal. The current lifecycle pass only ensures inactive units are not selected as new active candidates and withdrawal-owned active skill runtime is cancelled.
