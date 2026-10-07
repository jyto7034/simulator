# Boss Runtime v1 Phase 01 — Notes

## Activation

- P00 verdict is `Complete / High`, `ready_for_g0`.
- P01 owns CRA-P06-C01/C02/C03 and no Boss-specific production type.
- Milestone order is C01 → C02 → C03.
- Current dirty source/test files are the baseline. Only P01-specific symbol/hunk changes may be attributed here.
- Findings, failed tests and implementation decisions are written after each bounded milestone, not reconstructed at
  Phase end.

## Current narrow decisions

- Terminal outcome becomes state-owned and one-shot; caller-supplied replaceable finish authority is removed.
- Existing event priority bands do not change. AutoCastEnd precedes ManualCastEnd inside their shared band.
- World advance stages only state actually mutated by advance. Tactical player-command transactions stay separate.
- P01 introduces no Boss schema, live content, public wire or Unity work.

## M1 checkpoint — 2026-08-09

- Terminal authority now lives in `BattleExecutionState`; finalization cannot substitute a caller-provided result.
- All step entrypoints clamp time, evaluate the exact cap consistently and finish once.
- The real server-used tick reaches one terminal update/`CombatResult` at the cap.
- Focused new and retained tests are green. M1 is ready for combined regression after M2.

## M2 checkpoint — 2026-08-09

- All 19 variants now participate in a total deterministic order without changing established priority bands.
- The full pair/cross matrix, 720 insertion permutations and explicit cast-end ordering tests are green.
- M3's first failing test proves the current world path mutates clock and revision before rejecting a future cursor.

## M3 checkpoint — 2026-08-09

- All live advance variants now enter one scoped staged transaction; error returns restore the original exact active
  Core/world/VN/UUID state.
- Server fractional remainder and finished GameState/content transition are inside the same stage.
- Real future-cursor, revision, partial VN and employee reconcile failures are exact no-mutation green.
- Elapsed and next-bucket success paths produce identical DTO/log/queue/revision state for the same bucket.
- Core (100) and world combat (81) focused regressions are green. P01 now moves to canonical and broad validation.

## Completion checkpoint — 2026-08-09

- P01 implements exactly CRA-P06-C01/C02/C03: one terminal authority, total event order and atomic live advance.
- Focused Core/event/world suites, full Core, server and workspace all-target validation are green with zero failures
  and zero ignored tests. The one formatter failure was corrected in four scoped P01 hunks and is retained in the
  experiment history.
- The closest Core runtime canonical now records the implemented terminal, same-time ordering and transaction rules.
- No Boss schema/catalog/runtime, live RON opt-in, public Wire, server mapping or Unity change entered this Phase.
- The pre-existing 229-entry dirty baseline is preserved exactly after excluding P01-owned additions. The test-created
  server log was removed and no user-owned path was restored, reset, cleaned or reformatted wholesale.
- Independent exit review is `Complete / High`; P02 may become ready only through the Master registry transition.
