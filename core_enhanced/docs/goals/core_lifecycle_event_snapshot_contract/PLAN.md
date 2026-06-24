# Core Lifecycle Event Snapshot Contract

## Objective

Implement lifecycle-related event and snapshot contracts after lifecycle, redeploy, and inactive effect behavior are in place.

Confirmed policy source: `docs/goals/core_component_refactor_master/POLICY_DECISIONS.md`, decisions 11, 16, 17, and 18.

## Required Startup Protocol

At the beginning of this goal and every resumed run, read:

- `docs/goals/core_lifecycle_event_snapshot_contract/PLAN.md`
- `docs/goals/core_lifecycle_event_snapshot_contract/EXPERIMENTS.md`
- `docs/goals/core_lifecycle_event_snapshot_contract/EXPERIMENT_NOTES.md`
- `docs/goals/core_followup_policy_implementation_master/PLAN.md`
- `docs/goals/core_component_refactor_master/POLICY_DECISIONS.md`
- `docs/refactor_preparation_plan.md`

Do not start code search, edits, or validation before reading those documents.

## Scope

### In Scope

- Add or update `UnitWithdrawn` event to include `world_position` and projected tile `position`.
- Add or update `UnitDied` event to include `world_position` and projected tile `position`.
- Keep official gameplay checkpoint `units` Active-only.
- Keep death/withdraw presentation driven by event log.
- Expose inactive runtime units only through debug/admin/replay-specific query surfaces if needed.
- Include lifecycle in debug/admin/replay inactive-unit output if such output is added.

### Out Of Scope

- Core lifecycle implementation.
- Redeploy HP and identity.
- Projectile/delayed-effect resolution.
- Unity client rendering changes outside this repository.

## Implementation Plan

1. Read battle event DTOs, event log serialization, checkpoint DTOs, admin/debug surfaces, and Unity-facing docs in this repository.
2. Add/update event payloads for withdraw and death positions using body-derived world and tile positions.
3. Verify official checkpoint `units` includes only Active units.
4. Decide whether existing debug/admin surfaces already cover inactive unit visibility; if not, add a separate debug/admin/replay surface without changing gameplay checkpoint meaning.
5. Update snapshot/event tests and docs.
6. Run focused event/snapshot/server contract tests and `cargo check`.

## Policy Decision Completion Condition

If implementation requires a new Unity-facing event shape, external docs contract, or debug/admin endpoint design not already confirmed, complete the goal with `사용자와 정책 논의 필요` and evidence.

## Completion Conditions

- `UnitWithdrawn` and `UnitDied` events carry `world_position` and projected tile `position`.
- Official gameplay checkpoint remains Active-only.
- Inactive unit visibility, if implemented, is clearly debug/admin/replay-only and includes lifecycle.
- Focused and broad validation commands are recorded.

## Current Status

Status: complete; implementation and review passed.

Implementation:

- `src/game/battle/event_log.rs`
  - Bumped `BATTLE_EVENT_LOG_VERSION` to 27.
  - Added `world_position` and projected tile `position` to `UnitWithdrawn` and `UnitDied`.
- `src/game/battle/core/build.rs`
  - `withdraw_unit` records `UnitWithdrawn` position fields from retained `RuntimeUnit.body`.
- `src/game/battle/core/commands.rs`
  - `finalize_unit_death` records `UnitDied` position fields from retained `RuntimeUnit.body`.
- `src/game/battle/core/mod.rs`
  - Added debug/replay-only `debug_inactive_units()` with lifecycle and position fields.
  - Updated focused tests for withdraw event position, death event position, and inactive debug query.
- `docs/game_rulebook.md`
  - Documented that `UnitWithdrawn`/`UnitDied` event positions are the presentation source for inactive lifecycle transitions, while checkpoint `units` remains Active-only.

Focused validation:

- `cargo fmt`
- `cargo test apply_live_command_deploys_and_withdraws_player_unit`
- `cargo test apply_hp_delta_records_died_stop_with_latest_continuous_position`
- `cargo test debug_inactive_units_lists_retained_non_active_units_with_lifecycle_and_position`
- `cargo check`
- `cargo test` failed once due parallel battle-record JSON EOF in `retreat_from_live_defense_battle_reenters_until_attempts_are_exhausted`; focused rerun passed.
- `cargo test retreat_from_live_defense_battle_reenters_until_attempts_are_exhausted`
- `cargo test -- --test-threads=1`

Review: `docs/goals/core_lifecycle_event_snapshot_contract/REVIEW.md`
