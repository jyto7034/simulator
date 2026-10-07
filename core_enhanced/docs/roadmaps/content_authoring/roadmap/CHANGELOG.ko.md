# v1.0 → v2.0 변경 기록

개정일: 2026-09-09. 원래 계획·첨부 파일은 보존하고 v2를 별도 생성했다.

| 항목 | v1 | v2 |
| --- | --- | --- |
| 문서 성격 | 목적·작업 순서·불변식·검증 중심 | P/R/L 구분 + 구체 권장 구조 + 코드 기반 상세 설계 절차 |
| 전체 구조 | ContentGeneration/최종 계획 개념 | 논리 모듈·K01–K10·AST/compiled/binding·도메인 포트와 대안 비교 |
| 효과 작성 | effect ID/property 필요 | EffectSite/ActionSite·carrier/scope 경계·result slot·descriptor 연결 |
| 상태·실행 | source/lifetime·chain 요구 | owner×lifetime·ledger/lease·ActionHandle milestone·receipts·fault 경계 |
| 유물 | selector·합성·휴면 목표 | grant closure·signed dependency·typed write/preserve set·충돌/캐시·runtime 분리 |
| 보스 | CA16의 작업·패턴 확장 목록 | 전용 A05, task 포트·milestone/cancel·typed blackboard·상태 전이·scenario signal·네 작업 묶음 |
| 예제 | 최소 구현 목표 | E01–E06 작성 의사자료→compile→owner/order→예상 결과, 미검증 표시 |
| 구현자 역할 | 기술 선택을 구현에 위임 | D1 종단 조사·D2 L 설계·DG·검증 환류, L0–L3 변경 권한 |
| 각 작업 카드 | 목표·참조·검증 | 위 내용 보존 + 선택/이유·조사 질문·상세 설계 산출·구현 묶음·추가 증거 |
| 순서 | 매개체→보스도 유물 연쇄에 간접 종속 | CA14의 CA13 의존 해제, CA15에 명시, 보스 설계 조기 착수 가능 |
| 인수 | 네 도메인 data-only | 보스의 신규 pattern/skill task·기믹 사건·실패/cancel 경로 명시 |
| 날짜·증거 | 계획 v1.0 2026-09-08 | v2.0 2026-09-09, 현재 최소 재확인과 과거 source 입구 구분 |

범위는 확대하지 않았다. 조합기 UI·외부 스크립트·서버/Unity 전환·전체 공식 밸런스는 여전히 비범위다. 완성형 상세 설계가 이미 검증됐다고 주장하지 않는다. P 정책을 반복 질문하지 않되 원문 미정 gameplay를 설계 예제로 확정하지 않는다.

## v2.0 → v2.1 — 순서의 증거 수준과 실행 인수 정리

- 권장 아키텍처와 CA00–CA23 목적·정책·검증은 보존했다.
- 모든 선행 표·24개 카드의 표기를 `계획상 선행 후보`로 변경했다. 후보 edge는 재배열하지 않았다.
- DEPENDENCIES에 필수 기능·동시 전환·검증 선행·진행상 권장, 코드 근거와 중간 상태 검사, 실제 Phase Plan과 GEX readiness의 구분을 추가했다.
- CA00의 주요 공유 경계 의존성 1차 확인과 다음 범위의 직접 전제 조사를 명시했다. 전체 상세 의존성 검증 완료를 요구·주장하지 않는다.
- CA01/04/10/16에 과잉 선행·부분 계약과 동시 전환 조사 질문을 추가했다.
- 상세 설계·준비 점검·인수·전환 문서를 위 기준으로 연결하고 현황 문서를 실제 첫 진입점으로 지정했다.
- GOALS.json은 설명용 metadata를 추가해 `dependencies`가 후보임을 표시한다. 기존 문서 체크 결과는 재사용하지 않고 이번 파일로 새로 검사한다.
- P2G r2·GEX 본문, 프로젝트 코드·RON·정식 정책/Goal은 변경하지 않았다.
