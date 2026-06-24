# Core Follow-Up Policy Implementation Master Review

Review guide: `docs/goal_completion_review_guide.md`

Review date: 2026-06-23

## Summary Verdict

| Master Item | Subgoal | Verdict | Long-term fit | Evidence |
|---|---|---|---|---|
| Snapshot/error contract cleanup | `core_snapshot_error_contract_cleanup` | Complete | High | Core owns `RunSnapshotDto`; server flattens it. Static obstacle blocking now uses `StaticObstacleBlocked` / `static_obstacle_blocked`. |
| Static battlefield layout extraction | `core_battlefield_layout_extraction` | Complete | High | `BattlefieldLayout` owns only static layout data; battle unit positions derive from `RuntimeUnit.body`. |
| Runtime unit lifecycle | `core_runtime_unit_lifecycle` | Complete | High | `RuntimeUnitLifecycle` is the active/dead/withdrawn source; withdrawal cleanup emits explicit typed events. |
| Redeploy lifecycle policy | `core_redeploy_lifecycle_policy` | Complete | High | Redeploy creates a new runtime instance and applies explicit withdraw/death HP policies from live redeploy locks. |
| Inactive unit effect resolution | `core_inactive_unit_effect_resolution` | Complete | High | Withdrawn/dead targets are excluded at projectile/effect resolve boundaries; hostile projectiles aimed at withdrawn units cancel/miss. |
| Lifecycle event/snapshot contract | `core_lifecycle_event_snapshot_contract` | Complete | High for gameplay contracts, Medium for debug transport surface | `UnitWithdrawn`/`UnitDied` carry positions, checkpoint units remain Active-only, debug inactive visibility is separate. |

## Cross-Component Findings

1. No competing position source remains in battle runtime.
   - `BattlefieldLayout` stores dimensions, valid tiles, and static obstacles only.
   - Runtime position source is `RuntimeUnit.body`.
   - DTO/event projected tile positions are derived from body/world position.

2. Lifecycle is the participation source of truth.
   - `RuntimeUnitLifecycle::{Active, Withdrawn, Dead}` owns active/dead/withdrawn meaning.
   - HP 0 and `ActionState::Dead` are invariants of death, not the canonical lifecycle source.
   - New movement, targeting, skill, damage, and checkpoint candidate paths gate on `unit.is_active()`.

3. Redeploy state and old runtime entities no longer compete.
   - Old withdrawn/dead runtime units remain in `BattleCore.units` for references/debug/replay.
   - `LiveBattleDeploymentState.deployed_units` points only to the newest active deployment.
   - `LiveBattleRedeployState.current_hp_policy` owns next-deploy HP policy.

4. Withdrawal projectile behavior is consistent with the clarified policy.
   - Withdrawal is a strong defensive/evasion action against opponent hostile projectiles.
   - Basic attack projectile miss uses `BasicAttackProjectileImpacted { hit: false }`.
   - Skill projectile no-hit uses `SkillProjectileImpacted { first_hit_unit_id: None, ... }`.
   - Withdrawal does not globally clear the projectile registry; each projectile resolves at its normal advance/impact boundary.

5. Event presentation and checkpoint reconciliation are separated.
   - `UnitWithdrawn` and `UnitDied` carry `world_position` and projected `position`.
   - `battle_update.checkpoint.units` remains Active-only.
   - Inactive unit inspection is debug/replay-only through `BattleCore::debug_inactive_units()`.

## Runtime Evidence

- `src/game/battle/battlefield/mod.rs`: `BattlefieldLayout` fields are `width`, `height`, `valid_tiles`, and `static_obstacles`.
- `src/game/battle/core/types.rs`: `RuntimeUnitLifecycle` exists and `RuntimeUnit::is_active()` / `is_dead()` delegate to it.
- `src/game/battle/core/build.rs`: `withdraw_unit` sets lifecycle to `Withdrawn`, preserves body position, records `UnitWithdrawn`, clears active state, expires buffs with withdrawn reasons, and skips inactive units in `build_runtime_field`.
- `src/game/battle/core/commands.rs`: death finalization sets lifecycle `Dead`, records body-derived `UnitDied`, and projectile/damage paths skip non-active targets before hit/damage.
- `src/game/battle/core/skill_runtime/projectile.rs`: skill projectile impact filters first-hit targets through `unit.is_active()`.
- `src/game/world/state.rs`: checkpoint `units` filters `unit.is_active()` and stale deployment reconciliation creates withdraw/death redeploy HP policies.
- `src/game/world/combat.rs`: withdraw/deploy commands store and consume redeploy HP policy through `LiveBattleRedeployState`.
- `src/game/battle/event_log.rs`: `BATTLE_EVENT_LOG_VERSION` is 27; `UnitWithdrawn`/`UnitDied` carry positions; withdrawal cleanup has typed cancel/expire reasons.

## Data And Contract Evidence

- No live RON schema was changed by the follow-up lifecycle/layout/effect/event subgoals.
- Live RON loading remains covered by `cargo test -- --test-threads=1`, including `tests/ron_loading.rs`.
- `../game_server/src/game/player_game_actor/state.rs` uses `#[serde(flatten)] core: RunSnapshotDto<PlayerSelectedEventSnapshotDto>` for `PlayerStateSnapshotDto`.
- `GameError::StaticObstacleBlocked` maps to `static_obstacle_blocked`; `PositionOccupied` remains only for true occupied-slot style errors.
- `docs/game_rulebook.md` documents `UnitWithdrawn`/`UnitDied` event positions as inactive transition presentation source and checkpoint units as Active-only.
- `docs/goals/core_component_refactor_master/POLICY_DECISIONS.md` remaining status for this master scope is updated as implemented, with no unresolved policy blocking the sequence.

## Legacy / Fallback Audit

- Search found no battle runtime `Battlefield` dynamic occupant APIs: `position_of`, `place`, `remove`, `units_at`, battle tile `occupant`, or `unit_pos`.
- Remaining `occupant` references are in `src/game/resources/board.rs`, a roster/slot domain outside battle layout.
- `ProjectileMiss` is not a runtime event variant; projectile miss meaning is canonicalized through impact events.
- No serde defaults or dual-schema compatibility were added for the `UnitWithdrawn`/`UnitDied` position fields; event log version was bumped.
- No old runtime unit reactivation path was introduced for redeploy.
- No global projectile cleanup workaround was introduced for withdrawal.

## Tests Updated Or Covering The Contracts

Representative tests observed in the final serial run:

- `advance_basic_attack_projectile_misses_when_locked_target_withdraws_before_contact`
- `skill_projectile_impact_does_not_hit_withdrawn_target`
- `apply_live_command_deploys_and_withdraws_player_unit`
- `apply_hp_delta_records_died_stop_with_latest_continuous_position`
- `debug_inactive_units_lists_retained_non_active_units_with_lifecycle_and_position`
- `live_defense_deploy_and_withdraw_manage_cost_and_redeploy_lock`
- `live_deployment_reconciles_defeated_player_unit_before_state_dto`
- `static_obstacles_block_placement`
- `live_ron_defense_route_playable_path_runs_to_combat_result`
- `load_game_data_from_ron_reads_run_policy`
- `load_game_data_from_ron_reads_step_based_skill_schema`

## Remaining Risk

- `BattleCore::debug_inactive_units()` is a core debug/replay query, not a public server/admin transport command. Add a typed debug/admin endpoint later if external tooling needs inactive runtime inspection.
- Parallel full `cargo test` previously hit a battle-record JSON EOF in one retreat test, while focused rerun and serial full validation passed. This should be handled by a future test artifact isolation cleanup, not by changing gameplay contracts.
- Unity/external consumers must adopt the new event log schema version 27 and the `UnitWithdrawn`/`UnitDied` position fields.
- General server/admin `serde_json::Value` payload wrappers still exist in handler/message/admin boundaries, but the specific `PlayerStateSnapshotDto` root drift item is complete. Broader typed server/admin DTO cleanup belongs to a separate goal.

## Validation

Passed in this final audit pass:

- `cargo check`
- `cargo test -- --test-threads=1`

Previously recorded per-subgoal focused validation remains in each subgoal `REVIEW.md`.

## Final Verdict

Complete.

The master goal's confirmed follow-up policies were implemented, reviewed per subgoal, cross-checked against current runtime code and policy documents, and broad serial validation passed.
