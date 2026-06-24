# Core File Hierarchy Refactor Notes

## Source Of Truth Order

1. Runtime code under `src/game`.
2. Live RON/data under `../game_resources/data` when accessible.
3. Unity-facing snapshot/command contracts.
4. Current policy documents.

## Notes

- The current `admin.rs` split can proceed without a policy decision if command names, result types, and payload shapes are preserved exactly.
- Existing dirty worktree contains many unrelated changes. Treat them as user or prior-goal work and do not revert them.
- If a split exposes obsolete debug/export/backup assets, classify them first; deletion requires confidence that they are not live code or current docs.
- 2026-06-09: `AdminGrantCatalog` DTO/building code is a good first extraction because it is a contract-shaped unit with direct tests. Keep command parsing in `admin.rs` until the command/fixture/grant split is planned as a whole.
- 2026-06-09: Admin command parsing has now moved to `admin/commands.rs`, but the public type is re-exported from `admin.rs`. This avoids external import churn while still giving the enum a clear home.
- 2026-06-09: `combat_preview/template.rs` and `validation.rs` existed before this goal but were stale relative to the monolithic `combat_preview.rs`. The current source of truth has been corrected by extracting the latest implementation from `mod.rs` into those files before wiring them in.
- 2026-06-09: `#[path = "combat_setup/..."]` in `src/game/mod.rs` is intentional. It preserves existing public module paths while moving implementation files out of the `src/game` root. A full public path rename would require external import audit and user approval.
