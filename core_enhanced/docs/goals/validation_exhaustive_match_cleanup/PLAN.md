# Validation Exhaustive Match Cleanup

## Objective

Remove broad wildcard branches from gameplay skill/effect validation paths so new variants cannot silently bypass live RON validation.

This is a medium-sized refactor because the code change is simple, but each wildcard must be checked for intentional passthrough versus missing validation.

## Source Of Truth Order

1. Runtime code and live RON/data.
2. `src/game/data/validation.rs` and data type definitions.
3. Skill/effect runtime behavior.
4. Current canonical skill/data docs.
5. Refactor audit note.

## Plan

1. [x] Search validation code for broad wildcard branches in skill/effect/RON validation.
2. [x] For each wildcard:
   - Identify the enum and all current variants.
   - Decide whether each variant needs validation.
   - Replace `_ => {}` with explicit variant arms.
3. [x] If a variant requires validation that does not exist, implement validation or stop with a policy question if gameplay meaning is unclear.
4. [x] Keep variants with no extra validation explicit.
5. [x] Run live RON loading tests and relevant skill validation tests.
6. [x] Document any intentionally no-op validation arms in comments only when helpful.

## Completion Conditions

- Gameplay skill/effect validation no longer silently accepts future variants through broad wildcard branches.
- Existing live RON still validates.
- New enum variants should force validation code review at compile time.
- No live data fallback or compatibility path is introduced.

## Completion Notes

- `SkillFragmentEffectDef` validation now explicitly handles `BasicAttackModifier` and `ActiveSkill`.
- `SkillEffectDef` validation now explicitly handles every current effect variant.
- Runtime event-log validators still use wildcard branches to ignore unrelated `BattleLogEvent` variants; that is outside this goal's live RON skill/effect validation scope.

## Validation Commands

- `cargo test -p game_core --test ron_loading -- --test-threads=1`
- `cargo test -p game_core --test skill_refactor_validation -- --test-threads=1`
- `cargo test -p game_core live_skill_catalog --test live_skill_catalog_audit -- --test-threads=1`
- `cargo check -p game_core`

Adjust filters if exact test names differ.

## Stop Conditions

Complete the goal and report questions if:

- A variant's intended validation policy is unclear.
- Existing live RON uses a variant that runtime supports but current policy documents do not.
- Validation requires changing live RON schema or gameplay semantics.
