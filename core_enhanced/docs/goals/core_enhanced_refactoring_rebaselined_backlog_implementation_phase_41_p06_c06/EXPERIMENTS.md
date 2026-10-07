# Phase 41 CRA-P06-C06 Experiments

Status: complete
Implementation Complete: true

## 2026-08-11 — E00 Goal Initialization

### Hypothesis

The public `ScenarioEvent.once` field can be removed mechanically only if current code confirms it is unread, every
producer expresses the same one-shot value, no serialized/external consumer depends on the shape, and current
enqueue/order/group-dedupe behavior already owns lifecycle. Initializing the Child before current source inspection
preserves the one-active-Phase and dirty-worktree contracts.

### Performed checks

- Read the complete Goal authoring/execution guide and execution attachment earlier in this Master run.
- Verified Master Phase 40 is `complete — Complete / High`, Phase 41 is `ready`, Phases 42~44 are pending, and no
  implementation Child is active.
- Read the original P06-C06 candidate, P06 boundary evidence, post-Boss reaudit and final candidate/order/wave/test/
  policy records relevant to the ghost input.
- Verified Phase 41 path was absent and captured HEAD, expanded status identity and protected marker.
- No current Phase-41 production source, constructor, live data, consumer or test was inspected; no test or formatter
  ran.

### Actual result

- Repository/HEAD: `/mnt/f/work/simulator` / `f318414fb0156a67e04c5479000c2a2683fc0c09`.
- Direct dependency: Phase 40 audit is 229 lines at `ea94b13a...`, verdict `Complete / High`.
- Candidate: CRA-P06-C06, cleanup, Tier D, score `1/0/1/1/1/0/unknown = 4`.
- Historical evidence says the public field is unread and every constructor writes `true`; current proof still needs
  an exhaustive definition/value/constructor/serde/consumer inventory plus delayed-spawn, forced-end and validation
  guards.
- Pre-Child expanded status: 585 rows at canonical C-sort SHA-256 `54b165d7...`.
- Active implementation Child count: 0. Phase-41 path: absent. Marker: exact at `614d2fbd...`.

### Decision

Initialize exactly `PLAN.md`, `EXPERIMENTS.md` and `EXPERIMENT_NOTES.md`. Activation and a full reread of all three
documents must precede filename-only prospective fingerprints and current production discovery.

## 2026-08-11 — E01 Sole Activation

- The initialized Child contains exactly the required three documents.
- Their addition changes expanded status exactly from 585 to 588 rows at canonical C-sort SHA-256
  `34c0f60a706e4a8ff9bf3d4b29b9fccc1b843e69820d022a659f95521cbbf800`.
- All three headers change from `initialized` to `active`; `Implementation Complete` remains `false`. Master Phase 41
  changes from `ready` to `active`; it is the sole active Child and Phase 42 remains pending/absent.
- No current production content, constructor, data consumer or test was inspected and no production mutation/test ran.
- Next gate: fully reread all three active documents and direct/final evidence, then record filename-only prospective
  fingerprints before opening current production content.

## 2026-08-11 — E02 Active Reread And Prospective Fingerprints

### Procedure

- Fully reread all three active Child documents and the complete Phase-40 completion audit.
- Reconfirmed original/post-Boss/final candidate, exact-order and test requirements after activation.
- Used filename-only discovery plus mechanical line/hash/status capture for prospective scenario, data, setup, tests
  and downstream paths without opening their current source content.

### Activation state

- Expanded status remains 588 rows at canonical C-sort SHA-256 `34c0f60a...`; this PLAN is the sole active Child.
- Prospective primary carriers are already dirty: battle `scenario.rs` 626 lines `6a65d9f6...`, Core `build.rs` 424
  `1125b384...`, Core `mod.rs` 8,511 `2ea099ae...`, Core `sim.rs` 3,359 `af5130fc...`, setup `mod.rs` 9
  `1d2471ba...`, data `pve_data.rs` 1,605 `3dfb09a6...`, `live_ron.rs` 3,999 `1fb386ed...` and validation 1,399
  `c71c4208...`. Prospective setup `scenario_groups.rs` is clean, 48 lines at `e0ce06dd...`.
- Prospective dirty test/behavior carriers are event combat 1,970 `5c232c38...`, world combat tests 11,833
  `08177bd5...`, integration RON loading 1,971 `75558ff2...` and skill-test scenario helper 449 `a80cc0b4...`.
- Live PVE encounters RON is already dirty, 1,540 lines at `0b3f03b7...`.
- Workspace/Core/server manifests, lockfile and Core lib root retain `98c9cf4d...`, `40d8a9ec...`, `19b41cd...`,
  `4223318b...` and `3fbfdca2...`; gameplay/runtime/data canonicals retain `6c53ecd5...`, `ab7d2f98...`,
  `30ed5fd5...`.
- Protected marker remains exact at `614d2fbd...`, 57 bytes, with exact recorded mtime/ctime.

### Decision

The likely field/constructors overlap extensive completed-campaign dirty files, while `scenario_groups.rs` is clean.
Activation hashes are the attribution boundary; no prospective path is presumptively a target. Current definition,
constructor/value, reader, serde/data, enqueue/dedupe and test discovery may now begin.

## 2026-08-11 — E03 Current Contract And Consumer Inventory

### Exact shape and values

- `ScenarioEvent` is a public `Debug, Clone` Rust struct in the public `game::battle::scenario` module. It has public
  `id`, `trigger`, `action` and `once` fields; it has no serde derive, `Default`, builder, update constructor or
  custom serializer/deserializer.
- Before adding the retained validation assertion, current production/tests contained 23 literal constructors and
  all 23 wrote `once: true`. The assertion added in E04 makes that 24 literal writes while the field still exists.
- Exhaustive current searches found zero `once: false`, zero `.once` reads, zero assignments, zero destructuring
  matches, zero omitted/default construction and zero live RON/JSON/TOML authoring of this field. The unrelated
  `once:` text in `skill_data.rs` is inside another string/contract and is not a `ScenarioEvent` member.
- The pre-assertion 23 writers were: Core battle tests 11, Boss Runtime tests 2, field-work tests 1, combat setup 1,
  `live_item_skill_activation` 4, `skill_runtime_contract` 2 and skill-test scenario helper 2. No non-test runtime
  writer varies the value.

### Runtime authority

- `start_battle_execution` calls `enqueue_scenario_events` once after reset and scenario validation. Enqueue clones
  the event list once, maps `AtBattleStart`/`AtTimeMs` to queue time and maps actions to `SpawnGroup`/`EndBattle`;
  it never reads `once`.
- `spawn_scenario_group` checks `scenario_runtime.spawned_groups` before materialization and records the group ID
  after successful materialization. Duplicate requests return an empty unit list without advancing spawn order or
  changing unit-ref bindings. Group identity, not the ghost boolean, owns one-shot spawn materialization.
- Scenario validation owns unique event IDs, known group references and required-enemy-group reachability. Event
  heap ordering owns time/priority/identity order. Neither path reads or matches `once`.

### Public/data/deployment disposition

- The Core manifest has no `publish = false`, so the public Rust shape is treated as potentially publishable.
  Immediate verified breaking cleanup is covered by the user's explicit 2026-08-11 CRA-POL-006 approval; no shim,
  alias or deprecation is introduced.
- Current Core/server/data/tools searches found no serde/data/save/DTO/wire/Unity consumer. Live PVE RON is parsed
  into encounter data and then transformed by `combat_setup::scenario_groups`; `ScenarioEvent` itself is never
  deserialized.
- An accessible `/mnt/f/work` Rust/C#/RON/JSON/TOML scan found no consumer outside this repository. Its only hits
  were the current repository writers plus one unrelated skill-data string.

### Decision

The current field is a public but semantically false input: universally `true`, unread and outside serialized/live
data. Removal is authorized after retained one-shot, validation, delayed-spawn, forced-end and ordering assertions
are green with the field still present.

## 2026-08-11 — E04 Pre-Removal Behavior And Assertion Transfer

### Existing nonzero baselines

- delayed required group at 1,000 ms: selected 1, passed 1, failed 0, ignored 0;
- stepped execution does not jump to a future wave: selected 1, passed 1, failed 0, ignored 0;
- full versus stepped forced-end execution: selected 1, passed 1, failed 0, ignored 0;
- event heap time/priority/identity ordering: selected 1, passed 1, failed 0, ignored 0;
- existing scenario validation module: selected 4, passed 4, failed 0, ignored 0.

### Retained assertions added before deletion

- `scenario_group_identity_is_authoritative_one_shot_guard` directly spawns the same authored group twice and proves
  the second request produces no unit, unit-count change, spawn-order advance or unit-ref rebinding.
- `scenario_event_validation_keeps_ids_and_group_references_authoritative` proves a valid event is accepted while a
  duplicate event ID and an unknown spawn group are rejected with the exact current contract.
- Both exact controls passed with `once` still present: each selected 1, passed 1, failed 0, ignored 0. Core library
  count increased from 1,006 to 1,008 tests; no existing assertion was replaced or weakened.
- `cargo fmt --all` completed successfully before the exact controls.

### Decision

The useful meaning previously suggested by the public boolean is now explicitly guarded at its actual authorities.
Remove only the field declaration and the 24 mechanical `true` writes, then rerun the same controls before broader
families.

## Execution Log Contract

Future entries record activation fingerprints, exhaustive constructor/value/reader/serde/public inventory, nonzero
baselines, authorized cleanup, validation and audit handoff. Failed and zero-selected commands remain classified;
every test entry records selected/pass/fail/ignored counts.

## 2026-08-11 — E05 Authorized Ghost-Input Removal

### Applied scope

- Removed the public `ScenarioEvent.once: bool` member and every one of the 24 mechanical `once: true` literal
  writes classified in E03/E04. No replacement flag, default, alias, compatibility shim or repeat scheduler was
  added.
- The only previously clean path that became a production status row is
  `core_enhanced/src/game/combat_setup/scenario_groups.rs`; all other edited carriers were already dirty at the
  prospective fingerprint gate. Current expanded status is 589 rows at canonical C-sort SHA-256
  `cdf109f035c583079b80fd2bcfd8a1fe60a894c4b6215a5fcc2790e7502638de`.
- Exact post-removal search selects zero `.once` reads and zero `once:` scenario members/writes. The sole remaining
  `once:` text is an unrelated diagnostic string in `skill_data.rs`.

### Post-removal focused validation

- `cargo test -q -p game_core scenario_ -- --nocapture`: library 11/0/0 and `ron_loading` 1/0/0; all other targets
  selected zero and are not counted as behavior evidence.
- `cargo test -q -p game_core step_battle_execution -- --nocapture`: library 2/0/0, including future-wave stepping
  and full/stepped forced-end parity.
- An initial `--exact` heap-order command used an incomplete module path and selected zero tests; it is rejected as
  evidence. Retrying `cargo test -q -p game_core battle_event_heap_orders_by_time_then_priority_then_ids --
  --nocapture` passed 1/0/0.
- `cargo test -q -p game_core --test ron_loading -- --nocapture`: 25/0/0.
- `cargo test -q -p game_core game::battle::core::boss_runtime::tests -- --nocapture`: 22/0/0.
- `cargo test -q -p game_core game::events::combat::tests -- --nocapture`: 14/0/0.

### Decision

Removal is mechanically exact and every retained one-shot, validation, delayed-spawn, forced-end, ordering, live
data and Boss control is green. Proceed to both-profile broad and static verification.

## 2026-08-11 — E06 Broad Validation And Audit Handoff

### Broad results

- `cargo test -q -p game_core --all-targets`: 1,059 passed, 0 failed, 0 ignored. Distribution is library 1,008 plus
  integrations 4 + 4 + 25 + 14 + 4; six executable/fixture targets select zero tests.
- `cargo test -q -p game_core --release --all-targets`: the identical 1,059/0/0 distribution.
- `cargo check --workspace --all-targets` and `cargo check --workspace --all-targets --release`: both exit 0 and
  compile `game_core` plus `game_server`.
- `cargo clippy -p game_core --all-targets`: exit 0 with 80 inherited library and 122 inherited library-test
  warnings; no `ScenarioEvent.once` or Phase-41 warning exists.
- `cargo fmt --all -- --check` and whole-worktree `git diff --check`: both exit 0.
- Current sorted library test table is 1,008 rows at SHA-256
  `2ee9300f8f8a94fe614abd1828ad081b60544503a6cb8e2694ff3d5e73166e72`.

### Boundary verification

- Workspace/Core/server manifests, lockfile and Core lib root retain activation hashes `98c9cf4d...`, `40d8a9ec...`,
  `19b41cd...`, `4223318b...` and `3fbfdca2...`.
- Gameplay/runtime/data canonicals retain `6c53ecd5...`, `ab7d2f98...` and `30ed5fd5...`; live PVE RON retains
  `0b3f03b7...`. No canonical or live-data edit is required because `ScenarioEvent` was never a serialized input.
- The protected marker remains 57 bytes at SHA-256 `614d2fbd...` with exact recorded timestamps.
- Current central hashes are scenario `a33d434b...`, scenario-groups `292e0af3...`, Core module `a17dff27...`, Boss
  runtime `7388b91f...`, field work `fe7566e8...` and unchanged sim `af5130fc...`.

### Handoff

Implementation and required validation are complete, but the Child remains `active` with
`Implementation Complete: false` until a separate Phase-exit completion review reconstructs the current evidence
and returns `Complete / High`. Phase 42 remains pending and absent.

## 2026-08-11 — E07 Independent Completion Audit And Exit

- The separate 224-line Phase-exit audit at SHA-256
  `b800f55ac439eb33c8c8dc3b005f5e10324d7b0d41be613ac5c5f4f275d25497` independently reconstructed all 12
  completion rows and returned `Complete / High`, fit `High`, improvement class `none`.
- Its audit refresh reran the two exact authority controls 1/1 each and the complete Core debug all-target suite
  1,059/0/0, then repeated current legacy, consumer, manifest/canonical, dirty and marker checks.
- The audit artifact alone explains the exact 589→590 expanded-status increase at canonical C-sort SHA-256
  `21f63c04e1ef6b957449f412d6aa05b5619118602b500cb99372ea0b9ed49b45`; the marker remains exact.
- Phase 41 transitions to `complete` with `Implementation Complete: true`. Master Phase 42 may become `ready` only;
  no Phase-42 Child or production inspection exists at this transition.
