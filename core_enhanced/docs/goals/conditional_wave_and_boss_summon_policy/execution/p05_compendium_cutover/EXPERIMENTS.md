# P05 Compendium Cutover Experiments

## 2026-08-17 — GEX-00 intake

- Exact selected member: `GS-CWBS-P2G50-002 / GOAL-CWBS-P05-COMPENDIUM-CUTOVER`.
- Phase/node: `PHASE-CWBS-05-COMPENDIUM-CUTOVER / WP-CWBS-10`.
- Direct dependency evidence:
  - P01 `COMPLETION_AUDIT.md / Audit Pass 3 / Complete / High` owns process-local active-session lifetime and
    SavePoint exclusion;
  - P04 `COMPLETION_AUDIT.md / Audit Pass 1 / Complete / High` owns the committed stable outcome/verdict and atomic
    result-consumption seam.
- Initial full baseline is inherited: `game_core 1,080 + game_server 72 = 1,152 / 0 failed`; P05 does not repeat it.
- Current accepted revision is `PER-CWBS-BL-05-CODE-006`, not the stale result-log reconstruction draft.
- Relevant files are pre-existing dirty current actual. Prior Phase and user changes are preserved; no reset, restore,
  stash, cleanup or whole-tree format is authorized.
- Selective source inspection confirms the current delta:
  - P01 already supplies a dormant ordered/idempotent `BattleCore` candidate ledger, but successful spawn recording does
    not populate it and P04 result transition does not carry it;
  - compendium registration is reconstructed from `BattleEventLog` at result consumption and writes discoveries even
    for Defeat;
  - Core profile/snapshot still expose four discovered/defeated sets and two booleans per catalog entry;
  - server profile version 2 still decodes those four sets and contains a v1 legacy-file union migration.
- Targeted dirty gate found no unsafe overlap: all targets are already part of the accepted current dirty implementation;
  the deleted predecessor store has been replaced by the current untracked consolidated `player_profile_store.rs` and
  will not be restored.
- Current pre-mutation hashes:
  - `encounter_compendium.rs` `43c8361ea34caa74808af830e20cfc108c71a71add5c56f0562c6f7d44624eef`
  - `battle/core/spawn_kernel.rs` `84845f55cb80722683369941b5f18bf8bcb209119cb4a2436bb7eec260a1da84`
  - `battle/core/sim.rs` `df679fcaf2e1c192f7120b2ccc26cbf0d160c16f91b1b66d1bae534c953befb5`
  - `resources/selection.rs` `a61fa4d34edb2dbb49f4b9ea0dfe235fffeddc1aed0a3978eb4287f435b7035b`
  - `world/combat.rs` `97897daa4c6f4012db5e13ff231a2ce788d56ff2481181333e8b86dfc01044fa`
  - `behavior.rs` `b2c9c458caf577d667e460f8c8fbe2e962ecbfe8f7ae623983231fc9b711fd93`
  - `world/snapshot.rs` `9fac7f82576687156c668405df962473a8d61f99c83a1b41485b2eb7c1cb607b`
  - fixture generator `093af016a6fb00c467d422e276861d89d8b0f21ba9ce844f05197a406a18129d`
  - server `player_profile_store.rs` `5be842318715366f41c8b91de1c8447c4c6de06210850a8062ea390c18be188c`
  - server actor handlers `7a1d1bf268b09ef40d62eb40c3b95197dcae0ec958d9298b896d9a0f535a039c`
- Production mutation: none in P05 at this point.

## 2026-08-17 — GEX-10 implementation and focused validation

### Implemented cutover

- Connected the P01 dormant ordered/idempotent ledger to the shared successful `UnitSpawned` recording seam. Only
  opponent `Abnormality` and `CorrodedEmployee` source identities produce candidates; player units, facility/defense
  objects, test fixtures and `SummonedUnit` do not. Empty/rejected/canceled spawn paths never reach this commit seam.
- Candidate order uses the authoritative spawn event sequence. The ledger remains inside cloneable `BattleCore`, so
  death/despawn do not erase it and P01 same-process detach/attach retains it with the active battle; process death and
  Retreat discard that process-local state.
- Carried the frozen candidate vector into the process-local `CombatBattleState` when P04 finalizes a named outcome.
  It is not serialized into `RunCheckpointPayload` or the active-run checkpoint.
- Removed event-log/primary/death reconstruction and every Defeat-side compendium writer. Only a committed Victory forms
  one registration set from all candidates and merges it as the final infallible step of the existing transactional
  result-completion branches. Duplicate completion has no remaining result to consume.
- Replaced Core persisted compendium state with exactly `registered_ids: BTreeSet<String>`. Snapshot entries now expose
  one `registered` boolean, and metadata remains hidden until registration. Old four-set fields are strict unknowns.
- Replaced server profile envelope version 2 with version 3. Removed all legacy envelope readers and v1 migration/union
  code. Retired files are ignored during load and removed only by explicit full-profile reset; version 2 is rejected.
  Missing/new/reset profiles use the empty version-3 schema.
- Updated the contract fixture generator and focused Core/server test fixtures directly; no compatibility alias, dual
  writer, outcome credit field or production profile mutation was added.

### Corrected implementation findings

- The first server test-target compile hit a Rust 1.91.1 incremental metadata ICE while forcing an obsolete Actix trait
  query after the Core DTO changed. No source diagnostic was emitted. Only the task-owned
  `/tmp/cwbs_goal/debug/incremental` cache was removed; the identical command then compiled successfully.
- Test-target compilation found four `CombatBattleState` literals whose employee roster was held in a local variable,
  so the initial mechanical fixture insertion did not match them. They were updated explicitly; production compilation
  had already passed.
- No policy question, required legacy consumer or unsafe dirty overlap was found.

### Focused validation evidence

- `CARGO_TARGET_DIR=/tmp/cwbs_goal cargo check -p game_core -p game_server`: final pass, zero warnings.
- `cargo test -p game_core --no-run`: pass; lib and five integration targets build.
- `cargo test -p game_server --no-run`: pass after the task-local incremental-cache recovery; lib/main targets build.
- Core replacement schema/profile group (`encounter_compendium` filter): `6 passed / 0 failed`.
- successful actual opponent spawn→process-local ledger: `1 / 0`.
- ordered/idempotent candidate ledger: `1 / 0`.
- committed Victory all-candidate registration across SavePoint load/new run: `1 / 0`.
- retryable/final Defeat and Retreat permanent-set invariance: `1 / 0`.
- result-completion failure leaves rewards and registrations unmodified across retry: `1 / 0`.
- server profile store round-trip/backup/reset/v3/no-migration group: `11 / 0`.
- server compendium bootstrap/reset/attach/persistence-failure group: `6 / 0`.
- local-install profile shared across transport players: `1 / 0`.
- detached live battle no-TTL/reconnect-Paused lifecycle: `1 / 0`; combined with the Core active-session clone ledger
  assertion this covers same-process candidate retention.
- fixture generator: pass; emitted only `registered` booleans and no discovered/defeated fields.
- scoped retired-symbol audit: only the two deliberate old-schema rejection fixtures remain.
- scoped `git diff --check`: pass.
- The inherited P01 full baseline remains `game_core 1,080 + game_server 72 = 1,152 / 0 failed`; it was not repeated in
  P05 and final current counts remain reserved for P06.

### Current GEX-10 target identities

- `encounter_compendium.rs` `31831183a51be2d46b42716fa252727d127bf2854cbc95e1f4d12915cbf0b6c2`
- `battle/core/spawn_kernel.rs` `84845f55cb80722683369941b5f18bf8bcb209119cb4a2436bb7eec260a1da84`
- `battle/core/sim.rs` `0d3fb7af215d12ab34612e4e15f3d33faa03627b4101233e05f41153775b23ed`
- `battle/core/wave_runtime.rs` `23eadbd1192cfe28e2e94f3d555b1ebf283ec6d319e9cec5e2120a654a1bde93`
- `battle/core/build.rs` `cf8130d1bf10ccb996b1aab0354b606d10232371745e987dbfa55738d3178470`
- `battle/core/summon_runtime.rs` `9fc236ce1032422b89ac3b965be6ef25b465dad114c73498d189fa5301ecc095`
- `resources/selection.rs` `a12169d80849e5489824fb2c75cb837cb1c7372aafbbdff8c09ccd2ae6f3549d`
- `world/combat.rs` `aac20cc5e76a716c887c95261b7a482b5b57ce4d3cc51d0adcca86d3f4006382`
- `behavior.rs` `66f037715fd574277b04e8842f33cbb89d9ce3cd62f1c34be59f1d9915c2bd40`
- `world/snapshot.rs` `4077b23fa7b99837f23a099bac3993b498ca442b8522d935186601cbedbc6d82`
- fixture generator `5b30079d446c3c77857203ff780250b8da0271f1fb1a16eaef298c860ac54e58`
- world compendium tests `7e8812cf5da5642fb8d2789c5f7eafbf08c941cd6eca7e58af89460ec18ecf01`
- world node-session tests `7b0d9bc3718a6f1b314e9233ebad6b76e4efba22420f7df0ec34092ecee31feb`
- `world/active_run.rs` `63c0ccaa79a61a600a5161b122f76e9ad75c55f86761a3a8220488650662421a`
- `world/node_flow.rs` `605e456d06311adc0d835d713c8d8842eb1aad6205d21392686bf1047b682894`
- server profile store `d093656e63e4c9442a5729dcfc6638bdf40d145ddd8e13e42eb97e0dc9f464de`
- server player actor handlers `ed58856e81ab09863f95e245d05121bdf75de4eef0323bd81ae70deaa8351ddd`
- server load-balance handlers `e80e1da601155e1e1a850839f8077b7e0478162c143100324e435ff6a596ee5f`
