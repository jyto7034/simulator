# Enemy Range Preview Removal

Goal: G11

Status: complete

Implementation complete: true

Updated: 2026-07-14

## Objective

Implement accepted PD-08 by removing enemy range-preview projection and Unity consumption from the
current game without changing employee placement/allied previews or active action telegraphs.

Primary finding/policy: DCA-H-03, accepted PD-08.

## Fixed Decision

- Remove enemy `range_previews` from normal and checkpoint/reconcile battle projection.
- Remove Unity enemy preview parsing, state, rendering, interaction, mocks, tests, and canonical
  promises.
- Do not leave empty/reserved compatibility fields, aliases, or client fallbacks.
- Preserve employee deployment/facing preview queries, deployed allied preview needed for player
  interaction, actual executing actions, explicit telegraphs, projectile/area presentation,
  battlefield hazards, and visible wave boundaries.
- A future enemy preview is a new explicit DTO/UX decision, not restoration of a dormant field.

## Validation

- Trace every Core construction/serialization, server assumption, Unity reader/store/render path,
  mock/fixture, and document before editing.
- Exact DTO tests prove opponents cannot carry range preview while required allied/deployment
  previews remain intact.
- Core/server/Unity focused and broad suites, formatting, scoped diff, removed-field/alias/fallback /
  ignored-test searches, then completion review before G10.

## Exit Gate

Complete / High when no normal/checkpoint enemy preview contract remains, preserved player-facing
preview/telegraph surfaces still work, and no compatibility placeholder or local reconstruction is
left.

Result: Complete / High / no immediate correction. See `COMPLETION_REVIEW.md`.
