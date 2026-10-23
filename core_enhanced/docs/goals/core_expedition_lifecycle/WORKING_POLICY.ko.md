# U12 튜토리얼·원정 시작·귀환과 개인 상태 수명 — Working Policy

- Working Policy Document ID: WPD-CEXL-20261012
- Target Documentation Root: core_enhanced/docs
- Policy Directory Name: core_expedition_lifecycle
- Current Physical Path: core_enhanced/docs/goals/core_expedition_lifecycle/WORKING_POLICY.ko.md
- Current Working Revision Ref: WR-CEXL-008
- Policy Intake Snapshot Ref: PIS-CEXL-002
- Current Stage: authorized GEX-10 implementation/independent validation
- P2G-10 Completion Predicate: true — 15 decided elements, 14 current decisions, no open question
- Progression Mode: bounded_pipeline
- Requested Result: P2G-00→60 인계, 이후 기존 별도 사용자 연속 권한으로 GEX 구현·검증·독립 감사와 다음 승인 로드맵
- Target Project/Repository: simulator / /mnt/f/work/simulator

[P2G-00 초기 원문/WR001](P2G_00_INITIAL_001.json), [미결 WR002/원 질문](P2G_10_OPEN_001.json)을 원형 보존한다. 제거 합의 refs는 [사용자 직접 결정·successor intake/WR003→004](P2G_10_DECISION_001.json)에 있고, 기본 정책은 [전체 정책 WR005](P2G_10_POLICY_001.json), 현재 refs는 [장비 범위 문구 교정 WR007](P2G_10_CORRECTION_001.json)과 [현재 P2G-10 종료](P2G_10_RESULT_002.json)에 있다. 최신 원문은 [INTAKE_SOURCE_002](INTAKE_SOURCE_002.tar.gz), 과거 원문은 INTAKE_SOURCE_001에 보존했다.

## 현재 정책과 범위

사용자가 기억·신뢰 시스템 완전 제거의 기존 합의를 재확인했다. 이전 기억 보존 해석과 OPQ-CEXL-001의 보존/생성 대안은 철회하며 현재 Open Policy Question set은 비어 있다. DR-CEXL-001을 선택한 PS01/PS02는 decided다. 기억·해당 최근 반응/사건·신뢰 상태/초기화/입력/조회/신규 저장과 복원 재생성을 제거한다. 고정 성격 두 축·condition·도감/탐사/귀환/획득 기록, 미지원 구저장 원본은 별도로 보호한다. [현재 룰북/로드맵 정정](../../roadmaps/core_first/execution/u12_psychology_removal_20261012_001/RECORD.ko.md).

초기 승인 I01–I04/I06 및 O01/O03은 그대로이며 I05/O02만 실제 답변에 따라 successor scope로 갱신했다. 튜토리얼/원정·6중3·10/8·기본세트/예산·채용큐·총2copy 대여·실제 귀환/실패·회복 가능성·첫귀환 story와 정상 저장/소비 계약까지 15요소를 decided로 닫았다. 세 제외 candidate와 기존 PS01/PS02를 포함한 disposition18개를 보존한다. P2G-10 종료 당시에는 formal code review·freeze·Goal이 미완료였다. 이후 P2G-20→60 및 GEX00을 완료했고 현재 제품 구현·검증은 아래 실행 PLAN이 소유한다.

## 현재 실행

현재 [PLAN](execution/single_goal/PLAN.md)과 [실행 기록](execution/single_goal/EXPERIMENTS.md)을 따른다. WR008/FPF-CEXL-001/GS-CEXL-001을 유지하며 GEX10 제품·독립 시험의 부분 변경이 존재한다. 로드맵 검토에 따른 일시 중지는 사용자 재개 지시로 해제했고 남은 구현·관련 검증을 계속한다. 문서·책임 보완은 정책 의미·38수용·19통합·10차원을 바꾸지 않는다. 기능 PASS와 U12 독립 완료는 아직 없으며 실제 증거는 검증 뒤 후속 인계에 연결한다.

## 진행과 역할 — 시점별 이력

아래 현재/아직/0 표기는 각 기록 당시의 상태다. 현재 위치는 위 실행 PLAN을 따른다.

정책 답변 대기는 해소됐다. 개정 연속 지시에 따라 P2G-20 실제 코드와 최소 동시 전환→후속 승인 단계를 계속한다. Root Astra high는 정책/계획·실제 코드/품질 검토, 기존 Sol6.1 xhigh는 인계 후 제품 및 준비된 검사의 단독 실행, 독립 Astra high는 시험/fixture/oracle 작성이다. U12 제품·시험 작성/실행은 아직 없다. 최종 감사는 비참여 Astra high를 최종 시점에 배정한다.

[독립 읽기 전용 사전 보고](READONLY_REVIEW_001.ko.md)는 당시 source 사실로 유지한다. 그 보고의 비어 있지 않은 기억 표현/merge/recent8 구현 제안은 최신 제거 결정의 요구가 아니다. 나머지 사실은 후속 정식 검토에서 currentness를 확인해 사용한다. 일반 기술 선택/오류 교정에 재승인을 요구하지 않는다.

정식 첫 코드 검토에서 PE07의 요약이 장비를 무기로 좁혔음을 발견해 기존 승인 장비 범위로 owner 교정했다. [미완료 첫 batch](P2G_20_ABORTED_001.json)는 보존했고 성공 결과/Binding으로 사용하지 않는다. 현재 고정 입력은 WR007/[RSP002](P2G_20_SOURCE_PLAN_002.json)이며 기존 원시 source 관찰은 현재 basis에서 재검토해 결속한다. 새 정책 선택이나 사용자 재승인 대기는 없다.

2026-10-12 재개: [한 번의 현재성 점검](RESUME_20261012_001.json)에서 U11 runtime manifest의 4,678파일이 현재 bytes와 모두 일치했다. 메인 `/root/core_main_astra`(Astra high), 제품/실행 `u12_product`(Sol6.1 xhigh), 독립 시험 `u12_tests`(Astra high)를 새로 배정했다. 현재는 고정 RSP002에 대한 읽기 검토이며 제품·시험 입력 writer와 실행은 없다. 프로세스 조회는 sandbox 내부만 보였으므로 OS 전체 실행 부재를 주장하지 않는다. 정책·코드 검토 후 남은 P2G와 승인된 GEX를 계속한다.

현재 formal 검토는 [P2G-20 RESULT002](P2G_20_RESULT_002.json)이며 입력 WR007에서 정책 의미·38수용 항목을 유지한 code_reviewed WR008을 발행했다. RQ01–08 confirmed, RQ09–11 N/A, scoped91 source의 원본/해시·보고서를 보관했다. [FPF-CEXL-001](P2G_30_FREEZE_001.json)은 15요소·14결정·15Binding을 정확 투영한다. 현재 [단일 Phase 계획](P2G_40_PHASE_PLAN_001.json)은 validator 실행 전 후보이며 runtime PASS/제품 완료를 주장하지 않는다. 앞의 시간순 기록은 당시 상태다.

[P2G-40](P2G_40_RESULT_001.json)의 구조/의미 검토와 [P2G-50](P2G_50_RESULT_001.json)의 quality2 검증을 완료했다. 현재 [HA-CEXL-001](P2G_60_HANDOFF_001.json) / GS-CEXL-001은15요소·38수용·19통합·10필수 차원을1single Goal에 결속한다. 기존 별도 사용자 연속 권한으로 GEX00 복원 기준 고정 후 바로 구현/독립 시험/검증/감사를 진행한다. 아직 U12 runtime 변경/검사는0이다.
