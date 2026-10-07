# Core Enhanced Refactoring Audit Phase 06 — Battle Lifecycle And Result Goal

Status: complete
Implementation Complete: not_applicable
Goal Type: phase
Parent Master: `core_enhanced_refactoring_audit_master`
Phase: 6
Execution Mode: investigation_only
Production Mutation Authorized: no
Audit Baseline Date: 2026-08-07
Start Commit: `f318414fb0156a67e04c5479000c2a2683fc0c09`

## Objective

현재 working tree의 battle execution lifecycle, deterministic event queue/clock, scenario runtime, event log,
recording/revision, active-session playback, retreat/retry/finalization과 result-stat/run-result handoff를 runtime,
canonical과 test-code evidence로 감사한다. Phase 5가 넘긴 validated setup에서 시작해 spatial/effect/targeting
세부 규칙을 각 후속 Phase의 carrier로만 취급하면서 competing lifecycle authority, partial commit, duplicated
event/result conversion과 test fragility를 실행 가능한 후보 또는 rejected/deferred/unverified disposition으로
분류한다.

## Parent Scope And Direct Dependency

- Parent: `core_enhanced_refactoring_audit_master`
- Boundary: Phase 0 `BOUNDARY_PROPOSAL.md` and `SURFACE_INVENTORY.md`
- Direct predecessor: Phase 5 `Complete / High`
- Phase 5 handoff: preview/setup is validated before `ActiveBattleSession`; its cache/publication candidates are not
  reopened.
- Phase 2 owns durable run/checkpoint, Phase 3 persistent economy, Phase 4 interactive/Event session lifecycle.
- Phase 7 owns spatial/tactical mutation, Phase 8 effects/stats/death, Phase 9 targeting/delivery, Phase 10 DTO/server.
- Phase 7 is denied until this Phase has `Complete / High`.

## Approved Requirements

Parent CRA-01~CRA-14 all apply. Every confirmed candidate needs actual runtime/test/canonical evidence,
current→target authority, preserved behavior, removal scope, seven axes, independent readiness and policy decision.
Every assigned test family needs CRA-14 meaning/reachability/assertion/overlap disposition.

## Invariants

- `BattleExecutionState` and the authoritative event queue own battle time, finish and finalize lifecycle.
- `BattlePlaybackState` gates production time advance; pause/speed/focus must not create a second simulation clock.
- Event ordering is deterministic at equal time, append-only log seq is monotonic and result calculation consumes
  committed runtime events without replaying gameplay policy.
- Successful command-time mutations commit one battle revision; read-only recovery/preview and failed commands do
  not consume revision or advance time.
- Retreat/retry/result completion preserve exact attempt, entry snapshot, run/economy and compendium lifetimes.
- Invalid/stale lifecycle state fails before world/run/result mutation, or is covered by an explicit transaction.
- Phase 7~9 domain state may be carried and observed but its formula/geometry/target correctness is not decided here.
- No production code/data/test/canonical/server/Unity mutation occurs.

## In Scope

- top-level battle lifecycle modules: `battle/{enums,event_log,ids,mod,recording,result_stats,scenario,types}.rs`
- `battle/core/{mod,build,commands,ids,sim,types}.rs` lifecycle/event-queue portions
- validation harness only at scenario/spawn/state/event/result lifecycle seams
- `world/{combat,combat_trace,state}.rs` active session, playback/tick, revision, retreat/retry/result portions
- `determinism.rs` and UUID/session identity only where consumed by battle lifecycle
- run-policy fields consumed by playback, attempts and post-battle/result handling
- nearest canonical and focused core/world/result/event-log tests

## Out Of Scope

- production mutation
- preview generation, setup feasibility and enemy/player draft construction (Phase 5)
- movement, occupancy, field work, tactical resource/deploy/stance/relocation/swap semantics (Phase 7)
- stat pipeline, damage, buff, resonance, death/incapacitation formula correctness (Phase 8)
- basic attack, skill targeting/projectile/area delivery correctness (Phase 9)
- DTO/message/server/Unity refactor discovery (Phase 10)
- gameplay/balance changes, new battle modes, event-log compatibility or save migration policy

## Investigation Scope

1. Reconcile exact lifecycle source/test/canonical/consumer inventory.
2. Trace setup handoff → execution start → event scheduling/step → finish/finalize → world result/completion.
3. Trace playback pause/speed, revision, command/read-only and initial/ongoing advance authority.
4. Trace event-log seq/root/source-command recording, result stats and debug trace lifetimes.
5. Trace retreat, retryable/exhausted defeat, checkpoint decision and victory/run completion atomicity.
6. Search duplicated lifecycle state, event/result mappings, fallbacks and manual transaction/rollback code.
7. Apply CRA-14 to assigned test families and prove no production mutation.

## Focused Validation

- battle execution/event-order/log/result unit families
- world pause/speed/revision/setup-recovery/retreat/retry/result atomicity tests
- exact current initial/finish/finalize and stale/no-mutation boundaries
- runner only after a static gap is recorded

## Omitted Validation

- whole project by default; green count is not test-quality proof
- Phase 7~9 formula/geometry/target assertions except lifecycle carrier evidence
- server/Unity and external public consumers except explicit unverified classification
- formatter and production mutation

## Completion Conditions

- exact lifecycle/event/result authority trace and all signal dispositions
- predecessor/later seams classified without duplicate candidates
- deterministic ordering, revision, retreat/retry/result failure semantics mapped
- CRA-14 test-family catalog complete
- every candidate satisfies CRA-05/06/07 fields
- initial/final production hashes/status identical
- independent audit `Complete / High`

## Completion Evidence Matrix

| ID | Requirement | Required evidence | Status |
| --- | --- | --- | --- |
| P6-01 | exact boundary | source/test/canonical/consumer inventory | pass |
| P6-02 | execution authority | clock/queue/start/step/finish/finalize trace | pass |
| P6-03 | event/log authority | ordering/seq/root/recording trace | pass |
| P6-04 | world lifecycle | playback/revision/advance/read-only trace | pass |
| P6-05 | result lifecycle | stats/retreat/retry/complete/rollback trace | pass |
| P6-06 | predecessor/later seams | carrier/owner classification | pass |
| P6-07 | canonical alignment | current contract comparison | pass |
| P6-08 | CRA-14 | test family disposition catalog | pass |
| P6-09 | candidates/signals | complete work type/axes/readiness/disposition | pass |
| P6-10 | no mutation | initial/final content/status comparison | pass |
| P6-11 | completion | independent `Complete / High` | pass |

## Stop Conditions

- new playback/result/retry policy is required to classify a candidate
- Phase 7~9 mechanics must be treated as verified to decide lifecycle ownership
- required production/test/canonical evidence is inaccessible
- read-only work risks existing dirty state or production mutation
