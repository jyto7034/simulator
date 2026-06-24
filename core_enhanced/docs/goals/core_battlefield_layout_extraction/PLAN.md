# Core Battlefield Layout Extraction

## Objective

Extract `Battlefield` into a static layout object and remove dynamic unit position ownership from it.

Confirmed policy source: `docs/goals/core_component_refactor_master/POLICY_DECISIONS.md`, `Battlefield static layout extraction`, decisions 1, 2, 19, 20, and 21.

## Required Startup Protocol

At the beginning of this goal and every resumed run, read:

- `docs/goals/core_battlefield_layout_extraction/PLAN.md`
- `docs/goals/core_battlefield_layout_extraction/EXPERIMENTS.md`
- `docs/goals/core_battlefield_layout_extraction/EXPERIMENT_NOTES.md`
- `docs/goals/core_followup_policy_implementation_master/PLAN.md`
- `docs/goals/core_component_refactor_master/POLICY_DECISIONS.md`
- `docs/refactor_preparation_plan.md`

Do not start code search, edits, or validation before reading those documents.

## Scope

### In Scope

- Rename `Battlefield` to `BattlefieldLayout` or the documented final name.
- Keep only static layout ownership: width, height, valid tiles, void tiles, static obstacles, valid/walkable tile queries.
- Remove `unit_pos`, `position_of`, `place`, `remove`, and `units_at` from the layout object.
- Replace tile-position consumers with `RuntimeUnit.body.position.project_to_tile()` or `UnitBody::projected_tile()`.
- Rename `in_bounds` to `is_valid_tile` and add a raw rectangle helper only where needed.
- Preserve `is_walkable_tile` as `is_valid_tile && !is_static_obstacle`.
- Keep `void_tiles()` as a derived layout query.

### Out Of Scope

- Adding `RuntimeUnitLifecycle`.
- Redeploy HP/identity behavior.
- Projectile/delayed-effect withdrawn/dead resolution.
- Debug/admin inactive unit queries, except where current code must stop using `Battlefield.position_of`.

## Implementation Plan

1. Inventory all `Battlefield`, `position_of`, `place`, `remove`, `units_at`, `unit_pos`, and `in_bounds` call sites.
2. Split current static layout behavior from dynamic unit position behavior.
3. Replace spawn/deploy/death/tile DTO position use with unit body projection.
4. Remove dynamic position APIs from the layout object.
5. Rename valid-tile API and update all call sites.
6. Update tests to assert movement after spawn uses body-derived tile projection, not spawn-time layout storage.
7. Run focused battlefield/range/skill/deployment tests and `cargo check`.

## Policy Decision Completion Condition

If implementation exposes a new Unity-facing DTO shape change, save migration, or gameplay behavior change beyond the already confirmed `position` derived-tile semantics, complete the goal with `사용자와 정책 논의 필요` and report options.

## Completion Conditions

- No gameplay code uses `battlefield.position_of`, `battlefield.place`, `battlefield.remove`, `battlefield.units_at`, or `unit_pos`.
- The static layout object does not own unit position, deployment, death, withdrawn, or occupant state.
- Live checkpoint/deployment DTO tile `position` derives from `RuntimeUnit.body.position`.
- Valid/walkable/void tile APIs match the confirmed naming and meaning.
- Focused and broad validation commands are recorded.

## Implementation Result

Status: implemented; completion review pending.

- `Battlefield` was renamed to `BattlefieldLayout`.
- Layout-owned dynamic unit position state and APIs were removed.
- Runtime tile positions now derive from `RuntimeUnit.body` projection.
- `in_bounds` was replaced with `is_valid_tile`; raw rectangle checks use `position_in_bounds`.
- Focused validation is recorded in `EXPERIMENTS.md`.
