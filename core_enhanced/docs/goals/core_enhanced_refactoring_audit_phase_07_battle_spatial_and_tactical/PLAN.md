# Core Enhanced Refactoring Audit Phase 07 — Battle Spatial And Tactical Goal

Status: complete
Implementation Complete: not_applicable
Goal Type: phase
Parent Master: `core_enhanced_refactoring_audit_master`
Phase: 7
Execution Mode: investigation_only
Production Mutation Authorized: no
Audit Baseline Date: 2026-08-07
Start Commit: `f318414fb0156a67e04c5479000c2a2683fc0c09`

## Objective

현재 working tree의 battlefield/occupancy, continuous movement, deploy/withdraw, tactical resource, stance,
relocation, swap과 field-work authority를 runtime, published policy, canonical과 test-code evidence로 감사한다.
Position/occupancy/path/timing and transactional tactical command state가 중복되거나 drift하는 신호를 실행 가능한
후보 또는 rejected/deferred/unverified disposition으로 분류하며, effect와 targeting 의미는 후속 Phase carrier로만
취급한다.

## Parent Scope And Direct Dependency

- Parent: `core_enhanced_refactoring_audit_master`
- Boundary: Phase 0 `BOUNDARY_PROPOSAL.md` and `SURFACE_INVENTORY.md`
- Direct predecessor: Phase 6 `Complete / High`
- Phase 6 handoff: execution clock/advance/order corrections are recorded; this Phase consumes current clock and
  does not duplicate C01~C03.
- Phase 5 owns battlefield/scenario construction before execution.
- Phase 8 owns effects/stats/death; Phase 9 owns targeting/delivery; Phase 10 owns public projection/server mapping.
- Phase 8 is denied until this Phase has `Complete / High`.

## Approved Requirements

Parent CRA-01~CRA-14 all apply. Every candidate needs runtime/test/canonical evidence, current→target authority,
preserved behavior, removal scope, seven axes, readiness and policy/external decision. Every assigned test family
needs meaning/reachability/assertion/overlap disposition.

## Invariants

- Battlefield valid tiles/static obstacles plus runtime unit/object occupancy have one authoritative mutation path.
- Continuous movement uses authoritative battle time and deterministic path/steering/backend state without a second
  position model.
- Deploy, withdraw, redeploy, stance, relocation, swap and field work validate exact current state before cost,
  cooldown, position, revision or employee mutation.
- Successful tactical commands pay/commit once and project one revision; failure leaves all tactical/spatial state
  unchanged.
- Pause freezes time but does not forbid approved instantaneous commands; cooldown/locks remain battle-time based.
- Phase 8 effect/damage/death and Phase 9 target/range/delivery semantics are not decided here.
- No production code/data/test/canonical/server/Unity mutation occurs.

## In Scope

- `battle/battlefield/**`, `battle/tile_range.rs` only for tile/geometry authority
- `battle/core/movement/**`, `core/spatial.rs`, `core/{field_work,tactical_relocation,tactical_stance,tactical_swap}.rs`
- lifecycle slices of `battle/core/{mod,commands,sim,types}.rs` that mutate spatial/tactical state
- scenario/tactical carrier types and `range_preview.rs` only at the spatial ownership seam
- `world/{combat,state}.rs` live deployment/resource/focus/stance/relocation/swap/field-work state and transactions
- run-policy fields and published RON consumed by these mechanics
- nearest canonical and focused inline/world spatial/tactical tests

## Out Of Scope

- production mutation
- battle finish/log/result lifecycle corrections (Phase 6)
- setup battlefield/template/wave construction (Phase 5)
- damage, stats, buffs, control, resonance and death correctness (Phase 8)
- attack/skill target selection, projectile/area delivery and final range-preview policy (Phase 9)
- DTO/server/Unity refactor discovery (Phase 10)
- balance changes, new movement backend, tactical feature or migration policy

## Investigation Scope

1. Reconcile exact spatial/tactical source/data/test/canonical/consumer inventory.
2. Trace battlefield → occupancy → movement/path/backend → projected position authority.
3. Trace deploy/withdraw/redeploy and tactical resource/cost/cooldown/lock lifecycle.
4. Trace stance, relocation, swap and field-work validate→mutate→revision/failure boundaries.
5. Search duplicate geometry/occupancy/path models, fallbacks, stale generation and partial commits.
6. Classify Phase 5/6 and Phase 8~10 seams without duplicate candidates.
7. Apply CRA-14, prove no production mutation and perform an independent completion audit.

## Focused Validation

- movement engine/planner/backend and battlefield/spatial unit families
- world deploy/withdraw/stance/relocation/swap/field-work atomicity and Pause tests
- exact blocked/invalid/stale/cooldown/resource no-mutation boundaries
- runner only after static gaps are recorded

## Omitted Validation

- whole project by default; green count is not test-quality proof
- Phase 8/9 formula/target/delivery correctness except spatial carrier evidence
- server/Unity and unknown external consumers except explicit classification
- formatter and production mutation

## Completion Conditions

- exact spatial/tactical authority and transaction trace with all signals dispositioned
- predecessor/later seams classified without duplicate candidates
- policy/canonical/determinism/Pause/failure semantics mapped
- CRA-14 test-family catalog complete
- every candidate satisfies CRA-05/06/07 fields
- initial/final production hashes/status identical
- independent audit `Complete / High`

## Completion Evidence Matrix

| ID | Requirement | Required evidence | Status |
| --- | --- | --- | --- |
| P7-01 | exact boundary | source/data/test/canonical/consumer inventory | pass |
| P7-02 | spatial authority | field/occupancy/position trace | pass |
| P7-03 | movement authority | clock/path/steering/backend trace | pass |
| P7-04 | deployment authority | deploy/withdraw/redeploy/cost/lock trace | pass |
| P7-05 | tactical commands | stance/relocation/swap/field-work transaction trace | pass |
| P7-06 | predecessor/later seams | carrier/owner classification | pass |
| P7-07 | live data/canonical | current policy alignment | pass |
| P7-08 | CRA-14 | test family disposition catalog | pass |
| P7-09 | candidates/signals | work type/axes/readiness/disposition | pass |
| P7-10 | no mutation | initial/final content/status comparison | pass |
| P7-11 | completion | independent `Complete / High` | pass |

## Stop Conditions

- movement/tactical gameplay or balance policy is required to classify a candidate
- Phase 8/9 semantics must be treated as verified to decide spatial ownership
- required runtime/data/test/canonical evidence is inaccessible
- read-only work risks existing dirty state or production mutation
