# Core Enhanced Refactoring Audit Phase 02 — Run, Map, And Persistence Goal

Status: complete
Implementation Complete: not_applicable
Goal Type: phase
Parent Master: `core_enhanced_refactoring_audit_master`
Phase: 2
Execution Mode: investigation_only
Production Mutation Authorized: no
Audit Baseline Date: 2026-08-07
Start Commit: `f318414fb0156a67e04c5479000c2a2683fc0c09`

## Objective

현재 working tree의 `GameCoreState`/`RunState`에서 run 생성, authored map/floor/node progression,
checkpoint export/load, rollback/retry와 server active-run persistence로 이어지는 authority 경계를 실제
runtime/live-data/test/canonical evidence로 감사한다. 중복 authority, 부분 mutation, lifecycle 분산,
fallback/compatibility와 변경 증폭 신호를 실행 가능한 후보 또는 명시적
rejected/deferred/unverified disposition으로 분류한다.

## User/Contract-Visible Problem

Run/map/checkpoint는 gameplay 진행, retry/rollback, reconnect와 저장 수명을 함께 잇는다. Map이나
checkpoint projection이 authoritative run state를 재추론하거나, 실패 뒤 run/floor/node/persistence 일부만
commit되거나, Core와 server가 서로 다른 version/fallback 의미를 가지면 플레이어 진행 손실과 stale
복원이 발생할 수 있다. 반대로 물리적으로 같은 `world` 파일에 있다는 이유만으로 economy/session/
combat authority까지 합치면 독립 계약을 잘못 리팩토링하게 된다.

## Audit Finding ID

없음. Parent Master CRA-01~CRA-13의 두 번째 boundary investigation Phase다. Candidate ID는 실제
evidence 확인 뒤 `CRA-P02-CNN` 형식으로 부여한다.

## Parent Scope And Direct Dependency

- Master registry: [PLAN.md](../core_enhanced_refactoring_audit_master/PLAN.md)
- Direct predecessor: Phase 1
  [COMPLETION_AUDIT.md](../core_enhanced_refactoring_audit_phase_01_live_data_loading_and_validation/COMPLETION_AUDIT.md),
  verdict `Complete / High`
- Boundary decision:
  [BOUNDARY_PROPOSAL.md](../core_enhanced_refactoring_audit_phase_00_surface_inventory/BOUNDARY_PROPOSAL.md)
- Published-data handoff:
  [BOUNDARY_AUDIT.md](../core_enhanced_refactoring_audit_phase_01_live_data_loading_and_validation/BOUNDARY_AUDIT.md)
  and Phase 1 C02 baseline-only preview-validation caveat
- Activation authority: Parent CRA-13 and sequential registry.
- Next Phase is denied until this Phase has its own `Complete / High` completion audit.

## Approved Audit Requirements

| ID | This Phase responsibility |
| --- | --- |
| CRA-01 | current working-tree Core run/map/persistence boundary only |
| CRA-02 | frozen refactor standard and seven priority axes/readiness separation |
| CRA-03 | authoritative mutation, live published data, persistence producer/consumer, canonical, tests order |
| CRA-04 | no origin/style/file-size-only candidate |
| CRA-05 | every candidate has runtime/data/test authority and invariant evidence |
| CRA-06 | priority axes and implementation readiness remain separate |
| CRA-07 | correction, migration and behavior-preserving refactor remain distinct |
| CRA-08 | server persistence and external consumers are classified confirmed/unverified/out-of-scope |
| CRA-09 | run/map/checkpoint lifecycle only; economy/session/combat internals stay later Phases |
| CRA-10 | no production code/data/test/canonical/server mutation |
| CRA-11 | preserve initial dirty/untracked user work |
| CRA-12 | every investigated signal receives a disposition |
| CRA-13 | use approved sequential Phase boundary/name without expanding policy authority |

## Current Runtime And Live Data Evidence

Phase 0/1 establish only the following seams; detailed ownership and candidate status remain `discovery pending`.

- `GameCore` privately owns `GameCoreState` and shared validated `Arc<GameDataBase>`.
- `GameCoreState` contains root game/run, active-node/session, active battle, checkpoint and related progression
  state; Phase 2 audits only run/map/checkpoint parts.
- Phase 1 proves all current 23 live files are published through one strict loader. Phase 2 consumes the typed run
  policy and map catalogs as evidence but does not reopen their loading authority.
- Server active-run persistence is a confirmed supporting consumer/producer seam assigned to this Phase.
- Phase 1 Candidate C02 means startup preview validation is baseline-only; Phase 2 must not overstate manifested
  difficulty validation while tracing map→preview references.

## Source Of Truth And Ownership

1. `GameCore` command entry and `GameCoreState`/`RunState` authoritative mutations
2. Published typed run policy and map catalogs actually read by those mutations
3. Core checkpoint/export/import/version and rollback transaction code
4. Server active-run persistence reader/writer/bootstrap and message mapping
5. `docs/README.md`-routed runtime/gameplay/data canonical
6. Focused/integration tests and read-only probes when needed
7. Goal/archive/history only as supporting evidence

The server may own host storage mechanics and envelope lifetime without owning Core gameplay progression. The
audit must distinguish those authorities rather than selecting a physical repository as the only owner.

## Invariants

- Run/floor/map/node/progression, checkpoint payload, version and failure meanings are not changed.
- Failed command/load/restore/retry must be traced for partial Core or persistence mutation.
- Derived map/snapshot/checkpoint fields must not be treated as independent policy without producer evidence.
- Current published data is read-only supporting evidence; Phase 1 loading findings are not reimplemented here.
- Roster/economy/event/VN/combat behavior is read only at the direct run lifecycle seam and assigned onward.
- Host storage transaction/checksum/backup behavior is verified only to the extent it consumes/produces Core
  active-run state; server architecture itself is not audited for independent refactors.
- Unity is out-of-scope/unverified.
- Existing dirty/untracked production files are not restored, formatted, moved, deleted, or modified.

## Refactor Candidate And Behavior-Preservation Boundary

Every confirmed candidate must include:

- actual command/input → validation → authoritative mutation → checkpoint/persistence/consumer trace;
- current and target authority;
- duplicate/fallback/legacy/consolidation scope;
- preserved run identity, floor/node ordering, RNG, version, failure/rollback/retry/reconnect semantics;
- seven priority axes with evidence or `unknown`, plus separate readiness;
- characterization/focused/broad validation needs;
- policy/external-contract decision and disposition.

Changing save/checkpoint version, accepted old payloads, retry semantics, gameplay progression, map generation, or
external serialized shape is not a behavior-preserving refactor.

## Open Policy Decisions

- None at activation.
- If current code/canonical does not determine checkpoint compatibility, restore fallback, conflict resolution,
  progression or retry behavior, affected work becomes `policy_or_feature_migration`/deferred and this Phase does
  not choose it.

## In Scope

- `GameCoreState`, `RunState`, run start/continue/finish/failure root lifecycle
- map/floor/node generation, assignment, navigation and progression authority
- run identity/RNG/checkpoint/rollback transaction and restoration paths
- Core active-run export/import/version/validation shape
- direct server active-run message, store, bootstrap, checksum/backup/atomic-write seam
- typed run policy/map catalog direct consumption supporting the above
- focused Core/server tests that protect the same boundary
- relevant runtime/gameplay/data/persistence canonical statements
- candidate, disposition, readiness, consumer and no-mutation audit artifacts

## Out Of Scope

- production mutation of Core, live data, tests, canonical, server or Unity
- roster/inventory/economy transaction internals (Phase 3)
- Event/facility/VN session internals (Phase 4; direct map node handoff only)
- combat preview/setup and active battle internals (Phases 5–9; direct stored preview/handoff only)
- DTO/command projection refactors (Phase 10)
- game_server-wide storage architecture or Unity persistence implementation
- new save migration, compatibility policy, balance/content decision, formatter/commit/stash/cleanup

## Investigation Scope

1. Reconcile exact Core source/test/canonical and server persistence surface.
2. Trace start/continue command entry to `GameCoreState`/`RunState` construction and run identity/RNG ownership.
3. Trace floor/map generation, node selection/navigation/completion and next-floor/run completion transitions.
4. Identify transactional validation/commit/rollback boundaries and partial-mutation risk.
5. Trace checkpoint/export/import/version/reference validation and in-memory rollback/retry semantics.
6. Trace Core active-run payload through server messages/store/bootstrap/current+backup/checksum/atomic write.
7. Search legacy versions, alias/default/fallback, silent recovery, dual state and derived-policy recalculation.
8. Audit focused tests for success/failure/retry/reconnect/corruption/version/reference invariants.
9. Compare current runtime with nearest durable canonical and classify every signal.
10. Perform independent completion audit and production-content hash comparison.

## DTO/Data/Save Migration

No migration is implemented. Current DTO/data/save/version meanings are evidence. A candidate requiring a
version bump, old-save reader, dual-read/write, cutover, external rollout or data correction is classified as
`policy_or_feature_migration` or deferred with a separate implementation Goal requirement.

## Deployment And Compatibility

No deployment is performed. The audit records current producer/consumer topology, versions, backup/fallback
behavior and unknown consumers. Compatibility is not inferred from test fixtures.

## Focused Validation

- exact run/map/checkpoint/persistence surface and mutation trace
- focused Core unit/integration tests for run start, map progression, checkpoint and rollback
- focused server active-run store tests when they directly exercise the typed Core payload
- legacy/version/fallback/ignored-test search
- candidate-specific characterization requirements

## Broad Validation

- Phase 2 primary/shared coverage and complete signal disposition
- representative Core→server persistence round trip where current tests support it
- candidate work type/axes/readiness/invariant cross-check
- initial/final scoped dirty set and production-content hashes

## Omitted Validation

- Unity/client restore: approved out-of-scope/unverified.
- Domain economy/session/combat full suites unless directly needed for run rollback/persistence evidence.
- Formatter: investigation-only.

Actual execution, counts, failures and omissions are recorded in `EXPERIMENTS.md`.

## Completion Conditions

- Run start→state→map/floor/node progression authority is connected by evidence.
- Checkpoint/export/import/version/rollback/retry/reconnect failure meanings are mapped.
- Server active-run producer/store/bootstrap consumer seam is classified without redefining Core authority.
- Relevant live typed data, canonical and tests are separated from runtime authority and audited.
- Every investigated signal has candidate/rejected/deferred/unverified disposition.
- Every candidate satisfies CRA-05/06/07 evidence contract.
- Later domain Phases and external consumers are not over-verified.
- Only Goal/audit artifacts change; production content/status hashes are preserved.
- Phase completion audit is `Complete / High`.

## Completion Audit Evidence Matrix

| ID | Completion condition or invariant | Required evidence | Status |
| --- | --- | --- | --- |
| P2-01 | exact boundary coverage | Core/server/data/test/canonical inventory | pass |
| P2-02 | run root authority trace | entry, state construction, identity/RNG evidence | pass |
| P2-03 | map/floor/node progression trace | generation/navigation/completion mutation evidence | pass |
| P2-04 | atomicity/rollback lifecycle | validation/commit/failure/retry evidence | pass |
| P2-05 | checkpoint and version authority | export/import/reference/version evidence | pass |
| P2-06 | server persistence seam | message/store/bootstrap/checksum/backup evidence | pass |
| P2-07 | canonical and test safety net | current contract and focused runner evidence | pass |
| P2-08 | complete candidate/disposition quality | work type, axes, readiness, invariants and all signals | pass |
| P2-09 | dirty preservation/no mutation | initial/final scoped content/status comparison | pass |
| P2-10 | completion audit verdict | `COMPLETION_AUDIT.md` Complete / High | pass |

## Stop Conditions

- Current save/checkpoint/restore compatibility cannot be classified without user policy.
- Required server persistence source/test or live payload cannot be accessed.
- Material roster/economy/session/combat implementation is required to finish the run boundary.
- External consumer must be assumed verified without evidence.
- Read-only runner risks production/user state mutation.
- Frozen audit standard changes externally.
- Existing dirty changes overlap audit artifacts so they cannot be preserved/distinguished.
- Next Phase activation before this Phase obtains `Complete / High`.
