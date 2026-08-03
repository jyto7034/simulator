# Visual Novel Trigger Engine Experiments

| Date | Attempt | Result | Notes |
| --- | --- | --- | --- |
| 2026-07-09 | Reviewed current VN runtime ownership. | Success. | Current code uses `GameCoreState.visual_novel` as canonical owner with `active` and `queue`. Root snapshot and battle update projection already read from this top-level runtime. This is the correct base for trigger activation. |
| 2026-07-09 | Reviewed current hard-coded run-start VN activation. | Success. | `handle_start_new_game` still calls `activate_hq_briefing_visual_novel(game_mode)` directly. Trigger engine should replace this with `RunStart { game_mode }` evaluation and live trigger data. |
| 2026-07-09 | Reviewed data loading and validation path. | Success. | Live VN definitions load from `game_resources/data/visual_novels/base.ron`. There is no trigger catalog. New trigger data should be added to `GameDataBase`, builder plumbing, embedded live loading, and cross-reference validation. |
| 2026-07-09 | Reviewed allowed action and battle pause policy. | Success. | Blocking VN already gates allowed actions to VN progression and pauses later battle ticks. Trigger engine only needs to create/queue sessions; existing projection and gating policies can remain. |
| 2026-07-09 | Reviewed battle update insertion point. | Success. | Battle results are built first and then VN projection is attached. Battle trigger evaluation should happen after update construction and before transport response is sent, so a newly activated VN appears in the same `battle_update.visual_novel`. |
| 2026-07-09 | Reviewed older `global_visual_novel_engine` docs. | Mixed/historical. | Useful for trigger catalog direction, but old checkpoint/run ownership text conflicts with current top-level VN policy. This goal follows `pre_run_global_visual_novel_core` and runtime code instead. |
| 2026-07-09 | Phase 1 trigger catalog implementation. | Success. | Added `visual_novel_trigger_data`, `VisualNovelTriggerDatabase`, live RON loading, builder plumbing, and validation from triggers to VN definitions. Live RON now contains Standard/Endless RunStart HQ briefing triggers. |
| 2026-07-09 | Phase 2 trigger runtime state. | Success. | Added top-level `VisualNovelTriggerRuntimeState` with separate persistent and checkpoint-rewind consumed sets. Run checkpoint payload stores only `RewindWithRunCheckpoint` history; persistent history survives checkpoint load. |
| 2026-07-09 | Phase 3 evaluation engine. | Success. | Added `VisualNovelTriggerEvent` and trigger evaluation that matches source, applies repeat/history, orders by priority then id, creates a `VisualNovelSession`, activates or queues it, and marks consumed after activation/queue. |
| 2026-07-09 | Phase 4 RunStart replacement. | Success. | Removed hard-coded HQ briefing VN activation. `StartNewGame(game_mode)` now evaluates `RunStart { game_mode }`; missing RunStart trigger fails as invalid static data instead of silently skipping. |
| 2026-07-09 | Phase 5 Event/Map hooks. | Success. | `EventSceneEntered` is evaluated on event node entry and scene changes. `MapNodeEntered` is evaluated after confirmed node entry. Event node selected state and global VN overlay remain separate. |
| 2026-07-09 | Phase 6 Battle hooks. | Success. | `BattleStarted` and `BattleTimeReached` are evaluated from produced battle state before `battle_update.visual_novel` is attached. Time triggers use threshold crossing and do not refire on later ticks. |
| 2026-07-09 | Phase 7 docs update. | Success. | Updated `docs/core_runtime_contract.md` and `docs/maps/game-flow.md` with trigger-driven VN runtime policy. Unity-facing DTO shape did not change, so server/probe docs did not need a new transport example. |

## Failed Or Rejected Approaches

Rejected before implementation:

- Adding `vn_on_enter` or similar fields to Event scenes.
  - Reason: creates route-specific shortcuts and fragments VN activation policy.
- Exposing trigger id/repeat/priority/checkpoint policy in Unity DTOs.
  - Reason: Unity only needs active VN projection, not core trigger internals.
- Single global consumed history for every repeat policy.
  - Reason: long-term checkpoint behavior needs trigger-level
    `checkpoint_policy`.
- Interrupting active VN sessions in the first implementation.
  - Reason: interruption semantics for choices, completion hooks, and Unity
    presentation are a separate UX policy.

Failed during implementation:

- `cargo test live_game_data_loads_run_start_visual_novel_triggers missing_run_start_visual_novel_trigger_fails_start_new_game run_checkpoint_rewinds_only_checkpoint_scoped_visual_novel_trigger_history --lib -- --test-threads=1`
  - Result: cargo rejected multiple test filter arguments.
  - Fix: reran focused tests by module/filter instead of passing several names.
- First `cargo check -q`
  - Result: borrow checker rejected holding immutable trigger refs from
    `game_data` while mutating `GameCore` to create/queue VN sessions.
  - Fix: collect matched triggers as cloned values before evaluation.
- First map-flow trigger test compile
  - Result: used the wrong local `event_choice_definition` signature.
  - Fix: passed the expected immediate-effect vector and reran map-flow tests.

## Validation Log

Passed:

```text
cargo check -q
cargo test visual_novel_trigger_data --lib -- --test-threads=1
cargo test snapshots_and_start --lib -- --test-threads=1
cargo test support --lib -- --test-threads=1
cargo test map_flow --lib -- --test-threads=1
cargo test combat --lib -- --test-threads=1
cargo test --lib -- --test-threads=1
```

Final broad result:

```text
640 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

Not run:

- `game_server` tests and live WebSocket probe. Reason: trigger internals did
  not change Unity-facing DTO or server transport shape, and no server code was
  intentionally changed in this goal.
