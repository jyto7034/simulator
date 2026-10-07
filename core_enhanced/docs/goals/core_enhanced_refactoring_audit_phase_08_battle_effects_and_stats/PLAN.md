# Core Enhanced Refactoring Audit Phase 08 — Battle Effects And Stats Goal

Status: complete
Implementation Complete: not_applicable
Goal Type: phase
Parent Master: `core_enhanced_refactoring_audit_master`
Phase: 8
Execution Mode: investigation_only
Production Mutation Authorized: no
Audit Baseline Date: 2026-08-07
Start Commit: `f318414fb0156a67e04c5479000c2a2683fc0c09`

## Objective

현재 working tree의 persistent→runtime stat pipeline, damage/modifier, buff/control, cooldown/trigger,
resonance와 death/incapacitation authority를 runtime, published data, canonical과 test-code evidence로 감사한다.
Formula/state/lifecycle ownership 중복, partial effect mutation, stale generation과 test-only authority를 실행 가능한
후보 또는 rejected/deferred/unverified disposition으로 분류하며 targeting/delivery와 DTO 의미는 후속 Phase의
carrier로만 취급한다.

## Parent Scope And Direct Dependency

- Parent: `core_enhanced_refactoring_audit_master`
- Boundary: Phase 0 `BOUNDARY_PROPOSAL.md` and `SURFACE_INVENTORY.md`
- Direct predecessor: Phase 7 `Complete / High`
- Phase 7 handoff: authoritative body/tactical state is mapped; C01 owns when tile effects apply, while this Phase
  audits modifier composition and damage consumption without duplicating the transition candidate.
- Phase 3 owns persistent roster/inventory/equipment transactions; Phase 6 owns clock/order/result lifecycle.
- Phase 9 owns targeting/delivery; Phase 10 owns public projection/server mapping.
- Phase 9 is denied until this Phase has `Complete / High`.

## Approved Requirements

Parent CRA-01~CRA-14 all apply. Every confirmed candidate needs runtime/data/test/canonical evidence, current→target
authority, preserved invariants, removal scope, seven axes, readiness and policy/external decision. Every assigned
test family needs meaning/reachability/assertion/overlap disposition.

## Invariants

- Persistent level/growth/equipment/fragment inputs become one explicit runtime stat snapshot at battle ownership.
- Damage amount/type/modifier/crit and HP mutation have one authoritative calculation and log path.
- Buff, control, cooldown, trigger and resonance state use authoritative battle time with deterministic stale-event
  rejection, Pause semantics and exact stacking/expiry rules.
- Death/withdraw/incapacitation apply once, cancel or preserve the approved runtime state and reconcile world employee
  attempt state without duplicate facts.
- Phase 7 decides tactical activity/tile transition timing; Phase 8 only audits modifier/stat application.
- Phase 9 targeting/delivery and Phase 10 DTO/server behavior are not decided here.
- No production code/data/test/canonical/server/Unity mutation occurs.

## In Scope

- `battle/{buffs,cooldown,damage,death,stat_pipeline}.rs`
- `battle/core/{damage_runtime,death_runtime,resonance_runtime,triggers}.rs`
- relevant effect/state slices of `battle/core/{build,commands,mod,sim,types}.rs`
- relevant validation/event-log/result-stat carriers without reopening Phase 6 lifecycle ownership
- `stats.rs` and persistent employee/equipment/fragment/growth inputs only at runtime-stat conversion seams
- world combat/state employee-attempt/death/resource carriers only at effect/death commit seams
- published abnormality/buff/equipment/enemy/fragment/skill/run-policy data actually consumed by these mechanics
- nearest canonical and focused inline/integration/world effect/stat/death tests

## Out Of Scope

- production mutation
- persistent economy/loadout transaction refactors (Phase 3)
- battle clock/event-order/result-lifecycle corrections (Phase 6)
- position/occupancy/tactical-transition corrections (Phase 7)
- target selection/usefulness, cast acceptance, projectile/area delivery and range preview (Phase 9)
- DTO/server/Unity refactor discovery (Phase 10)
- balance changes, new effects/stats, authoring migrations or compatibility policy

## Investigation Scope

1. Reconcile exact stat/effect/death source/data/test/canonical/consumer inventory.
2. Trace persistent inputs → `BattleUnitDraft` → runtime stats/modifiers and distinguish snapshot from live state.
3. Trace damage snapshot/calculation → HP/resonance/trigger/log → death and world reconciliation.
4. Trace buff/control/cooldown/trigger/resonance stacking, time, generation, expiry and cancellation.
5. Search duplicated formulas, modifier authorities, fallbacks, partial commits and unused abstraction/test paths.
6. Classify Phase 3/6/7 and Phase 9/10 seams without duplicate candidates.
7. Apply CRA-14, prove no production mutation and perform an independent completion audit.

## Focused Validation

- stat pipeline, damage, buff/control/cooldown/trigger/resonance/death unit families
- selected skill integration and world employee-attempt/death transition tests
- exact stacking/expiry/same-time/stale/no-mutation boundaries
- runner only after static gaps are recorded

## Omitted Validation

- whole project by default; green count is not test-quality proof
- Phase 9 target/delivery correctness except effect carrier evidence
- server/Unity and unknown external consumers except explicit classification
- formatter and production mutation

## Completion Conditions

- exact stat/effect/death authority and state-transition trace with all signals dispositioned
- predecessor/later seams classified without duplicate candidates
- live data/canonical/determinism/Pause/failure semantics mapped
- CRA-14 test-family catalog complete
- every candidate satisfies CRA-05/06/07 fields
- initial/final production hashes/status identical
- independent audit `Complete / High`

## Completion Evidence Matrix

| ID | Requirement | Required evidence | Status |
| --- | --- | --- | --- |
| P8-01 | exact boundary | source/data/test/canonical/consumer inventory | pass |
| P8-02 | stat authority | persistent→draft→runtime trace | pass |
| P8-03 | damage authority | calculation/modifier/HP/log trace | pass |
| P8-04 | effect lifecycle | buff/control/cooldown/trigger/resonance trace | pass |
| P8-05 | death lifecycle | death/incapacitation/cancellation/world trace | pass |
| P8-06 | predecessor/later seams | carrier/owner classification | pass |
| P8-07 | live data/canonical | current contract alignment | pass |
| P8-08 | CRA-14 | test family disposition catalog | pass |
| P8-09 | candidates/signals | work type/axes/readiness/disposition | pass |
| P8-10 | no mutation | initial/final content/status comparison | pass |
| P8-11 | completion | independent `Complete / High` | pass |

## Stop Conditions

- a new damage/effect/balance policy is required to classify a candidate
- Phase 9 target/delivery semantics must be treated as verified to decide effect ownership
- required runtime/data/test/canonical evidence is inaccessible
- read-only work risks existing dirty state or production mutation
