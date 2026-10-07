# Phase 35 CRA-P10-C07 Experiments

Status: complete
Implementation Complete: true

## 2026-08-11 — E01 Initialization And Evidence Freeze

### Hypothesis

Phase 35 can be initialized as the sole next candidate because Phase 34 exited `Complete / High`, the Master marks
CRA-P10-C07 `ready`, no implementation Child is active and historical/final evidence defines a policy-complete,
mechanical empty-artifact cleanup boundary.

### Evidence reviewed

- the complete 874-line Goal authoring guide;
- Master scope, global CRA-POL-006 compatibility invariant, Phase registry and activation/validation gates;
- original and post-Boss P10-C07 candidate, boundary, test and completion evidence;
- final/rebaselined candidate catalogs, exact linear order, test cleanup/rebaseline catalogs and cross-boundary
  gates;
- the completed Phase-34 exit boundary.

The authoritative assignment is item 35: remove only the empty disconnected Core test-module artifact after exact
metadata/test-list proof. It begins Stage D and has no technical dependency beyond linear Phase-34 completion.

### Exact initialization checks

- repository HEAD: `f318414fb0156a67e04c5479000c2a2683fc0c09`;
- expanded porcelain status before this Child: 543 rows;
- canonical C-sort SHA-256:
  `04f967cbbf1377c1d156530ffe7776d5e2b31d4f828b2efcd3f49cc0d6c07c9b`;
- no Phase-35 path existed before initialization;
- Master is active, Phase 34 is `complete — Complete / High`, Phase 35 is `ready`, and no implementation Child or
  registry row is active;
- the prospective target path exists and is clean by path/status-only checks; its content was not read;
- the protected marker remains exact.

### Commands and tests

Initialization used Goal/history/registry/path/status evidence only. No new Phase-35-directed target content,
module graph, Cargo metadata, test listing, source, data, canonical or external-consumer read/search occurred.
Executed tests: 0; pass/fail/ignored: not applicable.

### Result and decision

Initialize the required Child documents around exact disconnected-artifact proof, metadata/test-list equality,
minimal deletion and dirty preservation. Current-tree inspection remains forbidden until sole activation, complete
active-document reread and prospective fingerprints close.

## Pending Evidence

- Master/Child sole activation and exact post-document status delta;
- complete activation reread and prospective target/manifest fingerprints;
- current bytes/directory/module-edge/metadata/test-list characterization;
- minimal deletion, before/after equality and Core runner validation;
- independent completion audit.

## 2026-08-11 — E02 Sole Activation

### Change and result

- All three Child headers changed from `initialized` to `active`; `Implementation Complete` remains `false`.
- Master Phase 35 changed from `ready` to `active`; Phase 34 remains `complete — Complete / High` and Phase 36
  remains pending.
- The three Child files explain the exact expanded-status increase from 543 to 546 rows. The canonical C-sort
  SHA-256 after initialization is `f9f0b1131b3a1ef7595839e2857a21e2ac220fadcb4385a934f603ef96b48ef4`.
- The protected marker remains exact. No target content, module graph, Cargo metadata/test listing, source, data,
  canonical, external consumer or Phase-36 path was inspected or changed.

### Commands and tests

Document, registry, status and marker checks only. Executed tests: 0; pass/fail/ignored: not applicable.

### Decision

Phase 35 is the sole active Child. Detailed target inspection remains gated on a complete reread of all three active
documents, the predecessor audit and prospective path-only fingerprints.

## 2026-08-11 — E03 Activation Reread And Prospective Fingerprints

### Procedure

- Reread all 383 active Child lines and the complete 322-line Phase-34 completion audit after sole activation.
- Reconfirmed original/post-Boss P10-C07 evidence, the final/rebaselined catalogs, exact linear item 35 and test
  cleanup/rebaseline rules.
- Used filename-only discovery and path-known SHA-256/status checks to fingerprint the target, manifests, Core roots,
  integration roots and fixture-bin sources without reading target content or running metadata/tests.

### Prospective fingerprints

| Path | SHA-256 | Activation status |
| --- | --- | --- |
| `core_enhanced/tests/test_actor/mod.rs` | `7eb70257593da06f682a3ddda54a9d260d4fc514f645237f5ca74b08f8da61a6` | clean |
| root `Cargo.toml` | `98c9cf4dfefe3eda0ff7c692af94666a491a3460f073b3022c4c360b1a3e849c` | clean |
| root `Cargo.lock` | `4223318b9d62b4b1ecfe966543795a00445b938add11cae9413bbe6fd54d45da` | clean |
| `core_enhanced/Cargo.toml` | `40d8a9ec74b9c36862fbb580ad4bef554be03ab27875ceb9dadf9db93b5929af` | clean |
| `core_enhanced/src/lib.rs` | `3fbfdca2626de2a511ef6ffbbefe798b93f5469b79e63bfd135fa4cbc3dc8017` | clean |
| `core_enhanced/src/main.rs` | `536e506bb90914c243a12b397b9a998f85ae2cbd9ba02dfd03a9e155ca5ca0f4` | clean |
| `tests/live_item_skill_activation.rs` | `84e4e600ba8816fed6ec888353bcc7c98ecd4774b17fef4969897ace4e27c61b` | modified before Phase 35 |
| `tests/live_skill_catalog_audit.rs` | `bf475aa3fb2e6fb81fc28a291cfaab8b81b937fb492a8ed8af27480a7f358d8a` | clean |
| `tests/ron_loading.rs` | `75558ff28be83cafcfe535252f552903b2077249c2cd81ef00b41d63195854b3` | modified before Phase 35 |
| `tests/skill_runtime_contract.rs` | `afd09b3f6392e8c0175a10d5be3781638e6221c956a55e8c4d129f80c1b49c58` | modified before Phase 35 |
| `tests/skill_test_suite.rs` | `4f2186b161f61c9192b6021895bbc3638e2444964ded36f9b9972dac82d835b2` | clean |

The complete 18-path `core_enhanced/tests` filename+content fingerprint is
`f1654222464883fd373a1815044dd198b9ddcfead0c5b71efed9d529a3ad0212`. The five fixture-bin source fingerprint is
`1efd1eb31076954990c0ed899af40e49846626ddf32c73e684665d26e9d3a5f8`. Existing additional dirty test rows are
`tests/common/mod.rs` and `tests/skill_test/common/scenario.rs`; they remain preservation-only.

### Activation preservation

- HEAD remains `f318414fb0156a67e04c5479000c2a2683fc0c09`.
- Expanded main status remains exactly 546 rows at canonical C-sort SHA-256
  `f9f0b1131b3a1ef7595839e2857a21e2ac220fadcb4385a934f603ef96b48ef4`.
- The target and all three manifests/lock inputs are clean. Phase 35 is the sole active row; Phase 36 remains pending
  and absent.
- The protected marker remains byte/hash/time exact.

### Commands and tests

Only active-document/history reads, filename discovery, path-known hashes, scoped status and marker checks were
used. Executed tests: 0; pass/fail/ignored: not applicable.

### Decision

The activation gate is closed. Current target content, directory membership, module reachability, Cargo metadata and
test-list characterization may now begin.

## 2026-08-11 — E04 Current Disconnection And Pre-Removal Graph Characterization

### Target proof

- `core_enhanced/tests/test_actor/mod.rs` is exactly 2 bytes, hexadecimal `0d 0a` (CRLF only), at its activation
  SHA-256 `7eb70257593da06f682a3ddda54a9d260d4fc514f645237f5ca74b08f8da61a6`.
- Filename discovery including ignored/hidden entries finds exactly one directory member: that `mod.rs`.
- Exhaustive Rust/TOML search finds no `mod test_actor`, `tests/test_actor`, `test_actor/mod`, `include*` or
  `path = ...test_actor` edge. The target is clean in Git status.

### Cargo metadata baseline

`cargo metadata --format-version 1 --no-deps` exits 0. Raw output SHA-256 is
`047cb60bb2122672c0b019d3856ff65f14bfa8a54f532ff7e66296fe19ed7127`. `game_core` exposes 12 targets:

- one library and the existing empty default binary;
- five fixture-generator binaries;
- five actual integration tests: `live_item_skill_activation`, `live_skill_catalog_audit`, `ron_loading`,
  `skill_runtime_contract` and `skill_test_suite`.

No target path/name contains `test_actor`. Every target's kind, crate type, source path, edition, required features,
doctest/test/doc flags is stored as the exact before-removal comparison table.

### Complete test-list and runner baseline

- `cargo test -p game_core --all-targets -- --list` exits 0 and reports 1,077 test names. The canonical C-sort name
  SHA-256 is `be0d705e1d8aadc3fbf669fc29372333fe5c42a8d309260fc33ea0720a7b3c38`.
- Harness summaries are Core library 1,026; default main and five fixture bins 0 each; integrations
  4 + 4 + 25 + 14 + 4. The zero-test harnesses are retained Phase-44 facts and excluded from behavior counts.
- Pre-removal `cargo test -p game_core --all-targets` passes all 1,077 behavioral tests, 0 failed, 0 ignored, with
  the same 1,026 + 4 + 4 + 25 + 14 + 4 distribution. Every zero-test harness reports 0/0/0.

### Decision

All deletion preconditions are currently proven. Remove only the two-byte file with `apply_patch`, then remove the
directory only after confirming it is empty. Do not touch manifests, real tests, default main or fixture bins.

## 2026-08-11 — E05 Minimal Removal And Exact Graph Equality

### Implementation

- Deleted only `core_enhanced/tests/test_actor/mod.rs` with `apply_patch`. The candidate diff is one deleted 2-byte
  CRLF line and no other path.
- `rg --files -uu core_enhanced/tests/test_actor` then returned no member (exit 1); only after that proof, `rmdir`
  removed the empty directory. No compatibility stub, ignored test, replacement test, manifest edit or Phase-44
  package cleanup was introduced.

### Before/after equality

- Post-removal `cargo metadata --format-version 1 --no-deps` exits 0 and is byte-for-byte equal to the retained
  pre-removal output. Its SHA-256 therefore remains
  `047cb60bb2122672c0b019d3856ff65f14bfa8a54f532ff7e66296fe19ed7127`.
- The normalized 12-row `game_core` target table is structurally exact before/after, including target name, kind,
  crate types, source path, edition, required features and doctest/test/doc flags. No row contains `test_actor`.
- Post-removal `cargo test -p game_core --all-targets -- --list` exits 0. Its sorted 1,077-name table is exactly equal
  to the retained pre-removal table, so the canonical SHA-256 remains
  `be0d705e1d8aadc3fbf669fc29372333fe5c42a8d309260fc33ea0720a7b3c38`.
- The post-removal test tree has 17 files. Its filename+content aggregate is
  `89ffe90e93e7da1a679d851db5e6094e9412134b8be8fdbfea3ede18c2dc1200`; the only activation-tree membership change
  is the proven candidate deletion.

### Preservation

- All activation manifests/lock, Core roots, five real integration roots and five fixture-bin sources retain their
  exact prospective SHA-256 values. In particular, already-dirty integration/support tests were not edited.
- Expanded main status is 547 rows at canonical C-sort SHA-256
  `715aae8024e710fbb3b592b633b15cec9f4fb1d7fe9b0bd1472a19471ad629e1`. Relative to the 546-row activation
  baseline, the sole expected new row is ` D core_enhanced/tests/test_actor/mod.rs`; no Phase-36 path exists.
- The protected marker remains 57 bytes at SHA-256
  `614d2fbdbdb0b29def7b90d1fdbc65148aba63339eea72fef5998d4a8ec22730`, with mtime/ctime unchanged at
  `2026-08-10 22:55:07.178423600 +0900`.

### Decision

The implementation is candidate-complete: Cargo never discovered the deleted artifact, and exact target/test-name
equality proves that removing it did not narrow behavioral coverage. Broad validation and an independent completion
audit remain required before status closure.

## 2026-08-11 — E06 Focused And Broad Validation

### Runtime and build gates

- Post-removal `cargo test -p game_core --all-targets` passes 1,077 tests, 0 failed, 0 ignored: library 1,026 and
  integrations 4 + 4 + 25 + 14 + 4. The default main and five fixture binaries each remain explicitly classified
  0-test Phase-44 targets.
- `cargo test -p game_core --all-targets --release` passes the same 1,077/0/0 distribution.
- `cargo check --workspace --all-targets` and `cargo check --workspace --all-targets --release` both exit 0.
- `cargo fmt --all -- --check` and candidate-scoped `git diff --check` both exit 0.
- Normal `cargo clippy -p game_core --all-targets --message-format short` and its release counterpart both exit 0
  with the inherited baseline: 82 library warnings and 134 library-test warnings (82 duplicates), plus the existing
  integration-target summaries. No warning is attributable to the deleted non-target file.

### Proportionate omissions

Separate server behavior, Unity, WebSocket, performance, data-only and canonical migrations remain omitted exactly
as planned. Byte-identical Cargo metadata proves the candidate did not change the compiled target graph; Core
all-target runs already include live-RON integration tests. Stage-D repository-wide validation remains a later
Master boundary.

### Decision

All implementation-time focused and broad gates are green. The separate completion audit must now reread the full
audit guide, rerun the decisive metadata/list/status/marker checks and evaluate every P35-C01..P35-C10 row without
relying on this implementation conclusion.

## 2026-08-11 — E07 Audit Readback Correction And Restart

The first audit readback found one bounded documentation mismatch: E06 named `P35-C01..P35-C12`, while the PLAN's
completion matrix has exactly `P35-C01..P35-C10`. Corrected that reference to C10. No candidate, production, test,
metadata, status-baseline or validation evidence changed. Per the completion-review guide, the independent audit is
restarted after this correction and will reread the corrected Child evidence before issuing a verdict.

## 2026-08-11 — E08 Completion Audit And Phase Exit

- The restarted independent 212-line audit at SHA-256
  `d93c81c4c598230997434bfba7ad7cbd6a57c9ce3c78c685622696c37fb92ee9` returns `Complete / High`, fit `High`,
  improvement class `none`, with every P35-C01..P35-C10 row passing and no material finding.
- Audit-time raw/normalized Cargo metadata remains exact to the pre-removal artifact, and the exact sorted 1,077-name
  test table remains equal. A fresh audit Core runner again passes 1,077/0/0.
- The audit artifact explains the exact 547→548 status increase. Canonical C-sort SHA-256 is
  `9c03cb1c82f8acdd41686da5291ac30222aa155aa810d28ed7c389b3f4690172`; the protected marker remains byte/hash/time
  exact and no Phase-36 path exists.
- Phase 35 transitions to `complete` with `Implementation Complete: true`. Master Phase 36 may become `ready` only;
  no Phase-36 initialization, inspection or implementation is included in this exit.
