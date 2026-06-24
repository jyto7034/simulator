# Core Tile Validity And Typed DTO Cleanup

## Objective

Implement the smaller unimplemented policy items that are mostly contract/naming cleanup:

1. Audit `Battlefield::in_bounds` call sites during the tile/range pass.
2. Move remaining server/admin snapshot JSON wrapper shape assembly to typed DTOs.

This goal intentionally groups these two because they are comparatively low gameplay-risk and can be validated with focused contract tests.

## Required Startup Protocol

Before code search, edits, or validation, read:

- `docs/goals/core_tile_validity_typed_dto_cleanup/PLAN.md`
- `docs/goals/core_tile_validity_typed_dto_cleanup/EXPERIMENTS.md`
- `docs/goals/core_tile_validity_typed_dto_cleanup/EXPERIMENT_NOTES.md`
- `docs/goals/core_unimplemented_policy_implementation_master/PLAN.md`
- `docs/goals/core_component_refactor_master/POLICY_DECISIONS.md`
- `docs/refactor_preparation_plan.md`

## Source Of Truth

1. Runtime code.
2. Live RON/data.
3. Unity/server-facing DTO and WebSocket contracts.
4. Current policy docs.
5. Goal history.

## Scope

In scope:

- Read all `Battlefield::in_bounds` call sites.
- Decide from code evidence whether the current name is clear enough or whether a rename/wrapper is needed.
- If needed, introduce a name that distinguishes raw rectangle bounds from authored valid-tile policy.
- Replace server/admin `serde_json::Value` shape mutation for Unity/server-facing snapshot fields with typed DTOs.
- Specifically address the current `selected_event["compressed_event_log"] = ...` style post-processing.
- Update tests that assert the public snapshot/server message shape.

Out of scope:

- Removing `Battlefield` tile `occupant`.
- Removing `SkillStepDef.range_units` or `FirstStepTarget`.
- Changing Unity-facing field names unless already confirmed.
- General admin API redesign not needed for typed snapshot boundary.

## Plan

1. Inventory `Battlefield::in_bounds` callers and classify each call as raw bounds, valid tile, target eligibility, or other.
2. If calls already consistently mean valid tile, record evidence and avoid adding wrapper.
3. If meaning is mixed or unclear, rename/wrap with the smallest long-term API that removes ambiguity.
4. Inventory server/admin `serde_json::Value` snapshot shape assembly.
5. Design typed DTO boundary for server-enriched run snapshots and selected-event attachments.
6. Move ad-hoc string-key insertion to typed DTOs.
7. Keep `Value` only at final serialization/logging/generic passthrough boundaries.
8. Run focused tests after each change group.
9. Run broad validation and record results.

## Validation Plan

Focused:

- Tests covering range preview/final valid tile clipping if `in_bounds` API changes.
- Server state snapshot test covering `compressed_event_log`.
- Admin snapshot/catalog tests touched by typed DTO changes.
- `cargo check -p game_server`.

Broad:

- `cargo check --lib`
- `cargo test --lib -- --test-threads=1`
- `cargo test --test ron_loading -- --test-threads=1`
- `cargo check -p game_server`
- `git diff --check`

## Completion Conditions

- `Battlefield::in_bounds` audit is recorded with evidence.
- If renamed/wrapped, all changed call sites use the clearer API and tests pass.
- Unity/server-facing snapshot shape is represented by typed DTOs at server/admin boundaries where this goal touches it.
- No gameplay/server-facing contract field is assembled by mutating `serde_json::Value` with string keys in the affected path.
- No compatibility duplicate shape is added.
- Validation results are recorded.

