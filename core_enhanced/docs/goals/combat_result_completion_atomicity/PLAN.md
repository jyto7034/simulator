# Combat Result Completion Atomicity

## Objective

Fix the `CompleteCombatResult` flow so combat result finalization is one authoritative success unit: post-battle roster changes, combat consumable decrement, combat reward grants, battle outcome result construction, current map node completion, session cleanup, and `CombatResult -> ViewingMap` transition must either all succeed together or leave no user-visible partial mutation.

This goal exists because Unity can receive a `combat_result` snapshot with `allowed_actions: ["CompleteCombatResult"]`, send `complete_combat_result`, and receive `invalid_action`; server logs show reward XP grants repeating on retry without a successful `CombatResult -> ViewingMap` transition.

## Source Of Truth Order

1. Runtime code and focused reproduction tests.
2. Live server logs and Unity wire logs.
3. Live RON/data and current reward/map policies.
4. Unity-facing snapshot/command/result DTO contracts.
5. Existing policy documents.

Do not trust docs alone. Re-read the runtime path before editing.

## Rechecked Evidence

- Server log: `../game_server/logs/app.log.2026-06-23:259` records `InBattle -> CombatResult`.
- Server log: `../game_server/logs/app.log.2026-06-23:262`, `:265`, `:268` records three XP grants after Continue.
- Server log: `../game_server/logs/app.log.2026-06-23:274`, `:277`, `:280` records the same three XP grants again on retry.
- Server log: no matching `CombatResult -> ViewingMap` transition appears between those grant lines.
- Unity wire log: `/mnt/c/Users/blast/AppData/LocalLow/DefaultCompany/ark/ark_wire_messages.log:22156` receives `invalid_action`.
- Unity wire log: `/mnt/c/Users/blast/AppData/LocalLow/DefaultCompany/ark/ark_wire_messages.log:22157` sends `{"behavior":{"type":"complete_combat_result"}}`.
- Runtime code: `src/game/managers/action_scheduler.rs:88` allows `CompleteCombatResult` from `GameState::CombatResult` based on high-level state only.
- Runtime code: `src/game/world/combat.rs:1252-1372` mutates post-battle roster state, decrements consumables, applies combat rewards, builds outcome, then calls `handle_complete_node()`.
- Runtime code: `src/game/world/node_flow.rs:152-184` can return `InvalidAction` while completing the map node, after earlier combat result mutations have already happened.
- Runtime code: `src/game/world/reward.rs:107-146` already applies reward sessions using clone/preflight/commit internally, so the gap is the wider combat-result transaction boundary rather than grant executor internals alone.
- Runtime code: `src/game/map/progression.rs:90-124` requires a valid `current_node_id`; if map progression is not consistent with `CombatResult`, completion fails.

## Current Diagnosis

The likely root cause is not Unity button mapping. Unity sends the expected command while the snapshot advertises `CompleteCombatResult`.

The core bug is that `handle_complete_combat_result()` treats combat finalization as a sequence of independently mutating steps. `apply_post_battle_resolution()`, `decrement_consumables_after_combat_node()`, and `apply_reward_session_with_context()` can commit state before `handle_complete_node()` proves that map node completion and state transition can succeed. If `handle_complete_node()` fails, the command returns `InvalidAction`, but earlier mutations remain and can be repeated on the next Continue.

This also means `allowed_actions` is too coarse for this command: the action is advertised solely by `CombatResult` state even if required lower-level invariants such as `node_session`, active combat content, and map progression are broken.

## Plan

1. Build a focused failing regression that reproduces `CompleteCombatResult` failure after combat-result mutation would otherwise be applied.
   - Prefer a test that enters a real combat node, reaches `CombatResult`, records roster XP/reward-relevant state, corrupts or constructs a missing completion invariant, calls `CompleteCombatResult`, and asserts no XP/reward/consumable/post-battle mutation is committed on error.
   - Also cover retry behavior: repeated failed Continue must not stack XP or duplicate side effects.
2. Re-read `handle_complete_combat_result()`, `handle_complete_node()`, reward claim, post-battle resolution, consumable decrement, battle record, and state transition helpers before editing.
3. Introduce an explicit combat result completion transaction boundary.
   - Do not add a compatibility fallback.
   - Do not merely suppress `InvalidAction`.
   - Prefer staged state or clone/preflight/commit over rollback after partial mutation.
4. Preflight all completion invariants before committing user-visible mutation.
   - Active node content must be combat battle content matching the result.
   - `node_session` must exist and correspond to the current combat node.
   - run state, map, and `map_progression.current_node_id` must support completion.
   - completion must not require support-node side effects for combat nodes.
5. Stage the full success path.
   - Post-battle roster/trauma/XP changes.
   - Combat-node consumable decrement.
   - Combat reward grants and diffs.
   - Map progression/node completion.
   - session and active node content cleanup.
   - `CombatResult -> ViewingMap`, `RunComplete`, or `ActComplete` transition as appropriate.
6. Commit staged state only after all validations and staged effects succeed.
7. Align `allowed_actions`/snapshot behavior with the final invariant if needed.
   - If `CompleteCombatResult` cannot succeed because required state is missing, the snapshot should not advertise it, or the inconsistency should be treated as a runtime bug caught before snapshot emission.
8. Update tests around user-visible behavior and Unity-facing result contract.
   - Successful player victory still returns `CombatRewardsGranted` with completion payload and reward diffs.
   - Failed completion does not mutate roster XP, rewards, consumables, map progression, node session, or game state.
   - Retry after a failed completion does not duplicate grants.
9. Run focused tests after each small change, then broader validation.
10. At completion, review implementation against the reusable review document and record whether the solution is truly long-term rather than a narrow test patch.

## Completion Conditions

- `CompleteCombatResult` is all-or-nothing for player-victory combat result completion.
- Retrying a failed `CompleteCombatResult` cannot duplicate XP, reward, consumable, post-battle, or map progression side effects.
- `CombatResult` snapshots and `allowed_actions` no longer advertise a command that the handler rejects due to predictable local invariants.
- Existing reward session atomicity remains intact and is not replaced by a weaker path.
- Focused regression tests cover the observed Unity/server failure shape.
- A successful live or test flow reaches `CombatResult -> ViewingMap` after Continue.
- No legacy fallback, dual schema, ignored legacy test, or compatibility layer is introduced.
- Any newly discovered policy decision is not guessed. The goal is completed with a report and question list for the user.

## Policy Stop/Completion Rule

If implementation reveals a policy decision not already covered, complete this goal and report the exact question list instead of continuing. This includes:

- Whether a partially applied combat result should ever be recoverable.
- Whether `allowed_actions` should hide locally invalid commands or expose a diagnostic action.
- Whether combat battle records should be written before or after node completion commit.
- Any Unity-facing DTO shape change.
- Any change to reward/XP/consumable timing visible to the player.

## Validation Commands

Use focused commands first, then broaden:

- `cargo test -p game_core <focused_combat_result_atomicity_test> -- --nocapture`
- `cargo test -p game_core game::world::tests::combat -- --nocapture`
- `cargo test -p game_core`
- If server behavior is touched: run `game_server` locally and replay a Unity/probe Continue flow against `ws://127.0.0.1:18083/game`.

## Completion Report

Status: complete by implementation and verification on 2026-06-23.

Change summary:

- Added a staged combat-result completion path for player-victory `CompleteCombatResult`.
- Split map node completion into planning and commit helpers so combat result completion can preflight map progression before committing post-battle/reward state.
- Reused the canonical grant executor against staged inventory/fragment/roster/uuid/enkephalin state.
- Made `get_allowed_actions()` and command execution read current local invariants, so `CompleteCombatResult` is not advertised when the combat result cannot locally complete.

Removed legacy:

- No compatibility fallback, retry marker, idempotency flag, or dual completion path was added.

New contracts fixed:

- Player-victory combat result completion is all-or-nothing for post-battle roster updates, combat consumable decrement, combat reward grants, map node completion, session cleanup, and state transition.
- Failed local completion does not mutate XP/reward state and does not expose `CompleteCombatResult` in `allowed_actions`.

Tests updated:

- Added `combat_result_completion_failure_does_not_partially_apply_rewards_on_retry`.
- The regression first failed on the old implementation because employee XP became `20` after a failed completion with expected `0`.
- The final regression directly calls `handle_complete_combat_result()` twice after corrupting `map_progression.current_node_id`, and verifies XP, Enkephalin, equipment inventory, map progression, node session, and game state remain unchanged while `CompleteCombatResult` is absent from `allowed_actions`.

Remaining risk:

- Non-player combat result branches still use the older direct mutation order. The observed Unity bug and this goal's verified atomicity contract cover player-victory combat reward completion. Broader non-player result transaction unification remains a follow-up candidate if needed.
- Live server probe was not rerun in this goal turn because package-level runtime tests covered the core contract. A Unity/server smoke replay is still useful if the exact saved Unity session must be reproduced.

Validation commands run:

- `cargo test -p game_core combat_result_completion_failure_does_not_partially_apply_rewards_on_retry -- --nocapture`
- `cargo test -p game_core game::world::tests::combat -- --nocapture`
- `cargo test -p game_core game::world::tests::snapshots_and_start -- --nocapture`
- `cargo test -p game_core game::world::tests::map_flow -- --nocapture`
- `cargo test -p game_core game::world::tests::node_sessions -- --nocapture`
- `cargo test -p game_core`
