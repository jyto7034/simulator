# Core Enhanced Refactoring Audit Phase 06 — Boundary Audit

Audit Date: 2026-08-07
Status: complete; phase-exit audit `Complete / High`

## Exact Boundary

Phase 6 owns the current validated `BattleScenario` to active execution to finalized `CombatBattleState` and the
subsequent retreat/retry/result-completion lifecycle. Mechanics that happen to enqueue events remain carriers for
Phases 7~9; public DTO and direct server transport remain Phase 10.

### Primary runtime

- `src/game/battle/core/{build,commands,ids,mod,sim,types}.rs` at start/queue/clock/finish/finalize seams
- `src/game/battle/{enums,event_log,ids,mod,recording,result_stats,scenario,types}.rs`
- all 14 files under `src/game/battle/validation/**`
- `src/game/world/{combat,combat_trace,state}.rs` at active session, playback, revision, result and rollback seams

### Direct carriers and consumers

- `behavior.rs`, `determinism.rs` and `managers/uuid_manager.rs`
- `data/run_policy_data.rs`, published `game_resources/data/run/policy.ron` and
  `pve_bonus_objectives.rs`
- focused world combat tests plus inline battle/scenario/validator/result tests
- `docs/{core_runtime_contract,game_rulebook,data_loading_contract}.md`
- read-only supporting server actor cursor/tick evidence; server refactor discovery itself remains Phase 10

Phase 5 owns scenario construction and validation before execution. Phase 7 owns spatial/tactical mutation, Phase 8
stats/effects/death, Phase 9 targeting/delivery, and Phase 10 projection/command/server shape.

## Authority Matrix

| Domain | Current authority | Derived/consumer state | Finding |
| --- | --- | --- | --- |
| Battle clock | `BattleExecutionState.last_event_time_ms` plus event queue | world server tick and DTO time | max-cap paths disagree and elapsed-time stepping can stay `Running` forever |
| Finish/finalize | step returns caller-held `BattleFinishSignal`; state stores booleans | `finalize_battle_execution`, world result | split authority; combine with max-cap correction |
| Same-time order | `BattleEvent::Ord`, priority and event identifiers | heap pop, bucket drain, append-only event seq | comparison is not total for distinct semantic events |
| Event history | `BattleCore.event_log` and `event_log_seq` | live delta, result stats, debug trace | runtime starts at seq 1 and appends monotonically |
| Playback | `ActiveBattleSession.playback`, fractional remainder | server tick delta and checkpoints | one pause-aware clock; retained |
| Projection revision | active session revision | every successful mutation DTO | overflow preflight exists; late advance failures remain non-atomic |
| Result facts | finalized event log plus participant results | employee aggregation, MVP, result DTO | aggregation lifetime is coherent; score calculation is duplicated |
| Retreat/retry | battle entry snapshot plus attempt state | node preview or retry/failure transition | exact rollback and attempt rules are strongly protected |
| Result completion | staged result state inside `with_state_transaction` | rewards, run/node completion, compendium | atomic transaction retained |
| Debug trace | best-effort export after final result creation | developer file only | correctly non-authoritative |

## Execution And Max-Time Trace

`start_battle_execution` resets the battle-local queue/log/runtime, validates the scenario, records `BattleStart` at
seq 1, enqueues scenario events and returns `BattleExecutionState`. Production wall ticks are converted by
`simulation_delta_for_tick`; Pause returns no delta and speed uses an integer remainder before calling
`step_battle_execution_by`.

The direct and elapsed-time step paths do not share one terminal rule:

1. Direct `step_battle_execution` copies the next event time into the execution state. If that time exceeds
   `max_battle_time_ms`, it returns a Draw signal at the cap but leaves the stored clock beyond the cap.
2. `step_battle_execution_until` caps the target first. If a queued event lies beyond that target and no winner is
   yet computed, it sets the clock to the cap and returns `Running`.
3. Every later production tick again caps its target to the same value and takes the same `Running` branch. The
   queued event is never consumed, no `BattleEnd` is recorded and world advance can continue committing revisions.
4. The existing max-time test uses `run_battle`, asserts Draw and a `BattleEnd` at the cap, but never exercises the
   production elapsed-time path or checks the execution clock.

Published policy uses a 50 ms movement tick and a 60,000 ms cap. Scheduled movement, cast, attack, buff or scenario
events may legally lie beyond the cap, so the elapsed-time state is production-reachable. A terminal outcome must be
stored by the execution state, clamp its clock to the cap, compute any already-satisfied condition at or before the
cap, otherwise preserve the established Draw-at-cap contract, and be finalized once without trusting a replaceable
caller signal.

## Event Ordering, Recording And Validation

The heap reverses time and priority ordering, then same-time buckets run `sort_by(|a, b| b.cmp(a))`. `Ord` has
identifier tie-breaks for only selected variants. It returns `Ordering::Equal` for distinct `ManualCastEnd` events,
for the `AutoCastEnd`/`ManualCastEnd` cross-variant pair, and for other events whose omitted fields differ despite
derived `PartialEq`. Two units can finish manual casts at the same frozen battle time, making this a live semantic
collision. Heap order among equal values is not insertion order; one completion can invalidate or kill the other,
so gameplay and append-only seq may differ.

The log itself remains the correct current-lifecycle authority. `record_event_log` owns seq, cause and transient
source-command correlation. `source_command_id` is intentionally correlation rather than causal ordering, matching
canonical. Results consume the committed log and participant results rather than replaying gameplay rules.

The separate `battle::validation` harness is not connected to any produced log in the workspace. Its default
contiguous-seq check requires the first entry to be seq 0, while actual logs start at seq 1 and the world setup test
asserts that fact. `require_movement_path_consistent` is declared/defaulted but unread. Its six tests validate the
harness with synthetic logs and custom-disabled fields. Useful invariant ideas should become direct producer tests;
the incompatible public harness should not remain a parallel event-contract authority.

## World Advance, Revision And Failure Trace

`advance_active_battle_by_internal` preflights only revision overflow, then mutates the battle by stepping and may
finalize it. It next evaluates fallible VN triggers, reconciles employee/tactical state, commits the revision, and
only then calls `battle_update_dto_after`, where a future `after_seq` is rejected. Thus a public mutating advance can
return an error after the clock/log/result/employee/revision state changed. The server normally supplies its own
delivered cursor and resets it for a new battle UUID, which reduces but does not eliminate the Core contract defect.
The current future-seq test covers only read-only `RequestBattleState`.

`advance_active_battle_to_next_event_bucket` duplicates the same step/finalize/trigger/reconcile/projection sequence
and is used only by world tests in the workspace. A staged single advance pipeline should validate the cursor first,
run all fallible work against staged active/VN state, and publish one revision/result transition only on success.
Phase 4 C01 still owns invalid VN trigger publication; Phase 6 does not duplicate that data candidate.

Pause, speed, fractional accumulation, blocking VN pause, supplied battle UUID/revision preflight and revision
overflow rejection otherwise have direct production tests and agree with canonical. Read-only setup recovery does
not advance time or revision.

## Result, Retreat And Retry Trace

Finalization logs one `BattleEnd`, snapshots participants and builds `CombatBattleState`. Result stats aggregate all
runtime instances by employee UUID, use the latest deployed instance for terminal HP/survival and retain accumulated
metrics and incapacitation, matching canonical.

MVP selection is the exception: comparison calls `score_total`, whose iterator `sum` can overflow after individual
products saturate; the published MVP DTO calls `score_employee`, which uses `saturating_add`. Typed policy accepts
any non-empty i64-weight list, so valid authored extremes can panic in debug, wrap in release and select a different
employee than the returned score implies. Current published weights are small and unaffected.

`handle_complete_combat_result` is wrapped by `with_state_transaction`. Reward, roster, inventory, UUID, node/run
completion and retry decisions are staged or rolled back; encounter compendium knowledge is deliberately max-merged
as a separate monotonic lifetime. Focused failure tests prove retry after a late reward error does not double-apply.
The long completion function is not a refactor candidate based on size alone.

Retreat takes the active battle and restores its exact entry snapshot. Attempt count, node/session, roster,
inventory, UUID/RNG continuation and cache state are restored; run-power peak and encounter compendium remain
monotonic by approved policy. Retryable defeat preserves the preview and returns to node confirmation, exhausted or
fatal defeat commits the correct run/node outcome. A missing entry snapshot can destroy an artificially-constructed
active session before error, but every production start installs the snapshot; no production reachability was
proved, so this is rejected as a current candidate.

Debug combat-trace failure is logged and ignored before authoritative result transition. Event logs do not become a
durable player history, and result completion does not depend on trace I/O.

## Scenario And Debug API Cleanup

Every workspace construction of `ScenarioEvent` writes `once: true`, but runtime and validation never read the
field because scenario events are enqueued exactly once and spawned groups have their own runtime dedupe. It is a
ghost public input, not an implemented repeat-event policy.

`BattleEventLog::{write_json,read_json}` have no workspace consumers. `to_pretty_json_string` only supports the
file helper, while `write_pretty_json` is test/debug output. `to_json_string` remains used for current log equality
and server compression. Removing unused general file I/O while keeping explicit debug export boundaries is a small
cleanup; unknown external Rust consumers remain unverified.

## Determinism, Identity And Canonical Alignment

- Battle/session/unit identifiers are derived or allocated once at their existing authority; no competing UUID
  lifecycle was found.
- Playback owns production time scaling and checkpoint state; raw direct stepping remains a domain/test primitive.
- Event seq, not source command id, owns semantic presentation order as canonical requires.
- Result stats and debug traces are derived from the current finalized lifecycle only.
- Retreat/retry/result completion match the game rulebook and runtime contract.
- The max-cap stall, non-total event order and late-error advance mutation contradict the same canonical clock/order/
  no-mutation invariants and are correctness corrections rather than new gameplay policy.

## Consumer Classification

- Confirmed authoritative consumers: Core world tick/commands, active session projection, result completion and
  current game-server battle cursor/tick call.
- Confirmed derived consumers: result stats, Core live DTO conversion and best-effort debug trace.
- Unity is explicitly out of scope and was not inspected. Phase 10 owns public projection/transport refactor work.
- Unknown external Rust users of public battle step, validator, scenario and event-log helper APIs are unverified;
  public removals require a compatibility inventory even when the workspace search is empty.

## Boundary Verdict

Three production-contract corrections are high priority: terminal max-cap ownership, total same-time ordering and
atomic world advance. One accepted-policy score defect and three cleanup surfaces are separately executable. The
current result transaction, retreat/retry snapshot, playback clock, event log, debug trace and identity lifetimes
are retained. Phase 6 has complete evidence without changing production files.
