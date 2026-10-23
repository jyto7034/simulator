# U16-1 상태와 수명

| 상태 | 권위 | 저장·복원·소멸 |
|---|---|---|
| 상품 정가/기본목록/랜덤 후보 | validated ShopDatabase | 콘텐츠 generation; 세션 중 이미 공개한 offer 가격은 고정 |
| offer ID/가격/재고/구매이력/페이지/갱신횟수 | run.shop_sessions + active ShopSessionState | 같은프로세스 유지, 미완료 새프로세스는 입장 전, ExitShop 완료 뒤 유지 |
| 유물 사본 | RelicInventory + UuidManager | 기존 grant/장착/침식/분쇄/체크포인트; 일반상점 매각 없음 |
| 부활 촉매 | inventory material RevivalCatalyst | 소지/창고 기존 U15; 구매는 소지. 실제 부화 U17 |
| 침식 촉매 | RunCraftingResources | 기존 침식 사용/복원. 부활용과 교환/혼동 없음 |
| 직원/기본세트/queue/채용횟수 | roster/inventory/fragments + run.recruitment | 기존 U12 정상 loadout/세션/원정 소유, 대여 원장 별개 |
| Event roll/선택/직원/장비증거/결과 | EventSession committed_check | node seed·receipt와 기존 branch/completion/rewind |
| 소비품 수량/직원활성효과 | inventory.consumables / employee modifier | 사용 시1소비/교체, 결과확정1감소, 후퇴는 진입전, 비전투 무감소 |
| durable active/profile 결과 | application staged candidate + paired host intent | 실제 IO실패 rollback/retry; 독립 영구기록/창고 전체 rewind 없음 |

복원 임시소유는 typed decode, checksum Value, stable-capture Value의 독립 누적 예약이다. Value의 객체노드 상계만 정밀화하며 세 소유자의 예약을 조기에 환급하지 않는다. 실제값 생성과 해제의 관측은 restore_cost001 및 RESTORE_MEMORY_REVIEW에 연결한다.

query/DTO는 상태의 표시이며 별도 지급/자격 권위가 아니다. 신규 상품을 normal Item registry의 가짜 장비/소비품으로 만들지 않는다. 필드·codec 수정 시 reader/writer/clone_shape/consumer를 함께 검사한다.
