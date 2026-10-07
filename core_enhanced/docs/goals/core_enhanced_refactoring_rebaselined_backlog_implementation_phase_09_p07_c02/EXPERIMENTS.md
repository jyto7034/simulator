# Phase 09 CRA-P07-C02 Experiments

Status: complete
Implementation Complete: true

## 2026-08-10 — E00 Activation And Required-Document Initialization

### Authorization And Dependency

- The user's Master execution request remains active and authorizes the single active Child.
- Phase 08 CRA-P07-C01 has a current `Complete / High` audit with improvement class `none`.
- Phase 09 is next in the exact registry; Phases 10~44 remain pending and unauthorized.
- No candidate-specific policy ID applies. CRA-POL-006 freezes existing command/result/checkpoint/DTO/wire/public
  shape while the existing failure-atomicity contract is restored.

### Evidence Carried Into Initialization

- CRA-P07-C02 remains a correctness correction with `3/3/3/3/3/3/unknown = 18/18`.
- Historical defect: five tactical world handlers can mutate/commit before final update cursor validation; deploy/
  withdraw also contained later fallible employee reconciliation.
- Required first proof: otherwise-valid future cursor for deploy, withdraw, stance, relocation and swap must compare
  exact complete world state, plus one actual late Core-deploy/reconciliation failure.
- Preservation boundary: exact reason priority, one success cost/revision/event, Pause, deploy/withdraw lifecycle,
  Phase 08 effects, live data and all public/serialized shapes.

### Initialization Result

- Required Child documents initialized: `PLAN.md`, `EXPERIMENTS.md`, `EXPERIMENT_NOTES.md`.
- Master Phase 09 coordination changes from `ready` to `active`; no other Child is active.
- Phase 09 production status/source/data investigation: not started.
- Test executions: none.
- Production/test/data/canonical mutation before initialization: none.

### Next Experiment

Read all three Child documents completely, capture expanded dirty status and prospective target hashes, then trace
the five handlers, complete state footprint, fallible ordering and current transaction helpers before baselines.

## 2026-08-10 — E01 Live Path, Dirty Baseline And Nonzero Baselines

### Dirty Preservation Baseline

- HEAD: `f318414fb0156a67e04c5479000c2a2683fc0c09`.
- Expanded status: 412 entries, SHA-256
  `b1f434f919a9b116a2ca7921db6bd84abdf792989e625894ae71dd7a9335fcf4`.
- All four eventual targets were already modified user-owned paths:
  - `world.rs`: `28589fc4206d341260473406d51a70672fb0b123755395f62a182dd5474ff537`
  - `world/combat.rs`: `aa08c5f088350915b8adcf87ffe349d26030a613af43763dd5ebd90c3a0e6f5e`
  - `world/tests/combat.rs`: `9bebdf426aa9a3e56ea6413d0806ea6a4a9366874f616e995a1470abd921574c`
  - `core_runtime_contract.md`: `eb5fbf7dca54b9720ccaab9bc20f1c8f4557aa8f6e057b7a2ef220586233bcb6`

### Complete Live Path Inventory

- `execute_with_transport_context` validates exact battle UUID/revision and action capability before domain entry,
  then normally performs a second fallible visual-novel/action-capability response projection after the handler.
- Deploy mutates Core scenario/units/items/log/queue/signals, instance salt, resource, locks/mapping and employee
  reconciliation; withdraw mutates lifecycle/action/target/buff state, log, mapping/lock and reconciliation.
- Stance, relocation and swap mutate Core tactical/body/block/field-work-derived state, event/log/queue, resource or
  facing as applicable. All five then increment battle revision and call cursor-based DTO projection.
- The five handlers each committed their live `ActiveBattleSession` before `battle_update_dto_after`; deploy and
  withdraw also called fallible `reconcile_employee_attempt` after Core mutation.
- The existing `with_state_transaction` intentionally rejects active battles. Battle advance has a broader staged
  transaction, but its clock/trigger/terminal semantics are not an authority for separate tactical commands.

### Nonzero Baselines

| Exact family | Result |
| --- | --- |
| `live_defense_` | 9 passed, 0 failed |
| `live_tactical_` | 2 passed, 0 failed |
| paused deploy/withdraw exact filter | 1 passed, 0 failed |

### Next Experiment

Add five otherwise-valid future-cursor whole-session comparisons and one real late employee-reconciliation failure
at the withdraw seam, then run them against the pre-fix implementation.

## 2026-08-10 — E02 Test-First Failure Proof

### Future Cursor Matrix

Added independent deploy, withdraw, stance, relocation and swap cases. Each prepares a valid command, submits
`after_seq = u64::MAX`, requires `InvalidBattleResyncSeq`, and compares the complete affected active-battle/world
snapshot: Core transaction state, queue/log, execution, employee attempt, deployment, resource, focus/playback,
revision and adjacent world-visible state.

Pre-fix result: `0 passed, 5 failed`. Every command returned the cursor error only after its Core/world state had
changed, so each exact before/after comparison failed.

### Real Late Failure

Two employees are deployed normally. The second unit is then made an unprocessed dead deployment whose attempt
roster entry is absent; withdrawing the first unit reaches the production `reconcile_employee_attempt` seam only
after Core withdrawal and deployment-lock mutation. Pre-fix result: `0 passed, 1 failed`; the handler returned
`UnitNotFound` while the first unit, event log and deployment state remained mutated.

An earlier fixture that failed before mutation was discarded and is not counted as evidence.

### Next Experiment

Stage one complete active battle per tactical request, preserve command-specific validation order, build the final
transport projection inside the boundary and publish only after success.

## 2026-08-10 — E03 One Atomic Tactical World Mutation Boundary

### Production Change

- Added private `with_staged_tactical_world_mutation`, called by exactly the five scoped handlers.
- The boundary clones the complete `ActiveBattleSession`, checks revision capacity before the first staged mutation,
  runs the existing command-specific Core/world/reconciliation order on the stage, increments revision once and
  constructs the cursor-based battle update from the stage.
- It then constructs the exact command result, temporarily publishes the completed stage, and performs the final
  visual-novel snapshot/refreshed action-capability projection. A projection error restores the original session;
  otherwise the original is dropped and the staged session is the single authoritative publish.
- `execute_with_transport_context` recognizes only these five command variants and skips its ordinary duplicate
  response attachment because the shared boundary already completed it.
- No error reason ordering, costs, cooldowns, Pause semantics, event correlation or DTO/result nesting changed.

### Exact Green Proof

| Phase-specific proof | Result |
| --- | --- |
| five future-cursor whole-state cases | 5 passed, 0 failed |
| actual late employee-reconciliation case | 1 passed, 0 failed |
| tactical revision-overflow before-mutation case | 1 passed, 0 failed |

The revision-overflow case sets the current revision to `u64::MAX`, submits an otherwise-valid deploy through the
normal authority gate, receives `BattleStateRevisionOverflow`, and proves exact state equality.

### Compatibility And Canonical

- `core_runtime_contract.md` now assigns these five commands to one staged world transaction through final transport
  projection, with exact failure rollback and one-success publish semantics.
- No command/result/event/checkpoint, RON, wire/public Rust, server, Unity or save shape changed.

### Next Experiment

Re-run owning success/Pause/authority suites, full Core/live RON and server compile; inventory all boundary callers
and post-commit fallibility; then capture final hashes.

## 2026-08-10 — E04 Broad Validation, Static Closure And Dirty Preservation

### Focused And Broad Validation

| Check | Result |
| --- | --- |
| phase-specific tactical world atomicity | 7 passed, 0 failed |
| `live_defense_` | 9 passed, 0 failed |
| `live_tactical_` | 2 passed, 0 failed |
| paused deploy/withdraw | 1 passed, 0 failed |
| stale battle authority | 1 passed, 0 failed |
| complete Core library | 955 passed, 0 failed, 0 ignored, 0 filtered |
| live item/catalog/RON/skill integration | 3/3, 4/4, 25/25, 13/13 and 4/4 |
| game_server `cargo check --all-targets` | pass |
| `cargo fmt --all -- --check` | pass |
| scoped `git diff --check` | pass |

### Static Closure

- Exactly five callers exist at `handle_deploy_unit`, `handle_withdraw_unit`,
  `handle_set_engagement_stance`, `handle_tactical_relocate_unit` and `handle_tactical_swap_units`.
- Revision increment, cursor DTO and final fallible visual-novel/action-capability projection are centralized inside
  the boundary. No scoped handler has a fallible post-publish tail.
- Exact UUID/revision authority and capability gates remain before domain entry; command-specific validation stays
  in its established relative order inside each handler/stage.
- Separate advance, playback and manual skill/cast commands remain under their existing owners.

### Dirty Preservation

- Expanded status remains exactly 412 entries with activation SHA-256
  `b1f434f919a9b116a2ca7921db6bd84abdf792989e625894ae71dd7a9335fcf4`; no path or tracked/untracked
  classification changed.
- Final hashes:
  - `world.rs`: `8965b7cf85f8729e1c0b442c424d4e5d12f486249b1032af0cf9f6ccd9594f01`
  - `world/combat.rs`: `0f9d6af051045f4c11646c373ed9c5706796dbf5a30bdc843f1a3f31ca3ec292`
  - `world/tests/combat.rs`: `2e0dbdc4a5fa4e840599d29581fa1875e6bb540fe936ca49e4f759e309a62328`
  - `core_runtime_contract.md`: `efe701dc50a5cd12d611819af173bb7cdf68ba480233bfe46fca5155ccad3efb`
- No reset, restore, clean, stash, deletion, move, broad formatting or live-data mutation was performed.

### Next Experiment

Re-read all 567 lines of the completion-review guide, independently audit runtime/data/DTO/persistence/consumer,
legacy/static ownership, canonical and dirty preservation, and write `COMPLETION_AUDIT.md`. Do not activate Phase
10 unless the verdict is `Complete / High`.

## 2026-08-10 — E05 Audit Evidence Tightening

- The independent audit found no runtime defect, but final transport attachment was protected only indirectly by
  broad success behavior. Existing deploy, withdraw, stance, relocation and swap success tests now each assert that
  their returned `battle_update.action_capabilities` is non-empty.
- This directly proves that the staged boundary completed its fallible final projection and that the outer transport
  skip did not return the pre-attachment base DTO.
- Revalidation after the test-only tightening: `live_defense_` 9/9, `live_tactical_` 2/2, complete Core 955/955,
  live item/catalog/RON/skill integration 3/3, 4/4, 25/25, 13/13 and 4/4; formatting and diff checks pass.
- Expanded status remains 412 entries with the same activation SHA-256. Production and canonical hashes are
  unchanged; final test hash is
  `2e0dbdc4a5fa4e840599d29581fa1875e6bb540fe936ca49e4f759e309a62328`.

### Next Experiment

Write the independent completion audit from the refreshed current-tree evidence. Do not activate Phase 10 unless
the primary verdict is `Complete` and long-term fit is `High`.

## 2026-08-10 — E06 Independent Phase Exit Audit

- Re-read all 567 lines of `goal_completion_review_guide.md` before the independent pass.
- Rechecked current authority entry, all five staged callers, revision/cursor/final projection, whole-state tests,
  live data, unchanged DTO/persistence, game_server mapping, canonical, legacy scope and dirty preservation.
- Audit evidence tightening added direct non-empty final action-capability assertions to each scoped success owner;
  the refreshed focused and complete Core suites passed.
- Creating `COMPLETION_AUDIT.md` moved expanded status from 412 to 413 entries; audit-final SHA-256 is
  `1b5ab02cea390e1b63f10ab10c558fdb5dd4b58d0cce8e2352d1c9a92aefeae8`.
- `COMPLETION_AUDIT.md` records all matrix rows pass, primary verdict `Complete`, fit `High` and improvement class
  `none`.
- Phase 09 status changes to `complete`; Master may make Phase 10 ready.
