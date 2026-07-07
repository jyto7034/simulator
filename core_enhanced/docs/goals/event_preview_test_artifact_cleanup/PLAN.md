# Goal: Event Preview Test Artifact Cleanup

## Objective

Clean up four low-risk boundary issues that have clear policy decisions and should not require new gameplay decisions:

1. Remove Event node `event_id: None` fallback.
2. Clarify or remove `CombatPreview.enemy_stat_scale` serde round-trip ambiguity.
3. Keep test/debug generated artifacts out of the repository root.
4. Rename refactor-era test files to current contract names.

The goal is long-term cleanup. Do not preserve legacy fallback behavior, dual schema, or compatibility aliases unless current runtime code and live data prove a temporary transition is necessary. If a transition is necessary, record the reason and removal condition here before implementing it.

## Source Of Truth Order

Verify facts in this order:

1. Actual runtime code.
2. Live RON/data.
3. Unity-facing snapshot/command/WebSocket contract and probe behavior.
4. Latest canonical docs.
5. Historical goal/audit docs only as context.

Relevant starting points:

- `src/game/world/map_content.rs`
- `src/game/world/event_node.rs`
- `src/game/data/event_data.rs`
- `src/game/data/validation.rs`
- `src/game/combat_preview/types.rs`
- `src/game/combat_preview/mod.rs`
- `src/game/behavior.rs`
- `tests/common/mod.rs`
- `tests/skill_runtime_contract.rs`
- `docs/core_runtime_contract.md`
- `docs/refactor_preparation_plan.md`
- `docs/README.md`

## Fixed Policies

### Event Node Identity

- Event nodes must explicitly name `event_id`.
- If an Event node lacks `event_id`, do not choose the first event as fallback.
- Missing `event_id` must fail live data validation or panic/fail-fast during local QA.
- Live RON must be updated so every Event node has an explicit event id.

### CombatPreview Scale

- `CombatPreview.enemy_stat_scale` must not be an ambiguous official wire field that silently disappears or resets through serde round-trip.
- If the scale is internal calculation state, keep it out of Unity-facing/storable DTOs and make the internal boundary explicit.
- If Unity needs display information, expose a presentation field such as `threat_level`, `enemy_power_hint`, or `floor_scaling_summary`, not raw internal scale.
- Before changing shape, verify current probe/Unity-facing snapshot usage.

### Test And Debug Artifacts

- Tests must not create or update generated artifacts under repository root.
- Test/debug generated output belongs under `target/`, a temp directory, or an explicitly requested export directory.
- Generated debug JSON is not a golden fixture unless a test explicitly treats it as such and documents why.
- Prefer assertions over “file exists” checks for gameplay contracts.

### Test Naming

- Test files should describe the current contract they protect, not the refactor that originally introduced them.
- Refactor-era test file names should be renamed to current-domain names if no external harness depends on the old name.
- Update Cargo targets, docs, scripts, and goal references that invoke the renamed tests.

## Plan

1. Inventory current code and data.
   - Check whether `event_id: None` fallback still exists for Event nodes.
   - Check live RON for Event nodes without explicit `event_id`.
   - Check whether `CombatPreview.enemy_stat_scale` is serialized into Unity-facing snapshots or only used internally.
   - Check current debug artifact paths and whether prior cleanup goals already moved some paths under `target/`.
   - Check references to refactor-era test file names in tests, docs, scripts, and CI-like commands.

2. Remove Event fallback.
   - Replace first-event fallback with validation failure or fail-fast runtime error.
   - Update live RON to provide explicit `event_id`.
   - Add or update tests that reject missing Event node `event_id`.

3. Resolve `CombatPreview.enemy_stat_scale`.
   - If internal only, keep it as an internal field and ensure it is not part of storable/Unity-facing DTO round-trip expectations.
   - If it currently leaks into wire DTOs, either remove it from that boundary or replace it with a presentation DTO field.
   - Add tests that prove serde round-trip does not silently change user-visible preview semantics.

4. Quarantine generated artifacts.
   - Move remaining test-generated export paths to `target/` or temp dirs.
   - Do not delete intentionally tracked fixtures unless their purpose is verified and replaced by assertions or explicit fixtures.
   - Update `.gitignore` or docs only if runtime code confirms they are needed.

5. Rename refactor-era tests.
   - Rename files only after checking all direct references.
   - Choose names that describe current policy, for example `skill_runtime_contract.rs` or `skill_step_pipeline.rs`.
   - Update commands in docs/goals if they remain relevant; avoid broad mechanical churn in stale goal docs unless they are active references.

6. Validate.
   - Run focused tests for event node loading/validation, combat preview serialization, debug export helpers, and renamed skill tests.
   - Run broader `cargo check -p game_core`.
   - Run `cargo fmt --check`.

## Completion Conditions

- Event nodes cannot silently fall back to the first event when `event_id` is missing.
- Live RON loading fails fast for missing Event node `event_id` or all live Event nodes explicitly provide `event_id`.
- `CombatPreview.enemy_stat_scale` is either clearly internal and safe from wire/storable round-trip ambiguity, or replaced with explicit presentation DTO data.
- Tests/debug runs no longer write generated artifacts to repository root by default.
- Refactor-era test filenames are renamed to current contract names, or the plan records a concrete reason not to rename.
- Focused tests and final validation commands pass.
- `EXPERIMENTS.md` records failed attempts, fixes, and reruns.
- `EXPERIMENT_NOTES.md` records any follow-up cleanup that is useful but outside this goal.

## Stop Conditions

Stop and report questions instead of deciding locally if:

- Unity currently depends on `CombatPreview.enemy_stat_scale` as a public wire field.
- A missing Event node `event_id` is intentionally authored for a gameplay reason.
- Renaming a test file would break external automation that cannot be updated in this goal.
- Existing repository-root JSON files are intentional golden fixtures rather than generated debug artifacts.
- A DTO shape change would require Unity migration policy beyond this low-risk cleanup.

## Suggested Verification Commands

```bash
rg -n "event_id: None|event_id\\)|event_id" src/game ../game_resources/data tests
rg -n "enemy_stat_scale|CombatPreview" src/game tests docs
rg -n "debug_event_log_exports|battle_records|logs/" src tests docs
rg -n "skill_refactor_validation|skill_runtime_contract" . ../game_server ../game_resources

cargo test -p game_core event_node --lib -- --test-threads=1
cargo test -p game_core combat_preview --lib -- --test-threads=1
cargo test -p game_core --test ron_loading -- --test-threads=1
cargo test -p game_core --test <renamed_skill_contract_test> -- --test-threads=1
cargo check -p game_core
cargo fmt --check
```
