# U16-2 기본 유물 보상 — Single Goal Set

## Goal Set

- Goal Set Ref: GS-CBRR-001
- Goal Structure Fact: single_phase
- Phase Plan Ref: PP-CBRR-002
- Final Policy Freeze Ref: FPF-CBRR-001
- Complete Goal Member Set: GOAL-CBRR-01
- Goal Quality Contract Version: 2
- Authoring Status: Authored
- Activation Status: Not Activated
- Implementation Authorized: No

P2G 작성 권한과 사용자의 U16-2 착수 요청에 따른 GEX 구현 권한은 별개다. 구현/자체검증/최종 target 고정 뒤 감사 요청 대기에서 종료한다. 서브에이전트 없음.

### Phase-to-Goal Member Mapping

| Phase ID | Goal ID | Assigned Final Execution Nodes |
| --- | --- | --- |
| PHASE-CBRR-01 | GOAL-CBRR-01 | AG-CBRR-RUN, WP-CBRR-VERIFY |

### Shared Validation Baseline Reuse Contract

- Initial Full Baseline Owner / Timing: GOAL-CBRR-01은 U16-1 independent001 Complete/High/open0, target u16_1_current_001 및 Core고유2188의 조합증거를 인수한다. 감사 이후 제품변경 없음. GEX00에서 현재dirty/원본을 보존한다. 재개 자체로 전체검사를 반복하지 않는다.
- Final Full Baseline Owner / Timing: GOAL-CBRR-01/WP-CBRR-VERIFY가17수용/10차원/5IVA와 변경된 layout/세션/명령/query/공통 지급/노드완료/저장의 영향 통합 검증을 소유한다.
- Phase-Start Rule: exact 입력 또는 비영향 근거가 있는 기존결과만 재사용. 새 활동은 미실행이면 미검증.
- Production Input Invalidation: layout/pool→loader/validation/map/preview; reward session→clone/store/query/choice/claim/combat reward; command→metadata/capability/application/host; fusion/grant→inventory/copy/UUID/rollback/Continue.
- Test/Fixture/Probe Change: 원시 실패와 동결 기록 보존. 초기자원/노드경로/선택/중간전이는 규칙과 실제입력에서 도출하며 현재경로를 과거fixture로 대체하지 않는다.
- Shared Boundary Change: 실제 reader/writer/public serialization/host/영향회귀를 함께 검증.
- Normal Full-Validation Count: 초기 유효증거 재사용과 최종 영향통합1회. 실패/새변경/미해결에는 관련검사 추가.

### Integration Acceptance-to-Goal Mapping

| Acceptance ID | Disposition | Owner | Evidence |
| --- | --- | --- | --- |
| IVA-CBRR-CHOICE | assigned_execution_node | GOAL-CBRR-01 WP-CBRR-VERIFY | execution/single_goal/COVERAGE.json 요소별 원시실행 |
| IVA-CBRR-DUST | assigned_execution_node | GOAL-CBRR-01 WP-CBRR-VERIFY | execution/single_goal/COVERAGE.json 요소별 원시실행 |
| IVA-CBRR-FUSION | assigned_execution_node | GOAL-CBRR-01 WP-CBRR-VERIFY | execution/single_goal/COVERAGE.json 요소별 원시실행 |
| IVA-CBRR-STORE | assigned_execution_node | GOAL-CBRR-01 WP-CBRR-VERIFY | execution/single_goal/COVERAGE.json 요소별 원시실행 |
| IVA-CBRR-AUTHOR | assigned_execution_node | GOAL-CBRR-01 WP-CBRR-VERIFY | execution/single_goal/COVERAGE.json 요소별 원시실행 |

## Single Goal Member `GOAL-CBRR-01`

- Goal ID: GOAL-CBRR-01
- Goal Kind: single
- Phase ID: PHASE-CBRR-01
- Assigned Final Execution Node Set: AG-CBRR-RUN, WP-CBRR-VERIFY
- Covered Policy Element Revision Ref Set: PER-CBRR-CHOICE-REVIEWED-001, PER-CBRR-DUST-REVIEWED-001, PER-CBRR-FUSION-REVIEWED-001, PER-CBRR-STORE-REVIEWED-001, PER-CBRR-AUTHOR-REVIEWED-001
- Covered Decision Revision Ref Set: DR-CBRR-CHOICE-001, DR-CBRR-DUST-001, DR-CBRR-FUSION-001, DR-CBRR-STORE-001, DR-CBRR-AUTHOR-001
- Derived Observable Policy Acceptance Coverage: FPF-CBRR-001 A01–A17 전부. 아래 Completion Matrix를 실제 증거에 연결한다.
- Objective / Contract-Visible Result: 고정된 유물 노드에서 단순선택·장비가루투자·유물2→1합성을 정상 작성/명령/조회/저장 경로로 이용한다.
- User-Visible Problem: 지도에 활동이 배정되지만 모든 reward_activity 진입을 막고 실제 투자/합성 명령이 없다.
- User-Visible Completion Condition Set: 유형/조건/현재 후보/다음비용/선택확률을 공개하고 선택·투입·원자 지급·노드소비를 수행한다. 불가 시 이유와 기존권리를 보존하며 재접속/종료복구/SavePoint/IO에서 비용과 결과가 분리되지 않는다.
- In-Scope / Out-of-Scope: U16-2/G26 세 활동과 공유 작성/복원/공개 consumer/영향검증. U16-1 재구현, U16-3딜러/미니게임, U16-4스트레스전투, U17/U21 고유콘텐츠/Unity/최종보스와 외부 임의변조 제외. 전체 U16 완료를 주장하지 않는다.
- Reviewed Current Behavior / Delta: CRA-CBRR-001 CODE_REVIEW의 MAP/REWARD/FUSION/AUTHOR/STORE/PUBLIC. 기존 배정/보상세션/공통grant/사본/복원을 재사용하고 세 활동의 producer-consumer를 추가한다.
- Goal-Local Source of Truth / Owning Layer: validated layout policy가 후보/비용을, facility metadata가 배정 종류를, reward session이 공개후보/투자/선택을, RelicInventory와 crafting resources가 사본/장착/가루를, node/checkpoint가 완료/복원을, paired host가 durable publish를 소유한다.
- Goal-Local Invariant Set / Source Refs: FPF-CBRR-001 A01–A17. distinct max3; whole-threshold cost; current rights preserved on failed upgrade; six fixed probability rows; two distinct unequipped input copies; exclude input definitions; Normal output; once/node; atomic cost/result; unfinished rewind/completed preserve.
- Boundary: RON→검증→고정 활동/세션→선택·투자·합성→공통 grant/노드완료→query/Continue/SavePoint/paired host.
- Consumer Verification Boundary: 정상 CoreApplication/실제host의 명령·query·저장·새프로세스·실제IO. 내부fixture만으로 완료를 대체하지 않으며 Unity/배포를 주장하지 않는다.
- Dependencies / Prerequisites: U04/U08/U12/U13/U14/U15/U16-1 현재 완료근거와 P2G40/50/60 및 GEX00 baseline. 단독실행/서브에이전트 없음.
- Inherited Baseline / Impact Map: 위 shared reuse와 CODE_REVIEW. 이전 source/audit는 보존하고 새 활동/공유 reward 변경영향을 현재 검증한다.
- Assigned Durable Integration Seam: PP-CBRR-002의5IVA 전부 WP-CBRR-VERIFY. 정상입력/3활동/공통grant/query/복원/실제host/비용을17수용에 연결한다.
- Migration/Compatibility/Rollout/Rollback: optional/default 세션추가로 현재 형식을 보존한다. 실제 codec에서 불가능하면 CF-D01에 따른 구개발저장 원본보호·명시미지원; 조용한 변환/삭제 없음. 지원 Continue/SavePoint/IO는 필수. 최종 정역복원은 격리경로에서 수행한다.
- Known Unknown / Blocker / Stop: 미결정책 없음. 정확한 typed schema/command/helper와 테스트 입력은 GEX에서 정한다. 초기 임계치3/9/18·가중치1은 플레이 조절값이며 공식 유물catalog 공백은 외부pack으로 검증한다. 구현/자체검증/target고정 후 감사요청대기. 자동감사/다음묶음 없음.
- Risk / Reviewer: 미래후보노출·부족/빈후보의차감·옛등급수령·잘못된확률/입력순서·장착사본소비·중복지급·미완료비용만유지·IO부분적용을 실제호출과 원시결과로 검토한다. 정식감사는 별도요청.
- Structural Completion / Legacy and Fallback Disposition: 기존 grant/자산/세션/transaction/paired publish 재사용. 유물 활동만 지원하고 일반/전투 reward는 유지. 확률재분배·임의대체·재추첨·일반상점매각·보유한도 추가 없음. 별도 자산권한/무관리팩토링 없음.
- Canonical Documentation Obligation: core_runtime_contract/data_loading_contract와 실제 작성정책/command/query/복원계약을 갱신하고 과거동결은 보존한다.

### Completion Matrix

| Acceptance | Requirement | Evidence Owner |
| --- | --- | --- |
| A01 | A01: 지도 생성의 고정 유형/초반 단순 선택 예약을 유지하고 입장 전 유형·가루 요구량·합성 조건을 공개한다. U16-3/4는 실행 지원으로 표시하지 않는다. | WP-CBRR-VERIFY / COVERAGE.json |
| A02 | A02: 단순 선택은 서로 다른 최대3종 중1개를 Normal 사본으로 지급한다. 후보1/2개는 실제 수,0개는 이유 안내/선택 생략을 제공한다. 기존 보유/장착 제한을 후보 획득 제한으로 추가하지 않는다. | WP-CBRR-VERIFY / COVERAGE.json |
| A03 | A03: 재조회/선택 변경으로 후보를 재추첨하지 않고 미이용 퇴장은 보상/활동 비용 없이 노드를 소비한다. 최초 입장 억제력은 별도다. 중복 수령/퇴장은 추가 지급하지 않는다. | WP-CBRR-VERIFY / COVERAGE.json |
| A04 | A04: 일반 최초 투자와 각 승급은 다음 누적 임계치 차액 전부를 지불한다. 부족/부분투입/최고등급 추가 투입을 거절하며 자원과 후보는 유지한다. | WP-CBRR-VERIFY / COVERAGE.json |
| A05 | A05: 투입 전에 다음 등급 후보가 존재하는지 검사한다. 불가하면 비용/기존 후보/확보한 권리를 보존한다. 현재 등급 후보/효과와 다음 등급/차액을 공개하되 미래 후보는 공개하지 않는다. | WP-CBRR-VERIFY / COVERAGE.json |
| A06 | A06: 성공한 투입은 보장 등급의 서로 다른 최대3후보로 이전 후보/선택을 대체한다. 이전 등급 수령과 누적 지급을 막고 최종 선택1개만 공통 grant로 지급하며 노드를 소비한다. 활동 내 취소 환불은 없다. | WP-CBRR-VERIFY / COVERAGE.json |
| A07 | A07: 미장착의 서로 다른 사본2개를 선택한다. 침식 입력은 허용하고 장착/없는 사본/동일 사본은 거절한다. 투입 종류는 결과에서 제외하며 그 외 보유 종류는 허용한다. | WP-CBRR-VERIFY / COVERAGE.json |
| A08 | A08: 선택 후 확정 전 일반/희귀/특급 확률을 표시한다. 여섯 조합은80/19/1,40/55/5,0/70/30,0/80/20,0/40/60,0/0/100이며 입력 순서에 무관하고0% 등급은 결과가 되지 않는다. | WP-CBRR-VERIFY / COVERAGE.json |
| A09 | A09: 입력 종류 제외 뒤 확률이 양수인 모든 등급에 결과 후보가 있어야 확정 가능하다. 불가 이유를 공개하고 소비/선택/원본을 유지하며 확률 재분배/대체 지급/재추첨을 하지 않는다. | WP-CBRR-VERIFY / COVERAGE.json |
| A10 | A10: 확정은 두 입력 제거와 Normal 결과1개 공통 지급 및 노드 완료를 원자적으로 처리한다. 노드당1회이며 결과 재투입/중복확정/재접속 재지급을 막는다. | WP-CBRR-VERIFY / COVERAGE.json |
| A11 | A11: 같은 프로세스 재조회/재접속은 현재 활동 진행을 유지한다. 미완료 노드 새프로세스 Continue는 입장 전 비용/사본/진행을 함께 복원하고 재입장 시 같은 지도/동일입력 후보와 추첨을 유지한다. | WP-CBRR-VERIFY / COVERAGE.json |
| A12 | A12: 완료한 수령/합성/미이용 퇴장의 결과와 소비를 새프로세스에서 유지한다. SavePoint도 그 시점의 비용/결과를 함께 복원하며 무관한 영구기록/창고를 보존한다. | WP-CBRR-VERIFY / COVERAGE.json |
| A13 | A13: 실제 정상host 저장 IO실패 때 메모리/active/profile의 부분 변경을 남기지 않고 유효 재시도로 한번만 처리한다. reader/writer와 공개consumer를 함께 검증한다. | WP-CBRR-VERIFY / COVERAGE.json |
| A14 | A14: 정상 RON/pack으로 후보 종류·양수 가중치·증가하는 양수 가루 임계치를 작성하고 실행한다. 누락 참조/중복 종류/미지원 등급/0가중치/합계 overflow/잘못된 임계치를 로더에서 거절한다. 빈/부족 후보는 유효 입력이다. | WP-CBRR-VERIFY / COVERAGE.json |
| A15 | A15: 대표 원정의 U13배정/예약·U14입장비용·정상조회/명령·공통grant·노드완료를 검증한다. 공식 유물 catalog 공백과 외부 지원pack fixture를 구별하며 U21 콘텐츠나 전체 U16 완료를 주장하지 않는다. | WP-CBRR-VERIFY / COVERAGE.json |
| A16 | A16: 정상/확장 후보·사본의 시간/실제 메모리 peak/회수와 내부 작업량을 구분 관측하고 기존 유한 한도를 평가한다. 지원 입력 오류/실제IO는 범위 안이며 임의변조 전용 사례는 제외한다. | WP-CBRR-VERIFY / COVERAGE.json |
| A17 | A17: 기존 보상/상점/Event/소비품/저장 영향 회귀를 관련 호출 경계로 선정하고 실제 실행과 동일/비영향 재사용을 구별한다. 감사 전 복원 가능한 baseline과 최종 target을 고정한다. | WP-CBRR-VERIFY / COVERAGE.json |

### Validation Dimension Matrix

| Dimension | Disposition | Scenario | Evidence |
| --- | --- | --- | --- |
| success | required | 3활동 정상 선택/투자/합성/Normal결과/노드완료 | PLAN/EXPERIMENTS raw nonzero results and COVERAGE.json |
| invalid_input | required | 잘못된pool/weight/threshold·장착/중복/없는사본·옛후보거절 | PLAN/EXPERIMENTS raw nonzero results and COVERAGE.json |
| failure_rollback | required | 후보없음/가루부족/실제IO 무부분변경 | PLAN/EXPERIMENTS raw nonzero results and COVERAGE.json |
| retry_idempotency | required | 반복선택/확정/수령/퇴장 재지급없음 | PLAN/EXPERIMENTS raw nonzero results and COVERAGE.json |
| ordering_concurrency | required | 입력순서불변·고정난수·비용/사본/완료/저장 commit | PLAN/EXPERIMENTS raw nonzero results and COVERAGE.json |
| boundary_empty_max | required | 3/2/1/0후보·임계치전후/최고등급·0%배제·정수합계 | PLAN/EXPERIMENTS raw nonzero results and COVERAGE.json |
| disconnect_restart_persistence | required | 같은process 유지·fresh Continue완료/미완료·SavePoint | PLAN/EXPERIMENTS raw nonzero results and COVERAGE.json |
| live_data | required | 정상RON 외부pack의 후보/비용/예약과 작성오류 | PLAN/EXPERIMENTS raw nonzero results and COVERAGE.json |
| producer_consumer | required | 정상loader/application/query/실제host | PLAN/EXPERIMENTS raw nonzero results and COVERAGE.json |
| legacy_fallback_absence | required | 확률재분배/대체/재추첨/부분투입/이전등급지급없음; 기존일반/전투보상유지 | PLAN/EXPERIMENTS raw nonzero results and COVERAGE.json |

## Policy–Phase–Goal Traceability

| Stable Element ID | Policy Element Revision Ref | Decision Revision Ref | Primary Phase ID | Single/Phase Goal ID |
| --- | --- | --- | --- | --- |
| PE-CBRR-CHOICE | PER-CBRR-CHOICE-REVIEWED-001 | DR-CBRR-CHOICE-001 | PHASE-CBRR-01 | GOAL-CBRR-01 |
| PE-CBRR-DUST | PER-CBRR-DUST-REVIEWED-001 | DR-CBRR-DUST-001 | PHASE-CBRR-01 | GOAL-CBRR-01 |
| PE-CBRR-FUSION | PER-CBRR-FUSION-REVIEWED-001 | DR-CBRR-FUSION-001 | PHASE-CBRR-01 | GOAL-CBRR-01 |
| PE-CBRR-STORE | PER-CBRR-STORE-REVIEWED-001 | DR-CBRR-STORE-001 | PHASE-CBRR-01 | GOAL-CBRR-01 |
| PE-CBRR-AUTHOR | PER-CBRR-AUTHOR-REVIEWED-001 | DR-CBRR-AUTHOR-001 | PHASE-CBRR-01 | GOAL-CBRR-01 |
