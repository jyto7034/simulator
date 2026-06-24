# Combat Setup Usage Audit Experiments

## Log

- Created goal working docs.
- Inspected every `src/game/combat_setup/*.rs` file and searched runtime/test callers.
- Result: all eight files have live callers through battle startup, combat preview, data validation, world snapshots, or damage feedback. No confirmed dead stubs were removed.
- Added module-level responsibility docs to each combat setup file to make the retained boundaries explicit without changing behavior/schema/DTO.
- Ran `cargo fmt`: passed.
- Ran `cargo check -p game_core`: passed.
- Ran `cargo test -p game_core game::events::combat -- --nocapture`: passed, 11 tests.
- Ran `cargo test -p game_core game::combat_preview -- --nocapture`: passed, 22 tests.
- Ran `cargo test -p game_core --test ron_loading`: passed, 16 tests.
- Ran `cargo test -p game_core game::world::tests -- --nocapture`: passed, 75 tests.
- Ran `cargo test -p game_core`: passed, 454 unit tests plus integration suites (`live_item_skill_activation`, `live_skill_catalog_audit`, `ron_loading`, `skill_refactor_validation`, `skill_test_suite`).
