# Phase 34 CRA-P10-C03 Completion Audit

Audit Mode: `phase_exit`
Audit Target: Phase 34 CRA-P10-C03 complete Core battle DTO transport reuse
Audit Date: 2026-08-11 (`Asia/Seoul`)
Current Status At Audit Start: `active`
Implementation Complete Claim At Audit Start: `false`
Policy Source: CRA-POL-005, CRA-POL-006 and the original/post-Boss CRA-P10-C03 disposition
Goal Documents: `PLAN.md`, `EXPERIMENTS.md`, `EXPERIMENT_NOTES.md`
Repository / Commit: `/mnt/f/work/simulator` / `f318414fb0156a67e04c5479000c2a2683fc0c09`
Audit Method: separate final-tree review after implementation evidence was recorded

## Audit Inputs And Scope

This audit independently reread the complete 567-line completion-review guide, all 747 pre-audit Phase-34 document
lines, the relevant Master policy/dependency/registry entries, final Core DTO definitions, final server
message/mapping/actor/session paths, exact serde tests, actor-order tests, the current fixture generator, local
canonicals and accessible Unity parser/store/runtime/test/canonical consumers. It reran the decisive current-tree
serde, actor-order, preview-union and server-preview tests and reconciled broad debug/release, static, fixture,
consumer, hash, status and protected-marker evidence instead of treating the implementation record as proof.

CRA-P10-C03 is a behavior-preserving ownership refactor. CRA-POL-005 freezes the one deterministic public
effect-preview union. CRA-POL-006 freezes setup/update/resync JSON, public Rust/Unity compatibility and all message
selection/order behavior. The target is one complete Core content-schema authority carried once by server
transport. Server envelope selection, resync routing wrapper, request correlation, actor order and reconnect remain
separate owners. Private Boss runtime remains absent from public transport.

Phase 33 and Phase 18 are both `complete — Complete / High`; Master Phase 34 is the sole active row. Phase 35
CRA-P10-C07 remains pending and its Child path is absent. This audit closes only Phase 34 and does not authorize or
prebuild Phase-35 cleanup.

## Current, Canonical And Target Policy

- Current actual behavior: Core `LiveBattleUpdateDto` serializes eight complete fields and
  `LiveBattleSetupSnapshotDto` serializes sixteen. Server standalone update/setup variants contain those complete
  DTOs and delegate their already-tagged objects at the top level. Server resync retains its tagged outer
  `{ battle_uuid, update }` object. Actor paths retain accepted response, optional setup, update/resync and optional
  final snapshot ordering.
- Current canonical behavior: local Core canonical states setup version 2, setup/checkpoint field-work and direct
  top-level Core DTO carriage. The server and Unity transport canonicals retain top-level setup/update, one nested
  resync update, response-first command ordering, reconnect setup/update and final update-before-snapshot rules.
- Approved target policy: observable JSON, discriminator, omission, nested content, CRA-POL-005 preview union,
  revision/event/checkpoint values, lifecycle order, data, persistence and consumer behavior remain exact while the
  server's duplicate 7+15 content declarations and destructuring disappear.

The final implementation matches all three statements. One pre-existing external maintenance observation is
recorded separately: Unity's untracked revision fixture and one external canonical JSON example still show the
older version-1/minimal fixture. Actual Unity code consumes raw objects and its exact positive fixture test passes;
no wire or consumer migration is required by this Phase.

## Completion Evidence Matrix

| ID | Completion condition or invariant | Policy basis | Runtime/data/contract evidence | Validation evidence | Verdict |
| --- | --- | --- | --- | --- | --- |
| P34-C01 | complete Core DTO/server transport/consumer graph identified | CRA-P10-C03 scope | Core 8/16 fields; server standalone/resync variants; command, tick, recovery, reconnect, fixture and Unity consumers inventoried | source/caller/static consumer review | pass |
| P34-C02 | dirty targets and concurrent work preserved | dirty-work and marker invariant | only three already-dirty server actor paths carry Phase-34 edits; Core/generator/canonicals/Unity and marker hashes preserved | 542-row activation set/hash retained before audit; external 2,161-row set retained | pass |
| P34-C03 | old setup/update/resync behavior characterized first | exact-serde-parity-tests-first disposition | three complete real-Core DTO comparisons were added and passed before production representation changed | old production 3/3 serde and 20/20 actor baseline; current audit rerun 3/3 and 20/20 | pass |
| P34-C04 | one Core content-schema authority exists | target authority | server variants hold complete Core DTOs once; direct newtype constructors; resync alone owns outer routing fields | field counts, enum review and no brace-form/custom serializer search | pass |
| P34-C05 | setup/update top-level JSON remains exact | CRA-POL-006 | variant-level whole-value delegation; Core DTO owns `type`; no setup/update/payload nesting | exact direct-Core `Value` and text equality plus round-trip, 2 standalone rows | pass |
| P34-C06 | nested resync JSON remains exact | CRA-POL-006 | tagged server wrapper retains outer UUID and one complete nested Core update | complete resync `Value` equality and unchanged Unity `ApplyResync` path | pass |
| P34-C07 | preview/revision/event/checkpoint content remains exact | CRA-POL-005/006 | entire typed DTO is forwarded; Core preview and DTO sources unchanged; rich serde rows cover revision/time/event/checkpoint/capability/field-work/focus/resource | serde 3/3; preview union 6/6; server preview seam 1/1 | pass |
| P34-C08 | command/tick/reconnect/publication order remains exact | frozen server lifecycle | mapper and actor still own response, side-message and optional snapshot sequence; reconnect remains snapshot then setup/update at session boundary | handler 20/20; broad server 62x2 | pass |
| P34-C09 | DTO/data/save/public/Unity compatibility is exact | CRA-POL-006 | no Core DTO/data/RON/save/public schema/canonical/Unity edit; parser keeps raw JSON and store dispatches unchanged tags | Core 1,077x2 including RON 25x2; Unity fixture consumer 1/1 | pass |
| P34-C10 | no fallback/alias/dual schema/Boss placeholder remains | explicit exclusions | exactly two supported untagged whole-value variants; no custom serializer/field re-enumerator/alias/fallback/Boss carrier | static legacy/custom-serde/Boss inventory | pass |
| P34-C11 | focused/broad debug/release/static validation is green | Phase validation gate | Core/server compile together in both profiles; no new normal-Clippy warning | Core 1,077x2; server 62x2; checks, format, diff and Clippy pass | pass |
| P34-C12 | independent completion audit is current | phase-exit guide | this final-tree review covers runtime, data, DTO, consumer, legacy, dirty, canonical and long-term boundaries | audit reruns: server 3+20+1 and Core 6 pass | pass |

All twelve required rows have direct current-tree evidence. No completion condition is missing or failed.

## Runtime And State Evidence

Phase 34 changes transport composition only. Core battle simulation, state mutation, revision advancement, event
production, checkpoint construction, range-preview production, save/RON loading and Boss scheduling are unchanged.
Core `behavior.rs` retains its activation hash, as do the fixture producer and local contract documents.

The final ownership path is:

```text
Core battle producer
  -> LiveBattleSetupSnapshotDto / LiveBattleUpdateDto (complete tagged content)
  -> BehaviorCommandResultProjection::BattleUpdate
  -> game_server selects standalone setup/update or resync wrapper
  -> actor publishes response, ordered side messages, optional final snapshot
  -> Unity dispatches by unchanged type and retains the raw object
```

`behavior_result_to_command_result` still emits a successful `CommandAccepted` response and builds side messages in
setup-before-update order only for a standalone update response. A requested resync uses the existing resync
wrapper and no setup. All battle mappings still set `send_state_snapshot: false`. The actor's
`emit_command_execution_result` sends response first, then side messages, then an optional snapshot. Missing/stale
authority recovery still returns error plus current resync with no snapshot or mutation.

Reconnect still builds the root state snapshot, obtains the same-battle Core recovery projection, then appends
standalone setup and update. The WebSocket session sends `authed`, root snapshot and those side messages in order.
Delivery-cursor bookkeeping now reads UUID/sequence from the contained Core DTO rather than destructuring a copied
server field list; its meaning is unchanged. Final battle tick remains update then combat-result snapshot.

## Data Evidence

The candidate neither reads nor authors production live data. No RON file, schema, loader, validator, balance value
or fixture producer changed. Both Core all-target profiles execute the independent 25-test `ron_loading` binary and
all 25 pass in each. No data migration, fallback, normalization or repair is needed.

## DTO, Transport And Persistence Evidence

Core remains the complete setup/update schema owner:

- `LiveBattleUpdateDto`: renamed top-level `type` plus battle UUID, revision, capabilities, server time, event
  delta, checkpoint and visual novel — eight fields;
- `LiveBattleSetupSnapshotDto`: renamed top-level `type`, setup version, identity/mission, optional omitted survive
  timer, battlefield, routes/zones/points/objects/field-work/units and catalog refs — sixteen fields.

Server `PlayerGameServerMessage` remains internally tagged for server-owned messages. Its final two variants are
variant-level untagged newtypes containing the complete already-tagged Core objects. This supported Serde
composition serializes the inner object itself, so there is one `type`, no duplicate key and no new nesting. The
constructors are direct `Self::BattleUpdate(update)` and `Self::BattleSetupSnapshot(setup)`. `BattleResync` remains
the server-owned tagged `{ battle_uuid, update: LiveBattleUpdateDto }` envelope.

The exact standalone tests compare both `serde_json::Value` and serialized text against direct serialization of the
same Core DTO and round-trip through `PlayerGameServerMessage`. The resync test compares the complete outer object
to one nested direct-Core update. Rich values exercise nonzero revision/time/event ranges, source command,
capability, tactical focus, field-work setup/runtime, resource, optional survive timer, catalog reference and null
visual novel. Core effect-preview tests independently protect the CRA-POL-005 union; because the server contains the
whole checkpoint DTO and has no enumerating serializer, nested preview content cannot be selectively dropped.

No V3/save/checkpoint body, event-log schema, public Core DTO, persistence reader/writer or restore code changed.
There is no rollout interval, dual reader/writer or coordinated migration.

## Consumer Evidence

Confirmed workspace consumers:

- Phase-33 typed battle result projection feeds game_server command and tick mapping;
- game_server command, live tick, authority recovery and reconnect paths call the three retained constructors;
- delivery-cursor tracking reads the contained update DTO;
- the typed fixture generator serializes all three public transport forms.

Confirmed accessible external consumers, inspected read-only:

- Unity `GameWireMessage` parses the unchanged discriminator and preserves the complete raw `JObject` at SHA-256
  `462a8d9423cca71db65c5f7ed9e12b00e964a5ba7a094b1bd0a38442b20484cc`;
- `GameSnapshotStore` dispatches unchanged setup/update/resync tags at SHA-256
  `f43c2df62eef86a477f2639d54fe39ba44fbfd0bc63aa87726619f8feb03f5f2`;
- `BattleRuntimeState` consumes raw setup/update/resync and validates UUID, revision, event range, checkpoint and
  resync wrapper at SHA-256 `75df7d1a8726a4bf9d3a9a636032dc8dccfd4ad15a0a7602001143a3a5494050`;
- the exact Unity fixture-consumer test passed 1/1 through the open Editor.

Unknown deployed Rust/server/Unity revisions remain explicitly unknown. Exact wire/public API behavior and
retention of all constructors/types are the CRA-POL-006 control; negative workspace search was not used as breaking
authorization.

## Legacy And Compatibility Audit

Static review finds no brace-form `BattleUpdate { ... }` or `BattleSetupSnapshot { ... }` server variant or pattern,
no server content-field declaration list, no field-by-field constructor, no custom
`Serialize for PlayerGameServerMessage`, no battle alias/flatten/default fallback, no dual message shape and no
compatibility carrier. Exactly two `#[serde(untagged)]` attributes delegate the complete Core objects. The unrelated
host snapshot flatten attributes in `state.rs` are outside battle transport and predate this Phase.

No public setup/update/checkpoint/event DTO contains Boss runtime, wake, scheduler or session state. The unrelated
Boss omen projection and `BossDefeated` failure reason elsewhere in `behavior.rs` are legitimate domain values, not
public battle lifecycle carriers. No ignored test, debug-only bypass or Phase-35 placeholder was introduced.

## Scoped Diff And Dirty Preservation

The Phase-34 implementation is confined to three paths already modified at activation:

- `game_server/src/game/player_game_actor/messages.rs` final SHA-256
  `8e3c53692a4c2c8acc9825617cbad8b40526c2cd749dd82220b220581da09359`;
- `game_server/src/game/player_game_actor/handlers.rs` final SHA-256
  `431abf19a853d2cc4a7b84d3caa558ff6bd670ddd8e6f67baf5eb2980468d2de`;
- `game_server/src/game/player_game_actor/state.rs` final SHA-256
  `1d2cb3233d14211f97bf2a5a9ae484ecda5e12377a2ad1221e203b445139219e`.

The Phase-attributed changes are the two message newtypes/direct constructors, exact serde tests, production
delivery-cursor tuple access and corresponding test patterns. Existing unrelated campaign changes in the same
already-dirty handlers/state files remain intact. Candidate-scoped `git diff --check` passes; no broad formatting or
cleanup was performed.

Core `behavior.rs` remains at its activation SHA-256 `eec607df...` and the fixture generator remains `6f196902...`.
All prospective local canonicals, manifests and server session source retain their activation hashes. Before this
audit artifact, expanded main status remained exactly the 542-row activation set at canonical C-sort SHA-256
`e1a402f67ddf629ca5bdd57fa776a71cd6f5acab2fc3540d75908d8b435a9927`.

External Unity status remains the pre-existing 2,161-row set at canonical C-sort SHA-256 `db1919da...`; no external
write occurred. The fixture remains untracked and byte-identical at SHA-256
`889da959ffaf4b54bbb717622dd11ac97d813729570a4308b567785f6c67e5a8`.

The protected marker remains 57 bytes at SHA-256
`614d2fbdbdb0b29def7b90d1fdbc65148aba63339eea72fef5998d4a8ec22730`, with mtime and ctime both
`2026-08-10 22:55:07.178423600 +0900`.

## Canonical Alignment

Observable behavior did not change, so a Phase-34 canonical edit would create false ownership-refactor churn. The
nearest local runtime canonical already states that setup version 2 contains immutable `field_work_objects`, update
checkpoint contains current field-work state and game_server carries the Core DTO unchanged at top-level
setup/update/resync. It retains activation SHA-256 `ab7d2f...`. Local rulebook and server bridge documents also
retain their activation hashes and do not contradict the final transport.

The external battle transport canonical accurately retains standalone top-level setup/update, one nested resync
update, response/setup/update ordering, reconnect recovery and final update-before-snapshot. Its illustrative setup
JSON still says `setup_version: 1`, matching the pre-existing stale untracked fixture rather than the current Core
version 2. This is a real external documentation/fixture maintenance observation, not a Phase-34 output drift:
current Core/server canonical and code are correct, the external parser stores unknown/current fields, the positive
consumer test passes, and no consumer migration is necessary. External files were explicitly read-only in this
Phase and were preserved.

## Validation Results

Focused pre-change characterization:

- `cargo test -p game_server serializes_battle_ -- --nocapture`: lib 3 passed, 0 failed/ignored, 59 filtered; main
  selected zero and is not behavior evidence;
- `cargo test -p game_server player_game_actor::handlers::tests -- --nocapture`: lib 20 passed, 0 failed/ignored,
  42 filtered; main zero-test target not counted.

Focused post-change validation returned the same 3/3 and 20/20 counts. The two standalone rows additionally prove
direct-Core text equality and round-trip deserialization.

Broad final-tree validation:

- `cargo test -p game_core --all-targets`: 1,077 passed, 0 failed, 0 ignored — lib 1,026 plus integrations
  4 + 4 + 25 + 14 + 4;
- `cargo test -p game_core --release --all-targets`: the same 1,077 passed, 0 failed, 0 ignored;
- `cargo test -p game_server --all-targets`: 62 passed, 0 failed, 0 ignored;
- `cargo test -p game_server --release --all-targets`: 62 passed, 0 failed, 0 ignored;
- workspace all-target checks in debug and release: exit 0;
- `cargo fmt --all -- --check` and candidate-scoped `git diff --check`: exit 0;
- normal Core Clippy debug/release: exit 0 with unchanged 82 library / 134 library-test warnings;
- normal server Clippy debug/release: exit 0 with unchanged 3 library / 7 library-test warnings.

Audit-time validity reruns after independent final-tree review:

- exact standalone setup/update and nested resync serde: lib 3 passed, 0 failed/ignored, 59 filtered;
- actor/reconnect/order handler module: lib 20 passed, 0 failed/ignored, 42 filtered;
- CRA-POL-005 effect-preview family: Core lib 6 passed, 0 failed/ignored, 1,020 filtered;
- server deployment-preview transport seam: lib 1 passed, 0 failed/ignored, 61 filtered;
- Unity EditMode exact revision-fixture consumer: 1 passed, 0 failed/skipped in 0.69 seconds.

## Baseline Failures

The typed generator comparison against external `battle_revision_contract.json` fails both before and after the
production refactor for the same pre-existing reason: fixture setup version 1 versus current 2, absent setup
field-work and absent checkpoint tactical-focus/field-work/resource values. Post-change generator stdout itself
succeeds at SHA-256 `fe7d072b5311b27122ae5c3e9d9495d6d67399b633206369de706c120c968c37`.
The file is an already-untracked external user/campaign artifact and was not overwritten. Unity's exact consumer
test passes it, proving the drift is not a current consumer incompatibility.

An initial broad-validation attempt ran Core debug and release concurrently. The debug
`live_ron_defense_route_playable_path_runs_to_combat_result` test read the shared diagnostic trace while the other
process was writing and reported JSON EOF. This is a runner collision: sequential debug and release reruns each
passed all 1,077 tests. It is not a product or Phase-34 failure and Core profiles should remain sequential while the
diagnostic path is shared.

No other focused, production or broad-test failure remains. Existing Core/server warnings are inherited diagnostic
baselines; normal Clippy exits zero and no candidate-specific warning was added.

## Omitted Validation And Residual Risk

- A live WebSocket probe was not run. Exact real server serde plus command/tick/recovery/reconnect/final-order tests
  close the unchanged transport path without a network-only branch.
- Unity PlayMode/full-class execution was not run. The fixture mismatch caused the exact positive EditMode consumer
  to be run, and it passed. No Unity source, field shape or rendering behavior changed.
- Performance benchmarking is irrelevant to this enum-composition ownership seam.
- Unknown deployed consumers remain explicitly unknown; exact JSON/public API retention is the approved control.

None of these omissions removes required evidence or creates a material finding.

## Cross-Component Findings

Core now owns every setup/update content field once. game_server retains only the transport decisions and resync
envelope that belong to it. Accessible Unity consumes unchanged raw JSON and owns no competing schema projection.
Phase-33's typed result classification feeds this seam without a temporary adapter; Phase-18's preview union passes
unchanged through it. No data, persistence, DTO migration, compatibility shim or Phase-35 placeholder crosses the
boundary.

There is no material correction finding. The external version-1 canonical example/untracked fixture is a
non-material pre-existing maintenance observation: it does not describe or induce the current server output, the
actual local canonical is current, and the exact Unity consumer remains green.

## Long-Term Direction Review

| Candidate ID | Work type | Current -> target authority result | Preserved invariant result | Removed/consolidated result | Validation | Verdict |
| --- | --- | --- | --- | --- | --- | --- |
| CRA-P10-C03 | `behavior_preserving_refactor` | Core complete DTO plus duplicate server field schema -> one complete Core content authority | setup/update/resync JSON, preview union, revision/events/checkpoint, actor/reconnect order, public/Unity behavior exact | 22 server declarations and field destructuring removed; server wrapper/order authority retained | 3-row old/new serde, 20-row lifecycle, Core 1,077x2, server 62x2, static and Unity consumer | pass |

- Fit: `High`.
- Improvement class: `none` for the refactor; external fixture/document refresh remains outside this code candidate.
- Reason: the smallest existing typed seam now carries the complete DTO once, removes synchronization points, keeps
  distinct server responsibilities distinct and introduces no serializer table, fallback, new schema, migration or
  future placeholder. Tests protect durable JSON and lifecycle behavior rather than the preferred private enum
  layout.

## Primary Verdict

- Primary verdict: `Complete`.
- Long-term fit: `High`.
- Combined verdict: `Complete / High`.
- Material findings: none.
- Non-material correction findings: none in the Phase-34 implementation.
- External maintenance observation: stale version-1 Unity fixture/example, preserved read-only and positively
  consumed; it requires no Phase-34 or consumer migration.
- Confidence: High, based on test-first complete DTO characterization, independent final-tree review, direct-Core
  text/Value equality, lifecycle tests, debug/release broad validation, static one-owner proof and actual accessible
  Unity consumer execution.

## Status Transition

- Child transition allowed: `active` / `Implementation Complete: false` -> `complete` /
  `Implementation Complete: true`.
- Master Phase-34 transition allowed: `active` -> `complete — Complete / High`.
- Phase 35 activation: not yet active; it may become `ready` only after Child/Master closure records are updated and
  this audit artifact/status/marker evidence is captured.

## Remaining Risk And Required Action

No immediate correction or policy decision is required. Preserve all pre-existing dirty work, the protected marker
and external Unity tree. Record this audit's final hash/status evidence, close Phase 34 and mark Phase 35 ready only.
Any later refresh of the external untracked fixture and version-1 documentation example must be an explicitly
authorized Unity fixture/document maintenance task and must regenerate from the current typed Rust producer; it must
not change the current wire contract.
