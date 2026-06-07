# Post Policy Code Repair Master Experiments

## 2026-06-07 - Goal Start

Actions:

- Read `docs/goal.md`.
- Read `docs/post_policy_code_repair_master_goal.md`.
- Read post-policy test verification and technical debt reports.
- Created master work memory files.

Result:

- Phase 1 must run before Phase 2.
- Current worktree is dirty; existing changes are treated as current/user work.
- No production code changed yet.

## 2026-06-07 - Phase 1 Baseline

Commands:

- `cargo test -p game_core authored_protect_unit_tactical_plan_overrides_default_defense_contract -- --nocapture`
- `cargo test -p game_server -- --list`

Result:

- `game_core` focused test ran 1 test and failed:
  - `0 passed; 1 failed; 434 filtered out`
  - failure: `InvalidStaticData("invalid battlefield instance: spawn wave references missing route")`
- `game_server -- --list` failed to compile:
  - missing `mobility_kind` and `target_traits` in `AbnormalityMetadata` test fixture;
  - missing `unit_deploy_costs` in two `LiveBattleDeploymentDto` test fixtures.
