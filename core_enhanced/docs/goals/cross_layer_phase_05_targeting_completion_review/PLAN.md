# Cross-Layer Phase 05 Targeting Completion Review

Review: R05

Status: complete

Implementation complete: true

Updated: 2026-07-14

## Objective

Independently review the integrated G11/G10/G09/G13/G18/G20 targeting, battlefield projection,
canonical example, and evidence-governance boundary before Phase 6 is allowed to start.

## Review Items

- Enemy preview absence with preserved Player deployment/deployed preview.
- Skill-owned explicit selection versus basic-attack weapon profiles.
- Explicit cast reach/air authority versus required TileArea footprint and captured target tile.
- Shared usefulness/projection/telegraph/execution geometry, including no-facing Global, target
  disappearance, edge/void clipping, air eligibility, and deterministic selection.
- Strict Core catalog v2/server forwarding/Unity reader and screen-space HUD consumption.
- Current canonical examples and reserved Facility wording.
- Manifest v2 evidence strength, update lifecycle, and complete 30-fragment derived index.

## Method

- Treat child completion reviews as inputs, not proof.
- Re-read runtime, live RON, DTO projection, server forwarding, Unity readers/renderers, canonical
  docs, and tests in source-of-truth order.
- Search production/current canonical surfaces for opponent preview compatibility, skill weapon
  inheritance, step range/air authority, mixed WholeField/TileArea, implicit reach/air, catalog v1,
  stale examples, billboard claims, produced Facility claims, and behavior-coverage overclaim.
- Run independent focused composition tests first, then full Core/server/Unity suites, formatting,
  diff/ignored-test/console checks.

## Exit Gate

Complete / High with no immediate correction only if every Phase 5 exit condition holds together,
no competing source or compatibility path remains, and broad cross-repository validation passes.

## Review Outcome

- Initial 2026-07-14 review: Partially complete / High runtime-contract fit / immediate test-harness
  correction recommended.
- Runtime/data/server/EditMode items passed. Full PlayMode is sequence-dependent after focused hover
  tests: the first starter-selection test expects Ada but receives pointer-hovered Cyra, and its
  aborted cleanup cascades into seven later failures.
- G31 `unity_playmode_pointer_isolation_correction` is required, followed by an R05 correction rerun.
- G31 passed Complete / High. The independent R05 correction rerun passed focused→full and repeated
  full gates on 2026-07-14.
- Current outcome: Complete / High / no immediate correction. Phase 6 may begin.
