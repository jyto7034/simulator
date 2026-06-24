# Allowed Actions Policy Experiments

## 2026-06-12

- Created goal working directory and memory files.
- Initial plan: inspect current `ActionScheduler` and world context adjustments before editing.
- Moved reward skip and maintenance action adjustments into `ActionScheduler::get_allowed_actions_for_context`.
- Updated `GameCore::allowed_actions_for_state_context` to pass `AllowedActionContext` instead of owning adjustment logic.
- Added unit tests for reward skip context and maintenance context.
- Ran `cargo test -p game_core managers::action_scheduler::tests -- --nocapture`: passed, 12 tests.
- Attempted `cargo test -p game_core world::tests::snapshots_and_start world::tests::support -- --nocapture`: failed because Cargo accepts only one test name filter. Re-run the filters separately.
- Ran `cargo test -p game_core world::tests::snapshots_and_start -- --nocapture`: passed, 7 tests.
- Ran `cargo test -p game_core world::tests::support -- --nocapture`: passed, 9 tests.
- Ran `cargo test -p game_core world::tests::equipment -- --nocapture`: passed, 20 tests.
- Ran `cargo fmt`: completed.
- Ran `cargo check -p game_core`: passed.
