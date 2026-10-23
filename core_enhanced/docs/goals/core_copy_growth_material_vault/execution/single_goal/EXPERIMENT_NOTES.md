# U15 판단

기존 G20/G21/G22·D01/USER11을 따르며 새 사용자 정책질문은 현재 없다. code review에서 기존 공식recipe가 armor만 강화함을 확인했으므로 무기전용으로 전환한다. D02의 authored 초기 수치는 완료된 밸런스가 아니다.

기반 개선은 장비분쇄의 live remove→grant를 staged candidate로 옮기고, 실제 공개경로에서 발견된 dust 단일잔액 및 transaction lifecycle rollback을 교정하는 범위다. 구현·runtime 자체검증은 완료했으며 정식 감사 판정은 아직 없다.

공개 persistence rollback token의 실제 소비 경계를 추가 확인: exactly-one unaccepted operation 뒤 run을 복원하면서 현행 lifecycle 전체를 유지하던 코드는 재료소비만 남길 수 있다. 원 transaction의 lifecycle/assets를 복원하고 monotonic knowledge만 merge하도록 교정한다. SavePoint/Continue는 계속 원정소비차이만 복원하며 이 transaction rollback과 구별한다. source: application.rs capture/restore_persistence_rollback; world/active_run.rs.

공개 소비 경로 선행조건: 3개 room, Event실제성공→SavePoint완료→Maintenance→depth3GateReturn으로 창고를 먼저 얻는다. 다음 원정은 혼합 bloom과 typed dust 강화 후 미완료종료, 다음 process는 entry복원, 완료 뒤 다음process는 투자유지, 귀환/재대여 환급을 별도로 확인했다. 정상 Event GrantEquipmentDust가 강화에 쓰이지 않는 실제 결함을 검출해 예약 id owner 통합으로 교정했다. profile 구버전도 실제 U14 저장 원본 사본으로 현재 host의 명시적 version5 거절과 원본불변을 확인했다.

환경사건: F드라이브932GB중잔여0으로 instrumented build001/비교baseline build001/worker복사에서 OS I/O5가 발생했다. 제품오류로 분류하지 않는다. 이번작업의 새 캐시와 대형 진단 바이너리 사본만 /tmp로 이동하고 원래 바이트를 gzip 보관했다. 사용자 파일/기존 캐시는 삭제하지 않았다. 정상host/worker완료와모든기능시험은보존한다. 비교비용은 양쪽동일 dev/debug0/incremental0으로 재빌드해 공간을 줄이며 앞선 debug정보포함빌드와 속도비율을 섞지 않는다. 현재측정결과는 후속 실행기록을 따른다.

현재 미해결 제품 오류나 정책 질문은 없다. 자원 한도는 유지한다(COST_REVIEW). 남은 필수 단계는 고정 target에 대한 별도 요청의 GEX20 완료 감사다. 이전 U14 verdict는 이전 target의 판정이며 이번 공유 계약 변경을 자동 승인하지 않는다. 2180 고유 Core coverage의 재사용/교정 범위와 기존 ignore6을 BASELINE_REUSE에 명시했다. U17 실제 부화, D02 최종 밸런스, 배포장치 성능SLO는 이번 완료 주장 밖이다.
