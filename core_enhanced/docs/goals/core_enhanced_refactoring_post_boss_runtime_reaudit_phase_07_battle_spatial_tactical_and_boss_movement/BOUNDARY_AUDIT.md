# Phase 07 Boundary Audit

## Authority Result

- BattleCore owns tactical tile/activity/stance effects and continuous movement state.
- World command handlers own roster/deployment/resource/revision publication and therefore own C02 transaction
  completeness; P06's clock-advance staging is a sibling transaction, not a replacement.
- Rapier is the current production translation backend. The planner remains the shared authority for ordinary
  movement goals and blocking/route policy.
- Boss runtime owns target identity, authored path/rejoin intent, task deadlines and cancellation. It publishes a
  private movement directive into the existing planner/backend leaf and does not own collision or tactical commands.

## Boss Goal/Suppress/Fallback Gate

The three directive outcomes remain semantically distinct:

- `Goal` supplies the authored Boss point/target to the ordinary movement leaf.
- `Suppress` prevents ordinary fallback while authored Boss work intentionally holds movement.
- `OrdinaryFallback` preserves non-Boss/idle behavior.

C03/C04 implementation must keep this seam or replace it with an equivalently tested private contract. Backend
cleanup must not move Boss target selection into Rapier, and Boss cleanup must not duplicate clamp/obstacle policy.

## Public And External Boundary

Workspace searches show production construction selects Rapier and Direct selection is confined to movement tests,
but publicly reachable Rust types and inaccessible/deployed consumers are not proven absent. C03/C04 therefore keep
their public/external inventory gate. No serialized Boss movement runtime, server DTO or Unity-owned movement state
was identified in this Phase; those public projection boundaries remain assigned to P10.

## Reconciliation And Handoff

All four original candidates remain valid. C01/C02 are independent correctness corrections; C03/C04 remain
deferred/conditional cleanup in the presence of the Boss movement consumer. No `CRR-P07-*` candidate is handed to
Final. P08 may rely on the private Boss-to-ordinary-leaf boundary but must independently audit effect/stat leaf seams.
