# Unity PlayMode Pointer Isolation Correction

Goal: G31

Finding: MSTR-07 / R05-H-01

Status: complete

Implementation complete: true

Updated: 2026-07-14

## Objective

Make `ArkClientMockRenderTests` deterministic across exact, focused-to-full, and repeated full
PlayMode orderings by neutralizing retained Input System pointer state and guaranteeing cleanup even
when a test assertion fails.

## Fixed Boundary

- Preserve starter-selection hover UX and every production runtime/DTO contract.
- Test setup owns a neutral off-screen pointer state before UI construction.
- Test teardown owns removal of every root GameObject created after that test's baseline, even when
  the test coroutine aborts on assertion.
- Do not weaken assertions, reorder tests, add retries, ignore failures, or special-case Cyra/Ada in
  production code.

## Validation

- Reproduce the exact failure before editing (already recorded by R05).
- Run the exact starter test, R05 focused hover tests followed by full PlayMode, then a second full
  PlayMode run to prove order independence.
- Run full EditMode, console/error, scoped whitespace, and no-ignore/no-retry audits.
- Complete an individual G31 review, then append an independent R05 correction rerun.

## Exit Gate

Complete / High when deterministic input and guaranteed teardown replace the local warp patch, all
required orderings pass without retries/skips, and no production behavior changes.

Exit gate result: Complete / High on 2026-07-14. See `COMPLETION_REVIEW.md`.
