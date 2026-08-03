# Phase 03 Illustrated Employee-Check Event Vertical Slice Notes

Status: active

## Current Active Child

- R03 `cross_layer_phase_03_event_completion_review`.

## G03 Handoff

- Completion review: `../../../illustrated_employee_check_event_flow/COMPLETION_REVIEW.md`.
- G03 is Complete / High with no immediate correction.
- Event now has one typed panels/employee-check route and no VN trigger/fallback dependency.
- G19 must preserve the Event DTO/commands and only clean global VN authoring/local presentation
  authority.

## G19 Handoff

- Completion review: `../../../visual_novel_authoring_and_local_presentation/COMPLETION_REVIEW.md`.
- G19 is Complete / High with no immediate correction.
- Trigger `blocking` is the only VN blocking authority; Standard and Endless exact local scripts are
  covered, and missing scripts fail closed without placeholder progression.
- R03 must independently trace the combined G02/G03/G19 boundary before Phase 4 unlocks.

## G02 Handoff

- Completion review: `../../../unity_wire_identity_alignment/COMPLETION_REVIEW.md`.
- Root `selected_event.event` and `EventState.event` are one typed seam.
- Existing candidate/reward/item/result/modifier identities are exact and alias-free.
- G03 may replace the Event inner DTO once across Core/server/Unity, but must not reopen unrelated
  generic wire fields.

## Ownership Boundaries

- G02: shared envelope and already-existing wire identity mismatches; no compatibility aliases.
- G03: Event authoring/runtime/session/d20/effect/presentation/Unity end-to-end behavior.
- G19: global VN authoring cleanup and actual VN resource proof after Event no longer depends on it.

## Cross-Phase Guardrails

- Preserve Phase 2 result/abandonment and BossOmen semantics for Event-started combat.
- Preserve Phase 1 capability and strict validation ownership.
- Do not infer Event eligibility, result, or persistence in Unity.
