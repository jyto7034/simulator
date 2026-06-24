# Allowed Actions Policy Plan

## Objective

Move allowed action decision-making into one source of truth that accounts for both `GameState` and active node/session context while preserving the Unity-facing `allowed_actions` snapshot shape.

## Current Scope

In scope:

- Re-read runtime code before relying on stale design notes.
- Preserve existing state/action behavior unless current code reveals a policy contradiction.
- Consolidate `ActionScheduler` base state policy and `world/helpers.rs` context adjustments.
- Lock Maintenance, Support, Reward, Shop, Battle, and Map allowed action behavior with focused tests.

Out of scope:

- ActionKind enum semantic changes.
- Unity-facing DTO shape changes.
- Game state transition policy changes.
- New player actions.

## Plan

1. Inspect current `ActionScheduler`, `GameState`, `GameCore::refresh_allowed_actions`, and snapshot uses.
2. Record current GameState/context adjustments in `EXPERIMENT_NOTES.md`.
3. Add a single allowed action policy entry point.
4. Route world transition/refresh initialization through that entry point.
5. Remove duplicate context adjustment logic.
6. Run focused world tests and `cargo check -p game_core`.

## Completion Conditions

- Allowed actions are computed by one policy entry point.
- Context-dependent actions are explicit and tested.
- Unity-facing `allowed_actions` field shape is unchanged.
- No compatibility layer or dual policy path is introduced.
- Verification commands pass.
- 사용자와 의논하여 정해야 할 정책이 있을 경우 goal을 종료한다.

## Completion Evidence

- `ActionScheduler::get_allowed_actions_for_context` is the single policy entry point for base state actions plus reward/maintenance context adjustments.
- `GameCore::allowed_actions_for_state_context` now only builds `AllowedActionContext` and delegates to `ActionScheduler`.
- Unity-facing snapshot code still reads `self.get_allowed_actions()` and the `allowed_actions` field shape is unchanged.
- Verification passed:
  - `cargo test -p game_core managers::action_scheduler::tests -- --nocapture`
  - `cargo test -p game_core world::tests::snapshots_and_start -- --nocapture`
  - `cargo test -p game_core world::tests::support -- --nocapture`
  - `cargo test -p game_core world::tests::equipment -- --nocapture`
  - `cargo fmt`
  - `cargo check -p game_core`

## Stop Conditions

- A state/action allowance requires a new UX or game-policy decision.
- Unity-facing action contract must change.
- A new ActionKind is required.
- 사용자와 의논하여 정해야 할 정책이 있을 경우 goal을 종료한다.
