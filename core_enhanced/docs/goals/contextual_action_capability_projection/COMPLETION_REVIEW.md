# Contextual Action Capability Projection Completion Review

Reviewed: 2026-07-13

Primary verdict: Complete

Long-term fit: High

Improvement class: none

## Review Summary

| Item | Findings | Verdict | Long-term fit | Immediate correction |
| --- | --- | --- | --- | --- |
| Core-owned contextual capability projection | DCA-M-13 | Complete | High | None |
| Checkpoint availability de-duplication | DCA-M-18 | Complete | High | None |
| Shared typed preflight seam | DCA-M-19 shared portion | Complete | High | None |
| Rust/server/Unity root cutover | DCA-M-13, DCA-M-18 | Complete | High | None |

## Core-Owned Contextual Capability Projection

Master item: G06 `contextual_action_capability_projection`

Subgoal: `docs/goals/contextual_action_capability_projection/`

Policy source: accepted PD-02 in the Master Goal.

Expected behavior:

Core emits typed context capabilities with gameplay/query/recovery scope. Context-irrelevant
actions are absent, relevant blocked actions carry a stable typed reason, and execution
revalidates against current authoritative state rather than trusting a snapshot.

Runtime evidence:

- `src/game/world/action_capability.rs` owns tri-state common projection and common command
  preflight.
- `src/game/world.rs` invokes current-state preflight before mutation.
- `src/game/behavior.rs` owns `ActionKind`, `ActionScope`, typed disabled reasons, and the exact
  root DTO.
- Legacy `src/game/managers/action_scheduler.rs`, `src/game/resources/action.rs`, cached
  `ActionValidator`, refresh paths, and old query methods were removed.

Data evidence:

- Capability state is derived runtime projection. Production loaders and live RON contain no
  capability list or validator cache; no live-data or save migration is required.

External contract evidence:

- game_server transparently serializes the typed root and preserves exact disabled reason codes.
- Unity `ActionCapability.cs`, `GameSnapshotStore.cs`, `GameCommandBus.cs`, and common composers
  consume the typed contract without an old-field fallback.
- `game_server/examples/generate_action_capability_fixture.rs` produces the exact fixture consumed
  by Unity at `Assets/Tests/Fixtures/action_capability_contract.json`.
- Canonical Core/server/Unity and runnable WebSocket probe contracts describe
  `action_capabilities`; checkpoint capacity facts no longer include `can_load`.

Test evidence:

- Core all targets: 669 nonzero tests passed (library 628; integration suites 3, 3, 18, 13, 4).
- game_server all targets: 26 nonzero tests passed.
- Unity MCP EditMode: 186/186 passed.
- Unity MCP PlayMode: 53/53 passed.
- Exact Rust fixture comparison, Python probe AST parsing, `cargo fmt --all -- --check`, scoped
  diff checks, and Unity Console error audit all passed.

Legacy/fallback audit:

- Production Core/server searches find no `allowed_actions`, scheduler, validator cache, old query,
  or old admin name.
- Unity production has no old root field/API/admin name. Its static battle-only action set remains
  deliberately bounded to G07 and is not used for common/root commands.
- Two Unity negative assertions explicitly prove the removed `allowed_actions` and checkpoint
  `can_load` fields are absent; they are contract guards, not compatibility readers.

Long-term direction review:

- Fit: High
- Improvement class: none
- Reason: the implementation removes duplicate state and state-table authority, makes the Core
  projection typed, preserves atomic domain validation, and gives later domains a bounded extension
  seam without a generic payload bag or dual schema.

Verdict:

- Complete

Remaining risk:

- Battle capability, expected revision, reconnect recovery, and Unity's battle-only static set are
  intentionally G07.
- Event, inventory, Safe Zone, Maintenance, and Shop payload eligibility remain with their named
  child goals. They are not alternate common/root authorities.

## Checkpoint Availability De-Duplication

Expected behavior:

Checkpoint DTO exposes factual capacity only. Current load executability is represented by the
recovery-scoped `LoadRunCheckpoint` capability and is checked again on execution.

Evidence:

- `RunCheckpointSnapshotDto` contains `exists`, `loads_used`, `max_loads`, and `remaining_loads`, but
  no `can_load` field.
- `RunCheckpointState::can_load()` remains the single domain predicate used by projection and load
  preflight.
- Core tests cover enabled, disabled, reason parity, and state change before execution; Unity's real
  Rust fixture asserts the removed field remains absent.

Verdict: Complete / High / no immediate correction.

## Shared Typed Preflight Seam

Expected behavior:

G06 supplies stable common capability types and a reusable pure preflight seam without claiming
ownership of every domain's concrete payload rules.

Evidence:

- Common map/checkpoint/start/support/reward/non-battle-VN paths compose projection and preflight.
- Selected Shop payload errors continue to come from the Shop handler until G28; battle VN bypasses
  common preflight until G07.
- Tests prove same-state projection/execution parity, typed rejection, blocking-VN mutual exclusion,
  and current-state revalidation.

Verdict: Complete for the DCA-M-19 shared portion / High / no immediate correction.

## Cross-Component Findings

- Rust, game_server, Unity, and runnable probe contracts agree on the typed root shape.
- No live RON or save schema carries derived capability authority.
- The only remaining local action substitution is battle-scoped and explicitly handed to G07.
- No competing common/root authority, compatibility alias, ignored legacy test, or fallback reader
  was found.

## Follow-Up Refactor Candidates

- None inside G06. Execute the already planned domain owners G07, G03, G21, G24, G28, and deferred
  G22 rather than expanding this completed common boundary.

## Policy Questions

- None.
