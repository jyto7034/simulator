# U16-1 코드 검토와 구현 인계

2026-10-15 Workspace Actual의 지정 경계 정적 검토. U15 independent001 Complete/High가 선행 기준이며 이번 상품/활동의 runtime 통과 증거는 아직 없다. U16-2/3/4는 로드맵의 별도 인수 묶음으로 남는다.

| 경계 | 현재 실제 owner/경로 | 변경과 검증 |
|---|---|---|
| SHOP | shop_data의 typed merchandise는 Equipment/Consumable/동적 Monolith만. validation의 정가와 차익검사도 장비/소비품만. world/shop은 고정 seed/page/offer, EconomyGrantState로 결제와 grant를 staging | 유물·부활촉매·침식촉매의 명시 상품을 추가. 재료 범용 상품은 열지 않음. 기존 정가 없는 종류는 상품 작성 정가로 처리하며 identity는 가격과 분리. 일반상인 기본 부활촉매를 페이지 추첨과 구분해 제공. 구매/갱신/불충분/중복을 한 세션에 묶음 |
| GRANT | RewardEffect GrantRelic/GrantRelicCorruptionCatalyst/GrantEquipmentMaterial, EconomyGrantState의7roots와 사본 UUID 존재. shop PurchaseItem은 inventory_diff만 반환 | 기존 grant 권한을 재사용. 유물/침식촉매 diff 및 현재 자원 조회를 구매 결과에서 전달. 유물 비중복 자격/재료 수량 overflow도 사전 자격과 명령에서 일치. 새로운 재화 엔진 없음 |
| HQ | headquarters::handle_recruit_employee는 queue/employee UUID를 준비하고 state transaction 안에서 가격100과 initialize_employees_with_loadouts, 세션2명/깊이4/보유10 제한 | 동작 재사용, 후보/장비파편/대여 분리를 실제 공개 경로에서 재검증. selected_event에는 후보만 있고 가격/횟수 설명이 없어 공개 terms 추가. 부족 시 후보가 빈 목록인 현상은 가격/자격 안내로 구분 가능하게 함 |
| EVENT | event_node roll_event_check는 node seed 전용 namespace, receipt에 raw/선택/직원/보정/분기 보존. choice/equipment eligibility와 raw1/20 classifier 존재; event corruption은 기존 소유 사본 경로 | authoritative 판정 재사용, 실제 public 명령과 정상 작성 Event로 재접속/새프로세스/분기/무료침식 확인. 표면 enum만 테스트해 노드 완료 검증을 대신하지 않음 |
| CONSUMABLE | maintenance::handle_use_consumable_item은 inventory/roster candidate. 일반 modifier는 교체하나 ReducePhysicalDamage는 치료만 하고 기존 modifier를 유지. combat final result의 두 경로에서 기간 감소, retreat는 checkpoint 복원 | 일반 소비품 새 사용은 기존 활성 효과를 대체한다는 명시 규칙과 다른 즉시치료 경로 수정. 효과/치료량 정책 자체는 유지. 승/패/후퇴/비전투/중복결과·50/60 초과 인벤토리 획득을 검증 |
| STORE | resources/selection의 ShopSessionState/offer와 run shop_sessions는 serde 저장. state checkpoint에 inventory/relic/crafting/roster/UUID/run 포함, Continue는 미완료 노드 입장 전 snapshot 선택. server application_intent가 active/profile paired publish 소유 | 새 상품 variant/필드의 writer-reader/clone_shape/checkpoint validation/Continue를 함께 전환. 현재 추가필드 없이 버전 재사용 가능 여부를 GEX에서 실제 codec로 확인하고 필요시 CF-D01에 따라 명시 버전상향. 무관한 profile/창고 rollback 금지 |
| PUBLIC | behavior PurchaseItem/ShopOfferSnapshotDto DisplayItem은 장비/소비품/모노리스 중심. snapshot은 monolith 외 모든 상품을 item_registry 조회. application owner dispatch/normal host가 실제 진입점 | 유물/촉매제 display DTO와 고정 가격·자격·재고/부활용 구분. typed 결과→application→서버 직렬화 전 경계 함께 변경. U15 정상 admission host와 replay collector를 재사용하되 새 source로 재빌드/승인; 이전 승인 cache를 현재 제품으로 주장하지 않음 |

상품·grant·query·저장·작성 검증은 같은 Atomic Group으로 전환한다. HQ/Event/소비품은 자체 기존 owner를 재사용하되 같은 U16-1 Goal에서 상호작용과 공개 증거를 닫는다. 서로 다른 후속 유물 활동은 이 Goal의 prerequisite가 아니며 임시 구현/대체 보상을 추가하지 않는다.

초기 콘텐츠 기준가는 P2G-10에 구분 기록한 위임 판단(유물60/부활150/침식30)이다. 기존 HQ100과 비교하는 플레이 조절값이며 실제 거래에는 기존 매입비율/올림이 적용된다. 기본 부활촉매는 일반 상인 초기 페이지의 고정 상품으로 작성해 희귀 추첨을 요구하지 않는다. 정확한 데이터 필드/slot 배치는 기술 설계이며 보유 상한이나 무료 보충은 없다.

검증은 먼저 정상 새원정 선발→기본세트→실제 배정된 지원 노드 입장→조회/명령으로 준비한다. 상점은 구매 전 공개된 offer ID/총가격·자산 개수를 기록하고 거래 뒤 차액/사본/재고를 비교한다. 갱신 부족은 통화 또는 실제 후보 소진으로 만들고 거절 전후 전체 snapshot을 비교한다. 부활/침식 자원을 각각 구매해 별도 잔량/사용 자격을 확인한다. 판매는 장착/귀속/미장착 및 유물 사본을 구분한다.

HQ는 세션 후보를 실제 선택해100씩 두 번 채용, 세 번째/반복은 무변경을 검증한다. 기본세트 사본 수와 대여원장을 함께 비교한다. Event는 작성된 선택장면과 seed별 raw1/20을 먼저 조회/정상 판정해 확인하고 선택 직원·열쇠/장비 보정·분기 결과를 비교한다. 무료 침식은 정상 획득 사본을 먼저 준비한다. 단순 조작 실패가 IO/부족 검증을 대신하지 않게 사전 상태를 확인한다.

소비품은 지도에서 구입/획득 스택을 사용한 뒤 전투 미리보기·입장→후퇴/승/패/결과확인으로 나눠 기간을 관측한다. 즉시치료는 실제 부상 직원+기존 modifier를 준비하여 치료·수량1·이전효과 제거를 관측한다. 내부 준비 fixture는 normal path 증거와 구별한다.

미완료 상점/HQ/Event의 중간 변경은 프로세스를 실제 종료한 뒤 Continue하여 입장 전 비용/결과를 비교한다. 완료시킨 별도 경로는 새프로세스에서 결과를 유지한다. SavePoint는 checkpoint 이전/이후 자산을 구분한다. IO failure는 유효한 구매/채용/선택 입력이 실제 저장 경계에 도달한 뒤 OS 오류로 유도하고 메모리·파일 보존과 재시도를 확인한다.

비용 검사는 대표/확장 상품·사본에 같은 workload를 사용하고 시간/실제 heap 또는 RSS peak·회수와 내부 work를 구분한다. 기존 회귀는 변경 영향과 exact/nonimpact 근거로 재사용한다. 정식 빌드/테스트는 GEX에서 수행한다. 배포/Unity 접근을 가정하지 않으며 외부 임의 변조 전용 방어는 범위 밖이다.
