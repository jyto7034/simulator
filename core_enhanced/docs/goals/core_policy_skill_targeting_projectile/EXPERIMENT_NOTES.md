# Experiment Notes

## Policy Notes

- Basic attack range and skill range may differ; Unity should request the selected skill preview when a skill is hovered/selected.
- Invalid targets caused by state changes may resolve to miss/no-op according to confirmed runtime policy.
- If a skill needs a new targeting concept that is not expressible as tile pattern plus clipping, record `사용자와 정책 논의 필요`.
- `range_units` still exists for basic attack/equipment movement spacing and internal test constructors. This subgoal removed it from live skill RON and Unity skill catalog targeting surfaces.
- Live skill RON uses explicit `cast_targeting` plus tile range presets/patterns. The first-step inference path is no longer accepted by the live loader.
- Public `SkillDef` RON deserialization now requires `cast_targeting`, but the enum `Default` remains available for non-RON test constructors until broader internal cleanup is worthwhile.
- `Battlefield::in_bounds` is the current valid-tile predicate; despite the name, it also checks the authored valid tile set.
- Projectile miss has one canonical event shape: `BasicAttackProjectileImpacted { hit: false }`.
- Projectile launch owner snapshot was not changed; existing test `advance_basic_attack_projectile_uses_launch_side_after_attacker_death` continues to pin the policy.

## Follow-Up Candidates

- Consider renaming or wrapping `Battlefield::in_bounds` so valid-tile clipping intent is less surprising.
- If internal `SkillStepDef.range_units` becomes fully unused for skills, remove it from the internal model in a later cleanup. Current policy-critical surface has already moved to tile preview/pattern data.
