# Phase 03 CRA-P02-C01 Experiments

Status: complete
Implementation Complete: true

## 2026-08-09 — E00 Activation And Required-Document Initialization

### Authorization And Dependency

- The user's request to perform the Master remains active and authorizes the single active Child.
- Phase 02 CRA-F-C01 has a current `Complete / High` completion audit with improvement class `none`.
- Phase 03 is the next exact registry entry; Phase 04 and later work remain pending and unauthorized.
- Global CRA-POL-006 applies, but this candidate has no additional direct policy decision.

### Evidence Carried Into Initialization

- Candidate CRA-P02-C01 remains a test-first correctness correction with priority
  `2/3/2/1/3/3/unknown = 14/18`.
- Historical gap: publish raises host monotonic values only on the previous generation before backup rotation, then
  installs unchanged lower incoming metadata as current.
- Required first proof: real-file higher-current to lower-incoming loads, peak and both must retain the incoming Core
  payload while the installed current uses component-wise maxima.
- Preservation boundary: envelope/schema/checksum/current-backup-temp-lock-fsync/tombstone/failure behavior,
  incoming Core bytes and current public errors remain exact; no lease/conflict or migration semantics.

### Initialization Result

- Required Child documents initialized: `PLAN.md`, `EXPERIMENTS.md`, `EXPERIMENT_NOTES.md`.
- Master Phase 03 coordination state changes from `ready` to `active`; no other Child is active.
- Production source/status investigation: not started for this Phase.
- Test executions: none.
- Production/test/canonical mutation before initialization: none.

### Next Experiment

Read all three Child documents completely, capture the expanded dirty baseline and exact prospective target hashes,
then inspect the current store publish/read/rotation code and establish the nonzero focused baseline.

## 2026-08-09 — E01 Current Authority, Dirty Baseline And Focused Baseline

### Current Authority And Defect

- `game_server/src/persistence/active_run_store.rs` is the sole envelope/current-backup/temp/lock/checksum/tombstone
  authority. It was clean at Phase activation with SHA-256
  `d8f0a7f79efe0f0d0b875cf34fad51fee2ad64a83b9fad208d37d003eaa7bbdc`.
- `ActiveRunStore::publish` currently encodes and syncs the unchanged incoming checkpoint to temp first. If current
  is readable, it max-merges incoming host metadata only into the previous checkpoint before rotating that previous
  payload to backup. It then installs the unchanged incoming temp as current. The historical rollback defect is
  therefore still present exactly.
- `ActiveRunEnvelope` owns version 3 serialization/checksum. `load` prefers current and restores backup only when
  current is missing/invalid. The actor calls `publish` after successful non-Continue mutations; it does not merge
  host metadata itself.
- Correct target seam: derive a merged incoming checkpoint from readable current host metadata before encoding the
  temp/current envelope, while retaining the existing previous-payload backup merge and rotation.

### Dirty Baseline And Overlap

- Commit remains `f318414fb0156a67e04c5479000c2a2683fc0c09`.
- Post-Phase-document expanded status: 383 entries; SHA-256
  `0b596408194ddc37e66604c39ff91c8512d54c9a026d0aa5aab3fc75176fce7d`.
- The store source/test file is clean. Actor handlers are pre-existing modified (73+/13-, hash
  `63afec7f591c80ba78a369513e403571a5502712b8e7b1190705271d69021e6b`) for weapon/Boss/battle-revision work;
  this Phase does not need to edit them.
- Unity bridge canonical is pre-existing modified (17+/1-, hash
  `05cd588f42e098b43bc94a8f22ff77f6c93f0872c72f03deb0ce95f932ad1197`) including Phase 02 V3 graph prose and
  unrelated battle serialization prose. Any Phase 03 edit must be confined to its active-run monotonic paragraph.

### Baseline Command And Result

```text
cargo test -p game_server persistence::active_run_store::tests -- --nocapture
```

The store module executed 9 tests and passed 9 with 0 failed/ignored; 47 library tests were filtered. The main test
binary selected 0 and is not counted. Existing coverage protects backup recovery, corruption/version/checksum,
invalidation and publish failure, but contains no higher-current to lower-incoming current assertion.

### Next Experiment

Add three real-file tests for lower incoming loads, lower incoming peak and both, with distinct current/incoming Core
bytes and exact current plus recovery-generation assertions. Run them against unchanged production and record the
intended failures before implementation.

## 2026-08-09 — E02 Test-First Monotonic Publication Matrix

### Test Construction

- Added three real-file cases for load-count-only rollback, peak-only rollback and both dimensions together.
- Each case first publishes Core bytes `higher-current`, then publishes distinct bytes `lower-incoming` with lower
  selected host metadata, drops/reopens the store and reads the preferred current generation.
- Expected current is the incoming Core bytes with component-wise metadata maxima. The same case then corrupts
  current and expects backup recovery to keep the previous Core bytes with those same maxima.

### Pre-Implementation Command And Result

```text
cargo test -p game_server publish_merges_current_monotonic --lib -- --nocapture
```

All 3 selected tests failed, with 0 passed/ignored and 56 filtered. In every failure, installed current retained the
incoming Core bytes but exposed the lower incoming metadata: loads `1` instead of `3`, peak `200` instead of `500`,
or both. Failure occurred at the preferred-current assertion before backup recovery, exactly reproducing the named
defect rather than a fixture, checksum or filesystem error.

### Implementation Decision

Within `ActiveRunStore::publish`, read at most one readable current generation when the store is not invalidated;
clone the incoming checkpoint; raise only its two host monotonic fields from that previous generation; encode and
install that merged incoming envelope through the existing path; and rotate the previous Core payload to backup
with the same maxima. Do not interpret Core bytes, merge an invalidated run, add a recovery path or change files,
versions, checksums, fsync, errors or actor behavior.

## 2026-08-09 — E03 Store Correction And Focused Proof

### Implementation

- `ActiveRunStore::publish` now determines terminal invalidation first and reads at most one decoded current
  generation only when it belongs to the active run.
- It clones the incoming export, raises only `loads_used` and `run_peak_certified_power` from readable current, then
  encodes/validates/syncs that merged incoming envelope through the existing temp-to-current path.
- Backup rotation reuses the same decoded previous checkpoint, preserves its Core bytes and assigns the installed
  metadata maxima before the existing backup-temp replacement.
- Corrupt/missing current remains ignored for normal publish exactly as before; an invalidated old run is not read
  or merged. No backup-as-current fallback, lease, schema, checksum, file, fsync, actor or Core change was added.

### Focused Commands And Results

```text
cargo fmt --all -- --check
cargo test -p game_server publish_merges_current_monotonic --lib -- --nocapture
cargo test -p game_server persistence::active_run_store::tests -- --nocapture
```

- The three new cases passed 3/3 with 0 failed/ignored and 56 filtered.
- The complete store module passed 12/12 with 0 failed/ignored and 47 library tests filtered; the main binary
  selected zero and is excluded from evidence.
- Existing publish-failure, invalidation, backup recovery, checksum and unsupported-version tests all remain green.
- Formatting check passed.

### Canonical Alignment

The host canonical now states that every publish component-wise max-merges readable current host metadata into the
incoming envelope, preserves incoming Core bytes, and keeps the previous Core payload in backup with the same
maxima. It does not claim Core payload interpretation, migration or multi-writer conflict handling.

### Next Experiment

Run the active-run actor/store boundary, full server package and all-target check; then perform scoped status/diff/
hash, legacy and phase-exit completion audits.

## 2026-08-09 — E04 Broad Validation And Static Audit

### Broad Results

- `cargo test -p game_server active_run --lib -- --nocapture`: 16 passed, 0 failed/ignored, 43 filtered. This
  jointly exercises all 12 store tests and four actor active-run persistence/Continue boundaries.
- `cargo test -p game_server -q`: 59 library tests passed, 0 failed/ignored; main and doc binaries selected zero and
  are not counted.
- `cargo check -p game_server --all-targets`: passed.
- Final `cargo fmt --all -- --check` and affected-file `git diff --check`: passed.

### Static, Contract And Dirty Result

- The diff adds one pre-encode merge of the two existing host fields, reuses the existing decoded previous
  generation for backup, and adds the three real-file tests. No envelope field/version, file name, error, lock,
  invalidation, checksum, fsync, actor or Core source changed.
- Scoped search found no repair/rebind, dual version, migration, backup-as-normal-publish fallback, lease,
  compare-and-swap, ignored test or panic-capture path. The one existing bare `.is_err()` is the active
  `publish_failure_keeps_previous_current_readable` assertion and remains covered by the full module.
- Expanded final pre-audit status: 384 entries; SHA-256
  `176b966b384cc1b87723e4fcbd8944ae88b9137dd4503779d5b5ee88560bbad6`.
- Store source/test final hash: `a8a7394caebf39b6346735095623e3f8435ed816f030db30a38ec1ee819b6b6e`.
- Actor handler hash is unchanged from activation:
  `63afec7f591c80ba78a369513e403571a5502712b8e7b1190705271d69021e6b`.
- Canonical final hash: `4d734af0115a8c57e17bb8d43cd18af76452345df936633dde9c5fa188dd0219`;
  the Phase edit is confined to the active-run monotonic paragraph while Phase 02 and unrelated user prose remain.

### Next Experiment

Apply the completion review guide as a separate phase-exit pass. Do not activate Phase 04 unless every completion
row passes and the verdict is `Complete / High`.

## 2026-08-09 — E05 Completion Review And Exit

- The completion review guide was reread in full before a separate phase-exit pass.
- `COMPLETION_AUDIT.md` independently traces all eight completion conditions through current store/actor code,
  envelope and file contracts, real-file tests, canonical alignment, compatibility and dirty preservation.
- Audit verdict: `Complete / High`; improvement class: `none`; no material finding or policy question remains.
- Child status changes to `complete` and `Implementation Complete: true`.
- Phase 04 may become `ready`, but no Phase 04 production work is authorized until its required documents are
  initialized and read completely.
