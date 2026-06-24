# Validation/Test Harness Refactor Audit

## Scope

Canonical refactor 기준은 `docs/refactor_preparation_plan.md`다.

읽은 범위:

- Validation code: `src/game/battle/validation/mod.rs`, `src/game/battle/validation/types.rs`, `src/game/battle/validation/validator.rs`, `src/game/battle/validation/attacks.rs`, `src/game/battle/validation/state.rs`, `src/game/battle/validation/buffs.rs`, `src/game/battle/validation/parent.rs`, `src/game/battle/validation/auto_attack.rs`, `src/game/battle/validation/spawns.rs`.
- Live/static data tests: `tests/ron_loading.rs`, `tests/live_skill_catalog_audit.rs`, `tests/common/mod.rs`.
- Skill behavior harness: `tests/skill_test_suite.rs`, `tests/skill_test/common/*`, `tests/skill_refactor_validation.rs`, `tests/live_item_skill_activation.rs`.
- World/server flow tests: `src/game/world/tests/mod.rs`, `src/game/world/tests/combat.rs`, `src/game/world/tests/equipment.rs`, `src/game/world/admin/tests.rs`, `../game_server/src/game/player_game_actor/messages.rs`, `../game_server/src/game/player_game_actor/state.rs`, `../game_server/src/game/player_game_actor/handlers.rs`.

## Current Structure

- `TimelineValidator` is a modular battle event-log validator with configurable checks (`src/game/battle/validation/types.rs:77`, `src/game/battle/validation/validator.rs:354`).
- Default timeline validation enables battle start/end, contiguous seq, non-decreasing time, attack kind, parent seq, outcome parent, autocast pairs, spawn stats, HP delta, dead unit action guards, position checks, movement path consistency, and auto attack min interval (`src/game/battle/validation/types.rs:96`).
- Buff validation can use injected buff data or live default buff data (`src/game/battle/validation/validator.rs:343`, `:350`).
- Live RON integration tests load game data through `tests/common/mod.rs:350` and assert schema/content/preview contracts in `tests/ron_loading.rs`.
- World tests have their own live RON loader in `src/game/world/tests/mod.rs:260`.
- Skill tests load live data, materialize board scenarios, run `BattleCore`, and export debug event logs (`tests/skill_test/common/run.rs:188`, `:214`).
- Server tests pin battle transport order and message shape (`../game_server/src/game/player_game_actor/handlers.rs:736`, `:879`; `../game_server/src/game/player_game_actor/messages.rs:522`).

## Source Of Truth

- Runtime gameplay behavior should be fixed by user-visible battle events, Unity-facing DTOs, live RON loading, and real world flow tests, not by internal helper shapes.
- `Timeline` is still the serialized/client-facing event-log name but is a battle event log, not a precomputed replay replacement (`src/game/battle/timeline.rs:121`).
- `docs/refactor_preparation_plan.md` says tests should pin user-visible behavior, Unity-facing DTOs, data validation, live RON loading, and actual gameplay flow.

## Refactor Candidates

### 1. `TimelineValidator` is not integrated into most gameplay tests

Evidence:

- `TimelineValidator` is exported from `src/game/battle/validation/mod.rs:18`.
- Search showed `TimelineValidator::...` usages concentrated in `src/game/battle/validation/validator.rs` unit tests.
- Skill test harness runs battles in `tests/skill_test/common/run.rs:188` but does not validate the produced timeline before returning `ScenarioRunResult`.
- World combat tests inspect specific `BehaviorResult::BattleAdvanced`/`battle_update` fields but do not run the full validator on emitted timelines.

Why it matters:

- The validator encodes many runtime/event-log invariants, but those invariants are not automatically applied to the high-value live skill, world flow, and server-facing tests.
- This means a regression can pass behavior-specific assertions while breaking another Unity-facing event invariant.

Candidate:

- Add a common helper that validates every completed `BattleResult.timeline` produced by skill and world test harnesses with `TimelineValidator::with_live_buff_data(TimelineValidatorConfig::default())`.
- For tests that intentionally build partial timelines, use named configs such as `partial_timeline_config()` instead of ad-hoc local structs.
- Record any validator exception with a reason and removal condition.

### 2. `TimelineValidatorConfig` suggests optional checks, but some checks are unconditional

Evidence:

- `validate_stateful_unit_invariants()` accepts `_config` but does not use it (`src/game/battle/validation/state.rs:14`).
- `validate_skill_cast_interrupt_relations()` is always called in `TimelineValidator::validate()` (`src/game/battle/validation/validator.rs:426`) even when `require_autocast_pairs` is false.
- `validate_attacks()` and `validate_buffs()` are always called (`src/game/battle/validation/validator.rs:435`, `:436`), while only some lower-level checks are config-gated.

Why it matters:

- The config is a second source of truth for which validator policies are active, but it does not fully describe actual behavior.
- This is fine if some invariants are mandatory, but then the config should not imply they can be disabled.

Candidate:

- Split config into `mandatory` validation and explicitly optional validation, or add config fields for every non-mandatory module.
- Rename local test configs to make partial-validator intent visible.

### 3. Timeline version validator still accepts legacy versions `6` and `7`

Evidence:

- Current `TIMELINE_VERSION` is `26` (`src/game/battle/timeline.rs:39`).
- Validator accepts `TIMELINE_VERSION`, `6`, or `7` (`src/game/battle/validation/validator.rs:362`).

Why it matters:

- This is an explicit compatibility path for very old serialized event logs.
- The current refactor policy says not to keep compatibility layers by default.

Candidate:

- Remove old version acceptance from normal validation, or move it behind an archive/import validator with documented removal conditions.

Policy:

- If archived logs or Unity tooling still need versions `6`/`7`, removal is a data/tooling policy decision. 사용자와 정책 논의 필요.

### 4. Live RON loader is duplicated across server and test harnesses

Evidence:

- Integration tests use `tests/common/mod.rs:350`.
- World tests have `live_game_data_from_ron()` in `src/game/world/tests/mod.rs:260`.
- Server has its own loader already documented in `data_ron_validation_refactor.md`.

Why it matters:

- Different test suites can accidentally load slightly different live data sets or inject starter skill fragments differently.
- This repeats the component 14 source-of-truth finding from the test-harness side.

Candidate:

- Provide one official core live RON loader and have server, integration tests, world tests, and skill tests call it.
- Keep tiny synthetic `create_test_game_data()` builders for narrow unit tests, but make live data tests use the same loader as runtime startup.

### 5. Live content roster tests encode policy directly in Rust lists

Evidence:

- `live_abnormality_ron_only_contains_catalog_roster()` hard-codes the full abnormality roster (`tests/ron_loading.rs:23`).
- `live_skill_catalog_matches_current_delivery_audit()` hard-codes spatial and instant skill sets (`tests/live_skill_catalog_audit.rs:13`).
- `live_skill_catalog_is_fully_covered_by_behavior_test_manifest()` hard-codes skill coverage sets (`tests/live_skill_catalog_audit.rs:85`).

Why it matters:

- These tests are useful as content gates, but Rust source becomes the catalog policy manifest.
- Adding or removing live content requires code edits even if the data change is intentionally authored.

Candidate:

- Keep the tests, but move expected live roster/coverage manifests into versioned data or a clearly named audit manifest file.
- The test should compare live RON against that manifest and require intentional manifest updates.

Policy:

- Live content roster additions/removals and whether the manifest should live in Rust or data are content policy choices. 사용자와 정책 논의 필요.

### 6. Debug event log exports are written during tests

Evidence:

- `tests/common::write_debug_event_log_export()` writes under `debug_event_log_exports` (`tests/common/mod.rs:39`).
- Skill test harness calls it unconditionally after every abnormality scenario run (`tests/skill_test/common/run.rs:214`).
- Core command unit tests also contain a local writer and call it in at least one test (`src/game/battle/core/commands.rs:2254`, `:2574`).
- World test helpers include another writer (`src/game/world/tests/mod.rs:863`).

Why it matters:

- Tests can create or modify files in the workspace, making the worktree dirty and making CI/local runs less hermetic.
- Debug exports are useful, but they should be opt-in or written to a temp/target path.

Candidate:

- Gate exports behind an env var such as `EXPORT_DEBUG_EVENT_LOGS=1`.
- Write to `target/debug_event_log_exports` or temp dirs by default, not the crate root.
- Consolidate duplicate writer helpers.

### 7. `GameDataBuilder::live_defaults()` leaks into tests as a quasi-runtime loader

Evidence:

- Tests use `GameDataBuilder::live_defaults()` in `tests/skill_refactor_validation.rs:206`, `:216`, `:2268`, `tests/live_item_skill_activation.rs:187`, and several core unit tests.
- Component 14 found `live_defaults()` currently loads only live buffs while most databases remain test-supplied/empty.

Why it matters:

- The name suggests a live data baseline, but the behavior is partial.
- Tests can accidentally assume they are using live-like data when only one subsystem is live.

Candidate:

- Rename to `with_live_buff_defaults()` or split into explicit test builders.
- Use the official live RON loader for integration-like tests.

### 8. Contract tests exist, but coverage is split across crates and layers

Evidence:

- Core world combat tests pin battle setup/update DTO details (`src/game/world/tests/combat.rs:781`, `:2220`).
- Server tests pin top-level message ordering and final combat-result snapshot (`../game_server/src/game/player_game_actor/handlers.rs:736`, `:879`).
- Message serialization tests pin `battle_update`, `battle_setup_snapshot`, and `battle_resync` envelope shapes (`../game_server/src/game/player_game_actor/messages.rs:522`).

Why it matters:

- This is good coverage, but the source of truth is scattered. A future change may update core tests but miss server envelope tests, or vice versa.

Candidate:

- Create a small "Unity contract suite" test grouping that runs:
  - core live battle setup/update DTO shape,
  - server top-level message shape,
  - command-result ordering,
  - final `combat_result` snapshot,
  - representative `selected_event` variants.
- Keep existing tests, but make the contract suite easy to run as a focused command.

## Existing Simplification Opportunities

- Instead of writing more per-scenario assertions, reuse `TimelineValidator` as a shared invariant pass for every finished timeline.
- Instead of duplicating live RON loading helpers, make test harnesses call the same official loader proposed in `data_ron_validation_refactor.md`.
- Instead of hard-coding live roster sets in multiple tests, make one manifest the source and use tests as consumers.

## Legacy / Compatibility Candidates

- Timeline version `6`/`7` acceptance in normal validator path.
- Debug event export side effects in regular tests.
- `live_defaults()` name/behavior mismatch as a legacy convenience builder.

## Do Not Change Yet

- Do not delete content roster tests; they catch important unintended live data drift.
- Do not replace all focused assertions with generic timeline validation; both are needed.
- Do not force synthetic unit tests to load all live RON. Tiny deterministic test data is still valuable for local behavior tests.
- Do not make validator config more abstract until current mandatory versus optional checks are classified.

## Validation Plan

Focused commands for implementation work:

- `cargo test -p game_core battle::validation`
- `cargo test -p game_core --test ron_loading`
- `cargo test -p game_core --test live_skill_catalog_audit`
- `cargo test -p game_core --test skill_test_suite`
- `cargo test -p game_core world::tests::combat`
- `cargo test -p game_server game::player_game_actor`

When integrating `TimelineValidator` into higher-level tests, start with a small subset of skill scenarios and one world combat flow, then broaden.

No Rust tests were run for this audit-only document.
