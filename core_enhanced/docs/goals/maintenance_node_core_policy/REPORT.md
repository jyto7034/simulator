# Maintenance Node Core Policy Report

## Summary

- Replaced legacy list-only `MaintenanceOptionsDto` with typed operation previews.
- Replaced skill-fragment same-rarity material consumption with `fragment_dust`-based upgrade and awakening.
- Kept `fragment_dust` as the existing skill-fragment inventory scalar.
- Reworked equipment live data to use `equipment_dust` through existing `EquipmentMaterialMetadata` and `equipment_materials` stacks.
- Allowed Maintenance convenience equip/unequip actions for equipment and skill fragments.
- Allowed equipped skill fragments and equipped equipment to be dismantled after Unity confirmation, with core automatically unequipping before removal.
- Disabled equipment awakening in Maintenance operation preview.
- Updated game_server request/result mapping for target-only skill-fragment growth commands.
- Updated Unity-facing contract docs and live RON/data tests.

## Removed Legacy

- Removed command payload fields `material_fragment_id` and `material_fragment_ids`.
- Removed skill-fragment same-rarity material upgrade/awakening runtime path.
- Removed equipped-fragment and last-copy protection from skill-fragment dismantle.
- Removed unequipped-only equipment dismantle validation.
- Removed live `damaged_weapon_fragment` / `damaged_armor_fragment` economy in favor of `equipment_dust`.
- Removed stale Maintenance list-only option use from runtime.

## New Contracts

- `maintenance_options.items[*]` now carries target identity, source, operation previews, costs, gains, before/after states, confirmation flags, auto-unequip flags, and warnings.
- `maintenance_options.materials` includes `fragment_dust` and `equipment_dust`.
- `upgrade_skill_fragment` payload is target-only.
- `awaken_skill_fragment` payload is target-only.
- `SkillFragmentUpgraded` and `SkillFragmentAwakened` results report `dust_spent`, `remaining_dust`, and progress.

## Tests Updated

- Skill-fragment unit tests now verify dust-based upgrade/awakening.
- Gameplay tests now verify fragment dust consumption, last-copy fragment dismantle, equipped-fragment auto-unequip, equipment operation previews, and equipped-equipment auto-unequip before dismantle.
- RON loading tests now verify `equipment_dust` rewards and equipment recipes.
- game_server tests now verify target-only Maintenance command deserialization.

## Verification

- `cargo fmt`: passed.
- `cargo check -p game_core`: passed.
- `cargo check -p game_server`: passed.
- `cargo test -p game_core --test ron_loading -- --nocapture`: passed with 16 tests.
- `cargo test -p game_core -- --nocapture`: passed with 434 lib tests plus all game_core integration tests.
- `cargo test -p game_server -- --nocapture`: passed with 11 tests.
- `git diff --check`: passed.

## Remaining Risk

- Balance numbers are initial policy defaults: skill-fragment upgrade costs 4 `fragment_dust`, post-awakening upgrade costs 8, and early awakening costs 4 dust per missing awakening progress.
- The existing warning `auth_server/Cargo.toml: unused manifest key: env` remains unrelated to this goal.
