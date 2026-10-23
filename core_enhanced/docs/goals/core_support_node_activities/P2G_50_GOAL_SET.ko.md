# U16-1 기존 지원 활동 — Single Goal Set

## Goal Set

- Goal Set Ref: GS-CSNA-001
- Goal Structure Fact: single_phase
- Phase Plan Ref: PP-CSNA-002
- Final Policy Freeze Ref: FPF-CSNA-001
- Complete Goal Member Set: GOAL-CSNA-01
- Goal Quality Contract Version: 2
- Authoring Status: Authored
- Activation Status: Not Activated
- Implementation Authorized: No

P2G 작성과 사용자 U16 착수 요청의 GEX 구현 권한을 구분한다. U16-1은 로드맵의 첫 분리 인수 묶음이며 U16-2/3/4는 후속 범위로 남는다.

### Phase-to-Goal Member Mapping

| Phase ID | Goal ID | Assigned Final Execution Nodes |
| --- | --- | --- |
| PHASE-CSNA-01 | GOAL-CSNA-01 | AG-CSNA-RUN, WP-CSNA-EVENT, WP-CSNA-CONSUMABLE, WP-CSNA-VERIFY |

### Shared Validation Baseline Reuse Contract

- Initial Full Baseline Owner / Timing: GOAL-CSNA-01은 U15 independent001 Complete/High/open0와 현재 target u15_current_001 및 core2180 고유통과의 조합 증거를 인수한다. 같은 실행자가 이어서 source를 변경하지 않았으므로 착수 자체로 전체 검사를 반복하지 않는다. GEX00에서 원본/dirty를 보존한다.
- Final Full Baseline Owner / Timing: GOAL-CSNA-01/WP-CSNA-VERIFY가18수용/10차원/5IVA 및 변경된 typed merchandise/loader/저장/결과/명령의 통합 회귀를 소유한다.
- Phase-Start Rule: exact 입력 또는 비영향 근거가 있는 기존 결과만 재사용한다. 새 기능 미실행은 미검증이다.
- Production Input Invalidation: 상품→loader/pack/가격/자격/구매/갱신/clone/저장/query/host; 소비품→효과/치료/전투결과/후퇴; HQ terms→조회/명령; 공통 grant/result→다른 획득과 공개 consumer.
- Test/Fixture/Probe Change: 목적과 기대값을 규칙에서 도출하고 원시 실패/과거동결을 보존한다. 실제 현재경로 입력을 과거 fixture로 대체하지 않는다.
- Shared Boundary Change: reader/writer/public serialization/normal host를 함께 검증한다.
- Normal Full-Validation Count: 초기 유효증거 재사용과 최종 영향통합1회. 실패/새변경/미해결이면 관련 경로 추가 검증.

### Integration Acceptance-to-Goal Mapping

| Acceptance ID | Disposition | Owner | Evidence |
| --- | --- | --- | --- |
| IVA-CSNA-G23 | assigned_execution_node | GOAL-CSNA-01 WP-CSNA-VERIFY | execution/single_goal/COVERAGE.json 해당요소와 원시실행 |
| IVA-CSNA-G24 | assigned_execution_node | GOAL-CSNA-01 WP-CSNA-VERIFY | execution/single_goal/COVERAGE.json 해당요소와 원시실행 |
| IVA-CSNA-G25 | assigned_execution_node | GOAL-CSNA-01 WP-CSNA-VERIFY | execution/single_goal/COVERAGE.json 해당요소와 원시실행 |
| IVA-CSNA-STORE | assigned_execution_node | GOAL-CSNA-01 WP-CSNA-VERIFY | execution/single_goal/COVERAGE.json 해당요소와 원시실행 |
| IVA-CSNA-AUTHOR | assigned_execution_node | GOAL-CSNA-01 WP-CSNA-VERIFY | execution/single_goal/COVERAGE.json 해당요소와 원시실행 |

## Single Goal Member `GOAL-CSNA-01`

- Goal ID: GOAL-CSNA-01
- Goal Kind: single
- Phase ID: PHASE-CSNA-01
- Assigned Final Execution Node Set: AG-CSNA-RUN, WP-CSNA-EVENT, WP-CSNA-CONSUMABLE, WP-CSNA-VERIFY
- Covered Policy Element Revision Ref Set: PER-CSNA-G23-REVIEWED-001, PER-CSNA-G24-REVIEWED-001, PER-CSNA-G25-REVIEWED-001, PER-CSNA-STORE-REVIEWED-001, PER-CSNA-AUTHOR-REVIEWED-001
- Covered Decision Revision Ref Set: DR-CSNA-G23-001, DR-CSNA-G24-001, DR-CSNA-G25-001, DR-CSNA-STORE-001, DR-CSNA-AUTHOR-001
- Derived Observable Policy Acceptance Coverage: FPF-CSNA-001 A01–A18 전부. Completion Matrix를 실제 evidence에 연결한다.
- Objective / Contract-Visible Result: 상점 유물/촉매제·유료 HQ 채용·Event·소비품을 정상 작성/명령/query/저장 경로에서 합의된 거래와 효과 수명으로 제공한다.
- User-Visible Problem: 상점 typed 상품에 유물/두 촉매가 없고 HQ 가격/횟수 안내가 부족하며 즉시치료 소비품이 기존 일반 효과를 유지한다. 기존 지원 활동의 실제 복원·거절·후퇴 검증이 필요하다.
- User-Visible Completion Condition Set: 상품/가격/자격과 실제 결과가 같고 구매/갱신/채용/판정/효과가 한번 적용된다. 재접속/종료복구/SavePoint/저장실패에서 비용만 남거나 사본만 복제되지 않는다.
- In-Scope / Out-of-Scope: U16-1 G23/G24/G25와 공유 저장/정상 제작/공개/영향 회귀. 일반 유물 매각·가루/파츠 거래·별도 자산엔진·인벤토리 상한 제외. U16-2/3/4 후속 분리 인수, U17 부화/U21고유도구/Unity/최종보스 제외. 전체 U16 완료를 주장하지 않는다.
- Reviewed Current Behavior / Delta: CRA-CSNA-001 CODE_REVIEW의 SHOP/GRANT/HQ/EVENT/CONSUMABLE/STORE/PUBLIC. 기존 권한을 재사용하고 누락된 상품/query/치료교체를 보완한다.
- Goal-Local Source of Truth / Owning Layer: validated shop data는 상품/가격, run shop session은 offer/페이지/구매, inventory/relic/crafting은 자산, recruitment는 후보/세션횟수, Event receipt는 판정, employee는 소비품효과, node/checkpoint는 완료/복원, paired host는 durable commit을 소유한다.
- Goal-Local Invariant Set / Source Refs: FPF-CSNA-001 A01–A18. 비용/결과원자·fixed offer/roll·no duplicate loadout·one modifier/count once·no UI capacity cap·unfinished rewind/completed preserve·typed catalyst separation.
- Boundary: RON→검증→고정 세션/상품→실제 payment/grant/choice/use→typed result/query→checkpoint/Continue/paired host.
- Consumer Verification Boundary: 정상 Core application/실제 승인 host의 명령/query/저장/새프로세스/실제IO실패. 내부 fixture만으로 전부 대체하지 않으며 Unity/배포 완료는 주장하지 않는다.
- Dependencies / Prerequisites: U04/U06/U08/U12/U13/U14/U15 현재 완료 근거, P2G40/50/60 및 GEX00 baseline. 단독수행/서브에이전트 없음.
- Inherited Baseline / Impact Map: 위 shared reuse와 CODE_REVIEW의 실제 도달 경로·전제. U15 원본/증거는 보존하고 새로운 상품/소비품 입력만 현재 테스트에 연결한다.
- Assigned Durable Integration Seam: PP-CSNA-002의5IVA 전부 WP-CSNA-VERIFY. 실제 지원노드/구매/채용/판정/소비품/복원/public host와 정상·확장비용을18수용에 연결한다.
- Migration/Compatibility/Rollout/Rollback: 현재 wire shape를 보존할 수 있으면 유지한다. 필수형식 변경이 필요하면 CF-D01에 따라 구개발저장 원본보호·명시미지원이며 조용한 재해석/삭제는 없다. 새지원 Continue/SavePoint/IO 필수. 최종 정역복원은 격리경로.
- Known Unknown / Blocker / Stop: 새 정책질문 없음. exact DTO/schema와 초기상품 콘텐츠 상세는 CODE_REVIEW 안에서 GEX가 결정. 실제측정/기능검증은 미수행. 구현·자체검증·target고정 뒤 감사요청 인계에서 종료하고 자동감사/후속묶음은 시작하지 않는다.
- Risk / Reviewer: 상품정체성과가격혼동·반복상품중복·paid grant 누락·촉매혼동·query자격불일치·중복기간차감·종료후부분복원을 실제 호출과 원시결과로 자체검토한다. 정식감사는 별도요청이다.
- Structural Completion / Legacy and Fallback Disposition: 기존 EconomyGrant/UUID/세션/receipt/paired publish를 재사용한다. 누락된 표시를 generic fake item으로 대체하거나 별도 자산권한을 만들지 않는다. 가루/파츠 범용 상품 fallback과 유물 일반 매각은 없음.
- Canonical Documentation Obligation: core_runtime_contract/data_loading_contract와 해당 실제 API/상품/기본공급/소비품 교체 계약을 갱신하고 과거 동결은 보존한다.

### Completion Matrix

| Acceptance | Requirement | Evidence Owner |
| --- | --- | --- |
| A01 | A01: 고정 상품의 사본/종류·수량·총가격·재고를 조회하며 구매는 비용/공통 grant/상품 소진을 한번 확정한다. 실패·부족·중복은 사본/UUID/재화/재고를 바꾸지 않고 빈자리를 자동 보충하지 않는다. | WP-CSNA-VERIFY / COVERAGE.json |
| A02 | A02: 갱신은 작성 비용/횟수로 페이지 전체를 교체하고 반복 불가 구매품을 제외한다. 후보 부족·통화 부족·실패 시 비용/횟수/페이지/상품이 유지된다. 매입 올림·매각 내림 및 차익 방지 계약을 유지한다. | WP-CSNA-VERIFY / COVERAGE.json |
| A03 | A03: 일반 유물 구매는 정상 상태의 개별 사본을 공통 grant로 지급하며 해당 종류의 기존 중복 자격을 따른다. 일반 상점 유물 매각, 장착/귀속 장비 판매, 가루/파츠 직접 구매판매, 0원 거래를 거절한다. | WP-CSNA-VERIFY / COVERAGE.json |
| A04 | A04: 부활 촉매제는 일반 상인의 기본 상품으로 제공하고 희귀 추첨으로 숨기지 않는다. 침식 촉매제는 별도 구매/보유 자원이다. 부활용은 비싸고 침식용은 과도한 비용을 피하는 초기 작성값을 두며 실제 부화는 U17이다. | WP-CSNA-VERIFY / COVERAGE.json |
| A05 | A05: 유료 채용은 가격에 기본 무기/파편 세트를 포함하고 기존 대여와 중복 지급하지 않는다. 1~4층·세션 최대2명·보유10명·부족/이미채용/비적격 거절과 후보 queue 순서를 유지한다. 자격 조회와 명령 결과가 일치한다. | WP-CSNA-VERIFY / COVERAGE.json |
| A06 | A06: 선택/직원 변경이나 조회/재접속/복원으로 d20이 달라지지 않는다. raw1 치명실패/raw20 치명성공이며 2~19는 합계>=DC, 허용 DC8/12/16/20과 적성+3을 적용한다. | WP-CSNA-VERIFY / COVERAGE.json |
| A07 | A07: 선택 직원의 장착/소유 장비 최고+1/+2 하나만 적용하고 동률은 사본ID 오름차순이다. 공용 장비 제외·열쇠는 비소비/무추가보정·잘못된 선택/직원/조건 거절 무변경을 증명한다. | WP-CSNA-VERIFY / COVERAGE.json |
| A08 | A08: 확정한 선택/직원/장비증거/결과를 반복해 재지급하지 않는다. 성공/실패의 다음 장면과 전투 분기의 승패/노드 소비는 한번 처리하며 후퇴는 기존 진입 전 복원이다. | WP-CSNA-VERIFY / COVERAGE.json |
| A09 | A09: 침식 Event는 대상 효과/패널티와 자격을 공개하고 촉매/통화/재료 없이 선택 사본만 변환한다. 거절·무효 대상·반복 요청 및 재시작에서 비용/사본 결과를 보존한다. | WP-CSNA-VERIFY / COVERAGE.json |
| A10 | A10: 지도/전투 진입 전 사용만 허용하고 직원당 활성 효과 하나를 새 효과로 즉시 교체한다. 성공 시 수량1 차감·환불 없음, 잘못된 대상/상태/재고 부족은 무변경이다. 즉시 치료도 기존 일반 활성 효과를 유지시키는 별도 예외를 추가하지 않는다. | WP-CSNA-VERIFY / COVERAGE.json |
| A11 | A11: 전투 결과 확정에 남은 기간을 한번 차감하고 결과 확인/중복명령에는 추가 차감하지 않는다. 비전투 완료/후퇴에는 차감하지 않으며 후퇴는 이미 사용한 전투 진입 전 효과/잔량을 복원한다. | WP-CSNA-VERIFY / COVERAGE.json |
| A12 | A12: 장비/소비품 보유가 50/60 표시 기준을 넘어도 정상 구매·보상 획득을 거절하지 않는다. 50부터10단위 표시 확장과 실제 화면은 U26-B이며 새로운 Core 보유 한도를 만들지 않는다. | WP-CSNA-VERIFY / COVERAGE.json |
| A13 | A13: 같은프로세스 재접속은 진행 중 offer/roll/선택/효과를 유지한다. 새프로세스 Continue는 미완료 노드 입장 전 거래·지불·채용·사본/원장을 함께 복원하며 노드 배정은 재추첨하지 않는다. | WP-CSNA-VERIFY / COVERAGE.json |
| A14 | A14: 완료한 상점/HQ/Event의 결과·소비를 새프로세스에서 유지하고 중복 완료/구매/채용/판정은 추가 지급하지 않는다. SavePoint는 당시 비용/결과를 함께 복원하되 무관한 영구기록/창고는 보존한다. | WP-CSNA-VERIFY / COVERAGE.json |
| A15 | A15: 실제 정상 host의 저장 IO실패는 메모리/active/profile의 부분 거래를 남기지 않고 재시도로 한 번만 반영한다. 지원 새 형식의 loader/writer·세대 입력·실제 소비자를 함께 검증하고 구개발 저장을 바꾸어 성공시키지 않는다. | WP-CSNA-VERIFY / COVERAGE.json |
| A16 | A16: 유물/촉매제 상품 및 기존 장비/소비품은 실제 RON/pack loader를 통과하여 공개 query/command에 나타난다. 누락 참조·가루/파츠 상품·0가격/0수량·overflow·불법 작성 조합을 로더에서 거절한다. | WP-CSNA-VERIFY / COVERAGE.json |
| A17 | A17: U13 실제 노드 배정·예약과 U14 최초 입장 비용·완료 의미를 유지한 대표 원정에서 상점/HQ/Event/소비품을 실행한다. 내부 helper fixture와 공개 host/fresh-process 검증을 구분하고 전체 U16 완료로 확대하지 않는다. | WP-CSNA-VERIFY / COVERAGE.json |
| A18 | A18: 정상/확장 상품·사본 workload의 시간/실제 메모리 peak/회수를 관측하고 기존 제한 조정 필요를 근거로 판단한다. 외부 임의 파일/메모리 변조 전용 시험은 제외하며 실제 IO장애와 지원 입력 오류는 검증한다. | WP-CSNA-VERIFY / COVERAGE.json |

### Validation Dimension Matrix

| Dimension | Disposition | Scenario | Evidence |
| --- | --- | --- | --- |
| success | required | 유물/두촉매 구매·HQ세트·Event·소비품 정상 활동 | PLAN/EXPERIMENTS raw nonzero results and COVERAGE.json |
| invalid_input | required | 금지상품/참조/가격/판매자격/선택직원/사용상태 거절 | PLAN/EXPERIMENTS raw nonzero results and COVERAGE.json |
| failure_rollback | required | 통화부족·갱신후보소진·실제IO실패 무부분변경 | PLAN/EXPERIMENTS raw nonzero results and COVERAGE.json |
| retry_idempotency | required | 같은offer/채용후보/판정/결과/완료 재입력 중복없음 | PLAN/EXPERIMENTS raw nonzero results and COVERAGE.json |
| ordering_concurrency | required | 결제/사본/세션/노드/paired publish 순서와 고정난수 | PLAN/EXPERIMENTS raw nonzero results and COVERAGE.json |
| boundary_empty_max | required | 0가격/수량·양수overflow·직원10/채용2·50/60초과 보유 | PLAN/EXPERIMENTS raw nonzero results and COVERAGE.json |
| disconnect_restart_persistence | required | fresh-process 미완료/완료Continue·SavePoint·현재재접속 | PLAN/EXPERIMENTS raw nonzero results and COVERAGE.json |
| live_data | required | 공식/외부RON 상품/직원/Event/소비품·잘못된 작성 | PLAN/EXPERIMENTS raw nonzero results and COVERAGE.json |
| producer_consumer | required | loader/public command/query/실제host 저장 일치 | PLAN/EXPERIMENTS raw nonzero results and COVERAGE.json |
| legacy_fallback_absence | required | 일반유물판매/가루파츠상품/부활희귀추첨/중복기본세트/묵시구저장재해석 없음 | PLAN/EXPERIMENTS raw nonzero results and COVERAGE.json |

## Policy–Phase–Goal Traceability

| Stable Element ID | Policy Element Revision Ref | Decision Revision Ref | Primary Phase ID | Single/Phase Goal ID |
| --- | --- | --- | --- | --- |
| PE-CSNA-G23 | PER-CSNA-G23-REVIEWED-001 | DR-CSNA-G23-001 | PHASE-CSNA-01 | GOAL-CSNA-01 |
| PE-CSNA-G24 | PER-CSNA-G24-REVIEWED-001 | DR-CSNA-G24-001 | PHASE-CSNA-01 | GOAL-CSNA-01 |
| PE-CSNA-G25 | PER-CSNA-G25-REVIEWED-001 | DR-CSNA-G25-001 | PHASE-CSNA-01 | GOAL-CSNA-01 |
| PE-CSNA-STORE | PER-CSNA-STORE-REVIEWED-001 | DR-CSNA-STORE-001 | PHASE-CSNA-01 | GOAL-CSNA-01 |
| PE-CSNA-AUTHOR | PER-CSNA-AUTHOR-REVIEWED-001 | DR-CSNA-AUTHOR-001 | PHASE-CSNA-01 | GOAL-CSNA-01 |
