# Movement Backend Policy Refactor Experiments

## Log

- Read master goal, movement backend goal, `engine.rs`, `rapier_backend.rs`, `types.rs`, `planner.rs`, `steering.rs`, and `RuntimeUnit` movement fields.
- Baseline `cargo check -p game_core` failed before this goal's code edits due existing partial weapon/archetype changes in `commands.rs`, `battle/types.rs`, `combat_defense_object.rs`, and `corroded_employee_data.rs`.
- Added `MovementTerrainPolicy::{Ground, Airborne}` to movement input.
- Removed Direct movement unit separation so unit overlap cannot be reintroduced by a Direct-only tuning field.
- Changed Direct/Rapier static obstacle correction to run only for `Ground`.
- Removed Rapier default local obstacle avoidance so authored route obstacles are not silently bypassed.
- Changed Rapier runtime tick to rely on core board clamp instead of syncing board wall colliders.
- Added Direct/Rapier airborne policy tests for static obstacle immunity and board clamp.
- Fixed build-blocking fixture/default-field fallout from existing weapon/target-trait partial changes so focused movement tests could compile.

## Verification

- `cargo check -p game_core`: failed before movement edits due pre-existing partial weapon/archetype compile errors.
- `cargo test -p game_core movement -- --nocapture`: failed once on exact float comparison, fixed with approximate comparison.
- `cargo test -p game_core movement -- --nocapture`: passed, 50 passed.
- `cargo test -p game_core blocking -- --nocapture`: passed, 1 passed. The filter is narrow.
- `cargo test -p game_core airborne -- --nocapture`: passed, 4 passed.
- `cargo test -p game_core fixed_defense -- --nocapture`: passed, 17 passed.
- `cargo check -p game_core`: passed.
