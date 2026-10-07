# P02 Wave Cutover Experiments

## 2026-08-16 — GEX-00 intake

- Exact selected member: `GS-CWBS-P2G50-002 / GOAL-CWBS-P02-WAVE-CUTOVER`.
- Phase/nodes: `PHASE-CWBS-02-WAVE-CUTOVER / AG-CWBS-02-WAVE-CUTOVER + WP-CWBS-03`.
- Direct predecessor: P01 `COMPLETION_AUDIT.md / Audit Pass 3 / Complete / High`; all P01 production hashes matched at
  dependency handoff.
- Initial full baseline is inherited: `game_core 1,080 + game_server 72 / 0 failed`; P02 does not repeat it.
- Current relevant files are pre-existing dirty current actual and match P2G-20 recorded hashes:
  - `data/pve_data.rs`: `bcd3d5daa1789058b9b3c4080723bae0a7d72ef6d538f1b84770aa7ee2373b00`
  - `battle/scenario.rs`: `a33d434b276b495dec8d9ba43823dbbcdd7b787dd1bb898f3f7703a83cf3195a`
  - `battle/core/build.rs`: `1125b3846c5b9e48579befe0b34521f6da36070cc83bc7553edf4e92cbed2122`
  - `battle/core/sim.rs`: `af5130fc76e5d80226767f8e43c3f6d6783e5eccacfd104df6abba768c19fae8`
  - `battle/core/boss_runtime.rs`: `2d8e45ca126b2508435bfd8f25e1bc5d815a0467b277564d4723e8ffd84ebe69`
  - `game_resources/data/pve/encounters.ron`:
    `e4bcbdcef0f372fda431ef670f1805b6ef1ba21f757b97673c5ce4380f6d873e`
- Dirty status: five tracked files modified and `boss_runtime.rs` untracked before GEX; no reset/stash/cleanup performed.
- Current code search confirms only `AtBattleStart`/`AtTimeMs` scenario triggers and no generic conditional Wave observer.
- Production mutation: none in P02 at this point.

## 2026-08-16 — GEX-10 implementation and focused rebaseline

- Live schema cutover:
  - `PveWaveData.time_ms` was removed and replaced by required typed `activation`.
  - `required_for_victory` no longer has a serde default.
  - the typed primary set is `AtTimeMs`, `GroupCleared`, actor HP, actual actor death and actor point arrival, with optional
    absolute `deadline_ms` where permitted.
  - `SpawnWave`, enemy group compilation and `ScenarioTrigger` now carry the typed activation without a legacy time
    fallback.
- Publication validation now rejects missing activation/gate fields, empty or duplicate trigger identity, invalid typed
  payloads, missing/self/cyclic source graphs, reversed source/deadline bounds, unknown actor slots/points, generated-Wave
  actor observation, survival required gates and required downstream paths that depend indefinitely on a cancelable source.
- Live migration: all 66 Wave rows in `game_resources/data/pve/encounters.ron` now explicitly author `activation`,
  `priority` and `required_for_victory`; no `time_ms` row remains. The two Clouded Monk survival waves are explicitly
  non-required. Existing two authored non-required reference waves remain non-required.
- Runtime:
  - a `WaveRuntimeState` active-battle ledger owns trigger/request binding, actor binding death epoch, activation cause,
    cancellation and exact-once status;
  - time/deadline wakes enter the normal simulation queue, but evaluation occurs only after the whole timestamp bucket;
  - ready triggers are recomputed one settlement at a time so newly bound already-true triggers participate in global
    priority-descending/trigger-ID-ascending order;
  - primary is tested before equal deadline, all crossed HP thresholds reach a stable state before terminal evaluation,
    and required unspawned groups continue to gate Victory;
  - group clear requires a successfully spawned nonempty group and every member inactive; `Despawned` is distinct from
    `Dead` and does not create a death epoch;
  - actor HP/point are evaluated at first binding, actor death consumes only a post-bind death epoch, and removed actors do
    not contribute historical point state;
  - non-required deadline-free chains cancel stably when their actor/source becomes permanently unavailable.
- Shared Spawn Kernel consumption:
  - Wave uses fixed authored cells with `AllowOccupied([Any])`, one placement commit per member and clone-backed atomic
    rollback;
  - Kernel trigger IDs use the `wave:` namespace, request IDs use `wave_spawn:`, and Wave cancellation is a terminal
    request settlement;
  - legacy scenario spawning remains only for non-Wave player/defense/scripted groups.
- Adapter intermediate: Boss Runtime exposes an authority-free actor-slot view containing slot, scenario unit ref, resolved
  unit ID and binding fact; no Boss phase/task authority moves into Wave.
- Focused commands/results executed after relevant mutations:
  - `cargo test -p game_core --no-run`: all Core unit/integration test targets compiled; 0 compile failures.
  - `cargo test -p game_core game::battle::core::wave_runtime::tests -- --nocapture`: 9 passed before the final ordering
    refinement; later revalidation is required and recorded below when run.
  - `cargo test -p game_core game::data::pve_data::tests -- --nocapture`: 13 passed before the final malformed fixture was
    added; later revalidation is required.
  - P01 Kernel focused suite: 10 passed, 0 failed.
  - battle event ordering suite: 5 passed, 0 failed.
  - Boss setup adapter suite: 3 passed, 0 failed.
  - live RON parse/reference/scenario tests and three typed publication failure tests: 9 passed, 0 failed in total.
  - combat preview→Scenario Wave conversion: 1 passed, 0 failed.
  - TrackAndRejoin raw/live and reconnect/route regressions: 2 passed, 0 failed after correcting the pre-existing survival
    fixture to explicit non-required.
  - `cargo check -p game_server`: passed.
- Full Core/server test suites were not repeated; the inherited 1,152-test full baseline remains reserved for P06 under the
  shared validation contract.

## 2026-08-16 — GEX-10 final current evidence and target identities

- After the final trigger-domain and dynamic-priority corrections:
  - `CARGO_TARGET_DIR=/tmp/cwbs_goal cargo test -p game_core game::battle::core::wave_runtime::tests -- --nocapture`:
    10 passed, 0 failed, 0 ignored.
  - `CARGO_TARGET_DIR=/tmp/cwbs_goal cargo test -p game_core game::data::pve_data::tests -- --nocapture`:
    14 passed, 0 failed, 0 ignored.
  - `CARGO_TARGET_DIR=/tmp/cwbs_goal cargo test -p game_core --test ron_loading live_pve_references_resolve -- --exact
    --nocapture`: 1 passed, 0 failed, 0 ignored.
  - `CARGO_TARGET_DIR=/tmp/cwbs_goal cargo check -p game_server`: passed.
- Other current focused successes not invalidated by the final correction:
  - P01 Spawn Kernel 10/0; BattleEvent ordering 5/0; Boss setup adapter 3/0.
  - live parse/scenario/publication 9/0; preview→Scenario 1/0; TrackAndRejoin raw/live 2/0.
- Frozen target hashes:
  - `data/pve_data.rs` `0c9c6540104927e2f34d14c2371f26b85c3b7880f67fc60416fa91fd12215931`
  - `combat_preview/types.rs` `84c21a423240c5f4a1f2f29130aa4ddaf30713cdceeb3d24846b880a8d4888f8`
  - `combat_preview/mod.rs` `ccad1f6c505940b0dd3855bd364881f448bc0ee5876ab63305f18e4ff166004d`
  - `combat_setup/enemy_spawns.rs` `c70672e4018d22eb2e978a134c7591255d03e88ebff77a2d3ee503792c6478cb`
  - `combat_setup/scenario_groups.rs` `2033834050577ba3f20e4fb2d2fb6c5414ccbba96fc7b5ee8552f719f215cfe7`
  - `combat_setup/boss_runtime.rs` `b1a7b7c17c96191e459a8cd4e703a343388a446ead90ee86d341f17189ae5e1b`
  - `events/combat.rs` `b87bb37ecc87a593187280d893253483b5265635b1df9b320c4550211aa0919d`
  - `battle/scenario.rs` `7c72b27663e49369b7fa4dfd7f0cdb683f587246b1394748ca2d1e64220de1fe`
  - `battle/enums.rs` `900f454199a458c4d787ea56e83f6c1706e55f520614f37326db211389b83154`
  - `battle/core/{mod,build,sim,death_runtime,types}.rs`
    `0240270aab213e35d23e7f0fba6aa76a22fa204fd3b6fc21cede337a980c338c /`
    `9e5536557b4576cd8819106acd3e80b727c8e42e73d3a9225f5036b50f2bf75e /`
    `2ee790263fbe1cce787704243ae079bf14f879cb5943d7de2c8c18a37f014096 /`
    `abdcb6de0b58487011f28df0652a7531fccabec2fa2e6ac833662d30eaf6ce71 /`
    `62c225ff67f74e9789447e375ae5eae4cd368ec618739d86ccf53f5028e88264`
  - `battle/core/spawn_kernel.rs` `209e07ebd96d2239d5914c56bf857a1bf3120599f081790af7dd7c897efca340`
  - `battle/core/wave_runtime.rs` `303a2afc55c3dcfcc663008f917d0e71df028b20beb0f01780ea13c1792714ed`
  - live `game_resources/data/pve/encounters.ron`
    `a3b38fa948208c225d90b197a2f4a449a7800c014d432caa692926d70a4a1e4d`
- Dirty attribution remains additive to the GEX-00 hashes; no unrelated restore/reset/stash/format was performed.
