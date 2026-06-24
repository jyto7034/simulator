# Core File Hierarchy Refactor Experiments

## Experiment Log

### 2026-06-09 - Initial Structure Read

- Action: inspected `src/game/world/admin.rs` section boundaries.
- Result: the file combines admin command parsing/output, grant catalog DTO/building, fixture node creation, grant mutation helpers, and admin tests.
- Decision: start with catalog extraction because it is self-contained and should preserve command/output behavior.

### 2026-06-09 - Admin Catalog Extraction First Test

- Action: extracted admin grant catalog DTO/building code into `src/game/world/admin/catalog.rs`, then ran `cargo fmt` and `cargo test -p game_core admin_dump_grant_catalog_lists_grantable_live_data -- --nocapture`.
- Result: compile failed because the admin test fixture had been relying on the parent module's private `SkillFragmentId` import.
- Fix direction: import `SkillFragmentId` directly in the test module. This keeps the test fixture explicit and avoids reintroducing catalog-only imports to `admin.rs`.

### 2026-06-09 - Admin Catalog Extraction Retest

- Action: added the explicit test import, reran `cargo fmt`, and reran `cargo test -p game_core admin_dump_grant_catalog_lists_grantable_live_data -- --nocapture`.
- Result: passed. The `AdminGrantCatalog` result type and payload assertions still hold after the catalog module split.
- Remaining validation: broader admin tests and `cargo check` are still required before this refactor phase can be considered complete.

### 2026-06-09 - Admin Module Responsibility Split

- Action: split `src/game/world/admin.rs` into `commands.rs`, `catalog.rs`, `fixtures.rs`, `grants.rs`, and `tests.rs`.
- Result: `admin.rs` now keeps the public command re-export, `AdminCommandOutput`, command dispatch, and shared output helper. Submodules own the narrower implementation areas.
- Verification: `cargo fmt` passed. `cargo test -p game_core admin -- --nocapture` passed with 11 tests and no local warnings from the refactor.
- Contract note: `pub use commands::AdminCommand` preserves the existing `world::admin::AdminCommand` path.

### 2026-06-09 - World Tests File Split First Test

- Action: mechanically split `src/game/world/tests.rs` into `src/game/world/tests/mod.rs` plus topic files for `snapshots_and_start`, `map_flow`, `support`, `node_sessions`, `equipment`, `combat`, and `placement`.
- Result: `cargo fmt` passed, but `cargo test -p game_core support -- --nocapture` initially failed because `include_str!` paths in `tests/mod.rs` were now one directory deeper.
- Fix: changed the shared `game_resources` include paths from `../../../../game_resources` to `../../../../../game_resources`.

### 2026-06-09 - World Tests File Split Retests

- Action: reran focused tests after the path fix.
- Verification:
  - `cargo test -p game_core support -- --nocapture` passed.
  - `cargo test -p game_core equipment -- --nocapture` passed.
  - `cargo test -p game_core maintenance -- --nocapture` passed.
  - `cargo test -p game_core combat -- --nocapture` passed.
- Note: the combat filter still writes `debug_event_log_exports/defense_combat_node_smoke.json`, confirming that debug export classification must be handled explicitly later in the goal.

### 2026-06-09 - Combat Preview Hierarchy Split

- Action: moved `src/game/combat_preview.rs` to `src/game/combat_preview/mod.rs` and split responsibilities into `types.rs`, `template.rs`, `threat.rs`, and `validation.rs`.
- Result: `types.rs` owns Unity-facing preview DTO/type definitions; `template.rs` owns ASCII battlefield template parsing; `threat.rs` owns briefing/rumor warning calculation; `validation.rs` owns instance contract checks; `mod.rs` owns generation/build orchestration and tests.
- Verification: `cargo fmt` passed. `cargo test -p game_core combat_preview -- --nocapture` passed after correcting moved module boundaries.

### 2026-06-09 - Root Combat Setup File Move

- Action: moved root `src/game/combat_*.rs` setup/handoff files into `src/game/combat_setup/`.
- Result: existing public module paths such as `crate::game::combat_balance` are preserved with `#[path = "..."]` declarations in `src/game/mod.rs`, avoiding Unity/server-facing import churn while making file placement clearer.
- Verification: `cargo fmt`, `cargo test -p game_core combat_preview -- --nocapture`, and `cargo test -p game_core combat -- --nocapture` passed.

### 2026-06-09 - Debug/Export/Backup Classification

- Action: inspected `tests/`, `tests_bak/`, `debug_event_log_exports/`, `timeline_exports/`, `logs/`, `tmp/`, and `src/old/` references.
- Result: added current classification to `docs/refactor_preparation_plan.md`.
- Decision: do not delete tracked files in `tests_bak/`, `debug_event_log_exports/`, or `logs/` in this goal because changing tracked artifact policy can break existing review/verification expectations and requires a cleanup decision.

### 2026-06-09 - Final Verification

- `cargo check -p game_core`: passed.
- `cargo check -p game_server`: passed.
- `cargo test -p game_core admin -- --nocapture`: passed.
- `cargo test -p game_server admin -- --nocapture`: passed.
- `cargo test -p game_core support -- --nocapture`: passed.
- `cargo test -p game_core maintenance -- --nocapture`: passed.
- `cargo test -p game_core equipment -- --nocapture`: passed.
- `cargo test -p game_core combat_preview -- --nocapture`: passed.
- `cargo test -p game_core combat -- --nocapture`: passed.
- `cargo test -p game_core --test ron_loading -- --nocapture`: passed.
- `cargo test -p game_core`: passed, 447 unit tests plus integration/doc test suites.
- `cargo test -p game_server`: passed, 16 tests plus doc tests.
