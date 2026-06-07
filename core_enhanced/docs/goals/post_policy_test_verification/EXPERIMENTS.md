# Post Policy Test Verification Experiments

## 2026-06-07 - Goal Start

Actions:

- Created post-policy test verification work memory directory.
- Created initial `PLAN.md`, `EXPERIMENTS.md`, `EXPERIMENT_NOTES.md`, and `REPORT.md`.

Result:

- Audit goal started.
- No production/test/server/live RON code changes.

## 2026-06-07 - Compile Baseline

Commands:

- `cargo check -p game_core`
- `cargo check -p game_server`

Result:

- `game_core` check passed.
- `game_server` check passed.
- The server check result is not enough to prove test health because the later `game_server` test build fails under `cfg(test)`.

## 2026-06-07 - Test Inventory And Zero-Test Filters

Commands:

- `cargo test -p game_core -- --list`
- `cargo test -p game_core damage_feedback -- --list`
- `cargo test -p game_core ron_loading -- --list`
- `cargo test -p game_core skill_refactor_validation -- --list`
- `cargo test -p game_core skill_test_suite -- --list`
- `cargo test -p game_core feedback_tags -- --list`
- `cargo test -p game_core combat_preview -- --list`
- `cargo test -p game_core retreat -- --list`
- `cargo test -p game_core consumable -- --list`
- `cargo test -p game_core movement -- --list`
- `cargo test -p game_core airborne -- --list`
- `cargo test -p game_core targeting -- --list`
- `cargo test -p game_core skill_fragment -- --list`
- `cargo test -p game_core fixed_defense -- --list`

Result:

- Full game_core inventory:
  - lib target: 435 tests.
  - `tests/live_item_skill_activation.rs`: 3 tests.
  - `tests/live_skill_catalog_audit.rs`: 3 tests.
  - `tests/ron_loading.rs`: 16 tests.
  - `tests/skill_refactor_validation.rs`: 10 tests.
  - `tests/skill_test_suite.rs`: 15 tests.
  - `tests/unit_test.rs`: 0 tests.
  - doctests: 0 tests.
- Confirmed current zero-test filters:
  - `damage_feedback`: 0 tests.
  - `ron_loading`: 0 tests when used as a filter instead of `--test ron_loading`.
  - `skill_refactor_validation`: 0 tests when used as a filter instead of `--test skill_refactor_validation`.
  - `skill_test_suite`: 0 tests when used as a filter instead of `--test skill_test_suite`.
- Current focused policy-area inventory:
  - `feedback_tags`: 7 tests.
  - `combat_preview`: 24 lib tests plus 1 `ron_loading` integration test.
  - `retreat`: 2 tests.
  - `consumable`: 16 lib tests plus 1 `ron_loading` integration test.
  - `movement`: 61 tests.
  - `airborne`: 17 lib tests plus 1 `ron_loading` integration test.
  - `targeting`: 8 lib tests plus 1 `skill_refactor_validation` integration test.
  - `skill_fragment`: 29 lib tests plus 2 `ron_loading` integration tests.
  - `fixed_defense`: 17 tests.

## 2026-06-07 - Integration Test Execution

Commands:

- `cargo test -p game_core --test ron_loading`
- `cargo test -p game_core --test skill_refactor_validation`
- `cargo test -p game_core --test skill_test_suite`
- `cargo test -p game_core --test live_item_skill_activation`
- `cargo test -p game_core --test live_skill_catalog_audit`
- `cargo test -p game_core --test unit_test`

Result:

- `ron_loading`: passed, 16 tests.
- `skill_refactor_validation`: passed, 10 tests.
- `skill_test_suite`: passed, 15 tests.
- `live_item_skill_activation`: passed, 3 tests.
- `live_skill_catalog_audit`: passed, 3 tests.
- `unit_test`: command succeeded but ran 0 tests, so it is suspicious rather than useful passing evidence.

## 2026-06-07 - Full Game Core Test

Command:

- `cargo test -p game_core`

Result:

- Failed.
- lib target result: 434 passed, 1 failed, 0 ignored.
- Failing test: `game::events::combat::tests::authored_protect_unit_tactical_plan_overrides_default_defense_contract`.
- Panic location: `core_enhanced/src/game/combat_preview.rs:737`.
- Error: `combat preview battlefield should be valid: InvalidStaticData("invalid battlefield instance: spawn wave references missing route")`.

Cause analysis:

- The failing fixture defines a `PveWaveData.route_id` of `black_box_breach_main` in `src/game/events/combat.rs`.
- `spawn_waves_for` copies authored `wave.route_id` directly before fallback route selection.
- `validate_instance` rejects spawn waves whose route id is not present in the generated battlefield instance.
- This looks like a stale/brittle test fixture after route validation became stricter, not a user policy question.

## 2026-06-07 - Game Server Test Build

Command:

- `cargo test -p game_server -- --list`

Result:

- Failed to compile server test target.
- `game_server/src/game/player_game_actor/handlers.rs:350`: test fixture initializes `AbnormalityMetadata` without `mobility_kind` and `target_traits`.
- `game_server/src/game/player_game_actor/state.rs:750` and `state.rs:789`: test fixtures initialize `LiveBattleDeploymentDto` without `unit_deploy_costs`.

Cause analysis:

- Production `cargo check -p game_server` passes, but test-only fixture code is stale.
- The missing fields are current Unity-facing/core contracts, so this creates a test coverage gap for server command/snapshot DTO serialization.

## 2026-06-07 - Ignored Test Search And Unity Contract Scan

Commands:

- `rg -n "#\\[ignore\\]|ignore\\]" src tests ../game_server/src`
- `rg -n "feedback_tags|threat_warnings|unit_deploy_costs|effective_deploy_cost|mobility_kind|target_traits|skill_fragment|compatibility|armor|magic_resist|DamageType" "/mnt/f/unity projects/ark/docs/unity_core_contract.md" "/mnt/f/unity projects/ark/docs/unity_client_implementation_goal.md"`

Result:

- No ignored tests were found by the `rg` search.
- External canonical Unity docs include current contracts for:
  - `feedback_tags`
  - `threat_warnings`
  - `unit_deploy_costs[*].effective_deploy_cost`
  - `mobility_kind`
  - skill fragment compatibility snapshot data
  - physical/magic/true damage display contract

## 2026-06-07 - Documentation Sanity

Commands:

- conflict-marker search over `docs/goals/post_policy_test_verification`
- leftover-placeholder search over `docs/goals/post_policy_test_verification`
- `rg -n "[[:blank:]]$" docs/goals/post_policy_test_verification/*.md`
- `git status --short docs/goals/post_policy_test_verification`

Result:

- No conflict markers were found.
- No leftover pending placeholders were found.
- No trailing whitespace was found.
- The goal directory is currently untracked: `?? docs/goals/post_policy_test_verification/`.
