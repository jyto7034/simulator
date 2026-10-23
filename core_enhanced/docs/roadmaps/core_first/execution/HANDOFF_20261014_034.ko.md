# Core 우선 로드맵 최신 인수인계

현재 위치: **추가 통합 실패17개 목적·원인·중복 분류 완료. 원본 교정·삭제는 미적용.** 사용자 “17개 실패를 방금 우리와 한 것같이, 유형을 분류해줄래?” 및 “목적뿐만 아니라 중복되는지도 잘 확인해줘.” 요청을 수행했다. 직전 기존138 교정 완료 인계는 [원문033](HANDOFF_20261014_033.ko.md)에 보존했다. U15는 미착수다.

현재 근거는 [17개 분류 보고서](../../../testing/integration_failure_triage_20261014_001/REPORT.ko.md), [개별 원인·고유 assertion·중복 후보 비교](../../../testing/integration_failure_triage_20261014_001/MATRIX.json), [원본 불변 확인](../../../testing/integration_failure_triage_20261014_001/INPUT_VERIFICATION.json)이다.

- 최초 실패: 폐기 Standard 준비16·시작 장비 기대1. 후속 정비 분류: 공통 시작 준비14·생명주기 복합 준비2·기대값/문서 정합화1.
- 권고: 유지·교정16, 고유 assertion 통합 후 기본 등록·장착 smoke 제거1. 무조건 삭제 가능한 완전 중복0. 후보와 입력·공개 경로·oracle를 비교했으며 이름만으로 중복 판정하지 않았다.
- 격리 최소 보정 후15개 전체 assertion 통과. 생명주기2는 성공 재시작의 금지 true 및 현재 지도에서 찾을 수 없는 records_annex 슬롯에서 중단했다. Continue 전에 실제 PlayerProfile 설치도 필요하며, 그 이후 assertion은 미검증이다.
- diagnostic001의 U08 content_acquisition은 진단용 resources symlink 구성 오류다. 실제 디렉터리로 복사한002에서 동일 바이너리를 재실행해14통과/2실패를 확인했다. 원 장비1은001에서 통과했다. 원본25시험/fixture 해시 불변·공식resource27복사 일치를 확인했다.
- containment_trainee는 현재 Tank shield/last_line이나 구 규칙표/시험은 greatsword/breakthrough다. 기대값과 구 문서의 정합화 항목으로 남긴다. hq_containment_aide는 Dealer greatsword로 별도다.
- 기존138 교정 완료·기본lib 고유2,097통과/기존ignore6·계측23통과·외부CLI12완주 이력은 유지한다. 통합 원본17이 해결되거나 전체 저장소가 green이라고 주장하지 않는다.

**다음 행동:** 실제 교정 요청이 오면 공통 시작 준비, 장비 기대/문서 정합화, 생명주기2의 현재 시설·재시작·프로필 준비를 교정하고 본래 assertion 전체를 검증한다. 기본 등록·장착1의 파일/pack/초기슬롯 검증을 공유 content와 성공 이전 테스트로 옮긴 뒤 독립 smoke를 제거한다. 이번 분류 자체를 실제 교정 완료로 취급하지 않는다.

제품 실행 로직·원본 시험·공식 데이터는 이번 분류에서 변경하지 않았다. U14 과거 동결·독립 감사와 이전 실패 기록을 보존했다. 정식 Goal·U14 재감사·U15 착수는 하지 않았으며 진행 중 실행 세션은 없다.
