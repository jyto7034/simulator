# Endless Completion Regression

Status: complete

Implementation complete: true

Started: 2026-07-18

Master: `docs/goals/documentation_code_alignment_corrections/PLAN.md` G03

Finding: `DCA2-TEST-001`

## Objective

Protect the existing Endless all-response-complete completion branch with one named, direct
regression that constructs the exact precondition and asserts the exact next outcome.

## In Scope

- Locate the authoritative response-completion branch and its state owners.
- Build an exact all-response-complete Endless state using existing test helpers where possible.
- Assert the immediate completion result and terminal state, including relevant preserved state.
- Run the focused test and the owning module/broad relevant suite.

## Out Of Scope

- Runtime behavior changes unless the direct regression disproves the completed audit finding.
- Standard-mode completion policy changes.
- WhiteNight mechanics or future completion design.
- Broad refactors of map/research test fixtures.

## Implementation Plan

1. [x] Inspect the branch and nearest indirect tests.
2. [x] Identify the minimal authoritative precondition and exact result/state contract.
3. [x] Add one named direct test without changing production behavior.
4. [x] Run focused and broad relevant validation.

## Completion Conditions

- A named test directly constructs all-response-complete Endless state.
- The next completion action is asserted against the exact intended result and state.
- The test does not depend on a long probabilistic playthrough or only indirect branch coverage.
- Relevant focused and broad tests pass.

All completion conditions satisfied on 2026-07-18.

## Result

- `endless_final_response_completion_returns_run_complete_without_planning_delivery` constructs
  every initialized response as complete except one entry exactly one victory gain short.
- The next combat-result completion must return `CombatRewardsGranted` containing
  `BehaviorResult::RunComplete`, transition Core to `GameState::RunComplete`, and make
  `all_response_complete()` true.
- A pre-queued Safe Zone research delivery remains pending, proving the terminal branch does not
  synthesize a planning return.
- Production runtime code was unchanged.

## Stop Conditions

- Direct construction reveals the runtime branch no longer matches the completed audit conclusion.
- Exact intended outcome conflicts with current canonical policy.
