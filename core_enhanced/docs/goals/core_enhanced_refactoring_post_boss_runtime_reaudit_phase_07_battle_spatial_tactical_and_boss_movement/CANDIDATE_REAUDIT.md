# Phase 07 Candidate Reaudit

## Current Dispositions

| Candidate | Current evidence and Boss impact | Disposition |
| --- | --- | --- |
| CRA-P07-C01 | relocation/swap still apply tile focus/rest before final field-work reconciliation; an `Any` profile worker that remains assigned across Ground/Platform can retain `FieldWorking` while receiving incompatible tile effects | `still_valid` |
| CRA-P07-C02 | deploy/withdraw/stance/relocate/swap mutate BattleCore, resources, deployment state and revision before `battle_update_dto_after` validates the caller cursor; no shared staged tactical transaction exists | `still_valid` |
| CRA-P07-C03 | production defaults to Rapier, but the Direct backend, runtime selector and parallel policy assertions remain; Boss directives consume the ordinary movement leaf and do not close the backend consolidation | `still_valid` |
| CRA-P07-C04 | `MovementTickInput`/unit plumbing and output variants still carry fields or signals with no production consumer, including private physics-handle and goal/segment scaffolding | `still_valid` |

## Deep Evidence

### C01 — final-state tactical effects are not one derived transition

`apply_tactical_position_update` subtracts and reapplies Ground focus, Platform block state and Platform rest cadence
from the old/new tile before `reconcile_field_work` runs. Field-work reconciliation computes only the difference of
the old and new worker sets. If the same `Any`-profile unit remains a worker while moving or swapping across tile
kinds, it is in neither set difference: its activity stays `FieldWorking`, but the earlier position update can add
Ground focus or schedule Platform rest. The current field-work relocation/swap test uses Ground cells and therefore
does not exercise this aggregate state.

The original correction remains one root cause: derive the complete final tactical aggregate from
`(activity, stance, tile kind, combat profile)` after final occupancy and apply it once. This remains independent of
Boss target/path logic.

### C02 — tactical publication is not failure-atomic

The world handlers preflight revision capacity, then mutate the live BattleCore and adjacent world state, commit the
projection revision, and only afterwards call `battle_update_dto_after`. That DTO method is where a future
`after_seq` is rejected. Thus a returned `InvalidBattleResyncSeq` can follow a committed command. Deploy and withdraw
also span deployment maps, resource payment, live signals and employee-attempt reconciliation; the stance,
relocation and swap handlers share the late cursor failure shape. The staged battle-advance transaction closed in
P06 is limited to clock advancement and does not cover these commands.

Existing “atomic” tactical tests prove selected validation rejections do not partially mutate BattleCore. They do
not inject the named post-mutation cursor/reconciliation failures or compare the complete world state. The original
staged/preflight correction therefore remains current.

### C03/C04 — backend and ghost-contract cleanup remain gated

`ContinuousMovementBackend` still has Direct and Rapier variants, test-only switching, and a large Direct assertion
surface. Rapier is the default production backend but also shares correction policy with the Direct implementation.
Removing Direct requires transferring unique deterministic movement and clamp/obstacle assertions to the retained
backend and confirming accessible/external Rust consumers before narrowing the public surface.

`MovementTickInput`, per-unit movement data and outputs still expose unused owner/target/physics or
goal/segment-oriented members. Cleanup remains conditional on C03 and a complete consumer inventory so that event
publication and useful backend conformance checks are not accidentally removed.

## Boss Movement Reconciliation

- Boss `MoveToTarget`/`MoveAndRejoin` produces a private goal/suppress/ordinary-fallback directive.
- The existing planner consumes that directive, and actual translation remains in the ordinary continuous movement
  leaf. Target identity, route cursor monotonicity, movement lock and timeout are Boss-owned; collision/clamp/static
  obstacle behavior remains movement-backend-owned.
- Consequently C03/C04 stay deferred/conditional exactly because their cleanup must preserve this ordinary leaf.
  C01/C02 remain independent tactical corrections.
- No distinct `CRR-P07-*` root cause was found. Boss integration adds consumers and assertions, not a second movement
  authority or a new public state contract.
