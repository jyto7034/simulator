# Phase 40 CRA-P07-C04 Experiments

Status: complete
Implementation Complete: true

## 2026-08-11 — E00 Goal Initialization

### Hypothesis

The ghost movement tick model can be reduced safely only after Phase 39 fixes the one retained backend and current
code proves each member has no runtime, serialization, external or assertion consumer. Initializing the Child before
current source inspection preserves the one-active-Phase and dirty-worktree contracts.

### Performed checks

- Read the complete Goal authoring/execution guide and the complete execution attachment.
- Verified Master Phase 39 is `complete — Complete / High`, Phase 40 is `ready`, Phases 41~44 are pending, and no
  implementation Child is active.
- Read the original P07-C04 candidate, Boss deferral registry, post-Boss P07 reaudit and final candidate/order/wave/
  test records relevant to the ghost movement contract.
- Verified Phase 40 path was absent and captured HEAD, expanded status identity and protected marker.
- No current Phase-40 production source, caller, field, test, manifest, data, canonical or consumer was inspected;
  no test or formatter ran.

### Actual result

- Repository/HEAD: `/mnt/f/work/simulator` / `f318414fb0156a67e04c5479000c2a2683fc0c09`.
- Direct dependency: Phase 39 audit is 232 lines at `456b37e1...`, verdict `Complete / High`; it leaves one private
  Rapier world and explicitly hands ghost-member review to Phase 40.
- Candidate: CRA-P07-C04, cleanup, Tier D, score `2/0/2/2/2/1/unknown = 9`.
- Historical field/variant names remain discovery hypotheses. Exit proof is a complete current field-use inventory
  plus world position/range/event and moving/stopped/blocked parity after Phase 39.
- Pre-Child expanded status: 578 rows at canonical C-sort SHA-256 `4665ba9b...`.
- Active implementation Child count: 0. Phase-40 path: absent. Marker: exact at `614d2fbd...`.

### Decision

Initialize exactly `PLAN.md`, `EXPERIMENTS.md` and `EXPERIMENT_NOTES.md`. Activation and a full reread of all three
documents must precede filename-only prospective fingerprints and current production discovery.

## 2026-08-11 — E01 Sole Activation

- The initialized Child contains exactly the required three documents.
- Their addition changes expanded status exactly from 578 to 581 rows at canonical C-sort SHA-256
  `71c6e602b0b6e1e1d55fcc22f693469015a2d4f7c0cb25e1bcf83535b433c258`.
- All three headers change from `initialized` to `active`; `Implementation Complete` remains `false`. Master Phase 40
  changes from `ready` to `active`; it is the sole active Child and Phase 41 remains pending/absent.
- No current production content, consumer, test or canonical was inspected and no production mutation/test ran.
- Next gate: fully reread all three active documents and the direct dependency/final evidence, then record
  filename-only prospective fingerprints before opening current production content.

## 2026-08-11 — E02 Active Reread And Prospective Fingerprints

### Procedure

- Fully reread all three active Child documents and the complete Phase-39 completion audit.
- Reconfirmed the original/Boss/post-Boss/final candidate, exact order, Wave-4 and test requirements after activation.
- Used filename-only discovery plus mechanical line/hash/status capture for prospective movement and downstream
  paths without opening their current source content.

### Activation state

- Expanded status remains 581 rows at canonical C-sort SHA-256 `71c6e602...`.
- Prospective movement paths and activation line/hash fingerprints are:
  - dirty `blocking.rs` 379 `10323071...`, `engine.rs` 819 `222836cd...`, `mod.rs` 27 `aaac13d9...`,
    `planner.rs` 984 `cb4cbec3...`, `rapier_backend.rs` 921 `b70188da...`;
  - clean `lifecycle.rs` 55 `46b54329...`, `path.rs` 125 `c75566b9...`, `steering.rs` 69 `3d88b375...`,
    `types.rs` 311 `2695e40e...`.
- Prospective dirty/untracked Core/Boss carriers are Core `mod.rs` 8,523 `9c9f01d0...`, `sim.rs` 3,359
  `af5130fc...`, `types.rs` 547 `ba33c56b...` and `boss_runtime.rs` 3,098 `a4b99993...`.
- Prospective downstream files are already dirty: event log 605 `397e2380...`, tile range 403 `73930102...`,
  world 829 `c533e0ff...`, range preview 1,172 `1a8fce57...`, world combat 3,251 `918e8659...`, world combat
  tests 11,833 `08177bd5...` and authored combat route 1,970 `5c232c38...`.
- Workspace/Core/server manifests, lockfile and Core lib root retain `98c9cf4d...`, `40d8a9ec...`, `19b41cd...`,
  `4223318b...` and `3fbfdca2...`. Live Boss RON is `f696c9d0...`; gameplay/runtime/data canonicals remain
  `6c53ecd5...`, `ab7d2f98...`, `30ed5fd5...`.
- Protected marker remains exact at `614d2fbd...`, 57 bytes, with exact recorded mtime/ctime.

### Decision

The likely candidate definitions overlap completed Phase-39 dirty carriers, while `movement/types.rs` and three
adjacent modules are clean. Activation hashes are the attribution boundary; clean files are not presumptive targets.
Current member/source/test/consumer discovery may now begin.

## 2026-08-11 — E03 Current Member Graph, Consumer Inventory And Baseline

### Current authority and candidate graph

- `BattleCore` builds sorted `MovementUnitInput` rows, the private Rapier world resolves them, and Core applies
  `BodyMoved`, `TargetReached` or `MovementStopped` to authoritative position/current-target/active-segment state.
- Actual input reads are unit ID, body position/radius/speed, explicit goal, terrain policy, movement permission and
  lifecycle. Tick `now_ms`, input owner/current target, and five stored `UnitBody` fields have no semantic reader.
- `UnitBody.goal` is read only after Core clones the body and overwrites that clone from the current goal map each
  tick. The goal is real authority but its persistent-body slot is false; it must move to an explicit input member.
- `GoalAssigned` and `SegmentStarted` are never constructed. Their sole consumer is one no-op apply arm.
  `MovementStopped.position/reason` and `BodyMoved.velocity` are produced but never used by Core; the latter is
  written only to the unread stored body velocity. These internal payloads are false shape, distinct from the
  retained serialized `BattleLogEvent::MovementStopped` reason/position and movement-segment event.
- `previous_position`, `velocity`, `mode` and stored `goal` are reset/written by relocation/lifecycle/Boss paths but
  never read in production. Only one tactical test asserts previous-position/velocity reset and one Rapier test
  asserts output velocity; both actual position/endpoint behaviors have stronger retained assertions.

### Public, serde and external consumers

- `movement::engine` publicly exposes tick inputs/outputs and `BattleCore` build/apply/run methods even though no
  in-tree consumer outside the movement implementation/tests names those engine types or methods. `steering` is also
  a public module with only module-private functions.
- `UnitBody` is public/serde and feeds actual Core spatial/range consumers; it remains, reduced only to its three
  authoritative fields. `WorldVec2`, `EventLogVec2`, `MovementGoal`, active segments and actual event DTOs remain.
- Repository/server/tool and accessible `/mnt/f/work` Rust searches find no external engine-type, ghost-field or
  public apply-method consumer. No RON, JSON, save, DTO, server mapping or Unity reader serializes `UnitBody` or the
  internal tick model; only derives create hypothetical shape.
- The crate remains publishable, so the user's Phase-40 verified breaking-cleanup approval is the compatibility
  authority. No wrapper, alias or deprecation copy is required or authorized.

### Prospective additional carriers

- Current discovery adds untracked `tactical_relocation.rs` 311 lines at `f61632ba...` and
  `tactical_stance.rs` 1,702 at `d6397d83...` as narrowly attributable ghost-write/test carriers.
- Clean `movement/lifecycle.rs` is 55 lines at `46b54329...`; its sole candidate hunk is one unread velocity reset.
  Any edit must remain exactly within that proven ghost write.

### Baseline results

```text
cargo test -q -p game_core --lib movement              61 passed; 0 failed; 0 ignored; 944 filtered
cargo test -q -p game_core --lib boss_runtime::tests   25 passed; 0 failed; 0 ignored; 980 filtered
cargo test -q -p game_core --lib tactical              34 passed; 0 failed; 0 ignored; 971 filtered
cargo test -q -p game_core --lib range                 52 passed; 0 failed; 0 ignored; 953 filtered
cargo test -q -p game_core --lib event                 76 passed; 0 failed; 0 ignored; 929 filtered
cargo test -q -p game_core --lib world                293 passed; 0 failed; 0 ignored; 712 filtered
```

The current all-target discovery is 1,056 rows at SHA-256 `35a11037...`; the immediately preceding Phase-39 audit
also reran the same unchanged-source debug all-target distribution at 1,056/1,056 and release at 1,056/1,056.
All Phase-40 baselines are nonzero and green.

### Decision

Proceed test-first. Add one default-Core control that characterizes semantic moving, locked/stopped and no-goal
observables using only retained fields plus final positions/event logs. Run it while every ghost member remains,
then apply the complete member disposition in one scoped cleanup.

## 2026-08-11 — E04 Retained Observable Transfer Before Cleanup

- Added `battle_core_default_moving_locked_and_no_goal_observables_are_stable` while all ghost fields, variants and
  public surfaces remained. It asserts the exact moved `from`/`to`, final position and serialized segment endpoints/
  times, plus unchanged final position and empty event log for movement-locked and no-goal stops.
- The first chained validation stopped before tests because `cargo fmt --all -- --check` reported one wrapping-only
  diff in the new test. Applied exactly that formatter shape; no bulk formatter mutation ran.
- Corrected results: exact new control 1/1/0 with 1,005 filtered; engine family 7/7/0 with 999 filtered; retained
  Rapier family 17/17/0 with 989 filtered. Format check passes.
- The transfer carrier is 903 lines at SHA-256 `591a3c2a...`. Expanded status remains 581 rows at `71c6e602...`;
  protected marker remains exact at `614d2fbd...`.

### Decision

The new test protects only semantic observables that survive the minimal contract and passes before deletion. The
full disposition matrix and candidate-scoped public-breaking approval now authorize cleanup.

## 2026-08-11 — E05 Authorized Cleanup And Implementation Validation

### Implementation

- Reduced `UnitBody` from eight members to the three current spatial facts: `position`, `radius` and `move_speed`.
  Removed `previous_position`, `velocity`, `mode`, stored `goal`, `physics_handle`, `MovementMode` and
  `UnitPhysicsHandle`, together with their constructor/reset/relocation/Boss writes and obsolete shape assertions.
- Moved current intent to explicit `MovementUnitInput.goal`. Removed tick `now_ms`, owner and current-target copies;
  the Core builder still consumes `now_ms` before constructing input to project action and hard-CC locks.
- Reduced output to applied transitions only: `BodyMoved { unit_id, from, to }`, `MovementStopped { unit_id }` and
  `TargetReached`. Removed never-produced goal/segment variants and unused velocity/position/reason payloads plus the
  velocity helper. The distinct serialized battle-log stopped and segment events were not changed.
- Made engine/steering modules and tick vocabulary private to movement/Core, narrowed the retained scheduler path,
  removed the public apply-position clone surface and made Core mutate only authoritative body position.
- Updated Rapier controls to construct the minimal input directly. Tactical relocation, lifecycle and Boss reset
  paths now clear only real lifecycle/target/segment state; facing, anchors, action state and route behavior remain.
- A release build exposed one test-only zero-goal convenience builder as `dead_code`. Removed that wrapper and made
  its two obstacle tests call the actual goal-aware builder. The release check then completed without a movement
  warning; the test list stayed byte-identical.

### Current reconstruction

- Legacy/member search across Core and server source selects zero files for every removed field, type, variant,
  payload helper and public apply method. Internal tick/build/run/apply vocabulary has zero consumer files outside
  the movement module. `UnitBody` has zero server/world/data consumer files; retained battle-Core spatial, targeting,
  command and build consumers still use its three actual facts.
- Current source fingerprints include engine 818 lines at `36e6b8e0...`, Rapier 903 at `787378c3...`, movement
  types 282 at `d18acf36...`, steering 57 at `30d0ffce...`, lifecycle 54 at `9e546c16...`, movement `mod.rs` 27 at
  `d24b9ace...`, Core types 541 at `b4b608f0...`, Core `mod.rs` 8,511 at `2ea099ae...`, Boss 3,097 at
  `224ed82d...`, relocation 304 at `8d95d775...` and tactical stance 1,690 at `61af1232...`.
- Unchanged prospective non-targets retain their activation fingerprints: blocking `10323071...`, planner
  `cb4cbec3...`, path `c75566b9...` and sim `af5130fc...`. Manifests, lockfile and lib root remain respectively
  `98c9cf4d...`, `40d8a9ec...`, `19b41cd...`, `4223318b...`, `3fbfdca2...`; live Boss RON and gameplay/runtime/data
  canonicals remain `f696c9d0...`, `6c53ecd5...`, `ab7d2f98...`, `30ed5fd5...`.

### Accepted current-tree validation

```text
exact retained-observable control             1 passed; 0 failed; 0 ignored; 1005 filtered
movement                                     62 passed; 0 failed; 0 ignored;  944 filtered
boss_runtime::tests                          25 passed; 0 failed; 0 ignored;  981 filtered
tactical                                     34 passed; 0 failed; 0 ignored;  972 filtered
range                                        52 passed; 0 failed; 0 ignored;  954 filtered
event                                        76 passed; 0 failed; 0 ignored;  930 filtered
world                                       293 passed; 0 failed; 0 ignored;  713 filtered
exact authored defense route                  1 passed; 0 failed; 0 ignored; 1005 filtered

cargo test -q -p game_core --all-targets
  lib 1006/1006; integrations 4+4+25+14+4; total 1057/1057
cargo test -q -p game_core --release --all-targets
  lib 1006/1006; integrations 4+4+25+14+4; total 1057/1057
cargo check --workspace --all-targets
  game_core + game_server; exit 0
cargo check --workspace --release --all-targets
  game_core + game_server; exit 0
cargo clippy -p game_core --all-targets
  exit 0 with inherited warnings; movement dead-code diagnostics 0
cargo fmt --all -- --check
  exit 0
git diff --check
  exit 0
```

The final 1,057-row sorted `: test` list has SHA-256
`361fb0181fd22cb7215116bca3579d970ef8022793c3ea04d8126748d3537658` and is identical before/after removing the
test-only wrapper. Debug and release distributions each contain library 1,006 plus integrations 4 + 4 + 25 + 14 +
4; six executable/fixture targets intentionally select zero tests and are not behavior evidence.

### Diagnostics not accepted as evidence

- A shortened exact filter for the new control selected 0 tests because `--exact` requires its full module path.
  It was rejected and corrected; the full route passed 1/1.
- An extra, non-required `-D warnings` Clippy diagnostic stopped on 122 inherited warnings distributed across
  unrelated existing production/tests. It did not identify a surviving Phase-40 movement warning and was not used
  as a success claim. The required normal Core Clippy completed with exit 0 and inherited warnings recorded.
- The first release validation after cleanup passed all tests but reported the test-only convenience builder as
  dead code. That state is not final evidence; the wrapper was removed and release check/full tests were rerun.

### Dirty and compatibility boundary

- HEAD remains `f318414fb0156a67e04c5479000c2a2683fc0c09`. Before the independent audit artifact, expanded status is 584
  rows at canonical C-sort SHA-256 `637c9c92e2d9091cc1b40ca573fdfd154b785a4df3b1d3fd4cf990d28ba7c83b`.
  Relative to sole activation, the only new source status rows are formerly clean lifecycle, steering and types;
  all other carriers were already dirty/untracked and are bounded by activation/current fingerprints.
- The protected marker remains 57 bytes at `614d2fbdbdb0b29def7b90d1fdbc65148aba63339eea72fef5998d4a8ec22730`;
  mtime and ctime remain exactly `2026-08-10 22:55:07.178423600 +0900`.
- No DTO, RON, save/checkpoint, server, Unity, manifest, dependency, live-data or canonical migration occurred.
  The public/serde narrowing is the verified candidate-scoped break approved by the user; no shim survives.

### Decision

Implementation and validation evidence satisfy P40-C01 through P40-C11. Hand off to a separate completion audit.
Phase 40 remains the sole active Child and Phase 41 remains absent/pending until that audit returns `Complete / High`.

## 2026-08-11 — E06 Completion Audit And Transition

- The separate 229-line completion audit at SHA-256 `ea94b13a...` independently reran current Core all-target debug
  1,057/1,057, repeated removed/public/fingerprint/status/marker checks and returned `Complete / High` with all twelve
  evidence rows passing.
- The audit artifact alone explains the pre-audit 584→585 expanded-status transition, whose canonical C-sort
  SHA-256 is `54b165d7...`. The protected marker remains exact.
- All three Child headers transition to `complete` / `Implementation Complete: true`; Master Phase 40 becomes
  `complete — Complete / High` and Phase 41 becomes `ready` only.
- No Phase-41 production content was inspected or mutated during this exit transition.

## Execution Log Contract

Future entries record activation fingerprints, complete member/consumer inventory, nonzero baselines, assertion
transfer, authorized member-level cleanup, validation and audit handoff. Failed and zero-selected commands remain
classified; every test entry records selected/pass/fail/ignored counts.
