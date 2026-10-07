# Core Enhanced Refactoring Audit Phase 04 — Interactive Node And Visual Novel Goal

Status: complete
Implementation Complete: not_applicable
Goal Type: phase
Parent Master: `core_enhanced_refactoring_audit_master`
Phase: 4
Execution Mode: investigation_only
Production Mutation Authorized: no
Audit Baseline Date: 2026-08-07
Start Commit: `f318414fb0156a67e04c5479000c2a2683fc0c09`

## Objective

현재 working tree에서 event/facility/node session과 visual-novel session/trigger lifecycle을 실제
runtime, published live data, canonical과 test-code evidence로 감사한다. Session identity와 admission,
validation→commit 원자성, replay/idempotence, expiration·completion·cleanup, persistent asset handoff,
VN trigger selection과 run/node progression ownership이 중복되거나 분산된 신호를 실행 가능한 후보 또는
명시적 rejected/deferred/unverified disposition으로 분류한다.

## User/Contract-Visible Problem

Interactive node와 VN 흐름은 선택 가능한 action, selected employee/choice, pending session, reward/cost,
node completion과 다음 trigger를 여러 command에 걸쳐 유지한다. 서로 다른 state가 같은 session을
authority처럼 소유하거나 실패 뒤 일부만 commit되고, stale/replayed action이 다시 적용되거나,
Event/Facility/VN 종료가 run progression과 어긋나면 플레이어 선택·자산·진행이 손상된다. 반대로 combat
preview/setup이나 DTO projection까지 이 Phase에서 합치면 독립 authority를 잘못 리팩토링한다.

## Audit Finding ID

없음. Parent Master CRA-01~CRA-14의 네 번째 boundary investigation Phase다. Candidate ID는 실제
evidence가 확인된 뒤 `CRA-P04-CNN` 형식으로 부여한다.

## Parent Scope And Direct Dependency

- Master registry: [PLAN.md](../core_enhanced_refactoring_audit_master/PLAN.md)
- Boundary decision:
  [BOUNDARY_PROPOSAL.md](../core_enhanced_refactoring_audit_phase_00_surface_inventory/BOUNDARY_PROPOSAL.md)
- Direct predecessor: Phase 3
  [COMPLETION_AUDIT.md](../core_enhanced_refactoring_audit_phase_03_roster_inventory_and_economy/COMPLETION_AUDIT.md),
  verdict `Complete / High`
- Phase 3 handoff: persistent asset grants are staged in current audited callers; Phase 4 must independently audit
  session identity, replay/idempotence and Event/VN/shop lifecycle.
- Phase 2 handoff: run/map/checkpoint is a confirmed carrier; Phase 2 candidates remain independent.
- Phase 1 handoff: published typed data is the input authority; loader Candidate C02 caveat remains separate.
- Activation authority: Parent CRA-13 and sequential registry.
- Next Phase is denied until this Phase has its own `Complete / High` completion audit.

## Approved Audit Requirements

| ID | This Phase responsibility |
| --- | --- |
| CRA-01 | current working-tree Core interactive-node/VN boundary only |
| CRA-02 | frozen refactor standard and seven priority axes/readiness separation |
| CRA-03 | authoritative mutation, published live data, nearest canonical and focused test code order |
| CRA-04 | no origin/style/file-size-only candidate |
| CRA-05 | every confirmed candidate has code/data/test authority and invariant evidence |
| CRA-06 | priority axes and implementation readiness remain separate |
| CRA-07 | correction, migration, cleanup and behavior-preserving refactor remain distinct |
| CRA-08 | supporting data/server/external consumers are confirmed/unverified/out-of-scope |
| CRA-09 | interactive session authority only; combat and projection internals stay later Phases |
| CRA-10 | no production code/data/test/canonical/server mutation |
| CRA-11 | preserve initial dirty/untracked user work |
| CRA-12 | every investigated signal receives a disposition |
| CRA-13 | use approved sequential Phase boundary/name without expanding material policy authority |
| CRA-14 | every assigned test family receives meaning, production-reachability, assertion-quality, duplication and retain/strengthen/merge/remove/unverified disposition |

## Current Runtime And Live Data Evidence

Detailed evidence is discovery pending. Phase 0 assigns these primary/shared seams:

- `src/game/visual_novel.rs`
- `src/game/world/{event_node,facility,visual_novel_flow,visual_novel_triggers}.rs`
- `src/game/employee_psychology.rs` only where node lifecycle advances employee-owned state
- `GameCoreState`/`RunState`, `world.rs`, node-flow/checkpoint and behavior/snapshot seams only where they carry,
  admit or consume the interactive session contract
- published event/facility/VN-related typed data and the nearest canonical routed by `docs/README.md`
- node-session, facility, VN, map-flow, snapshot/result and RON-loading test families for their directly protected
  contracts

Phase 3 proves only persistent grant staging at the handoff; it does not prove Phase 4 session lifecycle.

## Source Of Truth And Ownership

1. Actual interactive command admission and authoritative session/node/VN state mutation
2. Published live data actually consumed by those paths
3. Nearest durable gameplay/runtime/data canonical routed by `docs/README.md`
4. Focused test code using production types, assessed under CRA-14
5. Direct run/checkpoint, economy and behavior/server seams only to classify carrier/consumer ownership
6. Goal/archive/history only as supporting evidence

Projection does not become authority because it exposes selected event or VN state. Test fixtures do not become
production policy because they construct the same session type.

## Invariants

- Exactly one current authority governs each Event, Facility and VN session identity/lifecycle.
- Admission, choice/employee selection, validation, cost/grant and node/progression commit ordering stays explicit.
- Invalid, stale, duplicate or replayed actions do not leave partial session, asset, psychology or run mutation.
- Completion, cancellation/exit, trigger chaining and cleanup do not strand or repeat a session.
- Event/Facility/VN identity and authored choice/trigger references remain exact; missing references are not silently
  defaulted or repaired.
- Phase 2 run/map/checkpoint and Phase 3 persistent economy semantics are consumed, not redesigned.
- Combat preview/setup, active battle and command/DTO projection remain later-Phase authorities.
- Serialized shape, user-visible outcome, RNG, timing/order, rejection reason and compatibility are not changed by
  this investigation.
- Existing dirty/untracked production files are not restored, formatted, moved, deleted or modified.

## Refactor Candidate And Behavior-Preservation Boundary

Every confirmed candidate must include:

- actual command/input → admission/validation → authoritative session mutation → asset/progression/result consumer
  trace;
- current and target session/trigger authority;
- duplicate state/validator/fallback/parallel lifecycle removal or consolidation scope;
- preserved identity, ordering, failure/no-mutation, reward/cost, RNG, checkpoint and serialized semantics;
- seven priority axes with evidence or `unknown`, plus separate readiness;
- characterization/focused/broad validation needs;
- policy/external-contract decision and disposition.

Changing choice availability, aptitude/DC/reward/cost, session retry/expiry meaning, trigger order, node completion,
VN content, DTO/save/RON shape or external UX is not a behavior-preserving refactor.

## Open Policy Decisions

- None at activation.
- If current runtime/live data/canonical does not determine stale/replay, retry, cancellation, trigger priority,
  reward timing, node completion or compatibility meaning, affected work becomes
  `policy_or_feature_migration`/deferred and this Phase does not choose it.

## In Scope

- Event node session admission, selected employee/choice, aptitude/dice/check resolution and completion
- Facility-family session admission and completion (`SavePoint`, `Maintenance`, `Shop`, `Reward`,
  `HeadquartersContact`), while Phase 3 retains economy mutation internals
- VisualNovel session state, trigger selection, start/advance/choice/finish/cleanup and run/node handoff
- Event/Facility/VN session identity, replay/idempotence, stale action and lifecycle ownership
- direct `GameCoreState`/`RunState`/node-flow/checkpoint carrier seams required to classify current authority
- direct persistent economy mutation handoff without reopening Phase 3 ledger internals
- published typed live data and validation used by these paths
- nearest gameplay/runtime/data canonical
- assigned focused and integration test source with CRA-14 family disposition
- candidate, signal-disposition, readiness, consumer and no-mutation audit artifacts

## Out Of Scope

- production mutation of Core, live data, tests, canonical, server or Unity
- map/checkpoint/storage and economy refactor implementation from Phases 2~3
- combat preview/wave/setup/run→battle handoff internals (Phase 5)
- active battle lifecycle/effects/spatial/targeting (Phases 6~9)
- command/DTO/snapshot/server projection refactors (Phase 10)
- authored narrative/content changes, balance, new choices/triggers/rewards, schema/save migration or compatibility
  policy
- Unity runtime/UI/test/canonical verification
- formatter, commit, stash or worktree cleanup

## Investigation Scope

1. Reconcile exact interactive-node/VN source, data, canonical, test and direct-consumer surface.
2. Trace Event session admission through selection/check/outcome/grant/progression/cleanup.
3. Trace Facility-family session admission through action/cost/effect/psychology/progression/cleanup, reading
   Shop/Reward/HQ only at lifecycle and atomic handoff seams.
4. Trace VN trigger resolution and session start/advance/choice/finish/cleanup across run/node boundaries.
5. Audit identity, stale/replay/idempotence, partial mutation, retry/exit and checkpoint handoffs.
6. Search duplicate lifecycle state, validators, default/fallback, alias/legacy and parallel trigger paths.
7. Read assigned test families and record their unique invariant, production reachability, assertion observation,
   fixture coupling, overlap and CRA-14 disposition.
8. Compare current runtime/live data with nearest canonical and classify every signal.
9. Produce candidate/disposition artifacts with work type, priority axes and readiness.
10. Perform independent completion audit and initial/final production-content comparison.

## DTO/Data/Save Migration

No migration is implemented. Current DTO/data/save shapes are evidence. A candidate requiring schema/version,
authored content, repair/fallback or external rollout becomes `policy_or_feature_migration` or deferred and needs a
separate implementation Goal.

## Deployment And Compatibility

No deployment is performed. Public Core APIs, current serialized session/checkpoint fields and unknown external
Rust/Unity consumers are preserved/unverified rather than assumed safe to change.

## Focused Validation

- exact Event/Facility/VN authority and mutation/lifecycle trace
- invalid/stale/duplicate/replayed/wrong-session/wrong-choice/no-partial-commit cases where present
- session start→advance/action→finish/cleanup and node/progression handoff
- published live reference and trigger validation
- legacy/default/fallback/ignored/disabled-test search
- test-family meaning and duplication audit under CRA-14
- candidate-specific characterization requirements

Runner execution is not automatic. A runner/probe is used only when a specific static evidence gap is recorded
first; its exact test count and pass/fail/skip are then recorded.

## Broad Validation

- Phase 4 primary/shared coverage and complete signal disposition
- cross-domain session→asset/progression/result carrier seams without promoting carriers to authority
- candidate work type/axes/readiness/invariant cross-check
- all assigned test-family dispositions and cleanup catalog completeness
- initial/final scoped dirty set and production-content hashes

## Omitted Validation

- Unity/client presentation and external canonical: approved out-of-scope/unverified.
- Combat setup/battle internals and Phase 10 projection except direct carrier classification.
- Server-wide architecture: not a Core interactive-session audit target.
- Whole-project test runner: user baseline is historical only and CRA-14 requires source meaning audit instead.
- Formatter: investigation-only.

Actual commands, counts, failures and omissions are recorded in `EXPERIMENTS.md`.

## Completion Conditions

- Every assigned source/data/canonical/direct-consumer surface has an evidence-backed disposition.
- Event, Facility and VN authority→mutation→consumer/lifecycle traces are complete.
- Session identity, admission, stale/replay/idempotence, atomicity and completion/cleanup semantics are mapped.
- Run/checkpoint/economy handoffs are classified without reopening their authorities.
- Relevant live typed data, canonical and tests are separated from runtime authority and audited.
- Every assigned test family has CRA-14 meaning, boundary-quality and cleanup disposition evidence.
- Every investigated signal has candidate/rejected/deferred/unverified disposition.
- Every candidate satisfies CRA-05/06/07 evidence fields and preserves explicit invariants.
- Later combat/projection Phases and external consumers are not over-verified.
- Only Goal/audit artifacts change; production content/status is preserved.
- Independent completion audit verdict is `Complete / High`.

## Completion Audit Evidence Matrix

| ID | Completion condition or invariant | Required evidence | Status |
| --- | --- | --- | --- |
| P4-01 | exact boundary coverage | Core/data/test/canonical/direct-consumer inventory | pass |
| P4-02 | Event session authority/lifecycle | admission→selection/check→outcome→cleanup trace | pass |
| P4-03 | Facility session authority/lifecycle | admission→action/cost/effect→cleanup trace | pass |
| P4-04 | VN trigger/session authority/lifecycle | trigger→start/advance/choice/finish trace | pass |
| P4-05 | identity, replay and atomicity | stale/duplicate/failure/no-partial-mutation evidence | pass |
| P4-06 | run/checkpoint/economy and consumer seams | carrier/handoff classification | pass |
| P4-07 | live data and canonical alignment | published references and current contract evidence | pass |
| P4-08 | test-family quality disposition | unique invariant/reachability/assertion/overlap catalog | pass |
| P4-09 | candidate/disposition quality | work type, axes, readiness, invariants and all signals | pass |
| P4-10 | dirty preservation/no mutation | initial/final scoped content/status comparison | pass |
| P4-11 | completion audit verdict | `COMPLETION_AUDIT.md` Complete / High | pass |

## Stop Conditions

- Current session/replay/retry/trigger/reward/progression meaning cannot be classified without user policy.
- Required production live data, canonical or test surface cannot be accessed.
- Material combat/DTO/Unity implementation must be treated as verified to decide this boundary.
- Read-only command risks production/user state mutation.
- Frozen audit standard changes externally.
- Existing dirty changes overlap audit artifacts so they cannot be preserved or distinguished.
- Next Phase activation before this Phase obtains `Complete / High`.
