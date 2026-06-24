# DefenseRoute Sticky Block Engagement Plan

## Objective

Implement Arknights-style blocking for live `DefenseRoute` combat.

Blocked enemies must not leak through a deployed ground blocker because block assignment is recalculated every movement tick. A block is a persistent combat engagement, not a temporary per-tick selection result.

## Source Of Truth Order

Use this order while implementing:

1. Runtime battle code and exported battle records.
2. Live RON/data that creates employee, enemy, battlefield, route, and wave profiles.
3. Unity-facing battle setup/update DTOs and battle record JSON.
4. Current policy documents.

Do not trust older documentation over runtime behavior. If runtime code reveals a better long-term design, record the evidence in `EXPERIMENT_NOTES.md` before changing the plan.

## Current Evidence

Two battle record files were compared:

- `battle_records/run_af4bbe2e18145867/9fa8e909-e735-41c2-819a-d846194fb9a3.json`
- `battle_records/run_7e0092b971f809d1/26c58ffc-fe9d-44e0-82d1-2586c3d91ee7.json`

The first record is abnormal:

- Deployed player unit `c24aedd9-8251-4b48-a713-1a75b471cd79`
- `mobility_kind: ground`
- position `(3.5, 6.5)`
- Two ground/blockable enemies enter the blocker radius while the player unit is alive.
- Both enemies keep receiving `MovementSegmentStarted` events past the blocker:
  - `a1fbb727...` reaches distance `0.150` from the blocker and then moves to the right.
  - `3a9fe0ed...` reaches distance `0.244` from the blocker and then moves to the right.
- Enemy `3a9fe0ed...` attacks the player at `7200ms`, then attacks the defense object at `8700ms` while the player is still alive.

The second record is normal:

- Deployed player unit `82ea518f-79ae-4903-9e58-b16a03e7419b`
- `mobility_kind: ground`
- position `(2.5, 4.5)`
- One ground/blockable enemy approaches, stops near `(1.780, 4.468)`, and no later movement segment passes the player.

The difference is not ground/platform type. Both deployed player units are ground. The likely issue is the current block model.

## Current Code Findings

Relevant code:

- `src/game/battle/core/movement/blocking.rs`
- `src/game/battle/core/movement/planner.rs`
- `src/game/battle/core/movement/engine.rs`
- `src/game/battle/core/types.rs`
- `src/game/battle/types.rs`

Current behavior:

- `BlockRuntimeState` has:
  - `blocker_to_enemies`
  - `enemy_to_blocker`
- `refresh_block_state()` rebuilds the whole block state every movement tick.
- Blocker candidates are player combatants that are alive, have `block_capacity > 0`, and `block_radius_units > 0.0`.
- Enemy candidates are opponent combatants that are alive, `blockable`, and not airborne.
- Enemy priority is recomputed from route progress, spawn order, and unit id.
- `build_continuous_movement_input_with_goals()` sets `can_move = false` only when `blocked_by(unit_id)` exists in the current tick's block state.

This makes block state a per-tick cache. With one blocker and multiple enemies inside/near the block radius, route-progress priority can alternate and allow enemies to advance across ticks.

## Target Policy

Use Arknights-style blocking:

- A block engagement is persistent.
- Once a blocker catches an enemy, that enemy remains blocked until an explicit release condition occurs.
- Existing valid engagements are preserved before filling free capacity.
- Free blocker capacity may catch new enemies.
- Capacity overflow enemies may pass.
- Airborne enemies are not blocked.
- Platform-only player units cannot block even if authored data accidentally gives block capacity.
- A blocked enemy cannot continue route movement.
- A blocker prioritizes enemies it is blocking for melee/basic targeting.

### Release Conditions

An engagement is released when:

- The blocker dies.
- The blocked enemy dies.
- The blocker is withdrawn.
- The blocker is no longer a valid player combatant.
- The enemy is no longer a valid blockable ground opponent combatant.
- A forced displacement, teleport, knockback, special mechanic, or explicit block-break effect says to release it.
- The battle ends or runtime state resets.

Do not add distance-based release for normal route movement. Arknights-style block is a held engagement, not a proximity-only relationship after capture. If a future forced displacement or special mechanic moves a blocked enemy away, that effect must explicitly release the engagement; ordinary distance drift must not silently break block.

## Intended Design

Refactor `BlockRuntimeState` from a per-tick rebuilt cache into persistent engagement state.

Recommended shape:

```rust
pub(in crate::game::battle::core) struct BlockRuntimeState {
    blocker_to_enemies: HashMap<UnitInstanceId, Vec<UnitInstanceId>>,
    enemy_to_blocker: HashMap<UnitInstanceId, UnitInstanceId>,
}
```

The existing shape can remain, but its update semantics must change:

1. Remove invalid existing engagements.
2. Compute remaining capacity from still-valid engagements.
3. Find unblocked enemy candidates inside a valid block radius.
4. Assign only free capacity.
5. Preserve existing blocker/enemy pairings unless release conditions apply.
6. Sort each blocker's engaged enemies for target preference without changing the pairings.

Rename `refresh_block_state()` if useful, for example to `update_block_engagements()`, once the behavior is no longer a full refresh.

## Implementation Plan

1. Reproduce and pin the bug with focused tests.
   - Build a fixed-defense core with one ground blocker, `block_capacity = 1`, and two ground/blockable route enemies.
   - Advance multiple movement ticks.
   - Assert exactly one enemy remains blocked and does not receive route movement.
   - Assert the overflow enemy may continue route movement.
   - Assert the engaged enemy does not swap every tick due to route progress changes.

2. Refactor block update semantics.
   - Preserve valid engagements first.
   - Prune invalid engagements.
   - Fill remaining capacity from unblocked in-radius enemies.
   - Keep deterministic ordering for newly acquired enemies.
   - Keep existing `blocked_by()` and `first_blocked_enemy()` callers working unless a better name is needed.

3. Keep movement integration authoritative.
   - `build_continuous_movement_input_with_goals()` must keep blocked enemies `can_move = false`.
   - Blocked enemies must not receive `MoveToPoint` route goals.
   - If a blocked enemy had an active movement segment, the implementation should stop it through existing movement stop semantics rather than silently leaving an active segment.

4. Keep targeting behavior consistent.
   - Player melee/basic targeting should prioritize blocked enemies.
   - Blocked route enemies should target their blocker when in range.
   - If a blocked enemy is not in attack range because of authored range/body tuning, it must still remain movement-locked. Record any such mismatch in `EXPERIMENT_NOTES.md`.

5. Add live behavior tests.
   - Use a live `DefenseRoute` scenario or a focused battle core fixture to prove two enemies cannot both leak through one blocker over repeated ticks.
   - Add a battle-record-oriented assertion if feasible: while a blocker is alive and an enemy is engaged, no `MovementSegmentStarted` for that enemy may advance beyond the blocker.

6. Update documentation if code changes policy wording.
   - `docs/game_rulebook.md`: add concise blocking semantics.
   - `docs/skill_target_contract.md`: update blocked-target priority wording only if needed.
   - `docs/README.md`: update only if a new canonical blocking document is introduced.

7. Verify.
   - Run focused movement/blocking tests after each meaningful code change.
   - Run live DefenseRoute tests covering movement and battle record export.
   - Finish with `cargo check -p game_core` and the appropriate wider `game_core` tests.

## Test Plan

Focused tests:

- Sticky engagement:
  - One blocker, capacity 1.
  - Two blockable ground enemies enter radius.
  - The first acquired enemy remains blocked across multiple ticks.
  - The overflow enemy can continue.

- Release on enemy death:
  - A blocked enemy dies.
  - Capacity frees.
  - A waiting in-radius enemy becomes blocked.

- Release on blocker death/withdraw:
  - Blocker dies or is removed.
  - Previously blocked enemy can move again.

- Airborne policy:
  - Airborne enemy inside radius is not blocked.

- Platform policy:
  - Platform-only player unit cannot block even if profile data has capacity.

- Current target policy:
  - Blocker targets a blocked enemy before non-blocked candidates.
  - Blocked enemy targets its blocker when in range.

Live/behavior tests:

- DefenseRoute battle with two ground enemies and one deployed ground blocker.
- Assert no blocked enemy receives route movement through the blocker while the blocker is alive.
- Assert battle timeline remains valid and monotonically sequenced.

Regression records:

- The abnormal `9fa8e909-e735-41c2-819a-d846194fb9a3` behavior should be reproducible by test or equivalent fixture.
- The normal `26c58ffc-fe9d-44e0-82d1-2586c3d91ee7` behavior should remain normal.

## Non-Goals

- Do not tune enemy speed, body radius, block radius, route shape, or wave timing to hide the bug.
- Do not add Unity-side collision hacks.
- Do not introduce compatibility behavior that keeps old per-tick swapping semantics.
- Do not change platform/ground deployment policy unless tests prove the current policy is wrong.
- Do not redesign all movement collision; keep this goal scoped to block engagement semantics.
- Do not add new block-breaking effects unless existing runtime code already has an explicit source that needs to use them.

## Stop Conditions

Stop and report policy questions if any of these become necessary:

- Multiple blockers can catch the same enemy and the desired tie-breaker cannot be inferred from Arknights-style rules.
- Blocker capacity should depend on weapon/armor/equipment but current data lacks a reliable source.
- Distance-based block release is required to avoid broken runtime behavior.
- A Unity-facing DTO change is needed to expose block engagement state.
- Existing live RON data requires platform units to block.
- Implementing sticky engagement conflicts with a documented boss/elite mechanic.

## Completion Criteria

- Block engagement persists across movement ticks.
- A blocked ground enemy cannot route-move through its blocker while the engagement is valid.
- Capacity overflow enemies may still pass.
- Existing block release conditions are deterministic and tested.
- Existing ground/platform/airborne policies remain intact.
- Unity-facing movement timeline no longer emits through-blocker movement for engaged enemies.
- Relevant docs are updated if policy wording changes.
- `EXPERIMENTS.md` records failed attempts, fixes, and verification commands.
- `EXPERIMENT_NOTES.md` records policy judgments and follow-up ideas.
