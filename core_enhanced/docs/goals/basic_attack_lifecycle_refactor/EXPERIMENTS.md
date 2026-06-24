# Basic Attack Lifecycle Refactor Experiments

## 2026-06-12

- Created goal working directory and memory files.
- Initial plan: inspect `sim.rs`/`commands.rs` basic attack flow before editing.
- Added `src/game/battle/core/basic_attack.rs`.
- Moved basic attack target persistence/selection, pending auto-attack scan, AttackStart event handling, and AttackResolve event handling into the new lifecycle module.
- Updated `sim.rs` event loop to delegate basic attack lifecycle handling.
- Ran `cargo check -p game_core`: passed after removing unused imports.
- Ran `cargo test -p game_core battle::core::tests -- --nocapture`: passed, 44 tests.
- Ran `cargo test -p game_core battle::core::commands::tests -- --nocapture`: passed, 13 tests.
- Ran `cargo test -p game_core --test skill_refactor_validation`: passed, 10 tests.
- Ran `cargo fmt`: completed.
- Ran `cargo check -p game_core`: passed.
