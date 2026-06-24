# Experiment Notes

## Policy Notes

- Unity-facing DTO shape changes are policy-sensitive. If the new typed shape is not already determined by confirmed policy, record `사용자와 정책 논의 필요`.
- Serialized compatibility layers should not be added by default. Prefer one clean contract migration.
- WebSocket `Command` remains the transport envelope. `behavior` is now the shared core `PlayerBehavior` payload; `battle_response` is transport metadata for choosing `battle_update` vs `battle_resync` response envelope.
- `request_battle_resync`/`need_setup` was a mixed gameplay/transport command. The canonical gameplay request is now `request_battle_state`; setup recovery is the explicit core behavior `recover_battle_setup_loss`.
- Typed run snapshot migration is wider than the command migration because current root snapshot delegates to `selected_event`, `inventory`, `roster`, and `roster_order` contracts. Avoid a fake typed root that still hides most of the contract in `Value`; nested DTOs must stay explicit.
- Current snapshot migration status: `RunSnapshotDto`, `RunProgressionSnapshotDto`, `MapProgressionSnapshotDto`, `RunResourcesSnapshotDto`, `GameStateContextDto`, `SelectedEventSnapshotDto`, `InventorySnapshotDto`, `EmployeeRosterSnapshotDto`, and `RosterOrderSnapshotDto` are typed. Public `*_json` methods remain as serialization wrappers for existing admin/test call sites, not as the source of truth.
- Combat result timeline ownership is split by meaning vs transport: core exposes `CombatResultTimelineAttachmentDto { winner, event_log }`; server compresses that attachment and preserves the existing Unity-facing `selected_event.compressed_timeline` JSON field.
- `BehaviorResult` is no longer the only place where command outcome meaning exists. Core now projects it into domain contracts (`RunCommandResult`, `NodeCommandResult`, `FacilityCommandResult`, `InventoryCommandResult`, `ShopCommandResult`, `RewardCommandResult`, `BattleCommandResult`) before server transport mapping.
- Server still owns transport decisions: `BattleUpdate` domain contracts become `CommandAccepted` plus side `battle_update`/`battle_resync`, while `ReadOnlyPreview` contracts suppress trailing full `state_snapshot`.

## Follow-Up Candidates

- None yet.
