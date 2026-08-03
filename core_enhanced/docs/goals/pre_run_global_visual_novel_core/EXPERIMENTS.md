# Pre-Run Global Visual Novel Core Experiments

| Date | Attempt | Result | Notes |
| --- | --- | --- | --- |
| 2026-07-09 | Reuse audit for existing goal docs. | Success. | `global_visual_novel_engine` is useful background but too broad to reuse directly. It also contains older wording that active/queued VN sessions are run runtime state. This goal starts a focused follow-up for top-level VN runtime ownership and `StartNewGame -> PreRunBriefing -> SelectingStarterEmployees`. |
| 2026-07-09 | Current code boundary scan. | Success. | `StartNewGame` currently transitions directly to `SelectingStarterEmployees`; `RunState` is created only after `SelectStarterEmployees`; VN runtime is currently owned by `RunState.visual_novel`; VN snapshot, VN commands, action gating, battle pause/transport tests all currently read or mutate `run.visual_novel`. This confirms pre-run HQ briefing cannot be implemented cleanly without moving VN runtime ownership. |
| 2026-07-09 | Temporary design note review. | Success. | The note now preserved as `docs/decisions/pre_run_global_visual_novel_core_2026-07-09.md` correctly identified the need for VN runtime outside `RunState`, but its dual-owner fallback (`global_visual_novel` before `run.visual_novel`) is not the preferred final architecture. This goal fixes the target to a single canonical top-level `GameCoreState.visual_novel`. |
| 2026-07-09 | Goal document creation. | Success. | Created `PLAN.md`, `EXPERIMENTS.md`, and `EXPERIMENT_NOTES.md`. No runtime code was changed. |
| 2026-07-09 | PLAN versus runtime code validation. | Success, with notes. | The plan is directionally correct: current VN runtime is still run-owned, `StartNewGame` returns starter candidates immediately, VN commands require `RunState`, snapshot/action gating/battle pause read `run.visual_novel`, and checkpoint tests currently preserve active VN. Added policy notes: HQ briefing live VN ids must exist before Phase 3 production activation; missing HQ VN data should fail rather than skip; VN progression outcomes must carry completed session metadata before queue promotion; checkpoint VN restore tests should be replaced unless an explicit rewind policy is approved. |
| 2026-07-09 | Phase 0 re-read before implementation. | Success. | Re-read the current `PLAN.md`, `EXPERIMENTS.md`, and `EXPERIMENT_NOTES.md` before code search as required. Runtime scan reconfirmed the same implementation boundary: `GameCoreState` has no top-level VN, `RunState` and `RunCheckpointPayload` carry VN, `visual_novel_snapshot_dto`, action gating, battle pause, and battle transport read run-owned VN, and focused tests seed `run.visual_novel`. Existing worktree is dirty, so implementation must preserve unrelated edits and work with the current state. |
| 2026-07-09 | Phase 1-3 implementation. | Success. | Moved canonical VN runtime to `GameCoreState.visual_novel`, removed `RunState.visual_novel` and checkpoint-owned VN restore, changed snapshot/action gating/battle pause/battle update projection to the top-level runtime, added `PreRunBriefing { game_mode }`, changed `StartNewGame` to return `RunStartFlowStarted`, added mode-specific HQ briefing VN live data, and added focused tests for pre-run VN flow, missing live data, top-level null projection, battle projection, and checkpoint behavior. |
| 2026-07-09 | Focused core verification. | Success. | `cargo check`, `cargo test game::world::tests::snapshots_and_start --lib -- --test-threads=1`, and `cargo test visual_novel --lib -- --test-threads=1` passed after the implementation. |
| 2026-07-09 | Broad core verification. | Success after fixture fix. | `cargo test --lib -- --test-threads=1` passed with 631 tests after custom test fixtures that rebuild `GameDataBase` were updated to preserve `visual_novel_data`. |
| 2026-07-09 | Contract document update. | Success. | Updated local core runtime, game-flow maps, game_server Unity bridge protocol, and the temporary design note to document `StartNewGame -> PreRunBriefing -> visual_novel -> SelectingStarterEmployees`, top-level VN ownership, `RunStartFlowStarted`, and the no-run/no-map pre-starter snapshot policy. |
| 2026-07-09 | Server actor contract update. | Success. | Updated server actor tests to expect `RunStartFlowStarted` and `pre_run_briefing` after `start_new_game`. The server command-result mapper already uses the core payload contract generically, so no compatibility layer or special-case mapping was needed. |
| 2026-07-09 | Server live-data fixture repair. | Success. | Server live-battle tests now build defense fixtures from embedded live data and only apply survival timer overrides to live defense encounters. This preserves normal combat, reward, equipment, corroded wave, visual novel, and map references instead of constructing a narrow invalid fixture. |
| 2026-07-09 | Server startup smoke. | Partial. | `cargo run` in `../game_server` compiled, loaded embedded live game data, and logged `Game Server is running on 0.0.0.0:8081`. `curl` probes to `127.0.0.1:8081/health` and `/ready` failed to connect from a separate exec session, and `ss` could not inspect sockets due sandbox permission. The server shut down gracefully on Ctrl+C. |
| 2026-07-09 | Final verification. | Success. | `cargo test --lib -- --test-threads=1` passed in `core_enhanced` with 631 tests. `cargo test --lib -- --test-threads=1` passed in `../game_server` with 25 tests. |

## Known Current Dependencies To Re-Check During Phase 0

- `src/game/world/state.rs`
  - `GameCoreState` currently has no top-level VN runtime.
  - `RunState` owns `visual_novel`.
  - `RunCheckpointPayload` carries `visual_novel`.
- `src/game/world.rs`
  - `handle_start_new_game` prepares starter candidates and transitions directly
    to `SelectingStarterEmployees`.
  - `handle_select_starter_employees` consumes `pending_game_mode` and creates
    `RunState`.
- `src/game/world/visual_novel_flow.rs`
  - VN command handling reads active session from `run.visual_novel`.
  - VN command mutation uses `run_state_mut()?.visual_novel`.
- `src/game/world/snapshot.rs`
  - `visual_novel_snapshot_dto` projects from `run.visual_novel`.
- `src/game/world/helpers.rs`
  - allowed-action VN context is computed from `run.visual_novel`.
- `src/game/world/combat.rs`
  - blocking VN battle pause behavior currently checks `run.visual_novel`.
- `src/game/world/node_flow.rs`
  - run checkpoint save/load currently preserves `visual_novel` as part of
    `RunState`.
- `src/game/world/tests/snapshots_and_start.rs` and
  `src/game/world/tests/combat.rs`
  - focused tests currently seed active VN sessions through
    `core.state.run.as_mut().unwrap().visual_novel`.
- `../game_resources/data/visual_novels/base.ron`
  - current live catalog only contains `white_night_confession_intro`; this goal
    must add `hq_briefing_standard_intro` and `hq_briefing_endless_intro` before
    the `StartNewGame -> PreRunBriefing` production path depends on them.

## Failure Log Template

When a test, probe, or trial implementation fails, append:

| Date | Attempt | Failure | Fix | Revalidation |
| --- | --- | --- | --- | --- |
| 2026-07-09 | `cargo test live_battle_update_projects_active_visual_novel_overlay blocking_visual_novel_pauses_live_battle_tick_and_uses_battle_update_transport --lib -- --test-threads=1` | Cargo treated the second test name as an unexpected argument. | Re-ran a valid focused filter: `cargo test visual_novel --lib -- --test-threads=1`. | Passed, 13 tests. |
| 2026-07-09 | First `cargo test --lib -- --test-threads=1` after implementation. | 11 tests failed because custom fixtures built from `GameDataBuilder::empty()` omitted the newly required HQ briefing VN live data. | Updated fixture builders that clone production-like PvE data to also preserve `visual_novel_data`. | Re-ran `cargo fmt && cargo test --lib -- --test-threads=1`; passed, 631 tests. |
| 2026-07-09 | First broad `../game_server` lib test after server contract update. | Five live-battle actor tests failed because the server's narrow defense fixture no longer satisfied production map/PvE assumptions after the pre-run flow began using the full starter-selection path. | Rebuilt the defense fixture from embedded live data and applied only the survival timer override needed by the survival test. | Re-ran `cargo test --lib -- --test-threads=1` in `../game_server`; passed, 25 tests. |
| 2026-07-09 | Server health/ready curl after `cargo run`. | Server process logged successful startup and live data load, but separate `curl` commands could not connect to `127.0.0.1:8081`; socket inspection with `ss` was blocked by sandbox permissions. | Recorded as a sandbox/probe limitation, not a runtime startup failure. Stopped the server with Ctrl+C and confirmed graceful shutdown logs. | Server actor tests and startup log remain the verification for this environment. |
