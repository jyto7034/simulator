# Skill Contract Example Cleanup Notes

Goal: G18

Status: complete

## Guardrails

- Do not change Rust/C#/RON behavior or DTO shape.
- Do not revive G09 step range/air fields or G10 weapon inheritance language.
- Do not update historical findings to erase their original evidence.
- Treat live `base.ron` and typed Core DTO output as higher authority than prose.

## Immediate work

- Add the missing final-usefulness condition to direct `ReuseCastTarget` wording.
- Replace stale RetargetOnStep examples with all four current live skill/step pairs.
- Describe `FirstStepTarget` as removed legacy and name final constructor arguments.
- Correct the remaining Unity catalog version/example ordering, then run exact searches/tests and a
  completion review.

## Final Findings

- G09 intentionally superseded the original range portion of DCA-M-03: reused direct targets do not
  recheck acquisition range. Execution-time usefulness remains and is now explicit.
- DCA-M-04 was already materially corrected by G09's concrete footprint migration; final searches
  confirm the old step `range_units` example is absent.
- DCA-M-05 and the newly found catalog-version drift are corrected against current live/typed
  sources.
- No runtime, schema, content, or Unity reader correction was necessary.
- Final verdict: Complete / High / no immediate correction.
