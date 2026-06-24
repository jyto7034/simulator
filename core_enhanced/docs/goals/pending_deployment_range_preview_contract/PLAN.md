# Pending Deployment Range Preview Contract Plan

## Objective

Implement a long-term core-to-Unity contract for pending deployment range preview.

Unity must not calculate attack range overlays from `defense_tile_range`, `effective_weapon_profile`, `effective_basic_attack`, skill catalog metadata, or local fallback rules. This applies both to already deployed live units and to pending placement while the player is choosing a facing direction before sending `deploy_unit`.

Core should provide a request/response contract that lets Unity ask:

```text
If this employee were placed on this tile, what are the final range preview cells for each facing?
```

The response must return final core-computed cells for `up`, `right`, `down`, and `left` in one request so Unity can render direction hover instantly without recomputing range locally.

## Source Of Truth Order

Verify facts in this order during implementation:

1. Runtime code.
2. Live RON/data.
3. Unity-facing snapshot/command/update contracts.
4. Latest policy docs.

Do not trust this plan over runtime code. If code reading shows a better long-term shape, record the evidence in `EXPERIMENT_NOTES.md` and apply it unless it requires a user policy decision.

## Current Policy To Preserve

- Confirmed deployed units expose live range overlays through `battle_update.checkpoint.units[*].range_previews`.
- Pending placement is not a deployed unit and must not appear in `checkpoint.units`.
- Direction selection UX needs range preview before `deploy_unit` is sent.
- `deploy_unit(employee_uuid, position, facing)` remains the only command that actually deploys a unit.
- Pending preview is a hint for the current state, not a reservation or guarantee.
- Final deploy validation must run again when `deploy_unit` is received.
- Range preview cell filtering is based on valid battlefield tiles, not walkable tiles.
- Obstacle/blocked tiles remain valid range preview cells.
- Void, out-of-bounds, and invalid tiles are excluded from range previews.
- Actual hit eligibility is determined later by core target validation, `air_capable`, target policy, and hostile-target usefulness rules.

## In Scope

- Audit the current `range_preview` helper and fix it if it filters by walkability/obstacles instead of valid tiles.
- Add a Unity-facing pending deployment range preview request.
- Add a typed result DTO that returns four facing previews in one response.
- Reuse the existing core helper that computes one facing's `LiveBattleRangePreviewsDto` where appropriate.
- Validate that the requested employee and position are legal for pending deployment preview.
- Add the new request to `ActionKind`, `PlayerBehavior::kind()`, payload validation, and the `InBattle` allowed action list so it is accepted by the same state gate as other live battle requests.
- Ensure the request does not mutate battle state:
  - no deploy cost consumed
  - no unit instance id allocated for a live unit
  - no battlefield insertion
  - no timeline event
  - no checkpoint unit
- Ensure the command result does not emit a trailing full `state_snapshot`. Pending preview is an in-battle read request and must not let legacy snapshot transport overwrite live battle presentation state.
- Update external Unity canonical docs.
- Update Python WebSocket smoke probe to verify the new request/response on a live server.
- Add focused tests for DTO shape and gameplay-visible behavior.

## Out Of Scope

- Unity rendering implementation.
- Unity pending placement state machine.
- Manual tile-targeting skill UX.
- New skills, weapons, range patterns, balance values, or RON schema changes.
- Removing existing live checkpoint `range_previews`.
- Compatibility layers that allow Unity to keep calculating range from `defense_tile_range`.
- Precomputing every employee x every deployment tile x every facing preview at battle start.

## Target Command Contract

Suggested request:

```json
{
  "type": "command",
  "request_id": "preview-1",
  "behavior": {
    "type": "request_deployment_range_preview",
    "employee_uuid": "uuid",
    "position": { "x": 6, "y": 6 }
  }
}
```

Suggested successful result:

```json
{
  "type": "command_result",
  "request_id": "preview-1",
  "ok": true,
  "result_type": "DeploymentRangePreview",
  "payload": {
    "employee_uuid": "uuid",
    "position": { "x": 6, "y": 6 },
    "facings": {
      "up": { "range_previews": {} },
      "right": { "range_previews": {} },
      "down": { "range_previews": {} },
      "left": { "range_previews": {} }
    }
  }
}
```

Exact Rust type names and payload nesting may change after code reading, but the meaning must stay stable:

- one command request
- one command result
- no battle state mutation
- four facing previews returned together
- each facing uses the same final-cell `LiveBattleRangePreviewsDto` contract as live checkpoint units

## Error Policy

Use the existing command error style:

- invalid action or not in active battle
- unknown employee
- employee cannot deploy
- invalid deployment position
- missing deployable profile

Do not return partial facing results for invalid positions. If the pending position itself is invalid, the request should fail clearly.

## Initial Code Reading Targets

Read before implementation:

- `src/game/behavior.rs`
  - `PlayerBehavior`
  - `BehaviorResult`
  - `LiveBattleRangePreviewsDto`
- `src/game/range_preview.rs`
  - current filtering logic
  - basic attack fallback
  - active skill preview behavior
- `src/game/world/combat.rs`
  - `deploy_unit`
  - `deployment_range_preview_dto`
  - active battle/deployment validation
- `src/game/world/state.rs`
  - checkpoint unit `range_previews`
  - live deployment state
- `src/game/world/snapshot.rs`
  - skill catalog range source marker
- `../game_server/src/game/player_game_actor/messages.rs`
  - command behavior deserialize path
- `../game_server/src/game/player_game_actor/state.rs`
  - `BehaviorResult` to `command_result.payload` mapping
- external Unity docs:
  - `/mnt/f/unity projects/ark/docs/unity_core_contract.md`
  - `/mnt/f/unity projects/ark/docs/core_unity_battle_transport_contract.md`
  - `/mnt/f/unity projects/ark/docs/probe/unity_ws_smoke_probe.py`

## Implementation Sketch

1. Audit and, if needed, repair range preview filtering:
   - replace walkable/obstacle filtering with valid-tile filtering for range preview cells.
   - keep deploy position validation separate from preview cell filtering.
   - rename or refactor helper names such as `resolved_walkable_cells` if they keep implying walkability semantics after the policy changes.
2. Add typed DTOs:
   - `DeploymentRangePreviewResultDto`
   - `DeploymentRangePreviewFacingDto` or equivalent.
3. Add `PlayerBehavior::RequestDeploymentRangePreview`.
4. Add `BehaviorResult::DeploymentRangePreview`.
5. In `GameCore`, implement pending preview result:
   - validate active battle state
   - validate employee and deployable profile
   - validate requested placement tile
   - call one-facing helper for all four `FacingDirection` values
   - return final cells for each facing
6. Add the request to state/action gating:
   - `ActionKind::RequestDeploymentRangePreview`
   - `PlayerBehavior::kind()`
   - `validate_behavior_payload`
   - `ActionScheduler` `GameState::InBattle` allowed actions
7. Wire the result through game_server command/result serialization:
   - add `PlayerBehaviorRequest::RequestDeploymentRangePreview`
   - map it to the core behavior
   - serialize `BehaviorResult::DeploymentRangePreview` as `command_result`
   - set `send_state_snapshot: false` for this result so no full snapshot follows the preview response
8. Update external Unity contract docs:
   - explain the pending placement request
   - explain that Unity requests once after tile release and uses returned facing previews for hover
   - reiterate that `deploy_unit` is still the only state-changing deployment command
9. Update WebSocket probe:
   - request pending preview after entering battle and before deploy
   - assert right-facing fallback includes own tile and right cell
   - assert no player checkpoint unit exists before deploy
   - deploy unit and verify live checkpoint range preview matches the same policy

## Tests To Add Or Update

Focused tests should cover:

- Pending preview returns exactly four facing entries.
- Right-facing fallback returns own tile plus the right tile when both are valid.
- Invalid/out-of-bounds/void cells are excluded from final preview cells.
- Obstacle/blocked cells are not removed from final preview cells.
- Invalid pending deployment position returns an error instead of partial previews.
- Pending preview does not create a live unit, timeline event, deployment state, or deploy cost change.
- Pending preview command is allowed in `GameState::InBattle` and rejected outside live battle through the normal action gate.
- Pending preview command result is not followed by a full `state_snapshot`.
- Actual `deploy_unit` still performs authoritative validation and then checkpoint `range_previews` uses the same final-cell policy.
- WebSocket smoke probe confirms the real JSON shape.

Do not preserve tests that require Unity to calculate pending range from `defense_tile_range`.

## Stop Conditions

Stop and ask the user before proceeding if implementation discovers:

- Current DTO/result mapping cannot add a new command without broader server protocol redesign.
- Valid-tile filtering requires changing battlefield/RON schema.
- Pending preview requires reserving deploy cost, consuming state, or allocating live unit ids.
- Active skill preview needs a UX decision about target-dependent splash or manual tile targeting.
- A compatibility layer or dual schema appears necessary.
- Existing live Unity code requires preserving local `defense_tile_range` calculation as a fallback.

## Completion Criteria

- Pending deployment preview has a typed core command/result contract.
- The result returns final range preview cells for all four facings.
- The request is side-effect free.
- `deploy_unit` remains the only deployment state-changing command.
- Range preview filtering uses valid tiles and does not remove obstacle/blocked cells.
- The request is included in live battle action gating and is not followed by a full `state_snapshot`.
- Unity-facing contract docs are updated.
- WebSocket probe verifies the new contract against a live server.
- Focused tests and a final broad validation command have been run.

## Required Completion Report

When implementation finishes, report:

- Changed DTOs and JSON shape.
- New command/result behavior.
- New fixed range filtering policy.
- Removed or avoided legacy Unity-side range dependencies.
- Updated docs and probes.
- Tests added/updated.
- Remaining risks.
- Exact validation commands run.
