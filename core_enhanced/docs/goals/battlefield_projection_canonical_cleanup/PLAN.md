# Battlefield Projection Canonical Cleanup

Goal: G13

Status: complete

Implementation complete: true

Updated: 2026-07-14

## Objective

Close DCA-H-07 and DCA-M-10 by aligning canonical battlefield presentation/identity wording with
the already-coherent runtime boundary, without changing production behavior or activating reserved
content.

## Fixed Boundary

- A lower actor world anchor is input to `Camera.WorldToScreenPoint`; the HUD itself renders in a
  dedicated screen-space overlay. It is not a world-space canvas/mesh or camera billboard.
- `FacilityEntity` remains a wire-reserved future enum shape only. Live PVE validation rejects it,
  combat setup cannot construct it, and no canonical text may call it currently produced.
- Preserve G11's player-only range preview and G09's final reach/footprint DTO boundaries.

## Validation

- Re-read current Core DTO/runtime production, live validation/setup, Unity parser/renderer/tests,
  and all current canonical occurrences before editing.
- Run the existing live PVE reference test and exact HUD PlayMode test, then broad documentation
  searches and scoped whitespace checks.
- Completion review must prove no current canonical billboard claim or produced-Facility claim
  remains.

## Exit Gate

Complete / High when current canonical documents distinguish world-anchor input from screen-space
HUD rendering and wire-reserved Facility identity from current production, with runtime/tests still
green and no behavior/schema expansion.

Exit gate result: Complete / High on 2026-07-14. See `COMPLETION_REVIEW.md`.
