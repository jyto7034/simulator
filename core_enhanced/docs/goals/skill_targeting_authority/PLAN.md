# Skill Targeting Authority

Goal: G10

Status: complete

Implementation complete: true

Updated: 2026-07-14

## Objective

Implement accepted PD-07 so every skill explicitly owns its cast-target selection rule while basic
attacks continue to use the equipped weapon's targeting profile.

Primary finding/policy: DCA-H-02, accepted PD-07.

## Fixed Decision

- Basic attacks use the effective weapon/basic-attack `targeting_profile`.
- Every skill explicitly authors its own cast-target selection. Skills may reference an explicit
  reusable selection preset, but omission never means weapon inheritance.
- Missing selection, legacy fallback, simultaneous weapon/skill authority, and compatibility alias
  fail strict live loading.
- Changing a weapon cannot change the equipped skill's target selection behavior.
- Preserve PD-06/G09 ownership of target kind, Global reach, captured target tile, footprint,
  ground/air eligibility, and delivery geometry; G10 changes selection authority only.

## Validation

- Trace raw/live schema, strict validation, all live skills, runtime selection, AI/preflight,
  projection, fixture, and Unity metadata consumers before editing.
- Exact tests prove missing/legacy/dual authority rejection, weapon-independent skill selection,
  reusable preset resolution if retained, and unchanged basic-attack weapon targeting.
- Validate every live skill explicitly and update fixtures/docs without inventing targeting balance.
- Core/server/Unity focused and broad suites, formatting, scoped diff, fallback/alias/ignored-test
  searches, then completion review before G09.

## Exit Gate

Complete / High when one explicit skill-owned selection rule drives every skill cast path, basic
attack weapon targeting remains separate, all live content is explicit, and no inheritance or
compatibility seam remains.

Result: Complete / High / no immediate correction. See `COMPLETION_REVIEW.md`.
