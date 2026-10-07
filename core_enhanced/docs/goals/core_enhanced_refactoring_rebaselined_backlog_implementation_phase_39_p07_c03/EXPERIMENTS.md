# Phase 39 CRA-P07-C03 Experiments

Status: complete
Implementation Complete: true

## 2026-08-11 — E00 Goal Initialization

### Hypothesis

The production-default Rapier backend can become the sole movement authority only after the complete Direct-
dependent assertion surface, current production/Boss consumers and public compatibility gate are made explicit.
Initializing the required Child records before current Phase-39 source inspection preserves the Master one-active-
Phase and dirty-worktree contracts.

### Performed checks

- Read the complete 874-line Goal authoring/execution guide.
- Verified Master Phase 38 is `complete — Complete / High`, Phase 39 is `ready`, Phases 40~44 are pending, and no
  implementation Child is active.
- Read the original P07-C03 candidate, Boss deferral registry, post-Boss P07 reaudit, final catalog/order/test/wave/
  policy evidence and current Boss movement-leaf disposition.
- Read complete Phase 08 P07-C01, Phase 09 P07-C02, Phase 38 linear and Boss P03/Master completion audits relevant
  to the movement/tactical/Boss boundary.
- Verified reserved Phase-39 and Phase-40 Child paths were absent and recorded HEAD, status-set identity and marker.
- No current Phase-39 production content, caller, test, manifest, data, canonical or external-consumer detail was
  inspected.

### Exact commands

```text
wc -l and bounded complete reads of goal_authoring_and_execution_guide.md
rg/sed <candidate-scoped original, Boss, post-Boss and final evidence>
sed -n <complete Phase 08/09/38 and Boss P03/Master completion audits>
git rev-parse HEAD
find <Phase-39 and Phase-40 reserved paths>
rg -l '^Status: active$' <implementation phase PLAN glob>
git status --porcelain=v1 --untracked-files=all | wc -l
git status --porcelain=v1 --untracked-files=all | LC_ALL=C sort | sha256sum
stat/sha256sum game_server/logs/app.log.2026-08-09
```

### Actual result

- Repository/HEAD: `/mnt/f/work/simulator` /
  `f318414fb0156a67e04c5479000c2a2683fc0c09`.
- Direct linear gate: Phase 38 audit is 212 lines at `b1063d8b...`, verdict `Complete / High`.
- Technical gates: Phase 08/09 are `Complete / High`; Boss P03/Master are `Complete / High` and preserve the ordinary
  movement leaf while leaving P07-C03 separately valid.
- Candidate: CRA-P07-C03, cleanup, Tier D, `3/1/1/3/3/2/unknown = 13`.
- First proof: inventory 17 Direct-dependent tests and move every unique collision/clamp/obstacle/determinism/route
  assertion to retained Rapier/default production behavior before removal.
- Public gate: CRA-POL-006 requires current symbol/tool/deployment inventory plus migration or candidate-scoped
  breaking approval for public narrowing; prior cleanup approvals are not generalized.
- Pre-Child status: 573 rows at canonical C-sort SHA-256
  `a086d6ad75aae503d2da3649bc84bf8d38605e21b0466c9b5f15ed1c6251b149`.
- Active implementation Child count: 0. Phase 39/40 paths: absent.
- Protected marker remains 57 bytes at SHA-256 `614d2fbd...` with exact timestamps.
- Production/data/test/canonical mutation and test execution: none.

### Decision

Initialize only `PLAN.md`, `EXPERIMENTS.md` and `EXPERIMENT_NOTES.md`. Register and activate Phase 39 before any
current movement source/test/consumer inspection. Phase 40 remains pending and absent.

## 2026-08-11 — E01 Sole Activation

### Gate check

- The initialized Child contains exactly the required three documents.
- Adding them changes expanded status exactly from 573 to 576 rows at canonical C-sort SHA-256
  `93625e86d7ea3bce8539875054846dc9d48b1aea3e82526d6c060bfc1326376e`.
- Phase 38 is complete, Phase 39 dependencies are satisfied, Phase 40 remains pending/absent and there was no active
  implementation Child before transition.
- The protected marker remains exact at SHA-256 `614d2fbd...`.

### Result

- All three Child headers changed from `initialized` to `active`; `Implementation Complete` remains `false`.
- Master Phase 39 changed from `ready` to `active`; it is the sole active Child.
- Phase 40 and later remain pending and unauthorized for production inspection.
- Current Phase-39 production content/callers/tests/consumers, mutation and test execution: none.

### Next gate

Fully reread all three active Child documents and direct dependency/final records, then record filename-only
prospective fingerprints before opening current production content.

## 2026-08-11 — E02 Active Reread And Prospective Fingerprints

### Procedure

- Fully reread all three active Child documents.
- Reread the complete Phase-08/09/38 and Boss P03/Master dependency audits after activation, plus the candidate-
  scoped original/post-Boss and final catalog/order/test/gate/policy records.
- Used filename-only discovery to locate prospective movement, Boss/tactical/world/range, manifest, live-data and
  canonical paths without opening current production content.
- Recorded scoped status, line counts and SHA-256 fingerprints before Phase-39 current discovery.

### Activation state

- Expanded status remains 576 rows at canonical C-sort SHA-256
  `93625e86d7ea3bce8539875054846dc9d48b1aea3e82526d6c060bfc1326376e`.
- Prospective movement module paths and activation hashes/line counts are:
  - `blocking.rs` dirty, 379, `10323071...`;
  - `engine.rs` dirty, 1,364, `8e3dd851...`;
  - `lifecycle.rs` clean, 55, `46b54329...`;
  - `mod.rs` clean, 25, `e0a66ed9...`;
  - `path.rs` clean, 125, `c75566b9...`;
  - `planner.rs` dirty, 984, `cb4cbec3...`;
  - `rapier_backend.rs` dirty, 986, `5a488435...`;
  - `steering.rs` clean, 69, `3d88b375...`;
  - `types.rs` clean, 311, `2695e40e...`.
- Prospective consumers are already dirty/untracked at `boss_runtime.rs` `a4b99993...`, `core/mod.rs`
  `822bf01e...`, `sim.rs` `9d4eecc6...`, world tests `c8d52c36...`/`08177bd5...` and range preview
  `1a8fce57...`; battlefield field is clean at `66859782...`.
- Root/Core/server manifests, lockfile and Core lib root remain clean at `98c9cf4d...`, `40d8a9ec...`,
  `19b41cd...`, `4223318b...` and `3fbfdca2...`.
- Live Boss RON is already untracked at `f696c9d0...`; gameplay/runtime/data canonicals are already dirty at
  `6c53ecd5...`, `ab7d2f98...` and `30ed5fd5...`.
- Protected marker remains exact at `614d2fbd...`.

### Decision

The likely backend/assertion carriers overlap earlier campaign work. Any Phase-39 test transfer or cleanup must be
isolated by exact activation hashes and scoped hunks; clean movement files must remain untouched unless current
evidence proves a direct candidate need. Current content/caller/test/consumer discovery may now begin.

## 2026-08-11 — E03 Current Graph, 17-Assertion Inventory And Pre-Transfer Baseline

### Current graph

- `BattleCore::new_from_scenario` initializes `ContinuousMovementBackend::default()`, whose only production default
  is `Rapier(RapierMovementWorld::default())`. Production `sim.rs` invokes the ordinary continuous-movement leaf and
  resets that stored backend; no production caller selects Direct.
- The alternate `DirectContinuousMovement` implementation is selected only by `#[cfg(test)]`
  `BattleCore::use_direct_continuous_movement_backend` and tests. The public Rapier selector has one in-tree test
  caller and no production caller. The authored-route test was the sole Direct selector outside `engine.rs`.
- Exact Direct-dependent inventory is 17: four backend-parity tests, nine Direct-only engine tests, three Core tests
  that select Direct, and one authored-route integration test that selects Direct.
- The shared tick loop currently calls `DirectContinuousMovement::clamp_to_board`, so Direct deletion requires a
  neutral private clamp extraction even though the clamp behavior belongs to both paths.
- `rapier_backend.rs` contains 15 retained tests. Its board-wall storage/sync/remove path is test-created only; the
  production correction path explicitly ignores those wall colliders and otherwise uses the shared manual clamp.
- `game_core::game::battle::core::movement::{engine,rapier_backend}` are public through the crate's public module
  chain. The backend enum, Direct type, Rapier world/handle/report types, Rapier selector and inspection/sync methods
  are therefore externally reachable Rust surface.
- Complete in-repository source/docs search found no non-test consumer of those backend-selection/inspection
  symbols. `/mnt/f/work` outside this repository contains no Rust reference. `game_server` is the only path
  dependency on `game_core` and has no symbol consumer. However, `core_enhanced/Cargo.toml` does not set
  `publish = false`, and no deployed/standalone consumer registry proves external absence; CRA-POL-006 therefore
  does not authorize public narrowing from these negative searches alone.

### Exact baseline commands and results

```text
cargo test -p game_core --lib game::battle::core::movement::engine::tests -- --nocapture
  19 passed; 0 failed; 0 ignored; 997 filtered out

cargo test -p game_core --lib game::battle::core::movement::rapier_backend::tests -- --nocapture
  15 passed; 0 failed; 0 ignored; 1001 filtered out

cargo test -p game_core --lib authored_defense_route_enemy_progresses_past_spawn_boundary -- --exact --nocapture
  0 selected; filter error; not accepted as evidence

cargo test -p game_core --lib game::events::combat::tests::authored_defense_route_enemy_progresses_past_spawn_boundary -- --exact --nocapture
  1 passed; 0 failed; 0 ignored; 1015 filtered out

cargo test -p game_core --lib boss_runtime::tests -- --nocapture
  25 passed; 0 failed; 0 ignored; 991 filtered out
```

The engine family run exercised all 16 engine-local Direct-dependent tests plus three retained controls. The route
run exercised the seventeenth Direct-dependent test. The Boss run included both ordinary-leaf movement/rejoin and
movement-lock timeout controls. All accepted baselines are nonzero and green.

### Dirty/marker check

- Pre-transfer expanded status remains exactly 576 rows at canonical C-sort SHA-256
  `93625e86d7ea3bce8539875054846dc9d48b1aea3e82526d6c060bfc1326376e`.
- Existing scoped diffs before Phase-39 mutation are four additive lines in `engine.rs`, one additive derive in
  `rapier_backend.rs`, and earlier campaign-owned changes in `combat.rs`/`core/mod.rs`; activation hashes remain the
  attribution boundary.
- Protected marker remains 57 bytes at SHA-256 `614d2fbd...` with exact recorded timestamps.

### Decision

Proceed with policy-independent assertion transfer only: add/strengthen retained Rapier assertions and switch Core/
authored-route controls to the unselected default path while leaving every public backend symbol and Direct carrier
present. Run the transferred controls before removing any old assertion carrier. Public consolidation remains gated
by an exact candidate-scoped CRA-POL-006 approval or migration decision.

## 2026-08-11 — E04 Retained Assertion Transfer And Policy Stop

### Mutation

- Added retained Rapier controls for canonical input/result order, moving-candidate overlap, boundary overlap/no
  depenetration and `TargetReached` identity.
- Strengthened retained Rapier controls with exact movement velocity and exact ground board-clamp endpoint.
- Changed immutable segment, no-motion segment and authored-route controls to use the unselected `BattleCore`
  production default. Changed the existing explicit Rapier Core twin into an unselected default-Core control.
- Kept all old Direct/parity tests, Direct implementation, backend enum/selectors, public Rapier surface and test-only
  board walls intact. Thus retained proof ran before any old assertion carrier or public surface removal.

### Exact results

```text
cargo test -p game_core --lib game::battle::core::movement::rapier_backend::tests -- --nocapture
  19 passed; 0 failed; 0 ignored; 1001 filtered out

cargo test -p game_core --lib game::battle::core::movement::engine::tests -- --nocapture
  19 passed; 0 failed; 0 ignored; 1001 filtered out

cargo test -p game_core --lib game::events::combat::tests::authored_defense_route_enemy_progresses_past_spawn_boundary -- --exact --nocapture
  1 passed; 0 failed; 0 ignored; 1019 filtered out

cargo test -p game_core --lib boss_runtime::tests -- --nocapture
  25 passed; 0 failed; 0 ignored; 995 filtered out
```

The Core test total is now 1,020 before old-carrier cleanup. Transfer path hashes are `engine.rs` 1,361 lines at
`3fbe87ef...`, `rapier_backend.rs` 1,095 at `4a59d5d7...`, and `combat.rs` 1,970 at `5c232c38...`.
The protected marker remains exact at `614d2fbd...`.

The first `cargo fmt --all -- --check` selected the complete workspace and exited 1 solely for two wrapping diffs in
new Rapier tests. After applying exactly those formatter shapes, `cargo fmt --all -- --check` and whole-worktree
`git diff --check` both exit 0. No bulk formatter mutation was used.

### Policy result

The required one-private-backend target necessarily removes publicly reachable Rust symbols. Negative workspace and
accessible-directory searches plus the sole known `game_server` path consumer are strong local evidence, but the
publishable manifest and missing deployment registry leave external compatibility unknown. Per CRA-POL-006, all
three Child headers transition to `policy_decision_required`; Phase 40 remains pending/absent. Required input is the
exact breaking-cleanup approval stated in `PLAN.md`, or a named deprecation/versioned migration lifetime.

## 2026-08-11 — E05 Remaining-Phase Cleanup Approval And Resume

- The user instructed that Phase 39 be returned to active, cleanup resume, and all remaining Phases 39~44 proceed
  without further compatibility approval pauses.
- For Phase 39 this explicitly resolves CRA-POL-006 for removal of `ContinuousMovementBackend`,
  `DirectContinuousMovement`, both `BattleCore` backend selectors, public reachability of the retained Rapier
  implementation/inspection surface, and the test-only board-wall world, with no shim or deprecation period.
- The broader approval is limited to each registered cleanup candidate after its required consumer/data/tool
  inventory and behavior guards. It does not authorize gameplay changes, skipping assertion transfer, broadening a
  Child scope or activating later phases early.
- The user's targeting clarification is also binding for Phase 43: retain tile-range eligibility (`TileRangePolicy`,
  `target_range` and its runtime predicate); the accepted `Nearest` behavior filters by tile eligibility and then
  uses current-target preference/world-distance/UUID tie behavior. Only proven ghost `range_units`/public helper
  surfaces are cleanup candidates.
- All three Phase-39 headers return to `active`; Master Phase 39 returns to `active`. Phase 40 remains pending and
  absent until Phase 39 cleanup, broad validation and a separate `Complete / High` audit finish.
- Resume fingerprints are unchanged from E04: 576 status rows at `93625e86...`; Phase-39 source hashes remain
  `3fbe87ef...`, `4a59d5d7...`, `5c232c38...`; the protected marker remains exact at `614d2fbd...`.

## 2026-08-11 — E06 Authorized Cleanup And Validation

### Cleanup

- Replaced the stored enum with the sole `RapierMovementWorld`; reset now clears that world directly.
- Made `rapier_backend` private, restricted its world and `clear` operation to `battle::core`, and made the engine
  trait module-private. Removed the backend enum, Direct implementation, both selectors, swept-AABB/depenetration
  helpers, test-only Rapier board-wall world and unused public inspection/sync surface.
- Extracted the unchanged manual board clamp into a neutral module-private helper used by the shared tick and Rapier
  path. No collision, terrain, goal, movement-output, event, data or wire policy changed.
- Removed 14 old Direct/parity carrier tests and two test-only board-wall tests only after E04's retained controls
  passed. Added one default-Core repeatability test comparing two ticks, final positions and serialized logs under
  reversed unit insertion order. The test list therefore changed from 1,071 to 1,056: `-16 +1 = -15`.

### Failure classification

- The first post-cleanup Rapier command did not compile and selected no tests: Rust E0365/E0603 rejected a
  `pub(super)` world re-exported to the broader Core scope, and one binding was unnecessarily mutable. This was an
  implementation visibility error, not behavioral evidence.
- Restricted the world explicitly with `pub(in crate::game::battle::core)` and removed the unused `mut`. The
  corrected Rapier run selected 17 and passed all 17; the zero-selected attempt remains classified as failed.

### Focused results

```text
engine/default Core                         6 passed; 0 failed; 0 ignored; 999 filtered
Rapier retained backend                    17 passed; 0 failed; 0 ignored; 988 filtered
movement family                            61 passed; 0 failed; 0 ignored; 944 filtered
Boss runtime                               25 passed; 0 failed; 0 ignored; 980 filtered
tactical family                            34 passed; 0 failed; 0 ignored; 971 filtered
range family                               52 passed; 0 failed; 0 ignored; 953 filtered
event family                               76 passed; 0 failed; 0 ignored; 929 filtered
world family                              293 passed; 0 failed; 0 ignored; 712 filtered
authored defense route exact                1 passed; 0 failed; 0 ignored; 1004 filtered
```

The Rapier controls preserve deterministic result order, exact velocity and clamp, static-obstacle stop/no-slide,
airborne bypass, moving/boundary overlap, locked-body pass-through and target identity. Default Core preserves
immutable/negative segment publication and repeatable logs/final state. The Boss and authored-route controls still
consume the ordinary movement leaf.

### Broad and static results

```text
cargo test -p game_core --all-targets
  lib 1005/1005; integration 4+4+25+14+4; total 1056/1056
cargo test -p game_core --all-targets --release
  lib 1005/1005; integration 4+4+25+14+4; total 1056/1056
cargo check --workspace --all-targets
  game_core + game_server; exit 0
cargo check --workspace --all-targets --release
  game_core + game_server; exit 0
cargo clippy -p game_core --all-targets
  exit 0 with inherited warnings; target engine/Rapier path diagnostics: 0
cargo fmt --all -- --check
  exit 0
git diff --check
  exit 0
```

The final 1,056-row `: test` list has SHA-256 `35a110371b12841df4d86281f99c6d55dae6195860a5501e8334eae5fd7c3ecf`.
Legacy symbol search across Core/server source and tests returns zero for the enum, Direct type, selectors, board
world and wall methods. The only backend reachability is the private module, Core-restricted world/clear operation
and module-private engine trait.

### Attribution and compatibility

- Final movement carriers are `engine.rs` 819 lines at `222836cd...`, `rapier_backend.rs` 921 at `b70188da...`,
  movement `mod.rs` 27 at `aaac13d9...`, Core `mod.rs` 8,523 at `9c9f01d0...`, `sim.rs` 3,359 at
  `af5130fc...`, and unchanged post-transfer `combat.rs` 1,970 at `5c232c38...`.
- Workspace/Core/server manifests, lockfile and Core lib root remain at their activation hashes. Live Boss RON stays
  `f696c9d0...`; gameplay/runtime/data canonicals remain `6c53ecd5...`, `ab7d2f98...`, `30ed5fd5...`.
- Expanded status is 577 rows at canonical C-sort SHA-256 `7bb66e69...`; the one additional activation-relative row
  is the formerly clean movement `mod.rs` made private by this cleanup. Pre-existing dirty rows were preserved by
  activation hashes and scoped edits; the forthcoming independent audit will add one separate evidence path.
- The protected marker remains 57 bytes at SHA-256 `614d2fbd...`, with mtime/ctime exactly
  `2026-08-10 22:55:07.178423600 +0900`.

### Decision

Implementation and validation evidence satisfy P39-C01~C11. Hand off to a separate completion audit; do not
activate or inspect Phase 40 until that audit returns `Complete / High`.

## 2026-08-11 — E07 Completion Audit And Transition

- The separate 232-line completion audit at SHA-256 `456b37e1...` independently reran current Core all-target debug
  1,056/1,056, repeated legacy/public/active-Child/Phase-40-absence checks and returned `Complete / High` with all
  twelve evidence rows passing.
- The audit artifact alone explains the pre-audit 577→578 expanded-status transition, whose canonical C-sort
  SHA-256 is `4665ba9b...`. The protected marker remains exact.
- All three Child headers transition to `complete` / `Implementation Complete: true`; Master Phase 39 becomes
  `complete — Complete / High` and Phase 40 becomes `ready` only.
- No Phase-40 production content was inspected or mutated during this exit transition.

## Execution Log Contract

Future entries record meaningful prospective fingerprints, complete current backend/test/consumer discovery,
baseline and assertion transfer, compatibility disposition, cleanup, validation and audit handoff. Every test entry
includes selected/pass/fail/ignored counts; failed or zero-selected commands are retained and classified.
