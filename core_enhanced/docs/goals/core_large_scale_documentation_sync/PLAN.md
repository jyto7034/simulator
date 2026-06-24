# Core Large-Scale Documentation Sync

## Objective

Synchronize internal and external documentation with the current post-refactor core/server runtime after the large `core_enhanced` policy and lifecycle changes.

This goal is not a code refactor goal. Its primary deliverable is documentation and contract synchronization proven against actual runtime code, live RON/data, Unity-facing DTO/server contracts, and tests/probes.

## Required Startup Protocol

At the beginning of this goal and every resumed run, read:

- `docs/goals/core_large_scale_documentation_sync/PLAN.md`
- `docs/goals/core_large_scale_documentation_sync/EXPERIMENTS.md`
- `docs/goals/core_large_scale_documentation_sync/EXPERIMENT_NOTES.md`
- `docs/code_documentation_sync_guidelines.md`
- `docs/README.md`
- `docs/refactor_preparation_plan.md`
- `docs/goal_completion_review_guide.md`
- `docs/game_rulebook.md`
- `docs/skill_target_contract.md`
- `/mnt/f/unity projects/ark/docs/unity_core_contract.md`
- `/mnt/f/unity projects/ark/docs/core_unity_battle_transport_contract.md`
- `/mnt/f/unity projects/ark/docs/unity_client_implementation_goal.md`

Do not start code search, document edits, external document edits, validation, or probe work before reading those documents.

If an external Unity canonical document cannot be read or written because of filesystem permissions, record the exact blocker in `EXPERIMENTS.md` and ask for the required permission instead of silently updating a local stale copy.

## Source Of Truth Order

1. Actual runtime code.
2. Live RON/data.
3. Server/Unity-facing snapshot, command, event-log, WebSocket, admin DTO contracts.
4. External Unity canonical documentation.
5. Internal policy/domain documentation.
6. Goal documents and historical notes.

Do not trust existing documents by default. Treat documents as hypotheses until current runtime code, live data, and DTO/test evidence confirm them.

## Sync Scope

### In Scope

Synchronize documentation for the large recent core changes, including at minimum:

1. Runtime unit lifecycle
   - `RuntimeUnitLifecycle::{Active, Withdrawn, Dead}` as active/dead/withdrawn source of truth.
   - HP/action state as invariants/projections, not lifecycle source.
   - Withdrawn/dead units retained in `BattleCore.units` for references/debug/replay.

2. Battlefield/static layout and position source
   - `BattlefieldLayout` as static-only terrain/layout object.
   - `RuntimeUnit.body` as dynamic world-position source.
   - projected tile position as derived from body/world position.
   - no single tile occupant model in battle runtime.

3. Live deployment/redeploy lifecycle policy
   - withdraw redeploy creates a new runtime unit and uses `min(max_hp, withdrawn_hp + floor(max_hp * 0.30))`.
   - death redeploy creates a new runtime unit and uses `max(1, floor(new_runtime_max_hp * 0.60))` clamped by max HP.
   - redeploy lock/checkpoint responsibility.

4. Withdrawal cleanup and inactive effect resolution
   - withdrawal cleanup events for buff expiration and skill cancellation.
   - withdrawal as strong defensive/evasion behavior against opponent hostile projectiles.
   - hostile projectiles aimed at withdrawn targets cancel/miss at normal advance/impact boundaries.
   - no global projectile registry cleanup workaround.
   - withdrawn/dead targets excluded from new damage/effect resolution unless a future explicit policy says otherwise.

5. Battle event log / Unity-facing event contracts
   - `BATTLE_EVENT_LOG_VERSION = 27`.
   - `UnitWithdrawn` carries `unit_instance_id`, `world_position`, `position`.
   - `UnitDied` carries `unit_instance_id`, `owner`, `killer_instance_id`, `world_position`, `position`.
   - `SkillCastCancelled` and `SkillCastCancelReason::Withdrawn`.
   - `BuffExpireReason::{TargetWithdrawn, CasterWithdrawn}`.
   - `BasicAttackProjectileImpacted { hit: false }` as canonical basic projectile miss.
   - `SkillProjectileImpacted { first_hit_unit_id: None, ... }` as skill projectile no-hit/miss presentation.

6. Live battle checkpoint and transport
   - `battle_update.checkpoint.units` is Active-only.
   - inactive transition presentation comes from events, not checkpoint lookup.
   - debug/admin/replay inactive unit visibility remains separate from gameplay checkpoint.
   - no trailing full `state_snapshot` for battle commands if current transport contract says battle update/resync is the presentation source.

7. Snapshot/error/server contract cleanup
   - `RunSnapshotDto` owns core snapshot root shape.
   - `PlayerStateSnapshotDto` flattens core snapshot and only enriches selected event where appropriate.
   - `StaticObstacleBlocked` / `static_obstacle_blocked` is distinct from `PositionOccupied` / `position_occupied`.

8. Documentation index and stale-copy hygiene
   - `docs/README.md` points to the correct canonical documents.
   - No local `docs/unity_core_contract.md` or stale Unity copy is introduced.
   - Completed policy meaning is moved into durable docs instead of only goal docs.

### Out Of Scope

- New gameplay behavior implementation unless a documentation mismatch proves a small code/test fix is required to make the documented current contract true.
- Unity client scene/UI implementation.
- Broad server/admin typed DTO cleanup unrelated to the documented changed contracts.
- Parallel battle-record artifact isolation, unless needed to validate the documentation sync itself.
- New policy design for future mechanics.

## Implementation Plan

1. Inventory current authoritative runtime/data/DTO changes.
   - Read code paths for lifecycle, battlefield layout, redeploy, inactive projectile/effect resolution, event log, checkpoint DTO, server message mapping, and snapshot/error mapping.
   - Read live RON/data only where schema/content meaning may have changed.

2. Inventory existing documentation.
   - Internal: `docs/game_rulebook.md`, `docs/skill_target_contract.md`, `docs/refactor_preparation_plan.md`, `docs/README.md`, relevant goal review reports.
   - External: `unity_core_contract.md`, `core_unity_battle_transport_contract.md`, `unity_client_implementation_goal.md`.
   - Identify stale, missing, contradictory, or incomplete statements.

3. Build a sync checklist.
   - For every changed runtime/contract item, record:
     - authoritative code/data evidence,
     - document section that should describe it,
     - current doc status,
     - required edit,
     - validation/probe evidence needed.

4. Update internal docs.
   - Keep internal docs focused on gameplay rules, source-of-truth direction, and local document map.
   - Do not duplicate full Unity transport contracts locally.

5. Update external Unity canonical docs.
   - Update `unity_core_contract.md` for WebSocket/DTO/event payload shape and enum casing where needed.
   - Update `core_unity_battle_transport_contract.md` for battle setup/update/checkpoint/resync/event-log changes.
   - Update `unity_client_implementation_goal.md` if Unity implementation order or consumer responsibilities changed.
   - Do not create or rely on local stale copies.

6. Update or add validation.
   - Prefer existing shape/server mapping tests where possible.
   - Add focused tests only if a documented contract lacks runtime/DTO coverage.
   - Run appropriate checks from `docs/code_documentation_sync_guidelines.md`.
   - If WebSocket/admin shape changed and a probe exists, run the relevant probe.

7. Final consistency audit.
   - Re-read changed docs and compare them against runtime code, live data, DTO tests, and probes.
   - Search for old event shapes, old lifecycle wording, old checkpoint assumptions, old battlefield position ownership, stale Unity local copies, and contradicted legacy text.
   - Record final audit results in this goal.

## Policy Decision Completion Condition

If synchronization reveals that actual runtime code, live data, Unity-facing DTO shape, event-log schema, command behavior, UX meaning, balance, save migration, or source-of-truth ownership is ambiguous or not covered by existing policy, do not invent a new policy silently.

Complete this goal with a `사용자와 정책 논의 필요` report containing:

- the document/runtime conflict,
- affected files/contracts,
- concrete options,
- recommendation,
- code/data evidence.

For this documentation-sync goal, "complete with policy-decision report" is a valid completed outcome. Do not leave the goal paused or blocked merely because a policy decision is required.

## Completion Conditions

- Every in-scope recent core/server change has an entry in the sync checklist.
- Internal durable docs describe the current gameplay/source-of-truth policy without relying only on goal docs.
- External Unity canonical docs are updated for every Unity-facing DTO/event/transport change.
- No local stale Unity contract copy is introduced or used as canonical.
- Existing contradictory legacy wording is removed or explicitly marked superseded where historical context must remain.
- JSON/event field names, enum casing, nullability, source of truth, and consumer responsibilities in docs match actual DTO/event code.
- Focused tests/probes that prove the documented DTO/server/WebSocket contracts are run or explicitly recorded as unavailable with reason.
- Live RON loading/data validation is run if live data/schema meaning is touched.
- Broad validation appropriate to the final blast radius is run.
- `EXPERIMENTS.md` records commands, failures, fixes, and reruns.
- Final report lists:
  - changed docs,
  - contract sections synchronized,
  - removed stale/legacy wording,
  - tests/probes run,
  - unresolved follow-up risks.

## Recommended Validation Commands

Adjust based on the actual changed document/contract surface:

```text
cargo check
cargo test -- --test-threads=1
cargo test -p game_server player_game_actor -- --nocapture
cargo test --test ron_loading -- --nocapture
```

If WebSocket/admin contract docs changed and the relevant probe is available:

```text
APP__SERVER__BIND_ADDRESS=127.0.0.1 APP__SERVER__PORT=18083 ENABLE_ADMIN_COMMANDS=true ADMIN_COMMAND_TOKEN=dev cargo run -p game_server
ADMIN_COMMAND_TOKEN=dev WS_PORT=18083 python3 "<available-probe-path>"
```

## Current Status

Status: completed.

Completed on 2026-06-23.

Final sync summary:

- Created `SYNC_CHECKLIST.md` with runtime evidence, document status, and validation coverage for lifecycle, redeploy, withdrawal cleanup, projectile miss/no-hit, battle transport, snapshot/error contracts, and doc hygiene.
- Updated internal durable docs:
  - `docs/game_rulebook.md`
  - `docs/skill_target_contract.md`
- Updated external Unity canonical docs in place:
  - `/mnt/f/unity projects/ark/docs/unity_core_contract.md`
  - `/mnt/f/unity projects/ark/docs/core_unity_battle_transport_contract.md`
  - `/mnt/f/unity projects/ark/docs/unity_client_implementation_goal.md`
- Removed stale current-contract wording for `setup: null`, `request_battle_resync { known_seq, need_setup }`, and ambiguous "alive checkpoint unit" language from synced docs.
- Documented current `battle_response: "battle_resync"` + `request_battle_state { since_seq }` catch-up flow and `recover_battle_setup_loss` setup-loss flow.
- Documented Active-only checkpoint units, retained inactive debug/admin/replay entities, withdraw/death event positions, redeploy new instance/HP policy, `SkillCastCancelled`, withdrawn buff expiry reasons, and projectile miss/no-hit events.
- Aligned one user-facing server error string and related test names from `known_seq` to `since_seq`.

Validation run:

- `cargo test -p game_server player_game_actor -- --nocapture`
- `cargo test -p game_core withdraw -- --nocapture`
- `cargo test -p game_core live_defense_battle_state_rejects_future_since_seq -- --nocapture`
- `cargo check`
- `cargo test -- --test-threads=1`

Remaining risk:

- This pass focused on the large battle/runtime/server-contract drift identified in scope. External docs still contain broad future-facing Unity implementation details; future gameplay feature changes should repeat this checklist instead of relying on this goal as a permanent source of truth.
