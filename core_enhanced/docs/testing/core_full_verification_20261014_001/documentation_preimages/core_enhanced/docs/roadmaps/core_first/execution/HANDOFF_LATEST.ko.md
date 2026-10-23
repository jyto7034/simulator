# Core 우선 로드맵 최신 인수인계

현재 위치: **추가 통합 실패17개 교정 완료. 현재16개 대응 테스트 모두 통과.** 사용자 “응, 그러면 위 테스트도 교정해줄래?” 요청을 수행했다. 직전 분류 인계는 [원문034](HANDOFF_20261014_034.ko.md)에 보존했다. U15는 미착수다.

현재 근거는 [교정 보고서](../../../testing/integration_failure_correction_20261014_001/REPORT.ko.md), [원17→현재16 대응표](../../../testing/integration_failure_correction_20261014_001/CORRECTION_MATRIX.json), [실행 색인](../../../testing/integration_failure_correction_20261014_001/EXECUTION_INDEX.json), [요약](../../../testing/integration_failure_correction_20261014_001/SUMMARY.json)이다.

- 공통 Standard/필수loan 입력을 현재 Expedition 계약에 맞췄다. 등록·장착 smoke1의 파일/pack·초기copy/owner/slot0 assertion을 공유 content와 성공이전 테스트로 옮기고 독립 smoke만 제거했다.
- Abandon 전 거절/후 성공 모두 false 시작으로 활성 런 경계를 검증하고 실제 authoritative profile을 설치한 새 앱 Continue를 확인했다.
- 미완료 이벤트는 현행6A template와 작은 현재 FacilityLayout으로 실제 map 선택/입장한다. 임시 타락/이력 되감기, 사전 유료비용 보존, 재진입·완료·복원을 모두 통과했다. 폐기된 구 시설fixture2파일을 제거했다.
- 시작 Tank containment_trainee shield/last_line 기대와 규칙표를 정합화했다. 영입 Dealer hq_containment_aide greatsword/breakthrough는 유지했다.
- correction002: ron_loading25/25 + u08_relic_public15/15 =40통과·실패/ignore/filter0. correction001의 작성 중 typed ID 빌드 오류 원문은 보존했다. 최종 입력 해시 동일·rustfmt check 통과. 진행 세션 없음.
- 다른 통합6target32는 비영향 이전 PASS 재사용, 통합 커버리지 합집합72다. 기존138 교정·lib2097/ignore6·계측23·CLI12 이력도 비영향 재사용하며 이번 전체 재실행으로 세지 않는다. G11 manifest 등 이전 실행 조건은 이전 보고서대로다.

**다음 행동:** 이번 테스트 교정 요청 범위의 미해결 실패는 없다. 다음 작업은 별도 사용자 요청을 따른다. 전체 통합을 같은 명령으로 재실행했다거나 모든 환경에서 전체 저장소 green이라고 확대하지 않는다.

테스트 소스4파일·구 fixture2제거·규칙 문서1파일을 변경했고 제품 실행 로직·공식 데이터는 변경하지 않았다. U14 동결·독립 감사와 기존 실패 원문을 보존했다. 정식 Goal·U14 재감사·U15 착수는 하지 않았다.
