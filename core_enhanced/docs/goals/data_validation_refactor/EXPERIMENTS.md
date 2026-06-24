# Data Validation Refactor Experiments

## 2026-06-12

- Created goal working directory and memory files.
- Initial plan: move behavior-preserving cross-reference validation from `data/mod.rs` to `data/validation.rs`.
- Moved cross-reference validation helpers from `src/game/data/mod.rs` into new `src/game/data/validation.rs`.
- Kept `GameDataBase::new` validation order intact:
  index validation, skill/buff and content cross-reference validation, item registry creation, shop item validation.
- Ran `cargo check -p game_core`: passed.
- Confirmed `src/game/battle/replay/` contained no files and had no runtime/test references; removed the empty directory with `rmdir`.
- Ran `cargo test -p game_core --test ron_loading`: passed, 16 tests.
- Ran `cargo test -p game_core --test live_skill_catalog_audit`: passed, 3 tests.
- Ran `cargo fmt`: completed.
- Re-ran `cargo check -p game_core` after formatting: passed.
