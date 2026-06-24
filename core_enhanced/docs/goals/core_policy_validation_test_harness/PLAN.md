# Validation and Test Harness Policy Implementation

## Objective

Make validation and tests enforce the new policies across runtime code, live RON loading, Unity-facing DTO contracts, and gameplay flows.

## Policies Covered

- `remove old timeline log validation acceptance`
- `live content audit manifest`
- Cross-goal final comparison against `POLICY_DECISIONS.md`
- Cross-goal broad validation after focused tests

## Plan

1. Read validation harnesses, fixture loaders, timeline/log validators, live content coverage tests, and CI-style commands.
2. Remove normal validation acceptance for old timeline/log versions 6/7.
3. Move live content roster/coverage from hard-coded Rust lists into a versioned data/audit manifest.
4. Ensure every removed legacy schema/event/fallback from other subgoals is covered by validation failure or updated fixtures.
5. Build final policy coverage checklist against all confirmed headings in `POLICY_DECISIONS.md`.
6. Compare generated subgoal docs with actual code and live data after all implementation subgoals finish.
7. Run focused tests after each final validation change and a broad suite at the end.

## Completion Conditions

- Old timeline/log versions 6/7 are not accepted by normal validation.
- Live content audit coverage is versioned data, not a hard-coded Rust roster.
- Every policy implementation subgoal has validation evidence or documented risk.
- Final cross-check finds no undocumented drift between policies, docs, code, live data, and tests.
- Final report includes changed contracts, removed legacy, updated tests, risks, and commands run.
