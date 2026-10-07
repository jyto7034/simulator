# Phase 44 CRA-P10-C06 Experiments

Status: complete
Implementation Complete: true

## 2026-08-11 — E00 Goal Initialization

### Hypothesis

Historical evidence reports unused build/direct dependencies, an empty default binary/no-op ctor, fixture bins
participating as test harnesses, unconsumed logger surface and overly broad `tokio_unstable`. P10-C01~C05 and the
preceding cleanup seams are now stable and the user approved remaining package/tool cleanup, but only current Cargo
metadata, compiled-source/feature graph, retained generator baselines and standalone/accessible consumer inventory
can establish the exact safe delta.

### Required preserved boundary

- Core gameplay/domain library API/behavior and server path dependency stay exact; the only permitted public API
  removal is the proven-unconsumed logger/init package surface covered by the user's explicit approval.
- Retained generator target names, commands and semantic outputs stay exact; runs use disposable `/tmp` paths.
- Feature/default/target behavior and test discovery remain exact except proven zero-test target noise.
- No compatibility bin, wrapper dependency, alias feature, regenerated consumer fixture or unrelated cleanup.

### Initialization evidence

- Phase 43's 210-line audit at `a9b674b4...` is `Complete / High`; Master Phase 44 is `ready`.
- Original/post-Boss P10-C06 evidence and final catalog/order/wave/test records were reviewed.
- The user approved the remaining package/tool/public cleanup after current inventory without a shim.
- Pre-Child expanded state is 603 rows at `1510b29c...`; Phase 44 path was absent and no Child PLAN was active.
- Protected marker remains exact at 57 bytes and `614d2fbd...`.

### Decision

Initialize exactly PLAN/EXPERIMENTS/EXPERIMENT_NOTES without current candidate manifest/source/config/tool content
inspection, Cargo execution or production mutation. Activate only after all three initialized documents and Master
registry are synchronized and reread.

## Next Experiment

Fully reread the initialized Child and Master registry, verify the expected three-row status addition, activate the
sole Child, reread active documents, then capture filename-only prospective fingerprints before current package
content discovery.

## 2026-08-11 — E01 Sole Activation

- The initialized Child contains exactly the required three documents and explains the exact 603→606 expanded-
  status increase at canonical C-sort SHA-256 `1d7fae2a...`.
- All three headers change from initialized to active; `Implementation Complete` remains false. Master Phase 44
  changes from initialized to active as the sole Child.
- No current candidate manifest/source/config/tool content was inspected and no Cargo command or production
  mutation ran during initialization/activation.
- Next gate: fully reread all active documents and capture filename-only prospective fingerprints before current
  package/source/consumer discovery.

## 2026-08-11 — E02 Active Reread And Prospective Fingerprints

### Procedure

- Fully reread all three active Child documents.
- Used filename-only discovery and mechanical line/hash/status capture for prospective workspace/Core/server
  manifests, lock/config, Core roots and five bin roots, member-local workflow, server fixture examples and durable
  package/wire canonicals without opening their content.

### Activation state

- Expanded status remains 606 rows at canonical C-sort SHA-256 `1d7fae2a...`; Phase 44 is the sole active Child.
- Workspace/Core/server manifests and lockfile are clean at `98c9cf4d...`, `40d8a9ec...`, `19b41cd...` and
  `4223318b...`. Core `.cargo/config.toml` is clean, one line at `98a23201...`.
- Core `main.rs` is clean, one line at `536e506b...`; `lib.rs` is clean, 84 lines at `3fbfdca2...`.
- Five filename-discovered Core fixture bins are all clean: event 209 `29e3749c...`, encounter-compendium 67
  `3d3cc083...`, shop 114 `ee9b994e...`, inventory 74 `0d8c7b3b...` and research-delivery 48 `e0792e42...`.
- Member-local workflow is clean, 31 lines at `42462da0...`. Three server generator examples are clean except the
  already-dirty battle-revision carrier; their hashes are `6f196902...`, `845ccbe7...` and `b82a5e7b...`.
- README, engineering workflow, Core runtime and server wire canonicals are already dirty at `137ec7e5...`,
  `044d853b...`, `ab7d2f98...` and `c82000b1...`.
- Accessible adjacent-root filename discovery finds no sibling simulator/Unity project under `/mnt/f/work`; exact
  content/reference search remains discovery work, not a negative-consumer conclusion.
- Protected marker remains 57 bytes at `614d2fbd...` with recorded timestamps exact.

### Decision

All prospective package/config/Core target files are activation-clean, so any edit requires an exact Phase-owned
diff. Current Cargo metadata, manifest/source/config contents, dependency/feature graph and consumer references may
now be inspected. No historical candidate is presumed removable.

## 2026-08-11 — E03 Current Package, Source And Consumer Inventory

### Current graph

- Workspace metadata exposes one Core library, one empty implicit default binary, five implicit fixture binaries and
  five integration-test targets. The default and all fixture binaries currently have `test = true`; none contains a
  test. The server is the only workspace package depending on `game_core`.
- Core declares 38 normal and four build dependencies. There is no `build.rs`, so all four build edges are inert.
  Exact `src`/`tests` identifier inventory positively retains only `serde`, `serde_json`,
  `serde_path_to_error`, `rapier2d`, `uuid`, `rand`, `tracing` and `ron` after logger removal. All other direct
  edges have zero compiled-source/test references; the member coverage workflow installs its CLI independently.
- `event_log_checks` is the sole feature and has live `cfg` consumers, so it is retained. Core's local Cargo config
  unconditionally enables `tokio_unstable`, although Core has no Tokio source consumer once the unused dependency is
  removed.
- Core's `LoggerManager`, `init_logging` and disabled ctor are referenced only in Core `lib.rs`. Exact `game_core`
  logger import searches find no workspace consumer; the server owns and consumes a distinct server-local logger.

### Tool and accessible consumer inventory

- Exact target-name/default-bin searches find no current workspace caller outside historical goal records. No
  matching checked-in JSON fixture artifact exists in this repository.
- Exact searches across accessible sibling repositories under `/mnt/f/work/contract`, `familiy`, `mail` and
  `naver-trader` find no target command, Core logger import, default Core binary or `tokio_unstable` consumer.
- Standalone/external callers remain unknowable. Under CRA-POL-006 and the user's explicit package/tool breaking
  approval, retain all five generator names, paths, invocation shape and stdout semantics while making their Cargo
  target role explicit. Remove no generator implementation.

### Disposition

- Remove the empty default binary, inert build edges, zero-reference normal edges, unconsumed Core logger/disabled
  ctor and Core-local `tokio_unstable` config without a shim.
- Disable automatic binary discovery and declare the five retained tools explicitly with `test = false` and
  `bench = false`. Preserve the member coverage workflow, live feature, eight positive dependencies, tool source and
  output.

## 2026-08-11 — E04 Pre-Mutation Controls

- All candidate package/config/root/bin/workflow paths remain activation-clean. Expanded status is still 606 rows at
  canonical C-sort SHA-256 `1d7fae2a...`; protected marker remains 57 bytes at `614d2fbd...`.
- Metadata baseline is 38 normal + four build edges, one library + six test-enabled bins + five integrations and the
  retained `event_log_checks` feature. Core debug/release all-target checks and the feature-enabled debug all-target
  check pass.
- Canonically sorted `: test` inventory is 1,063 Core tests at `2db28d25...` and 63 server tests at
  `beca9e7a...`. The immediately preceding Phase 43 independently ran the complete debug/release suites with the
  same counts; no candidate path has changed since that evidence.
- Retained pre-change tool stdout controls are: event 6,839 bytes `54b3d90d...`; encounter compendium 815 bytes
  `ec9c851d...`; shop 3,101 bytes `88fb016e...`; inventory 2,225 bytes `9ec7f114...`; research delivery 456 bytes
  `f7c50711...`. Every output parses as JSON and its top-level key set was recorded.

### Gate decision

Consumer, metadata, source, feature, compile, test-inventory and exact tool-output controls support the bounded
cleanup. The next mutation may touch only Core manifest/lock/lib/main/config plus already-dirty durable documentation;
all five generator sources and the coverage workflow stay byte-exact.

## 2026-08-11 — E05 Bounded Package Cleanup

### Implemented delta

- `game_core` now sets `autobins = false`, has no default binary and declares exactly the five retained generator
  bins with their original names/paths plus `test = false` and `bench = false`.
- Removed all four inert build dependencies and 30 zero-reference normal dependencies, including the logger/ctor and
  Tokio/debug-console cluster. The manifest now contains exactly eight positive normal edges and retains the sole
  `event_log_checks` feature.
- Removed the unconsumed Core logger/init/no-op ctor implementation, leaving `lib.rs` as the library module root.
  Removed the empty `src/main.rs` and Core-local unconditional `tokio_unstable` config.
- Cargo regenerated the lock from 524 packages/5,244 lines to 311 packages/2,954 lines. No transitive package was
  manually forced. Current lock is `e1597f33...`.
- Added the current package/tool contract to the already-dirty durable engineering workflow. The Phase-owned
  subsection records library/server ownership, exact retained generator commands, non-test target role and opt-in
  feature policy; no inherited documentation content was rewritten.

### Preserved paths

All five generator roots remain byte-exact at their E02 hashes, and the member coverage workflow remains 31 lines at
`42462da0...`. Root workspace/server manifests and server generator examples are untouched.

## 2026-08-11 — E06 Post-Implementation Verification

### Structure and compatibility

- Post metadata reports one Core library, five explicit `test = false` bins, five integration tests, eight normal
  dependencies, zero build dependencies and the unchanged feature. There is no `game_core` default bin.
- Normal `cargo test -p game_core` passes all 1,063 tests with zero fixture/default-bin harness executions and five
  integration executions. `--all-targets` deliberately overrides target selection and still exercises explicit bin
  harnesses; this broader validation also passes, but is not the normal package test graph.
- Every retained generator stdout exactly matches its E04 byte count, SHA-256, JSON parse and top-level keys.
  Canonically sorted test inventories remain Core 1,063 at `2db28d25...` and server 63 at `beca9e7a...`.
- Removed logger/ctor/config identifier search is empty. Direct-edge `cargo tree` contains only rand, rapier2d, ron,
  serde, serde_json, serde_path_to_error, tracing and uuid.

### Executed validation

- Core all-target tests pass in debug and release: 1,012 library + 4 + 4 + 25 + 14 + 4 integration tests.
- Server all-target tests pass in debug and release: 63 library tests; existing server bin/examples remain zero-test
  targets outside this Phase.
- Workspace all-target checks pass in debug, release and locked debug modes. Core's `event_log_checks` all-target
  check passes. Core/server normal Clippy passes with inherited warnings; workspace fmt and `git diff --check` pass.

### Dirty-work accounting

- Exactly five activation-clean paths became status rows: modified `Cargo.lock`, modified Core manifest, deleted Core
  config, modified Core lib and deleted Core main. The durable engineering workflow was already dirty at activation.
- Expanded status is exactly 611 rows at canonical C-sort SHA-256 `c83e7bc0...`, the expected 606 + five. Protected
  marker remains 57 bytes at `614d2fbd...`.

### Gate decision

Implementation and proportional validation are complete. Phase status remains active/false until a separate full
Child reread and independent completion audit verifies scope, evidence, retained contracts and dirty attribution.

## 2026-08-11 — E07 Independent Completion Audit

- Fully reread all 258/194/72 Child lines and the 567-line completion-review guide before verdict. The audit first
  corrected an internal wording conflict so the preserved gameplay/domain API is distinct from the explicitly
  approved, verified-unconsumed logger/init package API removal.
- Locked metadata assertions, exact absence/retained-root searches, no-default/feature checks and five fresh
  disposable `/tmp` generator files all pass. Every generator byte/hash/key control is exact; only the created
  audit temp directory was removed afterward.
- Independent broad debug reruns pass Core 1,063/0/0 and server 63/0/0. Both-profile implementation evidence,
  Clippy, format/diff, lock explanation, 611-row pre-audit dirty attribution and marker remain current.
- The separate 210-line `COMPLETION_AUDIT.md` at SHA-256 `e27e8b23...` passes P44-C01~C12 and returns
  `Complete / High`, confidence `High`, improvement class `none`.
- The audit artifact alone changes expanded status from 611 to 612 rows at canonical C-sort SHA-256
  `c74c0615...`. Phase 44 is complete; the Master remains active/false until its distinct final audit.
