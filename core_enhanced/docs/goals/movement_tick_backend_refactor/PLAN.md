# Movement Tick Backend Refactor Plan

## Objective

Reduce duplicated Direct/Rapier continuous movement tick logic while keeping movement source-of-truth boundaries explicit.

## Current Scope

In scope:

- Re-read runtime movement code before relying on stale design notes.
- Decide whether Direct stays or is removed based on current code/tests.
- Preserve current gameplay movement behavior.
- Keep Rapier as static terrain/obstacle correction helper, not a gameplay source of truth for blocking, targeting, overlap, route progress, or hit decisions.
- Extract shared tick control flow where possible.

Out of scope:

- New movement backend.
- New collision/blocking/overlap policy.
- Route/pathfinding changes.
- Airborne policy changes.
- Targeting policy changes.

## Direction Decision

Keep Direct.

Rationale:

- Direct is test-only and provides fast deterministic coverage for movement semantics.
- Current policy documents say Rapier is a terrain/static obstacle correction helper, not the source of truth for gameplay semantics.
- Removing Direct would force tests to depend on Rapier for behavior that should remain authored gameplay logic.
- The long-term structure should keep one shared tick loop and narrow each backend to position correction/application details.

## Plan

1. Record existing Direct/Rapier duplicated tick responsibilities.
2. Extract a shared continuous movement tick loop in `movement/engine.rs`.
3. Give Direct and Rapier backend-specific hooks for preparation, terrain correction, and position application.
4. Keep Direct-specific static obstacle solver as the deterministic correction implementation.
5. Keep Rapier-specific world sync/correction/application in `rapier_backend.rs`.
6. Run movement and battle-core focused tests.

## Completion Conditions

- Direct/Rapier duplicated tick loop control flow is reduced.
- Direct remains as deterministic test harness.
- Rapier remains terrain/static obstacle correction helper.
- Movement behavior tests pass.
- No new gameplay movement policy is introduced.
- 사용자와 의논하여 정해야 할 정책이 있을 경우 goal을 종료한다.

## Completion Evidence

- `src/game/battle/core/movement/engine.rs` owns `run_continuous_movement_tick`, the shared Direct/Rapier tick loop.
- `DirectContinuousMovement` implements deterministic static obstacle correction through the shared loop.
- `RapierMovementWorld` implements preparation/sync, Rapier ground static obstacle correction, and position application through the shared loop.
- Direct was kept by explicit decision as deterministic test harness.
- Verification passed:
  - `cargo check -p game_core`
  - `cargo test -p game_core battle::core::movement -- --nocapture`
  - `cargo test -p game_core battle::core::tests -- --nocapture`
  - `cargo fmt`
  - `cargo check -p game_core`

## Stop Conditions

- A movement/collision/blocking/overlap policy decision becomes necessary.
- Rapier behavior changes gameplay-visible movement semantics.
- Focused tests reveal nondeterminism or performance concerns requiring user judgment.
- 사용자와 의논하여 정해야 할 정책이 있을 경우 goal을 종료한다.
