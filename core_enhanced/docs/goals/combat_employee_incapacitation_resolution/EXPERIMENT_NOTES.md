# Combat Employee Incapacitation Resolution Notes

Status: complete

## Fixed Decisions

- Tactical death and persistent employee death are distinct.
- Penalties are per tactical death; XP, summary, and MVP are per employee.
- Attempt-local employee state is world-owned and seeded from the battle-entry roster.
- Runtime-unit identity is the idempotence key for applying a tactical death exactly once.
- Updated employee state, not deployment count or a fixed percentage, drives redeployment.
- G26 supplies commit/discard semantics for employee conditions; G15 later composes exact complete
  battle-stage abandonment rollback.

## Resolved Runtime Mismatch

- Death reconciliation now applies one complete penalty bundle to the working employee per new dead
  runtime instance before projection/redeployment.
- Death redeploy has no HP override and uses the ordinary employee condition stat pipeline.
- Result completion commits the carried working roster once and grants/produces employee-level XP,
  result rows, MVP, reward context, and outcome changes.

## Closed Technical Decisions

- `ActiveBattleSession.employee_attempt` owns the roster clone and processed instance set.
- Every projection/command path reads that roster while the attempt is active; persistent state is
  unchanged until result settlement and naturally restored when the active attempt is discarded.
- The last chronologically spawned/deployed employee instance supplies terminal HP/survival and MVP
  representative identity; incapacitation and metrics aggregate across all employee instances.

These are implementation details under accepted policy, not open user policy questions.

## Out-Of-Scope Follow-Up

- G15 must prove exact pre-entry restoration for attempts, RNG, consumption, node/reward state, and
  all other gameplay mutations; G26 must not claim that broader rollback from an employee-only test.
