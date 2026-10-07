# Game Rulebook Policy Implementation Master Phase 04 — Notes

## Structural Judgment

- Sweep is one run-local node-resolution transaction, not a battle mode and not a permanent profile feature.
- Eligibility projection and command validation must share one evaluator; server/client inference is not authority.
- Existing reward/XP/duration reducers should be composed through a cloned plan. Reusing a full battle victory result
  is prohibited if it imports any forbidden side effect.

## Source-Of-Truth Boundary

- P02 run progression owns mastery, source completion, fragment identity and mastered repeat pool/cursor.
- Current node/session owns which assigned Elite can be resolved now.
- Core owns typed capability/command/result and the synthetic atomic commit.
- Active Run V4/checkpoint owns all sweep effects and continuation; PlayerProfile V2 owns none.
- game_server forwards the typed Core contract and persistence ordering; no sweep-specific server reducer.

## Dirty Worktree Warning

- The repository remains broadly dirty from user work and P01~P03, including likely P04 target paths.
- Production files require current hunk-level diff review before edit. No reset/stash/restore, unrelated formatting or
  cleanup is authorized.
- P03 formatting was restricted to its touched files; P04 will follow the same scoped rule.

## Consumer Scope

- Confirmed development boundary: Core, production live RON when relevant and game_server.
- Unity source/runner/client is explicitly unverified and unmodified. No compatibility alias, ignored command or
  local eligibility fallback will be created for it.
- Public release is not planned; exact current V4/V2 and internal atomicity still remain completion gates.

## Resolved Production Authority

- Direct battle remains `ConfirmEnterNode`; sweep is a separate common action shown beside it in `NodeConfirm` with a
  Core-owned disabled reason. Execution reuses the same sweep evaluator.
- A valid sweep requires the current node to be an Endless Elite whose authored source is complete, primary
  abnormality has Monolith victories 3, source remains in the mastered assigned set, and the roster has at least one
  `is_battle_deployable()` employee.
- Base rewards resolve from the current authored-combat override or encounter rewards with the current difficulty
  reward multiplier. Existing live Elite rewards are `ClaimAll`.
- Full living-roster XP uses `run_policy.growth.post_battle_survival_xp` through one ordered
  `SelectedEmployee` grant per living employee. This avoids the generic `AliveRoster` total-pool division, is not
  participant XP and does not run post-battle condition logic.
- Active-consumable duration uses the existing per-employee decrement operation exactly once on the staged roster.
- `EconomyGrantState` is the five-root reward transaction. Existing staged node completion owns map, session,
  navigation and Floor transition semantics.
- `SkillFragmentInventory` removes only the zero-count stack on final dismantle; its separate progress map remains, so
  upgrade/awakening survives reacquisition.
- The staged sweep result is actor-transient and must not change exact Active Run V4 or PlayerProfile V2. Its stored
  commit plan makes reconnect projection stable without confirmation-time replanning.

## Final Architecture

- `SweepMasteredElite {}` stages `MasteredEliteSweepResultDto`; `CompleteMasteredEliteSweep {}` consumes the exact
  stored commit plan. Both strict empty structs reject unknown compatibility fields.
- The transient `MasteredEliteSweepResult` world state is not serialized into V4. Reconnect to the live actor reads
  the same plan; durable Continue remains the last committed SavePoint by existing contract.
- Confirmation revalidates exact node/session/encounter, available state, mastery, source assignment and deployable
  roster but never recomputes reward choices or UUIDs.
- The generic server payload transport carries the typed Core result and explicitly contains no `battle_update`,
  `result_stats` or `event_log` fields.
- PlayerProfile is preserved exactly across sweep completion. Checkpoint load restores all committed run-local
  sweep effects and clears transient staged state while monotonic load metadata remains outside the checkpoint.

## Remaining Discovery

없음 for the approved P04 boundary. Unity consumption and final content/balance are deliberate future scope, not
unresolved implementation discovery.

## Alternatives Not Applied

- No auto-resolve on node selection and no hidden boolean command.
- No fake `CombatBattleState`, zero-duration battle, synthetic event log or forced `CombatResult`.
- No dedicated sweep item, currency/Enkephalin charge, cooldown, daily counter or permanent unlock.
- No combined fragment + repeat reward when owned count is zero.
- No mastery/source reset on dismantle or reacquisition.
- No Standard/Boss/FinalBoss/Monolith sweep and no new content.
- No Active Run version/profile migration placeholder before evidence requires it.

## Initial Risks

- Direct battle completion may currently bundle encounter progression, compendium and reward effects too tightly for
  safe partial reuse.
- Existing node completion may assume active battle/result lifecycle and need a pure staged seam rather than a
  parallel implementation.
- Duration semantics could be attached to combat completion rather than a reusable run progression reducer.
- Dismantle preserves progress records at zero count; repeated-cycle integration tests must still prove the whole
  source/mastery/progress lifetime.
- Broad dirty overlap may obscure ownership unless exact target paths and hunks are captured before mutation.

All five risks were closed by the isolated world module, cloned economy/node plan, selected-employee XP grants,
integration fingerprint/checkpoint tests and hunk-scoped edits. No correction item remains after the exit audit.

## Cross-Phase Impact

- P04 must consume P02's explicit source identity, 29 rows, mastery and pool cycle without redefining them.
- P04 may use P03's exact Active Run V4 run graph but cannot add permanent PlayerProfile state or affect fixed Standard.
- Master final audit will combine all four phases, so P04 evidence must explicitly prove no battle/profile side effects
  and re-run the V4/profile/Standard preservation seams.
