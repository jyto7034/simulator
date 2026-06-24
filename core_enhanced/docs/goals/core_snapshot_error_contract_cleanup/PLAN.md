# Core Snapshot Error Contract Cleanup

## Objective

Implement the easy-to-medium follow-up policies:

1. Remove `PlayerStateSnapshotDto` / `RunSnapshotDto` top-level field drift risk.
2. Clean up `PositionOccupied` / `position_occupied` naming so static obstacle blocking does not imply unit tile occupancy.

## Required Startup Protocol

At the beginning of this goal and every resumed run, read:

- `docs/goals/core_snapshot_error_contract_cleanup/PLAN.md`
- `docs/goals/core_snapshot_error_contract_cleanup/EXPERIMENTS.md`
- `docs/goals/core_snapshot_error_contract_cleanup/EXPERIMENT_NOTES.md`
- `docs/goals/core_followup_policy_implementation_master/PLAN.md`
- `docs/goals/core_component_refactor_master/POLICY_DECISIONS.md`
- `docs/refactor_preparation_plan.md`

Do not start code search, edits, or validation before reading those documents.

## Scope

### In Scope

- `../game_server/src/game/player_game_actor/state.rs` server snapshot wrapper structure.
- `RunSnapshotDto` / `PlayerStateSnapshotDto` composition or flattening strategy.
- Server/admin snapshot tests that protect field preservation.
- `GameError::PositionOccupied` call sites that now mean static obstacle or blocked tile.
- Core and server error-code mapping for placement/blocking errors.

### Out Of Scope

- `Battlefield` dynamic position ownership removal.
- `RuntimeUnitLifecycle`.
- Redeploy HP and identity behavior.
- Projectile/delayed-effect lifecycle behavior.

## Implementation Plan

1. Read the current `RunSnapshotDto`, server `PlayerStateSnapshotDto`, and selected-event compression code.
2. Replace manual top-level field duplication with a structure that composes the core snapshot as the source of truth while still exposing the intended external JSON shape.
3. Add or update tests proving newly added core snapshot fields cannot be silently omitted by the server wrapper.
4. Audit `PositionOccupied` uses.
5. Rename/split internal error meanings for static obstacle or blocked tile placement.
6. Audit external error-code impact. If changing the Unity/server-facing error code requires a new contract decision, complete this goal with a policy-decision report.
7. Run focused server/core checks and update this goal's experiment files.

## Policy Decision Completion Condition

If the implementation requires choosing a new Unity-facing error-code name, snapshot JSON shape, migration path, or compatibility behavior that is not already confirmed, complete the goal with `사용자와 정책 논의 필요` and report options with code evidence.

## Completion Conditions

- `PlayerStateSnapshotDto` no longer manually duplicates every `RunSnapshotDto` top-level field in a drift-prone way.
- No `serde_json::Value` mutation is reintroduced to assemble gameplay/server-facing contract fields.
- `PositionOccupied` no longer names static-obstacle blocked placement in a misleading way, unless the external contract decision is explicitly reported.
- Focused tests or `cargo check` results are recorded.
- `POLICY_DECISIONS.md` status is updated if implementation completes.
