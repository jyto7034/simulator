# Core Enhanced Refactoring Audit Phase 03 — Roster, Inventory, And Economy Goal

Status: complete
Implementation Complete: not_applicable
Goal Type: phase
Parent Master: `core_enhanced_refactoring_audit_master`
Phase: 3
Execution Mode: investigation_only
Production Mutation Authorized: no
Audit Baseline Date: 2026-08-07
Start Commit: `f318414fb0156a67e04c5479000c2a2683fc0c09`

## Objective

현재 working tree의 employee roster/condition/loadout, inventory/equipment/consumable/material, skill fragment
growth/research, Enkephalin, grant/reward/shop/headquarters transaction으로 이어지는 persistent economy
authority를 실제 runtime/live-data/test/canonical evidence로 감사한다. 중복 ledger, 부분 mutation, identity
불일치, lifecycle 분산과 변경 증폭 신호를 실행 가능한 후보 또는 명시적 rejected/deferred/unverified
disposition으로 분류한다.

## User/Contract-Visible Problem

Roster와 inventory/economy는 같은 직원·장비·파편 identity를 loadout, reward, shop, maintenance, recruitment,
checkpoint와 combat handoff에서 반복 소비한다. 보유 ledger와 장착 상태가 서로 다른 authority를 갖거나,
비용 지불 뒤 grant가 실패하거나, 성장/research가 중복 commit되면 플레이어 자산과 진행을 잃는다. 반대로
Event/VN session이나 active battle 내부까지 이 Phase에서 합치면 독립 transaction과 정책을 잘못
리팩토링하게 된다.

## Audit Finding ID

없음. Parent Master CRA-01~CRA-13의 세 번째 boundary investigation Phase다. Candidate ID는 실제 evidence
확인 뒤 `CRA-P03-CNN` 형식으로 부여한다.

## Parent Scope And Direct Dependency

- Master registry: [PLAN.md](../core_enhanced_refactoring_audit_master/PLAN.md)
- Direct predecessor: Phase 2
  [COMPLETION_AUDIT.md](../core_enhanced_refactoring_audit_phase_02_run_map_and_persistence/COMPLETION_AUDIT.md),
  verdict `Complete / High`
- Boundary decision:
  [BOUNDARY_PROPOSAL.md](../core_enhanced_refactoring_audit_phase_00_surface_inventory/BOUNDARY_PROPOSAL.md)
- Published-data handoff: Phase 1 `Complete / High`; loader Candidate C02 caveat remains separate.
- Run/checkpoint handoff: Phase 2 confirms current V3 carries roster, inventory, skill fragments and Enkephalin,
  but C03 means the manual carrier mapping has no golden parity gate.
- Activation authority: Parent CRA-13 and sequential registry.
- Next Phase is denied until this Phase has its own `Complete / High` completion audit.

## Approved Audit Requirements

| ID | This Phase responsibility |
| --- | --- |
| CRA-01 | current working-tree Core roster/inventory/economy boundary only |
| CRA-02 | frozen refactor standard and seven priority axes/readiness separation |
| CRA-03 | authoritative mutation, published live data, canonical and focused tests order |
| CRA-04 | no origin/style/file-size-only candidate |
| CRA-05 | every candidate has runtime/data/test authority and invariant evidence |
| CRA-06 | priority axes and implementation readiness remain separate |
| CRA-07 | correction, migration and behavior-preserving refactor remain distinct |
| CRA-08 | supporting server/data/external consumers are confirmed/unverified/out-of-scope |
| CRA-09 | economy authority only; session/combat/projection internals stay later Phases |
| CRA-10 | no production code/data/test/canonical/server mutation |
| CRA-11 | preserve initial dirty/untracked user work |
| CRA-12 | every investigated signal receives a disposition |
| CRA-13 | use approved sequential Phase boundary/name without expanding policy authority |

## Current Runtime And Live Data Evidence

Phase 0~2 established the incoming seams. Phase 3 completed the detailed ownership audit in
`BOUNDARY_AUDIT.md`, classified five candidates and fourteen other signals in `CANDIDATE_CATALOG.md`, and obtained
`Complete / High` in `COMPLETION_AUDIT.md`.

- `GameCoreState` carries roster, inventory, Enkephalin, skill fragments/policy and checkpoint copies.
- `RunState` carries recruitment and abnormality research; Phase 3 reads these only at economy ownership seams.
- Phase 1 confirms employee/equipment/consumable/reward/shop/skill-fragment/run-policy catalogs are typed published
  inputs through one strict loader.
- Phase 2 confirms valid SavePoint capture/restore round trips these persistent fields and host bytes are opaque.
- World reward/shop/headquarters/maintenance modules and shared grant execution mutate the same asset family;
  the completed trace distinguishes atomic caller staging from C02's enhancement defect and C04/C05's
  change-amplification candidates.

## Source Of Truth And Ownership

1. Employee roster, inventory, skill-fragment and currency authoritative state/mutation code
2. Shared grant/cost/ownership validators and staged commit code actually used by commands
3. Published typed employee/equipment/consumable/reward/shop/skill-fragment/run-policy data
4. Nearest gameplay/runtime/data canonical routed by `docs/README.md`
5. Focused unit/integration tests and read-only probes when necessary
6. Direct checkpoint/session/combat/server handoff only to classify consumers
7. Goal/archive/history only as supporting evidence

Projection does not become authority merely because it repeats balances/loadout. Test builders do not become
production economy sources merely because they create the same structs.

## Invariants

- Employee definition identity, run-local employee UUID, owned-item instance UUID and definition IDs remain
  distinct and stable.
- Roster/inventory/loadout ownership must remain exact across grant, equip/unequip, death, recruitment, reward,
  shop, maintenance, checkpoint and rollback.
- Cost, validation, grant and result commit order/failure meanings are not changed.
- Enkephalin/material/fragment/equipment quantities and caps do not silently wrap, duplicate or partially commit.
- Growth, research completion, pending delivery and unique reward idempotence remain unchanged.
- Phase 2's strict V3/checkpoint/host semantics are consumed but not redesigned.
- Event/VN session policy, battle calculation and DTO shape remain later-Phase boundaries.
- Existing dirty/untracked production files are not restored, formatted, moved, deleted or modified.

## Refactor Candidate And Behavior-Preservation Boundary

Every confirmed candidate must include:

- actual command/input → validation/cost → authoritative mutation/grant → checkpoint/result/consumer trace;
- current and target asset/identity authority;
- duplicate ledger/validator/fallback/consolidation scope;
- preserved ownership, quantity, ordering, atomic failure, RNG/UUID, reward/research and serialized semantics;
- seven priority axes with evidence or `unknown`, plus separate readiness;
- characterization/focused/broad validation needs;
- policy/external-contract decision and disposition.

Changing economy values, reward policy, drop/offer generation, employee eligibility, item/fragment schema,
checkpoint version or external serialized shape is not a behavior-preserving refactor.

## Open Policy Decisions

- None at activation.
- If live code/canonical does not determine pricing, duplicate ownership, cap/overflow, progression or reward
  conflict behavior, affected work becomes `policy_or_feature_migration`/deferred and this Phase does not choose it.

## In Scope

- employee definition→run-local roster identity, condition, experience/psychology persistent seams
- inventory, owned equipment instance, item slots/loadout, consumables and equipment materials
- skill-fragment ownership/equip/progression/research/pending delivery direct economy paths
- Enkephalin and other persistent spend/grant ledgers
- shared grant executor, reward application, shop buy/sell/refresh direct transaction semantics
- headquarters recruitment/emergency/shop and maintenance direct asset transaction seams
- death/recovery/checkpoint/rollback carry only where persistent ownership is affected
- typed live catalog values and validation supporting these mutations
- focused Core tests and nearest canonical
- candidate, disposition, readiness, consumer and no-mutation audit artifacts

## Out Of Scope

- production mutation of Core, live data, tests, canonical, server or Unity
- map/checkpoint/storage refactor implementation (Phase 2 candidate catalog)
- Event/visual-novel/session lifecycle beyond direct economy commit handoff (Phase 4)
- combat preview/certified-power formulas and battle setup internals (Phase 5)
- active battle stats/effects/death calculation beyond committed roster/inventory boundary (Phases 6/8)
- command/DTO/snapshot refactors (Phase 10)
- economy rebalance, new reward/shop/recipe content, save/data migration, compatibility policy
- formatter/commit/stash/cleanup

## Investigation Scope

1. Reconcile exact roster/inventory/economy source/test/live-data/canonical surface.
2. Trace employee definition→instance→roster→loadout ownership and condition/growth lifetimes.
3. Trace owned equipment/consumable/material/skill-fragment ledgers and identity validation.
4. Trace grant/reward/shop/headquarters/maintenance cost-validation-staging-commit failure paths.
5. Trace research/pending delivery, duplicate/idempotence and checkpoint/rollback handoff.
6. Search parallel balance calculations, duplicate registries, fallback/default/legacy and partial mutation.
7. Audit focused tests for success, insufficient funds, stale identity, duplicate, overflow, rollback and restore.
8. Compare current runtime/live values with nearest canonical and classify every signal.
9. Produce candidate/disposition artifacts with priority axes/readiness.
10. Perform independent completion audit and production-content hash comparison.

## DTO/Data/Save Migration

No migration is implemented. Current data/save/DTO shapes are evidence. A candidate requiring version/schema
change, live-data correction, compatibility reader, rebalance or external rollout is classified as
`policy_or_feature_migration` or deferred with a separate implementation Goal requirement.

## Deployment And Compatibility

No deployment is performed. Public Core APIs, local V3 save fields and unknown external Rust/Unity consumers are
preserved/unverified rather than assumed safe to change.

## Focused Validation

- exact employee/item/fragment/currency authority and mutation trace
- focused roster/inventory/equip/grant/reward/shop/research/maintenance tests
- invalid/stale/duplicate/insufficient/overflow/no-partial-commit cases
- checkpoint/rollback round trip only for the affected persistent field
- legacy/default/fallback/ignored-test search
- candidate-specific characterization requirements

## Broad Validation

- Phase 3 primary/shared coverage and complete signal disposition
- representative cross-domain asset transaction paths
- candidate work type/axes/readiness/invariant cross-check
- initial/final scoped dirty set and production-content hashes

## Omitted Validation

- Unity/client economy projection: approved out-of-scope/unverified.
- Active battle and interactive-session full suites unless directly needed for persistent commit evidence.
- Server-wide architecture: not a Core economy audit target.
- Formatter: investigation-only.

Actual execution, counts, failures and omissions are recorded in `EXPERIMENTS.md`.

## Completion Conditions

- Employee/roster/loadout and all persistent asset ledgers have evidence-backed authority traces.
- Grant/cost/reward/shop/recruitment/maintenance atomicity and failure semantics are mapped.
- Research/growth/duplicate/idempotence and checkpoint/rollback handoffs are mapped.
- Relevant live typed data, canonical and tests are separated from runtime authority and audited.
- Every investigated signal has candidate/rejected/deferred/unverified disposition.
- Every candidate satisfies CRA-05/06/07 evidence contract.
- Later session/combat/projection Phases and external consumers are not over-verified.
- Only Goal/audit artifacts change; production content/status hashes are preserved.
- Phase completion audit is `Complete / High`.

## Completion Audit Evidence Matrix

| ID | Completion condition or invariant | Required evidence | Status |
| --- | --- | --- | --- |
| P3-01 | exact boundary coverage | Core/data/test/canonical/direct-consumer inventory | pass |
| P3-02 | employee/roster authority trace | definition/instance/condition/growth evidence | pass |
| P3-03 | inventory/loadout authority trace | item instance/slot/consumable/material evidence | pass |
| P3-04 | fragment/research/currency trace | ownership/progression/pending/cost evidence | pass |
| P3-05 | transaction atomicity | grant/reward/shop/recruitment/maintenance failure evidence | pass |
| P3-06 | checkpoint/rollback and consumer seam | persistent carrier/lifetime evidence | pass |
| P3-07 | canonical and test safety net | current contract/live values/focused runner evidence | pass |
| P3-08 | complete candidate/disposition quality | work type, axes, readiness, invariants and all signals | pass |
| P3-09 | dirty preservation/no mutation | initial/final scoped content/status comparison | pass |
| P3-10 | completion audit verdict | `COMPLETION_AUDIT.md` Complete / High | pass |

## Stop Conditions

- Current economy/ownership behavior cannot be classified without user policy.
- Required production live data or focused test surface cannot be accessed.
- Material Event/VN/battle/DTO implementation must be audited to decide this boundary.
- External consumer must be assumed verified without evidence.
- Read-only runner risks production/user state mutation.
- Frozen audit standard changes externally.
- Existing dirty changes overlap audit artifacts so they cannot be preserved/distinguished.
- Next Phase activation before this Phase obtains `Complete / High`.
