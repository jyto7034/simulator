# Validation Exhaustive Match Cleanup Experiments

| Date | Attempt | Result | Notes |
| --- | --- | --- | --- |
| 2026-07-04 | Created goal from refactor audit decisions. | Planned. | No code changes yet. |
| 2026-07-04 | Re-read goal docs and searched gameplay skill/effect validation for broad wildcard branches. | Found two validation wildcards. | `SkillFragmentEffectDef` used `_ => continue`; `SkillEffectDef` validation used `_ => {}`. |
| 2026-07-04 | Replaced validation wildcards with explicit variant arms. | Implemented. | `BasicAttackModifier` is explicit no-op for skill reference validation; non-damage skill effects are explicit no-op for the `ModifyDamage requires Damage` check. |
| 2026-07-04 | Re-ran wildcard search in `src/game/data/validation.rs` and `src/game/data/skill_data.rs`. | Passed. | No `_ =>` branches remain in those validation files. |
| 2026-07-04 | Ran required validation commands from `PLAN.md`. | Passed after fixture repair. | `ron_loading`, `skill_refactor_validation`, filtered `live_skill_catalog_audit`, and `cargo check -p game_core` all pass. |

## Failure Log Template

| Date | Attempt | Failure | Fix | Revalidation |
| --- | --- | --- | --- | --- |
| 2026-07-04 | `cargo test -p game_core --test skill_refactor_validation -- --test-threads=1` | `ron_added_abnormalities_emit_expected_skill_event_categories_in_battle_smoke` failed because the test-only `skill_test_dummy` abnormality used live skill fragments but omitted `response_complete_skill_fragment_id`. | Set the test-only dummy's response fragment to the existing `starter_basic_attack_enhancement` fragment without changing runtime behavior. | `cargo test -p game_core --test skill_refactor_validation -- --test-threads=1`: passed. |
| 2026-07-04 | `cargo test -p game_core --test skill_refactor_validation -- --test-threads=1` | Same smoke test failed because it copied live abnormalities with `omen_chain_id` but did not copy live boss omen/event data. | Added live `event_data` and `boss_omen_data` to the test builder. | `cargo test -p game_core --test skill_refactor_validation -- --test-threads=1`: passed. |
