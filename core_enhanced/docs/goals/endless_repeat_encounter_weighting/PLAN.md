# Endless Repeat Encounter Weighting Plan

## Objective

Implement Endless encounter candidate weighting using run-local abnormality research state and recent Floor encounter history.

This goal controls how often the same abnormality can reappear through the normal non-omen encounter assignment path. It does not implement the research state itself, PveEncounter bonus objective evaluation, or boss omen chains.

## Source Of Truth Order

Verify facts in this order during implementation:

1. Runtime map generation and encounter assignment code.
2. Run-local abnormality research state and Floor progression code.
3. Live RON/data for encounter pools, abnormalities, and map generation policy.
4. Unity-facing snapshot contracts if encounter identity is exposed.
5. Latest policy docs.

Do not trust this plan over runtime behavior. If code reading shows a better long-term model, record the evidence in `EXPERIMENT_NOTES.md` and apply it unless it requires a user policy decision.

## Dependencies

- Depends on `docs/goals/run_floor_progression_contract/`.
- Depends on `docs/goals/endless_abnormality_research_state/`.
- Can be implemented before boss omen chains.

## Current Policy To Implement

- The same abnormality may appear multiple times in the same Endless run.
- Victory means the abnormality was driven off in that encounter, not permanently removed.
- The same abnormality must not reappear through normal non-omen selection within the last `1 Floor`.
- The recent-Floor exclusion applies regardless of response completion.
- After the exclusion window, normal candidate weighting applies.
- Response-complete abnormalities use reduced normal non-omen candidate weight.
- Initial response-complete normal non-omen candidate weight multiplier is `0.25`.
- Incomplete abnormalities may keep normal weight or receive a slight weight increase.
- Special chains, strengthened/awakened versions, and final boss conditions may bypass normal non-omen candidate weighting.
- All weighting values and exclusion windows must be live-RON configurable.
- If soft repeat-weighting constraints empty the candidate pool, reset only those soft constraints and choose from the original hard-constrained pool.
- Hard constraints are never relaxed by the empty-pool fallback: encounter class, map node category, map node kind, game mode, Standard final boss policy, and boss/elite/special chain policy.
- Soft repeat-weighting constraints that may be reset are recent Floor exclusion, response-complete weight reduction, repeated encounter suppression, and future soft preference/avoidance weights.

## In Scope

- Audit current encounter selection in `map_encounters.rs`.
- Keep Phase 4A's difficulty-free hard candidate selection and add Floor-aware repeat policy on top.
- Use `RunAbnormalityResearchState` to check:
  - response completion;
  - suppression wins if needed.
- Use separate run-local abnormality encounter history to check last appeared Floor.
- Add live RON policy for:
  - recent Floor exclusion window;
  - response-complete weight multiplier;
  - optional incomplete weight multiplier.
- Apply weighting only to the normal non-omen encounter assignment path and only when candidates have a primary abnormality identity.
- Ensure special chains/final conditions remain out of this repeat-weighting path.
- Add deterministic tests for candidate exclusion and weighting.
- Add live RON validation tests.

## Out Of Scope

- Boss omen/provisional boss chains.
- Strengthened/awakened abnormality generation.
- New encounter content.
- Bonus objectives.
- Research state mutation.
- Unity rendering/UI.

## Selection Policy

Normal non-omen Endless candidate selection should follow this conceptual order:

1. Build the hard-constrained candidate set for the node/floor.
2. Remove candidates whose abnormality appeared within the configured recent Floor window.
3. Apply response-complete weight multiplier to completed abnormalities.
4. Apply any incomplete or repeat-preference weight policy.
5. Deterministically choose from weighted candidates.

If soft filtering removes all candidates, do not relax hard constraints. Reset the soft repeat-weighting constraints and choose from the original hard-constrained candidate set.

This fallback is explicit and must be tested. It is not the old silent fallback to all encounters.

## Implementation Sketch

1. Audit `map_encounters.rs` and current candidate fallback behavior.
2. Identify where encounter appearance history is recorded and ensure it is separate from victory-only research progression.
3. Add RON policy fields and validation.
4. Implement a deterministic weighted selection helper.
5. Add tests for recent Floor exclusion, response-complete multiplier, soft-reset empty candidate behavior, and hard-constraint preservation.
6. Update docs/probes if visible DTO behavior changes.

## Completion Criteria

- Endless normal non-omen encounter selection uses Floor-aware recent exclusion when candidates have a primary abnormality identity.
- Response-complete abnormalities have reduced normal non-omen candidate weight.
- All balancing values come from live RON.
- Standard mode behavior remains unchanged unless explicitly intended.
- Candidate selection remains deterministic for a fixed seed/run state.
- Empty candidate fallback resets only soft repeat-weighting constraints and preserves hard encounter/node/mode/final-boss constraints.
- Tests cover the user-visible selection policy and data validation.

## Stop Conditions

Stop and report questions instead of deciding silently if implementation discovers:

- Existing PvE encounter data cannot identify abnormality candidates consistently.
- Difficulty scaling and repeat weighting conflict in a way requiring balance policy.
- The normal/special/final candidate path cannot be separated cleanly.
