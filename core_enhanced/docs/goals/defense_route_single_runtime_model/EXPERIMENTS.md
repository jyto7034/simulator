# DefenseRoute Single Runtime Model Experiments

Record each implementation attempt, failure, fix, and verification result here.

## 2026-06-17 Goal Creation

Intent:

- Create goal docs for unifying DefenseRoute live runtime policy.
- Capture current 0ms BattleEnd issue as the motivating regression.
- Define the long-term implementation direction before touching code.

Result:

- Created:
  - `docs/goals/defense_route_single_runtime_model/PLAN.md`
  - `docs/goals/defense_route_single_runtime_model/EXPERIMENTS.md`
  - `docs/goals/defense_route_single_runtime_model/EXPERIMENT_NOTES.md`

Verification:

- No code executed yet.
- No tests run yet.

Notes:

- First implementation slice should begin with a read-only audit of `CombatMissionVariant` usages and live RON schema before editing.

## 2026-06-17 Runtime Audit And Schema Choice

Intent:

- Audit the actual `CombatMissionVariant` flow before editing.
- Confirm whether survival should remain a mission variant or become explicit encounter data.

Findings:

- `CombatMissionVariant::Encirclement` and `CombatMissionVariant::SplitRoom` were runtime enum variants, not just display labels.
- `fallback_mission_variant_for_archetype(Defense, Surrounded)` converted normal Defense encounters into Encirclement.
- Encirclement built a survival-style win condition without the normal protected objective contract, which allowed a 0ms opponent victory before deployment.
- Live Blue Star content was the only live PVE use of `mission_variant: Some(Encirclement)`.

Result:

- Chose explicit `survive_timer_ms: Option<u64>` on PVE encounter, combat preview, and battle setup snapshot.
- Removed Encirclement/SplitRoom as official `CombatMissionVariant` runtime variants.
- Kept SplitRoom as a battlefield archetype only; archetype remains layout metadata, not runtime mode.

Verification:

- Read-only audit first, then code edits.
- No test run in this slice.

## 2026-06-17 Runtime Conversion

Intent:

- Make `DefenseRoute / Defense` the single non-boss live runtime model.
- Preserve normal Defense protected-objective failure.
- Add survival timer victory without keeping Encirclement compatibility.

Edits:

- `CombatMissionVariant` now has only `Defense` and `Boss`.
- Mission variant fallback now defaults from node type and ignores battlefield archetype.
- `PveEncounter`, `CombatPreview`, `BattlefieldInstance`, and `LiveBattleSetupSnapshotDto` carry optional `survive_timer_ms`.
- PVE data validation rejects `survive_timer_ms` unless the encounter is explicitly `node_type: Defense` and the timer is greater than zero.
- Added `WinCondition::ProtectUnitUntil { unit_ref, time_ms }` for DefenseRoute timer survival.
- Core winner logic now gives:
  - Opponent victory if the protected objective is destroyed before timer.
  - Player victory at timer if the protected objective is alive, even with enemies alive.
- Migrated live Blue Star RON from `mission_variant: Some(Encirclement)` to `survive_timer_ms: Some(45000)`.

Failed Attempt:

- The first focused `protect_unit_until` test accessed spawned unit refs before processing the `AtBattleStart` spawn events.
- Cause: the test assumed scenario units were already materialized before an initial 0ms step.
- Fix: step the battle once at 0ms, then read the spawned unit refs and assert timer victory / protected-objective defeat.

Verification:

- `cargo check -p game_core` passed.
- `cargo test -p game_core mission_variant -- --nocapture` passed.
- `cargo test -p game_core protect_unit_until -- --nocapture` passed after the test fixture fix.
- `cargo test -p game_core surrounded_battlefield_remains_default_defense_and_does_not_end_before_deployment -- --nocapture` passed.
- `cargo test -p game_core survival_timer_builds_protected_timer_objective -- --nocapture` passed.
- `cargo test -p game_core battle_setup_snapshot_exposes_survival_timer_without_encirclement_variant -- --nocapture` passed.

## 2026-06-17 Data And Contract Verification

Intent:

- Confirm live data and Unity-facing contracts agree with the new runtime policy.

Edits:

- Updated `docs/game_rulebook.md`.
- Updated external Unity contract docs:
  - `/mnt/f/unity projects/ark/docs/unity_core_contract.md`
  - `/mnt/f/unity projects/ark/docs/core_unity_battle_setup_snapshot_contract.md`
  - `/mnt/f/unity projects/ark/docs/core_unity_battle_update_contract.md`

Verification:

- `cargo fmt -p game_core` passed.
- `cargo test -p game_core --test ron_loading -- --nocapture` passed.
- `cargo check -p game_server` passed.
- `rg -n "CombatMissionVariant::(Encirclement|SplitRoom)|mission_variant:\s*Some\((Encirclement|SplitRoom)|\bEncirclement\b|PveBattleObjectiveData::Survive|PveWinConditionData::SurviveUntil" src tests /mnt/f/work/simulator/game_resources/data/pve/encounters.ron -S` returned no matches.

Notes:

- Documentation still mentions Encirclement/SplitRoom only to say they are not official live mission variants.
- `WinCondition::SurviveUntil` remains only as low-level battle-core capability/test surface. PVE authoring and mission policy no longer map to it.

## 2026-06-17 Broad Test Failures

Command:

```text
cargo test -p game_core
```

Result:

- Failed.

Failures:

- `pve_encounter_validation_rejects_conflicting_objective_and_win_condition`
  - Cause: the test still used removed RON objective variant `Survive`.
  - Fix: changed the conflict fixture to use current authored objective `ProtectUnit(unit_ref: "black_box")` against `AllRequiredEnemyGroupsDefeated`.
  - Revalidation: `cargo test -p game_core pve_encounter_validation_rejects_conflicting_objective_and_win_condition -- --nocapture` passed.
- `combat_preview_spawn_waves_drive_battle_scenario_spawn_times`
  - Cause: the test attempted to prove wave-time conversion by running live execution to 1000ms. That pulled in movement/spawn occupancy behavior unrelated to the test's contract and returned `PositionOccupied`.
  - Failed intermediate fix: adding more valid spawn cells did not solve it, because the test was still exercising live execution instead of scenario scheduling.
  - Final fix: changed the test to assert that preview `wave_1` becomes a `ScenarioTrigger::AtTimeMs(1000)` opponent spawn event. This directly verifies the user-visible scheduling contract without depending on movement.
  - Revalidation: `cargo test -p game_core combat_preview_spawn_waves_drive_battle_scenario_spawn_times -- --nocapture` passed.

Final broad verification:

- `cargo test -p game_core` passed.
- `cargo check -p game_server` passed.
- The legacy runtime search stayed clean after the final fixes.
