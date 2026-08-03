# Illustrated Employee-Check Event Flow

Goal: G03

Status: complete

Implementation complete: true

Updated: 2026-07-14

## Objective

Replace the provisional Event/VN-like contract with one Core-owned illustrated employee-check Event
state machine and complete player flow across live RON, strict validation, Core, game_server, Unity,
tests, and canonical documents.

## Primary Findings And Policy

- DCA-H-05/DCA-H-06: live Event content exists but Unity has no supported typed route/command flow
  and the documented Event shape does not match runtime.
- Accepted PD-03: Event is supported current functionality and must be a first-class persistent
  Core state machine, not Raw JSON, Unknown route, or Visual Novel fallback.
- Accepted PD-04: one illustrated DTO uses ordered image/text panels, tagged advance versus
  employee-check choices, Core eligibility, scene-qualified resolve, and a persisted raw d20 result.

## Required Resume Read Set

- `../cross_layer_contract_hardening_master/STATUS.md`
- active Phase 3 `PLAN.md` and `EXPERIMENT_NOTES.md`
- this file, `EXPERIMENTS.md`, and `EXPERIMENT_NOTES.md`
- `../unity_wire_identity_alignment/COMPLETION_REVIEW.md`
- Phase 2 `COMPLETION_REVIEW.md`
- Master PD-03/PD-04 detailed contract and the exact DCA/CPR Event findings
- current Event RON/schema/loader, world event/session/node flow, behavior/DTO/server mapping,
  Unity store/router/model/composer/handler/mock/tests, and canonical Event documents

## Accepted Player Flow

1. Enter Event and show ordered panels, each with an image id and ordered text ids.
2. Local click reveals current text, then advances local text/panel cursor without Core mutation.
3. After pre-choice panels, show authored checked choices and Core-projected eligible employees.
4. Local choice/employee selection enables progress; progress sends one scene-qualified resolve.
5. Core atomically validates identity/eligibility, rolls pure 1d20 once, persists choice/performer/raw
   roll/outcome, and commits exactly one success action or failure presentation.
6. Natural 1/20 are critical failure/success; 2..19 use `raw_roll >= required_roll`; no modifiers,
   advantage, reroll, or employee swap after resolution.
7. Success may grant the authored action or start the existing battle pipeline. Failure is a normal
   outcome and completes only after failure panels and the final advance.
8. Reconnect, duplicate delivery, checkpoint, and Event-started combat retry/abandonment return the
   same committed result and never reroll.

## Canonical Inner DTO

- `event_id`, `current_scene_id`
- `presentation.panels[] { image_id, text_ids[] }`
- tagged `interaction`:
  - `advance`, or
  - `employee_check_choices` with choices, `required_roll`, choice eligibility, and employee
    candidates with per-payload eligibility
- optional `committed_check { choice_id, employee_uuid, raw_roll, required_roll, outcome }`
- root `selected_event.event` and `EventState.event` remain the same inner DTO established by G02

## Ownership And Atomicity

- Core owns Event/node/scene identity, eligibility, RNG, committed check, effect marker, pending
  combat, failure presentation, and completion.
- Unity owns only local presentation cursor and uncommitted selection.
- game_server forwards typed envelopes and preserves normal command-result then snapshot/battle-side
  ordering; it performs no Event JSON repair.
- Live bundle validation rejects invalid ids, empty panel/text content, roll bounds, graph defects,
  unsupported/multiple actions, and broken encounter/item/reward references before publication.
- Event-started battle reuses Phase 2 result/attempt/`RetreatBattle`/BossOmen boundaries. The Event
  check remains committed and cannot be rerolled.

## Required Removal

- Event speaker/portrait/dialogue staging and singular/flat provisional presentation fields.
- bare selectable boolean, preview/hint/modifier fields, Raw parsing, dual command/schema paths.
- Event-to-Visual-Novel runtime trigger/fallback authority.
- unsupported/panic-only Event effects, implicit Event ids, and duplicate Event-specific combat
  reward policy.

## Non-Scope

- Event frequency/rarity/repeat rotation, new balance modifiers, alternate dice, rerolls, or new
  reward semantics.
- Global VN authoring cleanup owned by G19 beyond removing Event dependency.
- Phase 4 persistence/meta state and G05/G22 deferred content.

## Validation

- Core unit/flow tests for 1/20, threshold, no modifiers, one RNG draw, duplicate idempotence, stale
  identities, eligibility, atomic success/failure, reconnect/checkpoint, and Event-started combat.
- Strict live RON positive/negative bundle tests.
- game_server serialization/ordering tests.
- Unity exact reader/sender, local reveal/selection, route/composer/handler, full EditMode/PlayMode,
  and explicit live Rust-server flow where applicable.
- Broad Core/server all-targets, formatting, scoped diff, canonical, old-field/VN/fallback/alias,
  ignored-test, and zero-test accounting.
- Completion review guide before G19 unlocks.

## Exit Gate

Complete / High / no immediate correction with a real end-to-end Event player flow and no old
Event/VN or duplicate authority path.

Result: Complete / High / no immediate correction. See `COMPLETION_REVIEW.md`.
