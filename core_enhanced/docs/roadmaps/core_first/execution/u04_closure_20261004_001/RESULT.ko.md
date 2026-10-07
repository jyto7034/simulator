# U04 완료 인수와 U05 시작 위치

2026-10-04 · GS-CRRP-001 / GOAL-CRRP-01 **Complete / High**.
[최종 독립 GEX-20](../../../../goals/core_result_persistence/post_audit/u04_20261004_002/COMPLETION_AUDIT.md)와
[종료 currentness](../../../../goals/core_result_persistence/post_audit/u04_20261004_002/EXIT_CURRENTNESS.json)가 판정 owner다.
단일 Phase 전체 Goal 감사로 종료했으며 별도 Master GEX-30은 해당하지 않는다.

승패가 확정되는 transaction에서 직원 상태·소비품·무조건 보상·노드/조우·영구 기록을 정산한다.
결과 확인은 저장된 receipt를 소비하며 재정산하지 않는다. 고정 ChooseOne 후보와 원래 grant 문맥,
선택/수령 상태를 저장하고 중복·경쟁 수령을 거절한다. 후퇴는 성공2회까지이며 세 번째 도전에서는
불가하다. 미확정 노드의 종료 복구는 진입 전 상태와 성공 후퇴 사용량을 보존한다.

Core V10 Continue는 첫 SavePoint 이전을 포함한 최신 확정 상태와 결과/선택권을 복원한다.
명시 SavePoint load는 독립 snapshot으로 되감고 외부 monotonic load3·영구 profile·peak 수명을
유지한다. Host V5 generation/CAS·terminal tombstone과 paired application intent가 명령·자동 tick의
준비 root를 접수한 뒤 게시한다. 접수 전 실패는 폐기, 접수 후 실패는 같은 intent의 복구 대기다.
최신 확정 generation의 current/backup과 미지원 원본 보호, U03 first-fault·관측 수명을 함께 인수했다.
구형 V4/V9 및 미지원 profile/journal은 원본 보존·명시 거절하며 자동 호환을 주장하지 않는다.

첫 감사의 F01은 필수 host 프로토콜의 오래된 계약 설명이었다. 실제 DTO/명령·결과·복구·저장 계약으로
갱신하고, paired 모드와 기존 profile-only dirty/retry 모드를 구분해 재감사에서 닫았다.
첫 감사가 함께 제기한 ‘파일이 target에 없었다’는 주장은 원래 manifest와 실제 archive로 반증됐다.
그 사실 오류는 새 감사에서 명시적으로 정정했고 첫 감사·Not Complete 판정·중간 target 원본은 보존했다.

15 수락 조건·V01–V15·10 차원,112 선행 조건·별도31 R06 조건을 모두 독립 판정했다.
Core default/test-fixtures/observation1734/1734/1743, host89/97와 별도 heap1,
fmt/check/event_log/clippy/release, 공개 소비자16회와 PC576 non-timing 비교가 통과했다.
실제 stock WebSocket Continue/rewind78/114메시지와 각7회 process 시작, 다섯 fault 관측 모드,
저장 단계별 subprocess crash·경쟁 actor·동일 전투 identity 재실행을 검증했다.
문서 교정 후 제품 소스와 최종 실행40건이 동일함을 검증해 실행 증거를 재사용했다.

R05 raw101은 기존 승인 concurrent peak 단독 경고(+269,128 bytes)이며 다른 hard guard 실패는 없다.
할당 계측 시간값을 latency 판정으로 쓰지 않았고 기존 snapshot/export 한도도 유지했다.
OBS001과20분 시험 제외를 유지한다. 새 Unity UI·원격 운영·Windows crash·일반 power-loss 보장은 없다.

감사 종료 시2,891개 동결 구성원이 같았고 현재4개·외부8개 archive와 실제 fuzz0 정역 복원이 통과했다.
제품 SHA256은 `cb033469ed897c2bb38cdd5cd2bc4561b8b820acd0b680d45c5381abf4b3c629`다.
이 완료 게시는 현황 두 문서만 갱신하며 [변경 전 원문](DOCUMENTS_BEFORE.tar.gz)과
[전후 bytes·적용성](PUBLICATION_RECORD.json)에 실제 delta를 기록한다. 감사·실행 원본은 수정하지 않는다.

다음은 U05/B02·B03·B04·B09의 별도 P2G-00→60이다. 유효 입력의 공명 즉시 전량 소비와 접수 자격,
실제 사용 종료별 잠금·양수 획득 gate, 대상 유효성·취소·현재 침식 caller 호환을 함께 검토한다.
U01 취소와 U04 결과/복원, 현재127개 선행 조건(기존112+U04의15) 및 별도31 R06 조건을 인수한다.
침식의 새 피해 내용은 U19이며 이번 U05에서 미리 구현하지 않는다. 원 연속 요청에 따라
승인 범위 완료·사용자 중단·실제 해결 불가 장애까지 계속한다.
