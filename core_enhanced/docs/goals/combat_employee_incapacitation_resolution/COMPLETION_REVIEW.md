# Combat Employee Incapacitation Resolution Completion Review

Review date: 2026-07-14

Review procedure: `docs/goal_completion_review_guide.md`

Primary verdict: Complete

Long-term fit: High

Improvement class: none

## Item Audit Table

| Item | Evidence status | Verdict | Long-term fit | Immediate correction |
| --- | --- | --- | --- | --- |
| MSTR-01 fixed-60% death redeploy regression | Removed; redeploy uses current working employee condition and the ordinary Battle HP pipeline | Complete | High | None |
| CPR-H-01 per-instance penalties versus employee outcomes | Runtime deaths are instance-idempotent; XP/result/MVP/outcome rows aggregate by employee | Complete | High | None |

## MSTR-01 — Attempt-Local Employee Condition Owns Death Redeploy

Master item:

The legacy fixed 60% death-redeploy override replaced the intended immediate trauma-aware employee
condition for implementation simplicity.

Policy source:

- Accepted PD-16 employee incapacitation resolution.
- Master Q10 tactical death, persistent death, and whole-stage abandonment clarification.

Expected behavior:

Each dead player employee runtime instance applies the complete authored penalty bundle exactly once
to battle-attempt working state. A fresh deployment uses that current employee's normal stat and
Battle HP pipeline. Persistent death in the working state blocks redeployment. The persistent entry
roster remains unchanged until result commit and is restored by abandoning the attempt.

Runtime evidence:

- `src/game/world/state.rs` gives `ActiveBattleSession` one `BattleEmployeeAttemptState` containing
  the battle-entry roster clone and processed `UnitInstanceId` set.
- `src/game/world/combat.rs` prevalidates new dead deployed employee instances, composes the existing
  trust reaction and `Employee::apply_incapacitation` pipeline, marks each instance processed, and
  reconciles deployment before the revisioned projection is committed.
- Capability, preview, deploy, update, setup recovery, VN transport, and roster snapshot paths read
  `active.employee_roster()` while the attempt is active.
- Death redeploy locks carry no current-HP override. Withdraw redeploy retains its distinct explicit
  carried-HP rule.
- `CombatBattleState.employee_conditions` carries the completed working roster into result
  settlement. Settlement replaces staged roster condition once rather than applying participant
  penalties again.
- `RetreatBattle` removes the active session and therefore discards the working roster; the
  persistent battle-entry roster remains authoritative afterward.

Data evidence:

- No RON, loader, schema version, authored penalty value, or save format changed.
- The existing live post-battle trauma and Run HP values remain the sole authored balance inputs.

External contract evidence:

- No Rust serialized DTO field, server mapper, WebSocket message, or Unity reader/command shape
  changed.
- The existing employee roster DTO now projects working trauma/life/health during battle and the
  restored entry values after abandonment; the focused Core test serializes and checks both states.
- game_server compiles against the changed Core internals and all server tests pass.
- Unity's unchanged strict readers passed the full MCP EditMode suite 189/189. The post-run TestRunner
  save message was the sole console entry; after clearing that runner output, the error query was 0.

Test evidence:

- The pre-implementation focused test reproduced current HP 60 versus employee-pipeline HP 100.
- The corrected test covers one-death idempotence, three distinct death bundles, two trauma-aware
  redeployments, persistent-death lockout, unchanged entry roster, existing roster JSON projection,
  and abandonment restoration: 1/1 passed.
- Existing employee composition tests for trauma consumable mitigation, one-use death prevention,
  and trust trauma reduction: 3/3 passed.
- Battle-start HP policy group: 4/4 passed.

Legacy/fallback audit:

- Production/canonical searches contain no `DEATH_REDEPLOY_PERCENT`, `death_redeploy()`,
  `PercentOfMax`, fixed-death-60 wording, handler-specific compatibility, or relevant ignored test.
- Historical goal documents retain the superseded decision as evidence and are not runtime or
  canonical policy sources.

Long-term direction review:

- Fit: High.
- Improvement class: none.
- Reason: BattleCore still owns runtime lifecycle/replay, while one world-owned transaction state
  owns employee condition, projection, deployability, commit, and discard. No parallel persistent
  roster mutation or HP fallback remains.

Verdict:

- Complete.

Remaining risk:

- G15 must extend the already-correct employee discard into exact restoration of attempts, RNG,
  inventory/consumption, node/reward state, and every other authoritative gameplay mutation.

## CPR-H-01 — Employee-Level Non-Penalty Aggregation

Master item:

Runtime-instance participant rows mixed intended per-death penalties with employee-level survival
XP, result summaries, and MVP state, allowing duplicate or UUID-order-dependent persistent output.

Expected behavior:

Instance events and participant results remain intact for simulation/replay. The last
chronologically spawned/deployed instance represents terminal HP/survival and MVP identity; any
incapacitated instance marks the employee incapacitated; additive metrics combine across instances.
Survival XP, reward participant context, result rows, and node employee changes occur once per
employee.

Runtime evidence:

- `src/game/battle/result_stats.rs` tracks the representative instance in event chronology, ORs
  incapacitation across participant instances, applies the representative terminal state only after
  event aggregation, and emits one sorted employee row.
- MVP scores the one aggregated row and uses its representative instance identity.
- World survival XP, reward `CombatParticipants` context, and `NodeOutcomeEmployeeChange` consume
  the employee result rows rather than raw participant instances.
- Raw `participant_results` and the complete event log remain on `CombatBattleState` unchanged.

Data evidence:

- No growth amount, MVP weight, reward definition, or result schema changed.

External contract evidence:

- `selected_event.result_stats` keeps its exact DTO shape but now contains deterministic
  employee-level terminal semantics. Unity continues to consume Core output and does not aggregate
  raw logs locally.

Test evidence:

- A reversed participant list with two instances proves one row, two deployments, cumulative
  incapacitation, last chronological instance HP/survival, and the same MVP representative: 1/1
  passed.
- Victory settlement with a dead first instance and surviving second instance proves two working
  penalties commit without reapplication and survival XP is granted once: 1/1 passed.
- Defeat settlement proves one working penalty commits once and returns to retry flow: 1/1 passed.
- Result stats module 4/4, live deployment 2/2, and combat-result focused 4/4 passed at the recorded
  focused gate.

Legacy/fallback audit:

- Result settlement no longer iterates participant instances to apply penalties or create outcome
  rows. No dedupe fallback, UUID-order terminal overwrite, compatibility DTO, or ignored regression
  test remains.

Long-term direction review:

- Fit: High.
- Improvement class: none.
- Reason: instance facts and employee outcomes now have explicit separate owners and combine through
  one result-stats projection instead of repeated ad hoc dedupe.

Verdict:

- Complete.

Remaining risk:

- None inside G26. G15 owns transition priority and whole-attempt atomicity beyond employee state.

## Cross-Component Findings

- The Phase 1 battle revision commits the working deployment/roster projection after death
  reconciliation, so capability, deployability, and roster condition share one revisioned state.
- Persistent roster mutation occurs only through staged result settlement; active attempt
  projection never mutates the entry roster.
- Result aggregation changes semantics, not wire shape, and remains wholly Core-authored.
- No live-data, server-ordering, or Unity-authority seam was added.

## Validation Results

- Core focused G26 behavior: 5/5 named contract tests passed.
- Employee penalty composition: 3/3 passed.
- Battle-start HP: 4/4 passed.
- Core library: 671 passed, 0 failed/ignored.
- Core all-targets: 712 passed across nonzero targets, 0 failed/ignored.
- `cargo check -q -p game_core -p game_server`: passed.
- game_server all-targets: 28 passed, 0 failed/ignored.
- Unity MCP full EditMode job `a0d1447d68fa4a7da1b326076db448a6`: 189 passed, 0 failed/skipped.
- Unity post-run error console after clearing runner output: 0 entries.
- Formatting, scoped diff, new-document whitespace, legacy fixed-60%, and relevant ignored-test
  searches passed.

Failed experiments retained in `EXPERIMENTS.md`:

- The initial focused test failed on the intended fixed-60% regression.
- One full library run found a misplaced assertion block; it was corrected and the full suite rerun.
- One test filter matched zero tests and was replaced with exact nonzero filters.
- One hygiene command used the wrong repository-relative paths and was rerun correctly.

## Immediate Correction Candidates

None.

## Follow-Up Refactor Candidates

None inside G26. The required G15 expansion is a dependent policy implementation, not a G26
refactor.

## Policy Questions

None.

## Review Conclusion

G26 closes MSTR-01 and CPR-H-01 with Complete / High long-term fit and no immediate correction. G15
may begin on the accepted result/abandonment/failure transition policy.
