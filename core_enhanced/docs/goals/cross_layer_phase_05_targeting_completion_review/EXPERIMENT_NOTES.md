# Cross-Layer Phase 05 Targeting Completion Review Notes

Review: R05

Status: active

## Guardrails

- Do not trust child status or previous green totals without current source/test inspection.
- Do not implement any discovered material correction inside R05; record and route it.
- Do not reopen G05/G22 deferred content or implement the deferred expanding wave.
- Preserve dirty worktrees and all prior child evidence.

## Review focus

- Composition conflicts between G11 preview removal, G10 selection, and G09 reach/geometry.
- Live data and strict loader agreement with catalog v2 and Unity consumption.
- Deterministic selection/usefulness and projection/execution parity at disappearance/edge/air cases.
- Canonical/examples/evidence semantics after G13/G18/G20.
- Legacy/fallback/ignored-test/debug bypass audit and broad validation.

## R05-H-01 — PlayMode pointer state and failure cleanup are sequence-dependent

- The existing first-test `Mouse.current?.WarpCursorPosition(Vector2.zero); yield return null;`
  passed in an earlier ordering but does not neutralize the retained hover after R05's focused HUD /
  pending-preview tests.
- Full PlayMode and the exact starter-selection rerun both select Cyra instead of the expected Ada.
- Because the first assertion aborts before normal teardown, later tests observe leaked UI/root state
  and fail in unrelated status/admin/button/map/query assertions.
- This is not a gameplay targeting contract failure, but it invalidates the required broad Unity
  gate and is an immediate correction. G31 must establish deterministic pointer state plus teardown
  that runs even when assertions fail, then pass exact, ordered focused→full, and repeated full runs.

## Initial verdict

- Child/runtime contract items: Complete / High.
- Phase validation composition: Partially complete.
- Improvement class: immediate correction recommended.
- Phase 6 remains blocked pending G31 and R05 correction rerun.

## Correction rerun verdict

- G31's deterministic pointer/root fixture isolation passed its own review.
- R05 independently repeated the original focused→full ordering and an immediate second full run;
  the Ada/Cyra failure and cleanup cascade did not recur.
- All runtime/data/DTO/canonical/evidence items remain Complete / High and no new compatibility or
  source split appeared.
- Current verdict: Complete / High / improvement class none / no immediate correction.
- Phase 6 may begin.
