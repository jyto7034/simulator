# Experiment Notes

- This goal depends on lifecycle being canonical; do not implement redeploy by deleting old units or reusing old instance IDs.
- Withdraw redeploy carries only HP according to the confirmed formula. It does not carry battle buffs, pending casts, movement intent, action locks, or skill runtime state.
- Death redeploy is a fresh deployment that sets current HP to 60% of the new runtime unit's max HP. It does not require live-battle trauma mutation.
- Inventory finding: current deploy already creates a new `RuntimeUnit` through increasing `LiveBattleDeploymentState.next_instance_salt`, so identity policy is mostly aligned.
- Inventory finding: withdrawal redeploy HP can be implemented by storing a withdraw-only HP carry value in live redeploy state and applying it to the next draft.
- Resolved policy: death redeploy no longer uses "updated trauma" as the HP source. Current code can keep incapacitation/trauma in `apply_post_battle_resolution`; redeploy implementation should override only the new runtime unit's current HP to 60% of its max HP.
- Implementation correction: do not apply redeploy HP by mutating `BattleUnitDraft` before `effective_stats_for_draft`. That path preserves the base-profile HP ratio across equipment/artifact max-HP changes and can rescale an intended runtime HP value.
- Implemented source boundary: `LiveBattleRedeployState` owns the next-deploy HP policy because it is deployment/redeploy state; `BattleCore` owns applying that policy after the new unit's final runtime max HP is known.
- `BattleDeployCurrentHpPolicy` is command-scoped, not a compatibility layer. Normal first deployment passes `None`; redeploy locks pass an explicit fixed or percent policy.
- Death redeploy uses `BattleDeployCurrentHpPolicy::death_redeploy()` (`PercentOfMax(60)`) and does not mutate persistent employee trauma/injury during live battle.
- Withdraw redeploy uses `BattleDeployCurrentHpPolicy::withdraw_redeploy(current_hp, max_hp)` and carries only HP, not buffs, casts, movement, action locks, targets, or cooldown state.
- Review finding: applying HP before `record_spawned_units` keeps `UnitSpawned` stats and checkpoint stats consistent.
