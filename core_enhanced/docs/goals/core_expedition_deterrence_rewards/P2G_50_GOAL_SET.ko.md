# U14 입장 비용·억제력·Breakdown·황금가지·고정 난이도와 보상 — Single Goal Set

## Goal Set

- Goal Set Ref: GS-CEDR-001
- Goal Structure Fact: single_phase
- Phase Plan Ref: PP-CEDR-001
- Final Policy Freeze Ref: FPF-CEDR-001
- Complete Goal Member Set: GOAL-CEDR-01
- Goal Quality Contract Version: 2
- Authoring Status: Authored
- Activation Status: Not Activated
- Implementation Authorized: No

P2G 작성 상태와 사용자 U14 착수 요청에 따른 GEX 권한을 구분한다. 정식31수용 조건은 동결 원문 그대로 인수한다.

### Phase-to-Goal Member Mapping

| Phase ID | Goal ID | Assigned Final Execution Nodes |
| --- | --- | --- |
| PHASE-CEDR-01 | GOAL-CEDR-01 | AG-CEDR-RUN, WP-CEDR-PUBLIC, WP-CEDR-VERIFY |

### Shared Validation Baseline Reuse Contract

- Initial Full Baseline Owner / Timing: GOAL-CEDR-01은 U13 audit/u13_independent_002 Complete/High/open0 및 U04/U12 비영향 근거를 인수한다. 제품 수정 전에 현행 dirty 포함 recoverable baseline을 보관한다. 상시지시서§4의 유효 근거 재사용에 따라 착수만으로 전체시험을 반복하지 않는다.
- Final Full Baseline Owner / Timing: GOAL-CEDR-01/WP-CEDR-VERIFY가 최종31수용·10차원·10IVA와 실제 공유경계 변경의 관련 회귀를 검증한다. 원시 결과와 명령/선택수/종료를 EXPERIMENTS에 고정한다.
- Phase-Start Rule: current source/fixture/method가 같거나 비영향인 실제 증거만 재사용한다. 새 기능의 미실행은 통과로 바꾸지 않는다.
- Production Input Invalidation: 입장·Breakdown→노드/치료/실패/복원, 배율/보상→실전/XP/모든 지급경로, 황금가지→생성/지도/결과, 저장→Continue/SavePoint/host IO 회귀를 재검증한다.
- Test/Fixture/Probe Change: 규칙 기반 기대값과 실제 정상 자격을 보존하며 실패 원문을 남기고 영향범위 재실행한다.
- Shared Boundary Change: codec/clone/pack/normal host는 reader/writer·consumer 영향 회귀까지 확인한다.
- Normal Full-Validation Count: 유효 초기 재사용과 최종 영향통합1회가 기본이다. 실패·추가 변경·남은 의문이 있을 때 필요한 검증만 확대한다.

### Integration Acceptance-to-Goal Mapping

| Acceptance ID | Disposition | Owner | Evidence |
| --- | --- | --- | --- |
| IVA-CEDR-G13 | assigned_execution_node | GOAL-CEDR-01 WP-CEDR-VERIFY | execution/single_goal/COVERAGE.json의 해당 요소·원시 결과 |
| IVA-CEDR-G14 | assigned_execution_node | GOAL-CEDR-01 WP-CEDR-VERIFY | execution/single_goal/COVERAGE.json의 해당 요소·원시 결과 |
| IVA-CEDR-G15 | assigned_execution_node | GOAL-CEDR-01 WP-CEDR-VERIFY | execution/single_goal/COVERAGE.json의 해당 요소·원시 결과 |
| IVA-CEDR-PWR01 | assigned_execution_node | GOAL-CEDR-01 WP-CEDR-VERIFY | execution/single_goal/COVERAGE.json의 해당 요소·원시 결과 |
| IVA-CEDR-PWR02 | assigned_execution_node | GOAL-CEDR-01 WP-CEDR-VERIFY | execution/single_goal/COVERAGE.json의 해당 요소·원시 결과 |
| IVA-CEDR-G16 | assigned_execution_node | GOAL-CEDR-01 WP-CEDR-VERIFY | execution/single_goal/COVERAGE.json의 해당 요소·원시 결과 |
| IVA-CEDR-G17 | assigned_execution_node | GOAL-CEDR-01 WP-CEDR-VERIFY | execution/single_goal/COVERAGE.json의 해당 요소·원시 결과 |
| IVA-CEDR-G18 | assigned_execution_node | GOAL-CEDR-01 WP-CEDR-VERIFY | execution/single_goal/COVERAGE.json의 해당 요소·원시 결과 |
| IVA-CEDR-G19 | assigned_execution_node | GOAL-CEDR-01 WP-CEDR-VERIFY | execution/single_goal/COVERAGE.json의 해당 요소·원시 결과 |
| IVA-CEDR-PERSISTENCE | assigned_execution_node | GOAL-CEDR-01 WP-CEDR-VERIFY | execution/single_goal/COVERAGE.json의 해당 요소·원시 결과 |

## Single Goal Member `GOAL-CEDR-01`

- Goal ID: GOAL-CEDR-01
- Goal Kind: single
- Phase ID: PHASE-CEDR-01
- Assigned Final Execution Node Set: AG-CEDR-RUN, WP-CEDR-PUBLIC, WP-CEDR-VERIFY
- Covered Policy Element Revision Ref Set: PER-CEDR-G13-REVIEWED-001, PER-CEDR-G14-REVIEWED-001, PER-CEDR-G15-REVIEWED-001, PER-CEDR-PWR01-REVIEWED-001, PER-CEDR-PWR02-REVIEWED-001, PER-CEDR-G16-REVIEWED-001, PER-CEDR-G17-REVIEWED-001, PER-CEDR-G18-REVIEWED-001, PER-CEDR-G19-REVIEWED-001, PER-CEDR-PERSISTENCE-REVIEWED-001
- Covered Decision Revision Ref Set: DR-CEDR-G13-001, DR-CEDR-G14-001, DR-CEDR-G15-001, DR-CEDR-PWR01-001, DR-CEDR-PWR02-001, DR-CEDR-G16-001, DR-CEDR-G17-001, DR-CEDR-G18-001, DR-CEDR-G19-001, DR-CEDR-PERSISTENCE-001
- Derived Observable Policy Acceptance Coverage: FPF-CEDR-001 A01–A31 전부. Completion Matrix 원문을 실제 evidence에 연결한다.
- Objective / Contract-Visible Result: 작성 노드비용→유예/사망→소진선택→활동, authored 고정 난이도/무억제 snapshot, 승리 수량보상2배·실제성과·황금가지/봉인, 공개조회·저장·복원을 같은 정상 원정 경로에서 제공한다.
- User-Visible Problem: 현재 완료노드 기반 유예와 입장후 이어하기 snapshot은 새 규칙과 다르고 무억제/봉인/성과 producer가 없다. 편성 최고전투력이 적 난이도에 사용된다.
- User-Visible Completion Condition Set: 최초입장1회 규정비용과4유예가 치료 전에 적용되고 같은노드 재시도는 중복하지 않는다. 작성 적 난이도와 공개배율이 실전 일치하며 실제승리 재료/XP만2배다. 봉인 승리 가지/보충과 후퇴/Continue의 서로 다른 복원 경계를 지킨다.
- In-Scope / Out-of-Scope: U14 G13/G14/G15/PWR01/PWR02/G16/G17/G18/G19와 전요소의 지원저장/공개/제작경계를 포함한다. 최종보스 본체/Unity, U16 스트레스 활동 전체/U17 실제부화/U19·U21 개별4보스 본체, 배포·외부임의변조 전용 방어는 제외한다. 후속 공통 seam의 검증은 U14에 포함한다.
- Reviewed Current Behavior / Delta: CODE_REVIEW.ko.md의 ENTRY/COMBAT/REWARD/GOLD/STORE/PUBLIC 경계 및 CRA-CEDR-001. U12 receipt와 U04 staged result/Continue를 재사용하고 규정유예·전투 snapshot·plan을 확장하며 완료노드감소/adaptive power 의존을 제거한다.
- Goal-Local Source of Truth / Owning Layer: authored 비용/배율/계획/성과는 validated data, receipt/억제력/가지배정·회수는 run, 유예는 employee, 실전성과는 battle runtime, 결과는 staged grant와 confirmed result, 저장은 existing application/paired host가 소유한다. DTO/cache는 권위가 아니다.
- Goal-Local Invariant Set / Source Refs: FPF-CEDR-001 A01–A31. 첫입장1회, 실제 잔량과 별개 규정유예, 사망선행, 진입후 자격고정, HP/ATK만 강화, revival×unrestrained·seal중복금지, whitelist 보상 총량2배, victory-only gold, 미확정 pre-entry Continue/post-entry retreat, hidden identity 보호.
- Boundary: typed RON/pack→validation→facility/run→public command/preview→spawn/actual damage/time→staged result→checkpoint/Continue→normal durable host 재시작까지 연결한다.
- Consumer Verification Boundary: workspace 정상 Core application/loader·공개 DTO와 접근 가능한 normal host 저장/복원. 정상 테스트 조우를 별도 표시하고 미래 콘텐츠나 Unity 완료로 과장하지 않는다. helper-only 검증은 공개 경로 완료를 대신하지 않는다.
- Dependencies / Prerequisites: passed P2G40/50/60와 GEX00 recoverable baseline. U13 Complete/High/open0 및 U04/U12 유효 입력 인수. 단독 실행자이며 서브에이전트 없음.
- Inherited Baseline / Impact Map: 공통 재사용 계약과 CODE_REVIEW의 소비 경계를 따른다. 기존 완료노드/adaptive power 기대값은 최신계약으로 바꾸고 기존 실패를 숨기지 않는다. 나머지 지원경로 회귀는 원래 의미를 유지한다.
- Assigned Durable Integration Seam: PP-CEDR-001의10IVA 전부 WP-CEDR-VERIFY 소유. 정상 로더·명령·실제전투·결과·프로세스복원과 입력고정 비용측정으로31수용 조건을 확인한다.
- Migration/Compatibility/Rollout/Rollback: D01의 구개발저장 호환제외를 인수하여 새버전 명시거절/원본보호를 유지한다. 정상 새형식 Continue/SavePoint/결과/IO실패복원은 필수. source 동결의 정역 복원은 격리경로에서 확인한다.
- Known Unknown / Blocker / Stop: 미결정 의미 없음. Rust API/필드/버전·D02 authored 초기수치는 구현·검증에서 상세화한다. 실제시험·비용은 아직 미검증. 새 정책/필수근거 차단은 owner로 반환한다. 구현·자체검증·동결 후 감사요청 인계에서 종료하고 감사/U15 자동 착수하지 않는다.
- Risk / Reviewer: 입장 중복/유예소실·치료역전, 재진입/Continue 보상중복, 전투후 refill로 자격변경, 부활/봉인 중첩오류, 비대상수량배율, 실제성과 오집계·정보누출을 실제 호출과 원시증거로 자체검토한다. 정식 독립감사는 별도 요청과 비참여 역할을 요구한다.
- Structural Completion / Legacy and Fallback Disposition: 완료노드 유예·편성PW 적강화 의존을 제거한다. gold 적합안 부재를 개별 랜덤으로 숨기지 않는다. 기존 전투력평가·Tutorial·미지원 콘텐츠 거절은 유지한다. 동일 node 비용장부/보상권위 중복을 만들지 않는다.
- Canonical Documentation Obligation: core_runtime_contract/data_loading_contract와 관련 game_rules에 실제 API·작성필드·저장버전·지원수명·후속seam을 반영하고 과거 동결/감사 원문은 보존한다.

### Completion Matrix

| Acceptance | Requirement | Evidence Owner |
| --- | --- | --- |
| A01 | A01: 비용2/잔량1과 비용2/잔량0 모두 잔량0·규정 유예2이며, 비용0은 유예 무진행이다. 같은 노드 재입장/정비 반복은 중복 청구하지 않는다. | WP-CEDR-VERIFY / COVERAGE.json |
| A02 | A02: 공개 preview는 비용·예상 잔량·무억제 여부·직원별 예상 유예/사망을 실제 입장과 일치하게 표시하고 상태를 변경하지 않는다. | WP-CEDR-VERIFY / COVERAGE.json |
| A03 | A03: SavePoint/치료 노드도 비용과 사망을 먼저 확정하고 사망 직원을 그 노드 치료로 되살리지 않는다. 시작/Gate/방 이동은 무료다. | WP-CEDR-VERIFY / COVERAGE.json |
| A04 | A04: 새 유예4에서 비용1+2+1은 마지막 입장에서 사망한다. 완료/재입장/비용0/황금가지 보충으로 유예가 바뀌지 않는다. | WP-CEDR-VERIFY / COVERAGE.json |
| A05 | A05: 트라우마99 치료 뒤100 재진입은 새 유예4이고, 입장 사망은 장비/파편/유물 회수를 수행하며 전투 사망 방지로 취소되지 않는다. | WP-CEDR-VERIFY / COVERAGE.json |
| A06 | A06: 입장 사망으로 출전 불가이면 기존 런 실패/복구 자격을 적용한다. 잔량 부족·0에서도 같은 규정 비용으로 사망한다. | WP-CEDR-VERIFY / COVERAGE.json |
| A07 | A07: 깊이2/3 비용 소진 선택을 실제 공개 명령으로 구분한다. 깊이2 귀환은 거절, 계속은 해당 활동, 포기는 실패 정산이다. | WP-CEDR-VERIFY / COVERAGE.json |
| A08 | A08: 소진 귀환은 미시작 활동 보상·완료를 주지 않으며 기존 회수품의 정상 성공 귀환은 유지한다. | WP-CEDR-VERIFY / COVERAGE.json |
| A09 | A09: 이미0인 상태의 다음 입장/깊이3 도달만으로 선택을 다시 열지 않는다. 보충후 재소진은 새 선택이다. | WP-CEDR-VERIFY / COVERAGE.json |
| A10 | A10: 동일 seed/조우에서 편성·장비·최고전투력만 바꿔도 적 능력치/구성은 같다. 위험도/준비 정보는 유지할 수 있다. | WP-CEDR-VERIFY / COVERAGE.json |
| A11 | A11: 일반 적/엘리트/보스/부활의 생성 적과 지연 spawn에서 HP/ATK만 배율 적용되며 속도/사거리/저지/패턴은 불변이다. | WP-CEDR-VERIFY / COVERAGE.json |
| A12 | A12: 최초 비용 반영 후 전투 진입시 무억제 자격을 고정한다. 1.5배 부활×1.5배 무억제는2.25배이며 봉인 추가 스탯은 무억제와 중복하지 않는다. | WP-CEDR-VERIFY / COVERAGE.json |
| A13 | A13: 조우별 배율은 진입전 공개되고 작성 범위 밖/비정상 제작 입력을 로더가 거절한다. 전투마다 숨겨진 추첨은 없다. | WP-CEDR-VERIFY / COVERAGE.json |
| A14 | A14: 서로 다른 기본·추가·보너스 재료/XP의 합계가 정확히2배이고 실제 참가자/전체생존/선택직원 등 기존 XP 대상과1인당 수량을 유지한다. | WP-CEDR-VERIFY / COVERAGE.json |
| A15 | A15: 물품/알/파편/장비/유물/촉매제/선택횟수/황금가지/보충량과 비전투/판매/분쇄/반출은 배율 제외다. | WP-CEDR-VERIFY / COVERAGE.json |
| A16 | A16: 패배 기본 생존XP10, 후퇴0. 승리후 억제력 보충·결과재확인·Continue로 누락/중복배율·중복보상이 없다. | WP-CEDR-VERIFY / COVERAGE.json |
| A17 | A17: U16-4가 소비할 전투 전체1회 자격/배율 seam을 제공한다. U14는 스트레스 활동 전체 구현 완료를 주장하지 않는다. | WP-CEDR-VERIFY / COVERAGE.json |
| A18 | A18: 새 원정에서 plan을 선택하고 같은 원정의 재방문/후퇴/저장/Continue에서 plan과 노드 배정이 같다. 경로 선택에 따라 재배정하거나 개별 보스 코인추첨하지 않는다. | WP-CEDR-VERIFY / COVERAGE.json |
| A19 | A19: 적합 배치안 부재/잘못된 후보/도달불가/작성 누락을 거절하고 정상 테스트 조우로 배치·진입을 증명한다. 정식4종 본체 전체는 U21이다. | WP-CEDR-VERIFY / COVERAGE.json |
| A20 | A20: 미탐사 방에도 가지 위치·회수 여부·현재 경로 도달가능성을 표시하되 숨긴 방/보스 정체를 추가 노출하지 않는다. 모든 가지 회수 가능을 강제하지 않는다. | WP-CEDR-VERIFY / COVERAGE.json |
| A21 | A21: 억제/무억제 두 상태에서 봉인 진입→승리의 스탯과 수량보상을 확인한다. 해제만으로 가지/보충을 지급하지 않는다. | WP-CEDR-VERIFY / COVERAGE.json |
| A22 | A22: 후퇴는 봉인 미해제 상태·전투 손실을 복원하되 입장비용/유예/입장사망/후퇴횟수는 유지한다. 반복 승리/재접속 중복 지급을 막는다. | WP-CEDR-VERIFY / COVERAGE.json |
| A23 | A23: 패배/부활 전투는 황금가지/보충을 지급하지 않는다. 원래 시설 봉인승리의 보충으로 유예를 되돌리지 않는다. | WP-CEDR-VERIFY / COVERAGE.json |
| A24 | A24: 실제 참가 생존 직원에게 중복배치와 무관하게 기본XP1회, 미참가/Dead 제외. 승리 무억제만20, 패배10, 후퇴0이다. | WP-CEDR-VERIFY / COVERAGE.json |
| A25 | A25: 한번 지급으로 여러 레벨과 잔여XP를 보존한다. 일반 authored 성장 경로의 중반 목표를 관측하고 최종보스 플레이 검증은 제외로 기록한다. | WP-CEDR-VERIFY / COVERAGE.json |
| A26 | A26: 같은 방 Easy20/Standard50/Hard30 초기 혼합과 난이도별 재화/가루를 정상 콘텐츠로 확인한다. 편성전투력 연동은 없다. | WP-CEDR-VERIFY / COVERAGE.json |
| A27 | A27: ClearWithin 경계 이내/초과를 실제 전투시간으로, DecisiveDamage는 주환상체 최대HP 비율 경계/초과·다중타격·DOT 구별을 실제 action 결과로 검증한다. | WP-CEDR-VERIFY / COVERAGE.json |
| A28 | A28: 여러 번 조건 달성해도 목표당1회이며 패배/후퇴는 보너스0, 보너스XP는 모든 살아있는 직원 각자 전액·Dead제외·파편자격 불변이다. | WP-CEDR-VERIFY / COVERAGE.json |
| A29 | A29: 새프로세스 미확정 노드Continue는 비용/receipt/유예/입장사망/봉인을 입장전으로 복원하고 이미 사용한 후퇴횟수는 되돌리지 않는다. | WP-CEDR-VERIFY / COVERAGE.json |
| A30 | A30: 승패/노드 소비 확정 뒤 Continue/재확인은 보상/가지/비용을 반복하지 않는다. SavePoint는 비용·상태를 함께 과거로 복원한다. | WP-CEDR-VERIFY / COVERAGE.json |
| A31 | A31: 실제 지원 경로 IO실패/재시도에서 부분 비용/보상/가지 상태가 노출되지 않고 지원하지 않는 구저장 원본은 보호한다. 외부 임의변조 전용 방어는 범위 제외다. | WP-CEDR-VERIFY / COVERAGE.json |

### Validation Dimension Matrix

| Dimension | Disposition | Scenario | Evidence |
| --- | --- | --- | --- |
| success | required | normal authored cost→battle→victory/materialXP/gold/refill | PLAN/EXPERIMENTS actual nonzero outputs or exact scoped reuse, COVERAGE.json |
| invalid_input | required | invalid cost/plan/multiplier/objective and forbidden supported command rejection | PLAN/EXPERIMENTS actual nonzero outputs or exact scoped reuse, COVERAGE.json |
| failure_rollback | required | actual IO failure retry, all entry/reward/gold state atomic | PLAN/EXPERIMENTS actual nonzero outputs or exact scoped reuse, COVERAGE.json |
| retry_idempotency | required | same node reentry, retreat/Continue, duplicate result without recharge/regrant | PLAN/EXPERIMENTS actual nonzero outputs or exact scoped reuse, COVERAGE.json |
| ordering_concurrency | required | cost/death before healing/exhaustion/activity, snapshot before refill, existing accepted store ordering | PLAN/EXPERIMENTS actual nonzero outputs or exact scoped reuse, COVERAGE.json |
| boundary_empty_max | required | cost0/2 balance0/1 grace4, no suitable plan, time/damage exact threshold, supported finite upper input | PLAN/EXPERIMENTS actual nonzero outputs or exact scoped reuse, COVERAGE.json |
| disconnect_restart_persistence | required | pre-entry Continue, post-entry retreat, SavePoint/confirmed result in fresh process | PLAN/EXPERIMENTS actual nonzero outputs or exact scoped reuse, COVERAGE.json |
| live_data | required | official RON normal battles and supported authored normal test seal plans/bonus fixtures | PLAN/EXPERIMENTS actual nonzero outputs or exact scoped reuse, COVERAGE.json |
| producer_consumer | required | loader→public preview/command→actual combat facts→result→host persistence | PLAN/EXPERIMENTS actual nonzero outputs or exact scoped reuse, COVERAGE.json |
| legacy_fallback_absence | required | no completed-node grace/roster-adaptive enemy/gold fallback/extra reward whitelist; unsupported oldsave explicit | PLAN/EXPERIMENTS actual nonzero outputs or exact scoped reuse, COVERAGE.json |

## Policy–Phase–Goal Traceability

| Stable Element ID | Policy Element Revision Ref | Decision Revision Ref | Primary Phase ID | Single/Phase Goal ID |
| --- | --- | --- | --- | --- |
| PE-CEDR-G13 | PER-CEDR-G13-REVIEWED-001 | DR-CEDR-G13-001 | PHASE-CEDR-01 | GOAL-CEDR-01 |
| PE-CEDR-G14 | PER-CEDR-G14-REVIEWED-001 | DR-CEDR-G14-001 | PHASE-CEDR-01 | GOAL-CEDR-01 |
| PE-CEDR-G15 | PER-CEDR-G15-REVIEWED-001 | DR-CEDR-G15-001 | PHASE-CEDR-01 | GOAL-CEDR-01 |
| PE-CEDR-PWR01 | PER-CEDR-PWR01-REVIEWED-001 | DR-CEDR-PWR01-001 | PHASE-CEDR-01 | GOAL-CEDR-01 |
| PE-CEDR-PWR02 | PER-CEDR-PWR02-REVIEWED-001 | DR-CEDR-PWR02-001 | PHASE-CEDR-01 | GOAL-CEDR-01 |
| PE-CEDR-G16 | PER-CEDR-G16-REVIEWED-001 | DR-CEDR-G16-001 | PHASE-CEDR-01 | GOAL-CEDR-01 |
| PE-CEDR-G17 | PER-CEDR-G17-REVIEWED-001 | DR-CEDR-G17-001 | PHASE-CEDR-01 | GOAL-CEDR-01 |
| PE-CEDR-G18 | PER-CEDR-G18-REVIEWED-001 | DR-CEDR-G18-001 | PHASE-CEDR-01 | GOAL-CEDR-01 |
| PE-CEDR-G19 | PER-CEDR-G19-REVIEWED-001 | DR-CEDR-G19-001 | PHASE-CEDR-01 | GOAL-CEDR-01 |
| PE-CEDR-PERSISTENCE | PER-CEDR-PERSISTENCE-REVIEWED-001 | DR-CEDR-PERSISTENCE-001 | PHASE-CEDR-01 | GOAL-CEDR-01 |
