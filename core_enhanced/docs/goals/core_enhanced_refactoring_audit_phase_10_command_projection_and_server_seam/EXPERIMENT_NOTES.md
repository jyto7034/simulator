# Core Enhanced Refactoring Audit Phase 10 — Experiment Notes

Status: complete

## Boundary decision

- Primary authority: player/admin command carrier → Core execution/validation → derived result/capability/snapshot →
  direct server serialization/publication.
- Phase 1~9 domain mutations are inputs and are not reopened because behavior/world host files route them.
- Server is supporting evidence only at its direct Core bridge; Unity remains unverified/out of scope.

## Dirty precautions

- Current dirty Core/server tree is the baseline and belongs to the user.
- Exact source/test/canonical/status hashes precede detailed command/projection reads.
- Only Master/Phase audit documents are writable; fixture generators are read, never executed for output.

## CRA-14 policy

- A command/projection test is meaningful when it observes typed validation, authoritative state/no-mutation,
  rollback, exact result/error shape, capability parity, snapshot revision or server publication ordering.
- JSON/fixture equality is meaningful only when tied to a current consumer and explicit compatibility contract.
- A generator, example, autobin or test module is not production authority merely because it compiles.

## Handoff condition

Final consolidation stays inactive until exact Core/server boundary, candidate/test artifacts, no-mutation proof and
an independent `Complete / High` audit are complete.

## Authority decisions

- Core domain handlers remain gameplay mutation authority. Capability evaluators are shared preflights/projections,
  not a replacement authority.
- The four recovery commands are server-host operations despite sharing the `PlayerBehavior` carrier. Their wire
  names are compatibility surface and cannot be deleted merely because Core rejects direct dispatch.
- Core owns normal command-result semantics and battle DTO contents; the server owns transport envelope, durable
  persistence, reconnect ordering and host-recovery overlays.
- Stable error code identity belongs with Core `GameError`; user-facing server wording may remain adapter-owned.
- Admin commands remain a separate authority. Unifying their transaction/publication path requires an explicit
  durability and revision policy.

## Compatibility and external gates

- Preserve current result type strings, payload shape and battle top-level tags during any behavior-preserving work.
  `RunComplete` direct-map payload is specifically frozen until a compatibility decision says otherwise.
- Phase 9 C04 remains blocked on per-step versus union effect-preview policy and external UI shape. Phase 9 C07 has
  no direct server `range_units` consumer, but external/public consumers remain unverified.
- Core/server fixture generators have historical exact-fixture provenance. Current Unity and standalone Core
  packaging are outside scope, so absence of workspace runtime references is not deletion proof.

## Package finding classification

- Empty `tests/test_actor/mod.rs` is confirmed disconnected cleanup.
- The empty auto binary, inactive nested workflow, no-build-script build dependencies, unused dependencies,
  `tokio_unstable` flag and unconsumed Core logger are one package/tooling cleanup cluster. It is high-amplification
  evidence, but implementation must start with Cargo target/dependency checks and an external standalone-use gate.
- Fixture binaries are not folded into that cleanup cluster until their consumers are inventoried.
