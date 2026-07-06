# Endless Repeat Encounter Weighting Notes

## Initial Notes

- This goal depends on the Floor progression model because recent encounter exclusion is expressed in Floors.
- This goal depends on abnormality research state because response-complete weighting needs that state.
- Existing `map_encounters.rs` currently uses `act_index` as a difficulty offset. This should already be replaced or stabilized by `run_floor_progression_contract`.
- Do not implement boss omen logic here. Omen chains may intentionally bypass normal candidate weighting later.

## Risks To Check

- Candidate pools may be too small after recent-Floor filtering.
- Current fallback behavior may select from all encounters when preferred candidates are empty. That could silently violate encounter class, node kind/category, game mode, and final boss policy.
- Weighting must remain deterministic for fixed run seed and state.

## 2026-06-27 Runtime Audit

- `src/game/world/map_encounters.rs` is still the normal encounter assignment source.
- Current assignment only receives `PveEncounterDatabase`, `RunMap`, and `RunProgression`; it does not receive `RunAbnormalityResearchState` or `RunPolicyData`, so Phase 4 cannot be implemented without changing this API.
- `RunAbnormalityResearchEntry.last_encountered_floor` exists but is updated on Endless suppression victory. That is too late to represent "appeared" or "entered" encounter history.
- Live run policy has no repeat encounter weighting section yet.
- Live PVE encounter difficulties currently span `1..=7`.
- Current Endless difficulty window uses `difficulty_floor_index * 2` in `map_encounters.rs`. On sufficiently high Endless Floors, the normal difficulty candidate set can be empty before repeat filtering.
- Existing code silently falls back from an empty preferred candidate set to all non-boss encounters, then to all encounters. Phase 4 policy explicitly says empty candidate fallback must not be silent.

Follow-up after Phase 4A:

- Phase 4A removed encounter difficulty from `PveEncounter`, so the difficulty-window blocker is obsolete.
- Current hard constraints are explicit encounter class, map node category/kind, game mode, and Standard final boss policy.
- `PveEncounterClass::Normal` is pure corroded-employee combat and intentionally has no `primary_abnormality_id`.
- Repeat weighting therefore applies to the normal non-omen assignment path only when the hard candidate set contains primary-abnormality candidates, such as Elite or NormalBoss candidates.
- Pure corroded Normal candidates keep deterministic hard-constrained selection because there is no abnormality identity to weight.

## Empty-Pool Fallback Policy

Resolved user decision:

If recent/repeat soft constraints empty the candidate pool, reset only those soft constraints and choose from the original hard-constrained pool.

Hard constraints that must not be reset:

- encounter class;
- map node category;
- map node kind;
- game mode;
- Standard final boss policy;
- boss/elite/special chain policy.

Soft repeat-weighting constraints that may be reset:

- recent Floor exclusion;
- response-complete weight reduction;
- repeated encounter suppression;
- future soft preference/avoidance weights.

This fallback is explicit, deterministic, live-RON controlled, and tested. It is not a fallback to all encounters.

## 2026-06-27 Implementation Notes

- Added `RunPolicyData.encounter_repeat_weighting` with:
  - `recent_floor_exclusion_window`;
  - `response_complete_weight_multiplier_percent`;
  - `incomplete_weight_multiplier_percent`;
  - `empty_pool_fallback: ResetSoftRepeatConstraints`.
- `assign_map_encounters` now receives `RunAbnormalityResearchState` and `RunPolicyData`.
- Start-new-game and Floor-advance map generation pass the run-local research state into encounter selection.
- Gate transition now preserves run-local abnormality research when creating the next Floor `RunState`.
- The deterministic selection seed is unchanged; only the candidate pool/weights change for eligible Endless primary-abnormality candidates.
- Follow-up correction: recent Floor exclusion now uses separate run-local abnormality encounter history recorded when a primary-abnormality combat node is confirmed. Research state remains the source for response-complete weighting only.

## Out-Of-Scope Follow-Up Candidates

- Boss omen chains.
- Strengthened/awakened variants.
- Special chain bypass rules.
- Encounter pool authoring improvements.
