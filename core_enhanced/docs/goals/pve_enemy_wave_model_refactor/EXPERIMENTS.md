# PVE Enemy Wave Model Refactor Experiments

## Experiment Log

### 2026-06-19 - Initial Code Reading

Status: succeeded.

Read/search targets:

- `src/game/data/pve_data.rs`
- `src/game/data/corroded_wave_data.rs`
- `src/game/combat_preview/mod.rs`
- `src/game/combat_preview/types.rs`
- `src/game/combat_setup/enemy_spawns.rs`
- `src/game/data/validation.rs`
- `tests/ron_loading.rs`

Findings:

- `PveWaveEnemyData` currently stores `kind`, `profile_id`, and `abnormality_id` together.
- Corroded employee entries require `profile_id`, but the struct still requires `abnormality_id`.
- Generated corroded waves currently set `abnormality_id` to the same value as `profile_id`.
- Generated corroded wave composition lives in `combat_preview/mod.rs`, even though it is not purely presentation logic.
- Runtime spawn code already treats corroded employees as `BattleUnitSource::CorrodedEmployee`.

Decision:

- Use this goal to remove the ambiguous wave enemy identity shape and move generated wave resolution to shared domain code.

### 2026-06-19 - Plan Review Against Runtime Code

Status: succeeded.

Read/search targets:

- `docs/goals/pve_enemy_wave_model_refactor/PLAN.md`
- `src/game/data/pve_data.rs`
- `src/game/data/corroded_wave_data.rs`
- `src/game/combat_preview/mod.rs`
- `src/game/combat_preview/types.rs`
- `src/game/combat_setup/enemy_spawns.rs`
- `src/game/combat_preview/threat.rs`
- `src/game/world/state.rs`
- live `/mnt/f/work/simulator/game_resources/data/pve/encounters.ron`

Findings:

- The plan's main diagnosis is valid: the live model still mixes abnormality and corroded employee identity fields.
- `SpawnWaveEnemyEntry` also mixes `profile_id` and `abnormality_id`, and may be Unity-facing through preview/setup snapshot flows.
- Runtime battle setup consumes `CombatPreview.spawn_waves[*].enemy_entries`; it does not currently resolve raw `PveWaveData` itself.
- `primary_enemy_kind()` reads only legacy `wave.enemies.first()`, so source-based waves can be misclassified in preview briefing.
- `FacilityEntity` is currently an enum placeholder, not an implemented profile-backed enemy source.

Decision:

- Update the plan to preserve one preview-resolved wave composition instead of making preview and battle setup independently regenerate waves.
- Add a Unity-facing DTO stop condition for `SpawnWaveEnemyEntry`.
- Add preview briefing coverage to the test plan.
- Keep `FacilityEntity` unsupported in this goal unless a real data source exists.

### 2026-06-19 - Internal Wave Model Refactor

Status: succeeded.

Changes:

- Replaced `PveWaveEnemyData` struct with explicit variants:
  - `Abnormality { abnormality_id, tier, count }`
  - `CorrodedEmployee { profile_id, tier, count }`
  - `FacilityEntity { profile_id, tier, count }`
- Removed top-level legacy `PveWaveData.enemies`.
- Made `PveWaveData.source` the single wave composition source.
- Moved generated corroded wave resolution to `src/game/wave_resolution.rs`.
- Updated combat preview to call the shared resolver while preserving the current preview-resolved composition flow.
- Kept `SpawnWaveEnemyEntry` shape unchanged because it may be Unity-facing, but stopped using `abnormality_id` as a corroded employee profile fallback.
- Updated setup snapshot abnormality id collection to include only actual abnormality entries.
- Updated live PVE RON to use `source: Manual([...])` and `source: GeneratedCorroded(...)`.
- Updated preview briefing tests so source-based manual/generated waves no longer depend on legacy `wave.enemies`.

Decision:

- `FacilityEntity` remains unsupported; no facility profile source was invented.
- Generated corroded seed behavior was preserved.
- Battle setup continues to consume preview-resolved `SpawnWave.enemy_entries`; it does not regenerate waves separately.

## Failed Attempts

### 2026-06-19 - Mechanical Fixture Rewrite Broke Some Test Fixture Delimiters

Status: fixed.

Failure:

- A mechanical rewrite converted some unrelated `vec![...]` fixture closings to `}])`.
- This produced syntax errors in `src/game/combat_preview/mod.rs`, `src/game/events/combat.rs`, `src/game/world/tests/mod.rs`, and `tests/common/mod.rs`.

Fix:

- Restored non-wave fixture vector delimiters manually.
- Re-ran focused checks until test compilation succeeded.

### 2026-06-19 - `cargo test -p game_core ron_loading -- --nocapture` Did Not Execute RON Tests

Status: noted and compensated.

Finding:

- This command filters by test name, not integration test file name, so it ran zero `ron_loading.rs` tests.

Fix:

- Ran concrete test names such as `live_pve_references_resolve`.
- Final broad `cargo test -p game_core` executed all `ron_loading.rs` tests.

## Validation Commands

- `cargo check -p game_core`
  - Result: passed.
- `cargo test -p game_core live_pve_references_resolve -- --nocapture`
  - Result: passed.
- `cargo test -p game_core pve_encounter -- --nocapture`
  - Result: passed.
  - Note: expected `#[should_panic]` validation tests print panic messages while passing.
- `cargo test -p game_core generated_corroded_wave_source_resolves_during_preview_generation -- --nocapture`
  - Result: passed.
- `cargo test -p game_core combat_preview -- --nocapture`
  - Result: passed.
- `cargo fmt`
  - Result: passed.
- `cargo test -p game_core`
  - Result: passed. 473 lib tests, 3 live item tests, 3 live skill audit tests, 16 RON loading tests, 10 skill refactor validation tests, 4 skill suite tests, and doc tests all passed.
