# U16-4 스트레스 전투 — Single Goal Set

## Goal Set

- Goal Set Ref: GS-CSCB-001
- Goal Structure Fact: single_phase
- Phase Plan Ref: PP-CSCB-001
- Final Policy Freeze Ref: FPF-CSCB-001
- Complete Goal Member Set: GOAL-CSCB-01
- Goal Quality Contract Version: 2
- Authoring Status: Authored
- Activation Status: Not Activated
- Implementation Authorized: No

P2G 작성과 실행권한은 별개다. 사용자 요청이 GEX 구현과 이후 비참여 서브에이전트 감사를 승인한다.

### Phase-to-Goal Member Mapping

| Phase ID | Goal ID | Assigned Final Execution Nodes |
| --- | --- | --- |
| PHASE-CSCB-01 | GOAL-CSCB-01 | AG-CSCB-RUN, WP-CSCB-VERIFY |

### Shared Validation Baseline Reuse Contract

- Initial Full Baseline Owner / Timing: GOAL-CSCB-01 owns reuse: U16-3 current Complete/High and Core고유2213, native4/server31. GEX00 actual pre-mutation source archive; no repeat by stage alone.
- Final Full Baseline Owner / Timing: GOAL-CSCB-01/WP-CSCB-VERIFY owns final impact integration after focused correction.
- Phase-Start Rule: fixed identical inputs or justified nonimpact reuse; new gameplay requires new evidence.
- Production Input Invalidation: layout→loader/catalog, continuous battle→clone/pause/query, command→route/application/host, outcome→grant/XP/node/restore.
- Test/Fixture/Probe Change: preserve original; same required rules/workload/oracles.
- Shared Boundary Change: actual public writer/reader and restored state together.
- Normal Full-Validation Count: inherited valid baseline plus one final impact integration, additional affected reruns for failures/changes.

### Integration Acceptance-to-Goal Mapping

| Acceptance ID | Disposition | Owner | Evidence |
| --- | --- | --- | --- |
| IVA-CSCB-FLOW | assigned_execution_node | GOAL-CSCB-01 WP-CSCB-VERIFY | execution/single_goal/COVERAGE.json |
| IVA-CSCB-GAME | assigned_execution_node | GOAL-CSCB-01 WP-CSCB-VERIFY | execution/single_goal/COVERAGE.json |
| IVA-CSCB-LOAN | assigned_execution_node | GOAL-CSCB-01 WP-CSCB-VERIFY | execution/single_goal/COVERAGE.json |
| IVA-CSCB-SHOP | assigned_execution_node | GOAL-CSCB-01 WP-CSCB-VERIFY | execution/single_goal/COVERAGE.json |
| IVA-CSCB-STORE | assigned_execution_node | GOAL-CSCB-01 WP-CSCB-VERIFY | execution/single_goal/COVERAGE.json |

## Single Goal Member `GOAL-CSCB-01`

- Goal ID: GOAL-CSCB-01
- Goal Kind: single
- Phase ID: PHASE-CSCB-01
- Assigned Final Execution Node Set: AG-CSCB-RUN, WP-CSCB-VERIFY
- Covered Policy Element Revision Ref Set: PER-CSCB-FLOW-REVIEWED-001, PER-CSCB-GAME-REVIEWED-001, PER-CSCB-LOAN-REVIEWED-001, PER-CSCB-SHOP-REVIEWED-001, PER-CSCB-STORE-REVIEWED-001
- Covered Decision Revision Ref Set: DR-CSCB-FLOW-001, DR-CSCB-GAME-001, DR-CSCB-LOAN-001, DR-CSCB-SHOP-001, DR-CSCB-STORE-001
- Derived Observable Policy Acceptance Coverage: FPF-CSCB-001 A01–A10 all; Completion Matrix below
- Objective / Contract-Visible Result: 정상 StressCombat에서 보호 대상·5연속라운드·고정누적보상·성공철수/패배/수동후퇴·XP/무억제·복원
- User-Visible Problem: StressCombat 배정은 존재하나 실행기 미지원
- User-Visible Completion Condition Set: RULES 10수용조건과 공개 host/복원/비용 증거
- In-Scope / Out-of-Scope: U16-4 in scope; U17/U21/Unity/개별최종보스/외부임의변조 제외; 전체U16 완료 아님
- Reviewed Current Behavior / Delta: CRA-CSCB-001 CODE_REVIEW.ko.md; baseline532; continuous BattleCore/wave runtime/common settle reused
- Goal-Local Source of Truth / Owning Layer: validated layout StressCombatPolicy / kernel wave gate and round status / ActiveBattleSession / common result XP and grant / entry checkpoint / paired host
- Goal-Local Invariant Set / Source Refs: RULES.ko.md and frozen10acceptance; one battle, frozen choice time, pending reward until success, one survival XP, no repeat node
- Boundary: RON→validation→Reward node→live defense→wave gate→choice→common outcome/XP/grant/node→Continue/SavePoint/host
- Consumer Verification Boundary: normal CoreApplication commands/query, supported external pack and fresh-process paired host
- Dependencies / Prerequisites: U04/U06/U08/U12–15/U16-1/2/3 current sources; P2G40/50/60 and GEX00
- Inherited Baseline / Impact Map: baseline532 snapshot; command/loader/kernel/pause/XP/restore impact requires integration checks
- Assigned Durable Integration Seam: 5 IVA all WP-CSCB-VERIFY normal authoring/command/query/host/restore/cost
- Migration/Compatibility/Rollout/Rollback: optional layout policy and in-memory active state; current fresh Continue remains entry rollback; immutable baseline/target and bidirectional restore
- Known Unknown / Blocker / Stop: runtime/cost pending; initial representative defense content in RULES; after fixed GEX10 summon nonparticipating audit subagent per direct user exception; no U17/U21
- Risk / Reviewer: future waves bypass gate/time advances in choice/duplicate reward/round XP/retreat cost loss; independent auditor after fixed target
- Structural Completion / Legacy and Fallback Disposition: reuse BattleCore/wave/objective/XP/grant/entry checkpoint; no new battle per round/no implicit recovery/no automatic reward fallback
- Canonical Documentation Obligation: runtime/loading contracts plus authored round gate/command/query/outcome/persistence semantics

### Completion Matrix

| Acceptance | Requirement | Evidence Owner |
| --- | --- | --- |
| A01 | A01: 정상 RON의 StressCombat 배정에서 보호 대상 방어 연속 전투를 시작하고 첫 라운드 보상을 공개한다. | WP-CSCB-VERIFY / COVERAGE.json |
| A02 | A02: 선택 구간은 계속 방어와 보상 받고 철수만 허용하며 누적/다음 보상에 적용 배율을 공개한다. | WP-CSCB-VERIFY / COVERAGE.json |
| A03 | A03: 최대5라운드의 작성된 웨이브를 모두 처치해야 각 라운드를 완료한다. 뒤 라운드는 콘텐츠로 강화한다. | WP-CSCB-VERIFY / COVERAGE.json |
| A04 | A04: HP/배치/자원/쿨다운/전투당 기록/보호 대상을 같은 전투에서 유지하며 라운드 사이 시간은 정지한다. | WP-CSCB-VERIFY / COVERAGE.json |
| A05 | A05: 개별 라운드에는 실제 보상/XP/노드 소비를 하지 않고 성공 철수 또는5라운드 완료에서 누적분을 한번 지급한다. | WP-CSCB-VERIFY / COVERAGE.json |
| A06 | A06: 보호 대상 파괴 등의 패배는 미수령 보상을 전부 상실하고 공통 직원 손실/생존 XP는 전투 전체1회 적용한다. | WP-CSCB-VERIFY / COVERAGE.json |
| A07 | A07: 전투 중 수동 후퇴는 직원/소비/장비/유물을 진입 전 복원하고 노드 소비/보상 상실/재도전 불가를 유지한다. 최초 입장 비용은 유지하며 XP는 지급하지 않는다. | WP-CSCB-VERIFY / COVERAGE.json |
| A08 | A08: 무억제 진입의 적 HP/공격력 강화를 전 라운드 유지하고 성공 XP/재료/엔케팔린 총량2배, 유물 수량 불변, 패배 기본 XP 기존량을 검증한다. | WP-CSCB-VERIFY / COVERAGE.json |
| A09 | A09: 같은 프로세스 재접속은 선택/미수령 상태를 유지하고 fresh Continue 미완료는 입장전, 완료는 결과/소비를 함께 보존한다. SavePoint와 실제 IO 실패/동일명령 재시도를 검증한다. | WP-CSCB-VERIFY / COVERAGE.json |
| A10 | A10: 정상/오류 작성과 공개 application/host/query/consumer 및 기존 영향 회귀를 검증하고 정상/확장 비용과 실제 메모리 회수를 측정한다. 외부 임의 변조 방어는 제외한다. | WP-CSCB-VERIFY / COVERAGE.json |

### Validation Dimension Matrix

| Dimension | Disposition | Scenario | Evidence |
| --- | --- | --- | --- |
| success | required | protected defense five rounds and early extraction | execution/single_goal/COVERAGE.json raw evidence |
| invalid_input | required | invalid authoring/phase/battleUUID/completed round | execution/single_goal/COVERAGE.json raw evidence |
| failure_rollback | required | protected target death/lost pending/actual IO | execution/single_goal/COVERAGE.json raw evidence |
| retry_idempotency | required | duplicate continue/extract/retreat/finish | execution/single_goal/COVERAGE.json raw evidence |
| ordering_concurrency | required | same timestamp defeat priority/serial command/application candidate | execution/single_goal/COVERAGE.json raw evidence |
| boundary_empty_max | required | five rounds/nonempty groups/reward quantities/choice clock | execution/single_goal/COVERAGE.json raw evidence |
| disconnect_restart_persistence | required | same process/fresh incomplete-complete/SavePoint | execution/single_goal/COVERAGE.json raw evidence |
| live_data | required | normal authored RON pack and invalid references | execution/single_goal/COVERAGE.json raw evidence |
| producer_consumer | required | loader/application/host/query | execution/single_goal/COVERAGE.json raw evidence |
| legacy_fallback_absence | required | ordinary battle and other reward activities unchanged/no reset/no extra reward | execution/single_goal/COVERAGE.json raw evidence |

## Policy–Phase–Goal Traceability

| Stable Element ID | Policy Element Revision Ref | Decision Revision Ref | Primary Phase ID | Single/Phase Goal ID |
| --- | --- | --- | --- | --- |
| PE-CSCB-FLOW | PER-CSCB-FLOW-REVIEWED-001 | DR-CSCB-FLOW-001 | PHASE-CSCB-01 | GOAL-CSCB-01 |
| PE-CSCB-GAME | PER-CSCB-GAME-REVIEWED-001 | DR-CSCB-GAME-001 | PHASE-CSCB-01 | GOAL-CSCB-01 |
| PE-CSCB-LOAN | PER-CSCB-LOAN-REVIEWED-001 | DR-CSCB-LOAN-001 | PHASE-CSCB-01 | GOAL-CSCB-01 |
| PE-CSCB-SHOP | PER-CSCB-SHOP-REVIEWED-001 | DR-CSCB-SHOP-001 | PHASE-CSCB-01 | GOAL-CSCB-01 |
| PE-CSCB-STORE | PER-CSCB-STORE-REVIEWED-001 | DR-CSCB-STORE-001 | PHASE-CSCB-01 | GOAL-CSCB-01 |
