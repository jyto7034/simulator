# Core Enhanced Refactoring Audit Phase 10 — Boundary Audit

Audit Date: 2026-08-07
Status: complete
Scope: Core command/projection/package surface and direct `game_server` seam

## Boundary Verdict

The assigned boundary is fully traced. Core remains gameplay mutation and normal command-result authority; the
server remains persistence, session, recovery overlay and publication-order authority. The highest duplication is
not a second gameplay implementation: it is repeated command metadata, repeated result/error projection, untyped
snapshot overlay and field-by-field battle DTO transport mapping around the authoritative Core.

Four recovery operations deliberately cross this boundary: they use the Core `PlayerBehavior`/`ActionKind` wire
vocabulary but are implemented by the server host. Admin commands are separate again and do not currently share the
ordinary persistence/rollback/publication transaction. Those facts require explicit owner types and policy gates;
they do not authorize moving domain rules into the server or silently changing protocol shape.

## Exact Inventory And Disposition

| Surface | Current authority/use | Boundary disposition |
| --- | --- | --- |
| `game/behavior.rs` | player carrier, action metadata, result/error/battle DTOs and wire conversion | primary; C01/C02/C05 |
| `game/world.rs` | transport-context validation and domain dispatch | retain authority; C01 consolidates metadata only |
| `world/action_capability.rs` | common capability evaluator and projected root actions | retain shared preflight; C01 |
| `world/battle_capability.rs` | revisioned battle capability evaluator | retain shared preflight; C01 |
| `world/{snapshot,state}.rs` | typed Core snapshot and snapshot-local error projection | retain DTO authority; C04/C05 |
| `world/combat_trace.rs` | optional event-log attachment carrier | retained; server compression is transport-only |
| `world/admin/**` | separate admin fixture/grant mutation | retained separate; admin transaction policy deferred |
| Core package/tooling | manifest, empty auto binary, logger hook, member-local CI/config | package cleanup cluster C06/C07 |
| Core fixture bins | contract generators | retained; historical external provenance, current consumer unverified |
| direct server actor seam | host recovery, persistence, rollback, message mapping/order | supporting consumer; C01/C03/C04/C05 |
| server fixture examples | exact contract generators | retained; historical Unity provenance, current Unity unverified |
| durable Core/server docs | intended ownership and wire ordering | supporting canonical; current runtime wins on stale history |

Cargo metadata confirms the workspace root is `/mnt/f/work/simulator`, the Core library, an empty `game_core` binary,
five Core fixture binaries and five integration tests. The empty `tests/test_actor/mod.rs` has no target/module edge.

## Player Command Trace

1. The server deserializes a `PlayerBehavior`, preserves `request_id`, and supplies optional battle authority.
2. Four host-recovery variants are intercepted. Ordinary commands reach
   `GameCore::execute_with_transport_context`.
3. Core checks battle UUID/revision for battle-authority actions before mutation, then reuses the same common or
   battle capability evaluator that projection uses.
4. Core dispatches to the Phase 1–9 domain owner, translates attached VN/battle context and returns one
   `BehaviorResult`.
5. The server compares exported active-run checkpoints, persists a changed nonterminal run, invalidates terminal or
   explicitly replaced runs, and restores the captured active-run aggregate if that persistence fails. Encounter
   compendium persistence remains an independent monotonic aggregate.
6. Core's result contract becomes a command response and optional side messages. The actor emits response, then side
   messages, then an optional root snapshot. The session waits for that actor-owned batch and does not duplicate it.

This trace confirms execution authority, but also corrects an overly broad reading of the Phase invariant: rollback
after a host persistence failure is orchestrated by the server using Core capture/restore primitives, not completed
inside `execute_with_transport_context` itself.

## Command And Capability Authority

`ActionKind` variant identity is repeated through `scope`, `requires_battle_authority`, `PlayerBehavior::kind`,
projected action arrays, common/battle owner matches, Core dispatch and the server intercept. Rust exhaustiveness
protects many matches, but arrays and membership sets are not exhaustive. A typed route/descriptor can consolidate
metadata without deciding gameplay availability.

Common and battle capability evaluators are aligned with execution preflight for migrated slices. Payload-specific
validation may still return a more precise error after an aggregate capability is enabled; that is intentional and
does not make the DTO a competing authority. Root capabilities are empty during battle; the revisioned battle update
owns current battle actions. This matches the durable protocol.

`ContinueRun`, `ResetActiveRun`, `RetryEncounterCompendium` and `ResetEncounterCompendium` are host-owned despite
their Core carrier. Their current wire names, capability rules and host persistence behavior are preserved invariants.

## Result And Error Authority

`BehaviorResult` is public and serde-capable, then converted field by field into
`BehaviorCommandResultContract`/domain enums, which separately map variant to `result_type` and rebuild JSON
payloads. The server properly consumes that Core contract, but the Core has three representations that can drift.
`RosterCommandResult` is unconstructed, and several public result convenience methods are workspace-test-only.

The current `RunComplete` command payload is the map object directly, not `{ "map": ... }`. Neither semantic naming
nor lack of an exact server test authorizes changing it. C02 freezes this shape first; a shape change is a separate
compatibility migration signal.

`GameError` stable codes are independently matched twice in Core snapshot projection and once in the server. Current
codes agree. Human messages intentionally differ, so C05 centralizes code identity only.

## Snapshot, Battle DTO And Publication Authority

Core owns `RunSnapshotDto`, `LiveBattleSetupSnapshotDto` and `LiveBattleUpdateDto`. The server generically flattens
the root snapshot but mutates serialized JSON to merge host recovery capabilities and recovery metadata. It also
redeclares every battle setup/update field in its message enum. C03 and C04 remove these re-authoring seams while
freezing exact top-level JSON.

Observed ordering is internally consistent:

- accepted battle mutation: `CommandAccepted` then setup/update side messages, no root snapshot;
- stale/missing authority: error then current resync, no mutation;
- ordinary command: command result then side messages then optional root snapshot;
- battle tick completion: final battle update then root snapshot;
- reconnect in active battle: root snapshot then setup/update recovery batch.

The server does not interpret Phase 9 range cells. It forwards typed checkpoint/request previews. Direct server
search found no `range_units` consumer.

## Admin, Fixture And Package Boundary

Admin uses a distinct top-level command. Core admin tests exercise live typed fixtures, grant catalog resolution,
failure immutability and resource/condition mutation. The server gates admin by environment/token, but its admin path
does not participate in normal active-run persistence/rollback or battle side-message publication. Because no durable
admin transaction policy defines the intended behavior, unification is deferred as policy work.

Core has no `build.rs`, yet four build dependencies remain. Static source/test inventory found a large direct
dependency set with zero lexical use. The empty main binary, empty ctor hook, unconsumed Core logger,
`tokio_unstable` rustflag and member-local coverage workflow belong to the package cleanup cluster. GitHub's current
official workflow documentation says discovery searches `.github/workflows` at repository root; the audited workflow
is instead under `core_enhanced/.github/workflows`. Standalone packaging/automation remains unverified, so cleanup is
gated rather than assumed.

Fixture generators are not runtime authority and their zero-test compilation is not coverage. Historical Goal
evidence shows exact Unity fixture comparisons for server examples and the Core event fixture. Current Unity is out
of scope, so those tools are retained until a current external-consumer inventory exists.

## Predecessor Seam Reconciliation

| Phase authority | Phase 10 carrier/consumer conclusion |
| --- | --- |
| P01 live data | snapshots/results serialize derived values; no competing load/validation authority found |
| P02 run/map/checkpoint | Core owns payload; server owns durable publication/recovery; typed host overlay C04 |
| P03 roster/inventory/economy | result and snapshot fields are carriers; duplicate result projection C02 |
| P04 node/VN | shared capability preflight and VN attachment are aligned; retain focused gating tests |
| P05 preview/setup | Core owns setup DTO; server field copy is C03, not a setup-rule owner |
| P06 battle lifecycle/result | result/event-log carriers remain Core; server compression/order is transport-only |
| P07 spatial/tactical | revision/capability DTOs remain Core; C01/C03 consolidate the seam |
| P08 effects/stats | presentation/result DTOs carry effects; no server effect inference found |
| P09 targeting/delivery | server forwards previews; C04 policy and C07 external range cleanup remain unresolved gates |

## Coverage And Confidence

All P10-01 through P10-09 boundary questions have direct source/test evidence. External Unity, standalone Core
packaging and admin durability policy are explicitly unverified rather than treated as confirmed. Focused validation
executed 86 unique tests with zero failure/ignore after static gaps were recorded. Boundary confidence is High.
