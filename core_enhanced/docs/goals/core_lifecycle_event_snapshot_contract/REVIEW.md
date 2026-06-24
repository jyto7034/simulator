# Core Lifecycle Event Snapshot Contract Review

Review guide: `docs/goal_completion_review_guide.md`

Review date: 2026-06-23

## Item Verdict

| Item | Verdict | Long-term fit | Evidence |
|---|---|---|---|
| `UnitWithdrawn` event position contract | Complete | High | Event payload now includes `world_position` and projected tile `position`, emitted from retained `RuntimeUnit.body`. |
| `UnitDied` event position contract | Complete | High | Event payload now includes `world_position` and projected tile `position`, emitted from retained `RuntimeUnit.body` at death finalization. |
| Official checkpoint Active-only contract | Complete | High | Live checkpoint construction filters `unit.is_active()` and rulebook says checkpoint units are gameplay presentation/reconcile source only. |
| Debug/replay inactive visibility | Complete | Medium | Added `BattleCore::debug_inactive_units()` returning inactive retained runtime units with lifecycle and position. It is separate from gameplay checkpoint. |
| Event log schema versioning | Complete | High | `BATTLE_EVENT_LOG_VERSION` bumped to 27; no serde defaults or dual-schema fallback were added for the new event fields. |

## Evidence

Runtime evidence:

- `src/game/battle/event_log.rs`
  - `UnitWithdrawn` carries `unit_instance_id`, `world_position`, and `position`.
  - `UnitDied` carries `unit_instance_id`, `owner`, `killer_instance_id`, `world_position`, and `position`.
  - `BATTLE_EVENT_LOG_VERSION` is 27.
- `src/game/battle/core/build.rs`
  - `withdraw_unit` captures `unit.body.position` and `unit.body.projected_tile()` before recording `UnitWithdrawn`.
- `src/game/battle/core/commands.rs`
  - `finalize_unit_death` records `UnitDied` with body-derived world/tile position.
- `src/game/world/state.rs`
  - `LiveBattleStateCheckpointDto.units` is built from `.filter(|unit| unit.is_active())`.
- `src/game/battle/core/mod.rs`
  - `debug_inactive_units()` filters `!unit.is_active()` and returns lifecycle plus body-derived position data.

Data evidence:

- No live RON/data schema was changed.

External contract evidence:

- `docs/game_rulebook.md` now states that `UnitWithdrawn`/`UnitDied` event positions drive inactive transition presentation, while `battle_update.checkpoint.units` remains Active-only.
- The event log schema version was bumped instead of adding backwards-compatible default fields.

Test evidence:

- `apply_live_command_deploys_and_withdraws_player_unit` now verifies `UnitWithdrawn.world_position` and `position`.
- `apply_hp_delta_records_died_stop_with_latest_continuous_position` now verifies `UnitDied.world_position` and `position`.
- `debug_inactive_units_lists_retained_non_active_units_with_lifecycle_and_position` verifies inactive-only debug query output with lifecycle and positions.

Legacy/fallback audit:

- No inactive units were added to official gameplay checkpoint.
- No compatibility layer or old event shape fallback was added.
- No battlefield-layout position fallback was introduced; position source remains `RuntimeUnit.body`.

Long-term direction review:

- Fit: high for event/checkpoint contracts, medium for debug visibility.
- Improvement class: follow-up refactor for debug/admin transport surface only if an external admin endpoint needs inactive runtime inspection.
- Reason: event presentation and checkpoint reconciliation now have distinct sources. The debug query is intentionally read-only and separate, but it is currently a core API rather than a server/admin transport command.

## Validation

Passed:

- `cargo fmt`
- `cargo test apply_live_command_deploys_and_withdraws_player_unit`
- `cargo test apply_hp_delta_records_died_stop_with_latest_continuous_position`
- `cargo test debug_inactive_units_lists_retained_non_active_units_with_lifecycle_and_position`
- `cargo check`
- `cargo test retreat_from_live_defense_battle_reenters_until_attempts_are_exhausted`
- `cargo test -- --test-threads=1`

Observed and isolated:

- Parallel `cargo test` failed once in `retreat_from_live_defense_battle_reenters_until_attempts_are_exhausted` with battle-record JSON EOF. The same test passed alone, and serial full validation passed. This appears to be pre-existing parallel debug artifact contention, not a lifecycle event contract regression.

## Remaining Risk

- `debug_inactive_units()` is a core debug/replay query, not a public server/admin command. If Unity or external admin tooling needs this data over transport, add a typed debug/admin endpoint in a separate goal rather than mixing inactive units into gameplay checkpoint.
- Parallel tests can contend on battle record/debug output files. This is outside the current lifecycle contract scope but should be cleaned up by a future test artifact isolation goal.

## Verdict

Complete / High fit for lifecycle event and checkpoint contracts.
