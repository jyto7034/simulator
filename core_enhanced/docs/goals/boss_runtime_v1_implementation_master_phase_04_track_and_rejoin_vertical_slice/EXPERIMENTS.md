# Boss Runtime v1 Phase 04 — Experiments

## 2026-08-09 — activation baseline

- P03 independently completed `Complete / High` with `ready_for_p04`; P04 is the next required implementation Phase.
- The sorted repository status before P04 child creation contains 256 entries with SHA-256
  `e0e7f5cbf7d39f41a2ad8e2a97c4070f3a726d268a87242df096c373c5ce30fc`.
- Child path did not exist, so no prior P04 document or implementation history was overwritten.
- Repository HEAD is `f318414fb0156a67e04c5479000c2a2683fc0c09` and the current dirty tree is the user-owned baseline.
- No P04 production source was inspected or changed and no test/formatter was run before these required documents
  were created. Exact owner hashes, runtime/server path and runner targets are discovery pending.

## 2026-08-09 — postcreation status and owner baseline

- After the three required child documents were created, sorted status contains `259` entries with SHA-256
  `a8eafc1a25cb1c5721ace69cd10cbcd98fd6c6156c18355f9c56ddd5ae89879f`.
- Potential owner hashes before P04 production mutation:
  - `game/events/combat.rs`: `208682013a3f0042c5a79bb5843469d6705a8705d26d8ef8ad524bb0362815df`
  - `game/world/combat.rs`: `ce43e9c9c27285e9665ca89127f3a14c9982a2c16a9cc895d39fba53602e26d0`
  - `game/world/state.rs`: `74c3dabb70b44b8d6192e663e942663584afad9484a971351d74ed5573f58811`
  - `game/world/tests/mod.rs`: `412ad89eb50dbafb4ab57c6f5bdf4d5f88eb621195f70c39523d2b681458c44a`
  - `game/world/tests/combat.rs`: `088e7d72904e83db2b98c5c367036de9a9b8d679098046e4e0471b5dcd23cdbe`
  - `battle/core/boss_runtime.rs`: `8aa633c1a158a883115e96533394ae6d1db0bda9398ca08d3952789db76a0686`
  - `data/boss_runtime_data.rs`: `a5fa4f770365a8a5e2db5188bb1c5f7627fcca4fa844a89d12ebade9bf75f306`
  - server `messages.rs`: `dee1e0484bad0d4f1cc13c7f34162df93972bec90425f395223a8ff1a05c3480`
  - server `state.rs`: `2f9263f76150354cb4fa7cd1522971344e505b2a2ea532b59853ec558e811af0`
  - server `handlers.rs`: `d45515c4b131c73bf559d5ac24ddda31b2be765d77030e81839fb46802325dc6`
- All listed world/server/setup owners were already modified in the user baseline; the two Boss modules are untracked
  outputs of P02/P03. P04 must use symbol/hunk ownership and preserve these exact starting hashes for untouched owners
  rather than treating status shape as attribution.
- Existing diffs are large (`world/tests/combat.rs` alone is +3248/-469), so the preferred implementation is new
  test-only symbols in the narrowest current owner. No production mutation has occurred at this checkpoint.

## 2026-08-09 — pre-mutation path and runner baseline

- Runtime trace: `ConfirmEnterNode` builds one private `ResolvedCombatSetupPlan`, which resolves optional Boss setup,
  constructs `BattleCore` and initial execution before world commit. World deployment and elapsed/server-tick advance
  mutate the active Core transactionally; `active_battle_reconnect_projection` only reprojects the same active
  session. `PlayerGameServerMessage::battle_update/battle_resync` transports the produced Core DTO without Boss
  inference.
- The narrow fixture owner is `game/world/tests/combat.rs`: it can mutate test-only GameData, traverse actual
  select/confirm/deploy/advance entry points, inspect Core units and full test transaction snapshots, and call the
  real reconnect projection. A private Boss-helper-only test would not satisfy P04.
- Exact chained baseline commands and results:
  - bound authored setup identity: `1 passed`, `0 failed`, `0 ignored`, `915 filtered out`;
  - live-defense setup/state/repeated-cursor projection: `1/0/0/915`;
  - late Boss setup rollback: `1/0/0/915`;
  - game_server top-level battle update serialization: library `1/0/0/55`, binary `0`;
  - game_server nested resync serialization: library `1/0/0/55`, binary `0`.
- All five filters matched nonzero tests and passed on the current dirty pre-P04-production tree. These are regression
  anchors; the new vertical fixture and post-change focused/broad gates remain required.

## 2026-08-09 — first success-slice compile checkpoint

- Exact command: `cargo fmt --all && cargo test -p game_core --lib game::world::tests::combat::track_and_rejoin_fixture_runs_from_world_setup_through_reconnect_and_route_combat -- --exact`.
- No test ran: the new assertion used the non-existent `Side::Enemy` spelling; current Core authority names the hostile
  side `Side::Opponent`. This is a test compile defect, not runtime evidence.
- The fixture assertion is corrected to the authoritative enum spelling and the same exact test is rerun before any
  result is counted.

## 2026-08-09 — first success and paired-ordinary result

- The corrected exact success test passed: `1 passed`, `0 failed`, `0 ignored`, `916 filtered out`.
- Combined command after adding the paired fixture:
  `cargo fmt --all && cargo test -p game_core --lib game::world::tests::combat::track_and_rejoin_`.
- Result: `1 passed`, `1 failed`, `0 ignored`, `916 filtered out`. The bound success slice passed again. The paired
  ordinary fixture failed only because it required `MovementSegmentStarted` in the actor's exact spawn bucket; the
  ordinary planner legitimately admits its route movement on the following movement tick.
- This is an over-specific fixture timing expectation, not a production failure or Boss/non-Boss authority mismatch.
  The paired test now advances one ordinary 100 ms planner interval before requiring the existing route movement
  event, then the same two-test filter is rerun.

## 2026-08-09 — corrected success and paired-ordinary checkpoint

- Exact command: `cargo fmt --all && cargo test -p game_core --lib game::world::tests::combat::track_and_rejoin_`.
- Result: `2 passed`, `0 failed`, `0 ignored`, `916 filtered out`.
- The bound fixture traverses real select/confirm setup, t0 player deployment and time-10 exact actor spawn. It proves
  the actor binds only at its authored wave, locks the deployed runtime identity, enters approach then rejoin, keeps
  that identity throughout the Pattern, uses at least two existing movement segments, advances only a monotonic route
  cursor and ends in resident `RouteCombat`.
- Reconnect projection before spawn and after Pattern completion leaves the complete world/Core battle snapshot
  unchanged. Setup, initial update and reconnect JSON contain no `boss_runtime` public field.
- The paired fixture uses the same encounter/wave geometry with `boss_runtime == None`, constructs no Boss actor
  runtime and resumes the pre-existing authored-route planner on its normal following tick. Its reconnect is also a
  read-only projection with the same public shape.

## 2026-08-09 — first no-target/timeout/Pause matrix result

- Exact command: `cargo fmt --all && cargo test -p game_core --lib game::world::tests::combat::track_and_rejoin_`.
- Result: `3 passed`, `1 failed`, `0 ignored`, `916 filtered out`.
- Success, paired ordinary and the new zero-speed timeout/Pause fixture passed. The timeout fixture proves a paused
  50,000 ms server tick leaves the complete snapshot unchanged, approach stays active through battle time 5,009 and
  transitions exactly at 5,010, then rejoin likewise transitions to RouteCombat only at its exact second deadline.
- The intended no-target fixture instead remained in `pattern_call`: the default Defense mission spawns a protected
  Player-owned objective, which is a legitimate hostile query candidate even without deployed employees. The test's
  assumption—not runtime behavior—was wrong.
- To isolate true no-target through the same setup path, that fixture now authors the supported
  `AllRequiredEnemyGroupsDefeated` win condition so no protected Player unit is spawned. Production code is unchanged;
  the exact no-target test is rerun before the matrix is counted as closed.

## 2026-08-09 — no-target fixture setup correction

- Exact no-target rerun executed `0 passed`, `1 failed`, `0 ignored`, `919 filtered out`.
- Changing only the win condition removed the protected-unit spawn, but the default Defense tactical objective still
  referenced `black_box_device`; strict scenario validation correctly rejected that now-absent unit before battle
  start. No runtime assertion ran.
- The fixture now explicitly authors the supported `DefeatBoss { unit_ref: "enemy_wave_0_0" }` tactical objective
  together with `AllRequiredEnemyGroupsDefeated`. This preserves the required authored route while eliminating every
  Player-owned scenario unit, and the exact test is rerun again. Production setup validation is unchanged.

## 2026-08-09 — no-target fixture terminal correction

- The second exact no-target rerun executed `0 passed`, `1 failed`, `0 ignored`, `919 filtered out` with
  `InvalidAction` during `ConfirmEnterNode`.
- With no Player-side unit at all, Core terminal authority correctly ends/rejects the battle before the intended Boss
  no-target observation. Removing every player is therefore not a valid live vertical fixture.
- The fixture is changed to keep the normal protected Player unit alive, add the supported authored `Grounded` query
  filter, and mark current Player fixture units Airborne before the time-10 actor spawn. This produces a genuine
  non-terminal `NoEligibleTarget` through strict query semantics without changing production runtime or scenario
  contracts. The abandoned win/tactical overrides are removed.

## 2026-08-09 — corrected no-target focused result

- Exact command: `cargo fmt --all && cargo test -p game_core --lib game::world::tests::combat::track_and_rejoin_no_target_uses_authored_route_without_public_boss_state -- --exact`.
- Result: `1 passed`, `0 failed`, `0 ignored`, `919 filtered out`.
- The strict fixture now reaches `NoEligibleTarget`, consumes the authored `target_missing` outcome and resides in
  `RouteCombat` with no Pattern or target lock. The battle remains active, its public update contains no Boss field,
  and reconnect leaves the full battle snapshot unchanged.

## 2026-08-09 — invalidation and path-unavailable expansion result

- Exact command: `cargo fmt --all && cargo test -p game_core --lib game::world::tests::combat::track_and_rejoin_`.
- Result: `5 passed`, `1 failed`, `0 ignored`, `916 filtered out`.
- The public withdrawal case passed: one `UnitWithdrawn` invalidates the exact locked runtime identity, no subsequent
  Pattern boundary reacquires it, rejoin reaches RouteCombat and a further boundary does not recreate the resident
  State activation.
- The off-grid active-target failure injection correctly left `approach` at the first ordinary movement boundary, but
  the test over-assumed that forward rejoin also completed in that same 100 ms. Debug evidence shows the actor remains
  under `pattern_call`, consistent with an admitted rejoin movement leaf.
- The corrected assertion now requires the distinct `rejoin` State immediately after PathUnavailable, preserves the
  still-active locked identity until Pattern exit, then advances existing movement until RouteCombat and requires the
  lock to clear. No production behavior is changed.

## 2026-08-09 — corrected path-unavailable focused result

- Exact command: `cargo fmt --all && cargo test -p game_core --lib game::world::tests::combat::track_and_rejoin_active_locked_target_with_unusable_path_takes_authored_recovery -- --exact`.
- Result: `1 passed`, `0 failed`, `0 ignored`, `921 filtered out`.
- The active locked target remains lifecycle-valid but is moved outside the static grid. Before either deadline, the
  world boundary therefore produces only PathUnavailable, enters authored `rejoin`, preserves identity through that
  Pattern State, completes via existing forward movement, clears the lock on Pattern exit and reaches RouteCombat.
  Reconnect after recovery is read-only.

## 2026-08-09 — actor lifecycle-interrupt focused result

- Exact command: `cargo fmt --all && cargo test -p game_core --lib game::world::tests::combat::track_and_rejoin_actor_death_terminates_once_before_stale_runtime_work -- --exact`.
- Result: `1 passed`, `0 failed`, `0 ignored`, `922 filtered out`.
- Standard world-test failure injection marks the required bound actor dead during `approach`. A zero-time
  authoritative advance applies terminal priority before queued movement/wake work, returns one finished battle
  update, removes the active session and enters `CombatResult`; a second advance is rejected as `InvalidAction`.
- The public terminal update contains no Boss runtime field. Existing P03 focused cancellation remains the direct
  proof that unreleased work is cancelled while released delivery survives; this P04 test proves that lifecycle and
  world terminal integration do not double-complete or reenter stale Boss work.

## 2026-08-09 — complete P04 focused matrix and transport checkpoint

- Exact chained command applied `cargo fmt --all`, verified `cargo fmt --all -- --check`, then ran the P04 world
  filter, P03 Boss module and two exact server serialization tests.
- P04 world vertical matrix: `7 passed`, `0 failed`, `0 ignored`, `916 filtered out`.
- P03 Boss runtime regression: `22 passed`, `0 failed`, `0 ignored`, `901 filtered out`.
- game_server top-level update: library `1 passed`, `0 failed`, `0 ignored`, `55 filtered out`; binary `0`.
- game_server nested resync: library `1/0/0/55`; binary `0`.
- The success test now also requests reconnect during active `approach` using the exact current sequence cursor. It
  receives an empty delta while the typed actor snapshot and complete authoritative battle snapshot remain equal,
  directly covering no target/selector/activation/cursor reroll before continuation.
- The actual Boss-driven Core update uses only the existing `LiveBattleUpdateDto`; the unchanged server mapper
  destructures every current field into update/resync envelopes, and both exact wire shapes remain green. No server
  or public DTO file was modified.

## 2026-08-09 — post-implementation setup/reconnect regression anchors

- Exact chained command mechanically formatted, passed formatting and `git diff --check`, then ran P04 plus the three
  original Core anchors.
- P04 matrix remained `7 passed`, `0 failed`, `0 ignored`, `916 filtered out` after reducing the typed test snapshot to
  only fields used by the vertical contract.
- Bound P02 setup identity: `1 passed`, `0 failed`, `0 ignored`, `922 filtered out`.
- Existing live-defense setup/state/repeated-cursor projection: `1/0/0/922`.
- Late invalid Boss setup full world rollback: `1/0/0/922`.
- These post-P04 results distinguish the new vertical behavior from regressions in the original setup, ordinary
  reconnect and failure-atomicity anchors.

## 2026-08-09 — full Core broad gate

- Exact current-tree command: `cargo test -p game_core --lib -- --quiet`.
- Result: `923 passed`, `0 failed`, `0 ignored`, `0 measured`, `0 filtered out`.
- This combines P01 atomic lifecycle/order, P02 strict authoring/setup, P03 runtime 22, P04 world vertical 7 and all
  existing world/Pause/reconnect/combat/data tests in one Core binary.

## 2026-08-09 — server broad gate

- Exact current-tree command: `cargo test -p game_server -- --quiet`.
- Result: library `56 passed`, `0 failed`, `0 ignored`, `0 measured`, `0 filtered out`; binary and doc tests each ran
  `0` tests successfully.
- No server source changed. Existing update/setup/resync mapping and transport tests remain sufficient for the
  unchanged public DTO produced by the actual P04 Core slice.

## 2026-08-09 — workspace all-targets broad gate

- Exact current-tree command: `cargo test --workspace --all-targets -- --quiet`.
- Exit: `0`.
- Results: Core library `923 passed`; integration targets `3 + 4 + 24 + 13 + 4`; server library `56`; Core/server
  binaries, five Core fixture generators and three server examples `0`. Every listed target reports zero failures,
  ignored tests or measured benchmarks.
- Together with the separately recorded formatting/diff, focused P04/P03, setup/reconnect and server gates, this is
  the final broad validation stream for the current P04 implementation candidate.

## 2026-08-09 — pre-exit atomic-source evidence gap

- A pre-exit reread found that the seven P04 world tests compile strict `BossRuntimeCatalogData` and traverse the real
  world setup/runtime, while P02 separately proves raw live-RON parsing and whole-candidate rejection. That is valid
  cross-Phase composition, but it does not by itself prove P04-C01's stronger wording with one raw source candidate
  reaching typed world setup identity.
- The production loader currently inlines parse, whole-domain validation, Boss compilation and `GameDataBase`
  construction inside `GameDataBase::load_live_embedded`; world tests cannot inject a modified source set without
  bypassing that exact construction path.
- The narrow correction is to extract that existing body into one crate-private source-set loader used unchanged by
  `load_live_embedded`, then add a raw-RON fixture proof. This is a P04 integration seam, not a public API, live-data
  publication, fixture-only production branch or independent CRA refactor.
- Hashes immediately before this correction: `game/data/mod.rs`
  `c9272935a453e5bccbe322a757a42dd440dd7097c18af888d1ca3fa28879b77e`; already modified P04 test owner
  `game/world/tests/combat.rs` `325bff60bf450b60e3ab7548d383fdc1320141d0660f1651720e42438bfa9599`.

## 2026-08-09 — raw atomic-source to typed world identity proof

- `GameDataBase::load_live_embedded` now delegates its unchanged body to one crate-private
  `GameDataBase::load_live_sources`. The extracted function accepts the already-existing typed
  `GameplayRonSourceSet`; it adds no public API, alternate validation, fallback or test-only production branch.
- The new test serializes the same strict `valid_catalog`, injects it with an exact actor binding into cloned embedded
  PvE RON, and submits both through the complete source-set loader. It then uses normal new-run, map select/confirm,
  deployment and battle-time advance entry points.
- Exact command:
  `cargo fmt --all && cargo test -p game_core --lib game::world::tests::combat::track_and_rejoin_raw_live_bundle_reaches_typed_world_setup_identity -- --exact`.
- Result: `1 passed`, `0 failed`, `0 ignored`, `923 filtered out`.
- The raw candidate publishes atomically, resolves the authored late wave to the declared `primary` runtime slot,
  enters `track_and_rejoin/approach`, locks the deployed Player runtime identity and emits the ordinary hostile spawn.
  Setup/start and spawned update serialization still contain no `boss_runtime` public field. P04-C01 no longer relies
  only on cross-Phase composition.

## 2026-08-09 — post-correction P04 matrix

- Exact chained command: `cargo fmt --all -- --check && git diff --check && cargo test -p game_core --lib
  game::world::tests::combat::track_and_rejoin_`.
- Formatting and diff checks passed. The expanded P04 world matrix reports `8 passed`, `0 failed`, `0 ignored`,
  `916 filtered out`.
- The original seven success/parity/failure/Pause/reconnect/lifecycle cases remain green beside the new raw-source
  proof; the loader extraction did not create a behavioral fork between embedded production loading and fixture
  loading.

## 2026-08-09 — pre-exit scope, public-surface and dirty-tree audit

- Production `game_resources/data/boss_runtime/base.ron` remains exactly the valid empty catalog: empty `patterns`,
  `actor_behaviors` and `encounters`. No official Boss identity, balance or asset selection has entered P04.
- No `boss_runtime`/`BossRuntime` symbol occurs in the three game_server player actor transport owners. Their current
  hashes exactly reproduce the P04 baseline: messages `dee1e048...`, state `2f9263f7...`, handlers `d45515c4...`.
- Existing Core event/world owners also exactly reproduce their P04 baseline hashes: events `20868201...`, world
  combat `ce43e9c9...`, world state `74c3dabb...`, world test module `412ad89e...`, Boss data
  `a5fa4f77...`. P04 changed only the scoped crate-private loader extraction, a `#[cfg(test)]` typed runtime snapshot
  seam and world fixture/tests.
- `BossActorRuntimeDebugSnapshot` and its accessor occur only behind `#[cfg(test)]`; every call site is in world
  tests. They cannot enter a production Core/server build or public DTO.
- The closest canonical already states the P02/P03 current contract, including private optional setup, compiled-frame
  ownership, Track/Rejoin semantics and no public Boss graph field. P04 verifies that contract without changing it,
  so no canonical mutation is required.
- This workspace contains no Unity project/`Assets/*.cs` source. The previously classified Unity consumer therefore
  remains stale, external and unavailable for execution; because Wire is unchanged, this is an explicit external
  consumer non-mutation, not producer evidence or a completion gap.
- Sorted status remains `259` entries with hash `a8eafc1...`. Excluding exactly the three P04 child documents restores
  the `256`-entry precreation baseline hash `e0e7f5cb...`; unrelated user-owned dirty work has neither been normalized
  nor added to the Phase attribution.

## 2026-08-09 — post-correction production compile and full Core gate

- Exact command: `cargo check -p game_core --lib && cargo test -p game_core --lib -- --quiet`.
- Production Core compilation passed, proving the test-only snapshot seam is absent without breaking the extracted
  crate-private live loader.
- Full Core result: `924 passed`, `0 failed`, `0 ignored`, `0 measured`, `0 filtered out`. The count increased by
  exactly the new raw-source vertical proof from the earlier 923-test gate.

## 2026-08-09 — final post-correction server and workspace gates

- Exact command: `cargo test -p game_server -- --quiet && cargo test --workspace --all-targets -- --quiet`.
- Server result: library `56 passed`, zero failures/ignored/measured/filtered; binary and doc targets each completed
  their zero-test runners successfully.
- Workspace exit: `0`. Results are Core `924`, integration targets `3 + 4 + 24 + 13 + 4`, server `56`, plus the same
  zero-test Core/server binaries, five fixture generators and three server examples. Every executed target reports
  zero failures, ignored tests or measured benchmarks.
- These are fresh results after the loader extraction and raw-source test, superseding the earlier 923-count broad
  evidence for final P04 completion review.
