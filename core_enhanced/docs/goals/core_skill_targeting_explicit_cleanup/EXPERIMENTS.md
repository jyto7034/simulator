# Experiments

This file records implementation attempts, validation commands, failures, fixes, and final verification for `core_skill_targeting_explicit_cleanup`.

## Log

| Date | Scope | Attempt | Result | Follow-up |
| --- | --- | --- | --- | --- |
| 2026-06-23 | Goal setup | Created the subgoal document for removing skill targeting inference and step-level `range_units`. | Pending implementation. | Start with a code inventory of `range_units`, `FirstStepTarget`, and cast-targeting defaults. |
| 2026-06-23 | inventory | Searched runtime, tests, and live skill RON for `SkillStepDef.range_units`, `SkillCastTargetingDef::FirstStepTarget`, and `cast_targeting: Default::default()`. | Live skill RON already uses explicit `cast_targeting`; remaining legacy was in core types, loader conversion defaults, runtime dead parameters, and tests/fixtures. | Remove core/test legacy surfaces without touching basic attack/equipment/abnormality `range_units`. |
| 2026-06-23 | runtime cleanup | Removed `FirstStepTarget`, removed skill cast/step `range_units`, changed `SkillDef::cast_target_definition()` and target resolution to use explicit target/range policy/tile range/air capability only. | `cargo check --lib` passed after tuple-shape cleanup. | Update tests/fixtures to explicit cast targeting. |
| 2026-06-23 | fixture cleanup | Replaced `FirstStepTarget` and `cast_targeting: Default::default()` fixtures with explicit cast targeting. Removed step `range_units` from skill fixtures. | `cargo test --test skill_refactor_validation -- --test-threads=1` passed. Initial `cargo test skill --lib -- --test-threads=1` failed one air-capable test because it had been relying on first-step inferred cast `air_capable`. | Updated the test to mutate cast-level `air_capable` explicitly; reran focused tests successfully. |

## Failed Approaches

Record failed approaches and why they were abandoned.

- Initial focused `cargo test skill --lib -- --test-threads=1` failed `skill_cast_target_requires_air_capable_for_airborne_enemy`: the old test changed only `step.air_capable`, relying on first-step inference to change cast eligibility. With explicit cast targeting, cast-level `air_capable` is the source for cast target selection. Fixed the test to update `skill.cast_targeting` explicitly and reran successfully.

## Validation Commands

Record focused and broad validation commands here.

- `cargo check --lib` (pass)
- `cargo test skill --lib -- --test-threads=1` (pass after explicit cast-level air-capable test fix)
- `cargo test --test skill_refactor_validation -- --test-threads=1` (pass)
- `cargo test --test ron_loading -- --test-threads=1` (pass)
- `rg -n "FirstStepTarget|cast_targeting:\\s*Default::default\\(\\)|SkillCastTargetingDef::default|Default for SkillCastTargetingDef" src tests ../game_resources -S` (pass: no matches)
- `rg -n "SkillStepDef[\\s\\S]{0,220}range_units|step\\.range_units|SkillCastTargetingDef::Explicit \\{[\\s\\S]{0,160}range_units" src/game tests ../game_resources/data/skills -S` (pass: no matches)
