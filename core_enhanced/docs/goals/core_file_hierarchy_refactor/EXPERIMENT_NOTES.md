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
