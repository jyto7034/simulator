# Experiment Notes

- This is the only intentionally small/easy subgoal in the follow-up batch.
- Do not solve snapshot drift by returning to untyped `serde_json::Value` shape mutation.
- Internal error enum cleanup can proceed under confirmed policy, but external error-code rename may become a Unity-facing contract decision.
- If external rename is required, finish with a policy-decision report rather than inventing a compatibility alias.
- Implementation chose a typed selected-event extension point: `RunSnapshotDto<SelectedEvent = SelectedEventSnapshotDto>` plus `map_selected_event`. This keeps the core snapshot root fields owned by core and lets server replace only `selected_event` with a transport-enriched typed payload.
- `PlayerStateSnapshotDto` now flattens `RunSnapshotDto<PlayerSelectedEventSnapshotDto>` instead of manually listing root fields. The focused server test compares core snapshot root keys to server snapshot root keys, so future root field drift is behaviorally caught.
- Static obstacle placement/blocking now uses `GameError::StaticObstacleBlocked` / `static_obstacle_blocked`. `GameError::PositionOccupied` remains for actual occupied board slots, where the name still matches the domain.
