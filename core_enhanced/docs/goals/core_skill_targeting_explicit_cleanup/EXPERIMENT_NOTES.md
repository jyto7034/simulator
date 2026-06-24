# Experiment Notes

## Confirmed Policy

- Remove `SkillStepDef.range_units`.
- Do not reuse `SkillStepDef.range_units` for contagion, chain, contact, aura, or continuous mechanics.
- Add future distance limits as targeting/delivery-specific fields.
- Remove `SkillCastTargetingDef::FirstStepTarget`.
- Cast-level target and step execution target must both be explicit.
- Helpers/builders may exist, but they must not infer cast targeting from steps.

## Policy Questions To Watch

Record `사용자와 정책 논의 필요` and complete the goal if work discovers any item below. Do not pause, block, or leave the goal active; a policy-decision report is the completed deliverable.

- a new live RON schema shape beyond removing existing legacy fields,
- a need to implement contagion/chain semantics now,
- a Unity-facing skill catalog field rename not already confirmed,
- a gameplay behavior change hidden behind test fixture cleanup.

## Follow-Up Candidates

Record out-of-scope improvements here.

## Implementation Notes

- `SkillCastTargetingDef::FirstStepTarget` was removed; `SkillCastTargetingDef` now has only explicit targeting.
- `SkillStepDef.range_units` was removed from the skill step type.
- `SkillCastTargetingDef::Explicit.range_units` was also removed because runtime target selection already used `range_policy` and `defense_tile_range`; keeping a dead cast-level numeric range would preserve a misleading second source.
- `SkillCastTargetingDef::explicit(target, range_policy, defense_tile_range, air_capable)` was added as an explicit constructor. It does not read or infer from steps.
- Cast-level `air_capable` and step-level `air_capable` are now visibly separate. A focused test was updated to set cast-level `air_capable` when testing airborne cast target eligibility.
- Remaining `range_units` matches in the focused search are basic attack/equipment/abnormality domains or the negative legacy-RON validation test, not `SkillStepDef`.
