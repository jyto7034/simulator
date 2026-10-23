# U16-1 — GEX-10 구현·자체검증

사용자 2026-10-15 U16 착수 요청과 상시 지시서에 따라 GS-CSNA-001/GOAL-CSNA-01/PHASE-CSNA-01을 수행한다. HA-CSNA-001/FPF-CSNA-001/PP-CSNA-002를 인수한다. P2G40/50 구조검증 통과. 현재 GEX r8과 상시 지시서를 신규 적용한다. 중간 단계검토 opt-in 없음. 구현·자체검증·복원 가능한 최종 target 고정 후 감사요청 인계에서 종료한다. U16-2/3/4와 정식감사는 후속 요청 경계로 유지한다. 한 실행자가 직접 수행하며 서브에이전트 없음.

U15 independent001 Complete/High/open0, target u15_current_001와 Core2180 고유 통과의 조합 증거를 인수한다. 같은 실행자에서 제품이 바뀌지 않았고 U16 P2G 문서만 추가했다. BASELINE_SOURCE_001과 git status는 현재 기존 dirty를 보존한다. 선행 전체 실행을 착수만으로 반복하지 않는다. 18수용/10차원/5IVA의 현재 증거는 새 구현 후 수집한다.

문서 역할: PLAN=계획/진행, EXPERIMENTS=명령·원시 결과·증거 색인, EXPERIMENT_NOTES=기술판단/실패 원인, STATE_INVENTORY=소유/수명. 상위 CODE_REVIEW의 경로 준비와 기대값을 따른다.

구현 순서: 상품 authoring/기본상품/가격·자격·공통 grant→구매 결과와 query·HQ terms→소비품 치료교체→실제 codec/Continue·검증/공식 콘텐츠→집중 테스트→정상 공개 host/저장/새프로세스/IO→공유 영향 회귀·비용→자체검토·최종고정. 상품/저장/공개 producer-consumer는 동일 atomic 묶음이다. 기존 Event/HQ/consumable owner는 변경 필요한 부분만 보완한다.

상세 설계 방향: typed merchandise의 새 variant는 relic ID, revival material ID, corruption catalyst를 구분한다. 가격은 정의에 없는 새 상품에만 명시 기준가를 작성하며 판매 가능 장비/소비품의 정가/차익 검사와 분리한다. identity에는 가격을 포함하지 않아 가격 차이로 반복 불가 구매를 우회하지 않는다. 일반상인 기본 부활촉매는 일반 후보 추첨과 분리하는 authored 기본항목으로 제공하고 이미 구매한 비반복 항목은 갱신에서 보충하지 않는다. 사본 유물은 수량1, 촉매는 양수 수량과 checked 총량이다. 물품 종류별 snapshot을 제공하고 일반 item_registry를 억지 확장하지 않는다. 지급은 기존 EconomyGrantState/RewardEffect, durable transaction은 기존 application/paired host를 사용한다.

HQ는 기존 가격100/세션2/층4/보유10을 유지하고 공개 terms로 가격/남은채용/기본세트포함을 알린다. 이미 지급된 기본 세트/대여 원장과 queue 소유는 기존 authority에서 검사한다. 일반 즉시치료 소비품은 치료와 이전 활성효과 제거·수량1을 같은 candidate에 처리한다. Event의 fixed roll/receipt는 재사용한다.

검증 준비: 정상 loader와 원정 선발로 직원/기본세트를 준비한다. 실제 U13 배정 지원노드에서 공개 offer/query로 ID·가격·자격을 얻어 행동한다. 부정 입력은 의도한 거절지점까지 전제를 만족하고 snapshot/파일 전후를 비교한다. Event raw1/20은 결정론적 seed별 실제 결과를 관측하고 판정표 독립 기대와 비교한다. 소비품은 구매→지도사용→전투후퇴/승패/중복결과로 각각 기간을 확인한다. 정상 fresh-process Continue와 같은프로세스 호출을 구분한다. 미완료/완료 경로는 각각 실제 store를 쓰고 IO는 유효 거래가 writer까지 도달한 증거를 필수로 둔다. 내부 단위 fixture는 공개 host 증거로 세지 않는다.

집중 검사는 기존 shop/event/equipment/HQ/결과·저장 시험과 새 수용 사례를 묶는다. 공유 상품 enum/DTO/loader 변경 때문에 최종 Core와 필요한 서버/외부 consumer 회귀를 수행한다. U15 기존 normal host runner/CMVA 준비도구를 우선 재사용하고 새 제품에 맞는 approval을 생성한다. 현재 CARGO_HOME=/tmp/wp02-cargo-home, target=.cache/u12-vault-cost-target; offline locked. 최초 check/build 예상1~3분,60초 무진전 진단/5분이면 CPU·로그·메모리 확인. 검사 입력은 실행 중 고정하고 실패 원문 보존,0시험/빌드성공을 기능PASS로 기록하지 않는다. 실행 후 새 변경은 영향 범위만 재검증한다.

정상/확장 상품·사본 workload의 시간/실제메모리 peak/회수를 측정한다. 내부 work를 실제 메모리로 보고하지 않는다. 한도는 기존 의미를 유지하고 측정 없이 자동 상향하지 않는다. 외부 임의변조 전용 방어 제외, 지원 제작오류/명령오용/실제IO는 검증한다. 현재 디스크 F 여유248GB, /tmp6.2GB로 빌드 캐시의 추가 /tmp 이동은 피한다.


## 구현·자체검증 종료와 감사 인계

상점·HQ·Event·소비품의 GOAL-CSNA-01 구현과 자체검증을 마쳤다. 실제 확장 저장에서 발견한 Continue 예약 과대계산도 수정하고, 기존16MiB 및 세 단계 누적 계약을 유지한 채 최종 정상 host를 다시 승인·실행했다. 수용18/차원10/통합5는 COVERAGE.json, 정확 회귀와 비영향 재사용은 BASELINE_REUSE.json, 실제 결과는 TEST_RESULTS_INDEX.json에 연결한다.

고유 Core2188(기존ignore6), native4, 서버31, 최종 정상host15새프로세스84명령/실제IO4와 재시도, 작성오류12, 귀속·장착·재료 등 판매거절11, 실제 소비품 승리·패배2를 확인했다. 단일 전체녹색 호출이나 제품 배포 SLO를 주장하지 않는다. 초기 가격·공식 유물catalog 공백·유한 진단·Rust B-tree 구현 가정의 한계는 COVERAGE의 known_limits에 명시했다.

최종 target u16_1_current_001은 현재 소스·증거 manifest 및 격리 정역복원으로 고정한다. 실제 완료 여부와 수치는 GEX_10_RESULT_001.json/AUDIT_TARGET_RECORD.json/RESTORE.json을 따른다. 이후 별도 요청의 GEX20 감사가 다음 단계이며, Goal Complete와 U16-2/3/4는 이번 실행의 완료 주장에 포함하지 않는다.
