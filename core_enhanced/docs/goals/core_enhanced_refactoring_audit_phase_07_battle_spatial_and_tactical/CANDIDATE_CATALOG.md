# Core Enhanced Refactoring Audit Phase 07 — Candidate Catalog

Catalog Date: 2026-08-07
Status: complete; dispositions accepted by phase-exit audit `Complete / High`

Priority axes are ordered as: ownership duplication, correctness/atomicity, lifecycle complexity, change
amplification, test fragility, failure blast radius, change frequency. Current accumulated dirty work makes Git
history non-representative, so change frequency is `unknown` and no automatic total-band verdict is used.

## Confirmed Candidates

### CRA-P07-C01 — Derive Tactical Tile Effects Once From Full Final State

- Work Type: `correctness_correction`
- Problem Evidence: relocation changes Focus/block and schedules Rest from old/new tile kind without checking
  `FieldWorking`. A Ground worker has already removed Focus, so Ground→Platform subtracts it again; a Platform worker
  moving to Ground adds Focus before field-work exit and exit adds it again. A worker that remains assigned on a
  cross-kind work cell can expose a Rest deadline despite `FieldWorking`. Swap reuses the same helper. Existing tests
  cover Ground-only worker movement and do not assert aggregate modifier identity.
- Current Authority And Consumers: stance change, relocation and field-work transition each mutate unit Focus/block/
  Rest separately, while `unit_tactical_snapshot` independently derives the intended effect from activity, stance and
  tile. Damage intake, block planning, Rest events and tactical DTO consume the result.
- Target Authority: one typed tactical-effect transition/reconciliation function compares old and final
  `(activity, stance, tile, profile)` state and applies exactly one Focus delta, exact base/bonus block and one valid
  Rest epoch/deadline. Stance, relocation, swap and field-work lifecycle call it only after their logical state is
  complete.
- Preserved Invariants: Attack has no tile effect; FieldWorking has no Focus/block/Rest; Ready+Defense Ground uses
  authored base+bonus and Focus once; Ready+Defense Platform has zero block and one full-interval Rest; same-tile
  facing keeps cadence; relocation/swap remain allowed while working; unrelated damage modifiers remain exact.
- Removal Or Consolidation Scope: remove manual Focus arithmetic/block/Rest scheduling from three independent
  mutation paths. Do not change damage formula semantics, profile values, resource costs, cooldowns, field-work
  progress or target/delivery behavior.
- Priority Axes And Total: `3 / 2 / 3 / 3 / 3 / 2 / unknown`; known subtotal `16/18`, no automatic band. Current live
  RON has no Any weapon, but the typed validator and canonical already approve the capability.
- Validation Safety Net: first add Any-profile Ground↔Platform relocation and swap cases for workers that exit work
  and workers that remain assigned. Seed a distinct unrelated incoming modifier and assert exact preservation,
  Focus once/zero, block, activity, Rest epoch/deadline, stale event no-op and one final snapshot. Retain current
  Ground-only progress and Ready relocation/swap tests.
- Policy/External Contract Decision: none; this restores the existing canonical contract. Phase 8 should confirm
  only that the unrelated aggregate modifier assertion uses the established damage-modifier representation.
- Readiness: `test_first_then_implementation`.
- Disposition: `priority_candidate`.

### CRA-P07-C02 — Preflight And Commit One Atomic Tactical World Mutation

- Work Type: `correctness_correction`
- Problem Evidence: deploy, withdraw, stance, relocation and swap mutate Core state and commit revision before
  `battle_update_dto_after` can reject a future cursor. Deploy/withdraw additionally perform fallible employee
  reconciliation after Core mutation. The active-battle canonical promises zero state/resource/cooldown/event/
  revision change on failure; current future-cursor coverage is read-only and ordinary atomicity tests stop at
  earlier domain validation.
- Current Authority And Consumers: five world handlers independently sequence Core command, deployment mapping,
  resource/signal/employee state, revision and DTO construction. Core transport/server callers observe the returned
  error and later checkpoint.
- Target Authority: a common tactical-mutation plan/commit boundary validates battle authority, revision capacity,
  event cursor and all domain-specific fallible reconciliation inputs before mutation, then applies the Core command,
  resource/mapping changes and exactly one projection commit as an infallible publish step. If a truly fallible Core
  operation remains, stage the affected active-session state rather than relying on partial rollback.
- Preserved Invariants: exact command-specific disabled reasons, source-command correlation, one cost where required,
  one revision on success, Pause/frozen time, deploy salt/redeploy HP policy, field-work reconciliation, live signals,
  employee incapacitation and current response DTO.
- Removal Or Consolidation Scope: remove validation after authoritative commit and five repeated mutate→commit→DTO
  tails. Do not batch separate client commands, roll back an earlier successful request, merge Phase 6 advance or
  Phase 9 skill semantics into one broad transaction.
- Priority Axes And Total: `3 / 3 / 3 / 3 / 3 / 3 / unknown`; known subtotal `18/18`, no automatic band.
- Validation Safety Net: for every command, submit a future cursor after otherwise-valid preflight and compare exact
  Core units/scenario/log/signals, deployment maps/salt/locks/facing, resource, employee attempt, focus, revision and
  response state. Inject the rare reconciliation/Core-deploy failure seam and assert the same. Retain stale battle
  UUID/revision, overflow, Pause, cost and domain rejection tests.
- Policy/External Contract Decision: none for canonical failure atomicity. Final should coordinate a shared early
  cursor validator/transaction primitive with Phase 6 C03 and any Phase 9 mutation finding; server ownership remains
  Phase 10.
- Readiness: `test_first_then_staged_commit`; coordinate design with Phase 6 C03 before implementation.
- Disposition: `priority_candidate`.

### CRA-P07-C03 — Make Rapier The Single Private Production Movement Backend

- Work Type: `cleanup`
- Problem Evidence: `BattleCore` defaults to Rapier and the Direct selector is `#[cfg(test)]`; all Direct uses are
  tests. The Direct swept-AABB/depenetration backend has nine backend-only tests, four parity tests and three Core
  tests, while the authored-route integration test explicitly bypasses production Rapier. Rapier separately keeps a
  test-only board-wall collider path although production clamps bounds without walls. Many public backend inspection
  methods have only internal/test callers.
- Current Authority And Consumers: a public backend enum/trait exposes Direct and Rapier as peers; tests validate
  both plus test-only Rapier wall state. Production BattleCore and world battle consume only Rapier with a shared
  manual board clamp.
- Target Authority: one private Rapier-backed BattleCore movement implementation. Keep a neutral pure board clamp and
  the actual static-obstacle policy, migrate unique behavioral assertions and the authored-route integration test to
  default production Core, then remove Direct, backend selection and test-only wall/introspection surface not needed
  by production behavior.
- Preserved Invariants: deterministic unit order, ground obstacle stop/no-slide, airborne obstacle bypass, manual
  board clamp, intentional unit overlap, block locks, body endpoints, immutable movement segments and authored-route
  progress.
- Removal Or Consolidation Scope: remove alternate Direct physics, parity-as-authority tests, Core backend switches,
  test-only Rapier board walls and unused public instrumentation after assertion transfer. Do not replace Rapier,
  enable dynamic backend choice or change collision/gameplay policy.
- Priority Axes And Total: `3 / 1 / 1 / 3 / 3 / 2 / unknown`; known subtotal `13/18`, no automatic band.
- Validation Safety Net: inventory each of the 17 Direct-dependent tests, transfer unique behavior to Rapier/default
  Core, retain Rapier sync/static-obstacle/overlap cases that observe behavior, and run the authored-route test without
  a selector. Compare deterministic movement logs/results across repeated default runs.
- Policy/External Contract Decision: workspace production has no alternate consumer, but movement modules/types are
  public. Unknown external Rust callers require inventory or an approved breaking/deprecation decision before API
  removal.
- Readiness: `assertion_transfer_and_external_consumer_inventory`.
- Disposition: `candidate`.

### CRA-P07-C04 — Remove Ghost State From The Movement Tick Model

- Work Type: `cleanup`
- Problem Evidence: `MovementTickInput.now_ms`, `MovementUnitInput.owner/current_target`, `UnitBody.mode` and
  `physics_handle` have no engine/runtime reader; `GoalAssigned` and `SegmentStarted` outputs are never produced;
  movement applies no semantic use of stopped position/reason; stored body goal is overwritten from the current
  planner map each tick; previous position and velocity are written but have no production reader. These fields and
  enums make a second-looking motion lifecycle that does not exist.
- Current Authority And Consumers: public/serde movement structs carry placeholder state, tests construct it, and
  Core clones whole bodies into the backend. Actual authority is Core body position/radius/speed, current planner
  goal, terrain/locks/lifecycle and active movement segment.
- Target Authority: a minimal internal movement input view with explicit current goal and only values consumed by
  Rapier/steering; outputs contain only transitions BattleCore applies. RuntimeUnit retains the authoritative body
  facts actually used by spatial/range consumers. Public compatibility wrappers remain only if a confirmed external
  consumer needs them.
- Preserved Invariants: world coordinates/radii/speed, planner decisions, stopped/blocked behavior, opponent facing,
  target reach, body endpoint, event segment fields/order, relocation clearing and all serialized Unity DTOs.
- Removal Or Consolidation Scope: remove unread fields, never-produced variants and clone/write churn after C03
  establishes one backend. Do not remove `ActiveMovementSegment`, quantization, action locks, move epoch or approved
  event-log stop reasons from their separate authoritative lifecycle.
- Priority Axes And Total: `2 / 0 / 2 / 2 / 2 / 1 / unknown`; known subtotal `9/18`, no automatic band.
- Validation Safety Net: compile/search every field and variant, retain world position/range conversion, Rapier
  obstacle/route/block tests, immutable segment DTO tests and relocation state tests. Add before/after golden default-
  backend logs/checkpoints for moving, stopped and blocked units.
- Policy/External Contract Decision: active battle is not durable save state and these structs are not Unity DTOs,
  but public Rust/serde consumers are unverified. Inventory or approve API removal before implementation.
- Readiness: `after_C03_and_external_consumer_inventory`.
- Disposition: `cleanup_candidate`.

## Required Ordering

1. C01 begins with failing full-state field-work transition tests and can proceed independently.
2. C02 design coordinates with Phase 6 C03 before either creates a competing transaction helper.
3. C03 transfers behavior assertions to the production backend before deleting alternate paths.
4. C04 follows C03 so the model is reduced against one actual backend and one surviving test authority.

## Rejected, Deferred And Unverified Signals

| Signal | Disposition | Evidence/reason |
| --- | --- | --- |
| Battlefield valid/static topology | `retain` | one layout, deterministic output and placement/physics agreement |
| Runtime body vs active movement segment | `retain` | endpoint authority versus interpolation cache is explicit canonical behavior |
| Player occupancy and enemy overlap | `retain` | one derived current-body mapping; approved side/role semantics |
| Deterministic sticky block planner | `retain` | explicit route/spawn/ID order and distinct behavior tests |
| Tactical resource state | `retain` | one clamped pool, Pause-safe battle-time recovery and direct overflow tests |
| Field-work progress/generation | `retain` | deterministic BTree runtime, settle-before-change and stale generation guards |
| Deployment-zone cell in multiple zones | `cross_phase_signal` | current template generator is disjoint; add strict topology case to Phase 5 C02 rather than duplicate it |
| Core deploy partial failure for invalid TestFixture/reused salt | `unverified_hardening` | static partial path exists, but current world/live data precludes it; test under C02 and inventory direct Rust callers |
| Rapier missing-handle fallback to desired translation | `observe` | prepare-tick synchronizes all live units and internals are not externally mutable through BattleCore; no failure reachability proved |
| Rest/field-work zero interval loop | `rejected_current_candidate` | published typed validators reject zero Rest interval and required work |
| Epoch/generation integer wrap | `rejected_current_candidate` | bounded active battle lifetime; no credible wrap reachability |
| Effect/damage modifier interpretation | `deferred_phase_08` | C01 owns transition placement only; formula semantics remain Phase 8 |
| Target/range/projectile/area semantics | `deferred_phase_09` | spatial carrier read only; selection/delivery not verified here |
| DTO/server command consolidation | `deferred_phase_10` | cursor evidence supports C02, but transport/refactor ownership remains Phase 10 |
| Unity/external Rust consumers | `unverified_out_of_scope` | Unity not inspected; public movement/API cleanup is compatibility-gated |
