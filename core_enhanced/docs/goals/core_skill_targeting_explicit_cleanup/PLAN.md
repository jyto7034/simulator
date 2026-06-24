# Core Skill Targeting Explicit Cleanup

## Objective

Remove skill targeting inference and step-level generic range leftovers:

1. Remove `SkillStepDef.range_units`.
2. Remove `SkillCastTargetingDef::FirstStepTarget`.
3. Require explicit cast-level target and explicit step execution target in live RON, runtime code, tests, and fixtures.
4. Replace test/fixture convenience with explicit helpers that do not infer from steps.

## Required Startup Protocol

Before code search, edits, or validation, read:

- `docs/goals/core_skill_targeting_explicit_cleanup/PLAN.md`
- `docs/goals/core_skill_targeting_explicit_cleanup/EXPERIMENTS.md`
- `docs/goals/core_skill_targeting_explicit_cleanup/EXPERIMENT_NOTES.md`
- `docs/goals/core_unimplemented_policy_implementation_master/PLAN.md`
- `docs/goals/core_component_refactor_master/POLICY_DECISIONS.md`
- `docs/skill_target_contract.md`
- `docs/refactor_preparation_plan.md`

## Source Of Truth

1. Runtime skill/cast/targeting code.
2. Live skill RON and skill loader.
3. Unity-facing skill catalog/range preview DTOs.
4. Current skill targeting policy docs.
5. Historical goal notes.

## Scope

In scope:

- `src/game/ability.rs`
- `src/game/data/skill_data.rs`
- skill cast/target/usefulness code that still reads step `range_units`
- skill/range preview tests and fixtures
- `tests/skill_refactor_validation.rs`
- `tests/common` and `tests/skill_test` helpers
- internal code/tests using `SkillCastTargetingDef::FirstStepTarget` or `Default::default()` for cast targeting

Out of scope:

- Basic attack/equipment/abnormality `range_units` that belong to non-skill-step domains.
- New contagion/chain mechanic implementation.
- Battlefield tile `occupant` removal.
- Server/admin typed DTO cleanup.

## Plan

1. Inventory all `SkillStepDef.range_units`, `SkillCastTargetingDef::FirstStepTarget`, and `cast_targeting: Default::default()` uses.
2. Separate skill-step range usage from basic attack/equipment/abnormality range usage.
3. Design explicit cast-targeting constructors/helpers.
4. Ensure helpers require caller-supplied target/range/range_policy/air capability and never infer from steps.
5. Replace test/fixture `FirstStepTarget` uses with explicit helpers or explicit literals.
6. Remove `FirstStepTarget` variant and its default.
7. Remove `SkillStepDef.range_units` from public/internal skill step definition.
8. Update loaders, tests, validation, and docs impacted by the removal.
9. Run focused tests after each small change group.
10. Run broad validation.

## Validation Plan

Focused:

- skill RON loading tests rejecting old `range_units` and missing `cast_targeting`.
- `skill_refactor_validation` tests.
- tests covering explicit cast target versus step execution target.
- range preview tests.
- target usefulness tests if touched.

Broad:

- `cargo check --lib`
- `cargo test skill --lib -- --test-threads=1`
- `cargo test --test skill_refactor_validation -- --test-threads=1`
- `cargo test --test ron_loading -- --test-threads=1`
- `cargo test --lib -- --test-threads=1`
- `git diff --check`

## Completion Conditions

- `SkillStepDef.range_units` is removed from skill step runtime/test construction surfaces.
- `SkillCastTargetingDef::FirstStepTarget` is removed.
- No cast target is inferred from the first step.
- Live RON and internal tests/fixtures specify cast-level targeting explicitly.
- Step execution target remains explicit through `target` and `targeting`.
- Helper API, if added, is explicit and not inference-based.
- Tests cover the durable user-visible behavior and data validation contract.
- Validation results are recorded.

