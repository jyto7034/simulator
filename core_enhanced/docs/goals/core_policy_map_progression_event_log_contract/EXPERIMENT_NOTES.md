# Experiment Notes

## Working Principles

- Implement the long-term direction, not a compatibility patch.
- Runtime code, live RON/data, Unity/server-facing contracts, and latest policy docs are source of truth in that order.
- Do not keep old and new DTO fields in parallel.
- Do not keep old and new event-log schema names in parallel.
- Tests should pin external contract behavior and gameplay-observable flow, not only private helper layout.
- Small trial-and-error is allowed, but failed approaches and reasons must be recorded in `EXPERIMENTS.md`.

## Confirmed Decisions

### Map Progression

- `MapNodeDto.state` is the canonical external source for node availability and completion.
- `RunSnapshotDto.map_progression.available_node_ids` and `RunSnapshotDto.map_progression.completed_node_ids` should be removed.
- Internal `MapProgression` may keep list storage for progression calculation if it does not leak as a competing Unity-facing source.

### Battle Event Log Naming

- The old `Timeline` type/JSON contract should be renamed to event-log terminology.
- This is an intentional contract change; compatibility aliases are not desired.
- Preferred Rust domain name: `BattleEventLog`.
- Preferred public field naming: `event_log` or `battle_event_log`.
- Preferred compressed attachment naming: `compressed_event_log` or `compressed_battle_event_log`.
- `docs/refactor_preparation_plan.md` still records the previous baseline that `Timeline` may remain, but this goal uses the later user-confirmed policy: rename the public type/JSON contract instead of preserving the old name.

## Questions To Watch

Record `사용자와 정책 논의 필요` if implementation discovers:

- a Unity/server consumer that requires the old map progression lists for a user-visible workflow that cannot derive from `MapNodeDto.state`,
- saved data that stores old timeline/event-log payloads and requires a migration decision,
- event enum names that should or should not be renamed as part of the same public contract,
- server transport envelope fields where renaming changes client UX or recovery behavior,
- live debug/export artifacts that are currently used as official fixtures.

## Known Risks

- `MapProgressionSnapshotDto` cleanup is likely small, but tests may still read `map_progression.available_node_ids` as a convenience helper.
- `Timeline` naming is broad because it touches core battle runtime, validation, tests, server transport, and debug export wording.
- Some documentation may use `timeline` historically; update current policy/contract docs, but do not churn unrelated historical notes unless they would mislead future implementation.
- The event enum rename was broad, but it was included so no public Rust event-log type keeps the old timeline terminology.

## Implementation Notes

- The event enum was renamed too: `TimelineEvent` became `BattleLogEvent`. This avoids leaving a public Rust event-log type with the old timeline name.
- The validation API was renamed from `TimelineValidator` / `TimelineViolation*` to `EventLogValidator` / `EventLogViolation*`.
- `BattleCore.timeline` and `BattleResult.timeline` became `event_log`; sequence tracking became `event_log_seq`.
- The live battle update field `events_delta` remains unchanged because it already describes event entries rather than the old timeline contract name.
- `MapProgressionSnapshotDto` now only exposes `current_node_id`. Availability/completion remain visible through `MapViewDto.nodes[].state`.
- Internal `MapProgression.available_node_ids` / `completed_node_ids` remain as runtime calculation state. They are not serialized through `RunSnapshotDto.map_progression`.
- Server combat-result attachment fields now use `compressed_event_log`, `event_log_encoding`, and `event_log_gzip_base64`.

## Policy Questions

None discovered during implementation.

## Remaining Risks

- Unity client code outside this repository must be updated to read `has_event_log`, `compressed_event_log`, `event_log_encoding`, and `event_log_gzip_base64`.
- Historical docs and old goal records may still mention `Timeline` as past terminology. Current runtime/server-facing code and current policy baseline were updated.

## Follow-Up Candidates Outside This Goal

- Remove internal `SkillStepDef.range_units` and `SkillCastTargetingDef::FirstStepTarget`; replace test/fixture convenience with explicit cast-targeting helpers that do not infer from steps.
- Audit `Battlefield::in_bounds` during the next tile/range implementation pass; only rename or wrap it if the call sites confuse raw bounds with valid-tile clipping policy.
- Remove `Battlefield` single-tile `occupant` projection and related single-owner tile semantics; unit position/body state remains the source of truth, and any tile membership acceleration must be multi-occupant and derived.
- Move remaining server/admin snapshot JSON wrapper shape assembly to typed DTOs. `serde_json::Value` may remain only at final serialization/logging/generic passthrough boundaries, not as the owner of Unity/server-facing contract fields.
