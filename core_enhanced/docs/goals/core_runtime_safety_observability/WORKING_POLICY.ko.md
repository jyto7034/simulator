# U02 실행 한도 설정과 제한된 비용의 런타임 관측

Document Identity: `WPD-CRSO-20261004` · Target Documentation Root: `core_enhanced/docs`.
Policy Directory: `core_runtime_safety_observability` · Current Working Revision: `WR-CRSO-005`.
정확한 Ref와 승인 in/out 범위는 [initial revision](P2G_00_INITIAL_001.json)이 소유한다.

현재 P2G-00–60 완료, `GOAL-CRSO-S1` GEX-10 구현·자체 검증 진행 중이다. 정책 요소 12개,
미결 질문 0개이며 P2G-40/P2G-50 validator가 통과한 Goal Set은 [인계](P2G_60_HANDOFF_001.json)에 고정했다.
[실행 PLAN](execution/s1/PLAN.md)과 [실제 명령 기록](execution/s1/EXPERIMENTS.md)이 현재 구현 상태를 소유한다.

사용자의 2026-10-04 연속 요청에 따라 CONFIG→CORE→PORTS→S1VERIFY를 진행한다.
U01은 독립 감사 Complete/High로 종료됐고, U02 S1 감사는 아직 시작하지 않았다.
S2 관측 구현은 S1 독립 감사 통과 후에만 시작한다. 단계별 재승인은 요구하지 않는다.

원문은 [입력 아카이브](INTAKE_SOURCE_001.tar.gz), 초기 검증은 [결과](P2G_00_RESULT_001.json)에 보존했다.
기존 계획을 재사용하되 실제 source 확인·정책 의미 판정·완료 기준은 각 후속 owner에서 검증한다.
중단 요청·승인 범위 완료·현재 권한과 환경에서 해결할 수 없는 실제 장애가 실행 종료 경계다.

정책 의미는 기존 계획을 최신 요청의 위임으로 채택했다. 결정마다 선택·이유·관측 가능한 영향·다섯 위임 조건을 기록했으며 사용자 직접 답변을 새로 만든 것으로 표시하지 않는다.

[코드 검토](CODE_REVIEW_001.ko.md) · [완전한 Review Batch](P2G_20_REVIEW_001.json) · [검증 결과](P2G_20_RESULT_001.json).

[코드 검토](CODE_REVIEW_001.ko.md) · [완전한 Review Batch](P2G_20_REVIEW_002.json) · [검증 결과](P2G_20_RESULT_002.json).

현재 P2G-60 complete, GS-CRSO-001/GOAL-CRSO-S1 GEX-00 complete → GEX-10 active.
[실행 PLAN](execution/s1/PLAN.md), [Handoff](P2G_60_HANDOFF_001.json). S2는 S1 감사 후 실행한다.
