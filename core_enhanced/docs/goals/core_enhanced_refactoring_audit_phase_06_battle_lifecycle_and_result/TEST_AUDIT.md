# Core Enhanced Refactoring Audit Phase 06 — CRA-14 Test Audit

Audit Date: 2026-08-07
Status: complete; phase-exit audit `Complete / High`

Runner results are supporting evidence only. Each disposition below comes from reading setup/action/assertions and
comparing them with the production path, current live policy and canonical.

## Family Dispositions

| Family | Production invariant and assertion meaning | Reachability/overlap assessment | Disposition |
| --- | --- | --- | --- |
| Battle reset/start and RNG observation | start clears battle-local runtime, seq/queue/hooks and observation does not perturb deterministic decisions | direct domain path; observation variants protect distinct RNG identity | `retain` |
| Full-run vs stepped execution | current ordinary scenarios produce identical JSON log/result under full and incremental stepping | production-adjacent but no event beyond max cap; equality is meaningful | `strengthen` for C01 |
| Runtime max time/movement tick | live policy values drive tick cadence and full-run Draw/BattleEnd at cap | max test calls only `run_battle`, omits stored clock and world tick | `strengthen` for C01 |
| Same-time heap/bucket ordering | priority/selected IDs and AttackResolve-before-BattleEnd become ordered log seq | only selected variants; no `ManualCastEnd` collision or totality property | `strengthen` for C02 |
| Manual/autocast lifecycle | manual selection, interrupt/stale end, focus/cancel and deterministic autocast cause scheduling | live command paths; mechanics assertions stay Phase 9 but lifecycle assertions are unique | `retain`, add C02 simultaneous completion case |
| Scenario validation/event scheduling | invalid field/duplicate tactical points fail and delayed/empty scenarios have explicit events only | empty manual fixture is test-oriented but validates an approved construction seam | `retain`; remove `once` writes with C06 |
| Event-log serialization/debug output | JSON equality and debug artifacts make deterministic traces inspectable | general file writer smoke overlaps explicit trace boundary; output files are not gameplay | `merge` debug helpers; remove unused APIs with C07 |
| Event-log validator self-tests | synthetic logs exercise version, buff and cast-parent rules | no produced log; default seq contradicts runtime; all consumers are self-referential | `remove_after_extraction` with C04 |
| Result-stat aggregation | actual HP delta, player-object exclusion, redeploy merge and incapacitated MVP eligibility | finalized log/result path; four tests protect distinct aggregation lifetimes | `retain`; add overflow identity cases for C05 |
| Initial update/setup recovery/resync | first event seq 1, immutable setup recovery and read-only future cursor rejection | production Core route; future cursor test is read-only only | `retain_and_strengthen` mutating advance for C03 |
| Playback/VN tick/speed | Pause and blocking VN freeze ticks; speed and half-speed remainder advance exact battle time | production server-tick path; assertions observe time/revision/DTO, not private layout | `retain` |
| Battle UUID/revision preflight | stale supplied authority and revision overflow reject before playback mutation | production command boundary; exact no-mutation assertions | `retain` |
| VN battle-time triggers | threshold crossing fires once and overlay uses battle update transport | production-reachable authored triggers; failure-path publication belongs Phase 4 | `retain`; stage failure with C03 |
| Finish/result/trace transition | live route reaches typed CombatResult and trace failure cannot block transition | direct end-to-result boundary; slow full route adds unique assembly evidence | `retain` |
| Result completion atomicity | late reward failure rolls back, retry succeeds once, XP/conditions/rewards commit once | production result command and whole-state observations | `retain` |
| Retry/failure/retreat | retryable vs exhausted/fatal decisions, exact entry snapshot/RNG and repeated/final-boss retreat | production paths; tests protect distinct attempt and monotonic-lifetime rules | `retain` |

## Exact Gaps And Required New Tests

### C01 terminal cap

- Elapsed-time `step_battle_execution_by` with a queued event after the cap must return `Finished`, keep clock exactly
  at the cap and finalize once.
- Direct/until/by must agree for an unsatisfied condition, a condition satisfied before the cap and an exact-cap
  condition.
- Production world ticks must transition to `CombatResult` and stop consuming revisions after the cap.
- Finalization must consume the state-owned outcome and reject replay/replacement.

### C02 total same-time order

- Assert `cmp == Equal` only for equal events across every live variant/key combination.
- Insert the same simultaneous manual/manual and auto/manual completion set in multiple heap permutations and assert
  identical event seq, HP/death/cast state and result.
- Retain the current AttackResolve-before-BattleEnd case as a priority-regression anchor.

### C03 atomic advance

- Call the mutating advance API with future `after_seq`; assert exact active battle clock, queue/log, employee attempt,
  VN state, fractional remainder, revision and world state equality.
- Inject late trigger/reconciliation/projection failures before and at terminal transition and assert no partial
  finish/result state.
- Assert success parity between raw next-bucket test stepping and elapsed-time stepping while only one orchestration
  implementation remains.

### C04 validator cleanup

- Before removal, feed representative current produced logs to inventory which checks are valuable and which fail
  because the harness is stale.
- Move retained version/seq/parent/spawn/reference/buff/death/cadence invariants to tests of actual BattleCore logs.
- Delete the six self-tests only after equal or stronger producer assertions exist.

### C05 score arithmetic

- Multiple `i64::MAX` and `i64::MIN` weighted metrics must have deterministic saturating totals without debug panic.
- The value used to choose the MVP must equal the score emitted for that employee.
- Current live-policy score/title/tie behavior and UUID fallback remain unchanged.

## Merge/Removal Decisions

- The validator's six tests are the only Phase 6 `remove` family. They preserve a disconnected contract and cannot
  survive as evidence merely because they are green. Useful assertions move first.
- Pretty event-log file-output helpers in `tests/common`, world test support and a battle command test should share
  one explicit debug artifact helper; plain read/write API has no unique test contract.
- Ordinary scenario/result/world tests that look broad are retained when they observe distinct topology, result
  aggregation, run transaction or rollback lifetime. No test is removed based on happy-path status or file size.
- No `#[ignore]` marker exists in the Phase 6 scoped production/test files.

## Focused Runner Evidence

| Command/filter group | Passed | Failed | Ignored | Meaning |
| --- | ---: | ---: | ---: | --- |
| `game::battle::enums::tests` | 1 | 0 | 0 | current partial heap ordering |
| `game::battle::result_stats::tests` | 4 | 0 | 0 | aggregation/redeploy/incapacitation |
| `game::battle::scenario::tests` | 4 | 0 | 0 | scenario construction/validation |
| `game::battle::validation::validator::tests` | 6 | 0 | 0 | disconnected harness self-tests |
| four exact core execution/cap/order tests | 4 | 0 | 0 | current full-run/ordinary step semantics |
| five exact world cursor/recovery/playback/revision tests | 5 | 0 | 0 | read-only cursor and preflight protections |
| five exact world trigger/result/retreat/retry/trace tests | 5 | 0 | 0 | retained result and rollback boundaries |

Eighteen valid invocations executed 29 passing tests with no failure or ignore. Four earlier exact invocations used
unqualified names, selected zero tests and were discarded; the same four tests were rerun with fully qualified names
and each executed once. Green execution does not close any gap listed above.

## CRA-14 Verdict

Complete. Every assigned lifecycle/result family has a meaning, reachability, overlap and disposition. The retained
tests protect real clock, projection, result or rollback contracts; the only removal family is self-referential and
has an explicit stronger replacement; candidate prerequisites identify the exact unobserved failure boundaries.
