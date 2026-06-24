# SavePoint Checkpoint Support Experiments

| Date | Attempt | Result | Notes |
| --- | --- | --- | --- |
| 2026-06-23 | Initial evidence scan for Medical/SavePoint runtime drift. | Success. | Docs already describe SavePoint, but runtime code still contains Medical support type, treatment enum, Medical target/treatment selection, Medical HP heal, and no-deployable Medical fallback logic. |
| 2026-06-23 | Goal document creation. | Success. | Created this goal as a feature implementation plan, not a rename-only cleanup. |
| 2026-06-24 | Replace Medical support flow with SavePoint/checkpoint runtime and focused support tests. | Success. | `cargo test -p game_core game::world::tests::support -- --nocapture` passed 8 tests. Tests cover no target/treatment DTO fields, no HP heal, living-only trauma reduction, checkpoint save/load metadata, and 3-load limit. |
| 2026-06-24 | Validate map flow, snapshot/start, action scheduling, combat recovery, and live RON. | Success. | Focused tests passed for `map_flow`, `snapshots_and_start`, `action_scheduler`, `combat`, and `--test ron_loading`. Combat test covers no-deployable recovery through checkpoint-load availability instead of future Medical nodes. |
| 2026-06-24 | Broad game_core validation. | Success. | `cargo test -p game_core` passed: 529 unit tests, integration tests, skill tests, and doc tests. |
| 2026-06-24 | Server compile and live WebSocket smoke probe. | Success after one setup correction. | Running `game_server` from `core_enhanced` failed because the relative `config/development` file was not found. Re-running from `/mnt/f/work/simulator/game_server` succeeded, and `WS_PORT=18083 python3 '/mnt/f/unity projects/ark/docs/probe/unity_ws_smoke_probe.py'` passed with `snapshot_root_keys` including `run_checkpoint`. |
| 2026-06-24 | Dedicated live SavePoint WebSocket probe. | Success after probe hardening. | Added `/mnt/f/unity projects/ark/docs/probe/unity_savepoint_checkpoint_probe.py`. First draft missed the required auth frame; second draft treated `choose_support` as snapshot-producing; final probe passes and verifies SavePoint entry/completion, `run_checkpoint` after save, and `load_run_checkpoint` result with `loads_used=1`, `remaining_loads=2`. |

## Completed Experiment Coverage

- SavePoint completion proves trauma decreases for living employees only and HP is unchanged.
- Checkpoint save/load tests cover user-visible run state restore and maximum 3 loads per run.
- DTO/snapshot tests prove Medical target/treatment fields are absent and `run_checkpoint` metadata is exposed.
- Combat recovery tests prove no-deployable recovery uses checkpoint-load availability instead of future support-node healing.
- Live RON loading proves the new SavePoint node and run policy schema load from data.
- Live WebSocket probe proves the server emits the new snapshot root shape, including `run_checkpoint`.
- Dedicated SavePoint WebSocket probe proves live support node selection, completion, and checkpoint load behavior.

## Failure Log Template

When a test or probe fails, append:

| Date | Attempt | Failure | Fix | Revalidation |
| --- | --- | --- | --- | --- |
| YYYY-MM-DD | command/test | what failed | what changed | command/result |
