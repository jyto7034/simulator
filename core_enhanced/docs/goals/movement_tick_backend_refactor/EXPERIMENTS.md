# Movement Tick Backend Refactor Experiments

## 2026-06-12

- Created goal working directory and memory files.
- Inspected Direct and Rapier tick loops.
- Decision: keep Direct as deterministic test harness and extract shared tick control flow.
- Added shared `run_continuous_movement_tick` loop in `movement/engine.rs`.
- Added backend hook trait for prepare/correct/apply responsibilities.
- Routed Direct and Rapier `MovementEngine::tick` implementations through the shared loop.
- Removed obsolete Direct movement candidate type and Rapier-local BodyMoved helper.
- Ran `cargo check -p game_core`: passed after removing obsolete helper/type.
- Ran `cargo test -p game_core battle::core::movement -- --nocapture`: passed, 55 tests.
- Ran `cargo test -p game_core battle::core::tests -- --nocapture`: passed, 44 tests.
- Ran `cargo fmt`: completed.
- Ran `cargo check -p game_core`: passed.
