# Phase 10 Boundary Audit

## Authority Result

- Core domain handlers own gameplay command semantics; the server owns four persistence/recovery commands. A shared
  descriptor may classify ownership but must not move host recovery into Core dispatch.
- `BehaviorResult` owns semantic outcomes and Core owns projection content. Server owns transport envelopes,
  publication ordering and genuinely server-derived persistence overlays.
- Core battle setup/update DTOs own serialized battle content. Server owns top-level setup/update choice and resync
  wrapping, not a duplicate field schema.
- Core snapshot capabilities and server recovery capabilities are separate typed inputs; their flattened merge and
  fatal replacement semantics need one typed host overlay.
- `GameError` owns stable code identity; Core/server adapters may continue to own context-specific messages.

## Boss Public/Private Boundary

Boss catalog/setup is injected into Core, then runtime state and `BossRuntimeWake` stay private to battle simulation.
The wake is a scheduler event, not a `BattleLogEvent`; public event DTOs, checkpoint DTOs and server messages contain
no Boss lifecycle branch. Accessible C# source also contains no such carrier. Thus the original conditional rule is
confirmed: extend C03 only after a real first public Boss wire exists. Current work must preserve existing-event and
checkpoint compatibility and must not serialize private runtime state.

## Policy And External Gates

- C02 requires external public-Rust helper/API inventory; current RunComplete shape is frozen unless separately
  migrated.
- C03/C04 require exact wire/root parity and authorized server/external fixture comparison; Unity/deployed consumers
  beyond accessible source remain unverified.
- C06 requires standalone package/logger/generator consumer inventory before dependency or API removal.
- Admin transaction/publication semantics remain a deferred policy signal, not part of the seven-row current catalog.
- C01, C05 and C07 are policy-neutral after their named tests; this campaign still does not authorize implementation.

## Reconciliation And Handoff

All seven original rows remain valid. C06/C07 use normalized `cleanup`; C01~C05 remain
`behavior_preserving_refactor`. No new row is handed to Final. The Final phase must retain the C03 first-public-Wire
condition as a gate, not as evidence of a current Boss DTO candidate.
