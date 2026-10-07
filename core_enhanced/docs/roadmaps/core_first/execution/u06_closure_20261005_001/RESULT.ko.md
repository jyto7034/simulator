# U06 완료 인수와 U07 시작 위치

2026-10-05 · GS-CBST-001 / GOAL-CBST-01 / PHASE-CBST-01 **Complete / High**.
[최종 독립 GEX-20](../../../../goals/core_battle_starter_tactics/post_audit/u06_20261005_002/COMPLETION_AUDIT.md)와
[감사 종료](../../../../goals/core_battle_starter_tactics/post_audit/u06_20261005_002/AUDIT_CLOSURE.json)가 판정 owner다.
단일 Phase 전체 Goal Set이 종료됐고 별도 GEX-30은 없다. 첫 감사의 Not Complete와 두 target 원본은 보존한다.

정상 RON의 여섯 시작 세트와 active 파편을 실제 grant/equip/채용에 연결했다. 기본 단일·범위 치유는
현재 회복 가능한 아군을 선택하고 실제 HP만 회복한다. 로스터10·동시배치8, 출전자격 우선,
철수·새 개체 재배치, 태세·이동·양쪽 원자 교대·즉시 작업·완료 뒤 Attack을 정상 명령으로 실행한다.
이동 뒤1500ms 강제 Defense, 연속 위치의 실제 저지 접촉, 읽기 전용 전술 미리보기와 거절 사유를 제공한다.
일반·배치·스킬 선택과 설정의 시간 복원은 공통 owner가 처리하며 Pause·disconnect·종료 우선순위를 유지한다.
기존 V11 저장은 기본 Damage의 canonical serialization 생략으로 checksum 호환성을 유지했다.

첫 감사의 U06-F01은 회복량0인 기본 Heal도 자동 행동을 시작하고 query에 표시하던 결함이었다.
공통 현재 회복량 계산을 자격과 방출에 적용했다. 0·SP0·정수 계산으로0인 단일/범위 입력은 idle이고,
양수 대조군은 정상 치유한다. 준비 뒤 능력치 변화·고정 대상·clone·실패 action 정리까지 검증했다.
zero 계수의 작성 자체는 유효하며 현재 상태에서 쓸모없는 행동을 시작하지 않는다는 정책을 유지한다.
비참여 GPT-6 Astra High의 fresh 재감사가 F01을 닫았으며 미해결 finding은0개다.

20 수락 조건·10 차원·141 선행 조건·별도31 R06 조건을 독립 판정했다.
현재 최종 Core1794/1794/1803, host92/100와 별도 heap1, fmt/check/event-log/clippy/release가 통과했다.
공개 CoreApplication8사례·원래 감사 probe 재현·strict 작성11사례를 포함한15개 새 실행과,
영향을 직접 검토한30개 기존 실행을 구분했다. 시작/종료에3,123 구성원과 제품 identity가 일치했다.
원본/현재/참조 보관본의 구성원과 정역 patch fuzz0 복원도 확인했다.

PC576의36조건(31성공/5typed거절), 실제 Continue/rewind·WebSocket·5모드 fault와 공개 비용 검증의
원래 source와 raw는 그대로다. RuntimeUnit bucket40bytes와 초기 spawn element80bytes 증가를 명시한
이전 비용 비교를 인수했고 이번 F01 교정에는 layout 변경이 없다. R05 raw exit101은 기존 승인된
concurrent peak 단독 경고(9,635,797 대9,030,188; +605,609bytes)이며 다른 hard guard 실패는 없다.
20분 stress 제외·계측 latency 제한을 유지했고 새 수치 상한/예외나 이번 성능 재측정을 주장하지 않는다.

재사용 설명의 정정: public_guards의 실제 측정은 정상 starter/SavePoint의 snapshot·checkpoint export·
event-log query·거절 staging이다. Little Red/dummy 전투는 final_performance의 경로다.
[독립 재사용 판정](../../../../goals/core_battle_starter_tactics/post_audit/u06_20261005_002/REUSE_JUDGMENTS.json)이
변경된 basic-Heal 자격/방출을 실행하지 않는 정확한 경로와 불변 입력/layout을 확인했다.
과거의 넓은 설명 대신 이 판단을 사용하며 동결 기록은 고치지 않는다.

이번 게시에서는 현황 두 문서의 진행 설명만 갱신한다. [변경 전 원문](DOCUMENTS_BEFORE.tar.gz)과
[전후 내용·비영향 검증](PUBLICATION_RECORD.json)이 이를 인수한다. 동결 PLAN·검증·정책·감사 원본은 보존한다.
Unity UI·배포 서비스·개별 최종보스 구현은 U06 완료 범위가 아니다.

다음 위치는 U07/B13·C01의 P2G-00→60이다. 독립 소환·설치·함정·장판·벽·진화·분절HP와
래티씨아 작은/큰 친구·얼음벽의 실제 소비, HP비용+생성 원자성을 현재 코드/정책에서 좁혀 인수한다.
선행161조건(기존141+U06의20)과 별도31 R06을 넘긴다. 전종 relation/infection은 U18,
보상/wave/도감 권한 승격과 개별 최종보스는 이 작업으로 추가하지 않는다.
원 연속 요청에 따라 정책·계획·구현·자체검증·비참여 독립 감사와 필요한 교정을 계속한다.
