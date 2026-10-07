# Phase 34 CRA-P10-C03 Experiments

Status: complete
Implementation Complete: true

## 2026-08-11 — E01 Initialization And Evidence Freeze

### Hypothesis

Phase 34 can be initialized as the sole next candidate because Phase 33 and Phase 18 exited `Complete / High`, the
Master marks CRA-P10-C03 `ready`, no implementation Child is active and historical/final evidence defines a
policy-complete exact-parity boundary.

### Evidence reviewed

- the complete 874-line Goal authoring guide and complete 311-line Phase-33 completion audit;
- Master scope, CRA-POL-005/006 policy, Phase registry and activation/validation gates;
- original Phase-10 candidate, boundary, test and completion evidence for C03;
- post-Boss Phase-10 candidate, boundary, test and completion evidence;
- final rebaselined catalog, exact linear order, test catalog, dependency/consumer gates and approved policy;
- completed Phase-18 preview-union and Phase-33 result-classification exit boundaries.

The authoritative assignment is item 34: reuse complete Core battle DTOs in server transport after exact old/new
setup/update/resync JSON proof, while preserving actor/reconnect ordering and adding no public Boss wire.

### Exact initialization checks

- repository HEAD: `f318414fb0156a67e04c5479000c2a2683fc0c09`;
- expanded porcelain status before this Child: 539 rows;
- canonical C-sort SHA-256:
  `f17ab3657507e3cc39e91ac7136b8851d9db346cfeb421cdfbf5b79239b971cc`;
- no Phase-34 path existed before initialization;
- Master is active, Phase 33 is `complete — Complete / High`, Phase 34 is `ready`, and no implementation Child or
  registry row is active;
- protected marker remains 57 bytes at SHA-256
  `614d2fbdbdb0b29def7b90d1fdbc65148aba63339eea72fef5998d4a8ec22730` with unchanged mtime/ctime
  `2026-08-10 22:55:07.178423600 +0900`.

### Commands and tests

Initialization used Goal/history/registry/path/status evidence only. No new Phase-34-directed production, test,
data, canonical or external-consumer read/search occurred. Historical and completed-Phase observations are
non-baseline input. Executed tests: 0; pass/fail/ignored: not applicable.

### Result and decision

Initialize the required Child documents around exact DTO reuse, old/new JSON parity, actor/reconnect order and the
no-Boss-wire boundary. Production inspection remains forbidden until sole activation, complete active-document
reread and prospective path-only fingerprinting close.

## Pending Evidence

- Master/Child sole activation and exact post-document status delta;
- complete activation reread and prospective content fingerprints;
- current Core setup/update field/serde/builder and server message/constructor/consumer graph;
- old-production exact complete setup/update/resync JSON table;
- actor/reconnect/fixture/Unity consumer inventory;
- implementation, exact parity, broad validation and independent completion audit.

## 2026-08-11 — E02 Sole Activation

### Hypothesis

The initialized Child can become the only active implementation Phase without authorizing Phase 35 work.

### Change and result

- All three Child headers changed from `initialized` to `active`; `Implementation Complete` remains `false`.
- Master Phase 34 changed from `ready` to `active`; Phase 33 remains `complete — Complete / High` and Phase 35
  remains `pending`.
- The three Child files explain the exact expanded-status increase from 539 to 542 rows. The canonical C-sort
  SHA-256 after initialization is `e1a402f67ddf629ca5bdd57fa776a71cd6f5acab2fc3540d75908d8b435a9927`.
- The protected marker remains exact. No Phase-34 source, test, data, fixture, canonical, external consumer or
  Phase-35 path was inspected or changed.

### Commands and tests

Document, registry, status and marker checks only. Executed tests: 0; pass/fail/ignored: not applicable.

### Decision

Phase 34 is the sole active Child. Detailed production inspection remains gated on a complete reread of all three
active documents and path-only prospective fingerprints.

## 2026-08-11 — E03 Activation Reread And Prospective Fingerprints

### Procedure

- Reread all active PLAN, experiment and note lines after sole activation.
- Reconfirmed the complete Phase-33 audit and original/post-Boss/final P10-C03 assignment evidence.
- Used historical path evidence and filename-only discovery to capture likely Core/server/test/fixture/canonical/
  external-consumer hashes and scoped status without printing new Phase-34 production contents.

### Prospective Core fingerprints

| Path | SHA-256 | Activation status |
| --- | --- | --- |
| `core_enhanced/src/game/behavior.rs` | `eec607df972637d02d4a8d0cecc2b570363e1ba7afb7ebe5dd53d8a8e9150927` | modified before Phase 34 |
| `core_enhanced/src/game/world.rs` | `c533e0ffa4165ac7f61decf662a6dabc2a9bb801f3913708cef45d46c3014d5d` | modified before Phase 34 |
| `core_enhanced/src/game/world/combat.rs` | `918e8659cdcaff802a94dc823332e29429de95d7902cb40062a51a0016826d58` | modified before Phase 34 |
| `core_enhanced/src/game/world/state.rs` | `1e5f91772f82531a7c90f9676cfda25af463b702cf26ae13f320cae8da413281` | modified before Phase 34 |
| `core_enhanced/src/game/world/battle_capability.rs` | `c95635218a4b56a8a01be28813fc20354080c920f1f61fa2f62ec93e57a51389` | modified before Phase 34 |
| `core_enhanced/src/game/world/snapshot.rs` | `11c1d3b5696412d3e5457648256a2af9fbbc66c4ece40abdcc98cefc4133f0dd` | modified before Phase 34 |
| `core_enhanced/src/game/world/tests/combat.rs` | `08177bd514fa5836c47c4c8ce48efbb80a2b223a3efd1a5d0ac19203b48b384e` | modified before Phase 34 |
| `core_enhanced/tests/ron_loading.rs` | `75558ff28be83cafcfe535252f552903b2077249c2cd81ef00b41d63195854b3` | modified before Phase 34 |
| `core_enhanced/Cargo.toml` | `40d8a9ec74b9c36862fbb580ad4bef554be03ab27875ceb9dadf9db93b5929af` | clean |

### Prospective server/fixture fingerprints

| Path | SHA-256 | Activation status |
| --- | --- | --- |
| `game_server/src/game/player_game_actor/messages.rs` | `dee1e0484bad0d4f1cc13c7f34162df93972bec90425f395223a8ff1a05c3480` | modified before Phase 34 |
| `game_server/src/game/player_game_actor/state.rs` | `623c7343ac7f3382e1f004745bec5abf89b2e605e0433478cbaeb596940320ef` | modified before Phase 34 |
| `game_server/src/game/player_game_actor/handlers.rs` | `aa3ede7a03f38f040720c233f5b7782fbe4f7162b5812f14e20843a3a9f30d10` | modified before Phase 34 |
| `game_server/src/game/player_game_actor/session.rs` | `bde6f11f1fd299bff57da5916b255a3d10fb48f453a08216e1c1fa4f1fd5cee1` | clean |
| `game_server/examples/generate_battle_revision_fixture.rs` | `6f196902c61a332d54e854e59a8b3021bd97cc957f51daae57d8b29629fe4977` | modified before Phase 34 |
| `game_server/Cargo.toml` | `19b41cd19c6e60818698b8ecaa9d9cc0c44ed071bff2cfbb7296d92dbeee722f` | clean |

### Prospective local and external contract fingerprints

| Path | SHA-256 | Activation status |
| --- | --- | --- |
| `core_enhanced/docs/core_runtime_contract.md` | `ab7d2f98040f15bfcb9b253ee06479b9989efc764fb57192447a23bc502c167e` | modified before Phase 34 |
| `core_enhanced/docs/game_rulebook.md` | `6c53ecd5d0588aace1a2b043b31434d004327d60938ecf61066ac19acfd31951` | modified before Phase 34 |
| `game_server/docs/unity_bridge_protocol.md` | `c82000b172892df78efe33b16b1bb268076b89483f6b006653c109e98edd3dd5` | modified before Phase 34 |
| `game_server/docs/game_server_core_bridge_goal.md` | `124439562ae177a725306a19071efc7f385a269c9e0f1e891a2099c0d5e7b2ad` | clean |
| Unity `GameWireMessage.cs` | `462a8d9423cca71db65c5f7ed9e12b00e964a5ba7a094b1bd0a38442b20484cc` | external read-only |
| Unity `GameSnapshotStore.cs` | `f43c2df62eef86a477f2639d54fe39ba44fbfd0bc63aa87726619f8feb03f5f2` | external read-only |
| Unity `battle_revision_contract.json` | `889da959ffaf4b54bbb717622dd11ac97d813729570a4308b567785f6c67e5a8` | external read-only |
| Unity `unity_core_contract.md` | `a3224e244f9577fbb6a4d6d01e44b7ea85f3ba1969f656a22712b0ba972063f5` | external read-only |
| Unity `core_unity_battle_transport_contract.md` | `11609cd0240759784c0b441b3c6bfd6b17219bc71e0f0c672c309f763e072618` | external read-only |

The external Unity repository is already dirty with 2,161 expanded rows at canonical C-sort SHA-256
`db1919dae1ade35be6971178c4340393261bcbdc56a0a6a1bf8f2eee1e44d18b`; Phase 34 has no external write
authorization.

### Activation preservation

- HEAD remains `f318414fb0156a67e04c5479000c2a2683fc0c09`.
- Expanded main status remains exactly 542 rows at canonical C-sort SHA-256
  `e1a402f67ddf629ca5bdd57fa776a71cd6f5acab2fc3540d75908d8b435a9927`.
- Phase 34 is the sole active registry row; Phase 35 remains pending and its Child path is absent.
- The protected marker remains byte/hash/mtime/ctime exact.

### Commands and tests

Only active-document/history reads, filename discovery, path-known SHA-256, scoped status and marker checks were
used. Executed tests: 0; pass/fail/ignored: not applicable.

### Decision

The activation gate is closed. Detailed current-tree source/API/test/fixture/canonical/external-consumer inspection
and focused old-production characterization may now begin.

## 2026-08-11 — E04 Current Transport Graph And Old-Production Characterization

### Current graph

- Core `LiveBattleUpdateDto` owns eight serialized fields including its renamed `type`; Core
  `LiveBattleSetupSnapshotDto` owns sixteen fields including `type` and the optional omitted
  `survive_timer_ms`.
- Server `PlayerGameServerMessage::BattleUpdate` independently repeats the other seven update fields and
  `BattleSetupSnapshot` repeats the other fifteen setup fields. Their constructors destructure both complete Core
  DTOs field by field. The server adds no content invariant to those 22 declarations.
- `BattleResync` legitimately owns its outer `type`, repeated routing `battle_uuid` and nested complete Core update.
  The duplicate UUID is part of the frozen public envelope and is retained.
- Phase-33 state mapping, command/tick/recovery/reconnect handlers and the fixture generator all call the three
  server constructors. Production handler delivery tracking also destructures top-level update fields; tests use
  direct patterns to assert order and content.
- Core direct serde already emits exact standalone `battle_update` and `battle_setup_snapshot` objects. The smallest
  candidate is therefore variant-level untagged delegation for those two complete Core DTOs while retaining the
  internally tagged server-owned variants and resync wrapper.
- Workspace search finds no construction of the duplicated battle variants outside the server actor module. The
  server message enum remains an internal transport carrier; the externally frozen boundary is its JSON.
- Private Boss scheduler/wake state remains absent from both public DTOs and server variants.

### Test-first old-production characterization

Before any production message representation change, strengthened the three existing rich serde tests so they now
compare complete `serde_json::Value`, not selected fields only:

- server update equals direct serialization of the same complete Core `LiveBattleUpdateDto`;
- server setup equals direct serialization of the same complete Core `LiveBattleSetupSnapshotDto`;
- server resync equals the exact server-owned outer object containing the complete direct Core update.

`cargo test -p game_server serializes_battle_ -- --nocapture` passed on old production: lib 3 passed, 0 failed,
0 ignored, 59 filtered; main selected zero tests and is not behavior evidence. Rich values cover action capability,
event/cause/command ID, tactical focus, field-work state/setup, resource, optional survive timer, catalog references,
revision/time/checkpoint and null visual novel.

The old-production actor/order baseline also passes:
`cargo test -p game_server player_game_actor::handlers::tests -- --nocapture` — lib 20 passed, 0 failed/ignored,
42 filtered; main zero-test target not counted. This includes accepted setup/update, update-only tick, stale/missing
authority resync, reconnect recovery, paused/resumed and final update→snapshot paths.

### Accessible fixture baseline finding

The typed fixture generator run against Unity's checked-in `battle_revision_contract.json` fails before production
changes. The external fixture is stale: setup version 1 versus current 2 and it lacks current setup field-work plus
update tactical-focus/field-work/resource fields. The Unity EditMode consumer reads that fixture and intentionally
asserts only revision/sequence/playback behavior, so the stale fixture remains parseable but is not exact current
Rust output.

This is a classified pre-existing external-fixture drift, not permission to alter wire output. Phase 34 must compare
post-change generated output to the same current Rust baseline and determine whether the authorized consumer-aware
scope permits regenerating only that fixture; no Unity runtime/source or schema migration is implied.

### Decision

The required exact old-production setup/update/resync characterization is green. Implement only complete Core DTO
reuse in server messages, retain resync/order owners, and use the same tests to prove post-change parity. Treat the
external fixture mismatch as a separate baseline finding until the consumer-aware disposition is closed.

## 2026-08-11 — E05 Complete Core DTO Reuse And Focused Parity

### Implementation

- Replaced the server's 7-field `BattleUpdate` and 15-field `BattleSetupSnapshot` declarations with newtype variants
  containing the complete Core `LiveBattleUpdateDto` and `LiveBattleSetupSnapshotDto` respectively.
- Marked only those final enum variants with Serde's supported variant-level `untagged` delegation. Their Core DTOs
  already contain the exact `type`, so serialization delegates the entire object once without duplicate keys,
  nesting, `Value` mutation or a custom field enumerator.
- Reduced both constructors to direct newtype construction. `BattleResync` retains its server-owned tagged wrapper,
  outer routing UUID and complete nested Core update.
- Updated production delivery-cursor inspection to read the DTO directly. Test patterns now inspect the same typed
  DTOs; no actor publication logic changed.
- Removed server imports that existed only for the duplicate 22-field schema. No Core DTO/builder, server state
  mapper, fixture generator, canonical, Unity source or Boss carrier changed.

### Focused result

- The same three complete old-production JSON tests pass after implementation: 3 passed, 0 failed/ignored, 59
  filtered. Standalone update/setup now also prove exact serialized text equality with direct Core serde, preventing
  hidden duplicate `type` keys, and round-trip the retained server `Deserialize` API.
- All 20 server handler tests pass again with 0 failed/ignored, 42 filtered. Command ack, setup/update, tick,
  authority recovery/resync, reconnect, Pause and final update→snapshot behavior remains exact.
- `cargo fmt --all -- --check` initially reported only one import-wrap difference in the edited messages test import;
  the exact requested layout was applied and no unrelated formatting was touched.

### Decision

The local current transport now has one complete content-schema owner and exact focused parity. Proceed to broad
debug/release, static/diagnostic, fixture-consumer and dirty-preservation validation.

## 2026-08-11 — E06 Broad Debug/Release And Diagnostic Validation

### Test execution

- `cargo test -p game_core --all-targets` passed sequentially: 1,077 passed, 0 failed, 0 ignored across the
  1,026-test library and five integration targets with 4, 4, 25, 14 and 4 tests.
- `cargo test -p game_core --release --all-targets` passed sequentially with the same 1,077/0/0 counts.
- `cargo test -p game_server --all-targets` passed: 62 passed, 0 failed, 0 ignored.
- `cargo test -p game_server --release --all-targets` passed with the same 62/0/0 counts.
- Workspace `cargo check --workspace --all-targets` and its `--release` counterpart both exited 0.
- `cargo fmt --all -- --check` and candidate-file `git diff --check` both exited 0.

An initial attempt ran the two Core profiles concurrently. The debug
`live_ron_defense_route_playable_path_runs_to_combat_result` test observed an EOF while reading the same diagnostic
trace JSON that the other process was writing. This is a shared-debug-artifact runner collision, not a product
failure: both complete suites passed when rerun sequentially with identical nonzero counts. Core test profiles must
remain sequential while that diagnostic path is shared.

### Clippy diagnostics

- Core debug and release Clippy both exit 0 with the inherited baseline: 82 library warnings and 134 library-test
  warnings, with no candidate-specific new warning.
- Server debug and release Clippy both exit 0 with the inherited baseline: 3 library warnings and 7 library-test
  warnings, with no candidate-specific new warning.
- No warning cleanup was attempted because it is unrelated to the ownership refactor.

### Result

All affected Core/server debug and release behavior, compile, format and normal diagnostic gates are green. The
single failed concurrent run is fully classified and superseded by both sequential green executions.

## 2026-08-11 — E07 Static Ownership, Fixture Consumer And Preservation Closure

### Static ownership audit

- Machine count confirms Core `LiveBattleUpdateDto` has 8 serialized fields and
  `LiveBattleSetupSnapshotDto` has 16. The server enum now has only
  `BattleUpdate(LiveBattleUpdateDto)` and `BattleSetupSnapshot(LiveBattleSetupSnapshotDto)` for those complete
  bodies.
- Workspace/server-actor searches find no brace-form construction or pattern for either standalone variant, no
  custom `Serialize` implementation for `PlayerGameServerMessage`, and no alias, flattening, fallback or dual
  battle schema in the server enum.
- Exactly two variant-level `#[serde(untagged)]` attributes perform whole-value delegation. Constructors are direct
  newtype construction; resync remains the one server-owned tagged wrapper.
- The public Core setup/update DTO field ranges and server message enum contain no Boss runtime, wake, session,
  scheduler or placeholder carrier. The two unrelated `boss` occurrences elsewhere in `behavior.rs` are an omen
  result and a run-failure reason, not battle transport fields.
- Core `behavior.rs` and the fixture generator retain their activation hashes, proving Phase 34 did not alter Core
  content construction or the reproducible fixture producer.

### Current fixture and Unity consumer disposition

- Post-change generator stdout succeeds at SHA-256
  `fe7d072b5311b27122ae5c3e9d9495d6d67399b633206369de706c120c968c37`.
- Comparing that output to external `battle_revision_contract.json` reproduces the exact same pre-change failure:
  the already-untracked fixture has setup version 1 and omits current setup field-work plus checkpoint tactical
  focus/field-work/resource fields. It is not an old/new Phase-34 output difference.
- Read-only inspection proves `GameWireMessage` retains the entire raw `JObject`, `GameSnapshotStore` dispatches the
  three unchanged discriminators, and `BattleRuntimeState` accepts setup/update/resync from those raw objects while
  validating UUID/revision/sequence/checkpoint invariants. No consumer depends on server Rust field ownership.
- Because the fixture mismatch triggered the PLAN's reconsideration rule, the exact positive Unity EditMode test was
  run through the open Editor:
  `RustGeneratedBattleRevisionFixture_AppliesHigherRevisionAndSameRevisionResync` passed 1/1, 0 failed/skipped in
  0.69 seconds. The stale but parseable fixture therefore remains a pre-existing fixture-maintenance debt, not a
  compatibility failure or reason for an external migration.
- PLAN scope marks Unity sources/fixtures read-only, and the fixture was already an untracked user/campaign file.
  It was not overwritten. Its hash remains
  `889da959ffaf4b54bbb717622dd11ac97d813729570a4308b567785f6c67e5a8`.

### Dirty and boundary preservation

- Main expanded status remains the exact 542-row activation set at canonical C-sort SHA-256
  `e1a402f67ddf629ca5bdd57fa776a71cd6f5acab2fc3540d75908d8b435a9927` before the completion-audit path is added.
- Final candidate hashes are `8e3c5369...` for `messages.rs`, `431abf19...` for `handlers.rs` and `1d2cb323...`
  for `state.rs`. Core `behavior.rs` remains `eec607df...`; the fixture generator remains `6f196902...`.
- External Unity expanded status remains 2,161 rows at canonical C-sort SHA-256 `db1919da...`; the prospective
  consumer and fixture hashes are unchanged after the read-only test.
- The protected marker remains 57 bytes at SHA-256
  `614d2fbdbdb0b29def7b90d1fdbc65148aba63339eea72fef5998d4a8ec22730`, with mtime and ctime both
  `2026-08-10 22:55:07.178423600 +0900`.
- Phase 35 remains pending and its Child path is absent.

### Decision

All Phase-34 implementation and validation conditions are evidenced. The external fixture drift is explained,
unchanged, positively consumed and outside this no-Unity-mutation refactor; it does not require a consumer migration.
Proceed to the independent completion audit. Do not activate Phase 35 unless that audit returns `Complete / High`.

## 2026-08-11 — E08 Completion Audit And Closure

- The separate 322-line `COMPLETION_AUDIT.md` independently reviewed final Core/server transport composition,
  exact serde, actor/reconnect order, preview union, data/save boundaries, local canonicals, accessible Unity
  consumers, legacy/static ownership and dirty preservation. Its SHA-256 is
  `e7eb07e86835ae6c15a663ef60ffb5f60836117fccbf5b9a74e71d3f001e2454`.
- All P34-C01~P34-C12 rows pass. The Primary verdict is `Complete`, long-term fit is `High`, Combined verdict is
  `Complete / High` and improvement class is `none`; no material Phase-34 finding remains.
- Audit-time current-tree reruns pass exact server serde 3/3, actor/reconnect/order 20/20, Core effect-preview union
  6/6 and the server preview seam 1/1. The exact Unity revision-fixture consumer also passes 1/1.
- The external version-1 fixture/example remains an explicitly non-material pre-existing maintenance observation:
  it is unchanged, external/read-only, positively consumed and requires no wire or consumer migration.
- The audit artifact alone explains the exact 542→543 expanded-status increase at canonical C-sort SHA-256
  `04f967cbbf1377c1d156530ffe7776d5e2b31d4f828b2efcd3f49cc0d6c07c9b`.
- The protected marker remains 57 bytes at SHA-256
  `614d2fbdbdb0b29def7b90d1fdbc65148aba63339eea72fef5998d4a8ec22730`, with unchanged timestamps.
- Phase 34 closes with all three Child headers complete. Master Phase 34 becomes
  `complete — Complete / High`; Phase 35 becomes `ready` only. No Phase-35 Child, production inspection or
  placeholder is authorized by this transition.
