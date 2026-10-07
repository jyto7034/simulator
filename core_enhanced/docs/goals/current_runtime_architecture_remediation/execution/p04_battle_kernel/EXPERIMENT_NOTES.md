# P04 Battle Kernel Experiment Notes

## Intake Decisions

- “Private kernel” means default-release callers cannot name or construct the kernel module/types. It does not require a
  crate split, and it does not hide the internal type from the Core world/event modules that own it.
- Test/debug/test-fixtures exposure is a capability for the existing deterministic integration oracle. It is not a
  production API claim; default-release compile-fail evidence is mandatory.
- A scoped command/query port may remain on the internal kernel for the actual world owner. Public raw fields and raw
  event injection are bypasses; the internal methods themselves are not competing authority when only the P03
  application-owned active battle can reach them in production.
- Non-finite values are rejected, never clamped, defaulted, canonicalized or assigned an accepted heap order. Signed
  zero remains finite and preserves ordinary IEEE equality behavior.
- Integer tile coordinates and millisecond time remain canonical application inputs. Finite float wrappers protect the
  internal continuous-space conversion boundary without changing the public gameplay vocabulary.
- P05 owns history/event-log retention and cursor floor. P04 may validate queue/log equality but must not add a cap,
  expiry or result-lifetime policy.
- P07 owns the final transport-neutral public command/result/snapshot projection. P04 must not pre-implement that
  surface or make external compatibility claims.

## Evidence Rules

- A visibility text scan is insufficient: the default-release negative probe must actually fail to import/name the
  kernel and to reach direct unit/event mutation.
- A constructor-only NaN test is insufficient for rollback: the matrix must hold a real kernel authority snapshot,
  attempt the checked boundary, and prove first/retry equality of unit, queue/order, event, battlefield and sequence
  state.
- Finite parity must include actual validated live data and actual Core world producer-consumer paths, not only isolated
  math tests.
- Existing boss, movement and skill expectations may be mechanically adapted to a test capability gate, but their
  semantic assertions may not be weakened or replaced with broad snapshots.
- GEX-20 will audit a frozen target in a separate read-only pass. Any completion defect returns to correction GEX-10;
  it is not silently edited into a passing verdict.

## GEX-10 Findings

- Conditional module visibility is the authority boundary: internal `pub` fields/helpers are not a production-public
  bypass when the enclosing module is unnameable outside the crate in default release. The compile-fail probe, not a
  lexical `pub` count alone, is the deciding evidence.
- `BattleEvent` remains a public vocabulary enum, but default release has no public queue injection port and cannot
  construct its kernel-private `WorldVec2` payload unchecked. Event ordering therefore never receives an accepted
  non-finite position through a production boundary.
- `u64` milliseconds always convert within finite `f32` range. Keeping a checked `FiniteBattleSeconds::from_millis`
  makes that invariant explicit without changing step duration or introducing a fallible public command.
- Raw arbitrary-float test constructors remain available only under test/debug/test-fixtures so the rejection matrix
  can reach the scoped boundary. Production scenario values cross checked radius/speed preparation first.
- Capability-gating unused debug helpers and `target_traits` in optimized release is an API/dead-code cleanup, not a
  battle algorithm change. The focused deterministic corpus and 1,138-test full oracle are the semantic guard.
- Whole-library Clippy still encounters the pre-existing denied `never_loop` in
  `src/game/data/boss_runtime_data.rs`; release checking is warning-free and no lint exception was added for P04.
- P08 retains ownership of timing/memory comparison. P04 records no performance PASS from debug test durations.
