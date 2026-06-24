# Core Battlefield Occupancy Cleanup

## Objective

Remove `Battlefield` single-tile `occupant` projection and related single-owner tile semantics.

Unit position and continuous body state must be the source of truth. If tile membership acceleration is needed, it must be multi-occupant and derived from unit position/body state.

## Required Startup Protocol

Before code search, edits, or validation, read:

- `docs/goals/core_battlefield_occupancy_cleanup/PLAN.md`
- `docs/goals/core_battlefield_occupancy_cleanup/EXPERIMENTS.md`
- `docs/goals/core_battlefield_occupancy_cleanup/EXPERIMENT_NOTES.md`
- `docs/goals/core_unimplemented_policy_implementation_master/PLAN.md`
- `docs/goals/core_component_refactor_master/POLICY_DECISIONS.md`
- `docs/refactor_preparation_plan.md`

## Source Of Truth

1. Runtime battle position/body state.
2. Live scenario/spawn/deployment data.
3. Unity-facing battle setup/update/checkpoint contracts.
4. Current policy documents.
5. Historical goal notes.

## Scope

In scope:

- `src/game/battle/battlefield/*`
- `BattlefieldTile.occupant`
- `Battlefield::occupant`
- `place_unit`, `place_unit_allow_overlap`, remove/move position behavior
- single-owner tile checks in spawn/deployment/targeting/blocking code
- tests/helpers that assume one occupant per tile
- policy-preserving replacement with explicit unit position queries or multi-occupant derived index if needed

Out of scope:

- Changing the confirmed allied deployment rule unless code proves it is inseparable from tile occupancy removal.
- Reworking movement physics backend beyond removing single-tile ownership assumptions.
- Skill targeting cleanup.
- Server/admin typed DTO cleanup.

## Plan

1. Inventory all `occupant`, `PositionOccupied`, `place_unit`, `place_unit_allow_overlap`, and tile-occupancy reads.
2. Classify each use:
   - terrain/valid/static obstacle,
   - unit position lookup,
   - spawn/deployment validation,
   - targeting/AoE,
   - movement/blocking,
   - test/debug helper.
3. Identify current behavior that must remain user-visible, especially allied deployment occupancy rejection and opponent overlap allowance.
4. Remove `BattlefieldTile.occupant` as storage.
5. Replace tile occupant reads with explicit unit position queries or multi-occupant derived query helpers.
6. Remove single-owner tile semantics from spawn/deployment/targeting/blocking code.
7. Add/adjust tests for:
   - opponent overlap,
   - allied deployment occupancy policy if still official,
   - AoE/targeting hitting all valid units on same tile where relevant,
   - removal/movement consistency.
8. Run focused validation after each change group.
9. Run broad validation.

## Policy Decision Completion Condition

Complete this goal with `사용자와 정책 논의 필요` if code reading shows that removing single-tile occupant requires a new decision about the items below. Do not pause, block, or leave the goal active; a policy-decision report is the completed deliverable.

- allied deployment overlap rules,
- friendly/enemy stacking rules beyond already confirmed opponent overlap,
- target selection priority when multiple units share a tile,
- AoE behavior for multiple units on one tile,
- Unity display semantics for stacked units,
- performance-motivated indexing that could become a second source of truth.

## Validation Plan

Focused:

- battlefield placement/removal tests.
- opponent overlap spawn tests.
- live deployment tests.
- movement/blocking tests touched by the change.
- skill/AoE tests if occupant was used in targeting helpers.

Broad:

- `cargo check --lib`
- `cargo test battlefield --lib -- --test-threads=1`
- `cargo test movement --lib -- --test-threads=1`
- `cargo test live_deployment --lib -- --test-threads=1`
- `cargo test --test skill_refactor_validation -- --test-threads=1` if targeting/AoE is touched
- `cargo test --lib -- --test-threads=1`
- `cargo test --test ron_loading -- --test-threads=1`
- `git diff --check`

## Completion Conditions

- `BattlefieldTile.occupant` and single occupant projection are removed.
- No `occupant.is_some()` style single-owner tile gate remains in gameplay code.
- Unit position/body state is the source of truth.
- Any tile membership helper is multi-occupant and derived, not authoritative.
- Tests pin the intended overlap/deployment/targeting behavior.
- Validation results are recorded.
