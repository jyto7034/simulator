# P06 Checkpoint Migration Execution Plan

상태: `GEX-00 passed / GEX-10 exit criteria met; target freeze pending`

## Current Execution Identity

- Handoff / root target: `HA-CRAR-P2G60-002` / full `GS-CRAR-P2G50-002` goal-run.
- Selected member: `(GS-CRAR-P2G50-002, GOAL-CRAR-C2-P06-CHECKPOINT-MIGRATION)`.
- Phase / execution node: `PHASE-CRAR-C2-06-CHECKPOINT-MIGRATION` / `WP-CRAR-C2-07`.
- Goal Quality Contract: `2`; durable seam `IVA-C2-07` (`IVA-CRAR-C2-07` in the canonical Phase Plan).
- Policy / decision: `PER-CRAR-010-R3-CODE-REVIEWED` / `DR-CRAR-009-R1`; realization decision
  `TD-CRAR-020`.
- Current pass: separated GEX-00 intake/design only. No P06 production mutation occurred before this document set.
- Authorization: Core checkpoint codec, canonical development corpus, tests and actual Core runtime contract only.
  External store envelope, `game_server`, Unity, sibling resources, released/deployed save population, rollout and
  external consumers are excluded and are not inspected.

## Direct Dependencies and Currentness

- P01 `Complete / High`, current on `ATR-CRAR-C2-P01-GEX20-005`.
- P02 `Complete / High` remains the current validated-data prerequisite; P06 restores only into an existing
  `ValidatedGameData`-backed `GameCore`.
- P03 `Complete / High`, current on `ATR-CRAR-C2-P03-GEX20-003`; continue already executes through the staged
  application authority.
- P05 `Complete / High`, current on `ATR-CRAR-C2-P05-GEX10-001`; checkpoint/continuation contains no diagnostic event
  history and that lifetime split must remain exact.

## Objective and Exact Meaning

- Change the only production Core checkpoint writer from strict V4 to strict V5.
- Retain a separately named strict V4 importer. It maps the canonical V4 semantic payload into the current closed
  `RunCheckpointPayload`, validates the current graph and continues with identical identity, state and next decisions.
- Make V5 a strict envelope containing `version`, `checkpoint` and a SHA-256 checksum over a canonical JSON encoding of
  the checkpoint value. Canonical object-key ordering makes verification independent of `HashMap` iteration order;
  arrays retain their semantic order.
- Reject malformed JSON, missing/extra fields, unknown versions, invalid checksums and invalid restored graphs before
  authoritative publication. V4 has no invented checksum or aliases; it is parsed only by its exact legacy schema.
- Keep a reader-removal certification gate closed. Removing the V4 importer remains forbidden until a separate user
  migration-certification decision; passing this Goal does not certify a released/deployed save population.

## GEX-00 Current Inventory

- `src/game/world/active_run.rs` owns one strict `ActiveRunCorePayloadV4` reader/writer with
  `#[serde(deny_unknown_fields)]` and a loose version header. New writes and reads are both version `4`; no checksum,
  explicit importer boundary or removal-certification guard exists.
- `continue_from_active_run_checkpoint` runs inside `ApplicationRoute::ActiveRunContinue` and mutates only its private
  draft after parse and graph validation. This is the correct pre-mutation rejection boundary and will not be replaced.
- `src/game/world/tests/checkpoint_validation.rs` owns an actual validated-data SavePoint fixture, exact 19-field
  semantic parity, strict missing/extra checks, 27 one-edge graph mutations, foreign-seed rejection and
  restore→recapture continuation parity. The intake run passed eight matching library tests.
- The current fixture obtains its document from the production V4 writer. That is not sufficient after the cutover:
  P06 must create a test-only canonical V4 corpus builder from the captured checkpoint payload, independently of the
  V5 writer, and freeze its root/field set plus canonical digest as the legacy import oracle.
- The checkpoint excludes active battle/session state, profile/host metadata, run peak and diagnostic history. P05's
  release lifetime split is already structurally present and must remain unchanged.
- The Core manifest has no direct hash dependency, but the workspace lock already contains `sha2 0.10.9`. P06 may add
  only the direct Core dependency needed for the codec; it must not reopen removed server/web dependencies.

## Approved GEX-10 Design

1. Add exact V4 and V5 document types. Keep the V4 type deserialize-only; make the V5 type the only serializable
   production envelope and set the writer version to `5`.
2. Add a canonical JSON encoder for checksum input. Objects are written in lexical key order, arrays in stored order,
   and scalar spelling is delegated to `serde_json`. Compute/lowercase-hex SHA-256 from those bytes.
3. Decode a strict header, dispatch exactly `4` or `5`, and return a `RunCheckpointPayload` from separately named
   `import_v4_checkpoint` / `import_v5_checkpoint` functions. No best-effort parse, schema aliases, V4 checksum
   invention, V4 writer or dual write is allowed.
4. Verify the V5 checksum before graph validation and before any draft authority field is assigned. Continue then uses
   the existing current-data graph validation and staged application route.
5. Add a code-level removal guard whose default/current certification is false and a negative test that proves a V4
   reader removal request is rejected. This is a policy guard, not a migration-completion claim or live rollout path.
6. Refactor the checkpoint test fixture into a current V5 writer fixture plus an independent canonical V4 input built
   from the completed SavePoint payload. Freeze exact root/field meaning and a canonical corpus digest.
7. Add V4 import→V5 recapture, V5 exact envelope/checksum/round-trip, old-vs-new continuation differential, malformed /
   corrupt / missing / extra / unknown-version matrix, exact retry equality, max/empty semantic fixtures and reader
   removal negative-gate tests.
8. Update the actual Core runtime contract with V5-only writes, explicit V4 import, canonical checksum, strict
   pre-mutation rejection, reader-retention certification and canonical-development-only scope.
9. Re-run P01/P02/P03/P05 currentness. Any expected Cargo/runtime-contract/test locator change must receive a direct
   successor record; no predecessor completion may be assumed from a stale hash.

## Completion Contract

- every new production Core export has exact version `5`; production V4 writer and dual writer count `0`
- V5 checksum detects checkpoint corruption and is stable across object-map iteration/order reconstruction
- canonical V4 imports through the explicit V4 adapter and recaptures as semantically identical V5
- V4 and V5 continuation yield exact identity, restored state, host monotonic metadata and next Core decisions
- malformed, unknown, missing, extra, bad-checksum and graph-invalid input rejects before authority mutation; retry is
  equal and leaves no partial player/run/checkpoint/seed/UUID state
- V5 round-trips empty optional state and the actual maximum/boundary corpus selected by existing Core limits
- actual validated data writer→bytes→reader→runtime producer-consumer route passes
- checkpoint/continuation retains diagnostic history `0`
- certification-before-removal negative gate passes and the explicit V4 reader remains present
- V4 schema alias/best-effort parse/fallback, V4 production write and external compatibility claims `0`

## Validation Dimension Plan

| Dimension | Direct GEX-10/GEX-20 evidence |
| --- | --- |
| `success` | canonical V4 import→V5 recapture and V5 writer/read continuation |
| `invalid_input` | malformed/unknown/missing/extra/checksum/graph matrix |
| `failure_rollback` | full destination authority snapshot equality before/after failed import/restore |
| `retry_idempotency` | repeated failed input equality and repeated capture/continue semantic equality |
| `ordering_concurrency` | single P03 application writer plus competing/stale lifecycle pre-rejection |
| `boundary_empty_max` | empty optional state and exact current max collection/load boundary corpus |
| `disconnect_restart_persistence` | bytes captured from one Core, loaded into a fresh Core, continued identically |
| `live_data` | actual validated data publication and restored identity/reference graph |
| `producer_consumer` | actual Core SavePoint writer→bytes→fresh Core reader→runtime next decision |
| `legacy_fallback_absence` | static/call-site audit: V4 writer/dual write/alias 0; explicit reader and removal gate retained |

## Stop Conditions

- any V4 canonical identity, state, continuation or next-decision meaning cannot be preserved
- checksum verification depends on non-deterministic map iteration or accepts a modified semantic payload
- strict rejection occurs after partial authority publication or a retry differs
- P05 diagnostic history re-enters save/continuation authority
- correct implementation requires external store-envelope, server/client protocol or released/deployed-save assumptions
- evidence of a released/deployed save population appears within the authorized Core evidence set
- predecessor evidence becomes stale without a direct successor-currentness audit
- dirty overlap cannot be isolated safely, or policy/Goal/Phase/node meaning changes

## Pre-edit Identities

- `src/game/world/active_run.rs` `b395a930647a59e365946be712ac5036ef06750b6d6b80662eaa0b3cf18a4c76`
- `src/game/world/tests/checkpoint_validation.rs`
  `b94558c38159defe6e7d34f821caf0446b8a7ec8f52db7b8470833e6088be02d`
- `src/game/world/state.rs` `66b50f568571935e82f708becea67e033ecf2322a1243e4b7971193b52541b52`
- `src/game/world/application.rs` `262ad25bf7610cb8329861465b9de0d7ea9eb7a9678ec7bae20991be5348d1d6`
- `Cargo.toml` `ec456bbe068e8009575d6168986d2bddfa4e14a34f76bbaaa866e843f2af2025`
- `../Cargo.lock` `e1597f3369caa9637fb7e41c6ca5230b0954926610bc661933123a446ba17836`
- `docs/core_runtime_contract.md` `e5951267d4affdc14ea08b617305cb4ad701b6ccc5a2e0b6532ddfc95bf312b2`

These hashes were captured immediately after GEX-00 and before the first P06 production edit. All listed source paths
are pre-existing dirty/mixed user work; P06 owns only exact checkpoint-migration hunks.

## GEX-10 Implemented Boundary

- The production serializer is now exact V5 `{version, checkpoint, checkpoint_sha256}` only. The V4 document is
  deserialize-only and reachable solely through the named `import_v4_checkpoint` adapter; version dispatch accepts
  exactly 4 or 5 and never retries another schema.
- V5 checksum is lowercase SHA-256 over recursively canonical JSON. All object keys sort lexically, arrays retain
  order and scalar spelling uses `serde_json`; typed deserialize/reconstruction and reordered envelope bytes verify
  with the same semantic checksum.
- Checksum verification and current validated-data graph validation complete before the existing
  `ApplicationRoute::ActiveRunContinue` draft assigns any authority. Malformed/strict/checksum/graph failures and
  retries preserve the full destination snapshot.
- The canonical V4 corpus is constructed test-only from the completed SavePoint payload without invoking the V5
  writer. Its exact 19-field meaning is frozen at semantic SHA-256
  `64f78ce035c38e523c95b14456296750aa2a04a1b43f3f0f00b5c101d608bcc2`.
- V4 and V5 continuation into fresh Core instances produce equal state, map and next decision, and both recapture as
  equal V5. Exact 50-slot inventory reservation, overflow, maximum load count, empty export and active-lifecycle
  contention are directly covered.
- The removal-certification constant remains false and its negative gate rejects the exact removal request. This is
  reader-retention evidence, not released/deployed population certification.
- Direct `sha2 0.10.9` dependency and mechanical workspace/probe lock resolution are frozen as P06 codec deltas. The
  actual Core runtime contract states V5-only write, canonical checksum, explicit V4 import and Core-only scope.
- P01 through P05 all have direct `Complete / High` successor-currentness records. Full Core passes 1,099 library plus
  51 integrations = 1,150; default release is warning-free; every predecessor positive/negative release probe retains
  its expected result.
