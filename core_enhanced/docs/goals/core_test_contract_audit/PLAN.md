# Core Test Contract Audit Plan

## Objective

Review whether the current `core_enhanced` and related server tests meaningfully lock user-visible gameplay behavior, Unity-facing DTO/transport contracts, live RON/data validation, and runtime timing/order guarantees.

This goal is an audit-only goal. Do not change production code, test code, or live data as part of this goal unless the user explicitly starts a follow-up implementation goal.

## Meaningful Test Standard

A meaningful core test locks a contract that another system, player flow, live data file, or future refactor depends on.

Meaningful tests should primarily verify:

1. User-visible gameplay behavior.
   - deployment cost and state changes
   - withdraw/defeat redeploy cooldowns
   - battle success/failure/result transitions
   - retreat/re-entry policy
   - route movement, blocking, targeting, damage, skill, projectile, and timeline outcomes

2. Unity-facing DTO and transport contracts.
   - top-level server message type
   - DTO shape that Unity consumes
   - message order
   - absence of stale trailing state snapshots
   - `battle_setup_snapshot`, `battle_update`, `battle_resync`, `command_result`, `state_snapshot` behavior
   - checkpoint fields such as `hud`, `unit_source`, `range_previews`, `position`, and `world_position`

3. Live RON/data validation.
   - live data loads
   - ids resolve
   - forbidden legacy content stays out
   - schema-level policy violations fail loudly
   - generated previews or encounters are based on live data, not test-only assumptions

4. Core gameplay rules.
   - tile-based basic attack eligibility
   - `range_units` not being official basic attack eligibility
   - target usefulness with damage/effects
   - projectile delivery policy
   - skill cast interrupt/epoch behavior
   - blocking and movement policy
   - battle record persistence

5. Time, order, and replay contracts.
   - monotonic timeline `seq`
   - `after_seq`/`to_seq` ranges
   - invalid/future resync cursor rejection
   - attack resolve and movement sampling using the same battle-time basis
   - final `battle_update` before `combat_result` snapshot

Weak tests include:

- tests that only pin private helper structure
- exact full JSON snapshots with unrelated fields
- debug log wording checks
- enum order checks with no external contract meaning
- tests that keep legacy behavior alive through ignored or compatibility-only expectations
- tests that pass by checking only that "something exists" without proving a contract

## Source Of Truth Order

Do not judge tests by names alone. Check facts in this order:

1. Actual runtime code and test assertions.
2. Live RON/data loaded by tests.
3. Unity-facing snapshot/command/WebSocket contract tests.
4. Current policy documents:
   - `docs/game_rulebook.md`
   - `docs/skill_target_contract.md`
   - `docs/code_documentation_sync_guidelines.md`
   - external `/mnt/f/unity projects/ark/docs/core_unity_battle_transport_contract.md`
   - external `/mnt/f/unity projects/ark/docs/unity_core_contract.md`

## Primary Test Areas To Audit

Core tests:

- `src/game/world/tests/`
- `src/game/battle/**` inline tests
- `src/game/combat_preview/**` inline tests
- `src/game/data/**` inline tests
- `tests/ron_loading.rs`
- `tests/live_skill_catalog_audit.rs`
- `tests/live_item_skill_activation.rs`
- `tests/skill_refactor_validation.rs`
- `tests/skill_test_suite.rs`
- `tests/skill_test/**`

Server/Unity-facing tests:

- `/mnt/f/work/simulator/game_server/src/game/player_game_actor/messages.rs`
- `/mnt/f/work/simulator/game_server/src/game/player_game_actor/state.rs`
- `/mnt/f/work/simulator/game_server/src/game/player_game_actor/handlers.rs`
- `/mnt/f/work/simulator/game_server/src/game/player_game_actor/session.rs`

Historical or deleted test areas may be mentioned only as context. Do not restore them during this audit.

## Audit Passes

1. Inventory pass.
   - List test files and group them by domain.
   - Identify which tests are unit, integration, live RON, DTO/transport, or smoke tests.

2. Contract-value pass.
   - Classify representative tests as:
     - strong contract test
     - useful but narrow test
     - implementation-shape test
     - stale/legacy test
     - smoke-only test
     - missing contract coverage

3. Gameplay coverage pass.
   - Check whether deployment, redeployment, retreat, battle result, battle records, DefenseRoute movement, blocking, targeting, damage, skills, projectile delivery, and skill interrupt policies have meaningful test coverage.

4. Unity-facing coverage pass.
   - Check whether DTO shape, transport order, command result semantics, checkpoint shape, range preview, resync, setup-loss recovery, and final combat result snapshot are covered by meaningful tests.

5. Live data coverage pass.
   - Check whether live RON loading, abnormality roster policy, enemy profiles, skill fragments, equipment, encounters, wave presets, range presets, and schema policy violations are covered.

6. Weakness/gap pass.
   - Identify tests that are too coupled to implementation details.
   - Identify contracts that are documented but not meaningfully tested.
   - Identify tests that may pass while the player/Unity-visible contract is broken.

7. Report pass.
   - Do not fix tests in this goal.
   - Produce an audit summary with prioritized recommendations for follow-up goals.

## Completion Conditions

- Major test files are inventoried and classified.
- The audit explains which tests are meaningful by the standard above.
- The audit identifies high-value existing tests that should be preserved during refactors.
- The audit identifies weak tests or stale tests, if any.
- The audit identifies missing contract coverage, if any.
- `EXPERIMENTS.md` records commands, files inspected, classification results, and any failed assumptions.
- `EXPERIMENT_NOTES.md` records judgment calls, policy questions, and follow-up implementation candidates.
- No production code, tests, or live RON/data are changed unless the user explicitly authorizes a follow-up implementation goal.

## Stop Conditions

Stop and report instead of guessing if the audit finds:

- a test and current policy disagree, but it is unclear whether code or policy should change
- a DTO shape appears under-tested and fixing it would require schema changes
- live RON data violates a policy in a way that requires content deletion/replacement
- a gameplay rule is missing tests because the rule itself is not fully decided
- a test appears to preserve legacy behavior that may still be intentionally supported

## Suggested Commands

```text
find src tests -path '*/target' -prune -o -type f \( -name '*.rs' \) -print
rg -n "#\[test\]|#\[tokio::test\]|#\[actix_rt::test\]|#\[should_panic\]|ignore" src tests /mnt/f/work/simulator/game_server/src/game/player_game_actor
rg -n "battle_setup_snapshot|battle_update|battle_resync|command_result|state_snapshot|range_previews|unit_source|hud|known_seq|combat_result" src tests /mnt/f/work/simulator/game_server/src/game/player_game_actor
rg -n "live_defense|DefenseRoute|redeploy|withdraw|retreat|battle_records|blocking|MovementSegmentStarted|BasicAttackProjectile|TileArea|WholeFieldValidTiles|range_units" src tests
cargo test -p game_core -- --list
cargo test -p game_server -- --list
```

Run full test suites only if needed to understand coverage or reproduce failures. This goal can be completed by reading and classifying tests without executing every test.

## Completion Report Requirements

Report:

- audited test areas
- strongest existing contract tests
- weak/stale/implementation-shaped tests
- missing contract coverage
- recommended follow-up goals in priority order
- commands run and whether they changed anything
