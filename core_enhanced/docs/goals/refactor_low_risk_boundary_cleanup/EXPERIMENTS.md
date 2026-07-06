# Refactor Low-Risk Boundary Cleanup Experiments

| Date | Attempt | Result | Notes |
| --- | --- | --- | --- |
| 2026-07-04 | Created goal from refactor audit decisions. | Planned. | No code changes yet. |
| 2026-07-06 | Runtime/source re-read before editing. | Success. | Confirmed shop hidden stock leaked through `SelectedEventSnapshotDto::Shop`, boss omen distance used stack traversal, `ApplyBossOmenStepResult` was no-op, and battle records dedupe by abnormality was behaviorally codex-like. |
| 2026-07-06 | Shop selected-event DTO cleanup. | Success. | Removed `hidden_items` and `hidden_item_uuids` from Unity-facing selected-event DTO/projection while keeping `ShopSessionState.hidden_items` and reroll state intact. |
| 2026-07-06 | Boss omen distance cleanup. | Success. | Replaced terminal-distance traversal with BFS shortest path and added a branching graph regression test. |
| 2026-07-06 | `ApplyBossOmenStepResult` fail-fast. | Success. | Runtime now panics on the unimplemented effect. Live `white_night_confession_01` RON no longer contains this marker because boss omen step consumption belongs to node completion. |
| 2026-07-06 | Battle record codex clarification. | Success. | Documented `battle_records` as one representative abnormality codex/observation record per `abnormality_uuid`, not a per-battle timeline archive. |

## Required Experiment Log

Record each focused test/check here.

| Date | Attempt | Failure | Fix | Revalidation |
| --- | --- | --- | --- | --- |
| 2026-07-06 | `cargo fmt --check` | Import formatting in `src/game/boss_omen.rs`. | Ran `cargo fmt`. | Formatting succeeded before final validation. |
| 2026-07-06 | `rg -n "ApplyBossOmenStepResult" ../game_resources/data src tests docs -S` | Live event RON still used `ApplyBossOmenStepResult` in `white_night_confession_01`. | Removed the unimplemented marker from live event choices; kept the runtime fail-fast branch and focused test. | `rg -n "ApplyBossOmenStepResult" ../game_resources/data src/game/world/event_node.rs src/game/world/tests/map_flow.rs -S` now shows only runtime/test references. |
| 2026-07-06 | `cargo test -p game_core shop --lib -- --test-threads=1` | None. | None. | Passed: 13 tests. |
| 2026-07-06 | `cargo test -p game_core boss_omen --lib -- --test-threads=1` | None. | None. | Passed: 9 tests. |
| 2026-07-06 | `cargo test -p game_core event_node --lib -- --test-threads=1` | None. | None. | Passed: 4 tests. |
| 2026-07-06 | `cargo test -p game_core snapshot --lib -- --test-threads=1` | None. | None. | Passed: 23 tests. |
| 2026-07-06 | `cargo test -p game_core --test ron_loading -- --test-threads=1` | None. | None. | Passed: 18 tests. |
| 2026-07-06 | `cargo check -p game_core` | None. | None. | Passed. |
