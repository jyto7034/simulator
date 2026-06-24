# Experiments

This file records implementation attempts, validation commands, failures, fixes, and final verification for `core_policy_map_progression_event_log_contract`.

## Log

| Date | Scope | Attempt | Result | Follow-up |
| --- | --- | --- | --- | --- |
| 2026-06-23 | Goal setup | Created the goal documents for map progression DTO cleanup and battle event-log contract rename. | Pending implementation. | Start by re-reading goal docs and policy sources, then inventory affected code paths. |
| 2026-06-23 | Startup protocol | Re-read `PLAN.md`, `EXPERIMENTS.md`, `EXPERIMENT_NOTES.md`, `AUDIT_REPORT.md`, the timeline policy decision, combat result attachment policy, and `refactor_preparation_plan.md` before code search/edit/validation. | Success. The current goal policy supersedes the older "Timeline name may remain" baseline by confirming the rename. | Inventory map progression and event-log contract usage next. |
| 2026-06-23 | Map progression DTO cleanup | Removed `available_node_ids` and `completed_node_ids` from `MapProgressionSnapshotDto` serialization and added snapshot assertions that `map_progression` no longer exposes those lists. | Success. Internal `MapProgression` list storage remains for runtime calculation; external snapshot contract no longer exposes the duplicate lists. | Focused snapshot test and broad lib tests passed. |
| 2026-06-23 | Battle event-log contract rename | Renamed the battle event log module/type/entry/event/cause/validator/fields from `timeline`/`Timeline*` to `event_log`/`BattleEventLog*`/`BattleLogEvent`/`EventLog*` across core, tests, and server transport. | Success. `rg` found no old `timeline`/`Timeline` runtime/server-facing names in `src`, `tests`, or `../game_server/src`. | Keep historical mentions only in goal notes where they describe the migration. |
| 2026-06-23 | Server transport rename | Renamed combat result attachment transport from `compressed_timeline` with `timeline_encoding` / `timeline_gzip_base64` to `compressed_event_log` with `event_log_encoding` / `event_log_gzip_base64`. | Success. `cargo check -p game_server` passed after fixing the server import to `BattleEventLog`. | Unity client will need to consume the new field names. |
| 2026-06-23 | Documentation alignment | Updated `docs/refactor_preparation_plan.md` and current policy sections in `POLICY_DECISIONS.md` to describe `BattleEventLog` / event-log contract names instead of preserving `Timeline`. | Success. Current policy baseline now matches the implemented rename. | Historical goal text may still mention `Timeline` as the old name being removed. |

## Failed Approaches

- Initial focused command `cargo test run_snapshot_exposes_viewing_map_state -- --test-threads=1` matched 0 tests because the actual test name is `run_snapshot_exposes_current_flow_after_start`. Re-ran with the correct test name and it passed.
- Initial focused command `cargo test event_log_validation_rejects_legacy_event_log_versions -- --test-threads=1` matched 0 tests because the actual test name is `normal_validation_rejects_legacy_event_log_versions`. Re-ran with the correct test name and it passed.

## Validation Commands

- `cargo check --lib` - passed. Existing warning: `/mnt/f/work/simulator/auth_server/Cargo.toml: unused manifest key: env`.
- `cargo test --tests --no-run` - passed; all integration test targets compiled.
- `cargo test run_snapshot_exposes_viewing_map_state -- --test-threads=1` - passed compilation but matched 0 tests; replaced by the correct focused test below.
- `cargo test event_log_validation_rejects_legacy_event_log_versions -- --test-threads=1` - passed compilation but matched 0 tests; replaced by the correct focused test below.
- `cargo test ability_step_event_log_includes_presentation_metadata --test skill_refactor_validation -- --test-threads=1` - passed, 1 test.
- `cargo test run_snapshot_exposes_current_flow_after_start -- --test-threads=1` - passed, 1 test.
- `cargo test normal_validation_rejects_legacy_event_log_versions -- --test-threads=1` - passed, 1 test.
- `cargo fmt` - passed.
- `cargo test --test ron_loading -- --test-threads=1` - passed, 18 tests.
- `cargo test --test live_skill_catalog_audit -- --test-threads=1` - passed, 3 tests.
- `cargo check -p game_server` - passed. Existing warning: `/mnt/f/work/simulator/auth_server/Cargo.toml: unused manifest key: env`.
- `cargo test --lib -- --test-threads=1` - passed, 519 tests.
- `git diff --check` - passed.

## Completion Evidence

- `rg -n "Timeline|timeline|TIMELINE|compressed_timeline|has_timeline" src tests ../game_server/src` returns no matches.
- `rg -n "available_node_ids|completed_node_ids" src/game/behavior.rs src/game/world/snapshot.rs src/game/world/tests/snapshots_and_start.rs ../game_server/src/game/player_game_actor` returns only snapshot absence assertions and server-local variables derived from `MapNodeDto.state`.
