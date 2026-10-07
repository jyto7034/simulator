# P05 Bounded History and Result Lifetime Experiments

## 2026-08-23 — GEX-00 Intake

- Run mode: full Goal Set `goal-run`; selected
  `(GS-CRAR-P2G50-002, GOAL-CRAR-C2-P05-BOUNDED-HISTORY-RESULT)`.
- Execution node: standalone `WP-CRAR-C2-06`; parallel execution is not authorized.
- P01, P03 and P04 current completion evidence was read before selection. All direct prerequisites are
  `Complete / High`.
- Existing exact-identity P05 execution set: none. Canonical new directory
  `execution/p05_bounded_history_result` selected.
- Production/source mutation and test execution during intake: none.

### Actual Producer/Consumer Trace

- Kernel events flow through the single `BattleCore::record_event_log` push into an unbounded vector.
- Live Core projection derives latest sequence from the vector, scans all entries greater than `after_seq`, clones the
  tail, and maps it to presentation DTOs. Only future cursors are rejected today.
- Terminal finalization records `BattleEnd`, clones the full log into `BattleResult`, derives all result statistics from
  that log, then retains the log in `CombatBattleState`. The diagnostic attachment clones it from durable result state.
- Result statistics need UnitSpawned identity plus deployment/withdrawal, attack/skill, HP, death and battle-end events.
  A bounded window alone would lose those inputs, so an incremental aggregate is mandatory for semantic parity.

### Feasibility and Policy Interpretation

- `TD-CRAR-018` and the Goal's old-cursor resync condition imply a moving retained window: oldest entries are evicted
  while new diagnostic events continue. Stopping the battle or stopping all event sequence production at 180 seconds
  would contradict the explicit non-timeout rule.
- Count and time accounting are infallible. Compact JSON serialization of a single event defines byte accounting;
  serialization/accounting failure is handled as diagnostic loss and cannot return a gameplay error.
- Sequence identity remains monotonic independently of retention. On `u64` exhaustion the window closes
  diagnostically, keeps no duplicate retained sequence, and gameplay/result accumulation continues.
- Checkpoint encoding is not changed. P05 proves continuation state has no history field and remains semantically equal;
  P06 alone owns V4/V5 codec migration.
- P08 owns latency, heap/allocation measurement and performance PASS. Fixed upper-bound work is a P05 correctness
  property, not a performance acceptance claim.

### GEX-00 Result

- Result: `passed`.
- Approved next pass: GEX-10 bounded window, cursor floor/resync type, incremental result aggregate, production result
  lifetime cut, direct matrix/full validation, predecessor currentness and frozen target.

## 2026-08-23 — GEX-10 Implementation and Correction Loop

### Retention and Cursor Experiments

- Count exact/cap+1 retained 8,192 consecutive entries and advanced floor by exactly one. Floor projection repeated
  identically; cursor below floor returned the typed resync tuple and future returned the existing invalid tuple.
- Synthetic compact-byte accounting accepted an entry-array of exactly 4 MiB, dropped a 4 MiB+1 array, and closed the
  diagnostic window on injected encoding failure. Gameplay error count remained zero.
- Time-span exact/cap+1 accepted 180,000 ms and evicted the oldest at 180,001 ms. A first implementation treated
  nonmonotonic recorded event time as diagnostic corruption and broke two item-proc oracles; correction computes the
  max/min retained span while preserving legal causal recording order. Both focused failures and the full suite pass.
- `u64::MAX` sequence is retained once; retry emits no duplicate and the window closes diagnostically.

### Result and Lifetime Experiments

- The existing detailed result corpus was observed incrementally, its retained log was cleared, and completed stats
  remained byte/value equal across duration, kill, damage, attack/skill, deploy/withdraw/time, final HP, survival and
  MVP fields.
- The actual long live defense route initially exposed an old-cursor rollback when one command crossed the count floor.
  Correction preserves strict pre-mutation rejection for already-old/future cursors but returns a typed resync
  checkpoint when the floor advances during an already authorized command. The long route then passed to CombatResult.
- Default release compiled warning-free with diagnostic terminal fields and attachment removed. The invalid standalone
  probe failed exactly because both DTO and method are capability-gated; the valid live producer/consumer route ran.

### Broad Result

- Focused event-log tests: 6/6; typed live expired-history recovery: pass; long live defense terminal path: pass.
- Full Core: 1,092 library + 51 integrations = 1,143/1,143.
- P01 verifier: 57 features, 42 locator contents, composite
  `c0306c0536eb93f8b41b27714ae33d8d0ddcafbe582d26d0d1b7a2c5063de095`, errors 0.
- P03 and P04 exact frozen-member comparisons found only the declared P05 subordinate deltas and produced direct
  successor-currentness records. GEX-10 exit criteria are met; next operation is immutable target freeze.
