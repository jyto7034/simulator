# Illustrated Employee-Check Event Flow Notes

Goal: G03

Status: complete

## Fixed Predecessors

- G02: one Event inner DTO across root/result, exact generic identities, no aliases.
- Phase 1: typed capability/eligibility, strict bundle validation, revision/transport authority.
- Phase 2: committed Event check precedes battle entry; whole-stage abandonment restores only the
  battle entry state and never rerolls the Event.

## Non-Negotiable Decisions

- Pure 1d20, no modifiers or dormant extension fields.
- Exactly one check attempt and one committed outcome.
- Choice plus employee are local until progress is pressed.
- Failure is gameplay state, not command error.
- Event is not Visual Novel and has no VN fallback.
- One coordinated schema cutover; no old/new aliases or compatibility window.

## Discovery Questions

- Which current Event session fields can be retained as authoritative identities and commit markers?
- Which current immediate/staged effects are live-authored versus removable prototype surface?
- How should the one accepted success action be represented without duplicating reward/battle
  pipelines?
- Which Unity presentation primitives can host Event panels without coupling to VN cue state?
- What exact checkpoint/reconnect tests are needed before removing old Event-to-VN triggers?

## Current Inventory Judgment

- Reuse: Event/node/scene typed ids, persisted run-local session map, normal reward grant executor,
  normal battle start/result pipeline, shared capability/eligibility types, and G02 envelope.
- Replace: flat presentation, preview/selectable choice DTO, `SelectEventChoice`, split
  immediate/staged/reward-commit effect model, and Event-to-VN trigger calls/source.
- Add: selected employee, raw roll, required roll, outcome, once-only success marker, failure
  presentation state, scene-qualified resolve command, and pure Event payload validators.
- Current live choices have no authored success effect. Do not silently invent a White Night
  combat/reward/balance outcome. Apply the accepted rule that only meaningful success/failure
  branches roll: migrate this no-effect live story to advance-only panels while exercising the full
  employee-check contract through strict authored fixtures and actual Rust-to-Unity flow tests.
- Event command families must enter the same common capability/preflight owner established in
  Phase 1; adding only a Unity route would preserve an execution-authority bypass.
- Initial performer eligibility is intentionally minimal: roster membership plus living employee
  state. Combat availability, trauma, trust, equipment, and other stats do not enable/disable or
  modify the pure d20 check.

## Current Implementation Judgment

- Core now has one typed, persistent employee-check Event state machine and one common capability
  authority path. The old Event/VN and split effect/reward owners are removed from runtime.
- Live White Night remains deliberately advance-only because its previous choices had no gameplay
  action; employee-check behavior is exercised through strict authored fixtures rather than an
  invented live balance decision.
- Core, live data, game_server transport evidence, Rust serialization fixture, Unity typed route /
  reader/composer/commands/rendering, canonical documents, and focused/broad validation are now
  implemented.
- Completion review found and corrected one fail-closed Unity scalar/array parsing gap. No immediate
  implementation correction remains; write the formal review and only then change G03 status.
