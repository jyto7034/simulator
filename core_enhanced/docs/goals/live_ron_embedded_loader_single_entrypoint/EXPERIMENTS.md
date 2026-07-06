# Live RON Embedded Loader Single Entrypoint Experiments

| Date | Attempt | Result | Notes |
| --- | --- | --- | --- |
| 2026-07-04 | Created goal from refactor audit decisions. | Planned. | No code changes yet. |
| 2026-07-04 | Re-read `PLAN.md`, `EXPERIMENTS.md`, `EXPERIMENT_NOTES.md`, then inspected server/test live RON loaders. | Found three duplicated loader paths. | `game_server/src/main.rs`, `tests/common/mod.rs`, and `src/game/world/tests/mod.rs` assembled embedded RON separately; server path also ran combat preview validation while test paths did not. |
| 2026-07-04 | Added `GameDataBase::load_live_embedded()` and delegated server/test live loaders to it. | Implemented. | The official loader keeps `include_str!`, loads `run/policy.ron`, and keeps generated combat preview validation enabled. |
| 2026-07-04 | Ran `cargo check -p game_core` and `cargo check -p game_server`. | Passed. | Formatting also completed with `cargo fmt`. |

## Failure Log Template

| Date | Attempt | Failure | Fix | Revalidation |
| --- | --- | --- | --- | --- |
| 2026-07-04 | `cargo test -p game_core --test ron_loading -- --test-threads=1` | The official loader exposed a live combat preview validation failure: `invalid battlefield instance: spawn wave references missing spawn zone`. | Investigated the encounter/template mismatch without weakening validation. | Resolved by the later spawn-zone authoring/test update. |
| 2026-07-04 | `cargo test -p game_core --test live_skill_catalog_audit -- --test-threads=1` | Same loader-time combat preview validation failure. | Same fix path as above. | Resolved by the later spawn-zone authoring/test update. |
| 2026-07-04 | Added encounter/seed context to combat preview validation panic. | Compile failed because `GameError` does not implement `Display`. | Switched validation message formatting to `Debug` (`{:?}`). | `cargo check -p game_core`: passed. |
| 2026-07-04 | Re-ran live tests after removing the brittle `north_west_entry` authoring from `suppress_nameless_fetus`. | `live_skill_catalog_audit` passed; `ron_loading` exposed a stale test expectation for the removed authored spawn zone. | Updated the test to assert that generated preview spawn waves resolve to existing spawn zones. | `cargo test -p game_core --test ron_loading -- --test-threads=1`: passed. `cargo test -p game_core --test live_skill_catalog_audit -- --test-threads=1`: passed. |
| 2026-07-04 | Completion audit reran all validation commands from `PLAN.md`. | Passed. | Confirmed current state after document updates. | `cargo check -p game_core`, `cargo check -p game_server`, `cargo test -p game_core --test ron_loading -- --test-threads=1`, and `cargo test -p game_core --test live_skill_catalog_audit -- --test-threads=1` all passed. |
