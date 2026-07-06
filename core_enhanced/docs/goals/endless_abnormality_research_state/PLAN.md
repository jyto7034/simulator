# Endless Abnormality Research State Plan

## Objective

Implement the run-local abnormality response/research state for Endless mode.

This goal introduces the state model that tracks each suppression-target abnormality's response progress, response completion, suppression wins, first unique skill fragment grant, and repeat-complete fragment dust reward eligibility. It does not implement PveEncounter bonus objectives, repeat encounter weighting, or boss omen chains.

Status: complete on 2026-06-26.

## Source Of Truth Order

Verify facts in this order during implementation:

1. Runtime run state, battle completion, reward granting, snapshot, and checkpoint code.
2. Live RON/data for abnormalities, encounters, rewards, skill fragments, and run policy.
3. Unity-facing snapshot/command contracts and live WebSocket JSON.
4. Latest policy docs, especially `docs/endless_mode_research_policy_draft.md`.

Do not trust this plan over runtime behavior. If code reading shows a better long-term model, record the evidence in `EXPERIMENT_NOTES.md` and apply it unless it requires a user policy decision.

## Dependencies

- Depends on `docs/goals/run_floor_progression_contract/` for the Floor-centric progression model.
- Can be implemented before PveEncounter bonus objectives by granting only base victory research.

## Current Policy To Implement

- Endless completion is based on suppression-target abnormality response completion.
- Suppression targets include:
  - Elite abnormalities.
  - Normal boss abnormalities.
  - Final boss abnormalities.
- Suppression targets exclude:
  - Tool abnormalities.
  - Corroded employees.
  - Facility entities.
  - Normal wave enemies.
- Response progress is run-local, not account-wide.
- Runtime state stores points and required points; percent is presentation only.
- Default response research required is `100`.
- Default victory research gain is `20`.
- All balance values must be configurable from live RON.
- Response progress caps at required points.
- First response completion grants the abnormality's data-authored unique skill fragment.
- Already response-complete repeat suppression grants fragment dust instead of another unique skill fragment.
- Repeat-complete fragment dust defaults:
  - Elite abnormality: `5`.
  - Normal boss abnormality: `10`.
  - Final boss abnormality: `20`.
- Fragment dust amounts must be live-RON configurable.
- Skill fragment reward ids must be data-authored, not inferred by reverse lookup.
- Current content decision: not every abnormality has a final unique fragment design yet, so live data may use explicit placeholder skill fragments. The reward source remains `AbnormalityMetadata.response_complete_skill_fragment_id`.

## In Scope

- Add a run-local `RunAbnormalityResearchState` or equivalent owned by `RunState`.
- Track per-abnormality:
  - `research_points`
  - `research_required`
  - `response_complete`
  - `suppression_wins`
  - `last_encountered_floor`
  - `completed_at_floor`
  - `unique_fragment_granted`
- Persist the state through run checkpoint/save flows where run state is already persisted.
- Initialize suppression-target entries from live abnormality/PvE encounter data.
- Validate that every suppression target can resolve its required research, threat class, and first completion reward policy.
- Grant base victory research on successful suppression encounters.
- Detect the first response-complete transition.
- Grant the unique skill fragment once on first response completion.
- Grant fragment dust on later successful suppression of already response-complete abnormalities.
- Expose enough snapshot data for Unity to show run-local abnormality response progress if a DTO surface already exists or is added in this goal.
- Add focused tests for state mutation, rewards, cap behavior, and checkpoint persistence.
- Update docs/probes after runtime shape is final.

## Out Of Scope

- PveEncounter bonus objectives such as `ClearWithin` or `DecisiveDamage`.
- Abnormality part presentation.
- Repeat encounter candidate weighting.
- Boss omen/provisional boss chains.
- Account-wide/permanent abnormality encyclopedia research.
- Tool abnormality event/research systems.
- New abnormality content.
- Unity UI implementation.
- Compatibility layers that infer reward fragments from `SkillFragmentOrigin`.

## Target State Shape

Exact Rust names may change after code reading, but the long-term meaning should be:

```rust
RunState {
    abnormality_research: RunAbnormalityResearchState,
}

RunAbnormalityResearchState {
    entries: HashMap<AbnormalityId, RunAbnormalityResearchEntry>,
}

RunAbnormalityResearchEntry {
    research_points: u32,
    research_required: u32,
    response_complete: bool,
    suppression_wins: u32,
    last_encountered_floor: Option<u32>,
    completed_at_floor: Option<u32>,
    unique_fragment_granted: bool,
}
```

## Data Policy

Add or verify live RON can express:

- default response research required;
- default victory research gain;
- repeat-complete fragment dust by abnormality threat class;
- `response_complete_skill_fragment_id` on abnormality metadata or an explicit encounter override.

Missing or ambiguous unique fragment references are validation errors when an abnormality is expected to grant a unique fragment.

## Implementation Sketch

1. Audit current abnormality metadata, PvE encounter, reward, skill fragment, and run state code.
2. Decide the exact owner and module layout for `RunAbnormalityResearchState`.
3. Add live RON policy fields and validation.
4. Initialize run-local research state at run creation or first encounter resolution.
5. Wire combat victory resolution to update research state.
6. Route rewards through the canonical grant/reward executor where possible.
7. Add snapshot DTO if needed for user-visible progress.
8. Add focused tests around base victory research, first completion, repeat dust, cap behavior, and persistence.
9. Run broader checks and update docs/probes.

## Completion Criteria

- [x] Run-local abnormality response state exists and is owned by `RunState`.
- [x] Suppression target inclusion/exclusion policy is enforced by validation or deterministic initialization.
- [x] Victory grants base research using RON-configured values.
- [x] Response completion caps progress and records completion metadata.
- [x] First completion grants exactly one unique skill fragment.
- [x] Repeat completion suppression grants fragment dust, not duplicate unique fragments.
- [x] State persists through existing run checkpoint/save flows.
- [x] Focused tests cover visible behavior and reward outcomes.
- [x] Local docs are updated after final runtime shape is known.
- [x] Unity-facing contract docs describe `RunSnapshotDto.abnormality_research`.

Implementation summary:

- Added `RunAbnormalityResearchState` and per-abnormality entries.
- Added live RON `run/policy.ron` abnormality research balance values.
- Added explicit `response_complete_skill_fragment_id` to abnormality metadata and validation.
- Added placeholder response-completion fragments for abnormalities whose final fragment design is not settled yet.
- Added `RunSnapshotDto.abnormality_research`.
- Wired Endless combat victory to grant base research, first-completion skill fragments, repeat-complete fragment dust, and `RunComplete` when all response targets are complete.
- Updated the Unity-facing snapshot contract so clients treat `abnormality_research` as presentation/progress state, not as a local reward-computation source.

## Stop Conditions

Stop and report questions instead of deciding silently if implementation discovers:

- Existing abnormality or encounter data cannot identify suppression targets cleanly.
- Unique skill fragment reward ids are missing for expected reward abnormalities.
- Reward granting cannot be routed through the existing canonical grant path without policy changes.
- Snapshot DTO shape for research progress requires fields beyond the documented `abnormality_research` contract.
- Save migration for existing runs is required.
- Final boss completion semantics conflict with Standard/Endless progression rules.
