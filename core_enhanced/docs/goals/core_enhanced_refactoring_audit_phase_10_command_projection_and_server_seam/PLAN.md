# Core Enhanced Refactoring Audit Phase 10 — Command Projection And Server Seam Goal

Status: complete
Implementation Complete: not_applicable
Goal Type: phase
Parent Master: `core_enhanced_refactoring_audit_master`
Phase: 10
Execution Mode: investigation_only
Production Mutation Authorized: no
Audit Baseline Date: 2026-08-07
Start Commit: `f318414fb0156a67e04c5479000c2a2683fc0c09`

## Objective

Current working tree의 `PlayerBehavior`/`BehaviorResult`, execution/transport context, action and battle capability,
snapshot/range/revision projection, admin/fixture/package target와 direct `game_server` bridge를 실제 producer,
serialization, consumer와 test-code evidence로 감사한다. Domain 정책을 DTO shape가 재정의하지 않게 유지하면서
projection ownership, duplicate mapping, stale fixture/harness, partial validation과 external compatibility 후보를
실행 가능한 catalog로 분류한다.

## Parent Scope And Direct Dependency

- Parent: `core_enhanced_refactoring_audit_master`
- Boundary: Phase 0 `BOUNDARY_PROPOSAL.md` and `SURFACE_INVENTORY.md`
- Direct predecessor: Phase 9 `Complete / High`
- Phases 1~9 own domain state, mutation and gameplay semantics; this Phase owns only command/projection/transport
  refactor discovery around them.
- Final consolidation is denied until this Phase has `Complete / High`.

## Approved Requirements

Parent CRA-01~CRA-14 all apply. Confirmed candidates need exact authority/consumer/test/canonical evidence, preserved
invariants, removal scope, work type, seven axes, readiness and policy/external gates. Every assigned test/fixture
family needs meaning/reachability/assertion/overlap disposition.

## Invariants

- `GameCoreState` and audited domain handlers remain mutation authority; DTO/capability/snapshot code derives state.
- `GameCore::execute_with_transport_context` remains the single player-command entry and applies validation/rollback
  semantics before server persistence/publication.
- Server bridge serializes one Core result/snapshot contract and does not infer gameplay authorization independently.
- Capability and range projections describe commands accepted for the same revision/context without becoming a
  competing validation authority.
- Revision/reconnect/persistence ordering and error/result wire shape remain explicit and deterministic.
- Admin commands and generated fixtures are classified separately from player production authority.
- No production code/data/test/canonical/server/Unity mutation occurs.

## In Scope

- `core_enhanced/src/game/behavior.rs`
- command/projection slices of `src/game/world.rs`, `world/{action_capability,battle_capability,combat_trace,snapshot,
  state}.rs`
- `world/admin.rs` and `world/admin/**` as non-player command/fixture boundary
- `src/{lib,main}.rs`, `src/bin/**`, `Cargo.toml`, `.cargo/config.toml` and Core CI target wiring
- focused inline/world snapshot, capability, execution, rollback, revision and admin tests
- direct `game_server` path dependency and load-balance/player-game actor command/result/snapshot mapping
- server contract fixture examples and nearest bridge/protocol canonical as supporting consumer evidence
- Phase 9 C04/C07 projection/public-consumer gates, without reopening target gameplay

## Out Of Scope

- production mutation
- domain state/mutation candidate rediscovery already owned by Phases 1~9
- persistence implementation refactors except ordering/DTO evidence at the direct bridge
- independent `game_server` architecture/refactor audit
- Unity runtime/test/repository; protocol documentation is supporting evidence only
- new command, admin, authorization, DTO, reconnect, revision or compatibility policy
- deployment/ops/network performance and generated fixture regeneration

## Investigation Scope

1. Reconcile Core command/result/snapshot/capability/package and direct server consumer inventory.
2. Trace player request → transport context → Core validation/mutation/rollback → result/snapshot/revision.
3. Trace capability/range/snapshot projection back to audited domain authorities and server wire mapping.
4. Separate admin commands, fixture generation, package binaries and test harnesses from production player paths.
5. Search duplicated mapping, fallback/default wire shape, partial serialization, false capability authority and
   disconnected targets/tests.
6. Apply CRA-14, prove no production/server mutation and perform an independent completion audit.

## Focused Validation

- behavior serialization/validation/execution and rollback families
- action/battle capability, snapshot/range/revision and reconnect families
- server actor Core mapping/result publication tests if available
- admin command/fixture and generated-contract source assertions
- runner only after static gaps are recorded

## Omitted Validation

- whole Core/server workspace by default
- domain numerical/gameplay correctness already dispositioned
- Unity execution or regenerated artifacts
- formatter and production mutation

## Completion Evidence Matrix

| ID | Requirement | Required evidence | Status |
| --- | --- | --- | --- |
| P10-01 | exact boundary | Core/server/package/test/canonical inventory | complete — `BOUNDARY_AUDIT.md` |
| P10-02 | command entry | request/context/validation/mutation/rollback trace | complete — `BOUNDARY_AUDIT.md` |
| P10-03 | result/wire | BehaviorResult/error/serialization consumer trace | complete — `BOUNDARY_AUDIT.md` |
| P10-04 | capability | projection→domain authority alignment | complete — `BOUNDARY_AUDIT.md` |
| P10-05 | snapshot/revision | state/range/reconnect projection trace | complete — `BOUNDARY_AUDIT.md` |
| P10-06 | server seam | direct mapping/persistence/publication classification | complete — `BOUNDARY_AUDIT.md` |
| P10-07 | admin/fixture/package | reachability and authority disposition | complete — boundary/catalog |
| P10-08 | predecessor seams | Phase 1~9 carrier/owner classification | complete — boundary reconciliation |
| P10-09 | CRA-14 | test/fixture family disposition catalog | complete — `TEST_AUDIT.md` |
| P10-10 | candidates/signals | type/axes/readiness/disposition | complete — `CANDIDATE_CATALOG.md` |
| P10-11 | no mutation | initial/final Core/server/status comparison | complete — all ten hashes identical |
| P10-12 | completion | independent `Complete / High` | complete — `COMPLETION_AUDIT.md` |

## Stop Conditions

- a new gameplay/admin/authorization/DTO/reconnect/compatibility policy is required to classify rather than defer
- independent server or Unity semantics must be treated as verified beyond the direct Core seam
- required producer/consumer/test/canonical evidence is inaccessible
- read-only work risks existing dirty state or production/server mutation
