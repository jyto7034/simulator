# Global Target Area Delivery Contract Completion Review

Goal: G09

Review date: 2026-07-14

Guide: `docs/goal_completion_review_guide.md`

Primary verdict: Complete

Long-term fit: High

Improvement class: none

Immediate correction: none

## Item Audit

| Item | Integrated result | Verdict | Fit | Correction |
| --- | --- | --- | --- | --- |
| Acquisition authority | cast `reach` and `target_range` select one unit; Global never means whole-field damage | Complete | High | None |
| Delivery authority | every TileArea owns one required concrete footprint | Complete | High | None |
| Captured anchor | target death/withdrawal preserves the selected tile and other eligible occupants may be hit | Complete | High | None |
| Geometry parity | usefulness, projection, clipping, telegraph tiles, and execution share Core primitives | Complete | High | None |
| Cross-layer DTO | catalog v2 separates reach from footprint and Unity reads it strictly | Complete | High | None |
| Compatibility cleanup | step range/air, mixed WholeField/TileArea, implicit reach/air, and wave placeholders are absent | Complete | High | None |

## Expected Source Of Truth

`SkillDef.cast_targeting` owns target kind/rule, explicitly authored acquisition reach and air
eligibility, and the optional Pattern target range. `DeliveryDef::TileArea.footprint` independently
owns actual affected tiles. Core captures the selected target tile and is the sole geometry and
hit-eligibility authority; Unity consumes final projected cells/events.

## Schema And Content Evidence

- Live raw schema requires `reach` and `air_capable`; Pattern enemy targeting requires exactly one
  resolved target range, while Global rejects a competing range and Self target.
- `SkillStepDef` and its raw loader contain no range or air authority. TileArea requires a validated
  concrete footprint and an anchor-compatible origin.
- Target/impact-centered footprints must be rotation-invariant until an explicit orientation
  policy exists. Missing, legacy, dual, empty, and asymmetric invalid shapes fail strict tests.
- All 25 live skills explicitly author reach/air authority and all 21 TileArea steps own their
  footprint. There is no live Global skill requiring compatibility; the exact synthetic contract
  test covers the accepted stationary-final-boss semantics.

## Runtime Evidence

- A no-facing stationary caster selects a far valid unit with Global reach, projects a symmetric
  footprint at the captured target tile, and hits an eligible bystander rather than caster/field
  space.
- The same scenario succeeds after the selected unit withdraws and after it dies; `SkillAreaDeclared`
  affected tiles match the execution set.
- Reused direct targets do not reapply acquisition range, while `RetargetOnStep` uses the cast-level
  reach/range/air contract. Edge/void clipping and airborne eligibility remain Core-owned.

## Cross-Layer Evidence

- Core `skill_catalog.version` is 2 and projects cast `reach`/`target_range` separately from
  TileArea `footprint`; exact Rust snapshot assertions pin the shape.
- game_server forwards the typed snapshot unchanged and passes its full suite.
- Unity's strict catalog reader rejects v1, unknown/legacy step authority, Global plus a range, and
  TileArea without a footprint. Runtime presentation uses Core final cells and affected-tile events.
- The future caster-centered expanding field wave is documented as a follow-up contract only; no
  Rust, RON, snapshot, server, or Unity variant exists.

## Validation Review

- Focused Core schema/runtime/live-catalog tests passed, including missing reach/air, mixed-source,
  captured-anchor disappearance, facingless Global, clipping, air, and parity cases.
- Broad Core: 750 passed, 0 failed, 0 ignored. Broad server: 30 passed, 0 failed, 0 ignored.
- Unity EditMode: 215/215. PlayMode: 52 passed, 0 failed, four explicit environment-dependent
  live-server skips. Final Unity console contained zero errors.
- Formatting, Core/server diff checks, scoped Unity/new-file whitespace, live explicit-field counts,
  ignored-test, legacy authority, mixed schema, and unimplemented-wave searches passed.

## Long-Term Direction Review

- Fit: High.
- Improvement class: none.
- Reason: acquisition reach and delivery geometry now have distinct strict owners across live data,
  runtime, DTOs, Unity consumption, tests, and canonical docs without preserving a dual contract.

## Immediate Correction Candidates

None.

## Follow-Up Refactor Candidates

None inside G09. A future expanding field wave requires its own implementation goal and intentional
schema/transport revision.

## Policy Questions

None.

## Review Conclusion

G09 is Complete / High with no immediate correction. G13 and G18 may begin.
