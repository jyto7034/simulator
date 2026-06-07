# Post Policy Technical Debt Review Experiments

## 2026-06-07 - Goal Start

Actions:

- Created post-policy technical debt review work memory directory.
- Created initial `PLAN.md`, `EXPERIMENTS.md`, `EXPERIMENT_NOTES.md`, and `REPORT.md`.

Result:

- Audit goal started.
- No runtime/core/server code changes.

## 2026-06-07 - Source Reading And Evidence Collection

Actions:

- Read `docs/post_policy_technical_debt_review_goal.md`.
- Read master/policy notes:
  - `docs/core_policy_decisions_2026_06.md`
  - `docs/goals/core_policy_implementation_master/EXPERIMENTS.md`
  - `docs/goals/core_policy_implementation_master/EXPERIMENT_NOTES.md`
- Inspected current worktree shape with `git status --short` and code/data search.
- Read runtime/data/snapshot/server surfaces:
  - `src/game/data/mod.rs`
  - `src/game/combat_preview.rs`
  - `src/game/world/snapshot.rs`
  - `src/game/world/helpers.rs`
  - `src/game/combat_player_spawns.rs`
  - `src/game/battle/core/movement/engine.rs`
  - `src/game/battle/core/movement/rapier_backend.rs`
  - `src/game/battle/core/movement/steering.rs`
  - `../game_server/src/game/player_game_actor/state.rs`
  - `../game_resources/data/equipments.ron`
  - `../game_resources/data/equipments/base.ron`
- Read canonical external Unity docs:
  - `/mnt/f/unity projects/ark/docs/unity_core_contract.md`
  - `/mnt/f/unity projects/ark/docs/unity_client_implementation_goal.md`
- Searched for ignored tests and recorded prior goal evidence about zero-test filters.

Result:

- Confirmed no `#[ignore]` tests in `src`, `tests`, or `../game_server/src`.
- Confirmed several historical goal commands used broad filters that either ran 0 tests or matched only 1-2 tests; this is test-verification debt rather than a runtime bug.
- Confirmed `GameDataBuilder::empty()` is now truly empty, while `GameDataBuilder::live_defaults()` explicitly opts into `BuffDatabase::live_default()`.
- Confirmed movement/Rapier backend is mostly aligned with the new overlap/airborne policy: unit colliders are ignored for correction, airborne skips static obstacle correction, and board bounds use core clamp.
- Confirmed report findings in `REPORT.md`.
- No runtime/core/server code changes.

## 2026-06-07 - Documentation Verification

Actions:

- Ran `git diff --check -- docs/goals/post_policy_technical_debt_review/PLAN.md docs/goals/post_policy_technical_debt_review/EXPERIMENTS.md docs/goals/post_policy_technical_debt_review/EXPERIMENT_NOTES.md docs/goals/post_policy_technical_debt_review/REPORT.md`.
- Ran `rg -n "[[:blank:]]$"` over the four work-memory files.
- Ran a conflict-marker search over the four work-memory files.

Result:

- Passed. No whitespace/conflict-marker issues were reported.
