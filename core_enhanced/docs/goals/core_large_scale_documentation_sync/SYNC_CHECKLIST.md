# Sync Checklist

## Battle Runtime Lifecycle

| Item | Runtime evidence | Document status | Validation |
|---|---|---|---|
| Runtime unit lifecycle is `Active`, `Withdrawn`, `Dead`; HP/action state are projections/invariants, not lifecycle SoT. | `src/game/battle/core/types.rs`; `BattleCore::debug_inactive_units`. | `docs/game_rulebook.md`, external `unity_core_contract.md`, external `core_unity_battle_transport_contract.md`, external `unity_client_implementation_goal.md` now describe Active-only gameplay checkpoint and retained inactive debug/admin/replay entities. | `rg` stale wording search; `cargo test -- --test-threads=1`. |
| `battle_update.checkpoint.units` exposes Active units only. | `src/game/world/state.rs::live_checkpoint_dto` filters `unit.is_active()`. | External Unity docs now say checkpoint units are `RuntimeUnitLifecycle::Active` only and inactive transitions come from events. | `rg "현재 살아있는"` no longer finds stale wording in synced docs; broad test passed. |
| `UnitWithdrawn` and `UnitDied` event positions are presentation source. | `src/game/battle/event_log.rs`; `src/game/battle/core/build.rs`; `src/game/battle/damage.rs`. | External Unity docs now require `world_position` + projected tile `position` for withdraw/death presentation. | `cargo test -p game_core withdraw -- --nocapture`; `cargo test -- --test-threads=1`. |

## Deployment And Redeploy

| Item | Runtime evidence | Document status | Validation |
|---|---|---|---|
| Redeploy creates a new runtime unit and new `unit_instance_id`. | `src/game/world/combat.rs`; `src/game/world/tests/combat.rs`. | `docs/game_rulebook.md`, external `unity_core_contract.md`, external `core_unity_battle_transport_contract.md`, external `unity_client_implementation_goal.md` now document actor recreation. | `cargo test -p game_core withdraw -- --nocapture`; broad test passed. |
| Withdraw redeploy HP is current HP + 30% max HP capped by max HP. | `BattleDeployCurrentHpPolicy::withdraw_redeploy`; `LiveBattleRedeployState.current_hp_policy`. | Internal/external battle docs now document policy and say Unity must not calculate it pre-redeploy. | `cargo test -p game_core withdraw -- --nocapture`; broad test passed. |
| Death redeploy HP is 60% of new runtime max HP, clamped to 1..max HP. | `BattleDeployCurrentHpPolicy::death_redeploy`; `reconcile_live_deployment_with_battle_state`. | Internal/external battle docs now document policy. | `cargo test -- --test-threads=1`. |

## Withdrawal Cleanup And Projectile Semantics

| Item | Runtime evidence | Document status | Validation |
|---|---|---|---|
| Withdrawn caster creates `SkillCastCancelled { reason: Withdrawn }`, not `SkillCastInterrupted`. | `src/game/battle/core/build.rs`; `src/game/battle/event_log.rs`. | `docs/skill_target_contract.md`, external Unity docs now separate interruption vs lifecycle cancellation. | `cargo test -p game_core withdraw -- --nocapture`; broad test passed. |
| Withdrawn target/caster buff cleanup uses `TargetWithdrawn` / `CasterWithdrawn`. | `clear_active_buffs_for_withdrawn_unit`; `BuffExpireReason`. | External Unity docs now list the new `BuffExpired.reason` values. | `cargo test -p game_core withdraw -- --nocapture`; broad test passed. |
| Basic projectile miss is `BasicAttackProjectileImpacted { hit: false }`; skill projectile no-hit is `SkillProjectileImpacted { first_hit_unit_id: null }`. | `src/game/battle/event_log.rs`; projectile command/runtime code. | `docs/skill_target_contract.md`, `docs/game_rulebook.md`, external Unity docs now describe canonical miss/no-hit events and no `ProjectileMiss`. | `rg ProjectileMiss`; `cargo test -p game_core withdraw -- --nocapture`; broad test passed. |

## Battle Transport

| Item | Runtime evidence | Document status | Validation |
|---|---|---|---|
| `BattleResync` DTO has no `setup` field and wraps `update: battle_update`. | `../game_server/src/game/player_game_actor/messages.rs`; server serialization test. | External `unity_core_contract.md` and `core_unity_battle_transport_contract.md` now remove `setup: null`. | `rg "setup: null"` clean; `cargo test -p game_server player_game_actor -- --nocapture`. |
| Catch-up resync is command envelope `battle_response: "battle_resync"` + core behavior `request_battle_state { since_seq }`. | `../game_server/src/game/player_game_actor/messages.rs`. | External Unity docs and implementation goal now use the current envelope/payload split. | `rg request_battle_resync` clean in synced docs; game_server focused test passed. |
| Setup-loss recovery is `recover_battle_setup_loss`, not `request_battle_resync { need_setup: true }`. | `PlayerBehavior::RecoverBattleSetupLoss`; server tests. | External transport docs and implementation goal now use `recover_battle_setup_loss`. | `cargo test -p game_server player_game_actor -- --nocapture`; broad test passed. |
| `invalid_battle_resync_seq` message names `since_seq`. | `../game_server/src/game/player_game_actor/state.rs`. | Runtime message updated to match documented request field. | `cargo test -p game_server player_game_actor -- --nocapture`; `cargo test -p game_core live_defense_battle_state_rejects_future_since_seq -- --nocapture`. |

## Snapshot And Error Contracts

| Item | Runtime evidence | Document status | Validation |
|---|---|---|---|
| `RunSnapshotDto` owns the core snapshot root; server `PlayerStateSnapshotDto` maps/enriches selected event only. | `src/game/behavior.rs`; `../game_server/src/game/player_game_actor/state.rs`; handler tests. | Existing external docs already describe `state_snapshot.state` as core snapshot root and selected-event enrichment; no edit needed in this pass. | `cargo test -p game_server player_game_actor -- --nocapture`. |
| `StaticObstacleBlocked` / `static_obstacle_blocked` is distinct from `PositionOccupied` / `position_occupied`. | `src/game/world/snapshot.rs`; `../game_server/src/game/player_game_actor/state.rs` test. | External docs already list `position_occupied`; static obstacle distinctness is covered by code/tests and should be expanded if Unity needs error UI copy. | `cargo test -p game_server player_game_actor -- --nocapture`. |

## Documentation Hygiene

| Item | Runtime evidence | Document status | Validation |
|---|---|---|---|
| No local stale Unity contract copy is introduced. | Repo status and docs index. | External canonical docs were edited in place under `/mnt/f/unity projects/ark/docs`. | `git status`, path review. |
| Historical `battle_resync` wording distinguishes server message from old gameplay payload. | Server message DTO and `PlayerBehavior`. | `battle_resync` remains a server message; old `request_battle_resync`/`known_seq`/`need_setup` gameplay payload wording removed from synced docs. | `rg` stale term search. |
