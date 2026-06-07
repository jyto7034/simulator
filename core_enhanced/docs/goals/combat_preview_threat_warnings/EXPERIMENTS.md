# Combat Preview Threat Warnings Experiments

## 2026-06-07

- Inspected `docs/combat_preview_threat_warnings_goal.md`.
- Inspected `src/game/combat_preview.rs`.
  - `CombatPreview` is built from `BattlefieldInstance`.
  - `BattlefieldInstance` already contains materialized `spawn_waves` and `enemy_briefing`.
- Inspected `src/game/data/pve_data.rs`.
  - `PveEncounter` currently has no warning field.
- Inspected external canonical `/mnt/f/unity projects/ark/docs/unity_core_contract.md`.
  - The documented JSON already includes `threat_warnings`.
- Inspected enemy metadata sources.
  - Abnormalities and corroded employee profiles expose defense, magic resist, and movement speed.
  - Air, shield, regeneration, and block-resistance facts are not currently explicit in preview metadata.
- Implemented typed threat warning DTOs and generation.
- Added focused tests:
  - `combat_preview_serializes_typed_threat_warnings`
  - `rumor_threat_warning_is_seeded_and_limited_to_one_entry`
  - `rumor_threat_warning_does_not_duplicate_real_briefing_tag`
- Updated explicit `BattlefieldInstance` and `CombatPreview` test literals with `threat_warnings: Vec::new()`.

## Verification

- `cargo test -p game_core combat_preview -- --nocapture`
  - Result: passed, 23 tests.
- `cargo test -p game_core node_preview -- --nocapture`
  - Result: passed, 1 test.
- `cargo test -p game_core ron_loading -- --nocapture`
  - Result: command succeeded but ran 0 tests because the filter did not match test names.
- `cargo test -p game_core --test ron_loading -- --nocapture`
  - Result: passed, 14 tests.
- `cargo check -p game_core`
  - Result: passed.
