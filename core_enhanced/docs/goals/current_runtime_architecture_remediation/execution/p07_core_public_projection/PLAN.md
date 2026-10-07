# P07 Core Public Projection Execution Plan

상태: `correction GEX-10 exit criteria met / affected predecessor re-audits current / replacement target freeze pending`

## Current Execution Identity

- Handoff / root target: `HA-CRAR-P2G60-002` / full `GS-CRAR-P2G50-002` goal-run.
- Selected member: `(GS-CRAR-P2G50-002, GOAL-CRAR-C2-P07-CORE-PUBLIC-PROJECTION)`.
- Phase / execution node: `PHASE-CRAR-C2-07-CORE-PUBLIC-PROJECTION` / `WP-CRAR-C2-08`.
- Goal Quality Contract: `2`; durable seam `IVA-C2-08` (`IVA-CRAR-C2-08` in the canonical Phase Plan).
- Policy / decisions: `PER-CRAR-007-R6-CODE-REVIEWED`, `DR-CRAR-007-R2`, realization `TD-CRAR-017`.
- Current pass: separated correction GEX-10 after Master GEX-30 Attempt 2 found one runtime `cfg!(test …)` public
  projection switch contradicting P08 structural evidence.
- Authorization: Core public application types, ports, mappers, read models, visibility, tests, Core-owned probes and
  actual Core runtime contract only. `game_server`, Unity, sibling resources, external schema/wire compatibility,
  external consumers and rollout are excluded and were not inspected.

## Direct Dependencies and Currentness

- P03 through P06 are `Complete / High`. P07 consumes the single staged application authority, private battle kernel,
  bounded history/result semantics and V4-import/V5-write checkpoint boundary; it does not create a second writer.
- Current predecessor target identities are P01 `_006`, P02 `_005`, P03 `_004`, P04 `_003`, P05 `_002` and P06
  `ATR-CRAR-C2-P06-GEX10-001`. Any P07 source, manifest, contract, probe or test delta that changes a frozen member must
  receive a direct successor-currentness record.
- P08 remains the sole performance/non-regression owner. P07 records functional projection correctness and sizes only;
  it does not optimize or claim a performance PASS.

## Objective and Exact Meaning

- Publish one transport-neutral Core application façade owning `ApplicationCommand`, `ApplicationResult`,
  `ApplicationError`, `ApplicationSnapshot` and read-model documents.
- Keep the authoritative `GameCore`, its raw behavior/result projection and transaction rollback handles behind the
  crate boundary. The façade may own a private Core instance, but callers receive no domain aggregate, mutable field,
  commit token, rollback token, battle kernel or raw domain result.
- Map every current Core player command family and every current Core result/error meaning explicitly. Mapping is a
  compile-time exhaustive match plus a characterized corpus; no unknown command/result fallback, wildcard semantic
  arm, best-effort alias or dual legacy public API is allowed.
- Project snapshots and specialized application operations (battle advance/disconnect, admin, profile and checkpoint)
  through the same application-owned result/error/read-model vocabulary. Checkpoint bytes remain the P06 exact codec;
  P07 changes only their public ownership wrapper.
- Preserve observable Core meanings. This Goal does not define an external wire version, server/Unity mapper, schema,
  compatibility period, consumer cutover or release claim.

## GEX-00 Current Inventory

- `src/game/mod.rs` exposes `world` and `behavior` publicly. `src/game/world.rs` exposes constructible `GameCore` plus
  direct mutating execute/admin/battle/profile/checkpoint methods and direct DTO/JSON snapshot methods.
- The public command path is `PlayerBehavior` (58 characterized `ActionKind` members) into `BehaviorResult` (49
  characterized result variants) or serde-enabled `GameError`. This is the frozen legacy locator
  `CRAR-LEGACY-006-UNVERSIONED-WIRE` in the 57-feature manifest.
- `BehaviorResult::command_result_projection` and `into_command_result_contract` already contain exhaustive result
  characterization, but the types live beside the domain result, reuse domain-shaped payload types and collapse twelve
  distinct battle result meanings into one battle-update branch. They are an oracle/reuse seam, not the final public
  boundary.
- `GameCoreState`, application route/owner, prepared commit, rollback content and live battle state are private to the
  `world` module. The defect is therefore public aggregate construction and raw projection ownership, not exposed
  state fields.
- The actual external Core callers in scope are task-owned compile probes and package binaries. Integration tests do
  not construct `GameCore`; their direct battle/data test APIs are outside this projection cutover and remain governed
  by P02/P04 test-capability decisions.
- Existing run/inventory/roster/selected-event/action-capability projections can be serialized before crossing the new
  boundary. Their source remains the closed lifecycle/kernel/history ports; the façade must not reconstruct them by
  accessing domain fields.
- Public operational routes beyond the 58 behavior commands are admin execution, elapsed/server-tick/event-bucket
  battle advance, disconnect pause, profile import/reset and checkpoint export/continue. Transaction rollback is an
  internal staging mechanism and must not remain a public application capability.

## Approved GEX-10 Design

1. Add a root public Core-owned application module and façade. Its state field is private and it exposes no borrow,
   dereference, escape hatch or conversion back to `GameCore`.
2. Define an application-owned 58-member command kind and immutable command document. Decode/validate the exact
   current behavior shape before authority access, then map through an exhaustive kind↔domain-action match. Unknown
   kind, malformed fields and kind/payload mismatch become typed `ApplicationError` without mutation.
3. Define application-owned result kind/disposition and immutable read-model document. Exhaustively map all 49 result
   variants exactly once, preserving each distinct result identity including all battle variants. Recursive completion
   payloads retain their result identity and ordering.
4. Map every `GameError` variant into a stable application error code plus immutable detail document with an exhaustive
   match. Projection/serialization failure is prepared before commit or represented as an accepted recovery-pending
   application outcome; it must never turn a committed operation into `Err`.
5. Expose run, inventory, roster, selected-event and action-capability snapshot queries through typed snapshot kinds.
   Empty optional snapshots are explicit `null`; collection order and revision/event cursor semantics stay unchanged.
6. Route battle advance/disconnect, admin, profile and checkpoint operations through façade-owned ports returning only
   application result/error/snapshot/checkpoint types. Remove public rollback capture/restore rather than projecting a
   mutation token.
7. Hard-cut the Core public root: make `world` non-public and the raw `PlayerBehavior`, `BehaviorResult` plus their
   legacy contract/projection families crate-private. Retain internal domain serde only as a private mapper mechanism;
   there is no public alias or dual façade.
8. Migrate the package binary and all Core-owned valid compile probes to the new façade. Add a P07 valid probe and a
   negative probe proving `GameCore`, raw behavior/result, rollback and façade internals cannot be named or mutated.
9. Add exhaustive 58-command, 49-result and every-error mapping tests; old/new semantic parity, malformed/retry
   rollback, query idempotency, ordering/cursor, empty/large snapshot, checkpoint restart, actual façade round-trip and
   legacy absence tests.
10. Update the 57-feature manifest's legacy locator and the actual Core runtime contract. Re-run predecessor
    successor-currentness, focused probes/selectors, full Core, release warnings and manifest verification.

## Completion Contract

- all 58 current Core player command meanings have exactly one public application kind and one private domain mapping
- all 49 current Core result meanings, including twelve distinct battle identities, have exactly one application result
  kind and parity payload
- every current Core error variant maps to a stable typed application error with no wildcard/fallback semantic arm
- run/inventory/roster/selected-event/action-capability and bounded battle resync meanings use application snapshots or
  read models only; empty and large collections are unambiguous and untruncated
- invalid command/projection/codec/graph input and retry preserve exact authoritative state/event/revision/durable
  intent; query/repeated projection is mutation-free and identical
- application service→public caller, V5 checkpoint restart/continue and live validated-data routes pass
- public `GameCore`, raw `PlayerBehavior`/`BehaviorResult`, transaction rollback handle, legacy command-result contracts,
  domain-result serde path and alternative public mutation writer count `0`
- external source/schema/fixture/compatibility/consumer inspection and claims count `0`

## Validation Dimension Plan

| Dimension | Direct GEX-10/GEX-20 evidence |
| --- | --- |
| `success` | normal façade command/result plus all 58/49 exhaustive mapping corpus |
| `invalid_input` | unknown/malformed/mismatched command and exhaustive typed error mapping |
| `failure_rollback` | pre/post authority identity for decode, domain and projection failures |
| `retry_idempotency` | repeated invalid command and repeated snapshot/result/error projection equality |
| `ordering_concurrency` | event/revision/cursor ordering and single P03 application writer audit |
| `boundary_empty_max` | null selected event plus existing largest deterministic snapshot/result corpus |
| `disconnect_restart_persistence` | disconnect projection and P06 V5 export→fresh façade continue→next decision |
| `live_data` | actual `ValidatedGameData::load_live_embedded` façade construction and projection |
| `producer_consumer` | actual Core façade→public API compile-probe round-trip; no external substitution |
| `legacy_fallback_absence` | release privacy probe/static audit for raw Core/result/rollback/dual façade absence |

## Stop Conditions

- a Core semantic mapping requires guessing a server/Unity/external wire field or compatibility behavior
- any current command/result/error meaning cannot be represented without exposing domain mutation authority
- a mapping/projection fault can occur after commit and be returned as `Err`, or invalid input changes authority
- P03 single-writer, P04 private-kernel, P05 bounded-history or P06 checkpoint meanings would be weakened
- a public raw-domain alias, dual writer, wildcard semantic fallback or compatibility shim is required
- predecessor evidence becomes stale without a direct successor-currentness audit
- dirty overlap cannot be isolated safely, or policy/Goal/Phase/node meaning changes

## Pre-edit Identities

- `src/lib.rs` `550c6c2a3d84d0a97151e113aa8b0f3d37b8e06987e2b93e93a9649fadb4135f`
- `src/game/mod.rs` `4a55e852321a89a5abf3226e84dedeb9e7a0934d7378e8e82b511a0869f77442`
- `src/game/behavior.rs` `620cca3a12c5be93553ecaa294ca8d8819cf045ff3a2b1a0d19f9e9ac8ad11ed`
- `src/game/world.rs` `4821030666dbf7b0b0e40ac05b2a527659dc54a7b4728e638dc4e0e8698cc82e`
- `src/game/world/application.rs` `262ad25bf7610cb8329861465b9de0d7ea9eb7a9678ec7bae20991be5348d1d6`
- `src/game/world/active_run.rs` `0bba5a9a9284302fd0b60049589be23bcbdff9a7bcba1b37dc72655b90e91a56`
- `src/game/world/admin.rs` `701165fb609d559994c7299cdf872115bfc2b6aa8180a73aa24373102041a809`
- `src/game/world/combat.rs` `d9d65b3316adf3b70d25d2f95a3292563f727750830373f58b2215d0ebef4b6a`
- `src/game/world/snapshot.rs` `704ad2dc9baa8b7ffe1f5f2c16fb94418faf46bc8c1d432145870b3ef58aa0b6`
- `docs/core_runtime_contract.md` `56060826753d66af3cd10c725b351cb74a41d219b67cbf4033b1cf9192098a4e`
- `docs/architecture/runtime_feature_manifest.v1.json`
  `c25d3e974df465903b53640df689a583115299aad3fe066f0d63257e717313c7`

These hashes were captured immediately after GEX-00 and before the first P07 production edit. All listed paths are
pre-existing dirty/mixed user work; P07 owns only exact public-projection hunks.

## GEX-10 Implemented Boundary

- Root `application` now owns `CoreApplication` and immutable command/result/error/snapshot/checkpoint/admin documents.
  `CoreApplication` has one private `GameCore` field and no borrow, dereference, conversion, commit or rollback escape.
- `ApplicationCommandKind` is an exhaustive 58-action bijection. The command document decodes to the private behavior
  before authority access; malformed, unknown and missing payloads return `InvalidCommand`.
- `ApplicationResultKind` names all 49 actual player result meanings, including both previously uncharacterized
  Mastered Elite results and twelve distinct battle result identities. The corrected old/new corpus proves exact
  meaning and payload parity. Read-only reconnect retains the legacy `BattleAdvanced` payload identity while the
  application disposition is explicitly `ReadOnly`.
- All 28 domain error variants map by exhaustive match into stable application kinds/codes/details. The four
  application-local input/projection meanings are separate. No wildcard semantic fallback exists.
- Seven snapshot kinds are exhaustive, typed and repeatable. Empty selected event is explicit `null`; the live battle
  probe proves repeated bounded reconnect equality and the checkpoint test proves V5 export into a fresh application
  continuation.
- `game::world`, `GameCore`, `PlayerBehavior`, `BehaviorResult`, legacy command-result contracts and rollback tokens
  are not nameable from default release. Package fixture and all four predecessor positive probes use the application
  façade; five positive probes pass and five negative probes fail with the expected privacy/capability errors.
- The 57-feature manifest verifier understands an asserted removed locator and binds the canonical application
  replacement. It passes 57/57, 44 locator contents and zero errors. P01 through P06 each have a direct
  `Complete / High` successor-currentness record.
- Full GEX-10 Core passes 1,106 library plus 51 integrations = 1,157. Formatting, diff hygiene, release warnings,
  public rustdoc and manifest verification pass. No performance PASS is claimed; P08 remains the owner.

## Current Correction GEX-10

- Finding: `src/game/world/snapshot.rs` projected diagnostic availability through runtime
  `cfg!(any(test, debug_assertions, feature = "test-fixtures"))`, contradicting the final zero runtime-test-switch
  structural claim.
- Exact correction boundary: retain the approved configuration meanings while replacing the runtime switch with
  mutually exclusive compile-time `#[cfg]` constants. No field, default-release value, enabled diagnostic value,
  external schema or public authority changes.
- Pre-edit source SHA-256: `704ad2dc9baa8b7ffe1f5f2c16fb94418faf46bc8c1d432145870b3ef58aa0b6`.
- Required exit: focused configuration/source check, release/application/full Core and manifest/API validation, direct
  predecessor currentness review, then a replacement immutable P07 target and fresh GEX-20.
- Exit evidence: runtime switch search `0`; default and `test-fixtures` release checks pass; snapshot `1/1`, application
  `9/9`, event log `8/8`, result stats `7/7`, checkpoint validation `16/16`; manifest `57/57`; all eleven API probe
  outcomes expected; full Core `1,159/1,159`.
- Current predecessor records after the correction: P01 `_009`, P02 `_007`, P03 `_007`, P04 `_005`, P05 `_005`, P06
  `_003`; all `Complete / High`.
