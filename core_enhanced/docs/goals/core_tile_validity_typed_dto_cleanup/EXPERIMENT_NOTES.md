# Experiment Notes

## Confirmed Policy

- Audit `Battlefield::in_bounds` during the next tile/range implementation pass.
- Do not add wrapper/rename automatically.
- Rename or wrap only if call sites confuse raw bounds with valid-tile policy.
- Server/admin Unity-facing contract shape should move to typed DTOs.
- `serde_json::Value` may remain only at final serialization/logging/generic passthrough boundaries.

## Policy Questions To Watch

Record `사용자와 정책 논의 필요` and complete the goal if work discovers any item below. Do not pause, block, or leave the goal active; a policy-decision report is the completed deliverable.

- a Unity-facing field rename not already confirmed,
- an admin API shape change with UX/tooling implications,
- save/migration requirements,
- a meaning split between raw bounds and valid tile that changes gameplay behavior.

## Follow-Up Candidates

Record out-of-scope improvements here.

## Audit Notes

- `Battlefield::in_bounds` currently checks both rectangle bounds and authored `valid_tiles`.
- Runtime callers found during this pass all want valid gameplay tiles: range preview final cells, runtime `SkillAreaDeclared.affected_tiles`, explicit tile cast target validation, `WholeFieldValidTiles` target eligibility, `idx`, and `is_walkable_tile`.
- Raw rectangle-only validation is already represented separately by `position_in_bounds` in event-log validation.
- Because the two meanings are not mixed at current call sites, adding an `is_valid_battle_tile` wrapper or renaming `in_bounds` would add churn without reducing source-of-truth risk.

## Implementation Notes

- `PlayerGameActor::build_state_snapshot` now starts from `GameCore::get_run_snapshot_dto()` instead of `get_run_snapshot_json()`.
- Server-only transport enrichment is represented by `PlayerStateSnapshotDto` and `PlayerSelectedEventSnapshotDto`.
- `compressed_event_log` remains a server transport field on combat result selected events, but it is no longer inserted by mutating a `serde_json::Value` object with string keys.
- `serde_json::Value` remains only as the final `StateSnapshot` transport payload and admin/generic command payload boundary in the touched path.
- `AdminDumpState` already serializes `RunSnapshotDto` through `get_run_snapshot_json()` and does not perform post-serialization shape mutation, so no admin code change was needed in this subgoal.
