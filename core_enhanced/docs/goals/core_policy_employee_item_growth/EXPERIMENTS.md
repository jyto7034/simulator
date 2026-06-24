# Experiments

## Log

| Date | Scope | Attempt | Result | Follow-up |
| --- | --- | --- | --- | --- |
| 2026-06-22 | Goal setup | Defined employee/item/growth implementation scope. | Not started. | Start from employee state and live fragment data. |
| 2026-06-22 | Grade/growth policy first pass | Removed `EmployeeGrade` from employee runtime/candidate schema and moved level XP requirement, post-battle survival XP, and level-to-battle-tier mapping into `RunPolicyData.growth` live RON. Removed no-op `PveWinStack`/`QuestRewardStack`. | Success. `cargo check --lib` passed after updating runtime references and live RON. | Add focused RON/tests, then continue fragment equip-limit policy. |
| 2026-06-22 | Bound equipment interaction lock first pass | Added bound checks to equip, dismantle, enhance, combination material consumption, and shop sell paths; maintenance preview disables bound equipment dismantle/enhance. | Success. `cargo check --lib` and bound focused tests passed. | Continue growth/trauma focused validation. |
| 2026-06-22 | Skill fragment equip limit / dismantle repair | Added explicit `SkillFragmentEquipLimit` metadata and live RON fields. Runtime equip validates roster-wide active fragment usage against owned copy count or `GlobalExclusive`. Dismantle now auto-unequips only the employees exceeding the post-dismantle limit, sorted by UUID. | Success. Focused equip limit, global exclusive, dismantle, and live RON tests passed. | Continue bound equipment tests and trauma/growth focused validation. |
| 2026-06-22 | Starter fragment code injection boundary | Kept test-only `with_builtin_starter` fixture helper but removed it from production builds with `#[cfg(test)]`; live runtime starter fragment source remains RON/data validation. | Success. `cargo check --lib` and `cargo test --lib --no-run` passed. | Run broad subgoal validation. |
| 2026-06-22 | Bound maintenance command contract | Added a focused gameplay test proving bound equipment remains visible in maintenance preview but cannot be dismantled or enhanced even when recipes/materials exist. | Success. `cargo test bound_equipment_cannot_be_dismantled_or_enhanced_in_maintenance --lib` passed. | Run broad subgoal validation. |
| 2026-06-22 | Final subgoal validation | Ran live RON loading, server compile, and full core lib tests after formatting and diff hygiene checks. | Success. Subgoal completion conditions are met. | Mark subgoal complete and continue master sequence. |

## Failed Approaches

None yet.

## Validation Commands

- `cargo check --lib` - passed after grade/growth policy first pass and bound equipment interaction lock first pass.
- `cargo test skill_fragment_equip_requires_one_owned_copy_per_employee --lib` - passed, 1 test.
- `cargo test global_exclusive_skill_fragment_rejects_second_employee_even_with_extra_copy --lib` - passed, 1 test.
- `cargo test skill_fragment_dismantle --lib` - passed, 3 tests.
- `cargo test skill_fragment_equip --lib` - passed, 4 tests.
- `cargo test --test ron_loading` - passed after explicit `equip_limit` live RON update, 18 tests.
- `cargo test bound --lib` - passed after bound equipment interaction lock update, 12 tests.
- `cargo test run_policy --lib` - passed after growth policy migration, 2 tests.
- `cargo test trauma --lib` - passed after growth/trauma verification, 8 tests.
- `cargo test experience --lib` - passed after policy-based XP grant migration, 1 test.
- `cargo check -p game_server` - passed after removing server test-fixture grade references.
- `cargo check --lib` - passed after gating starter fragment injection helper to tests.
- `cargo test --lib --no-run` - passed after gating starter fragment injection helper to tests.
- `cargo test bound_equipment_cannot_be_dismantled_or_enhanced_in_maintenance --lib` - passed after adding bound maintenance command contract coverage.
- `cargo fmt` - completed after final subgoal edits.
- `rg -n "EmployeeGrade|\\.grade\\b|\\bgrade\\b|PveWinStack|QuestRewardStack|post_battle\\.survival_xp" src tests ../game_resources/data ../game_server/src -S` - no matches.
- `git diff --check` - passed after live RON whitespace cleanup.
- `cargo test --test ron_loading` - passed, 18 tests.
- `cargo check -p game_server` - passed.
- `cargo test --lib -- --test-threads=1` - passed, 503 tests.
