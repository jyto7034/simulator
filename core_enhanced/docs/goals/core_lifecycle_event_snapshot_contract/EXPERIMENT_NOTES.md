# Experiment Notes

- This goal should not start before lifecycle behavior is implemented; otherwise event/snapshot tests may lock the wrong source of truth.
- Official gameplay checkpoint and debug/admin/replay visibility must remain separate.
- Unity-facing contract changes may require policy discussion if existing external consumers expect the old event shape.
- `UnitWithdrawn` and `UnitDied` position fields are a confirmed Unity-facing event-log contract, so no compatibility/default fields were added. `BATTLE_EVENT_LOG_VERSION` was bumped to make the contract change explicit.
- Position source is retained `RuntimeUnit.body`, not battlefield layout or checkpoint. The projected tile `position` is derived from `world_position.project_to_tile()` / `body.projected_tile()`.
- Existing `BattleState` / `LiveBattleUpdateDto.checkpoint.units` should stay Active-only. Inactive retained runtime entities are now exposed through `BattleCore::debug_inactive_units()` as a debug/replay-only query surface, not through gameplay checkpoint.
