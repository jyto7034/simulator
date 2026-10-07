# Core Enhanced Refactoring Audit Phase 06 — Candidate Catalog

Catalog Date: 2026-08-07
Status: complete; dispositions accepted by phase-exit audit `Complete / High`

Priority axes are ordered as: ownership duplication, correctness/atomicity, lifecycle complexity, change
amplification, test fragility, failure blast radius, change frequency. Current accumulated dirty work makes Git
history non-representative, so change frequency is `unknown` and no automatic total-band verdict is used.

## Confirmed Candidates

### CRA-P06-C01 — Make The Execution State Own One Terminal Max-Time Outcome

- Work Type: `correctness_correction`
- Problem Evidence: direct stepping moves the stored clock to the next event and then returns Draw at the configured
  cap, whereas elapsed-time stepping caps its target and returns `Running` when the next event is later. At the cap,
  later production ticks repeat that branch forever. The state stores only `finished/finalized` booleans and
  finalization trusts a caller-supplied signal. Current max-time coverage runs only `run_battle` and does not assert
  the execution clock or live world transition.
- Current Authority And Consumers: `BattleExecutionState`, `BattleCore.event_queue`, three step APIs and the caller-
  held `BattleFinishSignal`; world server tick and result transition consume them.
- Target Authority: one state-owned terminal outcome and one cap rule shared by direct/until/by stepping. Clamp the
  authoritative clock, decide a winner already satisfied at/before the cap or the established Draw fallback, expose
  that outcome once and let finalization consume it exactly once.
- Preserved Invariants: configured max time, Draw when no win condition is satisfied by the cap, same-time bucket
  draining, `BattleEnd` time, deterministic log/result, pause/speed behavior, one final projection and no events
  processed after the cap.
- Removal Or Consolidation Scope: remove divergent cap branches, clock beyond cap, replaceable caller finish
  authority and repeated `Running` commits. Do not change movement/cast/effect formulas or live cap tuning.
- Priority Axes And Total: `2 / 3 / 3 / 3 / 3 / 3 / unknown`; known subtotal `17/18`, no automatic band.
- Validation Safety Net: first add direct/by/until parity with an event strictly after the cap, exact-cap satisfied
  and unsatisfied win conditions, clamped clock, one `BattleEnd`, second-finalize rejection and a production world
  tick that reaches `CombatResult` without further revision churn.
- Policy/External Contract Decision: no new balance policy is needed. Exact public step API compatibility and any
  external consumer holding `BattleFinishSignal` must be inventoried before shape changes.
- Readiness: `test_first_then_implementation`.
- Disposition: `priority_candidate`.

### CRA-P06-C02 — Give Every Same-Time Battle Event A Total Deterministic Order

- Work Type: `correctness_correction`
- Problem Evidence: `BattleEvent::Ord` can return `Equal` for events that are not `Eq`. In particular it has no
  `ManualCastEnd` tie-break and treats `AutoCastEnd`/`ManualCastEnd` at the same time as equal. Different units can
  complete casts simultaneously; heap order is not insertion order and one completion may invalidate the other.
  The only ordering test covers selected variants that already have identifier ties.
- Current Authority And Consumers: event variant priority plus partial per-variant identifiers own heap/bucket order;
  `process_event` and append-only log seq consume the resulting order.
- Target Authority: a documented stable semantic sort key that is total over every variant and every field required
  to distinguish live events, with an explicit variant discriminator and stable IDs/generation/cast sequence before
  any final deterministic fallback.
- Preserved Invariants: existing time and priority bands, spawn-before-action, resolve-before-end behavior, stale
  event guards, deterministic IDs/RNG, and event seq as presentation order. Do not choose order from pointer,
  insertion or hash iteration state.
- Removal Or Consolidation Scope: remove `_ => Ordering::Equal` for unequal events and scattered incomplete ties.
  Do not change Phase 7~9 targeting/effect semantics under the name of ordering.
- Priority Axes And Total: `2 / 3 / 3 / 3 / 3 / 3 / unknown`; known subtotal `17/18`, no automatic band.
- Validation Safety Net: table-driven `cmp == Equal iff Eq` cases for every live variant, heap permutation tests,
  simultaneous manual/manual and auto/manual completion integration cases, repeated log/result equality and current
  same-time AttackResolve-before-BattleEnd regression.
- Policy/External Contract Decision: preserve the current priority table. If two same-priority cross-variant events
  lack an inferable order, record and approve that narrow semantic decision rather than using incidental enum order.
- Readiness: `test_first_after_cross_variant_key_review`.
- Disposition: `priority_candidate`.

### CRA-P06-C03 — Stage One Atomic World Battle-Advance Pipeline

- Work Type: `correctness_correction`
- Problem Evidence: mutating advance steps/finalizes the BattleCore before fallible VN trigger evaluation, employee
  reconciliation and DTO cursor validation. `active_battle_advanced_result` commits revision before
  `battle_update_dto_after` rejects a future `after_seq`. The public call can therefore error after authoritative
  mutation. The next-event-bucket helper duplicates the sequence and current future-seq coverage is read-only.
- Current Authority And Consumers: two world advance functions manually orchestrate BattleCore, VN state, employee
  attempt/tactical state, projection revision, DTO and final result transition; server tick consumes the elapsed-
  time path and tests consume the bucket path.
- Target Authority: one staged advance transaction that prevalidates cursor/revision/authority, runs step/finalize,
  trigger and reconciliation on staged active/VN state, constructs the projection, then publishes one revision and
  optional result transition. Both direct-bucket test use and production elapsed-time use call that pipeline.
- Preserved Invariants: raw vs pause-aware step distinction, exactly +1 revision on success, zero mutation on error,
  setup snapshot only on initial advance, same trigger threshold behavior, employee live reconciliation, debug
  trace non-authority and server-owned cursor semantics.
- Removal Or Consolidation Scope: remove duplicated orchestration and validation after commit. Do not absorb Phase 4
  VN publication policy, Phase 7 tactical formulas or Phase 10 DTO/server ownership.
- Priority Axes And Total: `3 / 3 / 3 / 3 / 3 / 3 / unknown`; known subtotal `18/18`, no automatic band.
- Validation Safety Net: mutating future-seq tests before/after events and at finish; injected trigger/reconcile/DTO
  failure tests asserting exact clock/log/queue/employee/VN/revision/state equality; parity between raw-bucket and
  elapsed-time success; retain stale UUID/revision overflow and result-transition tests.
- Policy/External Contract Decision: none for canonical failure-no-mutation. Coordinate invalid-trigger fixtures
  with Phase 4 C01 instead of creating a second publication policy.
- Readiness: `test_first_then_staged_pipeline`; C01 terminal ownership should land first or in the same atomic Goal.
- Disposition: `priority_candidate`.

### CRA-P06-C04 — Remove The Disconnected Default-Incompatible Event-Log Validator

- Work Type: `cleanup`
- Problem Evidence: all workspace references to `EventLogValidator` and related public types are inside the 14-file
  validation module and its six tests. Default contiguous seq expects index 0 while produced logs begin at seq 1;
  `require_movement_path_consistent` is never read. No runtime, publication or integration test validates a real log.
- Current Authority And Consumers: a public but disconnected alternate log-contract API and self-referential tests;
  actual event producers/results do not consume it.
- Target Authority: BattleCore producer invariants and focused tests over real produced logs. Extract any useful
  parent/buff/death/cadence assertions into those tests, then remove or depublish the parallel harness.
- Preserved Invariants: log version, seq/cause relationships, spawn/reference validity, buff/death/attack behavior
  and current production log/result serialization.
- Removal Or Consolidation Scope: remove the 14-file harness, ghost config and six self-tests only after equivalent
  live-producer assertions exist. Do not delete runtime event-log types or result/debug consumers.
- Priority Axes And Total: `2 / 1 / 1 / 2 / 3 / 1 / unknown`; known subtotal `10/18`, no automatic band.
- Validation Safety Net: run the validator against representative current logs to inventory useful checks, add
  direct generated-log tests for retained invariants, then prove zero workspace references and package green.
- Policy/External Contract Decision: the module is public; unknown external Rust consumers are unverified. Inventory
  or approve a breaking removal/deprecation strategy before implementation.
- Readiness: `external_consumer_inventory_then_test_extraction`.
- Disposition: `candidate`.

### CRA-P06-C05 — Use One Saturating MVP Score For Selection And Publication

- Work Type: `correctness_correction`
- Problem Evidence: candidate comparison uses `score_total(...).sum()` after per-metric saturation; published score
  separately loops with `saturating_add`. Typed run-policy validation checks only non-empty lists and accepts i64
  weights, so extreme accepted values can panic/wrap and disagree with the DTO score. Current live weights are safe.
- Current Authority And Consumers: two score functions plus run-policy weights; comparator selects the employee and
  DTO construction reports the second calculation.
- Target Authority: one total, explicitly saturating score evaluation returning total and highlight, reused for
  comparison and publication; optionally validate duplicate/negative weights only if separately approved policy
  requires it.
- Preserved Invariants: current live policy scores/titles/tiebreakers, deterministic employee UUID fallback,
  incapacitated eligibility and aggregation across redeploy instances.
- Removal Or Consolidation Scope: remove `score_total` duplicate arithmetic and overflow-mode divergence. Do not
  rebalance weights or change the meaning of negative/duplicate authored entries without policy approval.
- Priority Axes And Total: `2 / 2 / 1 / 2 / 3 / 2 / unknown`; known subtotal `12/18`, no automatic band.
- Validation Safety Net: extreme positive/negative multi-weight tests in debug and release semantics, comparator/DTO
  score identity, live policy golden scores, deterministic ties and existing redeploy/incapacitation tests.
- Policy/External Contract Decision: arithmetic correction needs no policy. Bounds, uniqueness or sign restrictions
  on RON are separate authoring-policy decisions.
- Readiness: `test_first_then_implementation`.
- Disposition: `candidate`.

### CRA-P06-C06 — Remove The Ghost `ScenarioEvent.once` Input

- Work Type: `cleanup`
- Problem Evidence: no runtime or validator reads `ScenarioEvent.once`; every workspace constructor sets it `true`.
  Scenario events are enqueued once and group spawn runtime already deduplicates by group identity.
- Current Authority And Consumers: public scenario struct callers write a field that has no behavior.
- Target Authority: event enqueue/runtime identity owns one-shot lifecycle; the struct exposes no unused repeat
  affordance.
- Preserved Invariants: all current scenario timing/actions, group dedupe, validation and test scenarios.
- Removal Or Consolidation Scope: remove the field and workspace fixture writes. Do not introduce repeating scenario
  events or alter event ordering.
- Priority Axes And Total: `1 / 0 / 1 / 1 / 1 / 0 / unknown`; known subtotal `4/18`, no automatic band.
- Validation Safety Net: compile all constructors and retain delayed-spawn/forced-end/scenario-validation tests.
- Policy/External Contract Decision: public Rust consumer compatibility is unverified; repeat-event support would be
  a feature migration, not a reason to retain ghost state.
- Readiness: `consumer_inventory_then_mechanical_cleanup`.
- Disposition: `cleanup_candidate`.

### CRA-P06-C07 — Keep Battle-Log File I/O At Explicit Debug Boundaries

- Work Type: `cleanup`
- Problem Evidence: `BattleEventLog::{write_json,read_json}` have no workspace callers; pretty serialization exists
  only for test/debug file output. The authoritative world trace already owns explicit best-effort export.
- Current Authority And Consumers: a general public event-log type owns file creation/reading despite having no
  runtime file lifecycle; tests/debug helpers call only pretty output, while server compression uses
  `to_json_string`.
- Target Authority: serialization stays on the data type only where consumed; file path/lifetime/error policy stays
  in explicit debug/trace helpers.
- Preserved Invariants: serde shape/version, server compression, log equality tests, best-effort trace behavior and
  no durable player battle history.
- Removal Or Consolidation Scope: remove unused read/plain-write and redundant pretty-string API; relocate or keep a
  narrowly named debug writer until test-output callers are consolidated. Keep `to_json_string`.
- Priority Axes And Total: `1 / 0 / 0 / 1 / 1 / 0 / unknown`; known subtotal `3/18`, no automatic band.
- Validation Safety Net: workspace reference search, serde round trip through standard serde APIs, compression test
  and debug trace failure test.
- Policy/External Contract Decision: public external Rust consumers are unverified; inventory before removal.
- Readiness: `consumer_inventory_then_cleanup`.
- Disposition: `cleanup_candidate`.

## Required Ordering

1. C01 establishes terminal execution authority.
2. C02 can proceed independently after its cross-variant key review.
3. C03 follows C01 or shares one correction Goal so world staging consumes the final execution contract once.
4. C05 is independent and safe after its extreme-value characterization.
5. C04 extracts real producer safety nets before deletion; C06/C07 wait for external consumer inventory.

## Disposed Signals

| Signal | Disposition | Evidence/reason |
| --- | --- | --- |
| Playback pause/speed/fractional remainder | `retain` | one production clock and direct freeze/scale tests |
| Event seq/cause/source-command stacks | `retain` | seq owns order; command ID is correlation only per canonical |
| Result completion monolith | `retain` | staged state plus outer transaction; size alone is not evidence |
| Entry snapshot retreat | `retain` | exact state/RNG/attempt tests; approved monotonic exclusions |
| Retryable/exhausted/fatal defeat split | `retain` | direct world tests and current canonical agree |
| Debug combat trace | `retain` | best-effort failure cannot block result transition |
| Missing retreat snapshot after `take()` | `rejected_current_candidate` | only constructible by invalid/test state; every production start installs snapshot |
| Event-log seq integer overflow | `rejected_current_candidate` | bounded in-memory battle lifetime; no credible reachability before max time |
| VN trigger publication failure | `cross_phase_signal` | Phase 4 C01 owns startup/publication validity; C03 owns advance atomicity only |
| Scenario/setup invalid references | `cross_phase_signal` | Phase 5 C02 owns full setup feasibility publication |
| Tactical/spatial event mechanics | `deferred_phase_07` | lifecycle carriers only |
| Stat/effect/death semantics | `deferred_phase_08` | lifecycle carriers only |
| Target/projectile/area delivery | `deferred_phase_09` | lifecycle carriers only |
| DTO/server cursor ownership refactor | `deferred_phase_10` | server evidence read only to classify reachability |
| Unity/external Rust consumers | `unverified_out_of_scope` | not inspected; public removals explicitly gated |
