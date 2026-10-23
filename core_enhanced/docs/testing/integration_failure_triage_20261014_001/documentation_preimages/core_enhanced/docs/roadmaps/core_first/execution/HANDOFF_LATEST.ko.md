# Core 우선 로드맵 최신 인수인계

현재 위치: **사용자가 요청한 기존 Core 실패138개 교정 완료. 추가 통합17개 실패는 별도 잔여.** “응, 그러면 해당 테스트들을 모두 교정해줄 수 있어?” 요청에 따라 직전 진단의 유지·통합·부분 제거·환경 분리를 실제 적용했다. 단독 수행했으며 제품 실행 로직은 변경하지 않았다. 직전 진단 인계는 [원문032](HANDOFF_20261014_032.ko.md)에 보존했다. U15는 미착수다.

현재 근거는 [교정 보고서](../../../testing/core_baseline_failure_correction_20261014_001/REPORT.ko.md), [원138→현재139 대응표](../../../testing/core_baseline_failure_correction_20261014_001/CORRECTION_MATRIX.json), [요약](../../../testing/core_baseline_failure_correction_20261014_001/SUMMARY.json), [실행 색인](../../../testing/core_baseline_failure_correction_20261014_001/EXECUTION_INDEX.json)이다.

- 원138 전부 현재 대응 사례의 전체 assertion 통과. 유지135·시작 장비 중복통합1·정상 Continue 유지/외부 위조분기 제거1·socket 분리1을 적용했다. lib 총수2,103은 동일하다.
- 기본 lib: 전체016의2,092통과와 현재 G11 입력을 지정한019의4개 재실행, 허용 환경 socket017의1개를 합쳐 고유2,097통과·미해결0·기존 ignore6. 019는 비기본 배율110을 명시한 Continue 강화1개도 통과했다. 전체016 자체의4실패 원문은 보존한다.
- 계측판 G11 4/4·resource lifetime19/19 통과. 외부CLI12/12는 기본프로필6·기존 작성용개발프로필6으로 구분하며 U09 기본 한도 거절 원문을 보존했다.
- 추가 통합 검사8target은56통과/17실패다. 원138 밖의 Standard 구형시작16·구형 대검기대1을 [별도 목록](../../../testing/core_baseline_failure_correction_20261014_001/INTEGRATION_REMAINDER.json)에 남겼다. 이후 assertion은 미확인이다. 전체 저장소 green을 주장하지 않는다.
- 테스트/fixture35파일 변경. [최종 delta](../../../testing/core_baseline_failure_correction_20261014_001/final_focus_018.diff)와 최초 preimages를 보존했다. 전체 lib 이후 변경은 Continue 시험 강화와 들여쓰기이며 영향 시험만 재검증했다. 진행 세션 없음.

**다음 행동:** 이번 요청의 기존138 교정은 완료됐다. 후속 전체 테스트 정비의 후보는 별도 통합17개다. 모드 문자열만 일괄 변경해 통과로 처리하지 말고 현재 시작 선택/장비 자격·실제 시설 및 프로필/Continue 준비를 확인한 뒤 본래 oracle까지 검증해야 한다. 기존6ignore는 실제 승인 cache·별도 관리 조건을 필요로 하므로 일반 기능 PASS로 계산하지 않는다.

U14 과거 동결·독립 감사와 앞선 진단 기록은 수정하지 않았다. 현재 소스는 테스트 delta가 추가된 상태이며 당시 source713 해시와 동일하다고 주장하지 않는다. 이 일반 교정을 새 정식 Goal·U14 재감사·U15 착수로 해석하지 않는다. 기존 공식 고유보스/U16 전체활동/Unity/운영승인/최종SLO 등의 범위 한계는 유지한다.
