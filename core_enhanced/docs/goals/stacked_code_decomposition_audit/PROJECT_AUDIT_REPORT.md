# Project-Wide Stacked Code Audit

## Summary

- Verdict: project-wide second pass complete on the current worktree. The previous highest-risk shop DTO and boss-omen findings are now resolved in code; remaining stacked-code risk is concentrated in debug/export ownership, broad orchestration surfaces, and historical documentation volume.
- Main source-of-truth owners:
  - Runtime flow: `src/game/world.rs`, `src/game/world/*`.
  - Battle runtime: `src/game/battle/core/*`.
  - Live data: `src/game/data/mod.rs`, `src/game/data/validation.rs`, `game_resources/data/**`.
  - Unity-facing/core DTO surface: `src/game/behavior.rs`, `src/game/world/snapshot.rs`, `src/game/world/state.rs`.
- Main hidden context:
  - The codebase has many completed goal documents and historical notes; some are now stale because later cleanup goals have already changed runtime behavior.
  - Test/debug artifacts are generated under repository-root directories, so readers must know which files are gameplay source, debug output, or tracked noise.
  - Several broad helpers mix runtime mutation, DTO projection, persistence, and validation in one flow.
- Highest-risk current finding: `RunState::record_battle(...)` now documents codex-style records, but the runtime still mixes state mutation with repo-root JSON export and writes `BattleRecordExport.battle_uuid` from `abnormality_uuid`.
- Recommended next goal: `battle_record_debug_export_boundary`.

## Audited Surface

This pass covered the current project at a project-wide triage level, not a line-by-line proof of every function. The report was re-run after follow-up cleanup work changed several areas from "immediate correction" to "resolved in current worktree."

Inspected:

- `docs/goals/stacked_code_decomposition_audit/PLAN.md`
- canonical refactor/test/debug-output policy in `docs/refactor_preparation_plan.md`
- current runtime module structure under `src/game`
- major runtime flow owners in `src/game/world*`
- battle runtime entrypoints and movement backend structure under `src/game/battle/core`
- data loading and validation entrypoints under `src/game/data`
- Unity-facing DTO/snapshot surfaces in `src/game/behavior.rs`, `src/game/world/snapshot.rs`, and `src/game/world/state.rs`
- current tests under `tests/` and `src/game/world/tests`
- tracked/generated output directories `battle_records/`, `debug_event_log_exports/`, `logs/`
- existing completed goal documents relevant to discovered runtime contradictions

## Project Coverage Matrix

| Area | Current role | Audit result |
| --- | --- | --- |
| `src/game/world.rs`, `src/game/world/*` | Main runtime orchestration, state transitions, snapshots, shop/reward/support/combat/node flows | Active findings P-004, P-005, P-007. Previously reported shop DTO and boss omen no-op issues are resolved in the current worktree. Node completion commit split is improved; broader world dispatch and debug export ownership remain follow-up candidates. |
| `src/game/battle/core/*` | Live battle runtime, event queue, command processing, movement, event log | Active finding P-008. Movement backend policy is now documented by `movement_tick_backend_refactor`: Direct stays as deterministic test harness, Rapier remains terrain/static obstacle correction helper. |
| `src/game/events/combat.rs`, `src/game/combat_preview/*`, `src/game/combat_setup/*` | Combat preview, battle setup, reward/layout/wave handoff into battle runtime | No immediate runtime contradiction was proven in this pass. Existing tests show preview/setup validation is substantial; decomposition should be by source-of-truth boundary if this area is revisited. |
| `src/game/data/*`, `game_resources/data/**` | Live RON loading, static data validation, generated/builtin domain policies | Findings P-010 and P-013. Fail-fast validation is acceptable under current policy, but shop raw data still accepts both the older `hidden_items` authoring shape and newer `stock_items` shape. |
| `src/game/map/*`, `src/game/boss_omen.rs` | Map generation/progression and boss omen overlays | Previous P-002/P-003 are resolved in current code: terminal distance uses queue-based BFS and `ApplyBossOmenStepResult` now fails fast. |
| `src/game/resources/*`, `src/game/employee*.rs`, `src/game/reward*.rs`, `src/game/skill_fragment.rs`, `src/game/stats.rs` | Core resource, employee, reward, skill fragment, stat models | No immediate project-wide finding was proven here beyond their participation in world/data flows. Future audits should focus on one policy axis at a time, such as reward/shop grant duplication or skill fragment progression ownership. |
| `src/game/behavior.rs`, `src/game/world/snapshot.rs`, `src/game/world/state.rs` | Client-facing DTOs, runtime snapshots, live battle update/setup DTOs | Previous P-001 is resolved: selected shop snapshots expose visible stock only. Active battle-record findings remain in `world/state.rs`. |
| `tests/**`, `src/game/**/tests`, generated output dirs | Contract tests, debug output, smoke exports | Finding P-006. Tests are broad, but several helpers write generated artifacts under repository root. |
| `docs/**` | Canonical docs, active/completed goals, historical audits | Finding P-012. Current policy docs say completed goal docs should be consolidated/deleted, but many completed goal artifacts remain. |

## Findings

| ID | Category | Severity | Evidence | Recommendation |
| --- | --- | --- | --- | --- |
| P-004 | Naming drift | Immediate correction | `src/game/world/state.rs` documents `battle_records` as one representative record per `abnormality_uuid`, and `docs/game_rulebook.md` now says the same. However `BattleRecordExport` still has a `battle_uuid` field and `write_battle_record_file(...)` fills it with `battle.abnormality_uuid`; `battle_record_path(...)` also names its parameter `battle_uuid` while receiving an abnormality UUID. | Create `battle_record_debug_export_boundary`. Rename export/path fields to codex/abnormality terminology or restore true per-battle UUID semantics if policy changes. |
| P-005 | Responsibility pile-up | Follow-up refactor | `src/game/world/state.rs::record_battle(...)` both mutates run state and writes pretty JSON files under `CARGO_MANIFEST_DIR/battle_records`; tests and generated files still interact with that repo-root output. | In the same battle-record goal, separate run-state mutation from optional debug export or move generated files under `target/`/temp dirs. |
| P-006 | Test contract drift | Follow-up refactor | `tests/common/mod.rs`, `src/game/world/tests/mod.rs`, and `src/game/battle/core/commands.rs` write debug event logs under `debug_event_log_exports`; `git ls-files` shows tracked files under `battle_records/`, `debug_event_log_exports/`, and `logs/`; `docs/refactor_preparation_plan.md` classifies these as debug/runtime output, not source of truth. | Create `test_debug_output_quarantine`. Move generated output to `target/` or temp dirs by default, and keep gameplay assertions in tests instead of generated JSON files. |
| P-007 | Responsibility pile-up | Follow-up refactor | `src/game/world.rs::execute_with_source_command_id(...)` is now a short pipeline, but the behavior policy still has to be reconstructed across `gate_behavior_action(...)`, `helpers.rs::validate_behavior_payload(...)`, `dispatch_behavior(...)`, `allowed_actions_for_state_context(...)`, and global post-action roster sync. `validate_behavior_payload(...)` and `dispatch_behavior(...)` each match most of `PlayerBehavior`, while many variants skip payload validation and rely on handlers. | Defer until behavior/action work resumes. A follow-up should not add a generic command framework by default; it should first group behavior specs by domain or move validation next to the handler where that reduces the number of cross-file facts required for a new action. |
| P-008 | Responsibility pile-up | Follow-up refactor | `src/game/battle/core/mod.rs` and `src/game/battle/core/commands.rs` are among the largest files and mix runtime state, event-log recording, command processing, and large inline test fixtures. | Create focused battle-core decomposition goals by policy boundary, not file size: event recording, damage/death command application, live command handling, or test fixture extraction. |
| P-010 | Validation gap | Hold | `src/game/data/mod.rs`, `src/game/data/validation.rs`, `src/game/data/skill_data.rs`, and related data modules use `panic!`/`assert!` for live data validation. This is consistent with fail-fast loading, but makes typed startup/reporting and invalid-fixture tests harder. | Hold until loader policy changes. If server startup needs typed data errors, create a data-validation error-boundary goal. |
| P-011 | Naming drift | Follow-up refactor | `src/game/mod.rs` exposes `combat_setup/*` through flattened `#[path]` module names such as `combat_enemy_spawns`, `combat_rewards`, and `combat_scenario_groups`. | Defer unless public module paths are being cleaned. Prefer a single `combat_setup` namespace if external callers no longer need flattened compatibility paths. |
| P-012 | Documentation drift | Follow-up refactor | `find docs/goals -maxdepth 2 -type f` reports 259 goal files; `docs/refactor_preparation_plan.md` says completed goal docs should not be retained after current instructions are moved into canonical docs. | Create a documentation consolidation goal. Move current policy into canonical docs and delete/archive completed goal directories only after user approval. |
| P-013 | Compatibility layer | Follow-up refactor | `src/game/data/shop_data.rs::ShopMetadataRaw` accepts both `hidden_items` and `stock_items`; live shop RON uses `stock_items`; cleanup docs say `hidden_items` should be core-internal runtime state. | Create a small shop raw-schema cleanup only after confirming no authored data still uses `hidden_items`. Prefer one authoring field, likely `stock_items`, and keep internal `ShopSessionState.hidden_items` as derived reserve state. |

## Flow Trace

### World Runtime

- Entry point: `GameCore::execute_with_source_command_id(...)`.
- Planning/preflight boundary:
  - Action gate uses allowed actions.
  - Payload validation lives in `world/helpers.rs`.
  - Per-flow planning exists in focused areas such as node completion and combat result completion.
- Mutation owner:
  - `src/game/world/*` mutates `GameCoreState`, `RunState`, inventory, roster, battle sessions, reward/shop/support state, and snapshots.
- Commit/application boundary:
  - Improved in node completion through `CombatResultNodeCompletion`.
  - Still broad in some support/checkpoint/battle-record helpers.
- Validation boundary:
  - Runtime invalid action handling mostly uses `GameError::InvalidAction`.
  - Static data problems sometimes surface as runtime `InvalidStaticData` and sometimes as loader panics.
- Derived projections:
  - `BehaviorResult`, snapshot DTOs, live battle update DTOs, and map views are derived projections.
- External contracts:
  - `src/game/behavior.rs` and `src/game/world/state.rs` contain most Unity-facing DTO shape.
- Tests:
  - `src/game/world/tests/*` covers large gameplay flows.

### Battle Runtime

- Entry point:
  - `BattleCore::new_from_scenario(...)`, `start_battle_execution(...)`, `step_battle_execution_*`, `apply_live_command(...)`.
- Planning/preflight boundary:
  - Scenario build and combat preview validation occur before battle runtime.
  - Manual skill activation validates readiness/targeting before command application.
- Mutation owner:
  - `BattleCore` owns units, buffs, projectiles, active areas, movement segments, event queue, event log, and live signals.
- Commit/application boundary:
  - Battle event processing is centralized but spread across `mod.rs`, `sim.rs`, `commands.rs`, and skill runtime modules.
- Derived projections:
  - Event log entries, live signals, result stats, and world snapshots are projections of runtime state.
- Tests:
  - Unit-style tests are embedded in large runtime files and integration-style tests live under `tests/skill_*` and `src/game/world/tests/combat.rs`.

### Live Data

- Entry point:
  - `GameDataBase::load_live_embedded()`.
- Planning/preflight boundary:
  - RON deserialization and `GameDataBase::new(...)` validation.
- Mutation owner:
  - Data DBs are immutable after build.
- Validation boundary:
  - Current policy is fail-fast validation via `panic!`/`assert!` in many data modules.
- Derived projections:
  - `ItemRegistry`, generated combat preview contracts, and DTO catalogs are derived from data DBs.
- Tests:
  - `tests/ron_loading.rs`, data module unit tests, and live skill catalog audit.

## Readability Review

- Current reader burden:
  - A maintainer must distinguish current runtime contracts from historical goal documents.
  - A maintainer must distinguish live canonical data from test fixture builders and domain builtins.
  - A maintainer must know which files under root output directories are debug artifacts rather than source of truth.
- Misleading names or abstraction levels:
  - `battle_records`, `battle_record_path(battle_uuid)`, and `BattleRecordExport.battle_uuid` still sound per-battle, but current policy and storage are abnormality-codex style.
  - `ShopMetadataRaw.hidden_items` remains accepted as an authoring field even though live RON has moved to `stock_items` and `hidden_items` is now the internal reserve-state name.
- Facts that should become local/structural:
  - Battle-record export fields should structurally say whether they are keyed by `battle_uuid` or `abnormality_uuid`.
  - Shop authoring should use one visible external field; internal reroll reserve should stay internal.
  - Debug exports should write outside the repo-root source tree by default.
- Expected readability improvement:
  - The highest-value next work is still not a broad rewrite. It is a sequence of small contract cleanups that remove misleading surfaces before deeper decomposition.

## Follow-Up Goal Candidates

### battle_record_debug_export_boundary

Objective:

Make battle-record naming, in-memory state, and debug export shape match the current abnormality-codex policy.

Scope:

- `src/game/world/state.rs`
- `src/game/world/combat.rs`
- `src/game/world/tests/combat.rs`
- `docs/game_rulebook.md`
- `battle_records/` tracked output policy

Completion conditions:

- Runtime naming, dedupe key, file path, and docs describe the same lifecycle.
- `BattleRecordExport` does not call an abnormality UUID `battle_uuid`.
- `battle_record_path(...)` does not name an abnormality-key parameter `battle_uuid`.
- Core state mutation is separated from optional file export if file export remains debug-only.
- Tests no longer rely on repo-root generated files as primary correctness evidence.

Readable-code improvement:

- A reader can tell locally that current records are representative abnormality observations, not per-battle replay archives.

Focused validation:

- `cargo test -p game_core game::world::tests::combat --lib -- --test-threads=1`
- `rg -n "battle_records|record_battle|battle_record_path" src docs tests`
- `cargo check -p game_core`

Policy questions:

- None expected if the current rulebook policy remains: abnormality codex/debug records keyed by `abnormality_uuid`.

### test_debug_output_quarantine

Objective:

Move debug/test-generated artifacts out of the repository source tree by default.

Scope:

- `tests/common/mod.rs`
- `src/game/world/tests/mod.rs`
- `src/game/battle/core/commands.rs` test helper
- `battle_records/`
- `debug_event_log_exports/`
- `logs/`
- `.gitignore` or cleanup policy docs

Completion conditions:

- Test/debug helpers default to `target/` or temp directories.
- Existing tracked generated artifacts are classified for deletion, migration, or retention by user decision.
- Gameplay tests assert behavior directly, not by relying on generated JSON as source of truth.

Readable-code improvement:

- A maintainer no longer has to decide whether root-level JSON/log files are fixtures, debug output, or stale artifacts.

Focused validation:

- relevant event-log/debug-export tests
- `cargo test -p game_core game::world::tests::combat --lib -- --test-threads=1`
- `cargo test -p game_core --test skill_test_suite -- --test-threads=1`
- `git status --short battle_records debug_event_log_exports logs`

Policy questions:

- Should currently tracked generated files be removed now, moved under fixtures, or kept temporarily?

### movement_backend_policy_decision

Status:

- Resolved by `docs/goals/movement_tick_backend_refactor/PLAN.md`.
- Direct is explicitly retained as deterministic test harness.
- Rapier is explicitly retained as terrain/static obstacle correction helper.
- Future movement work should preserve that boundary instead of reopening the policy question.

### shop_raw_stock_schema_cleanup

Objective:

Remove the remaining shop authoring-schema ambiguity between external `stock_items` and internal `hidden_items`.

Scope:

- `src/game/data/shop_data.rs`
- `game_resources/data/events/shops/**`
- shop RON loading tests

Completion conditions:

- Live authored shop data uses one external field for total stock, preferably `stock_items`.
- `hidden_items` remains an internal runtime reserve-state name only.
- Validation rejects the old authoring shape after migration approval.

Readable-code improvement:

- A reader no longer has to infer why "hidden" appears in authoring data after the client-facing hidden-stock cleanup.

Focused validation:

- `cargo test -p game_core shop_data --lib -- --test-threads=1`
- `cargo test -p game_core --test ron_loading -- --test-threads=1`
- `cargo check -p game_core`

Policy questions:

- Can old `hidden_items` authored shop files be rejected immediately, or is a short migration window required?

## No-Issue Notes

- `CombatResultNodeCompletion` is a useful support-free projection of `StagedNodeCompletion`, not a source-of-truth split.
- `SelectedEventSnapshotDto::Shop` no longer exposes `hidden_items` or `hidden_item_uuids`; the selected shop snapshot now exposes visible stock only while `ShopSessionState.hidden_items` remains internal runtime state.
- `EventChoiceEffect::ApplyBossOmenStepResult` is no longer a silent no-op; current runtime fails fast until mechanics are implemented.
- `map_distance_to_terminal(...)` now uses queue-based BFS and has a shortest-path test.
- Direct/Rapier movement is no longer an unresolved policy split in this audit because `movement_tick_backend_refactor` records the current Direct-as-test-harness decision.
- `MapViewDto`, `BehaviorResult`, live battle update DTOs, and skill catalogs are derived projections when they are built from runtime/data state.
- `GameDataBuilder::live_defaults()` is explicitly documented as a fixture helper that loads only live buffs; it should not be treated as production live data ownership.
- `panic!`/`assert!` in data validation is not automatically wrong because current live data policy appears fail-fast. The issue becomes actionable only if startup/reporting needs typed errors or if invalid data reaches runtime.

## Validation Results

Validation run on 2026-07-06:

- `cargo check -p game_core` passed.
- `rg -n "hidden_items|hidden_item_uuids" src/game/behavior.rs src/game/world/snapshot.rs src/game/world/tests/snapshots_and_start.rs src/game/data/shop_data.rs ../game_resources/data/events/shops` confirmed:
  - `SelectedEventSnapshotDto::Shop` and `world/snapshot.rs` no longer expose hidden shop stock;
  - snapshot tests assert `hidden_items` and `hidden_item_uuids` are absent;
  - `ShopMetadataRaw` still accepts `hidden_items` while live shop RON uses `stock_items`.
- `rg -n "ApplyBossOmenStepResult|map_distance_to_terminal|battle_uuid|battle_record_path|debug_event_log_exports" src/game tests docs/game_rulebook.md docs/goals/movement_tick_backend_refactor/PLAN.md` confirmed:
  - `ApplyBossOmenStepResult` now panics instead of silently passing;
  - `map_distance_to_terminal(...)` has BFS implementation and shortest-path tests;
  - `docs/game_rulebook.md` documents `battle_records` as abnormality-codex records, but `BattleRecordExport.battle_uuid` and `battle_record_path(battle_uuid)` still use per-battle names for abnormality-keyed export;
  - debug event log export helpers still write under repository-root `debug_event_log_exports`.
- `git status --short battle_records debug_event_log_exports logs docs/goals/stacked_code_decomposition_audit` showed:
  - modified tracked generated files in `battle_records/...` and `debug_event_log_exports/...`;
  - the new audit report directory as untracked.
- `git ls-files battle_records debug_event_log_exports logs | wc -l` returned `18`, confirming tracked generated/debug files exist.

Observed warnings:

- `auth_server/Cargo.toml` has an unused manifest key warning for `env`.
