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
