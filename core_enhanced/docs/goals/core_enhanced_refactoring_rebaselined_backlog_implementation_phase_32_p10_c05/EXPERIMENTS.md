# Phase 32 CRA-P10-C05 Experiments

Status: complete
Implementation Complete: true

## 2026-08-11 — E01 Initialization And Evidence Freeze

### Hypothesis

Phase 32 can be initialized as the sole next candidate because Phase 31 exited `Complete / High`, the Master marks
CRA-P10-C05 `ready`, no implementation Child is active and the historical/final evidence defines a policy-complete
behavior-preserving boundary.

### Evidence reviewed

- the complete 874-line Goal authoring guide and complete 342-line Phase-31 completion audit;
- Master scope, Phase registry, initialization/migration/validation gates;
- original Phase-10 PLAN, candidate, boundary, test, experiment, note and completion records;
- post-Boss Phase-10 PLAN, candidate, boundary, test, experiment, note and completion records;
- final rebaselined catalog, exact linear order, test catalog, waves/gates, cross-boundary evidence and
  CRA-POL-006 decision.

The authoritative assignment is item 32: one stable `GameError` code authority, with exhaustive Core/server code
equality and retained context-specific messages as the first proof.

### Exact initialization checks

- repository HEAD: `f318414fb0156a67e04c5479000c2a2683fc0c09`;
- expanded porcelain status before this Child: 531 rows;
- normalized C-sort SHA-256:
  `5b566e36b4107b89ea83a1ed84b7ff10a68ccad0017e930af073fd5ee0cf836d`;
- no Phase-32 path existed before initialization;
- Master is active, Phase 31 is `complete — Complete / High`, Phase 32 is `ready`, and no implementation Child or
  registry row is active;
- protected marker remains 57 bytes at SHA-256
  `614d2fbdbdb0b29def7b90d1fdbc65148aba63339eea72fef5998d4a8ec22730` with unchanged mtime/ctime
  `2026-08-10 22:55:07.178423600 +0900`.

### Commands and tests

Initialization used Goal/history/registry/path/status evidence only. No Phase-32-directed production search, test
runner, formatter or source/data/canonical read occurred. Historical and prior-Phase overlap is explicitly
non-baseline input. Executed tests: 0; pass/fail/ignored: not applicable.

### Result and decision

Initialize the required Child documents around one Core stable-code authority and exact old-output parity.
Production inspection remains forbidden until sole activation, full active-document/history reread and prospective
path-only fingerprinting close.

## Pending Evidence

- Master/Child sole activation and exact post-document status delta;
- complete activation reread and prospective content fingerprints;
- current `GameError`, Core snapshot, server adapter, message/detail and consumer graph;
- old-production exhaustive parameterized code table and focused baseline;
- implementation, exact parity, broad validation and independent completion audit.

## 2026-08-11 — E02 Sole Activation

### Hypothesis

The initialized Child can become the only active implementation Phase without authorizing Phase 33 work.

### Change and result

- All three Child headers changed from `initialized` to `active`; `Implementation Complete` remains `false`.
- Master Phase 32 changed from `ready` to `active`; Phase 31 remains `complete — Complete / High` and Phase 33
  remains `pending`.
- The three Child files explain the exact expanded-status increase from 531 to 534 rows. The normalized C-sort
  SHA-256 after initialization is `7f12bd4a6f2a00a1f751b60940f19ad37aa9c622ae03b9f4c100466a6a510d6c`.
- The protected marker remains exact. No Phase-32 source, test, data, canonical or Phase-33 path was inspected or
  changed.

### Commands and tests

Document, registry, status and marker checks only. Executed tests: 0; pass/fail/ignored: not applicable.

### Decision

Phase 32 is the sole active Child. Detailed production inspection remains gated on a complete reread of all three
active documents and path-only prospective fingerprints.

## 2026-08-11 — E03 Activation Reread And Prospective Fingerprints

### Procedure

- Reread all 292 PLAN, 82 experiment and 59 note lines after sole activation.
- Reconfirmed the complete Phase-31 audit and unchanged original/post-Boss/final P10-C05 assignment evidence.
- Used symbol path discovery only to identify prospective Phase-32 source, test and canonical targets, then captured
  hashes and scoped status without printing production contents.

### Prospective Core fingerprints

| Path | SHA-256 | Activation status |
| --- | --- | --- |
| `core_enhanced/src/game/behavior.rs` | `4d3131b11bbb453415c45f7370922e2f9c2f5e5bf3b57a2fd628598595653996` | modified before Phase 32 |
| `core_enhanced/src/game/world/snapshot.rs` | `6e512c4d98f1af0a73d51d5dac786af297888773b7c67ad5411496efa49e5992` | modified before Phase 32 |
| `core_enhanced/src/game/world/action_capability.rs` | `73fc548322a1a652dfaef79b6ce3ebff45b05d6082f90d55403bdc82e009024e` | modified before Phase 32 |
| `core_enhanced/src/game/world/battle_capability.rs` | `c95635218a4b56a8a01be28813fc20354080c920f1f61fa2f62ec93e57a51389` | modified before Phase 32 |
| `core_enhanced/src/game/world/state.rs` | `1e5f91772f82531a7c90f9676cfda25af463b702cf26ae13f320cae8da413281` | modified before Phase 32 |
| `core_enhanced/src/game/world/tests/snapshots_and_start.rs` | `2144e86ed59ddfeb669c8e19c910447368ad3e2a10e5111a923e25b2233c5162` | modified before Phase 32 |
| `core_enhanced/src/game/world/tests/combat.rs` | `08177bd514fa5836c47c4c8ce48efbb80a2b223a3efd1a5d0ac19203b48b384e` | modified before Phase 32 |

### Prospective server and canonical fingerprints

| Path | SHA-256 | Activation status |
| --- | --- | --- |
| `game_server/src/game/player_game_actor/state.rs` | `419babe34521987bae54bdc93e661c847dd04ae366fad59efe855cd0f0b59dab` | modified before Phase 32 |
| `game_server/src/game/player_game_actor/handlers.rs` | `aa3ede7a03f38f040720c233f5b7782fbe4f7162b5812f14e20843a3a9f30d10` | modified before Phase 32 |
| `game_server/src/game/player_game_actor/messages.rs` | `dee1e0484bad0d4f1cc13c7f34162df93972bec90425f395223a8ff1a05c3480` | modified before Phase 32 |
| `game_server/src/game/player_game_actor/session.rs` | `bde6f11f1fd299bff57da5916b255a3d10fb48f453a08216e1c1fa4f1fd5cee1` | clean |
| `core_enhanced/docs/core_runtime_contract.md` | `ab7d2f98040f15bfcb9b253ee06479b9989efc764fb57192447a23bc502c167e` | modified before Phase 32 |
| `core_enhanced/docs/game_rulebook.md` | `6c53ecd5d0588aace1a2b043b31434d004327d60938ecf61066ac19acfd31951` | modified before Phase 32 |
| `game_server/docs/unity_bridge_protocol.md` | `c82000b172892df78efe33b16b1bb268076b89483f6b006653c109e98edd3dd5` | modified before Phase 32 |
| `game_server/docs/game_server_core_bridge_goal.md` | `124439562ae177a725306a19071efc7f385a269c9e0f1e891a2099c0d5e7b2ad` | clean |

The historical `core_snapshot_error_contract_cleanup` Goal was also identified without reading its contents; its
four artifact hashes are `5f45ed78...`, `ab2977d4...`, `6537367b...` and `363fdbad...`. It will be treated as
history/evidence, never as current runtime authority.

### Activation preservation

- HEAD remains `f318414fb0156a67e04c5479000c2a2683fc0c09`.
- Expanded status remains exactly 534 rows at normalized C-sort SHA-256
  `7f12bd4a6f2a00a1f751b60940f19ad37aa9c622ae03b9f4c100466a6a510d6c`.
- Phase 32 is the sole active registry row; Phase 33 remains pending and its Child path is absent.
- The protected marker remains byte/hash/mtime/ctime exact.

### Commands and tests

Only active-document/history reads, path-only symbol discovery, SHA-256, scoped status and marker checks were used.
Executed tests: 0; pass/fail/ignored: not applicable.

### Decision

The activation gate is closed. Detailed current-tree source/history/canonical inspection and a focused old-production
baseline may now begin. Every likely source/test/canonical target is either fingerprinted as pre-existing dirty work
or explicitly recorded clean before the first content read.

## 2026-08-11 — E04 Current Authority, Dirty Attribution And Baseline

### Current source result

- `GameError` currently has 26 semantic variants. Twenty-five have fixed codes; `ActionUnavailable` projects the
  exact `ActionDisabledReasonCode::as_str()` value across 41 current disabled reasons.
- Core snapshot construction owns one private exhaustive code match and a separate exhaustive contextual-message
  match. The server owns one exhaustive combined match that independently repeats the same code literals while
  intentionally using different English messages.
- Therefore the historical “two Core snapshot helpers” wording describes the code and message helpers together;
  there is one Core code projection plus one server code projection. The candidate remains current because stable
  code identity is duplicated across those two contract producers.
- Snapshot codes are used in `effective_profile_error_dto`; server codes flow through every
  `PlayerGameActorError::from(GameError)` command/authority error path. Existing tests cover only weapon/profile,
  static-obstacle and two disabled-reason representatives.
- Current static comparison finds no code divergence. Message wording and authority values differ by design and
  remain separate.

### Historical/current boundary

The earlier `core_snapshot_error_contract_cleanup` Goal introduced `StaticObstacleBlocked`, retained
`PositionOccupied` for real occupancy and added both current code literals. It also repaired snapshot-root
composition. It did not centralize the complete stable-code vocabulary and does not close CRA-P10-C05.

### Dirty attribution

- `behavior.rs` already contains large Phase-19/30 and other campaign hunks; its current `GameError` area has the
  pre-existing `WeaponRequired` addition.
- `snapshot.rs` has small pre-existing error/profile and snapshot changes, including the matching weapon code/message.
- server `state.rs` has Phase-31 typed-overlay work and a pre-existing weapon mapping/test hunk.
- The Phase-32 changes must preserve those hunks and limit overlap to adding the central method, adding the complete
  characterization, removing only the Core code helper and removing only server tuple code literals.

### Focused pre-change baseline

| Command | Result |
| --- | --- |
| `cargo test -p game_core --lib employee_roster_snapshot_surfaces_effective_profile_errors -- --nocapture` | 1 passed, 0 failed, 0 ignored; 1,021 filtered |
| `cargo test -p game_server player_game_actor::state::tests -- --nocapture` | lib 8 passed, 0 failed, 0 ignored; 52 filtered; main selected zero |

The baseline ran on the current dirty tree before any Phase-32 test or production edit. Both commands execute
nonzero contract cases; the zero-test server main target is not counted as behavior evidence.

### Decision

The smallest target is `GameError::code(&self) -> &'static str` or equivalent exhaustive method in the Core carrier,
consumed by snapshot and server mappings. First add and pass two old-production characterization tables: 25 fixed
variant rows plus all 41 `ActionUnavailable` reasons (66 error rows per layer), with exact contextual messages and
structured parameter examples.

## 2026-08-11 — E05 Old-Production Exhaustive Characterization

### Test-first change

- Added a Core snapshot table with 25 fixed-variant rows and all 41 disabled reasons. Each row asserts the exact
  code, Core contextual message and serialized `{code,message}` object; total is asserted as 66.
- Added an independent server table over the same 25 + 41 categories. Each row asserts exact code, server contextual
  message and fully serialized error message including request correlation and the current lowercase wire tag.
- Parameterized rows use nontrivial battle authorities, resource/stat/static-data/feature values and two fragment
  failure codes, so loss of message/detail inputs is observable.

### Old-production results

| Command | Result |
| --- | --- |
| `cargo test -p game_core --lib snapshot_error_projection_characterizes_every_game_error_code_and_message -- --nocapture` | 1 passed, 0 failed, 0 ignored; 1,022 filtered; 66 internal rows |
| first server characterization run | 1 failed; the new expectation incorrectly used `"Error"` while existing serde emits `"error"` |
| `cargo test -p game_server server_error_projection_characterizes_every_game_error_code_and_message -- --nocapture` after correcting only the test expectation | lib 1 passed, 0 failed, 0 ignored; 60 filtered; 66 internal rows; main selected zero |

The first server failure characterized an existing tag rather than a production defect. No production code changed
before both complete tables passed on the duplicated mapping implementation.

### Decision

The tables prove all current codes agree and freeze both distinct message vocabularies plus serialization. Proceed
with one exhaustive Core `GameError::code()` and replace only the two duplicate code projections.

## 2026-08-11 — E06 One Stable-Code Authority And Focused Parity

### Production change

- Added one exhaustive `GameError::code(&self) -> &'static str` authority beside the semantic Core error. Its 26
  arms preserve the 25 fixed code literals and delegate only `ActionUnavailable` to the existing typed
  `ActionDisabledReasonCode::as_str()` authority.
- Removed the private Core `snapshot_game_error_code` duplicate and changed snapshot projection to call
  `error.code()`. The complete Core contextual-message match remains in place.
- Removed code literals from the server `From<GameError>` tuple match. The server now captures `value.code()` and
  retains its complete exhaustive message-only match, including every parameter value used in presentation text.
- No command, preflight, mutation, DTO, serialization envelope, RON, save/checkpoint, canonical or Unity source was
  changed.

### Post-change exhaustive and focused results

| Command | Result |
| --- | --- |
| Core 66-row characterization, debug | 1 passed, 0 failed, 0 ignored; 1,022 filtered |
| server 66-row characterization, debug | lib 1 passed, 0 failed, 0 ignored; 60 filtered; main selected zero |
| Core 66-row characterization, release | 1 passed, 0 failed, 0 ignored; 1,022 filtered |
| server 66-row characterization, release | lib 1 passed, 0 failed, 0 ignored; 60 filtered; main selected zero |
| `cargo test -p game_core --lib game::world::snapshot::tests -- --nocapture` | 2 passed, 0 failed, 0 ignored; 1,021 filtered |
| `cargo test -p game_server player_game_actor::state::tests -- --nocapture` | lib 9 passed, 0 failed, 0 ignored; 52 filtered; main selected zero |
| stale supplied Core battle authority | 1 passed, 0 failed, 0 ignored; 1,022 filtered |
| server authority filter | lib 1 passed, 0 failed, 0 ignored; 60 filtered; main selected zero |
| second mutation using the same revision | lib 1 passed, 0 failed, 0 ignored; 60 filtered; main selected zero |
| future live-battle resync sequence | lib 1 passed, 0 failed, 0 ignored; 60 filtered; main selected zero |

Each characterization layer still executes exactly 66 internal rows: 25 fixed semantic errors plus all 41 disabled
reasons. Exact serialized objects, the lowercase `"error"` tag, request correlation, distinct contextual messages
and nontrivial authority/detail values remain identical. The authority/preflight cases retain rejection before
mutation and the existing error/resync behavior.

### Formatting correction

The first post-change `cargo fmt --all -- --check` reported only rustfmt import wrapping in the two new test modules.
Those exact import layouts were corrected; no production behavior changed. The final format and scoped diff checks
pass.

### Decision

Focused old/new parity is complete. Proceed to broad debug/release, diagnostic and consumer/static evidence before
the separate completion review.

## 2026-08-11 — E07 Broad Validation, Consumer Audit And Dirty Reconciliation

### Broad debug/release results

| Command | Runner result |
| --- | --- |
| `cargo test -p game_core --all-targets` | 1,074 passed, 0 failed, 0 ignored: lib 1,023; integrations 4 + 4 + 25 + 14 + 4 |
| `cargo test -p game_core --release --all-targets` | 1,074 passed, 0 failed, 0 ignored: lib 1,023; integrations 4 + 4 + 25 + 14 + 4 |
| `cargo test -p game_server --all-targets` | lib 61 passed, 0 failed, 0 ignored |
| `cargo test -p game_server --release --all-targets` | lib 61 passed, 0 failed, 0 ignored |
| `cargo check --workspace --all-targets` | exit 0; Core and server all targets checked |
| `cargo check --workspace --release --all-targets` | exit 0; Core and server all targets checked |
| `cargo fmt --all -- --check` | exit 0 |
| candidate-scoped `git diff --check` | exit 0 |

Zero-test binary/example targets are compile evidence only and are not included in the behavior totals. The 25-test
Core `ron_loading` integration suite is included independently in each 1,074-test debug/release total.

### Diagnostic result

Normal Clippy completed with exit 0 for Core and server, all targets, in both debug and release. The existing Core
baseline remains 82 library warnings / 134 library-test warnings; the existing server baseline remains 3 library
warnings / 7 library-test warnings. The Phase-32 carrier, snapshot and server adapter changes introduce no warning.
Warnings remain outside this ownership refactor.

### Static ownership result

- Exactly one production stable-code method exists: `GameError::code()` in `behavior.rs`.
- Exactly two production call sites consume it: Core snapshot `error.code()` and server `value.code()`.
- `snapshot_game_error_code` is absent. The two contextual-message matches remain exhaustive and distinct.
- The 25 fixed stable-code literals occur in production only in `GameError::code()`. Other occurrences are the
  pre-change characterization/golden tests, focused actor expectations or the independent typed disabled-reason
  vocabulary. No second `GameError` code match remains.
- The authority has no wildcard, default, unknown, alias or legacy branch. `ActionUnavailable` continues to consume
  the exhaustive 41-value disabled-reason authority.
- No Phase-33/34 source placeholder or Child directory exists.

Final implementation SHA-256 values are `84aa1a31...` for `behavior.rs`, `11c1d3b5...` for `snapshot.rs` and
`18c61fbe...` for server `state.rs`. Fingerprinted prospective files outside those three retain every activation
hash, including capability, world-state, focused-test, server handler/message/session and all four local canonical
surfaces.

### Canonical and accessible-consumer result

No canonical correction is required because serialized behavior is unchanged. The Core runtime canonical remains
`ab7d2f98...`, the game rulebook `6c53ecd5...`, server Unity bridge protocol `c82000b1...` and server bridge goal
`12443956...`.

External Unity evidence was inspected read-only. `GameWireMessage` accepts and retains arbitrary top-level error
code/message strings rather than owning a duplicate `GameError` whitelist (`462a8d94...`); `ActionCapability`
retains its separate typed disabled-reason subset (`5800c7b8...`); `GameSnapshotStore` remains `f43c2df6...`; the
Unity Core and battle-transport canonicals remain `a3224e24...` and `11609cd0...`. The already-dirty external tree
remains 2,161 rows at normalized hash `db1919da...`; Phase 32 made no external write.

### Main-worktree status reconciliation

The final expanded main status has 534 rows. A fresh canonical C-sort hash is
`a937cc3fc89824aa3a4b8110eaf8653c8b5b00233ac65a3f9eb5adffc47a058b`. Reconstructing the path set by excluding
the Phase-31 audit and the three Phase-32 Child files reproduces the independently recorded Phase-31 activation
baseline exactly: 530 rows at `848bd871dc0a39088be4b789cfeb80391762a555dbca8fd70d9acb53ed51ee56`.
Adding only the Phase-31 audit yields the correct 531-row hash `da983caec2b69a604695636935e55b1e08885c4bcbf9ea8e8b31fffd767255e7`;
adding only the three Phase-32 Child paths then yields the current 534-row hash above.

This reconstruction proves that the inherited `5b566e36...` Phase-31-closure hash and derived `7f12bd4a...`
Phase-32 activation hash recorded in E01~E03 do not correspond to the actual canonical C-sorted status stream. They
are superseded by this explicit correction; no unexpected path was added, removed or reclassified. All three
production targets were already modified at activation, and every other prospective fingerprint remains exact.

The protected marker remains 57 bytes at SHA-256
`614d2fbdbdb0b29def7b90d1fdbc65148aba63339eea72fef5998d4a8ec22730`, with mtime and ctime both
`2026-08-10 22:55:07.178423600 +0900`.

### Decision

Implementation evidence is ready for a separate read-only completion audit. Child and Master remain active until
that review independently returns `Complete / High`.

## 2026-08-11 — E08 Completion Audit And Closure

- The separate 286-line `COMPLETION_AUDIT.md` independently reread the final Child, production, exhaustive tests,
  local canonicals, accessible Unity consumers, migration/legacy boundaries and corrected dirty reconstruction. Its
  SHA-256 is `8b7c28421317cef272d71e2f3a47c86271a0ccb7cc3f323fab391d23b95a5b37`.
- All P32-C01~P32-C12 rows pass. The Primary verdict is `Complete`, the long-term fit is `High`, the Combined verdict
  is `Complete / High` and no material or non-material correction finding remains.
- Audit-time current-tree reruns again pass both exhaustive tables: Core 1 passed with 1,022 filtered and server lib
  1 passed with 60 filtered; each executes all 66 internal rows.
- The audit artifact is the sole status-path addition after implementation evidence. Expanded main status is exactly
  535 rows at canonical C-sort SHA-256
  `4f8c208d7f99ca7c1001fc0113910de5b80df2ec85e68e083e88cc2266a380f9`.
- The protected marker remains 57 bytes at SHA-256
  `614d2fbdbdb0b29def7b90d1fdbc65148aba63339eea72fef5998d4a8ec22730`, with unchanged timestamps.
- Phase 32 closes with all three Child headers complete. Master Phase 32 becomes
  `complete — Complete / High`; Phase 33 becomes `ready` only. No Phase-33 Child, source inspection or placeholder
  is authorized by this transition.
