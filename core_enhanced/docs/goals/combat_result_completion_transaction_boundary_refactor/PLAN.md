# Combat Result Completion Transaction Boundary Refactor

## Objective

Refactor the `CompleteCombatResult` player-victory completion flow so the transaction boundary is obvious from the code structure.

This is not a bug-fix goal. `docs/goals/combat_result_atomicity_regression_repair` re-audited the current runtime and found no concrete duplicate reward regression. The problem is readability and future safety: the current code can look like it commits combat rewards before node completion can still fail.

The goal is to preserve current behavior while making the intended contract explicit:

```text
planning / preflight may fail
commit / application should use prevalidated data
```

## Source Of Truth Order

1. Current runtime code.
2. Existing focused atomicity tests.
3. `docs/goals/combat_result_atomicity_regression_repair/PLAN.md`.
4. Existing completed `docs/goals/combat_result_completion_atomicity`.
5. Unity-facing command/result contract.
6. Live RON/data.

Do not treat the older audit wording as proof of a runtime bug. Re-read code and tests before editing.

## Non-Goals

- Do not change reward balance, XP amounts, research gain, consumable timing, node completion policy, or Unity-facing DTO shape.
- Do not add idempotency markers, retry flags, compatibility fallbacks, or dual paths.
- Do not widen the scope to all node completion flows unless required to make the combat result boundary readable.
- Do not reclassify this as an atomicity bug unless a new failing runtime path is demonstrated.

## Desired Design Direction

Prefer a structure close to:

```text
plan_combat_result_completion(...)
  -> may fail
  -> validates combat result content, node session, map progression, reward mode, research, and completion outcome
  -> builds all staged post-battle/reward/research/node-completion data

commit_planned_combat_result_completion(...)
  -> should not discover new local invalidity
  -> applies the prevalidated staged state and transition
```

An acceptable smaller step is to make existing helper names and contracts clearer:

```text
plan_complete_current_node(...)
commit_preplanned_node_completion(...)
```

The important rule is that a `commit_*` function called after reward state is committed must not look like it might newly reject local node completion invariants.

## Plan

1. Re-read the current implementation before editing:
   - `src/game/world/combat.rs`
   - `src/game/world/node_flow.rs`
   - `src/game/world/reward.rs`
   - current `CompleteCombatResult` tests in `src/game/world/tests/combat.rs`
2. Identify the smallest long-term refactor that improves the transaction boundary without changing behavior.
3. Record the chosen refactor shape in `EXPERIMENT_NOTES.md` before editing code.
4. Refactor names/types/helpers so the flow communicates:
   - fallible planning/preflight happens before mutation;
   - commit/application consumes prevalidated data;
   - reward/post-battle state is not committed before unplanned local invalidity can be discovered.
5. Keep or strengthen existing atomicity tests:
   - failed local completion does not mutate XP, inventory, skill fragments, enkephalin, research, consumables, map progression, node session, or game state;
   - retry after failed completion cannot duplicate rewards;
   - successful player victory still returns the same `BehaviorResult` shape.
6. Run focused tests first, then broader validation.
7. Update this goal's `EXPERIMENTS.md` after each test/failure/fix.

## Completion Conditions

- `CompleteCombatResult` player-victory completion reads as a planned transaction: failure-prone validation is visibly separated from commit/application.
- Existing behavior is preserved.
- Existing successful combat result flows still return the correct `BehaviorResult`.
- Failed completion still cannot be retried for duplicate rewards.
- No idempotency marker, compatibility fallback, DTO migration, or reward timing change is introduced.
- The final implementation and tests make the previous false-positive audit less likely.
- `PLAN.md`, `EXPERIMENTS.md`, and `EXPERIMENT_NOTES.md` are updated with the chosen design and verification results.

## Implementation Result (2026-07-04)

Status: complete.

Implemented the planned design clarity refactor without changing player-visible behavior:

- Added a `PlannedPlayerVictoryCombatResultCompletion` planning object.
- Extracted the player-victory path into:
  - `plan_player_victory_combat_result_completion(...)`
  - `commit_planned_player_victory_combat_result_completion(...)`
- Renamed node completion application from `commit_staged_node_completion(...)` to `commit_preplanned_node_completion(...)`.
- Updated existing callers to use the clearer preplanned commit name.

The resulting player-victory flow now reads as:

```text
handle_complete_combat_result()
-> plan_player_victory_combat_result_completion(...)
-> commit_planned_player_victory_combat_result_completion(...)
-> commit_preplanned_node_completion(...)
```

This keeps the existing atomicity behavior but makes the broad transaction boundary explicit. No idempotency marker, retry flag, compatibility fallback, DTO change, reward timing change, or balance change was added.

## Validation Commands

Start focused and broaden:

- `cargo test -p game_core combat_result_completion_failure_does_not_partially_apply_rewards_on_retry --lib -- --test-threads=1`
- `cargo test -p game_core game::world::tests::combat --lib -- --test-threads=1`
- `cargo test -p game_core game::world::tests::map_flow --lib -- --test-threads=1`
- `cargo check -p game_core`

If helper extraction affects more world flow code, add the relevant focused test module before broad validation.

## Validation Run (2026-07-04)

- `cargo test -p game_core combat_result_completion_failure_does_not_partially_apply_rewards_on_retry --lib -- --test-threads=1` passed.
- `cargo test -p game_core game::world::tests::combat --lib -- --test-threads=1` passed.
- `cargo test -p game_core game::world::tests::map_flow --lib -- --test-threads=1` passed.
- `cargo check -p game_core` passed.

## Stop Conditions

Complete this goal and report questions before continuing if:

- The refactor requires changing reward/XP/research/consumable timing.
- The refactor requires a Unity-facing DTO or command contract change.
- A real atomicity bug is discovered that changes this from design cleanup into behavior repair.
- SavePoint checkpoint persistence or battle record file persistence must become part of the authoritative combat-result transaction.
