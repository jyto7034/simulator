# Experiment Notes

- This goal should land before lifecycle work so later Active/Withdrawn/Dead gates do not have to support both layout-owned and body-owned position sources.
- Do not keep deprecated dynamic position APIs as compatibility wrappers.
- If tile-index acceleration is needed, design it as a derived multi-occupant index from `BattleCore.units` and `RuntimeUnit.body`, not as layout source of truth.
- Static obstacles belong to the layout, but unit occupancy does not.
- Implementation kept no deprecated wrapper around `position_of`, `place`, `remove`, or `units_at`; tests and runtime code now use unit body projection directly or through `BattleCore::unit_projected_tile` / `live_unit_projected_tile`.
- `build_runtime_field` now validates every materialized runtime unit's projected tile with `ensure_walkable_tile`; if later lifecycle work keeps withdrawn/dead units in `BattleCore.units`, this validation must be revisited under the lifecycle subgoal so inactive units do not accidentally need active placement.
- Projectile and area tests exposed an important fixture rule: when a test is validating world-space projectile/body behavior, do not use a tile helper that moves the body to tile center unless that center is actually part of the test contract.
- No new policy-decision requirement was found during this subgoal. The discovered fixture mismatch was an implementation/test setup issue under the already confirmed body-as-position-SoT policy.
