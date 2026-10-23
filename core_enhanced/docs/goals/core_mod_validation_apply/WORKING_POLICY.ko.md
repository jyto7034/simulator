# U11 실험실 검증에서 실제 mod 적용까지 연결 — Working Policy

- Working Policy Document ID: WPD-CMVA-20261010
- Target Documentation Root: core_enhanced/docs
- Policy Directory Name: core_mod_validation_apply
- Current Physical Path: core_enhanced/docs/goals/core_mod_validation_apply/WORKING_POLICY.ko.md
- Current Working Revision Ref: WR-CMVA-003
- Policy Intake Snapshot Ref: PIS-CMVA-001
- Current Stage: P2G-60 complete; authorized GEX-00 intake
- Progression Mode: bounded_pipeline
- Requested Result: P2G-00→60 인계, 이후 기존 별도 사용자 연속 권한으로 GEX 구현·검증·독립 감사·후속 로드맵
- Target Project/Repository: simulator / /mnt/f/work/simulator

초기 immutable payload와 exact refs는 [P2G_00_INITIAL_001.json](P2G_00_INITIAL_001.json), 원문은 [INTAKE_SOURCE_001.tar.gz](INTAKE_SOURCE_001.tar.gz), 초기화 결과는 [P2G_00_RESULT_001.json](P2G_00_RESULT_001.json)에 있다. initial element/decision/question set은 비어 있으며 이 초기화는 정책 의미·현재 코드·구현 완료 판정이 아니다.

승인 범위는 ROADMAP U11/A07의 실험실 검증→실제 mod 적용, approval/cache·팩/조합·host generation·required mod 저장 보호다. 각 in/out과 직접 사용자/원문 승인 locator는 초기 Working Revision Scope Items에 연결했다. U10 target002/pass001 Complete/High/open0를 선행으로 인수했다.

현재 연속 요청은 통상 내부 선택과 정책 위임을 유지하고 단계별 재승인을 요구하지 않는다. 실측 기반 기본 한도·개발 설정 조정은 원실패와 요구 동작·원자성·자원 거부를 보존한다. 시험/측정 실행과 성능 합격 기준 확정은 구별한다. 사용자 선택이 필요한 새 material 정책은 해당 P2G owner에서 처리한다. Core 우선·Unity 후속·개별 최종보스 제외·20분 시험 제외를 유지한다.

Root Astra/high는 정책·설계·실제 코드 검토, 기존 Sol/xhigh는 계획된 제품 구현·준비된 실행, 독립 Astra/high는 시험/fixture/oracle 작성을 맡는다. 현재 제품·시험 작성 및 실행은 시작하지 않았다. U10의 source/target/audit와 이전 실패는 보존한다. 명시적 단계검토 중단 opt-in은 없으며 승인된 후속 단계로 계속한다.

## 정책 검토 완료

[P2G-10 정책/결정](P2G_10_POLICY_001.json)과 [결과](P2G_10_RESULT_001.json)에15개 요소·30개 acceptance 항목,18개 candidate 처리(승격15/범위외2/기술선택1)를 결속했다. 미결 사용자 질문은 없고 current WR002의 completion predicate는 true다. 사용자 확정 정책 적용과 Root의 통상적 위임 결정을 구별하며 없는 사용자 답변을 만들지 않았다.

D-05는 근거 있는 유한 Lab profile의 채택·판정 절차로 닫았다. 실제 숫자나 calibration 완료를 주장하지 않는다. 구현 단계에서 정상·확장 workload의 실제 시간/heap/native 측정, 반복·집계·환경·여유와 구체 기준, 경계 검증 및 정상 Lab→Apply를 완료해야 한다. 진단만 준비하거나 미교정 profile을 전달한 상태로 U11을 완료할 수 없다. 기존 직접 측정 위임으로 후속 수치 선택을 수행하며 새 gameplay/저장 의미가 필요하면 owner P2G로 반환한다.

독립 시험 담당자의 읽기 전용 검토를 반영하여 실제 trigger 부재, cache 무조건 miss, 개별 통과/조합 실패, 승인 뒤 파일 교체, 필요한 mod 복구 후 같은 원본 save 복원, U03 fault actor의 Apply 우회를 명시했다. 제품·시험 작성과 실행은 아직 없으며 다음은 현재 코드의 producer/consumer·상태·저장·실측 경계를 검토하는 P2G-20이다.

## 코드 기반 검토 완료

[P2G-20 검토](P2G_20_REVIEW_001.json)와 [코드 기반 설계](CODE_REVIEW.ko.md)는15개 정책/30개 수용 항목의 의미를 보존한 WR003, source plan002의6개 required/1개 N/A 경계,7개 evidence/observation과15개 result binding을 하나의 complete batch로 연결한다. 정적 loader와 실제 Trial/World·서버·저장·계측 source를 읽고5개 implementation finding 및7개 technical decision을 후속 작업에 인계했다. 현재 필수 evidence gap이나 새 material policy 질문은 없다.

mod grouping/정의별 coverage/교정된 Lab profile/issuer cache/실제 host Apply/저장 사전 보호는 아직 미구현이다. actual calibration과 concrete 기준, normal local/server 경로 및 독립 whole Goal 감사는 후속 GEX 필수이며 P2G 검토 완료를 기능 PASS로 표시하지 않는다. 다음은 P2G-30→60이다.

## 동결·Goal·인계 완료

[P2G30](P2G_30_RESULT_001.json)은 WR003/RB001의15개 exact reviewed member/30개 항목을 변경 없이 동결했다. [P2G40](P2G_40_RESULT_001.json)은 candidate/Lab/issuer/host/save5개 WP의 atomic admission과 PUBLIC/VERIFY를 하나의 phase/3node로 배정했다. 표준 생성·검증과 [P2G50](P2G_50_RESULT_001.json)의 quality contract2 구조 검사가 실제 exit0/failures[]다.20통합·10필수차원과 exact Goal GS-CMVA001/GOAL-CMVA01을 [P2G60](P2G_60_HANDOFF_001.json) HA-CMVA001로 인계했다.

Root는 actual 의미와 coverage를 검토했다. 제품/시험 구현·실행은0이며 구체 calibration 수치나 functional PASS를 주장하지 않는다. 기존 별도 연속 권한으로 GEX00의 recoverable baseline과 구현 전 상세 준비를 인수한 뒤 진행한다.
