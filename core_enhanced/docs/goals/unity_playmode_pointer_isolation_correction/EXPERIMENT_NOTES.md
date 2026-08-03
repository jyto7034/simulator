# Unity PlayMode Pointer Isolation Correction Notes

Goal: G31

Status: complete

## Guardrails

- Test-only correction; no production C# or UX changes.
- No retry, order attribute, ignore, conditional assertion, or expected-name relaxation.
- Teardown removes only roots created after the per-test baseline.
- Validation must intentionally run the previously contaminating focused tests before full suite.

## Immediate work

- Replace method-local cursor warp with reusable setup neutralization.
- Add guaranteed per-test root cleanup.
- Clear current leaked scene objects once before validation, then prove exact and ordered/repeated
  full suites without manual cleanup between those proof runs.

## Final Findings

- `WarpCursorPosition` was queued/OS-position-sensitive and did not reliably update the control state
  before the UI input module processed hover.
- Immediate `InputState.Change` before and after a frame makes the test's pointer source deterministic.
- Method-local cleanup could never run after an NUnit assertion abort. `UnityTearDown` closes the
  cascade by removing only roots introduced by that test.
- No production code, UX, DTO, or gameplay contract changed.
- Final verdict: Complete / High / no immediate correction.
