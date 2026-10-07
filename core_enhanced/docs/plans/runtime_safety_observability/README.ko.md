# 실행 안전 설정·관측 — 검토 및 계획 인계

작성일: 2026-09-27 KST. 상태: **후속 설계 자료 보관 / 일반 RON·메트릭 계획 작성 / 실험실 상세 설계 미완료 / 제품 구현 미실행**.

> **로드맵 분리:** 확대된 모드 검증 범위의 내부 순서·선행·완료 조건은 [모드 검증 및 실행 안전성 편입안](../../roadmaps/mod_validation_safety/ROADMAP.ko.md)이 소유한다.
> 기존 S1/S2 설계는 MS01/MS02에서 재사용한다. 이 후속 작업을 선택할 때는 MS01의 현재 코드·host 전달 경계를 먼저 확인한다. 프로젝트의 현재 작업은 [단일 개발 현황](../../roadmaps/content_authoring/ROADMAP_STATUS.md)에서만 확인한다.

> 후속 실험실 상세 계획은 [1차 코드 검토·정책 결정](LAB_STRESS_REVIEW_001.ko.md)에서 진행 중이다.
> 실험실은 모드 제작 도구이며 실패 모드는 본편에 적용하지 않는다. 검증은 core 실행 주체가 맡고 저사양 예외는 두지 않는다.
> 실패 시 동작도 확정했다. 실패 조합 적용 보류, 필요한 모드가 제외된 세이브 로드 차단, runtime 안전 오류 시 해당 전투 중단을 따른다.
> 구체적인 복원·결과 반영 차단 경계는 상세 설계에서 확인한다. [정책 원본](LAB_STRESS_PROPOSAL.ko.md)을 참조한다.
> 일반 RON/메트릭 계획 완료와 실험실 상세 계획 완료를 구분한다.
> **사용자 후속 결정:** 20분 검증은 현재·후속 전체 단계, 자동 검증·등록·완료 조건에서 제외한다.
> 장기 시험은 기반이 마련된 뒤 사용자가 직접 수행하며 자동 후속 과제로 재추가하지 않는다. 긴 전투 지원 목표를 삭제하는 결정은 아니다.

사용자는 현재 대화에서 검토·계획을 수행하고 구현은 별도 대화에서 진행하도록 요청했다.
추천안 전체 채택 뒤 “검토 및 계획 수립을 진행해줘”를 이번 문서 작업의 근거로 삼았다.
별도 P2G 완료·Goal 활성화·제품 구현 승인·독립 감사 PASS를 발행하는 문서가 아니다.

## 결정과 읽는 순서

1. [코드 검토](REVIEW.ko.md): 현재 한도·오류·로그·관측 구조와 설명을 바로잡은 사항.
2. [설계 및 작업 계획](PLAN.ko.md): RON 형식, 적용 경계, 메트릭, 시험, 후속 기능의 순서.
3. [실험실 스트레스 검증 제안](LAB_STRESS_PROPOSAL.ko.md): 모드 단위 적용 거부·core 실행 주체별 검증·저사양 예외 없음은 확정. 시험 수치와 상세 절차는 설계 중이다.
4. [R06 비용 정책 변경](../../goals/content_authoring_value_expressions_reassessment/execution/single_goal/handoffs/R06_COST_POLICY_UPDATE_20260927.ko.md): 기존 concurrent peak 약 9MB 기준을 경고·관찰로 전환.
5. [문서 검증](evidence/VALIDATION.json): 제품/guard 무변경, 링크 및 원본 보존 확인. 제품 시험 결과가 아니다.

이번에 정한 방향:

- R06 필수 기능 범위에 설정·모니터링 전체를 추가하지 않고 **별도 후속 작업**으로 연결한다.
- 최초 구현은 기존 한도의 의미·숫자·오류 경계를 보존하며 trusted host RON으로 분리한다.
- 메트릭은 Rust `metrics`와 Prometheus exporter, 외부 Prometheus/Grafana를 재사용한다.
- 전체 집계와 선택한 전투 한 개의 상세 관측을 먼저 제공한다. 게임 내부 그래프·Tracy는 후속이다.
- 일반/집중 부하/기능별 스트레스/폭주 입력을 검증한다. 전투당 3MB 또는 새 평균 기반 한도는 확정하지 않는다.
- 사용자 후속 지시로 **안전 한도 초과 시 스킬을 자동 취소하는 기능은 계획에서 제거**했다. 기존 기절/퇴장 취소는 유지한다.
- 처리 분할은 필요성이 관측됐을 때 검토하는 후속 후보이며, 기존 한도 초과의 오류·복원 경계를 당장 바꾸지 않는다.
- 약 9MB의 작은 초과만으로 R06을 막지 않는다. 원 실패·raw·baseline·동결 검토는 보존한다.

## 인계 범위

별도 구현 대화의 첫 구현 후보는 PLAN의 **S1: 기존 한도의 RON 설정 분리(MS01)**다. 먼저 새 로드맵의 MS01 상세 계획에서 현재 경계와 지시서를 보완한다. S2 이후는 별도 범위로 인계한다.
R06의 다음 기능 설계/구현에 S1 완료를 선행 조건으로 걸지 않는다. 같은 파일을 바꾸는 구현은 동시에 진행하지 않는다.
후속 구현자는 현재 source와 이 계획의 [진입 기록](evidence/ENTRY.json)을 대조하고 변경된 R06 경계를 다시 확인한다.
확인 당시 D04-F01 동결 manifest와 333개 source의 현재성·양방향 복원은 [기록](evidence/CURRENTNESS.log)에 연결했다.
당시 source를 유지하라는 뜻은 아니며, 이후 R06 진전에 맞춰 구현 기준선을 새로 고정한다.

MS01 상세 계획 보완 후 사용할 S1 구현 요청 예시:

> 이 계획의 S1 범위인 기존 실행 한도의 trusted host RON 설정 분리를 구현해줘.
> 기본값·적용 범위·오류/rollback·parser 안전 상한을 보존하고 실제 host→core 전달 경로를 검증해줘.
> 메트릭 서버, 행동 취소, 처리 분할, 한도 상향은 이번 구현에 포함하지 마.
> 구현·자체 검증 뒤 before/after와 증거를 고정하고 독립 검토 인계에서 중단해줘.

## 보존 자료

- [편집 전 문서 archive](evidence/DOCUMENTS_BEFORE.tar.gz), [진입 hash](evidence/ENTRY.json).
- [문서 변경 patch](evidence/DOCUMENT_CHANGES.patch), [검증 코드](validate.py).
- 조사 과정에서 제품 빌드·부하 측정·설치·서비스 실행은 수행하지 않았다.
- 스킬 자동 취소 제거 전 계획/검증은 [원본 목록](revisions/skill_cancellation_removed_001/BEFORE.json)과
  [archive](revisions/skill_cancellation_removed_001/BEFORE.tar.gz)에 보존했다. 실험실의 사전 검증 방향은 제안으로 연결했다.
- 20분 시험 제외 및 플레이어 대상 확정 전 기록은 [원본 목록](revisions/long_duration_test_removed_001/BEFORE.json)과
  [archive](revisions/long_duration_test_removed_001/BEFORE.tar.gz)에 보존했다.
- 모드 적용 정책 정정 전 문서는 [원본 목록](revisions/mod_admission_policy_001/BEFORE.json)과
  [archive](revisions/mod_admission_policy_001/BEFORE.tar.gz)에 보존했다.
- 실패 시 동작 확정 전 문서는 [원본 목록](revisions/failure_behavior_policy_001/BEFORE.json)과
  [archive](revisions/failure_behavior_policy_001/BEFORE.tar.gz)에 보존했다.
