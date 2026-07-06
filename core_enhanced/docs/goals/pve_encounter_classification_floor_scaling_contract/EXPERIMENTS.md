# PVE Encounter Classification And Floor Scaling Contract Experiments

| Date | Attempt | Result | Next |
| --- | --- | --- | --- |
| 2026-06-27 | Goal creation | Created a standalone implementation goal for the newly decided PVE encounter classification and Floor scaling policy. This goal supersedes the narrower difficulty-removal planning target and captures explicit `Normal`/`Elite`/`NormalBoss`/`FinalBoss` classification, Floor 1-8 stage scaling, generated wave budget scaling, and explicit extra waves. | Begin implementation by auditing runtime `PveEncounter`, live RON, map encounter assignment, preview generation, and battle construction paths before editing code. |
| 2026-06-27 | Merge superseded difficulty-removal plan | Absorbed the useful audit context from the older difficulty-removal goal: `map_encounters.rs` uses encounter difficulty as a candidate filter, live PVE difficulties span `1..=7`, high Endless Floors can exceed that authored range, and `CorrodedWavePreset.difficulty` is a separate follow-up concern. The older goal should not remain a second source of truth. | Use this goal as the only implementation target. |
| 2026-06-27 | Phase 4A startup runtime/data audit | Confirmed the old implementation still has `PveEncounter.difficulty`, `PveEncounter.abnormality_id`, difficulty-window map assignment, and silent candidate fallback. Also confirmed live `pve/encounters.ron` currently consists of suppression-themed encounters with mandatory abnormality ids; no pure `Normal` corroded-employee encounter pool exists yet. | Stop for user policy before implementation: classifying existing suppression encounters as `Normal` would change content meaning, while true `Normal` support requires deciding how no-primary combat battles appear in result/record/research DTOs. |
| 2026-06-27 | Primary identity policy resolved | User clarified the final classification semantics: `Normal` is pure corroded employees, `Elite` is corroded employees plus an elite monster, `NormalBoss` is corroded employees plus a non-final boss monster, and `FinalBoss` is corroded employees plus a final boss. `primary_abnormality_id = None` is valid for `Normal` only and means no response research/completion/abnormality fragment reward. | Resume implementation with true generic Normal encounters and optional primary abnormality identity. |
| 2026-06-27 | Runtime implementation | Replaced encounter difficulty selection with explicit encounter classes, introduced optional `primary_abnormality_id`, migrated live PVE RON, added run-policy Floor scaling stages, applied enemy stat scaling and generated-wave budget scaling, and updated map assignment to fail on empty role-compatible pools instead of falling back silently. | Run focused and broad verification. |
| 2026-06-27 | Stale test repair | `cargo test --lib` initially failed because old preview fixtures modeled `Elite` encounters without spawning their primary abnormality, and live Defense smoke tests still used `suppress_burrowing_heaven`, which is now a `NormalBoss` encounter. Repaired tests by using `Normal` for pure corroded generated-wave previews, spawning only the configured primary abnormality in elite fixtures, and moving live Defense route smoke tests to `suppress_warm_hearted_woodsman`. | Re-run full validation. |
| 2026-06-27 | Phase 4A validation | Focused tests, broad lib tests, live RON loading, and checks all passed. | Mark Phase 4A complete and resume Phase 4B when requested. |

## Verification Log

Focused checks run during implementation:

```bash
cargo test pve_encounter --lib
cargo test map_encounter --lib
cargo test generated_map --lib
cargo test floor_scaling --lib
cargo test generated_wave_budget_scaling --lib
cargo test manual_wave_counts_are_not_scaled --lib
cargo test draft_stats_pipeline_applies_floor_stat_scale --lib
cargo test final_standard_floor_boss_node_selects_final_boss_encounter --lib
cargo test generated_standard_maps_use_gate_before_final_floor_and_boss_on_final_floor --lib
cargo test generated_corroded_wave_source_resolves_during_preview_generation --lib
cargo test spawn_waves_store_explicit_enemy_entries_separate_from_briefing --lib
cargo test authored_encounter_battlefield_overrides_drive_generated_preview --lib
cargo test authored_static_obstacles_replace_archetype_default_obstacles --lib
cargo test defense_combat_node_smoke_writes_debug_event_log_export --lib
cargo test live_ron_defense_route_playable_path_runs_to_combat_result --lib
cargo test battle_setup_snapshot_live_defense_state_request_returns_battle_update_without_advancing_cursor --lib
cargo test --test ron_loading
cargo check --lib
cargo check -p game_server
```

Final broad checks:

```bash
cargo test --lib
cargo test --test ron_loading
cargo check --lib
cargo check -p game_server
```

Result: all passed on 2026-06-27.

## Failed Attempts

Record any rejected or failed approaches here. Include why the approach failed and what was changed before retrying.

### 2026-06-27: Stopped before implementation on primary abnormality / Normal encounter policy

- Runtime evidence:
  - `PveEncounter` still requires `abnormality_id: String` and `difficulty: u8`.
  - `map_encounters.rs` still filters by difficulty window and silently falls back to broader pools.
  - `node_flow.rs`, `ActiveBattleSession`, `CombatBattleState`, selected-event snapshots, battle record export, and research reward application all treat battle abnormality identity as a mandatory string.
  - Live `pve/encounters.ron` entries are all named and authored as suppression encounters with an `abnormality_id`; the current live data does not contain an explicit generic `Normal` encounter pool with `primary_abnormality_id = None`.
- Why implementation stopped:
  - The goal says `Normal` may omit `primary_abnormality_id`, but the runtime/result/research surfaces currently require a primary abnormality identity.
  - The goal's stop conditions explicitly require stopping if the rename needs Unity-facing DTO/save decisions or if existing live encounters cannot be classified cleanly without replacing content.
- Required user decision:
  - Decide whether Phase 4A should introduce true generic `Normal` corroded-employee encounters with no primary abnormality and change affected battle/result/research surfaces accordingly, or whether the encounter policy should keep every current PVE encounter tied to a primary abnormality.

Resolution:

- User chose true generic `Normal` encounters:
  - `Normal`: pure corroded employees, `primary_abnormality_id = None`.
  - `Elite`: corroded employees plus elite monster, `primary_abnormality_id = Some(...)`.
  - `NormalBoss`: corroded employees plus non-final boss monster, `primary_abnormality_id = Some(...)`.
  - `FinalBoss`: corroded employees plus final boss monster, `primary_abnormality_id = Some(...)`.
- `Normal` encounters do not drive abnormality research, response completion, or abnormality fragment rewards.

### 2026-06-27: Broad lib test exposed stale post-policy fixtures

- Failing symptoms:
  - Preview fixtures declared `Elite` encounters but did not spawn the configured primary abnormality target.
  - One preview fixture spawned several non-primary abnormalities, violating the single-primary current policy.
  - Live Defense smoke tests used `suppress_burrowing_heaven`, which is now a `NormalBoss`/Boss encounter instead of a Defense encounter.
- Fix:
  - Pure corroded generated-wave preview fixture became `Normal` with `primary_abnormality_id = None`.
  - Elite preview fixtures now spawn only their configured primary abnormality plus allowed corroded employees.
  - Live Defense smoke tests now use `suppress_warm_hearted_woodsman` and assert route/victory behavior in a way that allows optional non-required support waves.
- Revalidation:
  - `cargo test --lib` passed.
  - `cargo test --test ron_loading` passed.
  - `cargo check --lib` passed.
  - `cargo check -p game_server` passed.
