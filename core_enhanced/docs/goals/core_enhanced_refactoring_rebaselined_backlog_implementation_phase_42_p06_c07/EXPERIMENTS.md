# Phase 42 CRA-P06-C07 Experiments

Status: complete
Implementation Complete: true

## 2026-08-11 — E00 Goal Initialization

### Hypothesis

General public battle-log file helpers can be narrowed only if current inventory proves them unused while standard
serde, server compression and one explicit best-effort debug trace boundary remain fully protected. Initializing the
Child before current source inspection preserves the one-active-Phase and dirty-worktree contracts.

### Performed checks

- Read the complete execution attachment and Master required documents earlier in this run.
- Read Phase 41's 224-line `Complete / High` audit as the direct predecessor gate.
- Read the original/post-Boss P06-C07 evidence and final catalog/order/wave/test/policy records.
- Verified Phase 42 path was absent, active implementation Child count was zero, and captured HEAD, expanded status
  identity and protected marker.
- No current Phase-42 production source/helper/caller/path/test/data/consumer was inspected; no test or formatter ran.

### Actual result

- Repository/HEAD: `/mnt/f/work/simulator` / `f318414fb0156a67e04c5479000c2a2683fc0c09`.
- Direct dependency: Phase 41 audit is 224 lines at `b800f55a...`, verdict `Complete / High`.
- Candidate: CRA-P06-C07, cleanup, Tier D, score `1/0/0/1/1/0/unknown = 3`.
- Historical evidence names unused public read/plain-write helpers, retained in-memory JSON, server compression and
  an explicit best-effort debug trace; every current fact remains discovery pending.
- Pre-Child expanded status: 590 rows at canonical C-sort SHA-256 `21f63c04...`.
- Active implementation Child count: 0. Phase-42 path: absent. Marker: exact at `614d2fbd...`.

### Decision

Initialize exactly `PLAN.md`, `EXPERIMENTS.md` and `EXPERIMENT_NOTES.md`. Activation and a full reread of all three
documents must precede filename-only prospective fingerprints and current production discovery.

## Execution Log Contract

Future entries record activation fingerprints, exhaustive method/caller/path/error/public inventory, nonzero
baselines, assertion transfer, authorized cleanup, validation and audit handoff. Failed/zero-selected commands are
classified and every accepted test result records selected/pass/fail/ignored counts.

## 2026-08-11 — E01 Sole Activation

- The initialized Child contains exactly the required three documents.
- Their addition changes expanded status exactly from 590 to 593 rows at canonical C-sort SHA-256
  `0a3ffece7401ea4bc7cee011fe93289ec213aa9c96ceb18d13dcdeef9221b256`.
- All three headers change from `initialized` to `active`; `Implementation Complete` remains `false`. Master Phase 42
  changes from `ready` to `active`; it is the sole active Child and Phase 43 remains pending/absent.
- No current production content, helper, caller, path, data consumer or test was inspected and no production
  mutation/test ran.
- Next gate: fully reread all three active documents and direct/final evidence, then record filename-only prospective
  fingerprints before current production discovery.

## 2026-08-11 — E02 Active Reread And Prospective Fingerprints

### Procedure

- Fully reread all three active Child documents and reconfirmed Phase-41/final P06-C07 evidence.
- Used filename-only discovery plus mechanical line/hash/status capture for prospective event-log, recording, world
  trace/combat, server actor, tests, manifests and canonical paths without opening current source content.

### Activation state

- Expanded status remains 593 rows at canonical C-sort SHA-256 `0a3ffece...`; this PLAN is the sole active Child.
- Prospective event log is already dirty: 605-line `event_log.rs` at `397e2380...`. Private recording is clean,
  79 lines at `aa2fca88...`.
- Prospective world combat is already dirty, 3,251 lines at `918e8659...`; explicit `combat_trace.rs` is clean,
  91 lines at `e7a33821...`. World combat tests are dirty, 11,833 lines at `08177bd5...`.
- Prospective server actor carriers are already dirty: handlers 3,200 `431abf19...`, messages 996 `8e3c5369...`
  and state 1,163 `1d2cb323...`. Server lib remains clean at `1c918b2d...`.
- Prospective integration/runtime carriers are dirty: event combat 1,970 `5c232c38...`, skill runtime 2,669
  `e68c49f8...`, active run 138 `bea18fa6...` and snapshot 1,476 `11c1d3b5...`.
- Workspace/Core/server manifests, lockfile and Core lib root retain `98c9cf4d...`, `40d8a9ec...`, `19b41cd...`,
  `4223318b...` and `3fbfdca2...`. Runtime/gameplay/server canonicals retain `ab7d2f98...`, `6c53ecd5...` and
  `c82000b1...`.
- Protected marker remains exact at `614d2fbd...`, 57 bytes, with recorded timestamps.

### Decision

The likely data type and consumers overlap completed-campaign dirty files, while the recording and explicit trace
modules are clean. Activation hashes are the attribution boundary; no path is presumptively a target. Current
method/caller/path/error/serde/compression/consumer discovery may now begin.

## 2026-08-11 — E03 Current Contract And Consumer Inventory

### Data-type surface

- `BattleEventLog` is public, derives `Serialize, Deserialize`, and contains public `version` plus ordered `entries`.
- It currently exposes compact `to_json_string`, unused `to_pretty_json_string`, unused compact `write_json`,
  test-only pretty `write_pretty_json` and unused `read_json`.
- Exact Core/server/tool/external searches find zero consumers of `write_json`, `read_json` and
  `to_pretty_json_string`. `write_pretty_json` has exactly three test-only debug-export wrappers: integration common,
  world test common and Core command-test common.

### Retained authorities

- Production `world::combat_trace` owns the only runtime filesystem lifecycle. It builds the canonical
  `target/combat_traces/run_<seed>/<battle_uuid>.json` path and directly uses standard
  `serde_json::to_writer_pretty` for a richer trace envelope.
- `transition_finished_live_battle_to_result` catches every trace export error, logs a warning and still commits the
  combat-result state transition. The existing failure test directly blocks its run directory and proves this.
- Server snapshot compression consumes only `BattleEventLog::to_json_string`, gzip-compresses those exact bytes and
  base64-encodes them in the current DTO. No file helper participates.
- There is no production readback, save/checkpoint field, replay/history store or Boss-specific log file lifecycle.

### Public/data/deployment disposition

- Core remains potentially publishable and the helpers are public, but the user's approved remaining-cleanup break
  applies after this complete inventory. No deprecation/forwarder is authorized.
- Current repository and accessible `/mnt/f/work` Rust/C#/script/config scans find no consumer of the four removable
  helper names outside the definitions and the three classified test-only pretty writers.
- Live RON, DTO/wire event content, server compression, combat trace envelope and canonical trace policy are retained
  unchanged.

### Decision

The false authority is the complete general file-I/O/pretty surface, not event-log serde. Establish direct standard
serde and compressed-byte assertions plus the existing explicit trace success/failure guards before replacing the
three test-only pretty method calls with standard serde at their own debug boundaries.

## 2026-08-11 — E04 Pre-Removal Behavior And Assertion Transfer

### Existing nonzero baselines

- event-log serde-shape module: selected 2, passed 2, failed 0, ignored 0;
- trace export failure cannot block result transition: selected 1, passed 1, failed 0, ignored 0;
- defense combat debug export/production trace success: selected 1, passed 1, failed 0, ignored 0;
- server final update before compressed combat-result snapshot: selected 1, passed 1, failed 0, ignored 0.

### Retained assertions added before cleanup

- Core `standard_serde_round_trip_matches_compact_event_log_json` proves compact helper bytes equal standard serde,
  then standard-deserializes and value-compares a nonempty versioned log.
- Server `compressed_battle_event_log_payload_round_trips_exact_compact_json` proves winner/encoding/byte counts,
  base64 decode, gzip decode, byte-for-byte compact JSON equality and standard `BattleEventLog` deserialization.
- Both exact controls passed with the full old public file/pretty API still present: each selected 1, passed 1,
  failed 0, ignored 0. Core library count increased from 1,008 to 1,009 and server main count from 62 to 63.
- The first server run reported one unused test import; it was removed without changing behavior and must be clean on
  the post-cleanup exact rerun.

### Decision

Serialization, compression and real trace ownership are directly protected. Remove the four general public file/
pretty helpers and replace only the three test debug-writer calls with local standard pretty-serde file output.

## 2026-08-11 — E05 Public File/Pretty Surface Removal

### Implementation

- Removed `BattleEventLog::{to_pretty_json_string, write_json, write_pretty_json, read_json}` and the now-dead
  `File`/`BufReader`/`BufWriter`/`Path` imports. Public versioned serde, ordered entries, `to_json_string` and
  `entries_after_seq` remain unchanged.
- Replaced the three classified test-only debug writers with local `File::create` plus
  `serde_json::to_writer_pretty(BufWriter::new(file), event_log)` at integration common, world-test common and
  command-test common. No production wrapper, readback API, compatibility shim or durable history was introduced.
- Exact executable-source search across Core and server selects zero removed helper names. Historical and current
  Goal/audit prose remains intentionally searchable as evidence.

### Focused post-removal proof

- standard serde/compact equality and round trip: 1/0/0;
- server gzip+base64 exact compact JSON round trip: 1/0/0 with the unused-import warning gone;
- production trace failure remains non-blocking: 1/0/0;
- defense trace/debug export success: 1/0/0;
- command-test local writer: 1/0/0; integration common writer suite: 4/0/0;
- final live-battle update before compressed result snapshot: 1/0/0.

### Attribution

All five implementation/test carriers were already dirty at activation. Current relevant hashes are event log
`5baad222...`, integration common `407fa603...`, world-test common `827bb1e0...`, command tests `8ee5a941...` and
server actor state `85fe7208...`. Clean production `combat_trace.rs` remains byte-identical at `e7a33821...`.

## 2026-08-11 — E06 Broad Validation And Completion Readiness

### Broad/runtime results

- Core debug and release all-target suites each pass 1,060/0/0: library 1,009 plus integrations
  4 + 4 + 25 + 14 + 4.
- Server debug and release all-target suites each pass 63/0/0.
- Workspace debug/release all-target checks exit 0.
- Normal Core and server all-target Clippy exit 0 with inherited warnings; no warning names `event_log.rs`, the
  removed API or the new server compression control.
- Workspace format check and whole-worktree diff check exit 0.

### Static/status results

- Sorted Core library test list is 1,009 rows at `8500e10743881d3bac73008ddbb4740c1176a8e16dc98a01665cba00378e5214`.
  Sorted server library test list is 63 rows at
  `beca9e7a9162c0a9a1511e30061703d965452e87a9d52f8c0a15cfe0923d4588`.
- Expanded dirty state remains 593 rows; current canonical C-sort SHA-256 is `4d503de4...`. No clean candidate
  implementation path became dirty and no generated debug export is a status row.
- Workspace/Core/server manifests, lockfile and Core lib root remain `98c9cf4d...`, `40d8a9ec...`, `19b41cd...`,
  `4223318b...` and `3fbfdca2...`; runtime/gameplay/server canonicals remain `ab7d2f98...`, `6c53ecd5...` and
  `c82000b1...`.
- Protected marker remains 57 bytes at `614d2fbd...` with exact recorded mtime/ctime.

### Decision

Implementation and validation are complete. The independent completion audit must now reread the rubric, rerun
removed-symbol plus direct serde/compression controls and refresh broad debug evidence before any status transition.

## 2026-08-11 — E07 Independent Completion Audit And Exit

- The separate 225-line `COMPLETION_AUDIT.md` at SHA-256
  `dc2abc41135453991d616ce9f501adf698d143dd3e6542a8b9db05a4c8d2762f` independently applies the completion
  guide and passes all 12 Child conditions.
- Audit refresh selects zero removed symbols in executable Core/server source, passes the standard-serde and exact-
  compression controls 1/1 each, Core debug all-targets 1,060/0/0 and server debug all-targets 63/0/0.
- The audit artifact alone changes expanded status from 593 to 594 rows at canonical C-sort SHA-256 `e44f1ce8...`;
  manifests, canonicals and the protected marker remain exact.
- Primary verdict is `Complete`, long-term fit `High`, improvement class `none`, confidence High. Phase 42
  transitions to complete and Master Phase 43 becomes ready only; no Phase-43 production inspection occurs here.
