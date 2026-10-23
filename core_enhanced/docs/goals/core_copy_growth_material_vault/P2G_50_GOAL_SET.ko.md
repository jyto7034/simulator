# U15 사본 성장·실제 투자 환급·창고 재료 소비 — Single Goal Set

## Goal Set

- Goal Set Ref: GS-CGMV-001
- Goal Structure Fact: single_phase
- Phase Plan Ref: PP-CGMV-002
- Final Policy Freeze Ref: FPF-CGMV-001
- Complete Goal Member Set: GOAL-CGMV-01
- Goal Quality Contract Version: 2
- Authoring Status: Authored
- Activation Status: Not Activated
- Implementation Authorized: No

P2G 작성 상태와 사용자 U15 착수에 따른 GEX 권한을 구분한다.

### Phase-to-Goal Member Mapping

| Phase ID | Goal ID | Assigned Final Execution Nodes |
| --- | --- | --- |
| PHASE-CGMV-01 | GOAL-CGMV-01 | AG-CGMV-RUN, WP-CGMV-PUBLIC, WP-CGMV-VERIFY |

### Shared Validation Baseline Reuse Contract

- Initial Full Baseline Owner / Timing: GOAL-CGMV-01은 U14 independent002 Complete/High/open0와 20261014 core_full_verification_001의2169통과/기존ignore6을 인수한다. 현재 source를 보관하고 기존 dirty를 보존한다. 착수만으로 전체 검사를 반복하지 않는다.
- Final Full Baseline Owner / Timing: GOAL-CGMV-01/WP-CGMV-VERIFY가21수용/10차원/5IVA 및 변경된 shared schema/loader/저장/명령의 통합 회귀를 담당한다.
- Phase-Start Rule: 입력 동일성과 영향범위가 확인된 증거만 재사용. 새 기능 미실행은 미검증이다.
- Production Input Invalidation: copy→grant/장착/형태/분쇄/저장, recipe→loader/모든 fixture, warehouse→return/failure/checkpoint/Continue/host, public→command/query/consumer.
- Test/Fixture/Probe Change: 준비/기대값은 규칙 기반으로 바꾸고 실패 원문 보존. 과거 동결 fixture는 수정하지 않고 새 현재경로 입력과 목적을 구분한다.
- Shared Boundary Change: clone/codec/pack/normal host reader·writer·consumer까지 검증한다.
- Normal Full-Validation Count: 유효 초기 결과재사용과 최종 영향통합1회. 실패/새변경/미해결만 추가 검증.

### Integration Acceptance-to-Goal Mapping

| Acceptance ID | Disposition | Owner | Evidence |
| --- | --- | --- | --- |
| IVA-CGMV-G20 | assigned_execution_node | GOAL-CGMV-01 WP-CGMV-VERIFY | execution/single_goal/COVERAGE.json 해당요소·원시실행 |
| IVA-CGMV-G21 | assigned_execution_node | GOAL-CGMV-01 WP-CGMV-VERIFY | execution/single_goal/COVERAGE.json 해당요소·원시실행 |
| IVA-CGMV-G22 | assigned_execution_node | GOAL-CGMV-01 WP-CGMV-VERIFY | execution/single_goal/COVERAGE.json 해당요소·원시실행 |
| IVA-CGMV-STORE | assigned_execution_node | GOAL-CGMV-01 WP-CGMV-VERIFY | execution/single_goal/COVERAGE.json 해당요소·원시실행 |
| IVA-CGMV-AUTHOR | assigned_execution_node | GOAL-CGMV-01 WP-CGMV-VERIFY | execution/single_goal/COVERAGE.json 해당요소·원시실행 |

## Single Goal Member `GOAL-CGMV-01`

- Goal ID: GOAL-CGMV-01
- Goal Kind: single
- Phase ID: PHASE-CGMV-01
- Assigned Final Execution Node Set: AG-CGMV-RUN, WP-CGMV-PUBLIC, WP-CGMV-VERIFY
- Covered Policy Element Revision Ref Set: PER-CGMV-G20-REVIEWED-001, PER-CGMV-G21-REVIEWED-001, PER-CGMV-G22-REVIEWED-001, PER-CGMV-STORE-REVIEWED-001, PER-CGMV-AUTHOR-REVIEWED-001
- Covered Decision Revision Ref Set: DR-CGMV-G20-001, DR-CGMV-G21-001, DR-CGMV-G22-001, DR-CGMV-STORE-001, DR-CGMV-AUTHOR-001
- Derived Observable Policy Acceptance Coverage: FPF-CGMV-001 A01–A21 전부. Completion Matrix 원문을 실제 evidence에 연결한다.
- Objective / Contract-Visible Result: 무기/개화 사본별 실제 투자와 출처선택 지불·50%내림환급·재료선택회수·원정별창고소비 복원을 정상 RON/public/저장 경로로 제공한다.
- User-Visible Problem: 현재 실제투자/창고재료가 없고 장착장비 분쇄 확인도 없다. 현재 armor강화는 승인된 무기전용 규칙과 다르다.
- User-Visible Completion Condition Set: 혼합소비→개화→분쇄→복구/귀환/재대여를 거쳐 소비와 결과가 함께 이동한다. 미사용 창고는 보존되고 같은 투자로 중복 환급하지 않는다. query와 실제 결과가 같다.
- In-Scope / Out-of-Scope: U15 G20/G21/G22와 정상 제작/공개/저장·영향회귀. 방어구강화·투자재화환급·현재recipe정가환급·전체창고rollback·구저장임의삭제 제외. U17부화전체/U16활동전체/U21고유도구/Unity/최종보스 제외. 그 후속 공통결제 seam 검증은 포함.
- Reviewed Current Behavior / Delta: CRA-CGMV-001와 CODE_REVIEW의 COPY/RECIPE/MAINTENANCE/VAULT/RESTORE/PUBLIC/STORE/AUTHOR. 기존사본/장착/정상grant/paired host를 재사용하고 실제투자·명시출처·선택회수·차이복원을 추가한다.
- Goal-Local Source of Truth / Owning Layer: recipe/purpose는 validated data, paid materials는 copy, warehouse/debit-current는 profile.lifecycle, rewind debit는 run, 요청/결과는 application candidate, 저장은 paired host. 조회는 권위가 아니다.
- Goal-Local Invariant Set / Source Refs: FPF-CGMV-001 A01–A21. 실제지불만투자/재료별합산후내림/환급은소지/확인과소멸원자/소비출처차이복원/무관창고보존/실패미환급/무기reset·개화유지/선택회수2+1.
- Boundary: RON→검증→사본/명시출처→candidate 지불/성장/분쇄→query→새저장·SavePoint·Continue→귀환/실패·paired host 재시작.
- Consumer Verification Boundary: 정상 Core application/CLI 및 접근가능 host의 실제 명령/query/저장/새프로세스/IO실패. 일반 helper로 전부 대체하지 않는다. Unity와 배포환경 완료 주장 없음.
- Dependencies / Prerequisites: P2G40/50/60 passed와 GEX00 보관기준선, U14독립재감사002 및 현재전체core근거. 단독 수행/서브에이전트 없음.
- Inherited Baseline / Impact Map: 위 shared reuse 계약과 CODE_REVIEW. 구 방어구강화 fixture는 현재 무기정책 검증으로 목적보존 교정하고 과거동결 보존. 저장/사본/loader 공유변경의 실제회귀는 최종 실행한다.
- Assigned Durable Integration Seam: PP-CGMV-002의5IVA 전부 WP-CGMV-VERIFY. 실제혼합소비/성장/분쇄/복원/귀환/재대여/public host와 정상·확장비용을21수용에 연결한다.
- Migration/Compatibility/Rollout/Rollback: D01 구개발저장 변환제외·원본보호·명시미지원. 필수투자 누락은 조용히0으로 채우지 않는다. 새지원 저장·실패·재시작 필수. 최종 source 정역복원은 격리경로.
- Known Unknown / Blocker / Stop: 현재 새 정책질문 없음. 정확한 module/DTO/버전/D02 초기 수치 상세화·실제측정은 GEX. 새 정책/근거부족은 owner로 반환. 구현·자체검증·고정 뒤 감사요청 인계, 자동감사/U16 없음.
- Risk / Reviewer: 장착분쇄부분성공·투자중복·창고복제·실패환급·전체profile복원·구투자추정 위험을 실제호출/원시결과로 검토한다. 구현자의 자체검토와 별도 요청의 정식감사를 구분한다.
- Structural Completion / Legacy and Fallback Disposition: 방어구강화 지원/묵시재료출처/정가환급/전체창고rollback 경로 없음. 무료grant와 구저장미지원 구분. 기존 EconomyGrant/UUID/Loadout authority 재사용, 별도 자산엔진 없음.
- Canonical Documentation Obligation: core_runtime_contract/data_loading_contract 및 관련 game_rules에 실제 API/저장버전/정비·창고 수명/작성 포트를 반영하고 과거동결은 보존한다.

### Completion Matrix

| Acceptance | Requirement | Evidence Owner |
| --- | --- | --- |
| A01 | A01: 무기만 강화하고 방어구/장신구 강화는 작성/실행에서 거절한다. recipe 없는 작업은 불가이며 authored 한도 직전/도달에서 preview와 실행이 일치한다. | WP-CGMV-VERIFY / COVERAGE.json |
| A02 | A02: 중복 material은 종류별 checked 합산하며 소지/선택창고 총액과 엔케팔린 전부 충족할 때만 성장·소비가 함께 적용된다. 하나라도 부족하면 상태와 UUID를 바꾸지 않는다. | WP-CGMV-VERIFY / COVERAGE.json |
| A03 | A03: 개화 해금 뒤 형태 선택/직원 이전/같은 형태 재개화는 비용·투자를 추가하지 않는다. 무료 grant/조우 해금은 투자0을 유지한다. | WP-CGMV-VERIFY / COVERAGE.json |
| A04 | A04: 같은 정비 노드에서 여러 성장/분쇄를 해도 억제력/유예는 최초입장 한번이다. Safezone에 정비를 추가하지 않는다. | WP-CGMV-VERIFY / COVERAGE.json |
| A05 | A05: 가루/파츠 종류별 홀수 합산 뒤50%내림이며 현재 recipe를 변경해도 실제 투자 기반 환급은 같다. 개별 거래마다 내림해 합산하지 않는다. | WP-CGMV-VERIFY / COVERAGE.json |
| A06 | A06: 무료 강화/개화는 투자 환급0이며 형태/소유 직원 변경으로 투자 증가나 중복 환급이 없다. 사본별 투자와 보상을 격리한다. | WP-CGMV-VERIFY / COVERAGE.json |
| A07 | A07: 장착 장비/파편 분쇄는 확인 필요를 query에 표시하고 미확인 거절은 무변경이다. 확인 뒤 해제·사본소멸·기본보상·환급을 함께 적용하고 반복 요청은 보상을 주지 않는다. | WP-CGMV-VERIFY / COVERAGE.json |
| A08 | A08: 무기 반출 때 강화와 그 투자 모두 초기화한다. 개화 파편 귀환→재대여→분쇄는 동일 사본의 실제 파츠 투자만 한번 환급한다. | WP-CGMV-VERIFY / COVERAGE.json |
| A09 | A09: 소지+창고 혼합 개화에서 선택량 그대로 차감하고 다른 출처로 자동 대체하지 않는다. 중복행/부족/과다/잘못된 출처는 전체 무변경이다. 창고가 없어도 소지 전용 정상 경로는 유지한다. | WP-CGMV-VERIFY / COVERAGE.json |
| A10 | A10: 공개 조회는 요구량·소지/창고 보유량·선택 비용·투자·환급을 실제 결과와 일치하게 표시하며 조회는 상태를 바꾸지 않는다. | WP-CGMV-VERIFY / COVERAGE.json |
| A11 | A11: 성공 귀환 회수 선택은 파츠 종류합계0~2와 부활촉매0~1을 각각 검증한다. 가루/알/엔케팔린/침식촉매/일반소모품은 보관하지 않으며 장비/파편2사본 대여한도와 혼동하지 않는다. | WP-CGMV-VERIFY / COVERAGE.json |
| A12 | A12: 실패/포기는 소비한 창고재료를 돌려주지 않고 미사용 재료/미대여 사본은 보존한다. 소지 환급 파츠도 귀환 선택 없이는 창고에 넣지 않는다. | WP-CGMV-VERIFY / COVERAGE.json |
| A13 | A13: U17이 사용하는 부활촉매 소지/창고 결제 포트를 제공하되 실제 알 부화/조우 실행은 U17이다. 공통 결제 경계의 원자성과 조회는 U15에서 검증한다. | WP-CGMV-VERIFY / COVERAGE.json |
| A14 | A14: 소지+창고 혼합 개화→분쇄→SavePoint 복구는 원래 사본/형태/투자/소지/창고 소비를 함께 되돌린다. 체크포인트 이전 소비는 유지하고 반복복구로 증가하지 않는다. | WP-CGMV-VERIFY / COVERAGE.json |
| A15 | A15: 미확정 정비 노드 종료 뒤 새프로세스 Continue는 입장 전 소비/결과로 복원한다. 정비 완료 후 Continue는 확정 거래를 유지한다. 같은프로세스 호출로 새프로세스 검증을 대체하지 않는다. | WP-CGMV-VERIFY / COVERAGE.json |
| A16 | A16: 복원은 현재 원정의 소비 차이만 반영하며 무관한 창고 보관과 영구기록을 보존한다. U12 회수품 합집합 merge를 재료 수량 복원에 재사용하지 않는다. | WP-CGMV-VERIFY / COVERAGE.json |
| A17 | A17: 실제 저장 IO실패와 재시도에서 소비만/성장만 반영하지 않는다. active/profile 묶음의 저장·재시작을 확인한다. | WP-CGMV-VERIFY / COVERAGE.json |
| A18 | A18: 투자 내역 없는 구지원 전제는 D01에 따라 미지원 형식으로 명시 거절하고 원본을 보존한다. 실제투자 추정/자동0이행/임의삭제하지 않는다. 새 grant의 무료 투자0은 구저장 이행과 구별한다. | WP-CGMV-VERIFY / COVERAGE.json |
| A19 | A19: 정상 외부 RON의 복수 파츠/중복material/엔케팔린 비용은 preview와 실행이 같다. 누락 material/금지강화대상/잘못된 비용종류/합산 overflow는 정상 로더에서 거절한다. | WP-CGMV-VERIFY / COVERAGE.json |
| A20 | A20: 공개 Core application/host의 명령→조회→저장→재시작에서 실제사본과 원장·창고 결과를 확인한다. helper만의 결과를 공개 경로 완료로 주장하지 않는다. | WP-CGMV-VERIFY / COVERAGE.json |
| A21 | A21: 대표 정상·확장 입력의 실행시간/실제메모리 peak와 회수를 관측하고 내부 retained/work 회계와 구별한다. 기존 한도 변경 필요는 근거로 판단하며 임의증가하지 않는다. | WP-CGMV-VERIFY / COVERAGE.json |

### Validation Dimension Matrix

| Dimension | Disposition | Scenario | Evidence |
| --- | --- | --- | --- |
| success | required | 혼합개화→분쇄→귀환/대여 및 무기강화 | PLAN/EXPERIMENTS raw nonzero results and COVERAGE.json |
| invalid_input | required | 금지장비/recipe/material/source/미확인분쇄 거절 | PLAN/EXPERIMENTS raw nonzero results and COVERAGE.json |
| failure_rollback | required | 재료부족·환급overflow·실제IO실패 무부분변경 | PLAN/EXPERIMENTS raw nonzero results and COVERAGE.json |
| retry_idempotency | required | 재개화/형태이전/반복분쇄/복구 중복없음 | PLAN/EXPERIMENTS raw nonzero results and COVERAGE.json |
| ordering_concurrency | required | 지불·해제·투자·성장·환급 candidate와 paired publish 순서 | PLAN/EXPERIMENTS raw nonzero results and COVERAGE.json |
| boundary_empty_max | required | 합계홀수/0/회수2+1/상한초과/양수 overflow | PLAN/EXPERIMENTS raw nonzero results and COVERAGE.json |
| disconnect_restart_persistence | required | 새프로세스 미확정/완료Continue와SavePoint·재대여 | PLAN/EXPERIMENTS raw nonzero results and COVERAGE.json |
| live_data | required | 공식무기recipe와 외부다중파츠 정상RON/잘못된작성 | PLAN/EXPERIMENTS raw nonzero results and COVERAGE.json |
| producer_consumer | required | loader/public command/query/paired host 일치 | PLAN/EXPERIMENTS raw nonzero results and COVERAGE.json |
| legacy_fallback_absence | required | 미지원구저장보존/정가환급없음/창고전체복원없음 | PLAN/EXPERIMENTS raw nonzero results and COVERAGE.json |

## Policy–Phase–Goal Traceability

| Stable Element ID | Policy Element Revision Ref | Decision Revision Ref | Primary Phase ID | Single/Phase Goal ID |
| --- | --- | --- | --- | --- |
| PE-CGMV-G20 | PER-CGMV-G20-REVIEWED-001 | DR-CGMV-G20-001 | PHASE-CGMV-01 | GOAL-CGMV-01 |
| PE-CGMV-G21 | PER-CGMV-G21-REVIEWED-001 | DR-CGMV-G21-001 | PHASE-CGMV-01 | GOAL-CGMV-01 |
| PE-CGMV-G22 | PER-CGMV-G22-REVIEWED-001 | DR-CGMV-G22-001 | PHASE-CGMV-01 | GOAL-CGMV-01 |
| PE-CGMV-STORE | PER-CGMV-STORE-REVIEWED-001 | DR-CGMV-STORE-001 | PHASE-CGMV-01 | GOAL-CGMV-01 |
| PE-CGMV-AUTHOR | PER-CGMV-AUTHOR-REVIEWED-001 | DR-CGMV-AUTHOR-001 | PHASE-CGMV-01 | GOAL-CGMV-01 |
