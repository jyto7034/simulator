# Phase 09 Battle Targeting Delivery And Projectiles Reaudit Goal

Status: complete
Implementation Complete: not_applicable
Goal Type: phase
Execution Mode: investigation_only
Production Mutation Authorized: no
Direct Dependency: P08 `Complete / High`

## Objective And Scope

Reaudit every CRA-P09 candidate against current targeting, range, skill delivery, projectile and area runtime
authorities. Deeply verify Boss target-lock/cancellation consumption without conflating it with ordinary target or
delivery policy, normalize the historical P09-C05 work-type wording, and investigate evidence-backed new candidates.

Artifacts: `CANDIDATE_REAUDIT.md`, `TEST_REAUDIT.md`, `BOUNDARY_AUDIT.md`, `COMPLETION_AUDIT.md`.

## Invariants

Preserve deterministic target identity/order, authored range/selection, release snapshots, impact/collision/lifetime,
released-work survival, ordinary encounter parity, event lineage and public/external gates. Production/history/dirty
content is read-only.

## Validation And Gates

Read the original P09 catalog and current targeting/delivery/projectile/area/Boss consumer source, tests and nearest
canonical. Run focused tests chosen by assertion meaning. New IDs require a distinct `CRR-P09-*` root cause.
Gameplay targeting/delivery policy and public/external consumer changes remain gated.

## Evidence Matrix

| ID | Condition | Status |
| --- | --- | --- |
| P09-C01 | exact original inventory/dispositions/work types | pass — eight rows, all still valid |
| P09-C02 | target/range/delivery/projectile/area authority | pass |
| P09-C03 | Boss target/cancellation impact/new candidate | pass — no new candidate |
| P09-C04 | tests, policy/external gates and preservation | pass — 146 physical tests and exact hashes |
| P09-C05 | Complete / High audit | pass |
