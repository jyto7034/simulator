# Combat Result Atomicity Regression Repair

## Objective

Re-audit and repair the `CompleteCombatResult` transaction boundary if current runtime code has regressed from the previously completed `combat_result_completion_atomicity` goal.

The player-visible action must be atomic:

```text
confirm combat result
-> post-battle roster updates
-> rewards/research rewards
-> node completion/floor advance/run completion
-> state transition
```

These must all succeed together or leave no user-visible partial mutation.

## Why This Goal Exists

`docs/goals/combat_result_completion_atomicity` is marked complete, but current code should be re-read because `handle_complete_combat_result()` still appears to commit staged combat result state before `commit_staged_node_completion()`.

This goal must verify whether the existing completion goal remains true or whether later changes reintroduced a split commit boundary.

## Source Of Truth Order

1. Current runtime code.
2. Focused regression tests.
3. Existing completed goal documents.
4. Live RON/data.
5. Unity-facing command/result contract.

Do not assume the older completed goal is still accurate.

## Plan

1. Re-read:
   - `src/game/world/combat.rs`
   - `src/game/world/node_flow.rs`
   - `src/game/world/reward.rs`
   - current tests for combat result atomicity.
2. Determine whether current code still has a real partial-commit risk.
3. If no regression exists, update this goal's notes with code/test evidence and stop.
4. If risk exists, add a focused failing regression:
   - failure after reward/post-battle planning must not commit XP, inventory, skill fragments, enkephalin, research, consumables, map progression, node session, or game state.
   - retry must not duplicate rewards.
5. Implement a long-term repair:
   - plan all failure-prone work before mutation, or
   - stage all affected world state and commit once.
6. Keep side effects such as battle record file writing and checkpoint writing outside partial gameplay commit risk.
7. Run focused tests first, then broader tests.

## Re-Audit Result (2026-07-04)

Status: complete by re-audit and validation. No runtime code repair was required in this pass.

The current implementation still has a superficially split shape in the player-victory branch:

```text
plan_complete_current_node(false)
-> stage post-battle/reward/research state
-> commit_staged_combat_result_state(...)
-> commit_staged_node_completion(...)
```

However, the failure-prone node-completion invariants are already pulled forward before combat reward state is committed:

- `can_complete_combat_result_locally()` rejects local invariant failures before `CompleteCombatResult` is exposed in `allowed_actions`.
- `handle_complete_combat_result()` calls `plan_complete_current_node(false)` before staging and committing post-battle/reward state.
- `plan_complete_current_node(false)` clones map/progression, completes the current node on the clone, and generates the next floor/run-complete completion payload before reward commit.
- The combat-result path passes `apply_support_effect = false`, so the staged completion carries `StagedSupportEffect::None`; the support-node side-effect failure paths are not part of combat result completion.
- `transition_to()` is currently infallible in implementation; it recomputes allowed actions and returns `Ok(())`.

The existing regression test covers the original observed bug shape: corrupt local completion state prevents `CompleteCombatResult` from being advertised, direct handler retry returns `InvalidAction`, and repeated failures do not mutate XP, Enkephalin, equipment inventory, map progression, node session, or game state.

No new failing regression was added because the current code did not expose a concrete post-reward node-completion failure path after the existing preflight and local allowed-action checks. Adding an artificial test hook solely to force `commit_staged_node_completion()` to fail after reward commit would not reflect a real current runtime path.

Remaining design note: the code shape can still look suspicious because `commit_staged_combat_result_state()` is called before `commit_staged_node_completion()`. If future work makes combat result completion carry a fallible support/checkpoint/file side effect, it must either move that fallible work into preflight/staging or combine combat-result and node-completion commits into a single infallible commit helper.

## Follow-Up Direction: Design Clarity Refactor

The audit finding was a false positive for the current runtime bug, but the false positive happened for a legitimate reason: the transaction boundary is difficult to read.

Do not frame the follow-up as another atomicity bug fix unless a new concrete failing path is found. Frame it as a design refactor:

```text
Combat Result Completion Transaction Boundary Refactor
```

The follow-up objective should be:

- Preserve current player-visible behavior.
- Make the `CompleteCombatResult` all-or-nothing contract obvious from code structure.
- Separate failure-prone planning/preflight from commit-only application.
- Make the commit step read as already prevalidated, not as another fallible node-completion operation.
- Avoid idempotency markers, retry flags, compatibility fallbacks, or reward timing changes.

Recommended long-term shape:

```text
plan_combat_result_completion()
  -> may fail
  -> validates combat result content, node session, map progression, rewards, research, and completion outcome
  -> builds all staged post-battle/reward/research/node-completion state

commit_planned_combat_result_completion(...)
  -> should not perform new validation or fallible gameplay work
  -> applies the prevalidated staged state and transition
```

An acceptable smaller step is to rename/split the current node completion helpers so the contract is explicit:

```text
plan_complete_current_node(...)
commit_preplanned_node_completion(...)
```

The important naming rule is that `commit_*` should not look like it might discover new local invalidity after rewards have been committed. If a commit helper can still fail, the caller must not commit combat rewards before calling it.

This follow-up is about preventing future mistakes and repeated false-positive audits. It is not required to fix the current observed duplicate reward bug, which remains covered by existing tests.

## Completion Conditions

- Current code is proven safe by tests, or repaired to be safe. Done by re-audit and tests.
- `CompleteCombatResult` cannot partially apply rewards/post-battle state before node completion failure. The currently fallible node-completion invariants are preflighted before reward/post-battle commit.
- Existing successful result flows still return the correct `BehaviorResult`. Covered by combat/map flow tests.
- Failed completion cannot be retried for duplicate rewards. Covered by `combat_result_completion_failure_does_not_partially_apply_rewards_on_retry`.
- Any discrepancy with the older completed goal is documented. The older goal remains directionally accurate; this re-audit documents that the split-looking commit order is safe only because completion failure is preflighted and combat support effects are `None`.
- No idempotency marker or compatibility fallback is used to mask partial commits. Confirmed; none was added.

## Validation Commands

- `cargo test -p game_core combat_result_completion --lib -- --test-threads=1`
- `cargo test -p game_core game::world::tests::combat --lib -- --test-threads=1`
- `cargo test -p game_core game::world::tests::map_flow --lib -- --test-threads=1`
- `cargo check -p game_core`

Adjust filters after reading actual current test names.

## Validation Run (2026-07-04)

- `cargo test -p game_core combat_result_completion_failure_does_not_partially_apply_rewards_on_retry --lib -- --test-threads=1` passed.
- `cargo test -p game_core game::world::tests::combat --lib -- --test-threads=1` passed.
- `cargo test -p game_core game::world::tests::map_flow --lib -- --test-threads=1` passed.
- `cargo check -p game_core` passed.

## Stop Conditions

Complete the goal and report questions if:

- The intended behavior of SavePoint checkpoint write failure is unclear.
- Battle record file write is considered gameplay-authoritative rather than debug/codex side effect.
- Atomic repair requires changing reward timing, XP timing, or Unity-facing DTO shape.
