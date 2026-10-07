# Gap Register

- 기준선: `B-001`
- 상태: 최종

## GAP-001. Future-cursor failure tests omit six live mutation commands

- criterion: C03, C06, C11
- current evidence: E-009 and RUNTIME-AUDIT-005 statically prove post-mutation cursor error. E-002 baseline names tests
  for mutating/finishing advance and five staged tactical world commands.
- missing: pause, resume, set-speed, begin-targeting, cancel-targeting, activate-skill before/after equality assertions.
- why unresolved: audit may not modify production tests; static proof is sufficient for finding but regression protection is
  absent.
- follow-up method: remediation tests should snapshot complete `ActiveBattleSession` (not only DTO), execute each command
  with `after_seq = latest + 1`, assert exact error and semantic equality, then retry with valid cursor.

## GAP-002. StartNewGame invalid-static-data test omits failed-state equality

- criterion: C03, C07, C11
- current evidence: E-002 ran an error-only missing-trigger test; E-010 proves mutation before error.
- missing: player, game state, pending mode, roster, run/session/battle/checkpoint/VN/UUID/resources full before/after equality.
- follow-up method: remediation test captures complete non-battle transaction state, executes missing-trigger start, asserts
  exact typed error, exact equality, and safe retry.

## GAP-003. Checkpoint continuation equivalence is representative, not exhaustive

- criterion: C04, C08, C11
- current evidence: E-011 tests recapture semantic identity and deterministic gate replay; several domain states round-trip.
- missing: one canonical command sequence executed from original checkpoint state and a separately continued state with
  canonical result/event/final-authority equality across RNG/UUID-sensitive event/shop/combat flow.
- follow-up method: construct writer-produced current payload, fork original/continued cores, run identical typed command
  sequence, compare ordered results/events and semantic authority without sorting away observable order.

## GAP-004. Live battle history has no growth/complexity regression guard

- criterion: C11, C12
- current evidence: E-018 proves append-only retention and O(total retained entries) delta extraction on each live tick;
  E-021 measures approximately linear 1K→10K→100K recent-tail growth against the actual method.
- missing: representative long battle measurement and an assertion that recent-cursor extraction cost/allocations depend on
  returned delta rather than full history; explicit maximum retained events/bytes/work budget contract.
- why unresolved: the audit probe supplies magnitude evidence but cannot choose a product retention/latency budget or add
  a permanent regression benchmark without remediation scope.
- follow-up method: choose a product budget, add indexed/ring/segmented history, benchmark increasing history with constant
  tail, and test deterministic overflow/limit behavior plus result-log retention policy.

## Closed investigation dimensions

- C01/C02 public authority and representative vertical mutations: A-001~A-005, T-001~T-003.
- C05/C06 dependency/DTO change amplification: E-015/E-016/E-020.
- C09 declared feature reachability: E-019/E-020, final PASS.
- C10 panic/log/I/O: E-019.
- C11 traceability: `TEST_TRACEABILITY.md`.
- C13 root causes: `FINDINGS.md` RC-01~RC-03.
