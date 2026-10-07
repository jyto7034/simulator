# Core Enhanced Refactoring Audit Phase 09 — Battle Targeting And Delivery Goal

Status: complete
Implementation Complete: not_applicable
Goal Type: phase
Parent Master: `core_enhanced_refactoring_audit_master`
Phase: 9
Execution Mode: investigation_only
Production Mutation Authorized: no
Audit Baseline Date: 2026-08-07
Start Commit: `f318414fb0156a67e04c5479000c2a2683fc0c09`

## Objective

Current working tree의 basic attack/skill cast targeting, usefulness, tile range, cast/step retarget, projectile and
area delivery, range preview authority를 runtime, published data, canonical과 test-code evidence로 감사한다. Selection,
acceptance, snapshot, collision/hit/expiry 권한 중복과 fallback, partial lifecycle, test-only semantics를 실행
가능한 후보나 rejected/deferred/unverified disposition으로 분류한다.

## Parent Scope And Direct Dependency

- Parent: `core_enhanced_refactoring_audit_master`
- Boundary: Phase 0 `BOUNDARY_PROPOSAL.md` and `SURFACE_INVENTORY.md`
- Direct predecessor: Phase 8 `Complete / High`
- Phase 7 owns body/position/occupancy; Phase 8 owns effect/damage outcome semantics.
- Phase 10 owns official DTO/range projection transport and direct server seam.
- Phase 10 is denied until this Phase has `Complete / High`.

## Approved Requirements

Parent CRA-01~CRA-14 all apply. Confirmed candidates need exact authority/consumer/data/test/canonical evidence,
preserved invariants, removal scope, work type, seven axes, readiness and policy/external gates. Every assigned test
family needs meaning/reachability/assertion/overlap disposition.

## Invariants

- Candidate discovery, cast acceptance, step resolution and delivery each have one explicit authority and typed
  failure/no-op semantics.
- Spatial queries consume Phase 7 authoritative bodies/tiles and do not create a second position authority.
- Damage/effects consume Phase 8 snapshots/results and do not recalculate formula policy.
- Released projectile/area/step state preserves approved source/target/anchor snapshots and deterministic identity.
- Target ties, collision/hit order, repeat/retarget and event causes are deterministic and independent of log sequence.
- Pause freezes battle time without wall-time catch-up.
- No production code/data/test/canonical/server/Unity mutation occurs.

## In Scope

- `battle/core/{basic_attack,basic_attack_projectile,projectile_math,target_usefulness,targeting}.rs`
- all `battle/core/skill_runtime/**`
- target/delivery slices of `battle/core/{commands,mod,sim,types}.rs`
- `ability.rs`, `range_preview.rs` and `battle/tile_range.rs` as typed model/projection/spatial carriers
- skill/unit/equipment/fragment published inputs and target/delivery validation actually consumed by runtime
- `skill_target_contract.md` and nearest runtime/gameplay/data canonical
- inline target/projectile/area/cast tests, `skill_runtime_contract`, `skill_test_suite`/`skill_test/**`, live item/
  skill catalog tests and focused world manual/automatic skill/range cases

## Out Of Scope

- production mutation
- Phase 7 position/occupancy/tactical correction implementation
- Phase 8 formula/effect/death policy implementation
- command DTO/server/Unity refactor discovery (Phase 10)
- new targeting, range, collision, balance, authoring or compatibility policy
- persistent fragment/equipment transaction refactors (Phase 3)

## Investigation Scope

1. Reconcile source/data/test/canonical/consumer inventory.
2. Trace basic attack selection → windup/release → instant/projectile hit.
3. Trace automatic/manual cast acceptance → target capture/retarget → step/repeat/delivery cleanup.
4. Trace projectile and area identity, geometry, snapshots, hit order, miss/stop/expiry and Pause behavior.
5. Trace usefulness/range preview projections back to the same runtime rules.
6. Search fallbacks, duplicated validators/selectors/geometry, partial lifecycle and test-only authority.
7. Apply CRA-14, prove no production mutation and perform an independent completion audit.

## Focused Validation

- basic attack target/windup/projectile families
- manual/automatic cast targeting, retarget/repeat and failure immutability
- skill projectile/area geometry, hit/stop/expiry and event-cause families
- range preview and live skill behavior scenarios
- runner only after static gaps are recorded

## Omitted Validation

- whole project by default
- damage/effect numeric correctness already Phase 8
- server/Unity transport semantics except explicit carrier classification
- formatter and production mutation

## Completion Evidence Matrix

| ID | Requirement | Required evidence | Status |
| --- | --- | --- | --- |
| P9-01 | exact boundary | source/data/test/canonical/consumer inventory | pass |
| P9-02 | basic attacks | selection→release→delivery trace | pass |
| P9-03 | skill targeting | cast→step/repeat/retarget trace | pass |
| P9-04 | projectile/area | snapshot/geometry/hit/cleanup trace | pass |
| P9-05 | usefulness/preview | shared authority and drift audit | pass |
| P9-06 | predecessor/later seams | carrier/owner classification | pass |
| P9-07 | live/canonical | current contract alignment | pass |
| P9-08 | CRA-14 | test family disposition catalog | pass |
| P9-09 | candidates/signals | type/axes/readiness/disposition | pass |
| P9-10 | no mutation | initial/final content/status comparison | pass |
| P9-11 | completion | independent `Complete / High` | pass |

## Stop Conditions

- a new target/range/collision/gameplay policy is required to classify rather than defer a candidate
- Phase 10 DTO/server semantics must be treated as verified to decide runtime ownership
- required runtime/data/test/canonical evidence is inaccessible
- read-only work risks existing dirty state or production mutation
