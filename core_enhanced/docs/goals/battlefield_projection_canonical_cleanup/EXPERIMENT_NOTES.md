# Battlefield Projection Canonical Cleanup Notes

Goal: G13

Status: complete

## Guardrails

- Do not change Unity HUD rendering code or Core HUD DTO semantics.
- Do not remove the reserved Facility enum/parser or add a facility profile/runtime implementation.
- Do not modify live RON.
- Keep historical goal/audit evidence unchanged; edit only current canonical wording and G13/master
  records.

## Immediate work

- Replace Core billboard wording with the exact world-anchor-to-screen-space projection contract.
- Mark both setup wave and live actor Facility shapes as wire-reserved/future and explicitly state
  that current live validation/setup cannot produce them.
- Verify current canonical searches, existing focused tests, formatting/whitespace, and completion
  review.

## Final Findings

- The lower world anchor was never the HUD render space; it is projection input. The stale Core
  billboard sentence was the entire DCA-H-07 mismatch.
- Facility identity remains present in Rust/Unity wire types so a future intentional revision has a
  known shape, but strict live validation and setup make it unreachable today. The stale
  “production variants” heading was the DCA-M-10 overstatement.
- No implementation correction or new policy was needed.
- Final verdict: Complete / High / no immediate correction.
