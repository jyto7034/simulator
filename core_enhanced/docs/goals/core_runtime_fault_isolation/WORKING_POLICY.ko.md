# U03 런타임 오류의 명시적 종료와 세션·저장 보호

Document Identity: `WPD-CRFI-20261004` · Target Documentation Root: `core_enhanced/docs`.
Policy Directory: `core_runtime_fault_isolation` · Current Working Revision: `WR-CRFI-007`.
Exact Ref·승인 in/out 범위는 [initial revision](P2G_00_INITIAL_001.json)이 소유한다.

현재 P2G-40 successor 완료, 다음은 P2G-50 Single Goal 작성이다. 정책 요소·결정 각12개, 미결 질문0개다.
P2G-10 Completion Predicate: true. [정책·결정](P2G_10_POLICY_001.json)과 [검증·분류](P2G_10_RESULT_001.json)가 근거다. 제품 구현 완료를 뜻하지 않는다.

U02 S1/S2와 Master successor의 독립 Complete / High 종료 뒤 활성 U03/A06를 선택했다.
사용자의 연속 요청과 CF-D06 재량 위임을 인수하며 필요한 P2G/GEX·감사·교정을 계속한다.
원래 MS04 확정 정책은 실제 오류 전투만 중단하고 정상 결과/보상으로 처리하지 않는 것이다.
Core fault 생산→실제 application 실패/commit→host tick/재접속→진단/저장 gate의 경계를 조사한다.
정상 입력 거절과 전투 실행 실패의 분류, 복구·조회 수명은 P2G-10/20에서 근거를 남겨 결정한다.
U04의 정상 결과/SavePoint 전환이나 전체 Run rollback 보장을 이 입력으로 채택하지 않는다.

[입력 archive](INTAKE_SOURCE_001.tar.gz)와 [초기 검증](P2G_00_RESULT_001.json)을 보존한다.
기존 유효 시험·선행 100조건(U02 12+U01/R06/R07 88)의 적용성을 조건별로 확인하며
새 source 시험으로 과거 결과를 표시하지 않는다. 중단은 사용자 요청·승인 범위 완료·실제 해결 불가 장애뿐이다.

CF-D06 위임으로 선택·이유·관측 가능한 영향·다섯 조건을 요소별 기록했다. 새 사용자 직접 답변을 만들지 않았다. 안전 복구는 기존 명시적 Quit→새 owner→유효 checkpoint Continue 경로이며 오류 전투의 자동 재개나 저장 삭제가 아니다.

[코드 검토](CODE_REVIEW_001.ko.md) · [완전한 Review Batch](P2G_20_REVIEW_001.json) · [검증 결과](P2G_20_RESULT_001.json).

[Final Freeze](P2G_30_FREEZE_001.json) · [membership/검증](P2G_30_RESULT_001.json). 12개 의미와 수락 조건을 변경 없이 동결했다.

[형식 finding과 owner 교정](P2G_SCHEMA_CORRECTION_001.ko.md). Freeze001은 stale이며 후속 계획의 입력으로 사용하지 않는다.

[successor review/bindings](P2G_20_REVIEW_002.json) · [O20 resolution](P2G_20_RESULT_002.json). 기존12개 정책 원문과59개 source bytes는 그대로다.

현재 [Freeze002](P2G_30_FREEZE_002.json) · [완료/모든 O10·O20·O30 해소](P2G_30_RESULT_002.json). 과거 Freeze001의 bytes는 유지하고 current 선택은002다.

[단일 단계 설계](P2G_40_DESIGN_001.ko.md) · [표준 생성 receipt](P2G_40_RECEIPT_001.json) · [표준 검증 passed](P2G_40_VALIDATION_001.json).

[Decision identity 표준 형식 교정](P2G_SCHEMA_CORRECTION_002.ko.md). 이전 Freeze002/Plan001은 stale이며 원본을 보존했다.

현재 [review003](P2G_20_REVIEW_003.json)과 [O20 closure](P2G_20_RESULT_003.json)를 선택했다.

현재 [Freeze003](P2G_30_FREEZE_003.json), [O30 resolution](P2G_30_RESULT_003.json). Change002 O40만 남았다.

현재 [Plan002](P2G_40_PHASE_PLAN_002.json)·[표준 검증](P2G_40_VALIDATION_002.json)·[모든 Change 의무 해소](P2G_40_RESULT_002.json). Phase 의미·배정은 그대로다.

P2G-50/60 complete; [Handoff](P2G_60_HANDOFF_001.json), [GEX-00 completed PLAN](execution/single_goal/PLAN.md). Next GEX-10 under original continuous authority.
