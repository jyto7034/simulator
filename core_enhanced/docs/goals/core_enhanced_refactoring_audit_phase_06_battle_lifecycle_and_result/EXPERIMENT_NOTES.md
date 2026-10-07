# Core Enhanced Refactoring Audit Phase 06 — Experiment Notes

Status: complete

## Boundary decision

- Primary authority: validated battle scenario/active session → playback-gated execution queue → event log/result →
  world retreat/retry/completion.
- Phase 5 owns setup; Phases 7~9 own mechanics that emit lifecycle events; Phase 10 owns public projection/transport.
- Large shared files are read by responsibility slice, not treated as one refactor target.

## Dirty precautions

- Current dirty tree is the baseline; production files may already be modified by the user.
- Baseline hashes/status must be captured before detailed source reads.
- Only Master/Phase Goal artifacts are audit-owned.

## CRA-14 policy

- Test families are read for protected production invariant and assertion meaning.
- Mechanics-heavy tests count only as lifecycle evidence when they assert clock/order/revision/log/result boundaries.
- `remove` requires no unique contract and an equal/stronger surviving safety net.
- Runner green counts do not close static coverage gaps.

## Questions to resolve

- Whether playback, execution state and event queue have one exact clock/finish authority.
- Whether event ordering/root/seq/source-command state is duplicated or can drift.
- Whether result stats, completion and retreat/retry rebuild facts from inconsistent sources.
- Whether invalid/late lifecycle failures can partially consume revision, attempt, run or persistent result state.

## Handoff condition

Phase 7 stays inactive until Phase 6 has complete boundary/candidate/test artifacts, no-mutation proof and an
independent `Complete / High` audit.

## Resolved findings

- Elapsed-time stepping can stay `Running` forever at the max-time cap while direct stepping finishes with a clock
  beyond the cap. C01 owns state-held terminal outcome and unified cap semantics.
- Same-time `BattleEvent::Ord` is not total; production-reachable manual cast completions can compare equal. C02 owns
  a stable semantic sort key without reopening mechanics.
- World advance performs fallible cursor/trigger/reconciliation work after BattleCore mutation and duplicates a
  test-only bucket pipeline. C03 owns staged no-mutation-on-error orchestration; Phase 4 retains trigger publication.
- The validation harness is workspace-dead and rejects actual seq-1 logs by default. C04 extracts useful produced-
  log assertions and removes the alternate contract after external consumer review.
- MVP selection and publication use different overflow arithmetic. C05 owns one saturating score; bounds/sign policy
  remains separate.
- Ghost `ScenarioEvent.once` and unused general log file I/O are C06/C07 cleanups with public consumer gates.
- Playback, event recording, result transaction, retreat/retry snapshot, debug trace and deterministic identifiers
  remain their current authorities.

## Final handoff

- Seven candidates: four correctness corrections and three cleanups.
- C01 precedes or joins C03; C02 is independently test-first; C04 requires produced-log test extraction; public
  C04/C06/C07 removals require external consumer inventory.
- Phase 7 may activate after the completion audit records `Complete / High`.
