# Experiments

| Date | Trial | Result | Notes |
| --- | --- | --- | --- |
| 2026-07-06 | Goal document creation from audit review. | Planned. | Covers P-004, P-005, and P-006: battle-record naming drift, mixed state/export responsibility, and repository-root generated outputs. |
| 2026-07-06 | Runtime/export boundary refactor. | Success. | Split run-state mutation from debug JSON export. Replaced abnormality-keyed `battle_uuid` export naming with `abnormality_uuid`. |
| 2026-07-06 | Debug output quarantine. | Success. | Moved default battle record and debug event-log export paths under `target/`. Existing repo-root tracked generated files were not deleted or rewritten. |
| 2026-07-06 | Extra skill suite check. | Failed, unrelated. | `cargo test -p game_core --test skill_test_suite -- --test-threads=1` failed before export writing because `skill_test_dummy` lacks `response_complete_skill_fragment_id`. No evidence this is caused by the debug export refactor. |

## Validation Log

- `cargo test -p game_core battle_record --lib -- --test-threads=1` passed. It currently matches 0 tests.
- `cargo test -p game_core combat_result --lib -- --test-threads=1` passed: 5 passed.
- `cargo test -p game_core game::world::tests::combat --lib -- --test-threads=1` passed: 39 passed.
- `cargo check -p game_core` passed.
- `cargo fmt --check` passed.
- `git status --short battle_records debug_event_log_exports logs` still reports two pre-existing modified repo-root debug artifacts: `battle_records/run_0000000000000000/34c54c75-9364-4a65-8779-87379b0e7903.json` and `debug_event_log_exports/defense_combat_node_smoke.json`.
- `find target/battle_records target/debug_event_log_exports ...` confirmed the smoke debug event log is now written to `target/debug_event_log_exports/defense_combat_node_smoke.json`.
