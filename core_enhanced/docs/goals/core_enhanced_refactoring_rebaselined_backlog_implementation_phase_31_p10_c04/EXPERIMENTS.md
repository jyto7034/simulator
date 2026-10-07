# Phase 31 CRA-P10-C04 Experiments

Status: complete
Implementation Complete: true

## 2026-08-11 — E01 Initialization And Evidence Freeze

### Hypothesis

Phase 31 can be initialized as the sole next candidate because Phase 30 exited `Complete / High`, the Master marks
CRA-P10-C04 `ready`, no implementation Child is active and the historical/final evidence defines a policy-complete
behavior-preserving boundary.

### Evidence reviewed

- the complete Goal authoring guide and complete Phase 30 post-correction completion audit;
- Master scope, Phase registry, initialization/migration/validation gates;
- original Phase 10 PLAN, candidate, boundary, test, experiment, note and completion records;
- post-Boss Phase 10 PLAN, candidate, boundary, test, experiment, note and completion records;
- final rebaselined catalog, exact linear order, test catalog, waves/gates and CRA-POL-006 decision.

The authoritative assignment is item 31: typed host snapshot overlay after Phase 30 route metadata, with the first
proof `game-state × active-run × compendium` and exact order/no-duplicates/fatal replacement.

### Exact initialization checks

- repository HEAD: `f318414fb0156a67e04c5479000c2a2683fc0c09`;
- expanded porcelain status before this Child: 527 rows;
- normalized C-sort SHA-256:
  `50611a48e9742620b40f47951efb6bb213b77c69b0fe661133603f86ff025d21`;
- no Phase 31 path existed before initialization;
- Master is active, Phase 30 is `complete — Complete / High`, Phase 31 is `ready`, and no implementation Child or
  registry row is active;
- protected marker remains 57 bytes at SHA-256
  `614d2fbdbdb0b29def7b90d1fdbc65148aba63339eea72fef5998d4a8ec22730` with unchanged mtime/ctime
  `2026-08-10 22:55:07.178423600 +0900`.

### Commands and tests

Initialization used Goal/history/registry/path/status evidence only. No new Phase-31-directed production search,
test runner, formatter or source/data/canonical read occurred. Prior Phase 30 overlap is explicitly non-baseline
input. Executed tests: 0; pass/fail/ignored: not applicable.

### Result and decision

Initialize the required Child documents around one typed server host overlay and exact old-output parity. Production
inspection remains forbidden until sole activation, full active-document/history reread and prospective path-only
fingerprinting close.

## Pending Evidence

- Master/Child sole activation and exact post-document status delta;
- complete activation reread and prospective content fingerprints;
- current typed Core snapshot, server wrapper, host merge/caller graph and dirty attribution;
- old-production complete overlay table and focused baseline;
- implementation, exact JSON parity, broad validation and independent completion audit.

## 2026-08-11 — E02 Sole Activation

### Hypothesis

The initialized Child can become the only active implementation Phase without authorizing Phase 32 work.

### Change and result

- All three Child headers changed from `initialized` to `active`; `Implementation Complete` remains `false`.
- Master Phase 31 changed from `ready` to `active`; Phase 30 remains `complete — Complete / High` and Phase 32
  remains `pending`.
- The three Child files explain the exact expanded-status increase from 527 to 530 rows. The normalized C-sort
  SHA-256 after initialization is `848bd871dc0a39088be4b789cfeb80391762a555dbca8fd70d9acb53ed51ee56`.
- The protected marker remains exact. No new Phase-31 source, test, data, canonical or Phase 32 path was inspected or
  changed.

### Commands and tests

Document, registry, status and marker checks only. Executed tests: 0; pass/fail/ignored: not applicable.

### Decision

Phase 31 is the sole active Child. Detailed production inspection remains gated on a complete reread of all three
active documents and path-only prospective fingerprints.

## 2026-08-11 — E03 Activation Reread And Prospective Fingerprints

### Procedure

- Reread all 302 PLAN, 81 experiment and 58 note lines after sole activation.
- Reconfirmed the complete Phase 30 audit and unchanged P10/final assignment evidence.
- Used historical/path discovery only to identify prospective Phase 31 source, test, persistence and canonical
  targets, then captured hashes and scoped status without printing new production contents.

### Prospective Core and test fingerprints

| Path | SHA-256 | Activation status |
| --- | --- | --- |
| `core_enhanced/src/game/behavior.rs` | `4d3131b11bbb453415c45f7370922e2f9c2f5e5bf3b57a2fd628598595653996` | modified |
| `core_enhanced/src/game/world/snapshot.rs` | `6e512c4d98f1af0a73d51d5dac786af297888773b7c67ad5411496efa49e5992` | modified |
| `core_enhanced/src/game/world/action_capability.rs` | `73fc548322a1a652dfaef79b6ce3ebff45b05d6082f90d55403bdc82e009024e` | modified |
| `core_enhanced/src/game/world/state.rs` | `1e5f91772f82531a7c90f9676cfda25af463b702cf26ae13f320cae8da413281` | modified |
| `core_enhanced/src/game/world/tests/snapshots_and_start.rs` | `2144e86ed59ddfeb669c8e19c910447368ad3e2a10e5111a923e25b2233c5162` | modified parent module |
| `core_enhanced/src/game/world/tests/combat.rs` | `08177bd514fa5836c47c4c8ce48efbb80a2b223a3efd1a5d0ac19203b48b384e` | modified parent module |

### Prospective server and persistence fingerprints

| Path | SHA-256 | Activation status |
| --- | --- | --- |
| `game_server/src/game/player_game_actor/handlers.rs` | `f05ec898c16af10cddecf80d12bcdffa8387777f70b9dd6001e1c7bb6f0ce12f` | modified |
| `game_server/src/game/player_game_actor/state.rs` | `18f31016d0269f0f6b3a7459da1e15fb53686cad8b6cfc8a61826e79d9f7f2b0` | modified |
| `game_server/src/game/player_game_actor/messages.rs` | `dee1e0484bad0d4f1cc13c7f34162df93972bec90425f395223a8ff1a05c3480` | modified |
| `game_server/src/game/player_game_actor/session.rs` | `bde6f11f1fd299bff57da5916b255a3d10fb48f453a08216e1c1fa4f1fd5cee1` | clean |
| `game_server/src/persistence/active_run_store.rs` | `a8a7394caebf39b6346735095623e3f8435ed816f030db30a38ec1ee819b6b6e` | modified |
| `game_server/src/persistence/encounter_compendium_store.rs` | `e79bf7243a4e4943a44e951ade18701b3a96326fb4081a232766b6810317292a` | clean |

### Prospective canonical/consumer fingerprints

| Path | SHA-256 | Activation status |
| --- | --- | --- |
| `core_enhanced/docs/core_runtime_contract.md` | `ab7d2f98040f15bfcb9b253ee06479b9989efc764fb57192447a23bc502c167e` | modified |
| `core_enhanced/docs/game_rulebook.md` | `6c53ecd5d0588aace1a2b043b31434d004327d60938ecf61066ac19acfd31951` | modified |
| `game_server/docs/unity_bridge_protocol.md` | `c9298eed52dcaa1fd68b0e76c98122eab57d498d4c0f98566d68f59487dcc757` | modified |
| `game_server/docs/game_server_core_bridge_goal.md` | `124439562ae177a725306a19071efc7f385a269c9e0f1e891a2099c0d5e7b2ad` | clean |
| external Unity `docs/unity_core_contract.md` | `a3224e244f9577fbb6a4d6d01e44b7ea85f3ba1969f656a22712b0ba972063f5` | outside root status |

### Repository and marker result

- HEAD remains `f318414fb0156a67e04c5479000c2a2683fc0c09`.
- Expanded activated document state remains 530 rows at normalized SHA-256
  `848bd871dc0a39088be4b789cfeb80391762a555dbca8fd70d9acb53ed51ee56`.
- The protected marker remains 57 bytes at SHA-256 `614d2fbd...` with exact timestamps.
- Phase 32 remains pending and absent.

### Commands and tests

Path discovery, `sha256sum`, scoped status, document/history reread and marker checks only. Executed tests: 0;
pass/fail/ignored: not applicable.

### Decision

The activation gate is closed. Detailed current-tree investigation may begin, but every dirty target must be
attributed before mutation and these activation fingerprints remain the preservation baseline.

## 2026-08-11 — E04 Current Graph And Old-Production 75-Row Safety Net

### Current ownership and caller graph

- Core `RunSnapshotDto` owns the complete typed gameplay root and public typed `action_capabilities` vector.
- Server `PlayerStateSnapshotDto` currently flattens that Core DTO and decorates only a selected combat-result event
  with its compressed log attachment.
- `PlayerGameActor::build_state_snapshot` is the single host composition authority and all six current snapshot
  publication sites call it. It serializes first, then mutates root `Value` keys for active-run status, compendium
  status, host capability append and fatal-compendium replacement.
- Active-run persistence is inspected only at `NotStarted`: missing emits an unavailable status, loaded current or
  backup appends `ContinueRun`, and load/lock failure appends `ResetActiveRun` with `system_issue` present.
- Compendium persistence is inspected in every game state: clean/recovered/dirty/fatal always emit the host object
  including a nullable `system_issue`; only `NotStarted` appends its recovery actions. Fatal title state clears all
  Core and active-run actions before appending retry then reset.
- Phase 30 `ActionKind` descriptors already own all four typed host identities and recovery scopes. Store state
  remains the availability authority.

### Test-first characterization

Added one old-production actor/store characterization covering the complete Cartesian product:

- game state: title, ordinary non-battle pre-run briefing, active live battle;
- active run: unconfigured, missing, loaded current, loaded backup, corrupt;
- compendium: unconfigured, clean, recovered from backup, dirty, fatal.

All 75 rows compare the complete final JSON value against the Core flattened root plus a declarative host oracle,
assert exact ordered capabilities and assert duplicate absence. Real stores construct missing/current/backup/
corrupt and clean/recovered/dirty/fatal outcomes.

### Commands and result

- The first command included libtest `--exact` with only the suffix, selected zero tests and is rejected as
  validation evidence: 0 passed, 0 failed, 0 ignored, 60 filtered.
- Corrected command:
  `cargo test -p game_server state_snapshot_overlay_matrix_preserves_old_full_json_order_and_uniqueness -- --nocapture`
- Result on the untyped production implementation: 1 passed, 0 failed, 0 ignored; 59 filtered in the server lib,
  0 selected in the binary target. The assertion inside the test confirms exactly 75 matrix rows executed.

### Decision

The required first safety net passes before production extraction. The smallest implementation boundary is a
server-private typed overlay beside `PlayerStateSnapshotDto`, with actor persistence reads producing typed status
inputs and one final serialization after explicit typed capability composition.

## 2026-08-11 — E05 Typed Overlay Implementation And Focused Parity

### Change

- Added private typed active-run and compendium snapshot status DTOs beside `PlayerStateSnapshotDto`. Their
  constructors encode the previously characterized valid field combinations and preserve the distinct omission
  rule for active-run `system_issue` versus nullable compendium `system_issue`.
- Added one private `HostSnapshotOverlay`. It receives the existing typed Core capability vector, preserves Core
  order, appends typed host recovery capabilities in the frozen order, and performs only the explicit fatal-title
  replacement.
- Host capability construction asserts Phase 30's exact `ActionRoute::HostRecovery` identity and obtains scope from
  the same `ActionKind` descriptor. It asserts duplicate absence in debug without introducing a release-mode
  deduplication or precedence policy.
- `build_state_snapshot` retains snapshot/compressed-log production and persistence read order, maps store states to
  the private typed overlay, then performs one final `serde_json::to_value` call. The former host root indexing,
  serialized capability push and serialized fatal clear paths were removed.
- Corrected the stale server canonical sentence that referred to `get_run_snapshot_json`; it now records the typed
  flattened Core-plus-host composition and one final serialization. No contract shape changed.

### Focused validation

- Initial post-edit compile found one local unmatched delimiter in `state.rs`; removed it before any passing result.
- Debug complete matrix after correction: 1 passed, 0 failed, 0 ignored; 59 filtered, with all 75 rows executed.
- Full server lib after typed implementation: 60 passed, 0 failed, 0 ignored.
- Added byte equality of the final serialized `Value` to the full-row oracle, then ran the release matrix: 1 passed,
  0 failed, 0 ignored; 59 filtered, all 75 rows executed.
- Targeted rustfmt and `cargo fmt --package game_server -- --check`: pass.

### Static ownership result

- Six production publication sites still converge on the single `build_state_snapshot`; the seventh search hit is
  the matrix test itself.
- Production has one `PlayerStateSnapshotDto` serialization in that builder and no remaining indexed host-field or
  serialized `action_capabilities` mutation.
- The only production host capability push and fatal clear are inside `HostSnapshotOverlay`; test-oracle mutations
  remain intentionally test-only.
- No fallback serializer, compatibility alias, public DTO migration or Phase 32 placeholder was added.

### Decision

Focused old/new parity and the one-owner static proof pass. Continue with complete debug/release Core/server gates,
canonical/consumer compatibility, final fingerprints and a separate completion audit.

## 2026-08-11 — E06 Broad Debug/Release Validation

### Behavior suites

| Command | Result |
| --- | --- |
| `cargo test -p game_core --all-targets` | 1,073 passed, 0 failed, 0 ignored: lib 1,022; integrations 4 + 4 + 25 + 14 + 4; zero-test bins reported separately |
| `cargo test -p game_core --release --all-targets` | 1,073 passed, 0 failed, 0 ignored with the same binary split |
| `cargo test -p game_server --all-targets` after final implementation correction | lib 60 passed, 0 failed, 0 ignored; main/examples selected zero |
| `cargo test -p game_server --release --all-targets` after final implementation correction | lib 60 passed, 0 failed, 0 ignored; main/examples selected zero |

The Core totals include the 25-test live RON loader/integration target. The server total includes the 75-row matrix,
root preservation, active-run missing/current/backup/corrupt lifecycle, compendium clean/recovered/dirty/fatal
lifecycle, active-battle attach, response/snapshot and persistence-order coverage.

### Compile, format and diagnostic gates

- `cargo check --workspace --all-targets`: pass after the final correction.
- `cargo check --workspace --release --all-targets`: pass after the final correction.
- `cargo fmt --all -- --check`: pass.
- scoped `git diff --check`: pass.
- `cargo clippy -p game_server --all-targets --no-deps` and the release equivalent: exit 0. They report only the
  established unrelated server warnings; no warning points to the Phase 31 overlay/status/composition code.
- `cargo clippy -p game_core --all-targets --no-deps` and the release equivalent: exit 0 with the existing Core
  diagnostic warning inventory (82 lib and 134 lib-test warnings, plus reported integration warnings).

### Strict-Clippy diagnostic attempts

- Workspace and server-with-dependencies `-D warnings` attempts stop on the existing Core 82-warning inventory.
- A server-only `--no-deps -D warnings` attempt initially exposed one Phase-31 `manual_map` warning plus established
  unrelated warnings. The active-run projection was corrected to `Option::map`; the focused matrix passed again.
- The repeated server-only strict attempt contains only seven established unrelated warnings/errors across lib and
  lib-test: `LoadBalanceActor` default, two persistence lock-file open options, old probe type complexity, old test
  fixture reassignment, session test placement and old compendium test expression. No Phase-31 overlay code remains
  in the strict output.
- These strict attempts are diagnostic non-gates; the approved Clippy gate is the warning-reporting baseline run,
  which exits 0 in debug and release. Unrelated cleanup is outside this candidate.

### Decision

All required behavior, compile, format and diagnostic gates pass in debug and release. No test failure or ignored
test remains; zero-test binaries/examples are not counted as behavior evidence.

## 2026-08-11 — E07 Final Compatibility, Consumer And Dirty Preservation

### Final fingerprints

| Path | SHA-256 | Phase 31 result |
| --- | --- | --- |
| `game_server/src/game/player_game_actor/handlers.rs` | `aa3ede7a03f38f040720c233f5b7782fbe4f7162b5812f14e20843a3a9f30d10` | typed store-state production plus 75-row test |
| `game_server/src/game/player_game_actor/state.rs` | `419babe34521987bae54bdc93e661c847dd04ae366fad59efe855cd0f0b59dab` | private typed overlay authority |
| `game_server/docs/unity_bridge_protocol.md` | `c82000b172892df78efe33b16b1bb268076b89483f6b006653c109e98edd3dd5` | corrected current typed composition sentence |
| Core behavior descriptor | `4d3131b11bbb453415c45f7370922e2f9c2f5e5bf3b57a2fd628598595653996` | unchanged from activation |
| Core action capability producer | `73fc548322a1a652dfaef79b6ce3ebff45b05d6082f90d55403bdc82e009024e` | unchanged from activation |
| Core runtime canonical | `ab7d2f98040f15bfcb9b253ee06479b9989efc764fb57192447a23bc502c167e` | unchanged |
| game rulebook | `6c53ecd5d0588aace1a2b043b31434d004327d60938ecf61066ac19acfd31951` | unchanged |
| server bridge canonical | `124439562ae177a725306a19071efc7f385a269c9e0f1e891a2099c0d5e7b2ad` | unchanged |
| external Unity canonical | `a3224e244f9577fbb6a4d6d01e44b7ea85f3ba1969f656a22712b0ba972063f5` | inspected, not changed |

### Accessible consumer evidence

External Unity source was read without modification. `GameSnapshotStore` consumes the flattened root capability
array and rejects duplicates; `ActionCapability.Parse` requires the exact typed action/scope/enabled/reason shape;
`EncounterCompendiumPersistenceModel` requires all five exact compendium fields and validates state/flag/null
semantics; `RunFlowScreenComposer` reads the optional active-run fields. Their final hashes are respectively
`f43c2df6...`, `5800c7b8...`, `ecba5f5a...` and `d09ff22e...`. The external repository was already heavily dirty
(2,161 rows, normalized hash `db1919da...`) and Phase 31 made no external write.

### Repository and protected marker

- Expanded main status remains exactly the activation set: 530 rows at normalized SHA-256
  `848bd871dc0a39088be4b789cfeb80391762a555dbca8fd70d9acb53ed51ee56`. No status path was added, removed or
  reclassified by implementation because all touched production/canonical paths were already dirty and the Child
  directory already explained the activation delta.
- Protected marker remains 57 bytes at SHA-256
  `614d2fbdbdb0b29def7b90d1fdbc65148aba63339eea72fef5998d4a8ec22730`, with mtime and ctime both
  `2026-08-10 22:55:07.178423600 +0900`.
- No RON, save/checkpoint/envelope, server message, public Core DTO, Unity source, Phase 32 path or unrelated file was
  changed.

### Decision

Implementation evidence is ready for a separate read-only completion audit. Child/Master status remains active
until that review independently returns `Complete / High`.

## 2026-08-11 — E08 Completion Audit And Closure

- The separate 342-line `COMPLETION_AUDIT.md` reread the final production, tests, persistence state machines,
  Phase-30 descriptor seam, canonicals, accessible Unity consumers and dirty boundary. Its SHA-256 is
  `7704d01389707e224118e0eb079b50b8edbf99a68a1d01ee82d73c5286d5e9ba`.
- All P31-C01~P31-C11 conditions pass. The Primary verdict is `Complete`, the Combined verdict is
  `Complete / High`, the long-term-fit class is High and no material or non-material correction finding remains.
- The audit artifact is the sole expected status-path addition: the expanded main worktree is exactly 531 rows at
  normalized C-sort SHA-256 `5b566e36b4107b89ea83a1ed84b7ff10a68ccad0017e930af073fd5ee0cf836d`.
- The protected marker remains 57 bytes at SHA-256
  `614d2fbdbdb0b29def7b90d1fdbc65148aba63339eea72fef5998d4a8ec22730`, with mtime and ctime both
  `2026-08-10 22:55:07.178423600 +0900`.
- Phase 31 closes with all three Child headers complete. Phase 32 becomes `ready` only; no Phase-32 Child,
  production inspection, test, placeholder or implementation is authorized by this closure.
