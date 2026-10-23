# U14 successor 독립 재감사

사용자의 2026-10-14 U14 독립 재감사 요청에 따른 GEX-20 stage-run이다. 이번 대화 실행자는 U14 구현·교정에 참여하지 않았고 대상 제품을 편집하지 않는다. 위임/서브에이전트 없음. target002(source713/evidence600) 및 predecessor evidence980, 원 감사의 조건별 판단을 인수한다. 과거 원문과 동결 PLAN은 보존한다.

입구/출구에서 고정 소스·증거·HEAD를 확인한다. 정역복원 archive 목록/bytes/delta는 독립 검산하고 기존 filesystem 격리 복원은 재사용한다. 변경된 제품3파일, 회귀1파일, 문서·진행기 delta를 직접 검토한다. 31수용·10차원·10IVA의 영향을 추적하여 변경 경계는 새 증거와 교정 원시 결과, 무영향 계약은 원감사/유효한 시험을 연결한다. 임의변조 전용 사례는 finding으로 삼지 않는다.

F-U14-001: 두 방 동일 slot, B1-01에만 가지, 미회수 억제/무억제와 실제승리 회수 후 공개 node/activity preview, hidden identity, 조회 무변경, 왕복과 새 프로세스 Continue를 확인한다. 작성 배율180/170과 무억제150/150을 구별하며 배율을 기대값에 맞춰 바꾸지 않는다.

C-U14-002: 초기12에서 SavePoint 비용1, 보스 비용1, 실제승리 보충8로18, 정비/상점 각1 후16, 다음 방 SavePoint 이후15가 정상이다. 각 snapshot 자체의 collected 내역으로 공급 상한을 판정하는지 검토한다. 현재/과거 SavePoint의 서로 다른 내역, 수집 뒤 Continue/SavePoint 소비 경계를 확인한다. 공개 시나리오 준비는 기존 package/loader와 실제 전술접촉 진행기를 사용하며 저장/메모리/체크섬 변조는 없다.

새 검사는 기존 correction 공개3사례를 검토해 재사용하고, 누락 여부를 판정한 뒤 필요한 감사용 소비자만 별도 디렉터리에서 실행한다. 빌드는 직전 release80초를 근거로1–3분 예상,60초 진단,5분 초과시CPU/log 확인. 공개3사례는6초 내외. 기존 전체Core138실패는 baseline 비교 범위로 유지하며 전체 green을 주장하지 않는다. 완료판정 후 U15는 시작하지 않는다.

추가 감사용 공개 소비: correction 3사례를 현재 소스로 빌드한 audit-local binary에서 재현한다. 실제 회수 후 away(잔액16/이전 SavePoint11·미회수)와 returned(잔액13/회수후 SavePoint15·회수1)의 정상 export 각각을 새 프로세스로 인수한다. ViewingMap에서 공개 load_run_checkpoint를 호출하여 각각11/미회수0/원방 및15/회수1/옆방, load사용1, profile 불변을 확인한다. 이어서 다시 export→child Continue의 roots 일치를 검증한다. 이는 C-U14-002의 snapshot별 검증과 실제 SavePoint 소비를 검사한다. 기존 test/fixture의 요구조건·원문은 변경하지 않는다.

기록 생성 중 상대경로에 absolute relative_to를 적용한 Python ValueError 1회는 첫 출력파일 작성 전 발생했다. 저장소상대경로 그대로 기록하도록 교정했으며 제품/시험실패가 아니고 기능 재실행 사유로 사용하지 않았다.
