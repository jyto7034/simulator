# Post Policy Test Verification Report

## Executive Summary

The post-policy test suite is meaningfully improved but not clean.

Strong areas: damage feedback tags, consumable validation/modifiers, movement/Rapier overlap policy, airborne mobility, weapon targeting, skill fragment compatibility, AD/AP damage/resistance, and live RON loading all have real current tests.

Blocking issues:

- `cargo test -p game_core` currently fails: 434 passed, 1 failed.
- `cargo test -p game_server -- --list` currently fails to compile server tests because test fixtures have not been updated for current Unity-facing/core DTO fields.

Audit conclusion:

- Most new core policy areas have focused test coverage.
- Several older goal verification commands were false positives because they ran 0 tests.
- Unity-facing server test coverage is currently broken under `cfg(test)`, despite `cargo check -p game_server` passing.
- No production code, test code, server code, or live RON was changed in this audit.

## Commands Run

- `cargo check -p game_core`: passed.
- `cargo check -p game_server`: passed.
- `cargo test -p game_core -- --list`: listed current game_core test inventory.
- `cargo test -p game_core feedback_tags -- --list`: 7 tests.
- `cargo test -p game_core combat_preview -- --list`: 24 lib tests plus 1 `ron_loading` integration test.
- `cargo test -p game_core retreat -- --list`: 2 tests.
- `cargo test -p game_core consumable -- --list`: 16 lib tests plus 1 `ron_loading` integration test.
- `cargo test -p game_core movement -- --list`: 61 tests.
- `cargo test -p game_core airborne -- --list`: 17 lib tests plus 1 `ron_loading` integration test.
- `cargo test -p game_core targeting -- --list`: 8 lib tests plus 1 `skill_refactor_validation` integration test.
- `cargo test -p game_core skill_fragment -- --list`: 29 lib tests plus 2 `ron_loading` integration tests.
- `cargo test -p game_core fixed_defense -- --list`: 17 tests.
- `cargo test -p game_core damage_feedback -- --list`: 0 tests.
- `cargo test -p game_core ron_loading -- --list`: 0 tests.
- `cargo test -p game_core skill_refactor_validation -- --list`: 0 tests.
- `cargo test -p game_core skill_test_suite -- --list`: 0 tests.
- `cargo test -p game_core --test ron_loading`: passed, 16 tests.
- `cargo test -p game_core --test skill_refactor_validation`: passed, 10 tests.
- `cargo test -p game_core --test skill_test_suite`: passed, 15 tests.
- `cargo test -p game_core --test live_item_skill_activation`: passed, 3 tests.
- `cargo test -p game_core --test live_skill_catalog_audit`: passed, 3 tests.
- `cargo test -p game_core --test unit_test`: succeeded but ran 0 tests.
- `cargo test -p game_core`: failed, 434 passed and 1 failed.
- `cargo test -p game_server -- --list`: failed to compile server test target.
- `rg -n "#\\[ignore\\]|ignore\\]" src tests ../game_server/src`: no ignored tests found.
- Unity canonical docs were scanned for `feedback_tags`, `threat_warnings`, `unit_deploy_costs`, `effective_deploy_cost`, `mobility_kind`, `target_traits`, skill fragment compatibility, and damage type contract terms.

## Passing Verification

- Compile baseline:
  - `game_core` compiles.
  - `game_server` compiles in normal check mode.
- Live RON/data validation:
  - `ron_loading` passes 16 tests.
  - Coverage includes live airborne mobility kind, AD/AP threat warnings, consumable schema, skill fragment schema, buff schema, PVE references, live map content pools, reward contracts, and skill references.
- Skill runtime/catalog:
  - `skill_refactor_validation` passes 10 tests.
  - `skill_test_suite` passes 15 tests.
  - `live_item_skill_activation` passes 3 tests.
  - `live_skill_catalog_audit` passes 3 tests.
- Damage feedback DTO:
  - `feedback_tags` lists 7 tests covering plain damage, critical, mitigated, fixed damage, immune, command timeline emission, and snake_case serialization.
- Consumable modifier and validation:
  - `consumable` lists 17 total matching tests across lib and `ron_loading`.
  - Coverage includes percent validation, zero duration rejection, deploy cost reduction, initial skill charge cap, defense mitigation, trauma mitigation, battle HP setup, live deployment cost, dead target rejection, alive non-dead target allowance, active modifier snapshot, replacement without refund, and RON loading.
- Movement/Rapier/unit overlap:
  - `movement` lists 61 tests.
  - Coverage includes direct and Rapier movement, moving-unit overlap allowance, locked-body overlap allowance, board clamp, static obstacle stop/correction, backend parity, unit ordering, and Rapier not depenetrating unit colliders.
- Airborne:
  - `airborne` lists 18 total matching tests across lib and `ron_loading`.
  - Coverage includes terrain immunity, board clamp, blocking immunity, protected target priority, nearest target fallback, route attack/continue behavior, single-target air-capable gating, area hit inclusion, preview warning, target trait source-of-truth rejection, and live RON mobility loading.
- Weapon targeting:
  - `targeting` lists 9 total matching tests across lib and `skill_refactor_validation`.
  - Coverage includes forward, air-first, low-defense, low-magic-resist, splash-cluster profiles, explicit cast/step targeting, and invalid permanent targeting rejection.
- Skill fragment compatibility:
  - `skill_fragment` lists 31 total matching tests across lib and `ron_loading`.
  - Coverage includes compatibility requirements, stable failure codes, static validation, equip accept/reject, active slot rules, upgrade/awakening/research flow, snapshot-facing equipment actions, and RON loading.
- Fixed defense/blocking:
  - `fixed_defense` lists 17 tests.
  - Coverage includes route endpoint targeting, fixed player facing/range, block capacity priority by route progress, tie-breaks, release behavior, and fixed defense objective construction.

## Failed Verification

### Full game_core suite

- Command: `cargo test -p game_core`
- Result: failed.
- Tests Executed: 435 lib tests before integration targets; 434 passed, 1 failed.
- Area: CombatPreview/PVE tactical plan fixture.
- Evidence:
  - Failing test: `game::events::combat::tests::authored_protect_unit_tactical_plan_overrides_default_defense_contract`.
  - Panic: `core_enhanced/src/game/combat_preview.rs:737`.
  - Error: `InvalidStaticData("invalid battlefield instance: spawn wave references missing route")`.
- Likely Cause:
  - The fixture in `src/game/events/combat.rs` hardcodes `route_id: Some("black_box_breach_main")`.
  - `spawn_waves_for` preserves authored route ids.
  - `validate_instance` now rejects spawn wave route ids that do not exist in the generated battlefield.
  - The test likely predates stricter battlefield route validation.
- Policy Conflict:
  - No user policy decision appears required. The stricter validation matches the current direction; the test fixture is stale/brittle.
- Recommended Follow-up:
  - Create a focused test cleanup goal that makes this fixture route-aware or gives the authored encounter an explicit battlefield/template containing the route it references.
- Needs User Policy Decision:
  - No.

### game_server test target

- Command: `cargo test -p game_server -- --list`
- Result: failed to compile.
- Tests Executed: 0, compile failed before listing.
- Area: Unity-facing DTO/server command result tests.
- Evidence:
  - `game_server/src/game/player_game_actor/handlers.rs:350`: missing `mobility_kind` and `target_traits` in `AbnormalityMetadata` fixture.
  - `game_server/src/game/player_game_actor/state.rs:750`: missing `unit_deploy_costs` in `LiveBattleDeploymentDto` fixture.
  - `game_server/src/game/player_game_actor/state.rs:789`: missing `unit_deploy_costs` in `LiveBattleDeploymentDto` fixture.
  - Core source confirms these are current fields:
    - `src/game/data/abnormality_data.rs`: `target_traits`, `mobility_kind`.
    - `src/game/behavior.rs`: `LiveBattleDeploymentDto.unit_deploy_costs`.
- Likely Cause:
  - Server test fixtures were not updated after airborne metadata and effective deploy cost DTO work.
- Policy Conflict:
  - No policy decision appears required. The missing fields are current contracts.
- Recommended Follow-up:
  - Create a server DTO test refresh goal. Update stale server fixtures and add assertions for `unit_deploy_costs[*].effective_deploy_cost`, `mobility_kind`, and target traits where serialized.
- Needs User Policy Decision:
  - No.

## Zero-Test Or Suspicious Commands

- `cargo test -p game_core damage_feedback -- --list`
  - Result: 0 tests.
  - Historical context: the damage feedback goal recorded this command as an initial false-positive filter before switching to `feedback_tags`.
  - Use `feedback_tags` instead.
- `cargo test -p game_core ron_loading -- --list`
  - Result: 0 tests.
  - Use `cargo test -p game_core --test ron_loading`.
- `cargo test -p game_core skill_refactor_validation -- --list`
  - Result: 0 tests.
  - Use `cargo test -p game_core --test skill_refactor_validation`.
- `cargo test -p game_core skill_test_suite -- --list`
  - Result: 0 tests.
  - Use `cargo test -p game_core --test skill_test_suite`.
- `cargo test -p game_core --test unit_test`
  - Result: command succeeds but runs 0 tests.
  - This target should be removed or populated.
- Older notes also cite `cargo test -p game_core blocking` as a narrow filter. Current policy evidence should use `movement`, `fixed_defense`, `airborne`, and explicit Rapier/backend tests instead of `blocking` alone.

## Coverage Gaps By Policy

### Damage Feedback DTO

- Current coverage: strong.
- Evidence: 7 `feedback_tags` tests cover calculation tags and timeline serialization.
- Gap: Server-side Unity command/result test coverage cannot currently be trusted because `game_server` test target does not compile.
- Follow-up: include `feedback_tags` in refreshed server DTO serialization tests after game_server test cfg is fixed.

### CombatPreview Threat Warnings

- Current coverage: medium-strong for generation and serialization.
- Evidence: `combat_preview` lists warning tests, rumor warning tests, typed threat warning serialization, airborne warning, and live AD/AP warning RON test.
- Gap: The hardcoded false-rumor disproof lifecycle is not clearly covered. Current tests mainly prove preview generation and live warning presence.
- Follow-up: add tests for 20% false rumor selection, candidate exclusion from actual resolved spawn wave warnings, and retreat/re-entry `Disproved`.

### Attempt/Retreat/Re-entry

- Current coverage: medium.
- Evidence: 2 `retreat` tests cover re-entry attempts and consumable modifier survival/expiry.
- Gap: Edge cases are thin:
  - commands after `BattleEnd`,
  - all units dead but `BattleEnd` not yet emitted,
  - exact third-attempt exhaustion semantics,
  - UI-facing command availability while re-entering.
- Follow-up: add focused gameplay-flow tests for the above states.

### Consumable Modifier Duration

- Current coverage: strong.
- Evidence: 17 matching consumable tests across lib and RON loading.
- Gap: Server command/result DTO coverage is indirectly blocked by `game_server` test compile failure.
- Follow-up: include consumable active modifier snapshot/command result assertions in the server DTO refresh goal.

### Unit Overlap/Blocking

- Current coverage: strong if interpreted through the right filters.
- Evidence: `movement` and `fixed_defense` together cover moving overlap allowance, fixed defense block capacity, route progress priority, tie-breaks, release behavior, and Rapier not depenetrating unit colliders.
- Gap: Historical `blocking` filter is misleadingly narrow. It should not be used as sole evidence.
- Follow-up: add a named aggregate test or documented verification command for blocking policy to avoid future 1-2 test false confidence.

### Movement/Rapier

- Current coverage: strong.
- Evidence: 61 `movement` tests include direct/Rapier parity, static obstacle policy, board clamp, airborne terrain immunity, and unit collider exclusion.
- Gap: Prior Rapier goal found exact float equality brittleness. Current tests appear to use tolerance in the relevant backend parity path, but future movement tests should continue avoiding exact float assertions.
- Follow-up: keep Rapier as correction/backend helper only; do not let it become source of truth for blocking, targetability, route progress, deploy occupancy, or airborne terrain immunity.

### Airborne

- Current coverage: strong.
- Evidence: 18 matching tests cover movement, targeting, block immunity, area hit inclusion, preview warning, target trait source-of-truth rejection, and live RON mobility loading.
- Gap: Unity/server serialized `mobility_kind` contract is not covered while `game_server` tests fail to compile.
- Follow-up: add server DTO assertions for `mobility_kind` once stale fixtures are updated.

### Weapon Targeting

- Current coverage: medium-strong.
- Evidence: 9 matching targeting tests cover profile selection and explicit skill targeting.
- Gap: Server/Unity snapshot coverage for effective weapon profile cannot be fully assessed through `game_server` tests because they do not compile.
- Follow-up: include weapon profile/effective targeting snapshot assertions in DTO refresh or a dedicated Unity contract test goal.

### Skill Fragment Compatibility

- Current coverage: strong in core.
- Evidence: 31 matching tests cover compatibility requirements, equip rejection/acceptance, active slot policy, progression, and RON loading.
- Gap: As above, server-side serialized command/snapshot coverage is blocked by game_server test cfg failure.
- Follow-up: add Unity-facing snapshot assertions for `roster.employees[*].skill_fragments.compatibility[*]` after server test build is repaired.

### AD/AP Validation

- Current coverage: medium-strong.
- Evidence: damage tests cover physical/magic/true mitigation; `combat_preview` and `ron_loading` cover typed threat warnings; external Unity docs specify damage color/feedback contract.
- Gap: False-rumor `Disproved` lifecycle is not clearly tested, and server DTO compile failure blocks end-to-end Unity contract confidence.
- Follow-up: add lifecycle tests for rumor `Disproved` and refreshed server DTO tests for `damage_type`/`feedback_tags`.

## Brittle Or Stale Tests

- `game::events::combat::tests::authored_protect_unit_tactical_plan_overrides_default_defense_contract`
  - Stale/brittle because it hardcodes an authored route id while relying on generated battlefield selection.
  - Current validator correctly rejects missing route references.
- `../game_server/src/game/player_game_actor` test fixtures
  - Stale because they do not include current DTO/schema fields.
  - This blocks Unity-facing server test coverage.
- `tests/unit_test.rs`
  - Empty target, currently runs 0 tests.
- Historical integration target filters
  - `ron_loading`, `skill_refactor_validation`, and `skill_test_suite` used without `--test` run 0 tests.
- Historical `blocking` verification
  - Too narrow by itself. Use policy-area filters that actually match movement and fixed defense tests.

## Suggested Test Cleanup Goals

1. `post_policy_test_suite_green_goal`
   - Fix or replace the stale route fixture in `authored_protect_unit_tactical_plan_overrides_default_defense_contract`.
   - Preserve strict battlefield route validation.
   - Run `cargo test -p game_core`.

2. `game_server_unity_dto_test_refresh_goal`
   - Update stale `game_server` test fixtures for `AbnormalityMetadata` and `LiveBattleDeploymentDto`.
   - Add assertions for `unit_deploy_costs[*].effective_deploy_cost`, `mobility_kind`, target traits, `feedback_tags`, and relevant snapshot/command payloads.
   - Run `cargo test -p game_server -- --list` and then focused server tests.

3. `cargo_test_filter_hygiene_goal`
   - Replace goal/documented verification commands that use integration target names as filters.
   - Add a lightweight script or checklist that fails verification notes when a command runs 0 tests unexpectedly.

4. `attempt_retreat_edge_case_tests_goal`
   - Add focused tests for third-attempt exhaustion, command rejection after `BattleEnd`, all-units-dead-before-`BattleEnd`, and re-entry command availability.

5. `threat_warning_false_rumor_lifecycle_goal`
   - Implement 20% false rumor selection.
   - Exclude warning tags already present in actual resolved spawn waves.
   - Mark false rumor warnings as `Disproved` across retreat/re-entry.

6. `empty_unit_test_target_cleanup_goal`
   - Remove or populate `tests/unit_test.rs`.

## Policy Questions For User

None at this audit stage.

The failures found look like stale fixtures and missing test coverage, not unresolved game policy decisions.
