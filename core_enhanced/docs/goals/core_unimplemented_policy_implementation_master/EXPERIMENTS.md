# Experiments

This file records master-level attempts, validation commands, failures, fixes, and final verification for `core_unimplemented_policy_implementation_master`.

## Log

| Date | Scope | Attempt | Result | Follow-up |
| --- | --- | --- | --- | --- |
| 2026-06-23 | Goal setup | Created master and subgoal documents for the unimplemented confirmed policy implementation pass. | Pending implementation. | Start with `core_tile_validity_typed_dto_cleanup`. |
| 2026-06-23 | `core_tile_validity_typed_dto_cleanup` | Audited `Battlefield::in_bounds`; moved server `selected_event.compressed_event_log` enrichment from `serde_json::Value` mutation to typed server snapshot DTOs. | Focused validation passed. | Continue to `core_skill_targeting_explicit_cleanup`; run broad validation after all subgoals. |
| 2026-06-23 | `core_skill_targeting_explicit_cleanup` | Removed `FirstStepTarget`, removed skill cast/step `range_units`, and updated runtime/tests/fixtures to explicit cast targeting. | Focused validation passed after one test fix for explicit cast-level `air_capable`. | Continue to `core_battlefield_occupancy_cleanup`; run broad validation after all subgoals. |
| 2026-06-23 | `core_battlefield_occupancy_cleanup` | Removed `Battlefield` single occupant storage/API and changed placement to allow multi-unit tile occupancy while retaining static obstacle rejection. | Focused validation passed. | Re-read subgoal outputs against `POLICY_DECISIONS.md`, then run broad validation. |

## Validation Commands

Record master-level broad validation here after subgoals finish.

- `cargo test --lib -- --test-threads=1` (pass, 519 tests)
- `cargo test --test ron_loading -- --test-threads=1` (pass, 18 tests)
- `cargo check -p game_server` (pass)
- `git diff --check` (pass)
- Policy/status search: `rg -n "FirstStepTarget|cast_targeting:\\s*Default::default\\(\\)|SkillStepDef[\\s\\S]{0,220}range_units|step\\.range_units|occupant|place_allowing_unit_overlap|as_object_mut|Value::Object|get_mut\\(\"selected_event\"" src tests ../game_server/src ../game_resources/data/skills -S` (reviewed; remaining `occupant` matches are unrelated roster board slot terminology, not `Battlefield`)
