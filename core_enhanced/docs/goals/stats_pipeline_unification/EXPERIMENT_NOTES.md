# Stats Pipeline Unification Experiment Notes

## 2026-06-12

- The worktree is already dirty with unrelated and prior-goal changes. Do not revert them.
- Source of truth order for this subgoal: runtime battle setup code, live RON/data, Unity-facing snapshot/command contracts, then design notes.
- Do not change balance numbers or live schema in this subgoal.
- If the modifier order is not currently policy-defined, stop and ask instead of inventing one.
- Runtime inspection found the existing order split across `Employee::combat_profile_for_battle` and `BattleUnitDraft::effective_stats`.
- Preserved current order as the explicit pipeline contract:
  1. base employee combat profile,
  2. skill fragment loadout,
  3. run HP / trauma battle-start condition,
  4. active consumable battle profile modifier,
  5. draft combat profile including equipped weapon profile overrides,
  6. growth stacks,
  7. equipped item Permanent effects,
  8. equipped item enhancement modifiers,
  9. artifact Permanent effects,
  10. current HP ratio preservation after max HP changes.
