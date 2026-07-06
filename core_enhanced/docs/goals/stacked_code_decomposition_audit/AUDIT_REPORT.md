# Node Completion Commit Stacked Code Audit

## Summary

- Verdict: mostly decomposed; no immediate correction required in the current node-completion commit split.
- Main source-of-truth owner: `src/game/world/node_flow.rs`.
- Main hidden context: before the split, readers had to know that combat-result completion called `plan_complete_current_node(false)`, so its later node-completion commit should not apply support/savepoint effects after rewards were staged.
- Highest-risk finding: remaining mixed-level shared helpers still combine node state mutation, boss-omen update, event-session cleanup, support effect application, transition, and checkpoint persistence.
- Recommended next goal: no immediate implementation goal. If this area is touched again, create a focused node-flow primitive cleanup goal that separates support/checkpoint side effects from map/session mutation helpers.

## Findings

| ID | Category | Severity | Evidence | Recommendation |
| --- | --- | --- | --- | --- |
| F-001 | Responsibility pile-up | Follow-up refactor | `src/game/world/node_flow.rs`: `commit_node_completed_with_support(...)`, `commit_floor_advanced_with_support(...)`, and `commit_run_complete_with_support(...)` still mix map/run mutation, support effect application, state cleanup, transition, and checkpoint persistence. | Defer unless the support/save/floor flow is edited again. A follow-up should split by domain step only if it reduces caller knowledge without changing node policy. |
| F-002 | Documentation drift | Document only | `docs/goals/combat_result_completion_transaction_boundary_refactor/*`, `docs/goals/combat_result_atomicity_regression_repair/*`, and historical sections of `docs/goals/node_completion_commit_boundary_split/*` still mention `commit_preplanned_node_completion(...)` or `commit_staged_node_completion(...)`; current runtime has replaced them with `commit_interactive_node_completion(...)`, `CombatResultNodeCompletion`, and `commit_combat_result_node_completion(...)`. | Treat older goal docs as historical evidence only. When closing completed goals, consolidate current policy into canonical docs or remove completed goal dirs per `docs/refactor_preparation_plan.md`. |

## Flow Trace

- Entry point:
  - Interactive node completion: `handle_complete_node()` plans with `plan_complete_current_node(true)` and commits through `commit_interactive_node_completion(...)`.
  - Event scene/choice completion: `event_node.rs` also plans with `plan_complete_current_node(true)` and commits through `commit_interactive_node_completion(...)`.
  - Player-victory combat result completion: `plan_player_victory_combat_result_completion(...)` plans with `plan_complete_current_node(false)`, converts to `CombatResultNodeCompletion`, then later commits through `commit_combat_result_node_completion(...)`.
- Planning/preflight boundary:
  - `plan_complete_current_node(...)` clones map/progression, plans support only when requested, completes the node on staged data, and builds `StagedNodeCompletion`.
  - `CombatResultNodeCompletion::try_from(StagedNodeCompletion)` rejects support-bearing staged completions before reward state is committed.
- Mutation owner:
  - `node_flow.rs` owns map/progression/run mutation for node completion variants.
  - `combat.rs` owns staged combat result state, reward/research application, and wrapping the node completion result into `CombatRewardsGranted`.
- Commit/application boundary:
  - `commit_interactive_node_completion(...)` explicitly keeps support/savepoint behavior.
  - `commit_combat_result_node_completion(...)` accepts only `CombatResultNodeCompletion`, which has no `support_effect` field.
- Validation boundary:
  - Local node completion validity is checked during `plan_complete_current_node(...)`.
  - Combat-result support exclusion is checked by `CombatResultNodeCompletion::try_from(...)`, not by debug-only assertions.
- Derived projections:
  - `MapViewDto` values are built from staged or committed map/progression state.
  - `BehaviorResult` is a response projection, not the source of node progression truth.
- External contracts:
  - No Unity-facing DTO shape change was observed in this split.
  - Save checkpoint payload remains an internal runtime checkpoint structure in `RunCheckpointPayload`.
- Tests:
  - Combat result atomicity test exists: `combat_result_completion_failure_does_not_partially_apply_rewards_on_retry`.
  - Broader combat/map/support test modules cover user-visible completion flows.

## Readability Review

- Current reader burden:
  - Reduced: combat-result completion no longer requires remembering that a broad shared commit helper is safe only because `apply_support_effect` was false.
  - Remaining: readers still need to understand that lower-level `*_with_support` helpers intentionally perform several post-completion side effects in a fixed order.
- Misleading names or abstraction levels:
  - Resolved: `commit_preplanned_node_completion(...)` is gone from runtime code.
  - Remaining: `commit_*_with_support` names are accurate but broad; they include support effect application, active-node cleanup, transition, and optional checkpoint persistence.
- Facts that should become local/structural:
  - Already structural: combat-result node completion cannot carry support effects because `CombatResultNodeCompletion` omits the field.
  - Possible future structural improvement: represent support/checkpoint application as a separate explicit post-completion step if support flow changes grow.
- Expected readability improvement:
  - The current runtime now reads as:

```text
combat result victory
  -> plan node completion without support
  -> convert to support-free CombatResultNodeCompletion
  -> stage rewards/research
  -> commit rewards
  -> commit support-free node completion

interactive/event completion
  -> plan node completion with support
  -> commit interactive node completion
```

## Follow-Up Goal Candidates

### Node Flow Completion Primitive Cleanup

Objective:

Split the remaining lower-level node completion helpers only if future work needs to modify support, checkpoint, boss-omen, or transition ordering.

Scope:

- `commit_node_completed_with_support(...)`
- `commit_floor_advanced_with_support(...)`
- `commit_run_complete_with_support(...)`
- `save_completion_checkpoint_if_needed(...)`
- support/savepoint tests affected by the changed ordering

Completion conditions:

- Map/progression/run mutation helpers do not also hide support/checkpoint policy unless the policy is intentionally documented at the entrypoint.
- Support/savepoint effects remain user-visible equivalent.
- Combat-result completion continues to accept only support-free completion data.
- Focused support, map-flow, and combat-result atomicity tests pass.

Readable-code improvement:

- A reader can see whether a completion step mutates map state, applies support effects, changes game state, or persists a checkpoint without opening one broad helper.

Focused validation:

- `cargo test -p game_core game::world::tests::support --lib -- --test-threads=1`
- `cargo test -p game_core game::world::tests::map_flow --lib -- --test-threads=1`
- `cargo test -p game_core combat_result_completion_failure_does_not_partially_apply_rewards_on_retry --lib -- --test-threads=1`
- `cargo check -p game_core`

Policy questions:

- None now. Ask before changing savepoint timing, support effect timing, floor advance policy, or Unity-facing result shape.

### Completed Goal Documentation Consolidation

Objective:

Remove or consolidate completed/historical node-completion goal docs so old helper names no longer look like current guidance.

Scope:

- `docs/goals/combat_result_atomicity_regression_repair/`
- `docs/goals/combat_result_completion_transaction_boundary_refactor/`
- `docs/goals/node_completion_commit_boundary_split/`
- canonical docs that should retain current policy, if any

Completion conditions:

- Current policy is represented in canonical docs or the latest active goal notes.
- Historical references to `commit_staged_node_completion(...)` and `commit_preplanned_node_completion(...)` cannot be mistaken for current runtime guidance.
- No runtime code changes are required.

Readable-code improvement:

- Maintainers following docs no longer have to reconcile obsolete helper names with current runtime code.

Focused validation:

- `rg -n "commit_preplanned_node_completion|commit_staged_node_completion" docs src`
- Documentation review only unless canonical docs are edited.

Policy questions:

- Confirm whether completed goal directories should be deleted now or kept temporarily as audit history.

## No-Issue Notes

- `CombatResultNodeCompletion` is not a duplicated source of truth. It is a support-free projection of `StagedNodeCompletion` for the combat-result commit boundary, and the conversion rejects unsupported support effects before reward commit.
- The old broad runtime entrypoint `commit_preplanned_node_completion(...)` is no longer present in `src/`; current matches are historical docs and the completed split goal.
- `MapViewDto` in these flows is a derived response view. It should not be treated as a second canonical map/progression source.
- `RunCheckpointPayload` is internal checkpoint state. It is relevant to support/savepoint behavior but was not observed as a Unity-facing DTO contract in this audit.

## Validation Results

Validation run on 2026-07-06:

- `cargo test -p game_core combat_result_completion_failure_does_not_partially_apply_rewards_on_retry --lib -- --test-threads=1` passed.
- `cargo test -p game_core game::world::tests::support --lib -- --test-threads=1` passed.
- `cargo test -p game_core game::world::tests::map_flow --lib -- --test-threads=1` passed.
- `cargo check -p game_core` passed.

Observed warnings:

- `auth_server/Cargo.toml` has an unused manifest key warning for `env`.
- test builds report unused imports in `src/game/world/tests/mod.rs`.
