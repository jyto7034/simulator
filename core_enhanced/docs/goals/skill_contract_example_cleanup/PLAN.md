# Skill Contract Example Cleanup

Goal: G18

Status: complete

Implementation complete: true

Updated: 2026-07-14

## Objective

Close DCA-M-03, DCA-M-04, and DCA-M-05 by auditing current canonical skill examples against the
completed G10 selection and G09 reach/footprint/runtime contract.

## Fixed Boundary

- `ReuseCastTarget` direct unit execution does not recheck cast acquisition range, but it still
  requires a live hostile air-compatible target and step usefulness at execution.
- TileArea reuse resolves from the captured tile even after target death/withdrawal.
- Retarget examples must name actual current live skill/step ids.
- All RON/JSON examples use the final strict schema and skill catalog version 2.
- Historical audit and goal evidence remains unchanged.

## Validation

- Compare each current canonical example with Rust schema/runtime, live RON, typed DTO tests, and
  Unity reader.
- Search current canonical documents for stale example ids, version 1, removed fields/variants, and
  old step range/air authority.
- Run the exact reuse-target runtime, strict skill loader, live catalog audit, and Unity catalog
  tests; reuse the immediately preceding broad production gate if no code/data changes occur.

## Exit Gate

Complete / High when current examples describe final execution and schema exactly, name only live
examples, and introduce no compatibility wording or production behavior change.

Exit gate result: Complete / High on 2026-07-14. See `COMPLETION_REVIEW.md`.
