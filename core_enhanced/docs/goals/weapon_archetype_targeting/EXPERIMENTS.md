# Weapon Archetype Targeting Experiments

## Log

- Started after Airborne Enemy Mobility completed.
- Read `docs/weapon_archetype_targeting_goal.md`, `equipment_data.rs`, live equipment RON, and failure output from full `game_core`/`ron_loading`.
- Observed that the schema and validation already exist, but live/test equipment fixtures have not been migrated to explicit `weapon_profile`.
- Migrated current live equipment RON to explicit profiles for `Sword`, `Spear`, `Shield`, `Bow`, `Gun`, `GrenadeLauncher`, and `Staff`.
- Added runtime targeting profile comparison:
  - `DefaultForward`: route remaining distance, fallback to route progress/spawn order/id.
  - `AirFirst`: airborne first, fallback to `DefaultForward`.
  - `LowDefenseFirst`: lowest defense, fallback to `DefaultForward`.
  - `LowMagicResistFirst`: lowest magic resist, fallback to `DefaultForward`.
  - `SplashClusterFirst`: highest nearby valid enemy count around direct target, fallback to `DefaultForward`.
- Updated player attack selection so melee blocked targets stay highest priority, while weapon profile selection takes precedence over old hinted/persisted target behavior.
- Updated Spider Bud skill test from old current-target expectation to the skill data's `Nearest` target contract.
- Exposed `weapon_profile` in equipment DTO/snapshots and `effective_weapon_profile` in roster combat profile snapshot.

## Verification

- `cargo test -p game_core targeting -- --nocapture`: passed.
- `cargo test -p game_core --test ron_loading -- --nocapture`: passed.
- `cargo test -p game_core --test skill_test_suite -- --nocapture`: passed after updating Spider Bud to the current skill contract.
- `cargo test -p game_core -- --nocapture`: passed.
- `cargo check -p game_server`: passed.
- `git diff --check`: passed.
