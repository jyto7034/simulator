# Phase 30 CRA-P10-C01 Experiments

Status: complete
Implementation Complete: true

## 2026-08-11 — E01 Initialization And Evidence Freeze

### Hypothesis

Phase 30 can be initialized as the sole next candidate because Phase 29 exited `Complete / High`, the Master marks
CRA-P10-C01 `ready`, no other Child is active, and the historical/final evidence defines a policy-complete
behavior-preserving boundary.

### Evidence reviewed

- the complete Goal authoring guide and the direct Phase 29 completion audit;
- Master scope, exact Phase registry, initialization contract and migration/validation gates;
- original Phase 10 plan, boundary, candidate, test, experiments, notes and completion audit;
- post-Boss Phase 10 plan, boundary, candidate, test, experiments, notes and completion audit;
- final rebaselined catalog, exact linear order, test catalog, waves/gates and CRA-POL-006 decision.

### Exact initialization checks

- repository HEAD: `f318414fb0156a67e04c5479000c2a2683fc0c09`;
- expanded porcelain status: 522 rows;
- normalized C-sort SHA-256:
  `3ec521d59411b6a660dcd28f24510332f9725677ca4ddbebb2780657d6f4ef7c`;
- no Phase 30 path existed before initialization;
- Master was the only Goal with primary `Status: active`; no implementation Child or Master registry row was
  active;
- protected marker remained 57 bytes at SHA-256
  `614d2fbdbdb0b29def7b90d1fdbc65148aba63339eea72fef5998d4a8ec22730` with unchanged mtime/ctime
  `2026-08-10 22:55:07.178423600 +0900`.

### Commands and tests

Read-only document/path/status checks only. No production search, test runner, formatter or source/data/canonical
read occurred during initialization. Executed tests: 0; pass/fail/ignored: not applicable.

### Result and decision

The candidate is Phase 30 item 30: one typed route descriptor for owner/scope/battle-authority facts, with current
availability and host-recovery lifecycles retained. CRA-POL-006 closes the policy boundary. Initialize the three
required documents; sole activation and prospective fingerprinting remain mandatory before production inspection.

## Pending Evidence

- Master/Child sole activation and post-document status delta;
- complete activation reread and prospective content fingerprints;
- current command/capability/dispatch/server graph and dirty attribution;
- pre-change focused baseline and exhaustive characterization;
- implementation, focused/broad validation and independent completion audit.

## 2026-08-11 — E02 Sole Activation

### Hypothesis

The initialized Child can become the only active implementation Phase without authorizing any Phase 31 work.

### Change and result

- The three Child headers changed from `initialized` to `active`; `Implementation Complete` remains `false`.
- Master Phase 30 changed from `ready` to `active`; Phase 29 remains `complete — Complete / High` and Phase 31
  remains `pending`.
- The three Child files explain the exact expanded-status increase from 522 to 525 rows. The normalized C-sort
  SHA-256 after initialization is `9aa631b3fa562f8b568648d18d24774f3bebbd4aa02a6fabacb707773ccdfa20`.
- The protected marker remained exact. No source, test, data, canonical or Phase 31 path was inspected or changed.

### Commands and tests

Document, registry, status and marker checks only. Executed tests: 0; pass/fail/ignored: not applicable.

### Decision

Phase 30 is the sole active Child. Production inspection remains gated on a complete reread of all three active
documents and prospective path-only fingerprints.

## 2026-08-11 — E03 Activation Reread And Prospective Fingerprints

### Hypothesis

The activated candidate boundary and user-owned overlap can be frozen before any production content is opened.

### Procedure

- Reread all 287 PLAN, 76 experiment and 53 note lines after sole activation.
- Used filename discovery only to identify historical command/capability/dispatch/server/test/canonical targets.
- Captured SHA-256 and scoped Git status without printing file contents.

### Prospective Core source and test fingerprints

| Path | SHA-256 | Activation status |
| --- | --- | --- |
| `src/game/behavior.rs` | `65349de8...` | modified |
| `src/game/world.rs` | `c533e0ff...` | modified |
| `src/game/world/action_capability.rs` | `c8833bcd...` | clean |
| `src/game/world/battle_capability.rs` | `ccb696cb...` | modified |
| `src/game/world/snapshot.rs` | `6e512c4d...` | modified |
| `src/game/world/state.rs` | `1e5f9177...` | modified |
| `src/game/world/tests/combat.rs` | `08177bd5...` | no separate status row |
| `src/game/world/tests/snapshots_and_start.rs` | `2144e86e...` | no separate status row |
| `src/game/world/tests/mod.rs` | `c8d52c36...` | no separate status row |

### Prospective server source fingerprints

| Path | SHA-256 | Activation status |
| --- | --- | --- |
| `game_server/src/game/load_balance_actor/handlers.rs` | `1720aed7...` | clean |
| `game_server/src/game/load_balance_actor/messages.rs` | `6b283bc0...` | clean |
| `game_server/src/game/player_game_actor/handlers.rs` | `63afec7f...` | modified |
| `game_server/src/game/player_game_actor/messages.rs` | `dee1e048...` | modified |
| `game_server/src/game/player_game_actor/session.rs` | `bde6f11f...` | clean |
| `game_server/src/game/player_game_actor/state.rs` | `18f31016...` | modified |

### Prospective canonical/consumer fingerprints

| Path | SHA-256 | Activation status |
| --- | --- | --- |
| `core_enhanced/docs/core_runtime_contract.md` | `6e20d6a6...` | modified |
| `core_enhanced/docs/game_rulebook.md` | `6c53ecd5...` | modified |
| `game_server/docs/game_server_core_bridge_goal.md` | `12443956...` | clean |
| `game_server/docs/unity_bridge_protocol.md` | `4d734af0...` | modified |
| external Unity `docs/unity_core_contract.md` | `a3224e24...` | outside root status |

### Repository and marker result

- HEAD remains `f318414fb0156a67e04c5479000c2a2683fc0c09`.
- Expanded activation document state remains 525 rows at normalized SHA-256 `9aa631b3...`.
- The protected marker remains 57 bytes at SHA-256 `614d2fbd...` with unchanged timestamps.
- Phase 31 remains pending and absent.

### Commands and tests

Path discovery, `sha256sum`, scoped status and document reread only. Executed tests: 0; pass/fail/ignored: not
applicable.

### Decision

The activation gate is closed. Detailed current-tree investigation may begin, but every dirty target must be
attributed before mutation and all activation fingerprints must remain available for preservation proof.

## 2026-08-11 — E04 Current Graph, Baseline And Test-First Characterization

### Hypothesis

CRA-P10-C01 still exists on the activated tree, current observable behavior is green, and a complete descriptor
contract can be frozen before production ownership changes.

### Current graph result

- `ActionKind` has 56 variants. `scope()` separately groups four queries and six recoveries;
  `requires_battle_authority()` separately groups fourteen battle/VN mutations.
- Root projection repeats a 36-action common list. Its shared common preflight repeats a narrower migrated-owner
  list while payload/domain-owned inventory, Monolith conversion and Shop actions deliberately retain domain
  validation.
- Battle projection/evaluation repeats 18 actions, including two Visual Novel actions shared contextually with the
  common route. Core domain dispatch remains an exhaustive payload-routing match and is not removable metadata.
- Four host recovery commands are intercepted in server actor lifecycle code and their Recovery scope is repeated
  again at four snapshot capability construction sites.
- The exact behavior-preserving route model is therefore five-valued: Core common shared preflight, Core domain
  preflight with common projection, Core battle shared preflight, contextual common/battle VN, and host recovery.
  Scope and battle-authority are orthogonal facts on the same descriptor.

### Pre-change baseline

| Command | Passed | Failed | Ignored | Filtered |
| --- | ---: | ---: | ---: | ---: |
| Core `cargo test --lib game::behavior::tests --quiet` | 8 | 0 | 0 | 1,013 |
| Core exact state projection/revalidation test | 1 | 0 | 0 | 1,020 |
| Core exact map/checkpoint capability test | 1 | 0 | 0 | 1,020 |
| Core exact blocking-VN omitted-action no-mutation table | 1 | 0 | 0 | 1,020 |
| Core exact stale battle-authority no-mutation test | 1 | 0 | 0 | 1,020 |
| server `cargo test --lib --quiet` | 59 | 0 | 0 | 0 |

The baseline library inventory is 1,021 tests before the new characterization. No baseline product failure exists.

### Test-first change and result

- Added one test-only 56-row table in the already-dirty `behavior.rs` before any descriptor or production consumer
  change. It freezes one unique row per `ActionKind`, five route classes, exact scope, exact authority requirement,
  four host-owned identities and every current serialized enum name.
- Exact selection `cargo test --lib every_action_kind_has_one_characterized_route_scope_and_authority_contract
  --quiet`: 1 passed, 0 failed, 0 ignored, 1,021 filtered on duplicated old production.
- The test-only source hash after formatting is `adfedacf...`. Expanded status remains the activation 525-row set;
  no clean target acquired a status row.
- The first format check reported only two mechanical layouts inside the new table. They were corrected with a
  scoped patch; the rerun exits 0. This was not a product/test failure.

### Decision

The required characterization precedes production extraction. Implement one descriptor that preserves the five
observed routes, lets common/battle/host consumers derive membership/scope, and leaves payload/domain execution and
host recovery lifecycle distinct.

## 2026-08-11 — E05 Typed Descriptor Implementation

### Hypothesis

The five observed route classes, scope and battle-authority facts can be consolidated without moving availability,
payload validation, mutation, persistence or transport ownership and without changing observable ordering.

### Change

- Replaced the handwritten `ActionKind` declaration with one macro invocation that emits the same enum in the same
  order plus an internal exhaustive `ActionKind::ALL` vocabulary view.
- Added an exhaustive private `ActionDescriptor` table and minimally public, non-serialized `ActionRoute` and
  `HostRecoveryAction` types. Existing `scope()` and `requires_battle_authority()` now delegate to the descriptor.
- Changed common projection membership and common/battle preflight route checks to consume descriptor helpers.
  State-dependent evaluators and exhaustive payload/mutation dispatch remain untouched.
- Retained the historical 36-action `COMMON_ACTION_PROJECTION_ORDER` after detecting that raw enum order would alter
  the observable capability vector. Bidirectional debug assertions prove the ordered view and descriptor membership
  contain exactly the same commands.
- Added equivalent bidirectional parity between battle-routed descriptors and the existing 18-row battle evaluator.
- Changed the server actor to derive its four host recovery operations and their capability scopes from typed
  metadata. Compendium early return, pre-sync, reset-active-run and continue-run ordering remain exact.
- Updated only the nearest Core runtime and Unity bridge canonicals to describe this unchanged split of authority.

### Compatibility result

- All 56 `ActionKind` variants and serialized names are unchanged; `ActionScope`, `PlayerBehavior`, result/error,
  snapshot, RON, V3/save and checkpoint shapes are unchanged.
- `ActionRoute` and `HostRecoveryAction` are additive Rust-only types required by the separate server crate and have
  no serde derive. No existing public symbol was removed, narrowed or reinterpreted.
- `ActionKind::ALL` is internal. The final characterization compares its set with all 56 expected rows and verifies
  exact typed identity for all four host operations.
- No Phase 31 directory, source hook, overlay placeholder or future-route adapter exists.

### Result and decision

Implementation matches the frozen boundary. The discarded enum-order projection attempt was caught by design review
before broad validation and never became the retained behavior. Continue to focused, broad, static and dirty-tree
validation; do not mark the Phase complete before the separate audit.

## 2026-08-11 — E06 Implementation Validation And Preservation Evidence

### Focused test results

| Command/family | Passed | Failed | Ignored | Filtered |
| --- | ---: | ---: | ---: | ---: |
| final exhaustive route/scope/authority characterization | 1 | 0 | 0 | 1,021 |
| Core behavior module | 9 | 0 | 0 | 1,013 |
| state-transition capability projection/revalidation | 1 | 0 | 0 | 1,021 |
| common map/checkpoint capability and revalidation | 1 | 0 | 0 | 1,021 |
| blocking-VN omitted battle actions, no mutation | 1 | 0 | 0 | 1,021 |
| stale battle authority, no playback mutation | 1 | 0 | 0 | 1,021 |
| server library actor/persistence/recovery suite | 59 | 0 | 0 | 0 |

The characterization was rerun after the final `ActionKind::ALL` parity assertion and remains 1/0/0 with 1,021
filtered. No selected command produced zero tests.

### Broad test and compile results

| Target | Debug result | Release result |
| --- | --- | --- |
| Core full library | 1,022 passed, 0 failed/ignored | covered by all-target release |
| Core all targets | 1,073 passed, 0 failed/ignored | 1,073 passed, 0 failed/ignored |
| server all targets | 59 passed, 0 failed/ignored | 59 passed, 0 failed/ignored |
| Core `cargo check --all-targets` | exit 0 | exit 0 |
| server `cargo check --all-targets` | exit 0 | exit 0 |
| Core `cargo clippy --all-targets` | exit 0, established warnings only | exit 0, established warnings only |
| server `cargo clippy --all-targets` | exit 0, established warnings only | exit 0, established warnings only |

Core's behavioral total is library 1,022 plus integration suites 4 + 4 + 25 + 14 + 4 = 1,073. Six Core and four
server zero-test binary targets are compile evidence only and are not included as behavioral passes.

### Static, format and dirty-tree result

- Core and server `cargo fmt --all -- --check`: exit 0.
- Candidate-scoped `git diff --check`: exit 0.
- `common_owned` and server hardcoding of `ActionScope::Recovery` are absent from the affected consumers.
- One descriptor function owns route, scope and authority. Common/battle ordered evaluators remain only where they
  encode observable order or state-dependent availability and have bidirectional membership checks.
- Core direct host-behavior branches remain deliberately exhaustive InvalidAction payload dispatch; the server owns
  actual execution. This is not a fallback route.
- No Phase 31 path or production placeholder exists.

### Final fingerprints before completion audit

| Path | SHA-256 | Attribution |
| --- | --- | --- |
| `core_enhanced/src/game/behavior.rs` | `4d3131b11bbb453415c45f7370922e2f9c2f5e5bf3b57a2fd628598595653996` | descriptor + characterization in pre-existing dirty file |
| `core_enhanced/src/game/world/action_capability.rs` | `17d8ac5fedc457b5758573793257864e4d652b08458896b6bd141b532377faff` | newly dirty ordered common consumer |
| `core_enhanced/src/game/world/battle_capability.rs` | `c95635218a4b56a8a01be28813fc20354080c920f1f61fa2f62ec93e57a51389` | descriptor consumer in pre-existing dirty file |
| `game_server/src/game/player_game_actor/handlers.rs` | `f05ec898c16af10cddecf80d12bcdffa8387777f70b9dd6001e1c7bb6f0ce12f` | typed host consumer in pre-existing dirty file |
| `core_enhanced/docs/core_runtime_contract.md` | `ab7d2f98040f15bfcb9b253ee06479b9989efc764fb57192447a23bc502c167e` | current ownership canonical |
| `game_server/docs/unity_bridge_protocol.md` | `c9298eed52dcaa1fd68b0e76c98122eab57d498d4c0f98566d68f59487dcc757` | current host-boundary canonical |

Activation-clean `action_capability.rs` explains the sole status-set increase from 525 to 526 rows. All other
activation prospective targets that were not in implementation scope retain their activation hashes. Expanded
status is 526 rows at normalized C-sort SHA-256
`882d5f6bfdfe9c4e53fab15eaf99a478573cdba88fb622a8d6f44765db73f5d2`.

The protected marker remains exactly 57 bytes at SHA-256
`614d2fbdbdb0b29def7b90d1fdbc65148aba63339eea72fef5998d4a8ec22730`, with mtime and ctime both
`2026-08-10 22:55:07.178423600 +0900`. Its approval scope has not expanded.

### Failure classification and decision

- Product/test/build failures: none.
- Test-only mechanical format finding: corrected before production consolidation.
- Implementation design finding: enum declaration order was unsuitable as the existing public capability order;
  retained explicit order plus parity checks, with no observed-behavior change.
- Existing Clippy warnings: diagnostic baseline outside this candidate; no strict-warning claim and no unrelated
  cleanup performed.

All implementation validation is green. The next permitted action is the independent completion audit; Phase 31
remains unauthorized.

## 2026-08-11 — E07 Completion-Candidate Input Normalization

The mandatory full reread before audit found that PLAN still called the five-valued pre-change authority a current
three-valued authority and retained two initialization-era `discovery pending` phrases. Runtime, tests, canonical
and scope were already exact. The PLAN wording was corrected to distinguish pre-change authority, enumerate all
five implemented route classes and point to E04–E06 evidence. This is a Goal-evidence correction only; no
production, test, data, save, wire, consumer or Phase 31 path changed. The independent audit begins only after this
correction and a fresh full document reread.

## 2026-08-11 — E08 Audit-Rerun Count Correction

The fresh audit focused pass reran all six Core selections and the server library suite. All remain green, but it
showed that E06 had copied the pre-characterization 1,020-filtered count into four final one-test rows. The final
library contains 1,022 tests, so each current one-test selection correctly reports 1 passed and 1,021 filtered.
Those four documentation cells were corrected. The exhaustive characterization reports 1/0/0/1,021, the behavior
module 9/0/0/1,013 and server 59/0/0/0. No production correction was needed. The completion audit restarts from this
corrected evidence set.

## 2026-08-11 — E09 Completion-Audit Authority Correction

### Finding

The first source pass found a material distinction hidden by the existing debug parity checks: in
`get_action_capabilities`, the descriptor was consulted only by `debug_assert!`, while release execution still
iterated `COMMON_ACTION_PROJECTION_ORDER` directly. The 36-row view was necessary for exact output order, but it
therefore remained the effective membership authority in release, contrary to P30-C04.

### Correction

The runtime now derives the common action set from `ActionKind::ALL` filtered by
`route().projects_common_capability()`, then sorts that derived set by the historical ordered view. All current
actions have a known order and the bidirectional debug checks remain. Thus the descriptor owns membership in both
debug and release, while the array owns only the observable relative order. A future common route omitted from the
order view is still included at the end in release and fails the debug parity gate.

### Immediate validation

- format check: exit 0;
- exhaustive descriptor characterization: 1 passed, 0 failed/ignored, 1,021 filtered;
- state-transition capability projection: 1 passed, 0 failed/ignored, 1,021 filtered;
- common map/checkpoint projection/revalidation: 1 passed, 0 failed/ignored, 1,021 filtered;
- blocking-VN no-mutation: 1 passed, 0 failed/ignored, 1,021 filtered;
- stale battle-authority no-mutation: 1 passed, 0 failed/ignored, 1,021 filtered.

The corrected `action_capability.rs` SHA-256 is
`73fc548322a1a652dfaef79b6ce3ebff45b05d6082f90d55403bdc82e009024e`. The expanded status set remains the same
526 rows and normalized hash `882d5f6b...`; the marker and Phase 31 boundary remain unchanged. Because production
changed during review, the prior audit pass is discarded. A fresh full-document/source/static/focused/broad audit
is required before any verdict.

## 2026-08-11 — E10 Post-Correction Audit Candidate

### Fresh validation

- Core focused: characterization 1/0/0 with 1,021 filtered; behavior module 9/0/0 with 1,013 filtered; each of
  state-transition projection, map/checkpoint projection, blocking-VN no-mutation and stale-authority no-mutation
  1/0/0 with 1,021 filtered.
- Core `cargo test --all-targets --quiet`: 1,073 passed, 0 failed/ignored, composed of 1,022 library and
  4 + 4 + 25 + 14 + 4 integration tests; six zero-test binary targets excluded from behavior count.
- Core release all-targets: the same 1,073/0/0 composition.
- Server debug and release all-targets: 59 passed, 0 failed/ignored each; four zero-test targets excluded.
- Core/server all-target checks in debug/release: all exit 0.
- Core/server all-target Clippy in debug/release: all exit 0 with the established repository warning baseline;
  targeted diagnostic output contains no `action_capability.rs` warning.
- Core/server format and candidate-scoped diff checks: all exit 0.

### Fresh preservation/static result

- One exhaustive descriptor remains the only route/scope/battle-authority fact owner. Common runtime membership now
  derives from it in release; the ordered view only sorts. Battle state evaluation and server host lifecycle remain
  distinct current authorities.
- No scoped ignored/debug bypass, compatibility shim, dual route, fallback route or Phase 31 placeholder exists.
- Final corrected source/canonical hashes are `behavior.rs` `4d3131b1...`, `action_capability.rs`
  `73fc5483...`, `battle_capability.rs` `c9563521...`, server handlers `f05ec898...`, Core canonical
  `ab7d2f98...` and server canonical `c9298eed...`.
- Every non-mutated prospective runtime/test/canonical consumer retains its activation fingerprint, including
  `world.rs` `c533e0ff...`, snapshot `6e512c4d...`, world state `1e5f9177...`, server messages `dee1e048...`, server
  state `18f31016...`, rulebook `6c53ecd5...`, server bridge `12443956...` and external Unity canonical
  `a3224e24...`.
- The pre-artifact tree is still 526 rows at normalized SHA-256 `882d5f6b...`; the 57-byte protected marker remains
  exact at `614d2fbd...` and Phase 31 remains absent.

The corrected candidate is ready for a new completion audit artifact. E07–E09 remain transparent history and are
not reused as the final verdict.

## 2026-08-11 — E11 Completion Audit And Phase Exit

- The fresh 356-line post-correction `COMPLETION_AUDIT.md` returns `Complete / High`, fit `High`, improvement class
  `none`, with no material finding in the corrected candidate.
- The audit artifact alone explains the exact 526→527 status-row increase. The normalized C-sort SHA-256 is
  `50611a48e9742620b40f47951efb6bb213b77c69b0fe661133603f86ff025d21`.
- All P30-C01 through P30-C11 rows are complete/audit pass. This Child transitions from active/false to
  complete/true.
- The protected marker remains exact. Phase 31 was absent throughout implementation and audit; only its Master
  coordination state may now become `ready`.
