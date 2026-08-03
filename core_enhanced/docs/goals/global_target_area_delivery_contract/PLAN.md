# Global Target Area Delivery Contract

Goal: G09

Status: complete

Implementation complete: true

Updated: 2026-07-14

## Objective

Implement accepted PD-06 by separating global unit-target acquisition reach from the required
target-centered `TileArea` delivery footprint and by preserving the selected tile through delayed
resolution.

Primary finding/policy: DCA-H-01, accepted PD-06.

## Fixed Decision

- Cast targeting owns target kind, G10's explicit selection rule, acquisition reach, and ground/air
  eligibility.
- Global reach permits any currently valid authored unit target; it never means whole-field damage.
- `TileArea` owns a required, non-empty concrete footprint projected from the tile captured when
  the cast target is selected.
- Target death or withdrawal before impact does not cancel, relocate, or empty delivery.
- Reused-target steps do not re-author reach or air eligibility. Remove mixed legacy range meaning,
  compatibility aliases, fallbacks, and accepted-but-empty shapes.
- Projection, preflight/AI usefulness, telegraph tiles, and execution share the same geometry and
  hit-eligibility primitives, including edge/void clipping and ground/air filtering.
- Do not implement or expose a schema placeholder for the deferred caster-centered expanding field
  wave.

## Validation

- Trace schema resolution, every live skill, cast selection/revalidation, queued execution state,
  target death/withdrawal cleanup, area anchors, geometry, usefulness, event projection, catalog /
  snapshot DTOs, server forwarding, Unity readers, fixtures, and canonical docs before editing.
- Exact tests cover a stationary no-facing global caster, far target, captured-tile delivery after
  disappearance, other eligible occupants, edge/void clipping, air policy, deterministic selection,
  and projection/execution parity.
- Strict tests reject empty footprint, competing cast pattern, duplicated step authority, legacy
  mixed schema, incompatible anchors/tracking/filter, and any unimplemented wave variant.
- Core/server/Unity focused and broad suites, formatting, scoped diff, fallback/alias/ignored-test
  searches, then completion review before G13/G18.

## Exit Gate

Complete / High when Global is acquisition-only, target-centered TileArea always resolves from an
immutable selected tile with one shared geometry contract, all live/Unity shapes are migrated, and
no mixed legacy or future-wave compatibility surface remains.

Exit gate result: Complete / High on 2026-07-14. See `COMPLETION_REVIEW.md`.
