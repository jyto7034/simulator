# Boss Omen Chain Experiments

## Experiment Log

| Date | Experiment | Result | Next Action |
| --- | --- | --- | --- |
| 2026-06-26 | Goal creation | Created as the last dependent Endless goal. No runtime implementation should start until prerequisite goals settle. | Revisit after progression, research, bonus objective, and repeat weighting goals are complete. |
| 2026-06-27 | Phase 5 startup document audit | Master plan, this goal's PLAN/EXPERIMENTS/EXPERIMENT_NOTES, prerequisite goal plans, and `docs/boss_omen_chain_policy_draft.md` were read before code search or implementation. The old normal-combat-only first implementation note is superseded by the active Event/Combat source-kind policy draft. | Audit runtime code and live RON against the policy draft before editing code. |
| 2026-06-27 | First `cargo check --lib` after adding Event/BossOmen data/runtime skeleton | Failed on mechanical integration issues: missing `MapNodeOmenOverlayDto` re-export, manual `MapNode` fixtures missing `omen`, new `BehaviorResult`/`PlayerBehavior`/`ActiveNodeContent` variants not covered, and Event handler borrow conflicts. No policy contradiction found. | Patch integration surfaces and rerun `cargo check --lib`. |
| 2026-06-27 | Second/third `cargo check --lib` after integration fixes | Passed. Event/BossOmen data models, map Event category, run-local omen state, Event command surface, and core snapshot/result wiring compile in the core library. | Check game_server loader and add focused behavior tests. |
| 2026-06-27 | `cargo test --lib --no-run` after adding `omen_chain_id` and `omen` DTO fields | Failed because test-only `AbnormalityMetadata` and `MapNode` fixtures did not include the new fields. This was a mechanical fixture update, not a policy failure. | Add `omen_chain_id: None` and `omen: None` to manual fixtures and rerun. |
| 2026-06-27 | `cargo test --lib --no-run` after fixture updates | Passed. The full lib test binary compiles with the new Event/BossOmen DTO fields. | Add focused behavior tests. |
| 2026-06-27 | Focused forced boss node test | Passed. A completed boss omen chain now creates a separate Boss node, sets it as terminal, disables other available nodes, and leaves only the forced boss selectable. | Keep as the core contract test for post-chain boss forcing. |
| 2026-06-27 | Focused Event node command test | Initially failed because the live event graph uses a final `end` scene after choice selection rather than ending immediately. After updating the test to advance the terminal scene, it passed. | Keep Event flow as `enter -> advance -> select choice -> advance terminal scene -> complete`. |
| 2026-06-27 | Focused Event generation test | Passed. Generated maps now guarantee at least one Event node across sampled seeds. | Keep as coverage for the per-Floor Event node policy. |
| 2026-06-27 | Focused boss omen overlay/deferral tests | Passed. Active chains overlay matching Event nodes with hint-only map data, and defer without mutating the map when the required source kind is absent. | Keep as coverage for placement and missing-candidate policy. |
| 2026-06-27 | Tried to run three focused tests in one `cargo test` command | Failed because `cargo test` accepts one test-name filter in that position. This was a command invocation error, not a code/test failure. | Rerun focused tests separately or use a broader module filter. |
| 2026-06-27 | Final focused test reruns | Passed separately: forced boss node, Event command flow, and generated Event node coverage. | Run broad compile checks. |
| 2026-06-27 | Final broad compile checks | Passed: `cargo test --lib --no-run`, `cargo check -p game_server`, and full `cargo test --lib` with 574 passed. | Ready for goal completion review. |
| 2026-06-27 | `cargo check -p game_server` | Passed. The server loader compiles with new live RON includes for Event and BossOmen chain data. | Run final focused/broad checks before closing. |

## Verification Commands

Record every focused and broad check here while implementing.

Potential future checks:

```bash
cargo test --lib completed_boss_omen_chain_forces_single_boss_node_ahead_of_current_room
cargo test --lib event_node_commands_advance_scene_select_choice_and_complete_node
cargo test --lib generated_map_always_contains_an_event_node
cargo test --lib active_boss_omen_chain_overlays_matching_event_node
cargo test --lib active_boss_omen_chain_defers_when_required_source_node_is_missing
cargo test --lib --no-run
cargo check -p game_server
cargo test --lib
cargo test boss_omen --lib
cargo test endless --lib
cargo test map_encounters --lib
cargo test combat_result --lib
cargo test ron_loading --test ron_loading
cargo check --lib
```

## Failed Attempts

- Initial `cargo test --lib --no-run` failed on fixture-only missing fields after DTO expansion. Fixed by updating fixtures rather than adding compatibility/default runtime paths.
- First Event command test expected immediate completion after choice selection. Live Event RON intentionally advances to an `end` scene first, so the test was corrected to match the scene graph contract.
- One combined focused-test command failed because multiple test filters were passed where Cargo accepts one. Reran the three focused tests separately.
