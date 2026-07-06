# Node Completion Commit Boundary Split

## Objective

Split the broad node-completion commit boundary so each gameplay flow has a readable, prevalidated commit path.

This is a follow-up to `docs/goals/combat_result_completion_transaction_boundary_refactor`. The current player-victory combat result flow is behaviorally safe, but it still ends by calling the shared `commit_preplanned_node_completion(...)` helper. That helper also owns support effects, save checkpoint persistence, floor advancement, boss omen source consumption, event-session cleanup, and map/game-state transition.

The goal is design clarity and future safety, not a behavior change:

```text
combat-result commit should look like combat-result commit
support/save/gate/event completion should not be hidden inside the same broad helper
```

## Source Of Truth Order

1. Current runtime code.
2. Existing combat result atomicity tests.
3. Existing map/node-flow tests.
4. Canonical game rulebook and node-map policy docs.
5. Unity-facing snapshot/command contracts.
6. Live RON/data.

Do not assume the shared helper is currently buggy. Re-read code and tests before editing.

## Non-Goals

- Do not change reward, XP, research, consumable, retry, gate, support, savepoint, or floor-advance policy.
- Do not change Unity-facing DTO shape.
- Do not introduce idempotency markers, retry flags, fallback paths, or dual schemas.
- Do not alter event-node choice semantics or boss-omen semantics unless the current flow cannot be split safely.
- Do not widen this into a full map progression rewrite.

## Current Suspect Area

- `src/game/world/combat.rs`
  - player-victory result planning and commit;
  - `commit_planned_player_victory_combat_result_completion(...)`.
- `src/game/world/node_flow.rs`
  - `plan_complete_current_node(...)`;
  - `commit_preplanned_node_completion(...)`;
  - `apply_staged_support_effect(...)`;
  - floor advance/run complete branches.
- `src/game/world/event_node.rs`
  - event-node completion callers.

## Desired Design Direction

Prefer separating flow-specific application from common primitives:

```text
plan_complete_current_node(...)
  -> creates validated completion data

commit_combat_result_node_completion(...)
  -> applies only the node/map/session transition needed after combat result victory

commit_support_node_completion(...)
  -> applies support effects and checkpoint persistence explicitly

commit_gate_or_floor_node_completion(...)
  -> applies floor advancement explicitly

shared lower-level helpers
  -> mutate map/progression/session fields only after prevalidation
```

An acceptable implementation may keep one internal primitive, but public/super-visible call sites must not make combat result completion look like it can newly discover unrelated support/savepoint failures after rewards are already staged.

## Plan

1. Re-read the current code paths before editing:
   - combat result completion;
   - normal node completion;
   - event node completion;
   - support/save/gate completion;
   - floor advance/run complete.
2. Draw the current call graph in `EXPERIMENT_NOTES.md`.
3. Identify which `StagedNodeCompletion` variants are used by which gameplay flows.
4. Choose the smallest long-term split that removes misleading shared responsibility from combat-result commit.
5. Record the chosen split before editing.
6. Refactor toward flow-specific commit helpers while preserving existing staged data contracts.
7. Strengthen tests around user-visible behavior:
   - player-victory combat result still grants rewards once;
   - failed completion still cannot partially apply rewards;
   - event-started combat completion still consumes/completes the event node according to policy;
   - support/savepoint/gate/floor advance behavior is unchanged.
8. Run focused tests after each small change, then broader checks.
9. Update `EXPERIMENTS.md` with each failed attempt, correction, and validation result.

## Completion Conditions

- Combat result completion no longer depends on a broad helper whose name/contract implies unrelated support/save/floor responsibilities.
- Support/save/gate/floor completion paths remain explicit and behaviorally unchanged.
- Existing player-visible node completion behavior is preserved.
- No DTO, live RON, reward timing, retry policy, or save policy changes are introduced.
- Tests cover combat result atomicity and at least one non-combat node completion path touched by the refactor.
- Goal docs are kept current while implementing.

## Implementation Result (2026-07-06)

Status: complete.

Implemented the responsibility split without changing node completion policy or Unity-facing DTOs:

- Replaced the broad `commit_preplanned_node_completion(...)` entrypoint with flow-specific commit entrypoints:
  - `commit_interactive_node_completion(...)` for normal node/event/support completion flows.
  - `commit_combat_result_node_completion(...)` for player-victory combat result completion.
- Added `CombatResultNodeCompletion`, a support-free completion type converted from `StagedNodeCompletion` immediately after combat-result node-completion planning.
- Extracted variant-specific lower-level helpers for:
  - completed node transitions;
  - floor advancement;
  - run completion;
  - savepoint checkpoint persistence after the original transition order.
- Updated Event scene/choice completion to use the interactive completion entrypoint.
- Updated player-victory combat result completion to use the combat-result completion entrypoint.
- Removed debug-only support-effect assertions from the combat-result commit path. A support-bearing staged completion now fails during conversion before reward commit can proceed.

No reward, XP, research, consumable, retry, gate, support, savepoint, floor-advance, live RON, or DTO policy was changed.

## Validation Commands

Start focused:

- `cargo test -p game_core combat_result_completion_failure_does_not_partially_apply_rewards_on_retry --lib -- --test-threads=1`
- `cargo test -p game_core game::world::tests::combat --lib -- --test-threads=1`
- `cargo test -p game_core game::world::tests::map_flow --lib -- --test-threads=1`

If event-node completion code changes:

- `cargo test -p game_core game::world::tests::event --lib -- --test-threads=1`

Always finish with:

- `cargo check -p game_core`

## Validation Run (2026-07-06)

- `cargo check -p game_core` passed before focused tests.
- `cargo test -p game_core combat_result_completion_failure_does_not_partially_apply_rewards_on_retry --lib -- --test-threads=1` passed.
- `cargo test -p game_core game::world::tests::combat --lib -- --test-threads=1` passed.
- `cargo test -p game_core game::world::tests::map_flow --lib -- --test-threads=1` passed.
- `cargo test -p game_core game::world::tests::support --lib -- --test-threads=1` passed to cover non-combat support/savepoint completion touched by the refactor.
- `cargo check -p game_core` passed after formatting and tests.

Observed unrelated existing warnings:

- `auth_server/Cargo.toml` has an unused manifest key warning.
- `src/game/world/tests/mod.rs` has unused import warnings in test builds.

## Stop Conditions

Complete the goal and report questions before continuing if:

- The split requires changing any user-visible node completion, reward, retry, savepoint, or gate policy.
- A Unity-facing DTO or command contract would need to change.
- Combat result completion cannot be separated without changing event-node or boss-omen semantics.
- Save checkpoint persistence must become part of the authoritative combat-result transaction.
