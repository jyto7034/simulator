# Visual Novel Authoring And Local Presentation

Goal: G19

Status: complete

Implementation complete: true

Updated: 2026-07-14

## Objective

Remove duplicate global Visual Novel blocking authoring and prove that every live VN presentation
identity, including Endless pre-run briefing, resolves through one exact Unity-local catalog without
Event fallback or silent placeholder presentation.

## Required Resume Read Set

- `../cross_layer_contract_hardening_master/STATUS.md`
- active Phase 3 `PLAN.md` and `EXPERIMENT_NOTES.md`
- this file, `EXPERIMENTS.md`, and `EXPERIMENT_NOTES.md`
- `../illustrated_employee_check_event_flow/COMPLETION_REVIEW.md`
- Phase 2 `COMPLETION_REVIEW.md`
- Master DCA-M-11/DCA-L-01 assignments and the exact audit sections for those findings
- current VN definition/trigger schema, live VN/trigger RON, runtime activation/session projection,
  Unity local catalog/overlay, live-flow tests, and canonical VN documents

## Fixed Policy

- `VisualNovelTrigger.blocking` is the sole live authoring source for session blocking policy.
- Remove unused duplicate `VisualNovelDefinition.default_blocking`; do not connect it as a fallback,
  alias it, or preserve a compatibility window.
- Event is a separate typed route fixed by G03. Global VN must not read Event panels or restore an
  Event-origin trigger source.
- Unity-local content is keyed by the exact live `script_id`/presentation ids. Every trigger-reachable
  live VN script id must resolve; missing content fails visibly/closed instead of rendering silent
  `"..."`. Dormant unreferenced definitions are not silently treated as active presentation.
- Align existing live ids and local resources only. Do not invent new gameplay, branching, balance,
  trigger, speaker, or blocking policy.

## Scope

- Core VN definition schema, strict RON parsing/validation, live VN RON, tests, and canonical docs.
- Existing trigger-owned blocking runtime and exact source/id projection.
- Unity `VisualNovelLocalCatalog`, overlay/composer error behavior, mocks/tests, and explicit Standard
  plus Endless live Rust-server proof where practical.
- Legacy/default/fallback/alias/ignored-test searches and Phase handoff evidence.

## Non-Scope

- New VN scenes, dialogue writing, artwork, audio, choices, triggers, or presentation modes.
- Event DTO/commands/panels/employee-check behavior.
- Reworking battle/map/run-start VN queue, repeat, priority, or checkpoint policy.

## Validation

- Strict Core parsing tests prove removed `default_blocking` is rejected and trigger blocking still
  controls projected sessions.
- Live bundle and focused run-start trigger tests cover Standard and Endless exact ids.
- Unity tests enumerate all trigger-reachable live script ids, reject missing local resources without
  placeholder fallback, and render both briefing modes.
- Actual Rust-server Standard/Endless flows where applicable.
- Core/server all-targets reuse only if scoped production fingerprints remain unchanged; otherwise
  rerun. Unity focused/full EditMode and needed PlayMode, formatting, canonical, legacy, alias,
  fallback, ignore, and zero-test accounting.
- Completion review guide before R03 unlocks.

## Exit Gate

Complete / High / no immediate correction with trigger-only blocking authority, exact complete live
Unity-local VN presentation coverage, no silent placeholder fallback, and no reopened Event/VN seam.

Result: Complete / High / no immediate correction. See `COMPLETION_REVIEW.md`.
