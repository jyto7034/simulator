# Pending Deployment Range Preview Contract Notes

## Working Decisions

- Pending deployment preview is separate from live checkpoint unit preview because the unit is not deployed yet.
- Unity should request pending preview once after tile release, then use the returned four facing previews for direction hover.
- The request is informational and must not mutate battle state.
- `deploy_unit(employee_uuid, position, facing)` remains the authoritative deployment command.
- Range preview cells are not walkability cells. Obstacle/blocked tiles stay visible in range preview; void/out-of-bounds/invalid tiles are excluded.

## Source-Of-Truth Checks To Perform

- Confirm whether current `range_preview` helper uses `is_walkable_tile` or any obstacle-aware filtering.
- Confirm the existing `deployment_range_preview_dto(employee_uuid, position, facing)` helper is side-effect free.
- Confirm `PlayerBehavior` and `BehaviorResult` serialization style before naming new variants.
- Confirm game_server command result mapping can carry the new typed payload without ad-hoc JSON.
- Confirm external Unity docs and smoke probe expectations after DTO shape is final.

## Code Review Findings Before Implementation

- `src/game/range_preview.rs` currently resolves preview cells through `resolved_walkable_cells(...)`, which filters with `battlefield.is_walkable_tile(...)`. This conflicts with the latest policy because obstacle/blocked tiles must stay visible in attack/skill range preview. The implementation should switch this to valid-tile filtering and rename/refactor the helper so the code no longer implies walkability semantics.
- A new pending preview request must be added to `ActionKind`, `PlayerBehavior::kind()`, payload validation, and `ActionScheduler`'s `GameState::InBattle` allowed actions. Otherwise the normal state gate will reject it before the command reaches the preview handler.
- `game_server` maps ordinary non-battle `BehaviorResult` values to a `command_result` followed by a full `state_snapshot`. Pending deployment preview is an in-battle read request, so its mapping must explicitly avoid the trailing `state_snapshot`; otherwise Unity could receive a legacy snapshot after a preview response and risk stale battle presentation overwrite.
- The existing `GameCore::deployment_range_preview_dto(employee_uuid, position, facing)` helper is a good one-facing building block because it takes `&self`, validates the deployment cell/profile, and returns `LiveBattleRangePreviewsDto` without allocating a live unit or mutating battle state.

## Implemented Resolution

- `range_preview.rs` now resolves final preview cells through valid battlefield bounds, not walkability, so static obstacle cells remain visible in attack range overlays.
- The pending preview request is wired through core action gating and game server command deserialization/serialization as a typed `DeploymentRangePreview` command result.
- `DeploymentRangePreview` is explicitly mapped as a read-only in-battle result and does not request a trailing full `state_snapshot`.
- The live WebSocket probe verifies the real command shape, all four facing previews, and the absence of a trailing `state_snapshot`.

## Questions To Stop For

Stop and ask the user if any of these appear:

- The pending preview must show target-dependent splash around a hovered enemy.
- A skill currently requires manual tile targeting during deployment preview.
- The request must reserve deploy cost or lock the tile.
- Unity needs a temporary legacy fallback to keep local `defense_tile_range` calculation.
- The server protocol cannot represent the result without changing command envelope semantics.

## Follow-Up Candidates Outside This Goal

- Full manual tile-targeting UX and core request contract.
- Target-hover skill effect preview after a concrete enemy target exists.
- Authoring/debug tool to visualize `defense_tile_range` and resolved `range_previews`.
- Removing obsolete Unity-side `BattlefieldDeploymentRangeModel` after the new pending preview command is consumed.
