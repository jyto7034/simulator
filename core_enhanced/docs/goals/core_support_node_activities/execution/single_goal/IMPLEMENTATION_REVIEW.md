# U16-1 구현 자체 검토

GOAL-CSNA-01의 수용 조건 18개, 검증 차원 10개와 통합 조건 5개를 검토한다. 정식 GEX20 감사나 Goal 완료 판정은 아니다. 원시 실패와 교정, 실제 호스트 경로와 내부 fixture는 COVERAGE와 TEST_RESULTS_INDEX에서 구분한다.

| 계약 | 실제 구현과 검토 결론 |
|---|---|
| 상품·가격·기본 공급 | shop_data의 상품 세 종류와 base_price/guaranteed를 loader·validation에 연결했다. 새 상품 정가만 양수로 작성하고 기존 정가 덮어쓰기는 금지한다. 사본 수량1, 총가격 overflow, 참조·재료 용도·기본 상품 수·작성 모노리스 금지를 확인했다. 가격은 상품 identity와 분리한다. 기본 상품을 배치한 뒤 남은 자리만 기존 가중 추첨으로 채운다. |
| 지급·결제·실패 | world/shop이 기존 EconomyGrantState를 준비해 비용·사본·UUID·재료·소진을 함께 확정한다. 유물은 정상 사본 하나, 촉매는 각 기존 소유자에게 지급한다. 조회와 구매는 같은 통화·재고·overflow 자격을 검사한다. PurchaseItem과 ShopCommandResult의 양방향 codec에 같은 diff를 전달한다. |
| 판매 금지 | 기존 sale_evaluation의 귀속·장착·유물·수량·가격 검사와 장비/소비품만의 판매 writer를 유지했다. 새 상품을 item_registry의 가짜 물품으로 만들지 않는다. public_edges001에서 지원 제작한 귀속 장비, 실제 선발 장착품, 보유 재료의 거절 사유와 전체 공개 상태 불변을 확인했다. |
| HQ | 기존 가격100·세션2명·4층·보유10명, 기본 세트와 queue/대여 원장 구분을 유지한다. recruitment_terms에 같은 자격을 공개한다. 최종 호스트에서 각각100 차감, 기본 세트4사본 증가, 미완료 되돌림과 완료 유지를 확인했다. |
| Event | 기존 node seed의 d20, committed_check, 직원 적성, 최고 장비 보정, 사본 UUID 동률, 열쇠 조건과 결과 분기를 유지한다. 모든 면/DC 행렬과 실제 공개 raw20 재시작 증거를 구분한다. 무료 침식은 별도 ChooseRelicCorruption 권한과 source-scene/copy receipt를 재사용한다. 외부 공개 consumer와 작성 입력의 fresh-process 회귀가 대상·비용0·중복·복원을 확인한다. |
| 소비품 | maintenance의 즉시 치료 성공 때 이전 modifier도 같은 후보에서 제거한다. inventory/roster 후보와 실패 불변을 유지한다. 전투 기간은 기존 공통 결과 확정에서 감소하고 결과 확인 명령은 receipt만 소비한다. 후퇴는 전투 진입 전 snapshot을 복원한다. 실제 HQ 저장본의 consumable007 승리·패배 두 사례에서 후퇴유지/결과확정만료/중복확인 불변을 확인했다. |
| 저장·호스트 | 기존 session/offer codec에서 새 typed variant가 왕복한다. 작성용 정가/기본 상품 필드를 별도 저장 권한으로 복제하지 않는다. 기본 필드 추가에 따른 semantic identity 변화는 실제 package 결과로 입력 manifest에 반영했다. 미완료 구매의 Continue export가 입장 전 상태를 유지하므로 writer에 닿지 않음을 확인했고, 노드 완료와 SavePoint의 실제 paired publish에서 OS21을 검증했다. |
| 복원 계량 | 실제 장비71개 정상 저장의 복원 실패를 발견해 Value의 B-tree 예약만 정밀화했다. 독립3회 누적, typed decode, checked overflow, 16MiB와 원자적 거절은 유지한다. 실제 allocator 관측, 실패 당시 저장 bytes 복원, 최종 default-release 호스트가 모두 통과했다. RESTORE_MEMORY_REVIEW 참조. |
| 구조 | 새 자산 엔진, 임의 fallback, 보유 개수 제한, 유물 판매, 촉매 혼합이나 추가 환불 정책을 만들지 않았다. 기존 owner에서 producer와 consumer를 연결한다. cmva_build의 --with-worker는 선택 plain host와 worker를 같은 graph에서 준비하며 최종 실제 빌드가 사용했다. |

공유 상품 enum·DTO·loader의 영향은 Core 전체 실행, integration72와 서버 persistence31로 확인했다. 기존6실패는 fixture와 입력 identity를 교정한 exact focused로 닫았다. 이후 복원 예약만 바뀌었으므로 U08·U04·U16·공개 consumer·현재 호스트·비용을 다시 검증했다. 한 번의 전체 녹색 실행을 주장하지 않는다. 기존 ignore6과 socket의 동일 소스 실제 허용환경 증거 재사용을 기록한다.

초기 계획과의 차이는 판단 기록에 남겼다. 가격은 위임된 초기 조절값이고 공식 유물 catalog가 비어 있어 지원 외부 pack을 사용했다. IO 경계, 침식 variant oracle, calibration 전투 준비, private directory 권한, cost probe 선택과 pilot 대기는 준비 교정이며 실패 원본을 보존했다. 복원 계량은 실제 제품 결함이므로 제품 수정과 최종 재빌드·호스트 검증을 수행했다. 확장 준비 기준 분리는 입력이나 한도를 줄이지 않으며 별도의 실제 상점·복원 실행을 유지했다.

U15의 과거 Complete/High 판정과 동결 원본은 보존한다. U16의 공유 상품·복원 변경과 비영향 근거는 후속 U16 GEX20의 검토 대상이다. 자체 검토를 새 독립 감사로 표기하지 않으며 U16-2/3/4는 후속 묶음으로 남는다.
