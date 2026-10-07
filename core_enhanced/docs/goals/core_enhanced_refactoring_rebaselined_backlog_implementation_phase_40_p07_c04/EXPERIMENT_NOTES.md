# Phase 40 CRA-P07-C04 Experiment Notes

Status: complete
Implementation Complete: true

## Structural Decision

- This is one cleanup boundary: reduce the surviving movement tick contract against the single backend finalized in
  Phase 39. It is not a movement-policy refactor or a container for later event/range/package cleanup.
- Historical candidate names locate likely ghost members but do not prove current existence or non-use. Current
  field/variant producer-reader-serde-public inventory owns every disposition.
- The target is a minimal actual internal contract, not a renamed compatibility wrapper or a new abstraction.

## Direct Dependency

- Phase 39 `Complete / High` is mandatory because the retained Rapier/default path determines which members are
  actually consumed. Its audit hash is `456b37e1...`.
- Boss directives remain consumers of the ordinary movement leaf; target/route/task authority must not be changed.
- Phase 08/09 tactical aggregation/publication behavior and all later completed movement assertions are inherited
  invariants, not cleanup targets.

## Policy And Compatibility

- The user approved all remaining registered cleanup, including verified public/serde breaking removal without shim
  or deprecation after each Phase's inventory and behavior guards.
- Approval is not blanket field deletion. A current runtime reader, produced transition, serialized consumer or test
  assertion is evidence to retain or transfer, and an unknown semantic contract stops that member.
- No dual schema, alias, fallback, deprecated copy or ignored test is authorized.

## Dirty Worktree

- Pre-Child expanded status is 578 rows at `4665ba9b...`; existing rows are user/campaign-owned.
- Phase 40 path did not exist. The three initialized documents are the only expected pre-activation additions.
- Movement carriers are known to be dirty from completed Phases, but their current hashes/diffs are deliberately not
  opened until activation and prospective fingerprinting.
- Protected marker `game_server/logs/app.log.2026-08-09` must remain 57 bytes at `614d2fbd...`.

## Current Evidence Gate

Prospective fingerprints are now fixed. The likely definitions sit in dirty Phase-39 engine/Rapier/Core carriers;
`movement/types.rs`, lifecycle, path and steering are clean and require affirmative current evidence before any edit.
Current discovery must distinguish runtime facts, canonical contracts and the approved target cleanup.

## Current Member Disposition — 2026-08-11

| Current member/surface | Current evidence | Disposition |
| --- | --- | --- |
| `MovementTickInput.now_ms` | destructured directly to `_`; time is consumed before input construction for locks | remove field; retain builder `now_ms` parameter |
| `MovementUnitInput.owner` | copied from RuntimeUnit; no reader | remove |
| `MovementUnitInput.current_target` | copied from RuntimeUnit; no reader; attack identity comes from explicit goal | remove |
| `UnitBody.previous_position` | constructor/relocation/move writes; no production read; one shape test | remove writes/field and shape assertion |
| `UnitBody.velocity` | move/lifecycle/Boss/relocation writes; no production read; one shape/output test | remove writes/field; retain exact endpoint/final-position proof |
| `UnitBody.mode` / `MovementMode` | constructor/relocation writes only | remove field and enum |
| `UnitBody.goal` | body clone is overwritten from planner map each tick, then steering reads it | transfer to explicit `MovementUnitInput.goal`; remove stored slot/writes |
| `UnitBody.physics_handle` / type | always `None`; no reader/writer | remove |
| `MovementOutput::GoalAssigned` | never produced; one no-op match | remove |
| `MovementOutput::SegmentStarted` / reason type | never produced; one no-op match; real event segment is separate | remove |
| `BodyMoved.velocity` | calculated, asserted once, then stored only in unread velocity | remove payload/helper; retain exact `from`/`to`, final position/log proof |
| `MovementStopped.position/reason` / continuous reason type | produced but ignored; actual event stop reasons are a separate retained type | reduce to `MovementStopped { unit_id }` |
| engine tick types and module visibility | no consumer outside movement implementation/tests | make engine/internal tick vocabulary module-private |
| public build/apply/run helper visibility | no external/in-tree non-movement consumer except ordinary Core scheduler | narrow to the smallest Core/movement visibility; remove public apply clone surface |
| `UnitBody.position/radius/move_speed` | live spatial/range/movement authority | retain public/serde exactly |
| `MovementGoal`, `WorldVec2`, `EventLogVec2`, active segments/events | actual intent, geometry and publication authority | retain |

This matrix is exhaustive for the historical candidate and every newly exposed dependent write/payload. The
serialized `BattleLogEvent::MovementStopped` and `MovementSegmentStarted` contracts are explicitly not candidates.

## Out-Of-Scope Handling

Independent improvements are recorded as follow-up candidates only. Phase 41~44 work is not inspected or pulled
forward, and tile-range/`Nearest` behavior remains outside this Phase and unchanged.

## Implemented Boundary — 2026-08-11

- The matrix above was applied without exception: three authoritative `UnitBody` facts remain; intent is an explicit
  tick input; outputs contain only applied transitions; internal engine/steering surfaces are private.
- Serialized battle-log movement events, active segments, targeting, facing, locks, terrain, Boss route directives,
  spatial range and `range_units` remain unchanged. No Phase-41+ source was touched.
- Current legacy/member and outside-module tick-surface searches both select zero. No server/world/data `UnitBody`
  consumer exists, so the user's verified public/serde cleanup approval applies without a compatibility shim.
- Current focused families and both 1,057-test all-target profiles are green. Workspace debug/release checks, normal
  Core Clippy, format and diff checks pass; inherited Clippy warnings are not Phase-40 changes.
- The independent completion audit owns the verdict. Until it returns `Complete / High`, this Child remains active
  and Phase 41 remains absent/pending.
