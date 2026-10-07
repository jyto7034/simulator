# P03 Application Authority Experiment Notes

## Intake Decisions

- The public `GameState` semantic variants remain the observable phase vocabulary. P03 changes their authority: a
  private `LifecycleContext` owns the closed phase and exact context checks, while `GameState` is returned as a
  snapshot. This is a boundary rewrite, not a gameplay-state rename.
- The complete-root draft is deliberately the first correctness implementation. P08, not P03, owns retained-memory
  optimization and performance PASS. No performance normalization or threshold is introduced here.
- A successful command result is prepared before root replacement. Therefore ordinary Core execution has no
  post-commit fallible projection. Typed accepted/recovery-pending publication exists to prevent a future diagnostic
  publisher from reclassifying an accepted commit as `Err`.
- Local domain transactions may remain temporarily inside the draft because they do not compete for live authority.
  Public/root entrypoints may not call domain handlers outside the application executor.
- Host recovery variants present in `PlayerBehavior` remain rejected by the domain behavior router as today; the actual
  active-run continue/rollback APIs are separately registered mutation routes. P07 owns later public application
  command projection changes.

## Evidence Rules

- Equality assertions must include phase/context, run/checkpoint/durable state, battle revision/event/projection and
  `run_seed` as applicable. A single response-only comparison is insufficient.
- CA-003 evidence must exercise all six actual commands with a future cursor after each command has passed its normal
  domain preconditions.
- The missing RunStart trigger fixture remains test-only and must show both exact rollback and identical retry error.
- Registry completeness must use exhaustive enum mapping and compare every `ActionKind`; string lists alone are not
  evidence.
- GEX-20 will use a frozen content target and a fresh read-only pass. Audit discovery returns to correction GEX-10; it
  cannot be silently edited into a Complete verdict.

## GEX-10 Notes

- Direct assignments remaining inside domain methods operate only on the private complete `GameCore` draft supplied by
  `execute_application`. Public/root callers have no path to those handlers; the only live root replacement is
  `commit_application`. Static assignment count without call-context is therefore not a valid bypass count.
- Read-only behavior variants currently traverse the same complete draft path because the existing public behavior API
  takes `&mut self` and P07 owns the transport-neutral command/query split. This favors one enforced boundary in P03;
  P08 may optimize proven read-only and scoped draft costs after all functional cutovers.
- `AcceptedRecoveryPending` is intentionally not constructed by current production: every current result projection is
  prepared before commit. The typed branch and direct unit test pin the rule that a future post-commit diagnostic fault
  cannot become `Err`.
- The invalid release probe is expected to fail compilation. Its exit 101 and three distinct compiler errors are the
  PASS condition.
