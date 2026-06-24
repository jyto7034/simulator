# Allowed Actions Policy Experiment Notes

## 2026-06-12

- The worktree is already dirty with unrelated and prior-goal changes. Do not revert them.
- Source of truth order: runtime world state/action code, snapshot contract, tests, then design notes.
- Keep Unity-facing `allowed_actions` JSON shape unchanged.
- If a state/action allowance is ambiguous, stop and ask instead of inventing a new UX policy.
- Runtime inspection confirmed the policy split:
  - `ActionScheduler::get_allowed_actions(GameState)` owns the base state list.
  - `GameCore::allowed_actions_for_state_context` removes `ExitReward` when the current reward cannot be skipped.
  - `GameCore::allowed_actions_for_state_context` adds Maintenance actions when `GameState::InNode` and `active_node_content` is Maintenance.
- Current context adjustments to preserve:
  - `InReward` with `can_skip=false`: `ExitReward` is hidden.
  - `InReward` with `can_skip=true`: `ExitReward` remains available.
  - `InNode` with Maintenance content: equip/unequip item, equip/unequip skill fragment, upgrade/awaken/dismantle skill fragment, dismantle/enhance equipment become available.
  - Non-Maintenance support nodes must not expose Maintenance actions.
