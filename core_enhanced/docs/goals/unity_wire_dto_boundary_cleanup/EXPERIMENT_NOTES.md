# Experiment Notes: Unity Wire DTO Boundary Cleanup

## Policy Decisions

- Prefer explicit wire DTOs over exposing internal runtime structs.
- Accept some DTO duplication to keep the Unity-facing contract stable and readable.
- Do not use `Debug` formatting as contract values.
- Do not keep legacy aliases or compatibility layers without explicit transition approval.
- Preserve existing Unity JSON shape when the client already consumes it and the goal can be satisfied by moving the Rust type boundary.
- Treat `CombatPreview` as an intentional Unity-facing pre-battle DTO for now. Splitting it would affect battle setup/preview/UI coordination and is outside this cleanup unless a later goal explicitly chooses that migration.
- Treat `NodeSession` as an intentional current-node session snapshot wire struct. It should stay small and identity/payload oriented.

## Initial Risk Notes

- `BattleEventLogEntry` is used by tests and internal validation heavily; do not rename or reshape it casually. Add conversion at the wire boundary instead.
- `CombatPreview` may have mixed responsibilities. If it is too deeply shared, split in small stages and document each boundary.
- External Unity docs are canonical for client wire shape, so they must be checked and updated if JSON changes.

## 2026-07-07 Findings

- External Unity docs and Unity code consume `battle_update.events_delta.events` but do not require the Rust type name `BattleEventLogEntry`; preserving JSON shape avoids a Unity migration.
- `BattleEventLog` remains valid for battle records/debug exports/result attachments. Live transport now converts entries through `LiveBattlePresentationEventDto`.
- `CombatPreview.enemy_stat_scale` is already skipped and covered by existing combat preview tests; no split was needed in this goal.
- The remaining `Debug` derives found by broad search are ordinary Rust derives or internal/test/debug structures, not wire-visible `format!("{:?}")` contract values in the touched snapshot DTOs.

## Follow-Up Candidates

- A broader wire versioning strategy may be needed if Unity already consumes internal shapes.
- Probe scripts may need their own cleanup if they currently accept legacy aliases.
