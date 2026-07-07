# Goal: Unity Wire DTO Boundary Cleanup

## Objective

Separate Unity-facing wire DTOs from internal runtime structs where internal types currently leak across the JSON/WebSocket boundary.

The long-term direction is explicit wire contracts, even if that creates some DTO duplication. Unity-facing JSON must be stable, documented, and intentionally shaped for client consumption. Internal runtime structs may evolve for simulation needs without silently changing wire format.

## Source Of Truth Order

Verify facts in this order:

1. Actual runtime code.
2. Live RON/data.
3. Unity-facing snapshot/command/WebSocket contract and probe output.
4. External Unity canonical docs.
5. Latest core docs.
6. Historical goal/audit docs only as context.

Relevant starting points:

- `src/game/behavior.rs`
- `src/game/world/snapshot.rs`
- `src/game/world/state.rs`
- `src/game/battle/event_log.rs`
- `src/game/combat_preview/types.rs`
- `src/game/combat_preview/mod.rs`
- `src/game/events/combat.rs`
- `tests/ron_loading.rs`
- `tests/live_item_skill_activation.rs`
- External `/mnt/f/unity projects/ark/docs/unity_core_contract.md`
- External `/mnt/f/unity projects/ark/docs/core_unity_battle_transport_contract.md`
- Probe scripts and logs under the Unity/ark side if available.

## Fixed Policies

- Unity-facing JSON must not directly expose internal runtime structs unless they are explicitly declared to be DTO structs.
- `BattleEventLogEntry` is an internal/replay/export log entry type. Unity live transport should receive presentation/event delta DTOs with only the fields the wire contract promises.
- `CombatPreview` should not simultaneously be domain generation intermediate, battle setup input, and public wire DTO unless current code proves this is the intentional contract and no safer separation is practical.
- Debug strings such as `format!("{:?}")` are not stable contract values. Wire values must use explicit enum/string mapping with documented casing.
- Do not keep legacy aliases or fallback fields just to avoid updating tests. If Unity needs a transition, document the transition and removal condition and ask the user before implementing.

## Plan

1. Inventory current wire surfaces.
   - Search `Serialize`/snapshot/result DTOs for `BattleEventLogEntry`, `BattleEventLog`, `CombatPreview`, `NodeSession`, internal runtime state, and `Debug` string formatting.
   - Compare each occurrence against external Unity canonical docs and live probe output.
   - Classify occurrences as internal-only, debug/export-only, test-only, or Unity-facing production wire.

2. Define explicit DTO boundaries.
   - For each production Unity-facing leak, create or reuse a DTO with explicit field names and enum casing.
   - Keep internal structs private to simulation/domain layers where feasible.
   - Preserve behavior while changing shape only where contract docs and probes are updated together.

3. Battle event log boundary.
   - Do not mutate `BattleEventLogEntry` just to satisfy Unity.
   - Add presentation DTO conversion where live `battle_update.events_delta` or command results expose internal log entries.
   - Ensure event seq/cause/source fields remain sufficient for Unity presentation and resync.

4. Combat preview boundary.
   - Decide from code evidence whether `CombatPreview` remains an official pre-battle DTO or should be split into internal generated preview plus Unity preview DTO.
   - If split is needed, keep battle scenario handoff using the internal authoritative preview while Unity receives presentation-only fields.
   - Coordinate with `event_preview_test_artifact_cleanup` if `enemy_stat_scale` is part of the split.

5. Debug string cleanup.
   - Replace wire-visible `Debug` formatting with explicit enum/string serializers.
   - Add tests that assert exact JSON values for affected DTOs.

6. Update docs and probes.
   - Update external Unity contract docs if wire shape changes.
   - Update probe expectations and smoke tests.
   - Do not rely on old aliases unless a transition was explicitly approved.

7. Validate.
   - Run focused DTO/probe contract tests.
   - Run live RON loading tests if preview or encounter shapes are touched.
   - Run `cargo check -p game_core` and `cargo fmt --check`.

## Implementation Notes

- Live `battle_update.events_delta.events` no longer exposes `BattleEventLogEntry` directly. Core converts append-only log entries into `LiveBattlePresentationEventDto` at the state/transport boundary while preserving the existing JSON field shape.
- `CombatPreview` remains an intentional Unity-facing pre-battle DTO. It is still the source for node-confirm preview and battlefield preview UI, while `enemy_stat_scale` remains skipped/internal.
- `NodeSession` remains an intentional run snapshot wire struct for current-node identity/category/payload, not an accidental hidden runtime state leak.
- Wire-visible roster/inventory string labels that previously used `format!("{:?}")` now use explicit mapping helpers.
- No external Unity docs/probes were changed because the JSON shape was intentionally preserved.

## Completion Conditions

- All production Unity-facing JSON DTOs touched by this goal are explicit DTOs or explicitly documented wire structs.
- Internal runtime types are not accidentally used as public JSON just because they derive `Serialize`.
- `BattleEventLogEntry` live exposure is either removed behind a presentation DTO or explicitly documented as debug/export-only.
- `CombatPreview` has a clear boundary: internal, Unity-facing, or deliberately split.
- Wire-visible enum/string values do not depend on `Debug` formatting.
- External Unity docs/probes are updated when wire shape changes.
- Focused tests and final validation commands pass.

## Stop Conditions

Stop and report questions if:

- Unity currently parses an internal type directly and changing it requires a migration policy.
- `BattleEventLogEntry` is confirmed as canonical live wire shape in external docs.
- `CombatPreview` split would cascade into battle setup, map preview, and Unity UI beyond this goal.
- A debug string is already persisted or displayed as user-facing save data.
- DTO changes would require coordinated Unity code changes that are not yet agreed.

## Suggested Verification Commands

```bash
rg -n "BattleEventLogEntry|BattleEventLog|CombatPreview|NodeSession|format!\\(.*\\{:\\?\\}|Debug" src/game tests docs
rg -n "BattleEventLogEntry|CombatPreview|Debug|battle_update|combat_preview" "/mnt/f/unity projects/ark/docs"

cargo test -p game_core --test ron_loading -- --test-threads=1
cargo test -p game_core live_item_skill_activation --test live_item_skill_activation -- --test-threads=1
cargo test -p game_core combat_preview --lib -- --test-threads=1
cargo check -p game_core
cargo fmt --check
```
