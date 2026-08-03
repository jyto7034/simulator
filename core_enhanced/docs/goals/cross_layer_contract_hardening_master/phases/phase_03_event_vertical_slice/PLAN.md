# Phase 03 Illustrated Employee-Check Event Vertical Slice

Status: complete

Implementation complete: true

Updated: 2026-07-14

## Objective

Deliver one canonical illustrated Event flow across Core, live data, game_server, Unity, tests, and
documentation while keeping global Visual Novel and the fixed Phase 2 battle transaction separate.

## Required Resume Read Set

- `../../STATUS.md`
- this file
- `EXPERIMENT_NOTES.md`
- the active Child's three documents
- `../phase_02_combat_transaction/COMPLETION_REVIEW.md`
- the active Child's direct predecessor completion review once one exists

## Child Order And Status

| Child/review | Status | Review | Role |
| --- | --- | --- | --- |
| G02 `unity_wire_identity_alignment` | complete | Complete / High | shared selected-event and existing wire identity alignment |
| G03 `illustrated_employee_check_event_flow` | complete | Complete / High | complete illustrated employee-check Event state machine and flow |
| G19 `visual_novel_authoring_and_local_presentation` | complete | Complete / High | remove duplicate VN authoring authority and prove separation |
| R03 `cross_layer_phase_03_event_completion_review` | complete | Complete / High | independent Phase integration review |

## Fixed Predecessor Boundary

- Phase 1 owns UUID/revision/capability/strict-loader authority.
- Phase 2 owns employee attempt condition, result commit, exact `RetreatBattle` restoration,
  BossOmen Gate/load, and Unity recovery consumption.
- Event-started combat must reuse those boundaries without an Event-specific transport, reward,
  retry, rollback, or recovery path.

## Accepted Event Direction

- One typed Event DTO uses ordered image/text panels and tagged advance versus employee-check
  interaction.
- Core owns session identity, eligibility, one d20 roll, committed performer/result/effects, and
  recovery. Unity owns only local reveal/cursor and uncommitted selection.
- Event is not Visual Novel presentation and has no VN fallback.
- G02 aligns the shared envelope and existing wire identities first; G03 performs the coordinated
  Event schema/runtime/Unity cutover; G19 removes remaining duplicate VN authority afterward.

## Exit Gate

R03 must independently return Complete / High with no immediate correction before Phase 4 begins.
