# U16-3 딜러와 미니게임 — Single Goal Set

## Goal Set

- Goal Set Ref: GS-CDMG-001
- Goal Structure Fact: single_phase
- Phase Plan Ref: PP-CDMG-001
- Final Policy Freeze Ref: FPF-CDMG-001
- Complete Goal Member Set: GOAL-CDMG-01
- Goal Quality Contract Version: 2
- Authoring Status: Authored
- Activation Status: Not Activated
- Implementation Authorized: No

P2G 작성과 실행권한은 별개다. 사용자 요청이 GEX 구현과 이후 비참여 서브에이전트 감사를 승인한다.

### Phase-to-Goal Member Mapping

| Phase ID | Goal ID | Assigned Final Execution Nodes |
| --- | --- | --- |
| PHASE-CDMG-01 | GOAL-CDMG-01 | AG-CDMG-RUN, WP-CDMG-VERIFY |

### Shared Validation Baseline Reuse Contract

- Initial Full Baseline Owner / Timing: GOAL-CDMG-01은 U16-2 independent001 current Complete/High and Core고유2196, native4/server31. GEX00 actual pre-mutation source archive; no repeat by stage alone.
- Final Full Baseline Owner / Timing: GOAL-CDMG-01/WP-CDMG-VERIFY owns final impact integration after focused correction.
- Phase-Start Rule: fixed identical inputs or justified nonimpact reuse; new gameplay requires new evidence.
- Production Input Invalidation: layout/VN→loader/catalog, session/escrow→clone/restore/query, command→route/application/host, asset→grant/UUID/public inventory.
- Test/Fixture/Probe Change: preserve original; same required rules/workload/oracles.
- Shared Boundary Change: actual public writer/reader and restored state together.
- Normal Full-Validation Count: inherited valid baseline plus one final impact integration, additional affected reruns for failures/changes.

### Integration Acceptance-to-Goal Mapping

| Acceptance ID | Disposition | Owner | Evidence |
| --- | --- | --- | --- |
| IVA-CDMG-FLOW | assigned_execution_node | GOAL-CDMG-01 WP-CDMG-VERIFY | execution/single_goal/COVERAGE.json |
| IVA-CDMG-GAME | assigned_execution_node | GOAL-CDMG-01 WP-CDMG-VERIFY | execution/single_goal/COVERAGE.json |
| IVA-CDMG-LOAN | assigned_execution_node | GOAL-CDMG-01 WP-CDMG-VERIFY | execution/single_goal/COVERAGE.json |
| IVA-CDMG-SHOP | assigned_execution_node | GOAL-CDMG-01 WP-CDMG-VERIFY | execution/single_goal/COVERAGE.json |
| IVA-CDMG-STORE | assigned_execution_node | GOAL-CDMG-01 WP-CDMG-VERIFY | execution/single_goal/COVERAGE.json |

## Single Goal Member `GOAL-CDMG-01`

- Goal ID: GOAL-CDMG-01
- Goal Kind: single
- Phase ID: PHASE-CDMG-01
- Assigned Final Execution Node Set: AG-CDMG-RUN, WP-CDMG-VERIFY
- Covered Policy Element Revision Ref Set: PER-CDMG-FLOW-REVIEWED-001, PER-CDMG-GAME-REVIEWED-001, PER-CDMG-LOAN-REVIEWED-001, PER-CDMG-SHOP-REVIEWED-001, PER-CDMG-STORE-REVIEWED-001
- Covered Decision Revision Ref Set: DR-CDMG-FLOW-001, DR-CDMG-GAME-001, DR-CDMG-LOAN-001, DR-CDMG-SHOP-001, DR-CDMG-STORE-001
- Derived Observable Policy Acceptance Coverage: FPF-CDMG-001 A01–A21 all; Completion Matrix below
- Objective / Contract-Visible Result: 딜러 VN/세 게임/갬빗/담보/정산/고정상품/나가기의 정상 명령·조회·저장
- User-Visible Problem: Gambling is allocated but executor absent
- User-Visible Completion Condition Set: all fixed flow/rules/asset atomicity and unfinished/completed recovery evidence
- In-Scope / Out-of-Scope: U16-4/U21/Unity/individual final bosses/arbitrary external tampering; U16-3 in scope
- Reviewed Current Behavior / Delta: CRA-CDMG-001 CODE_REVIEW.ko.md; existing reward/grant/VN/paired store reused
- Goal-Local Source of Truth / Owning Layer: validated layout DealerPolicy; RewardSession dealer state; escrow owns locked copies; inventory live copies; run explanation; node checkpoint; paired host
- Goal-Local Invariant Set / Source Refs: RULES.ko.md and all frozen acceptance; no duplicated grant/hidden RNG/point clamp/partial asset publication
- Boundary: RON→validation→map/VN→dealer commands/rounds→escrow/shop/grant→completion→Continue/SavePoint/host
- Consumer Verification Boundary: normal CoreApplication commands/query, supported external pack and fresh-process paired host
- Dependencies / Prerequisites: U04/U08/U12–15/U16-1/U16-2 current audited sources; P2G40/50/60 and GEX00
- Inherited Baseline / Impact Map: Shared reuse contract and CODE_REVIEW.ko.md; preserve historical targets
- Assigned Durable Integration Seam: 5 IVA all WP-CDMG-VERIFY normal authoring/command/query/host/restore/cost
- Migration/Compatibility/Rollout/Rollback: optional/default new session and run field; preserve readable current saves; explicit original-preserving unsupported successor only if codec requires; isolated bidirectional restore
- Known Unknown / Blocker / Stop: runtime/cost pending; initial tuning in RULES; after GEX10 summon nonparticipating audit subagent per direct user exception; no U16-4
- Risk / Reviewer: hole-card leakage/stale revision/duplicate base points/unlocked collateral/debt or purchase loss/unfinished cost-only restore; nonparticipating audit after fixed source
- Structural Completion / Legacy and Fallback Disposition: reuse grant/asset/VN/session/transaction, no separate asset engine, no U16-4 or generic relic sales, no fallback outcome rerolls
- Canonical Documentation Obligation: runtime/loading contracts plus new authored rules/command/query/persistence semantics

### Completion Matrix

| Acceptance | Requirement | Evidence Owner |
| --- | --- | --- |
| A01 | A01: 고정 Gambling 배정을 정상 Reward 경로로 실행하고 제안 VN의 수락/거절을 처리한다. 거절은 무보상 노드 소비다. 원정 첫/이후 설명과 다음 원정 초기화를 구분한다. | WP-CDMG-VERIFY / COVERAGE.json |
| A02 | A02: 기본 포인트는 조우당 한번이고 첫 게임 선택 후 시작 전에 공개된2/3배 갬빗을1회 선택해 조우 전체에 고정한다. 비활성1배이며 게임 순손익에만 적용한다. | WP-CDMG-VERIFY / COVERAGE.json |
| A03 | A03: 양수 잔액에서 판수 제한 없이 게임 변경/반복하고0이하 시작/계속은 거절한다. 음수 손실을 자르지 않고 대출/정산은 허용한다. | WP-CDMG-VERIFY / COVERAGE.json |
| A04 | A04: 판 전 취소는 무차감/갬빗 유지, 시작 후 포기는 갬빗 손실1회다. 단계/현재 revision이 다른 중복 명령은 무변경 거절한다. | WP-CDMG-VERIFY / COVERAGE.json |
| A05 | A05: 블랙잭은52장 단일덱 S17, 유연한Ace, natural3:2/일반1:1/push0, hit/stand/첫2장double을 구현한다. split/insurance는 선택 판본에서 제공하지 않는다. NPC2명은 각자 딜러와 승부한다. | WP-CDMG-VERIFY / COVERAGE.json |
| A06 | A06: 주사위 포커는5주사위 최대3회, 선택hold/해제/조기stand와 두NPC 조합비교를 구현한다. five/four/fullhouse/straight/three/twopair/pair/high 순서와 kicker/tie를 검증한다. | WP-CDMG-VERIFY / COVERAGE.json |
| A07 | A07: Flip7 기본판3.1의94장, duplicate bust/SecondChance/Freeze/FlipThree/score modifier/7숫자 즉시전체종료/+15, 강제3장 중 지연action과 대상선택을 구현한다. | WP-CDMG-VERIFY / COVERAGE.json |
| A08 | A08: Flip7은 한 라운드1판이며 자기 점수 환산과 비버스트1위 보너스/공동1위 균등분배·나머지버림 후 갬빗을 적용한다. bust에는 공개 기본손실을 적용한다. | WP-CDMG-VERIFY / COVERAGE.json |
| A09 | A09: 플레이어1/NPC2의 공개 카드·주사위·점수·상태를 표시하고 NPC는 자기/공개정보만으로 보수/공격 결정을 내린다. dealer hole/미래draw/RNG seed는 조회에 노출하지 않는다. | WP-CDMG-VERIFY / COVERAGE.json |
| A10 | A10: 판 사이 복수 미장착 사본별 원금/지급후잔액을 미리 공개한다. 동일/장착/없는/이미담보사본을 거절하고 합계지급후양수여야 확정한다. 취소는 무변경이다. | WP-CDMG-VERIFY / COVERAGE.json |
| A11 | A11: 대출확정은 사본잠금/포인트지급을 원자적으로 처리하고 게임선택으로 복귀한다. 갬빗은 원금에 적용하지 않는다. 담보의 사용/장착/침식/합성/분쇄를 막는다. | WP-CDMG-VERIFY / COVERAGE.json |
| A12 | A12: 정산 후 게임/새대출로 돌아가지 않는다. 복수 회수선택의 총상환액/회수후잔액/포기사본을 공개하고 전액원금지불 가능할 때만 회수한다. 일부회수/나머지포기와 전부포기를 지원한다. | WP-CDMG-VERIFY / COVERAGE.json |
| A13 | A13: 회수는 원래UUID/정의/침식상태를 보존하고 포기는 해당사본제거/빚소멸만 한다. 상환부족/중복확정은 자산/선택/포인트 불변이다. | WP-CDMG-VERIFY / COVERAGE.json |
| A14 | A14: 게임 전에 일반/묶음/희귀/특급 authored상품 효과·Normal상태·가격·재고를 공개하고 해당조우에서 재추첨/변경하지 않는다. 공식공급 공백은 빈상점으로 처리한다. | WP-CDMG-VERIFY / COVERAGE.json |
| A15 | A15: 담보정산 뒤 실제 남은포인트로 여러상품을 구매한다. 가격/재고/사본grant를 원자화하고 잔액부족/품절/잘못된상품/오래된revision은 무변경 거절한다. | WP-CDMG-VERIFY / COVERAGE.json |
| A16 | A16: 가격/대출/원금/기본지급에는 갬빗을 적용하지 않는다. 잔액0/음수에서도 상점목록과나가기 제공, 나가기에서잔여포인트소멸/노드완료를 확정한다. | WP-CDMG-VERIFY / COVERAGE.json |
| A17 | A17: 정상 RON으로 포인트/갬빗비중/원금/상품과VN을 작성하고 누락참조/중복/0가격·재고/잘못된비중·숫자·VN선택을 로더가 거절한다. | WP-CDMG-VERIFY / COVERAGE.json |
| A18 | A18: 같은process 재접속은 현재 단계/카드/고정상품/담보를 유지한다. 미완료 fresh Continue는 모든중간게임/대출/정산/구매를입장전으로 함께 복원하고 동일입력 난수를 유지한다. | WP-CDMG-VERIFY / COVERAGE.json |
| A19 | A19: 완료 Continue는 원래사본의 회수/포기/구매/노드소비를 보존하고 재지급하지 않는다. SavePoint도 비용·결과·원정설명 상태를 함께 복원한다. | WP-CDMG-VERIFY / COVERAGE.json |
| A20 | A20: 실제host 저장IO실패에서메모리/active/profile 원본불변과유효동일명령재시도를 확인한다. 일반reward/U16-2/shared store/command consumer 회귀를 검증한다. | WP-CDMG-VERIFY / COVERAGE.json |
| A21 | A21: 정상/확장상품·담보·반복판의시간/실제heap/RSS/회수와내부작업량을 구분측정한다. 유한한도/공통grant/복원가능target과17선행증거를 보존하고 구현비참여 서브에이전트가 독립감사한다. | WP-CDMG-VERIFY / COVERAGE.json |

### Validation Dimension Matrix

| Dimension | Disposition | Scenario | Evidence |
| --- | --- | --- | --- |
| success | required | 3 games/VN/loan/redeem/shop/exit | execution/single_goal/COVERAGE.json raw evidence |
| invalid_input | required | authoring/phase/target/card/dice/copy/stale revision | execution/single_goal/COVERAGE.json raw evidence |
| failure_rollback | required | insufficient points/equipped copies/IO | execution/single_goal/COVERAGE.json raw evidence |
| retry_idempotency | required | same revision/confirm/purchase/exit/reconnect | execution/single_goal/COVERAGE.json raw evidence |
| ordering_concurrency | required | serial command revision and paired atomic publication | execution/single_goal/COVERAGE.json raw evidence |
| boundary_empty_max | required | empty offers/zero/negative/full wager/natural/dice ties/Flip7 modifiers | execution/single_goal/COVERAGE.json raw evidence |
| disconnect_restart_persistence | required | same process/fresh incomplete-complete/SavePoint | execution/single_goal/COVERAGE.json raw evidence |
| live_data | required | normal authored RON pack/VN and invalid references | execution/single_goal/COVERAGE.json raw evidence |
| producer_consumer | required | loader/application/host/query | execution/single_goal/COVERAGE.json raw evidence |
| legacy_fallback_absence | required | existing3reward activities unchanged/no point clamp/re-roll/asset sales | execution/single_goal/COVERAGE.json raw evidence |

## Policy–Phase–Goal Traceability

| Stable Element ID | Policy Element Revision Ref | Decision Revision Ref | Primary Phase ID | Single/Phase Goal ID |
| --- | --- | --- | --- | --- |
| PE-CDMG-FLOW | PER-CDMG-FLOW-REVIEWED-001 | DR-CDMG-FLOW-001 | PHASE-CDMG-01 | GOAL-CDMG-01 |
| PE-CDMG-GAME | PER-CDMG-GAME-REVIEWED-001 | DR-CDMG-GAME-001 | PHASE-CDMG-01 | GOAL-CDMG-01 |
| PE-CDMG-LOAN | PER-CDMG-LOAN-REVIEWED-001 | DR-CDMG-LOAN-001 | PHASE-CDMG-01 | GOAL-CDMG-01 |
| PE-CDMG-SHOP | PER-CDMG-SHOP-REVIEWED-001 | DR-CDMG-SHOP-001 | PHASE-CDMG-01 | GOAL-CDMG-01 |
| PE-CDMG-STORE | PER-CDMG-STORE-REVIEWED-001 | DR-CDMG-STORE-001 | PHASE-CDMG-01 | GOAL-CDMG-01 |
