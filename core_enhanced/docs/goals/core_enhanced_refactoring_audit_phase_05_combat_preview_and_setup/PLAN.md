# Core Enhanced Refactoring Audit Phase 05 — Combat Preview And Setup Goal

Status: complete
Implementation Complete: not_applicable
Goal Type: phase
Parent Master: `core_enhanced_refactoring_audit_master`
Phase: 5
Execution Mode: investigation_only
Production Mutation Authorized: no
Audit Baseline Date: 2026-08-07
Start Commit: `f318414fb0156a67e04c5479000c2a2683fc0c09`

## Objective

현재 working tree의 PvE encounter/wave resolution, `CombatPreview`, mission/battlefield/player/enemy setup과
run→active-battle handoff를 runtime, published live data, canonical과 test-code evidence로 감사한다. Preview
cache/manifested difficulty와 setup input의 authority, deterministic seed/UUID, player eligibility/deployment,
enemy wave/scenario/reward assembly와 plan→commit 원자성이 중복되거나 분산된 신호를 실행 가능한 후보 또는
rejected/deferred/unverified disposition으로 분류한다.

## Parent Scope And Direct Dependency

- Parent: `core_enhanced_refactoring_audit_master`
- Boundary: Phase 0 `BOUNDARY_PROPOSAL.md`
- Direct predecessor: Phase 4 `Complete / High`
- Phase 4 handoff: common node completion and Event-started combat session persistence are carriers only; this Phase
  independently audits preview/setup/handoff.
- Phase 1 C02: startup generated-preview validation is baseline-only despite current every-stage canonical. Reuse
  that candidate; do not duplicate it unless a distinct setup root cause is proven.
- Phase 2: run/map/checkpoint and manifested difficulty are confirmed carriers.
- Phase 3: roster/inventory/reward authority is consumed, not reopened.
- Phase 6 is denied until this Phase has `Complete / High`.

## Approved Requirements

Parent CRA-01~CRA-14 all apply. In particular, every confirmed candidate needs actual runtime/data/test evidence,
current→target authority, preserved behavior, removal scope, seven axes, independent readiness and policy decision;
every assigned test family needs CRA-14 meaning/production reachability/assertion/overlap disposition.

## Invariants

- Run node payload plus manifested `MapNodeCombatDifficulty` is the exact preview/setup input authority.
- Preview cache does not become a second gameplay tuning authority or allow stale serialized input to drive setup.
- Encounter/wave resolution, bonus objectives, mission variant, battlefield template/route, player/enemy drafts and
  reward multiplier use deterministic approved seeds and exact live references.
- Invalid or stale node/encounter/difficulty/roster/equipment/setup state fails before battle/node/session commit.
- Preview-to-setup fields preserve qualitative/public projection while internal fields remain Core-owned.
- Player certified combat membership/deployment constraints agree with the battle-entry roster source.
- Active battle lifecycle, clock/events/results are Phase 6; spatial/tactical mutation is Phase 7.
- No production code/data/test/canonical/server/Unity mutation occurs.

## In Scope

- all `src/game/combat_preview/**` and `src/game/combat_setup/**`
- `wave_resolution.rs`, `combat_power.rs`, Phase 5 seams of `pve_bonus_objectives.rs`, `reward_policy.rs`
- `world/combat.rs` preview cache, plan/build/commit and initial battle handoff only
- `RunState.combat_previews`, `ActiveBattleSession` construction fields only as carriers
- published encounter, run-policy, battlefield, map difficulty and equipment/profile data consumed by setup
- nearest canonical and focused preview/setup/world-combat/live-RON tests

## Out Of Scope

- production mutation
- active battle clock/event/result/retreat internals after successful handoff (Phase 6)
- live deploy/withdraw/stance/relocation/occupancy after handoff (Phase 7)
- stat/effect/damage/death and targeting/delivery internals (Phases 8~9)
- DTO/server/Unity refactor discovery (Phase 10)
- balance/content changes, new mission variants or migration policy

## Investigation Scope

1. Reconcile exact preview/setup source/data/canonical/test/consumer inventory.
2. Trace node manifest → preview cache/generation → wave/mission/battlefield/reward/player/enemy plans.
3. Trace plan validation and commit into `ActiveBattleSession`/`BattleCore` without auditing later simulation.
4. Audit deterministic seed/UUID, stale cache, repeated preview, retry and failure/no-mutation semantics.
5. Search duplicated type conversions, resolvers, validators, fallback/defaults and manual mapping.
6. Apply CRA-14 to assigned test families.
7. Produce boundary/candidate/test/completion artifacts and initial/final no-mutation proof.

## Focused Validation

- current preview/setup/wave unit and world handoff tests
- invalid node difficulty, missing references, no eligible employee, deployment/battlefield contract and atomic start
- exact current generated preview/setup alignment and deterministic repeat
- runner only after a static gap is recorded

## Omitted Validation

- whole project by default; green count is not test-quality proof
- simulation/spatial/effect/targeting internals after initial build
- server/Unity and external public consumers except explicit unverified classification
- formatter and production mutation

## Completion Conditions

- exact authority/lifecycle/input→plan→commit trace and all signal dispositions
- four predecessor handoffs classified without duplicate candidates
- data/canonical alignment and deterministic/failure semantics mapped
- CRA-14 test-family catalog complete
- every candidate satisfies CRA-05/06/07 fields
- initial/final production hashes/status identical
- independent audit `Complete / High`

## Completion Evidence Matrix

| ID | Requirement | Required evidence | Status |
| --- | --- | --- | --- |
| P5-01 | exact boundary | source/data/test/canonical/consumer inventory | pass |
| P5-02 | preview authority | node/difficulty/cache/generation trace | pass |
| P5-03 | setup authority | wave/mission/battlefield/player/enemy/reward plan trace | pass |
| P5-04 | run→battle handoff | plan/validate/commit/rollback trace | pass |
| P5-05 | determinism and identity | seed/UUID/retry/cache evidence | pass |
| P5-06 | predecessor/later seams | carrier/owner classification | pass |
| P5-07 | live data/canonical | published/current contract alignment | pass |
| P5-08 | CRA-14 | test family disposition catalog | pass |
| P5-09 | candidates/signals | complete work type/axes/readiness/disposition | pass |
| P5-10 | no mutation | initial/final content/status comparison | pass |
| P5-11 | completion | independent `Complete / High` | pass |

## Stop Conditions

- gameplay/balance/mission policy is required to classify a candidate
- active battle internals must be treated as verified to decide setup ownership
- required production/data/test/canonical evidence is inaccessible
- read-only work risks existing dirty state or production mutation
