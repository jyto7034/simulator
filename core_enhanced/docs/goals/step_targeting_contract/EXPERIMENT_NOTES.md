# Step Targeting Contract Notes

## Runtime Findings

- `StepTargetingMode::ReuseCastTarget` is the default.
- In `resolve_skill_step_context`, `ReuseCastTarget` reuses the cast-level target for enemy/cast-target steps.
- For an enemy unit target, reuse still checks that the target is alive, enemy-owned, and compatible with `air_capable`.
- For `SkillTarget::CastTarget`, reuse can preserve a unit target, preserve a tile target, or fall back to `cast_target_anchor_position`.
- `StepTargetingMode::RetargetOnStep` calls `resolve_skill_target_definition` again using that step's `target`, `range_units`, `defense_tile_range`, and `air_capable`.

## Responsibility Split

- `SkillDef.cast_targeting`: determines the cast-level anchor/target before scheduled steps run.
- `SkillStepDef.target`: declares what this step wants to affect or anchor to.
- `SkillStepDef.targeting`: decides whether this step reuses the cast target or resolves a fresh step target.
- `SkillStepDef.defense_tile_range`: DefenseRoute range/tile pattern source of truth.
- `DeliveryDef::TileArea`: applies effects to units inside the already selected tile pattern; it does not define a separate geometric range.

## Follow-Up Candidates

- The runtime step-targeting methods still live in `sim.rs`. A future skill runtime refactor could move them into a dedicated skill targeting module if `sim.rs` continues to carry too many responsibilities.

## Policy Questions

- None found. Current runtime, tests, and live data agree enough for a documentation-only goal.
