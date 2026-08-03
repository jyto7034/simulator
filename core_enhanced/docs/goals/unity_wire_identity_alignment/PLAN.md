# Unity Wire Identity Alignment

Goal: G02

Status: complete

Implementation complete: true

Updated: 2026-07-14

## Objective

Align ArkClient and canonical Unity documentation to existing typed Rust identities, and establish
the single shared `selected_event`/`EventState` envelope that G03 will extend. Remove stale field
keys, reader names, result names, examples, and identity fallbacks without compatibility aliases.

## Primary Findings

- DCA-H-09: Unity item command sends a field key that differs from the typed Core request.
- DCA-H-10: Unity reward and recruitment readers use identities that differ from Core.
- DCA-M-14: Unity skill catalog example differs from the typed DTO.
- DCA-M-15: Unity documentation names nonexistent `ActComplete` result.
- DCA-M-16: combat-result abnormality identity names diverge.
- DCA-L-02: root snapshot example omits active typed fields.
- DCA-L-03: Unity reads `display_name` while Core emits `name`.
- PD-04 shared boundary: root `selected_event.event` and `EventState.event` use one canonical inner
  DTO; G02 owns generic envelope/document alignment, while G03 owns the new Event inner behavior.

## Required Resume Read Set

- `../cross_layer_contract_hardening_master/STATUS.md`
- `../cross_layer_contract_hardening_master/phases/phase_03_event_vertical_slice/PLAN.md`
- `../cross_layer_contract_hardening_master/phases/phase_03_event_vertical_slice/EXPERIMENT_NOTES.md`
- this file
- `EXPERIMENTS.md`
- `EXPERIMENT_NOTES.md`
- `../cross_layer_contract_hardening_master/phases/phase_02_combat_transaction/COMPLETION_REVIEW.md`
- the exact DCA/CPR sections for the seven primary findings

## Scope

- Trace each Rust request/DTO/result identity through game_server forwarding and actual Unity
  reader/sender/presenter/tests/docs.
- Correct existing identity mismatches at the consumer or canonical producer owner selected by the
  accepted typed contract.
- Make the shared Event envelope structurally identical across root snapshot and command result,
  but do not implement G03 panels/check choices/d20/effects.
- Remove stale examples/names and add behavior-level or exact-wire regressions.
- Update canonical Core/Unity contract documents.

## Non-Scope

- Event authoring schema, scene graph, panels, employee checks, roll semantics, success/failure
  actions, or Event screen implementation.
- New item/reward/recruit/skill/combat-result gameplay behavior.
- Compatibility aliases, dual readers, Raw-JSON fallback, save migration, or live balance/content.
- G05/G22 deferred content.

## Validation

1. Run focused red tests for every reproducible mismatch before correction.
2. Validate Core DTO serialization and game_server forwarding where identities cross Rust layers.
3. Run focused Unity exact-wire/reader/presenter tests, full EditMode, and ordinary PlayMode.
4. Run Core/server affected and all-target suites, formatting, scoped diff, canonical, alias,
   fallback, and ignored-test searches.
5. Apply `docs/goal_completion_review_guide.md` before marking complete.

## Exit Gate

Complete / High / no immediate correction, with all seven findings closed or evidence-backed Not
applicable and a precise G03 handoff for the accepted Event inner DTO cutover.
