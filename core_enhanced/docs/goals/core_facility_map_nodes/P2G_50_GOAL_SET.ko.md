# U13 고정 시설·노드·탐사와 브리핑 — Single Goal Set

## Goal Set

- Goal Set Ref: GS-CFMN-001
- Goal Structure Fact: single_phase
- Phase Plan Ref: PP-CFMN-001
- Final Policy Freeze Ref: FPF-CFMN-001
- Complete Goal Member Set: GOAL-CFMN-01
- Goal Quality Contract Version: 2
- Authoring Status: Authored
- Activation Status: Not Activated
- Implementation Authorized: No

P2G 작성 상태와 기존 사용자 구현 권한은 구분한다. 정식30수용 조건은 동결 원문 그대로 상속한다.

### Phase-to-Goal Member Mapping

| Phase ID | Goal ID | Assigned Final Execution Nodes |
| --- | --- | --- |
| PHASE-CFMN-01 | GOAL-CFMN-01 | AG-CFMN-ROOM, WP-CFMN-PUBLIC, WP-CFMN-VERIFY |

### Shared Validation Baseline Reuse Contract

- Initial Full Baseline Owner / Timing: GOAL-CFMN-01은 U12 post_audit/u12_scope_reassessment_001 Complete/High/open0와 U04/U11 비영향 증거를 인수한다. 제품 변경 전 recoverable baseline을 보관한다. 착수만으로 전체 시험을 반복하지 않는다.
- Final Full Baseline Owner / Timing: GOAL-CFMN-01/WP-CFMN-VERIFY가 최종30항목·10차원·10IVA와 실제 변경된 공유 경계의 회귀를 검증한다. 실제 명령·선택수·원시 결과를 PLAN/EXPERIMENTS에 고정한다.
- Phase-Start Rule: 동일 입력의 유효 증거를 재사용하고 runtime 미실행과 분리한다.
- Production Input Invalidation: layout/assignment→작성검증·배정·공개생성, navigation→Gate/비용/복원, profile→monotonic knowledge와 paired store를 재검증한다.
- Test/Fixture/Probe Change: 기대값은 정책·수작업 oracle에서 도출하고 변경된 증거만 재실행한다. 실패 원문을 보존한다.
- Shared Boundary Change: checkpoint/Continue/clone/pack/API 변경은 해당 producer와 consumer의 기존 회귀까지 검증한다.
- Normal Full-Validation Count: 초기 재사용 + 최종 영향 통합1회가 기본이며, 실제 실패/후속 입력 변경에 따른 필요한 추가 검사만 수행한다.

### Integration Acceptance-to-Goal Mapping

| Acceptance ID | Disposition | Owner | Evidence |
| --- | --- | --- | --- |
| IVA-CFMN-ALLOCATION-01 | assigned_execution_node | GOAL-CFMN-01 WP-CFMN-VERIFY | execution/single_goal/COVERAGE.json의 해당 요소와 원시 결과 |
| IVA-CFMN-ALLOCATION-02 | assigned_execution_node | GOAL-CFMN-01 WP-CFMN-VERIFY | execution/single_goal/COVERAGE.json의 해당 요소와 원시 결과 |
| IVA-CFMN-GRAPH | assigned_execution_node | GOAL-CFMN-01 WP-CFMN-VERIFY | execution/single_goal/COVERAGE.json의 해당 요소와 원시 결과 |
| IVA-CFMN-ASSIGNMENT | assigned_execution_node | GOAL-CFMN-01 WP-CFMN-VERIFY | execution/single_goal/COVERAGE.json의 해당 요소와 원시 결과 |
| IVA-CFMN-OMEN | assigned_execution_node | GOAL-CFMN-01 WP-CFMN-VERIFY | execution/single_goal/COVERAGE.json의 해당 요소와 원시 결과 |
| IVA-CFMN-NAVIGATION | assigned_execution_node | GOAL-CFMN-01 WP-CFMN-VERIFY | execution/single_goal/COVERAGE.json의 해당 요소와 원시 결과 |
| IVA-CFMN-KNOWLEDGE | assigned_execution_node | GOAL-CFMN-01 WP-CFMN-VERIFY | execution/single_goal/COVERAGE.json의 해당 요소와 원시 결과 |
| IVA-CFMN-SUPPORT | assigned_execution_node | GOAL-CFMN-01 WP-CFMN-VERIFY | execution/single_goal/COVERAGE.json의 해당 요소와 원시 결과 |
| IVA-CFMN-BRIEFING | assigned_execution_node | GOAL-CFMN-01 WP-CFMN-VERIFY | execution/single_goal/COVERAGE.json의 해당 요소와 원시 결과 |
| IVA-CFMN-PERSISTENCE | assigned_execution_node | GOAL-CFMN-01 WP-CFMN-VERIFY | execution/single_goal/COVERAGE.json의 해당 요소와 원시 결과 |

## Single Goal Member `GOAL-CFMN-01`

- Goal ID: GOAL-CFMN-01
- Goal Kind: single
- Phase ID: PHASE-CFMN-01
- Assigned Final Execution Node Set: AG-CFMN-ROOM, WP-CFMN-PUBLIC, WP-CFMN-VERIFY
- Covered Policy Element Revision Ref Set: PER-CFMN-ALLOCATION-01-REVIEWED-001, PER-CFMN-ALLOCATION-02-REVIEWED-001, PER-CFMN-GRAPH-REVIEWED-001, PER-CFMN-ASSIGNMENT-REVIEWED-001, PER-CFMN-OMEN-REVIEWED-001, PER-CFMN-NAVIGATION-REVIEWED-001, PER-CFMN-KNOWLEDGE-REVIEWED-001, PER-CFMN-SUPPORT-REVIEWED-001, PER-CFMN-BRIEFING-REVIEWED-001, PER-CFMN-PERSISTENCE-REVIEWED-001
- Covered Decision Revision Ref Set: DR-CFMN-001, DR-CFMN-002, DR-CFMN-003, DR-CFMN-004, DR-CFMN-005, DR-CFMN-006, DR-CFMN-007, DR-CFMN-008, DR-CFMN-009, DR-CFMN-010
- Derived Observable Policy Acceptance Coverage: FPF-CFMN-001 A01–A30 전부. 아래 Completion Matrix 원문은 추가/완화 없이 실행 증거로 연결한다.
- Objective / Contract-Visible Result: 실제 작성된 시설 graph를 정상 콘텐츠 로더로 읽고 방 선택·활동 배정·Gate 이동·재방문·공개 조회·영구 탐사·Continue/SavePoint를 동일 정체성으로 처리한다.
- User-Visible Problem: 현재 층별 새 map 교체는 같은 층의 복수 방과 되돌아가기, 지난 방 진행 보존, 전체 시설 지도를 표현하지 못한다.
- User-Visible Completion Condition Set: 정상 연결로 방에 진입하고 Gate로 동층 왕복/하강하며 기존 배정·완료·입장비용 기록이 유지된다. 미탐사 정보는 숨기고 탐사 지식은 실패 뒤에도 보존한다. 미구현 콘텐츠는 정확한 사유로 거절한다.
- In-Scope / Out-of-Scope: U13 S11/G07–G12, 최신89방/846활동·15동선 및 §1.8 확률, U12 계약·U20 전조 호환 경계를 포함한다. U14 이후 독립 기능, U16 전체 보상 실행, U20 실제 producer, 최종보스 고유 executor·Unity 렌더링·배포는 제외한다.
- Reviewed Current Behavior / Delta: CODE_REVIEW.ko.md의7경계 및 CRA-CFMN-001. 현재 floor 교체와 current-map-only reader에서 room identity 기반 저장·lookup·이동으로 전환하고 typed pack/clone/public consumer를 함께 바꾼다.
- Goal-Local Source of Truth / Owning Layer: 정적 시설/활동정책은 validated catalog, 현재/방문방 map·진행·배정은 run, 확정 탐사/도감은 profile, commit/rollback은 기존 prepared application 및 paired accepted store가 소유한다. DTO는 authority가 아니다.
- Goal-Local Invariant Set / Source Refs: FPF-CFMN-001 A01–A30. 한 현재방 단일 소유, 전체 node UUID 정체성, 재방문 재추첨/중복비용 없음, 좌표와 연결 분리, support 선추첨, 하강만 회복, 확정지식 monotonic, 숨은정보 비누설, 미지원 가짜완료 금지.
- Boundary: typed RON/pack→validator→generator→run room state→application command/query→checkpoint/Continue/profile→normal host 저장과 재시작까지 연결한다.
- Consumer Verification Boundary: 현재 workspace의 정상 Core application, 공식 pack loader, 기존 approved host/local/server 저장 경계와 public DTO를 검증한다. 실제 Unity/deployment는 범위 밖이며 내부 helper 시험만으로 전체 경로 완료를 주장하지 않는다.
- Dependencies / Prerequisites: passed P2G40/P2G50/P2G60와 GEX00 recoverable baseline. U12 현재 범위 재판정과 U04/U11 유효 계약 인수. 한 실행자가 순차 구현·시험·교정하며 서브에이전트 없음.
- Inherited Baseline / Impact Map: 공유 재사용 계약 및 CODE_REVIEW 경계를 따른다. node cache/receipt 참조검증은 모든 방문방으로, clone/native store는 신규상태로, public capability는 새 명령으로 영향 확대한다.
- Assigned Durable Integration Seam: PP-CFMN-001의10IVA를 WP-CFMN-VERIFY가 소유한다. 정상 로더→공개 명령→저장→복원과 실제 전투 preview/승리 도감을 소비 지점까지 검증한다.
- Migration/Compatibility/Rollout/Rollback: 지원 old writer는 명시 converter 또는 직렬화 호환 기본값으로 유지한다. 지원하지 않는 identity/version은 원본을 보존해 거절한다. prepared draft와 기존 store 원자성 유지, 저장된 방배정/해결/지식은 수명별 복원한다. 격리 경로에서 source 정역 복원을 검증한다.
- Known Unknown / Blocker / Stop: 현재 미결 정책 없음. 구체 Rust API/필드/version은 구현 선택이다. 실제 실행·비용은 미검증이며 새로운 의미 결정/필수 소비자 접근 불가는 owner로 반환한다. 구현·자체검증·target/복원을 마친 뒤 감사 요청에서 중지하며 U14와 감사에 자동 진입하지 않는다.
- Risk / Reviewer: 상태 유실/중복 node ID, 비용 재소비, profile 지식 되돌림, 숨은정보 leak, 지원분포 왜곡, 허위 지원을 실제 코드와 원시 증거로 자체 검토한다. 정식 감사는 별도 요청 경계이며 현재 실행을 감사 Complete로 표시하지 않는다.
- Structural Completion / Legacy and Fallback Disposition: 공식 고정graph 경로에서 floor replacement와 forced boss/omen fallback을 쓰지 않는다. 튜토리얼/명시 구형 profile의 유효 경로는 유지한다. 누락참조 무시·가짜보스·자동클리어·조용한 저장필드 유실은 허용하지 않는다.
- Canonical Documentation Obligation: core_runtime_contract/data_loading_contract와 run-and-map/relic-reward-nodes에 실제 API·작성필드·지원상태·공개조건·저장수명을 반영한다. 과거 동결/감사 원문을 바꾸지 않는다.

### Completion Matrix

| Acceptance | Requirement | Evidence Owner |
| --- | --- | --- |
| A01 | A01: 같은 제약의 후보에 추가 위치 가중치가 없고 seed와 입력이 같으면 동일 배정이다. | WP-CFMN-VERIFY / COVERAGE.json |
| A02 | A02: 좌표만 바뀌면 연결/도달성과 배정 의미가 유지된다. | WP-CFMN-VERIFY / COVERAGE.json |
| A03 | A03: 초반 예약은 단순 선택형이며 나머지는 다섯 유형이 양의 동일 비중이다. 매5개 각1개 보장/플레이어 직접 유형 선택을 추가하지 않는다. | WP-CFMN-VERIFY / COVERAGE.json |
| A04 | A04: 재방문/Continue/SavePoint 복원에서 유형과 원정 배정을 보존하고 진입 전 공개 조건 내 유형·이용 조건을 제공한다. 활동 executor 전체 구현은 U16이다. | WP-CFMN-VERIFY / COVERAGE.json |
| A05 | A05: 초기89방/846활동을 작성 입력과 대조하고 엔진의 고정 상한으로 하드코딩하지 않는다. | WP-CFMN-VERIFY / COVERAGE.json |
| A06 | A06: 동층은 작성된 양방향 연결·층간은 하강만 허용하며 모든7층은 지정8층과 연결된다. 심층 동간 횡단·흉탄의 A6-05외 직접 연결·상승을 거절한다. | WP-CFMN-VERIFY / COVERAGE.json |
| A07 | A07: 방당 한 계열을 유지하며 합류나 진입 방향으로 바뀌지 않는다. 표시 좌표/자산 ID는 전장 판정 좌표와 분리한다. | WP-CFMN-VERIFY / COVERAGE.json |
| A08 | A08: 중복/누락ID·끊어진 참조·Gate 미도달·잘못된 방향/예약 충돌을 정상 작성 오류로 거절한다. | WP-CFMN-VERIFY / COVERAGE.json |
| A09 | A09: 지원 총수에 고정 HQ/정비 포함 최대2, SavePoint/유물은 제외. 예약을 이중 집계하거나 삭제하지 않는다. | WP-CFMN-VERIFY / COVERAGE.json |
| A10 | A10: 지원 총수의 분포를 유지한 채 이벤트·추가보상·정예의 유효 개수 조합을 추첨하고 남는 곳은 일반전이다. 1~2층 일반전>=2,3~7층 치환전>=3/후>=2를 유지한다. | WP-CFMN-VERIFY / COVERAGE.json |
| A11 | A11:1층 정예0/2~7층 최대1, 작성 후보 제한과 고정 조우 우회·동층 거점 접근을 유지한다. 예약 포화로 분포를 실현할 수 없는 입력을 거절한다. | WP-CFMN-VERIFY / COVERAGE.json |
| A12 | A12: 일반전 난도20/50/30 정수배분을 소비하며 지원 종류 가중치30/45/25·예약별 지원 분포와 개수표는 §1.8 초기값을 따른다. 최종 밸런스 완료를 주장하지 않는다. | WP-CFMN-VERIFY / COVERAGE.json |
| A13 | A13:전조 없는 층/계열·후보없음·후보있음+자리있음/없음을 구분해 검증하고 불가능한 작성 입력은 거절한다. | WP-CFMN-VERIFY / COVERAGE.json |
| A14 | A14:호환 자리는 전조 확정 배정이 아니다. U20의 계열/층별 첫진입·미완료 후보·재방문/부화대체 소비 경계를 인계한다. 두건 선행 전 늑대 발생 금지는 유지한다. | WP-CFMN-VERIFY / COVERAGE.json |
| A15 | A15:실제 연결/도달 가능한 Gate를 통하여 목적 방으로 이동한다. 정보 미공개만으로 일반 방을 잠그지 않고 조회만으로 이동/배정/비용을 바꾸지 않는다. | WP-CFMN-VERIFY / COVERAGE.json |
| A16 | A16:동층 왕복/재방문 시 기존 노드 배정·해결·최초 입장 기록을 보존하며 방 사이 이동 추가 비용/중복 입장 비용이 없다. | WP-CFMN-VERIFY / COVERAGE.json |
| A17 | A17:하강 Gate에서만 생존 직원 현재 트라우마10% 올림 회복(Critical/Breakdown 포함/Dead 제외); 동층 이동·귀환 회복 없음. 장비/인벤토리/자원은 수명대로 보존한다. | WP-CFMN-VERIFY / COVERAGE.json |
| A18 | A18:Gate 도달에 추가 일반 보스 처치·노드 완료 수 조건이 없고 안전지대 허용 장착은 무료로 유지한다. U12 귀환/실패/첫입장 계약을 보존한다. | WP-CFMN-VERIFY / COVERAGE.json |
| A19 | A19:일반 위치·연결·방향은 처음부터, 인접 이름/컨셉은 조건 내 공개하고 미탐사 보스 정체/내부정보·숨은 연결·초상은 누설하지 않는다. | WP-CFMN-VERIFY / COVERAGE.json |
| A20 | A20:실제 Gate 떠남/깊이>=3 Gate 정상귀환에서 방완료·영구 지식 확정, Gate 조회/취소·소진 조기귀환·클리어 전 전멸은 신규 완료를 만들지 않는다. 이미 확정한 지식은 실패/Checkpoint 복구에서 유지한다. | WP-CFMN-VERIFY / COVERAGE.json |
| A21 | A21:실패 소비와 성공 개방을 구분하고 지난 원정 상품/보상/배정 정보를 새 원정에 공개하지 않는다. | WP-CFMN-VERIFY / COVERAGE.json |
| A22 | A22:승리의 실제 등장 적 profile만 도감 등록하고 미출현wave/소환/미리보기는 제외한다. 미인식 외부/기존 보스ID와 기존 영구 기록을 보존한다. | WP-CFMN-VERIFY / COVERAGE.json |
| A23 | A23:전체graph·연결·기존기록을 보존하고 유효참조와 실행지원 상태를 분리한다. 누락 참조 무시·가짜boss executor·자동귀환/클리어/보상을 만들지 않는다. | WP-CFMN-VERIFY / COVERAGE.json |
| A24 | A24:공개 전 미구현 표시로 숨은정보를 누설하지 않고 공개 후 지원상태/입장불가 사유를 제공한다. 자격 충족/미충족·직접/재전송·복원후 입장을 거절하고 비용/진행 무변경이다. | WP-CFMN-VERIFY / COVERAGE.json |
| A25 | A25:일반 활동 개발profile의 정상 로더/입장/결과/복원을 실행하고 제외된 공식 콘텐츠 사유를 명시한다. 구조 검증 성공을 정식 전체 플레이 완료로 집계하지 않는다. | WP-CFMN-VERIFY / COVERAGE.json |
| A26 | A26:실제 전장과 브리핑이 일치하고 배치/비용/입장 후 유예사망·자격 정보를 제공한다. 숨긴wave/미공개보상은 공개하지 않고 예상/확정 상태를 구분한다. | WP-CFMN-VERIFY / COVERAGE.json |
| A27 | A27:조회/미리보기 취소는 비용/진행 무변경이며 입장 전에 변경된 자원/직원 자격을 재검증한다. 보상 활동의 사전 조건을 공개하고 실제 실행 지원과 구분한다. | WP-CFMN-VERIFY / COVERAGE.json |
| A28 | A28:동일 원정 배정/현재방·방별 진행/최초입장/보상유형과 공개 상태를 새프로세스 Continue 및 SavePoint복원에서 보존한다. 새 원정은 새 가변상태, 영구 지식은 유지한다. | WP-CFMN-VERIFY / COVERAGE.json |
| A29 | A29:지원 구writer/미지원 저장·잘못된 지원 작성/명령·실제 IO실패에서 원본 보호와 실패 원자성을 검증한다. 임의 변조만 가능한 입력은 검증범위 밖이다. | WP-CFMN-VERIFY / COVERAGE.json |
| A30 | A30:공식89방/846활동 및 제한된 확장 입력에서 결정론·실시간 비용·메모리peak/회수·작업량을 구분해 측정하고 정상profile과 미지원/초과입력 거부를 구분한다. U12/U04/U11의 유효 비영향 증거를 재사용한다. | WP-CFMN-VERIFY / COVERAGE.json |

### Validation Dimension Matrix

| Dimension | Disposition | Scenario | Evidence |
| --- | --- | --- | --- |
| success | required | 正常 authored room→activity→Gate→same-floor/descent→query | PLAN/EXPERIMENTS commands, nonzero actual outcomes and raw evidence; exact claim in COVERAGE.json |
| invalid_input | required | broken authored refs/reservation conflict/wrong supported command; unchanged state | PLAN/EXPERIMENTS commands, nonzero actual outcomes and raw evidence; exact claim in COVERAGE.json |
| failure_rollback | required | actual prepare/store IO failure and retry; no partial room/knowledge commit | PLAN/EXPERIMENTS commands, nonzero actual outcomes and raw evidence; exact claim in COVERAGE.json |
| retry_idempotency | required | repeated query/entry/room revisit/Continue without reroll or debit | PLAN/EXPERIMENTS commands, nonzero actual outcomes and raw evidence; exact claim in COVERAGE.json |
| ordering_concurrency | required | existing paired CAS and gate resolution ordering; accepted ownership maintained | PLAN/EXPERIMENTS commands, nonzero actual outcomes and raw evidence; exact claim in COVERAGE.json |
| boundary_empty_max | required | empty/unreachable layout, support0/2, constrained rooms and host bounded expansion | PLAN/EXPERIMENTS commands, nonzero actual outcomes and raw evidence; exact claim in COVERAGE.json |
| disconnect_restart_persistence | required | fresh process Continue and SavePoint after multiple visited rooms and permanent completion | PLAN/EXPERIMENTS commands, nonzero actual outcomes and raw evidence; exact claim in COVERAGE.json |
| live_data | required | official89room/846activity catalog through loader; runnable development content separately | PLAN/EXPERIMENTS commands, nonzero actual outcomes and raw evidence; exact claim in COVERAGE.json |
| producer_consumer | required | authored identity→generator→query/actual battle→host store→restore | PLAN/EXPERIMENTS commands, nonzero actual outcomes and raw evidence; exact claim in COVERAGE.json |
| legacy_fallback_absence | required | fixedgraph excludes auto next-floor/forced omen; oldwriter and tutorial explicit compatibility | PLAN/EXPERIMENTS commands, nonzero actual outcomes and raw evidence; exact claim in COVERAGE.json |

## Policy–Phase–Goal Traceability

| Stable Element ID | Policy Element Revision Ref | Decision Revision Ref | Primary Phase ID | Single/Phase Goal ID |
| --- | --- | --- | --- | --- |
| PE-CFMN-ALLOCATION-01 | PER-CFMN-ALLOCATION-01-REVIEWED-001 | DR-CFMN-001 | PHASE-CFMN-01 | GOAL-CFMN-01 |
| PE-CFMN-ALLOCATION-02 | PER-CFMN-ALLOCATION-02-REVIEWED-001 | DR-CFMN-002 | PHASE-CFMN-01 | GOAL-CFMN-01 |
| PE-CFMN-GRAPH | PER-CFMN-GRAPH-REVIEWED-001 | DR-CFMN-003 | PHASE-CFMN-01 | GOAL-CFMN-01 |
| PE-CFMN-ASSIGNMENT | PER-CFMN-ASSIGNMENT-REVIEWED-001 | DR-CFMN-004 | PHASE-CFMN-01 | GOAL-CFMN-01 |
| PE-CFMN-OMEN | PER-CFMN-OMEN-REVIEWED-001 | DR-CFMN-005 | PHASE-CFMN-01 | GOAL-CFMN-01 |
| PE-CFMN-NAVIGATION | PER-CFMN-NAVIGATION-REVIEWED-001 | DR-CFMN-006 | PHASE-CFMN-01 | GOAL-CFMN-01 |
| PE-CFMN-KNOWLEDGE | PER-CFMN-KNOWLEDGE-REVIEWED-001 | DR-CFMN-007 | PHASE-CFMN-01 | GOAL-CFMN-01 |
| PE-CFMN-SUPPORT | PER-CFMN-SUPPORT-REVIEWED-001 | DR-CFMN-008 | PHASE-CFMN-01 | GOAL-CFMN-01 |
| PE-CFMN-BRIEFING | PER-CFMN-BRIEFING-REVIEWED-001 | DR-CFMN-009 | PHASE-CFMN-01 | GOAL-CFMN-01 |
| PE-CFMN-PERSISTENCE | PER-CFMN-PERSISTENCE-REVIEWED-001 | DR-CFMN-010 | PHASE-CFMN-01 | GOAL-CFMN-01 |
