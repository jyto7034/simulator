# Automatic Hostile Target Usefulness Experiments

This file records implementation attempts, failures, fixes, and verification results for the automatic hostile target usefulness goal.

## 2026-06-16 Goal Document Setup

Goal:

- Create the working goal documents for implementing the automatic hostile target usefulness policy.
- Preserve the policy as a general targeting rule, not a The Silent Orchestra special case.

Files read:

- `docs/skill_target_contract.md`
- `docs/game_rulebook.md`
- `docs/skills/abnormality_skill_design_notes.ko.md`
- `docs/goals/step_targeting_contract/PLAN.md`
- `docs/goals/basic_attack_lifecycle_refactor/PLAN.md`

Files changed:

- `docs/goals/automatic_hostile_target_usefulness/PLAN.md`
- `docs/goals/automatic_hostile_target_usefulness/EXPERIMENTS.md`
- `docs/goals/automatic_hostile_target_usefulness/EXPERIMENT_NOTES.md`

Result:

- Created the required goal tracking documents.
- Defined the implementation objective, source-of-truth order, stop conditions, and expected verification.
- Scoped out direct manual target selection because current design does not expose it.
- Scoped out boss-specific targeting hardcoding and balance tuning.

Failure cause, if any:

- None. This was a documentation-only setup step.

Fix or next action:

- Start by reading runtime targeting and damage/effect execution code before adding helpers.
- Record evidence in `EXPERIMENT_NOTES.md` before changing behavior.

Verification:

- Not run. Documentation-only goal setup.

## 2026-06-16 Runtime Inventory And Shared Helper

Goal:

- Find the long-term runtime point for the target usefulness policy.
- Avoid boss-specific targeting and avoid duplicating target ordering rules.

Files read:

- `src/game/battle/core/basic_attack.rs`
- `src/game/battle/core/targeting.rs`
- `src/game/battle/core/sim.rs`
- `src/game/battle/core/skill_runtime/cast.rs`
- `src/game/battle/core/skill_runtime/area.rs`
- `src/game/battle/core/commands.rs`
- `src/game/battle/damage.rs`
- `src/game/ability.rs`
- `../game_resources/data/skills/base.ron`
- `../game_resources/data/abnormalities/base.ron`

Files changed:

- `src/game/battle/core/target_usefulness.rs`
- `src/game/battle/core/mod.rs`
- `src/game/battle/core/basic_attack.rs`
- `src/game/battle/core/targeting.rs`
- `src/game/battle/core/sim.rs`
- `src/game/battle/core/skill_runtime/cast.rs`
- `src/game/battle/core/skill_runtime/area.rs`

Result:

- Added a shared battle-core target usefulness helper using existing deterministic damage calculation.
- Basic attacks, cast target selection, retargeted skill steps, and tile-area automatic cast gating now share the same policy.
- Existing target ordering remains after usefulness filtering.

Failure cause, if any:

- Initial helper treated only direct `Damage`, `ApplyBuff`, and hostile `ModifyStats` as hostile payloads. Live RON showed Punishing Bird uses `ExtraAttack` as its skill payload.

Fix or next action:

- Extended usefulness to treat `ExtraAttack` as useful only if the scheduled basic attack would be useful against the target.
- Keep richer stack/refresh/immunity prediction out of this goal.

Verification:

- `cargo fmt`
- `cargo check -p game_core --tests`

## 2026-06-16 Focused Policy Tests

Goal:

- Pin visible targeting behavior for zero-damage targets, hostile effects, enemy heals, basic attacks, and automatic AoE casting.

Files changed:

- `src/game/battle/core/mod.rs`
- `src/game/battle/core/targeting.rs`

Result:

- Added tests for:
  - excluding zero-damage hostile skill targets with no hostile effect
  - allowing zero-damage hostile buff targets
  - excluding enemy heal/no-hostile-effect targets
  - excluding zero-damage basic attack targets with no hostile effect
  - waiting on automatic tile-area casts with no useful enemy
  - allowing automatic tile-area casts with a hostile buff

Failure cause, if any:

- After adding cast-target usefulness by first step, integration tests failed for skills whose first step is self-targeted and whose later step is the actual hostile step.

Fix or next action:

- Added a cast-target usefulness step selection helper so explicit enemy cast targets use the first hostile target-consuming step instead of blindly using the first skill step.
- Added full-skill automatic cast usefulness so `RetargetOnStep` can find useful hostile targets at execution time.

Verification:

- `cargo test -p game_core hostile_skill_targeting -- --nocapture`
- `cargo test -p game_core automatic_enemy_tile_area_cast -- --nocapture`
- `cargo test -p game_core basic_attack_targeting_excludes_zero_damage_without_hostile_effect -- --nocapture`
- `cargo test -p game_core --test skill_refactor_validation explicit_cast_targeting_separates_cast_context_from_step_execution_targets -- --nocapture`
- `cargo test -p game_core --test skill_refactor_validation self_then_retargeted_enemy_skill_resolves_second_step_at_execution_time -- --nocapture`

## 2026-06-16 Live RON Skill Regression

Goal:

- Verify live skill data still produces expected gameplay events under the new usefulness gate.

Files read:

- `../game_resources/data/skills/base.ron`
- `../game_resources/data/abnormalities/base.ron`
- `tests/skill_test/punishing_bird.rs`
- `tests/skill_test/common/scenario.rs`

Files changed:

- `src/game/battle/core/target_usefulness.rs`
- `tests/skill_test/punishing_bird.rs`

Result:

- `ron_added_abnormalities_emit_expected_skill_event_categories_in_battle_smoke` passed after `ExtraAttack` was recognized through scheduled basic-attack usefulness.
- Punishing Bird's focused skill test now places the target in the live `front_3x2` skill range instead of relying on a broad test-only range.

Failure cause, if any:

- First failure: `ExtraAttack` was not considered hostile payload, so `punishing_bird_rapid_peck` never started.
- Second failure: the focused Punishing Bird fixture placed the enemy below the caster while the live skill uses `front_3x2`; the broader smoke test patched skill ranges and therefore did not expose this fixture mismatch.

Fix or next action:

- Treat `ExtraAttack(count > 0)` as hostile payload only when the scheduled basic attack is useful.
- Align the focused fixture with live skill targeting by moving the dummy to the caster's front tile.

Verification:

- `cargo test -p game_core punishing_bird_rapid_peck -- --nocapture`
- `cargo test -p game_core ron_added_abnormalities_emit_expected_skill_event_categories_in_battle_smoke -- --nocapture`
- `cargo test -p game_core skill -- --nocapture`

## 2026-06-16 Final Verification

Goal:

- Verify the final implementation across focused battle runtime tests, integration skill tests, live RON loading, full `game_core`, and server-facing compile checks.

Files changed:

- No new behavior changes in this step.
- Updated goal documentation to mark broad verification complete.

Result:

- Focused and broad checks passed.
- No Unity-facing DTO shape changes were required.
- No live RON schema changes were required.
- No compatibility layer or dual behavior was added.

Failure cause, if any:

- None in final verification.

Fix or next action:

- None required for this goal.

Verification:

- `cargo fmt`
- `cargo check -p game_core --tests`
- `cargo test -p game_core targeting -- --nocapture`
- `cargo test -p game_core skill -- --nocapture`
- `cargo test -p game_core battle::core -- --nocapture`
- `cargo test -p game_core --test skill_refactor_validation -- --nocapture`
- `cargo test -p game_core --test ron_loading -- --nocapture`
- `cargo test -p game_core`
- `cargo check -p game_server --tests`

## Experiment Log Template

Use this format for each implementation attempt:

```text
## YYYY-MM-DD <short attempt name>

Goal:

Files read:

Files changed:

Result:

Failure cause, if any:

Fix or next action:

Verification:
```
