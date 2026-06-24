# Stats Pipeline Unification Experiments

## 2026-06-12

- Created goal working directory and memory files.
- Initial plan: inspect runtime stat paths before deciding the pipeline shape.
- Added `src/game/battle/stat_pipeline.rs` and moved employee battle profile modifier order plus draft final stats order into it.
- Updated `Employee::combat_profile_for_battle` and `BattleUnitDraft::effective_stats` to delegate to the pipeline module.
- Added order-sensitive tests for:
  - skill fragments before consumable offense boost,
  - growth before equipment percent modifiers,
  - consumable offense boost before equipment Permanent effects.
- Ran `cargo test -p game_core battle::stat_pipeline::tests -- --nocapture`: passed, 3 tests.
- Ran `cargo test -p game_core employee::tests -- --nocapture`: passed, 11 tests.
- Ran `cargo test -p game_core battle::types::tests -- --nocapture`: passed, 8 tests.
- Ran `cargo test -p game_core world::tests::equipment -- --nocapture`: passed, 20 tests.
- Ran `cargo test -p game_core --test live_item_skill_activation`: passed, 3 tests.
- Ran `cargo fmt`: completed.
- Ran `cargo check -p game_core`: passed.
