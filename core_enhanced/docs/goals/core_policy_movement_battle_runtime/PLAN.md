# Movement and Battle Runtime Policy Implementation

## Objective

Align action gates, movement blocking, battle runtime events, attack-resolution ordering, and timeline naming with confirmed runtime policy.

## Policies Covered

- `allowed_actions source`
- `action validation gates`
- `RequestDeploymentRangePreview gate`
- `forced movement clears engagement`
- `MovementStopped event`
- `movement tick policy source`
- `opponent spawn occupancy policy`
- `UnitWithdrawn timeline event`
- `BehaviorResult boundary`
- `battle runtime numeric policy`
- `Timeline naming`
- `same timestamp attack resolve batch`

## Plan

1. [x] Read behavior request handling, allowed-action projection, action validation, movement/block engagement, spawn handling, battle scheduler, timeline events, and result mapping.
2. [x] Confirm allowed actions are UI projection and command handlers retain authoritative validation gates.
3. [x] Keep `RequestDeploymentRangePreview` gated so Unity can disable invalid buttons before command submission.
4. [x] Confirm forced movement/block engagement is already recomputed by movement/block state rather than tile occupancy.
5. [x] Confirm `MovementStopped(reason)` is emitted for natural arrival, obstacles, and no-goal stops.
6. [x] Move movement tick cadence to data-driven policy.
7. [x] Remove nearest-open opponent spawn fallback; authored spawn position is used, and unit overlap is allowed by policy.
8. [x] Add `UnitWithdrawn` timeline event for live withdraw.
9. [x] Keep touched `BehaviorResult::BattleUnitWithdrawn` behavior stable and defer broad result-contract split to `core_policy_unity_server_contract`.
10. [x] Move numeric runtime policy to canonical run policy data/config.
11. [x] Leave broad timeline naming normalization to `core_policy_unity_server_contract` because it crosses Unity/server DTO naming.
12. [x] Drain same-timestamp attack resolve batches before final battle-end confirmation.
13. [x] Update behavior gate, movement, spawn, timeline, and scheduler tests.

## Completion Conditions

- Action availability has one authoritative state source plus command validation gates.
- Forced movement, stop reasons, and spawn occupancy follow confirmed policy.
- Timeline and result contracts no longer rely on ambiguous legacy names where touched.
- Same-timestamp attack resolves are deterministic and tested.
- Runtime numeric policy is no longer duplicated across code/data.

## Completion Report

- Implemented `RunPolicyData.battle_runtime` with `movement_tick_ms` and `max_battle_time_ms`; live `run/policy.ron` is now the runtime source for movement tick cadence and max battle time.
- Removed opponent nearest-open spawn fallback. Opponent spawns now use authored positions and allow unit overlap while still rejecting out-of-bounds/static-obstacle placement.
- Added `TimelineEvent::UnitWithdrawn` and wired live withdraw commands to record it with command time.
- Added same-timestamp attack resolve draining before battle-end confirmation and pinned it with a focused test.
- Confirmed `allowed_actions`, deployment range preview gating, command validation gates, `MovementStopped(reason)`, forced engagement recompute, and movement overlap behavior already matched policy after source inspection.
- Deferred broad `BehaviorResult` splitting and timeline naming normalization to `core_policy_unity_server_contract`, where shared DTO migration belongs.
- Validation passed: `cargo check --lib`; focused tests for withdraw, spawn overlap, runtime policy, movement tick, same-timestamp attack resolve, and overlap placement; `cargo test movement --lib`; `cargo test live_deployment --lib`; `cargo test run_policy --lib`; `cargo test attack_resolve --lib`; `cargo test --test ron_loading`; `cargo check -p game_server`; `cargo test --lib -- --test-threads=1`; `git diff --check`.
