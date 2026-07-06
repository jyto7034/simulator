# Embedded Data Loader Ownership Contract Experiments

Use this file to record loader inventory attempts, ownership decisions, failed approaches, and validation results.

## Trial Log

| Date | Trial | Result | Notes |
| --- | --- | --- | --- |
| 2026-07-06 | Loader inventory with `rg -n "include_str!|load_live_embedded|from_ron_str" src tests ../game_server -S`. | Success. | Classified every embedded RON/builtin loader before implementation in `EXPERIMENT_NOTES.md`. |
| 2026-07-06 | Kept map node definitions, map generation policy, battlefield archetypes/templates, and run policy as explicit domain-owned builtin loaders. | Success. | Added ownership comments and canonical `docs/data_loading_contract.md` entries instead of forcing unrelated catalogs into `GameDataBase`. |
| 2026-07-06 | Removed duplicate public live buff loader paths. | Success. | Replaced `BuffDatabase::live_default()` with a private data-module helper used by `GameDataBase::load_live_embedded()` and the fixture builder; removed unused `EventLogValidator::with_live_buff_data()`. |
| 2026-07-06 | Updated canonical docs. | Success. | Added `docs/data_loading_contract.md` and linked it from `docs/README.md`. |

## Validation Log

| Date | Command | Result | Notes |
| --- | --- | --- | --- |
| 2026-07-06 | `rg -n "BuffDatabase::live_default|with_live_buff_data" src tests ../game_server -S` | Passed | No matches; duplicate public live buff loader entrypoints are gone. |
| 2026-07-06 | `rg -n "include_str!|load_live_embedded|from_ron_str" src tests ../game_server -S` | Passed | Remaining embedded readers match the classification table: official live bundle, documented domain-owned builtin, or test-only direct load. |
| 2026-07-06 | `cargo check -p game_core` | Passed | Only existing workspace manifest warning: `auth_server/Cargo.toml` unused `env` key. |
| 2026-07-06 | `cargo check -p game_server` | Passed | Confirms production server still builds through `GameDataBase::load_live_embedded()`. |
| 2026-07-06 | `cargo test -p game_core --test ron_loading -- --test-threads=1` | Passed | 18 tests passed. |
| 2026-07-06 | `cargo test -p game_core --test live_skill_catalog_audit -- --test-threads=1` | Passed | 3 tests passed. |
| 2026-07-06 | `cargo test -p game_core combat_preview --lib -- --test-threads=1` | Passed | 27 tests passed; validates retained battlefield/combat-preview builtins. Test build reported unrelated unused-import warnings in `src/game/world/tests/mod.rs`. |
| 2026-07-06 | `cargo test -p game_core map:: --lib -- --test-threads=1` | Passed | 22 tests passed; validates retained map builtin loaders. Test build reported unrelated unused-import warnings in `src/game/world/tests/mod.rs`. |
