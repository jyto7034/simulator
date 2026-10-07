# P03 Summon Cutover Experiments

## 2026-08-16 — GEX-00 intake

- Exact selected member: `GS-CWBS-P2G50-002 / GOAL-CWBS-P03-SUMMON-CUTOVER`.
- Phase/nodes: `PHASE-CWBS-03-SUMMON-CUTOVER / AG-CWBS-03-SUMMON-CUTOVER`.
- Direct predecessor: P02 `COMPLETION_AUDIT.md / Audit Pass 1 / Complete / High`; its audit explicitly records the
  authority-free actor adapter and shared Kernel as a valid P03 intermediate.
- Initial full baseline is inherited: `game_core 1,080 + game_server 72 / 0 failed`; P03 does not repeat it.
- Relevant current files are pre-existing dirty current actual. P01/P02-owned changes are preserved; no reset, restore,
  stash, cleanup or whole-tree format is authorized.
- Current code inspection confirms:
  - `BossTaskData`/`CompiledBossTask` and `BossTaskResultData` have no summon variants;
  - `BossRuntimeDatabase` contains patterns, actor behaviors and encounters but no summon definition catalog;
  - `BattleUnitSource` has no summoned-unit identity;
  - `BossEncounterRuntime` owns reusable per-actor behavior state, while phase authority is still encounter-specific;
  - the P01 Spawn Kernel and P02 `RuntimeUnitLifecycle::Despawned` seam are present;
  - live `data/boss_runtime/base.ron` has only the Little Red reference path and no summon opt-in.
- Current pre-mutation hashes:
  - `data/boss_runtime_data.rs` `6ddd7baea472fcec6302cd27b1200acaacfb99015db4dc62ae7de002e0bd3103`
  - `data/live_ron.rs` `b761ceb66c6989245eb2803a5ecbb96678ab23520e14c606b5920fdd9fe7dc5a`
  - `data/mod.rs` `859e9d1d2951e58aeb5234cff07118daf1ae597145c528f84105b20fd944c9b6`
  - `battle/types.rs` `5427f0d63baa5c940e82ae9a59a726fe92bb05fb7878b2118a05c8127970fb83`
  - `battle/scenario.rs` `7c72b27663e49369b7fa4dfd7f0cdb683f587246b1394748ca2d1e64220de1fe`
  - `battle/core/boss_runtime.rs` `76e31f809c06dd2892b90837f5cf3dba09d73f7614e6e2172a4ac26125648ba3`
  - `battle/core/build.rs` `9e5536557b4576cd8819106acd3e80b727c8e42e73d3a9225f5036b50f2bf75e`
  - `battle/core/mod.rs` `0240270aab213e35d23e7f0fba6aa76a22fa204fd3b6fc21cede337a980c338c`
  - `battle/core/death_runtime.rs` `abdcb6de0b58487011f28df0652a7531fccabec2fa2e6ac833662d30eaf6ce71`
  - `battle/core/spawn_kernel.rs` `209e07ebd96d2239d5914c56bf857a1bf3120599f081790af7dd7c897efca340`
  - live `data/boss_runtime/base.ron` `f696c9d054ed1e4397809f80c2707e06614f4324a1829bb7efa43ccb666c223a`
  - bundle manifest `06b0f18905acf5efb75e3c790be85d66879735614c3b2e9ee1aa42ac81056d4d`
- Production mutation: none in P03 at this point.

## 2026-08-16..17 — GEX-10 implementation

- Added `data/summoned_units/base.ron` as a strict, empty-by-default live source and compiled it before Boss Runtime
  publication. Invalid local definition data is attributed to the summoned-unit source rather than the Boss source.
- Added independent definition/source identity and provenance:
  `definition_id + summoner_instance_id + summon_sequence + batch_uuid`. The encounter difficulty stat scale is applied
  once to definition stats; summoner stats, buffs and classification are not read.
- Added typed Boss Runtime data/compiler contracts for Summon, all four results, pattern-local batch handles,
  `SummonBatchCleared`, Around/SpawnZone, closed occupancy sets, Fail/bounded Wait, lifetime, removal signal and signal
  emission. Publication rejects invalid combinations, missing definitions/behaviors/routes, recursive summon and
  unproduced or encounter-unbound signal graphs.
- Added ActiveBattle summon state and Kernel consumer:
  - per `(summoner instance, definition)` `alive/total` counters;
  - deterministic successful-batch UUID/member continuation only;
  - atomic all-member placement/commit with no partial, eviction or fallback;
  - waiter ordering `(admitted_at_ms, request_id)` and space-before-equal-timeout;
  - summoner/state cancellation and limit recheck before allocation;
  - exact batch handle binding and clear-once observation.
- Added Persist/Despawn/Kill settlement. Despawn uses nondeath inactive cleanup and ends at the distinct
  `RuntimeUnitLifecycle::Despawned`; it creates no `UnitDied`, graveyard, death trigger, killer credit or reward path.
  Kill invokes normal death/on-death with `killer_instance_id = None`.
- Summoned actors reuse Boss task/query/movement execution with explicit route context, but are appended outside
  `encounter_actor_count` and cannot drive phase transitions, Wave actors or terminal outcomes.
- Generic required-group victory explicitly ignores `SummonedUnit`; focused coverage proves Persist units remain active
  while the prior required group can still finish.
- Live production opt-in remains zero definitions. No SavePoint DTO or active-run durable payload was extended.

## Focused validation and rebaseline

All commands use `CARGO_TARGET_DIR=/tmp/cwbs_goal`. The initial 1,152-pass full baseline was not repeated in P03.

| Command / seam | Result |
| --- | --- |
| `cargo check -p game_core` | pass, zero warnings before final signal binding refinement |
| `cargo test -p game_core game::data::summoned_unit_data::tests` | 3 passed / 0 failed |
| `cargo test -p game_core game::data::boss_runtime_data::tests::summon_` | 4 passed / 0 failed |
| `cargo test -p game_core game::battle::core::summon_runtime::tests` | 10 passed / 0 failed |
| `cargo test -p game_core game::data::boss_runtime_data::tests` | 11 passed / 0 failed |
| `cargo test -p game_core game::battle::core::boss_runtime::tests` | 22 passed / 0 failed |
| `cargo test -p game_core game::battle::core::spawn_kernel::tests` | 10 passed / 0 failed |
| `cargo test -p game_core game::battle::core::wave_runtime::tests` | 10 passed / 0 failed |
| `cargo test -p game_core game::combat_setup::boss_runtime::tests` | 3 passed / 0 failed |
| `cargo test -p game_core encounter_compendium` | 7 passed / 0 failed |
| `cargo test -p game_core game::data::live_ron::tests` | final 77 passed / 0 failed |
| `cargo test -p game_core game::events::combat::tests` | final 14 passed / 0 failed |
| `cargo check -p game_server` | pass |

Two focused suites first exposed stale P02 test fixtures rather than production regressions:

- Live RON: `75 passed / 1 failed`. The old fixture set `survive_timer_ms` on a wave still marked
  `required_for_victory = true`, so current Wave activation validation correctly rejected `waves` before the intended
  zero-timer assertion. The test wave was changed to non-required; exact retry passed and the suite rebaselined 76/0,
  then 77/0 after the summoned-source provenance test was added.
- Combat assembly: `13 passed / 1 failed` for the same survival fixture premise. The test-only wave was changed to
  non-required; exact retry passed and the suite rebaselined 14/0.

No production data was changed to accommodate either failure.

## Current GEX-10 target identities

- `data/summoned_unit_data.rs` `ba387bf3d492950eb7d0cb65bf886901689a29b3717373d17c4579c1b1df96f8`
- `data/boss_runtime_data.rs` `5c259a9892c30c5ab3f818370ac3a8f241ffd2b3562a365d0525da4d13c6701d`
- `data/live_ron.rs` `f1c9f4beb34e9191da4d2b0a2f40f9b8b4a843b3d051672cfc8a3b174aa3db57`
- `data/mod.rs` `24ccad25afb91616ea2dfee2eccb3b0dbd550ab74f1cd62a03dcf76c354c4f66`
- `battle/types.rs` `3567bbdb2105074258e024bf1504c54fdaf352eb686166cd315abe79dcdd11fb`
- `battle/core/summon_runtime.rs` `4eb0c40e262d40b23300702cb48a3dcfa5ea5ca30624b539020fcf04c8102590`
- `battle/core/boss_runtime.rs` `05623acaba2d5f37de8762ba69098631c99c18cc60c39caccac3a17f9ffb0c1b`
- `battle/core/spawn_kernel.rs` `84845f55cb80722683369941b5f18bf8bcb209119cb4a2436bb7eec260a1da84`
- `battle/core/mod.rs` `b15d6e5cf6b364034ced313511e0d24ff921dbfbeb632ea616ea2762e8d1c2`
- `battle/core/sim.rs` `8aa218da7b2e8df01d0aa851e4aacdf0088de5d5f5c208e0f4e21ef9f059b794`
- `combat_setup/boss_runtime.rs` `c44ddded94dd7107cfa6ca9cc6e4ff9e6f5d8dd09ef6b9d5084b046b3e8eaebe`
- `events/combat.rs` `b0dd16350acd3f5d975661fae5c36a3da94ad854501eac5584a76e59a017a708`
- `encounter_compendium.rs` `43c8361ea34caa74808af830e20cfc108c71a71add5c56f0562c6f7d44624eef`
- live summoned RON `22937c02fa51fd7f30b202d62810c0b4bcd11433f823bc642394f396735ab0e2`
- live Boss RON `86d23909b1f7ca9e282a2f2d51f621bbf2f55ab4bef49af8e8c1cc6193bcedca`

## Post-record freeze check

- `git diff --check`: passed.
- `CARGO_TARGET_DIR=/tmp/cwbs_goal cargo check -p game_core -p game_server`: passed, zero warnings.
- `CARGO_TARGET_DIR=/tmp/cwbs_goal cargo test -p game_core game::data::boss_runtime_data::tests`:
  11 passed / 0 failed.
- `CARGO_TARGET_DIR=/tmp/cwbs_goal cargo test -p game_core game::battle::core::summon_runtime::tests`:
  10 passed / 0 failed.
- All identities above were recomputed after these commands and matched exactly. They are the frozen P03 GEX-10 target
  for fresh GEX-20.
