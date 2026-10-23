# U14 구현 자체 검토

현재(2026-10-14 교정001): 두 제품 결함의 교정·자체검증을 마쳤으며 비참여 재감사 대기다. 마지막 교정001 절과 `correction/u14_correction_001/RESULT.json`을 따른다. 앞선 구현pass의 최종 표시는 당시 이력이다.

GS-CEDR-001 / GOAL-CEDR-01 / PHASE-CEDR-01, FPF-CEDR-001의31수용 조건을 검토했다. 이 문서는 구현자의 소스·증거 검토이며 독립 GEX20 판정이 아니다. 신규 완료차단 결함0, 구현·자체 검증 종료와 감사요청 인계가 결론이다. 기존 전체 Core138실패는 실제 pre-U14 입력에서도 재현했으며 BASELINE_REUSE.json에 범위와 원문을 분리했다.

## 입장·상태·복원

`world/expedition_lifecycle.rs`는 activity→node definition→run 기본값으로 비용을 해석하고 Start/Gate를0으로 처리한다. 기존 entry_receipts 한 곳만 최초입장을 소유한다. 실 잔량은 saturating 차감하고 직원은 규정비용 전체를 소비한다. `employee.rs`는 remaining_cost만 유예 권위로 사용하며 기존 entered_node는 발생 위치 정보다. 완료와 방 이동의 감소 호출은 제거됐다. trauma99 회복 시 즉시 reconcile해 SavePoint 내부 캡처 이전에 counter가 해제된다.

`node_flow.rs`는 Continue를 비용 전에 캡처하고 비용·사망·자산 회수 뒤 소진선택/활동을 진행한다. 실제 새 사망이 생겼을 때 기존 recovery/failure 판단을 실행하므로 원래 출전불가 상태의 합법 소진 Return을 막지 않는다. `expedition_recovery.rs`의 치료 가능성도 같은 작성 비용으로 입장사망을 제외한다. 자산은 기존 clone/grant owner로 회수하며 relic binding도 reconcile한다. 외부 사망은 전투 DeathPrevent와 경로가 다르다.

미확정 같은 node의 Continue payload를 재입장으로 덮어쓰지 않는다. Gate pending 선택은 안정 상태로 저장하고, 활동 소진선택은 비용전 상태로 돌아간다. 수동후퇴는 비용/사망 이후 BattleEntrySnapshot과 시도횟수의 기존 소유권을 유지한다. current writer15만 Core가 이어하기로 받으며 host의 이전 opaque payload 보관은 호환 지원이 아니다. SavePoint13 별도 형식은 유지한다. 공개 reader는 정렬된 직원 UUID와 같은 cost resolver를 사용하고 조회가 receipt를 만들지 않는다.

## 전투·보상

`RunPowerAuthorityState::manifest_floor`는 authored_minimum_power만 적 난이도 anchor로 사용한다. certified power/peak와 이전 minimum_rise 필드는 기존 진단/저장·작성 구조로 남지만 적 선택에 다시 사용하지 않는다. 실제 강화된 roster와 큰 peak 대조에서도 Normal/Elite/Boss 지연 spawn의 구성과 능력이 동일했다.

`BattleDeterrenceSnapshot`은 진입 비용 이후 자격과 HP/ATK·수량배율을 고정한다. cached preview 갱신은 독립 encounter scale 필드를 대입해 반복 조회로 누적하지 않는다. `stat_pipeline.rs`가 difficulty/revival 및 exact override 뒤 encounter HP/ATK를 적용하고 다른 통계는 유지한다. late spawn도 같은 draft scale을 소비한다. 무억제는 sealed scale 대신 적용하며 revival과는 곱해1.5×1.5=2.25다. U16은 이 snapshot seam을 소비할 수 있지만 스트레스 활동 구현은 포함하지 않는다.

`PerformanceBonusState`는 주대상 UUID, 최고 실효손실 분자와 당시 최대HP 분모의 고정 크기 상태다. 실제 damage mutation 이후 플레이어→주적의 BasicAttack/Ability만 관측해 DOT·흡수·요청 overkill·여러 타격 합산을 배제한다. u64 교차곱은 u32 입력 두 개의 곱 범위 안이다. 실제 종료시간과 목표별 조건을 Victory에서 한 번 평가하고 기존 staged reward로 전달한다. clone shape가 새 상태를 포함한다.

생존XP는 실제 result의 참가자와 alive employee condition을 대조해 UUID 집합으로 중복을 제거한다. Victory의 수량배율만 적용하므로 패배10과 후퇴0을 유지한다. 일반 보상 및 encounter 추가/성과 effects를 기존 지급 전에 한 번 배율화하며 checked_mul 실패는 기존 transaction을 실패시킨다. whitelist는 Enkephalin/EquipmentDust/EquipmentMaterial/Experience뿐이다. XP target과 선택횟수·사본·촉매제·알·가지/보충에는 새로운 의미를 넣지 않는다. bonus XP의 AliveRoster는 기존 per-recipient grant owner와 diff 경로를 재사용한다.

기존 inventory equipment_materials의 equipment_dust와 crafting_resources의 equipment_dust는 U14 전부터 구분된 owner다. 실제 승인 호스트에서 기본 보스재료14와 별도 성과가루16을 각각 조회했다. 이번 작업은 이 둘의 의미를 통합하거나 새 장부를 추가하지 않았다.

## 황금가지·작성·공개 경계

`map/golden_bough.rs`는 최초 원정에 seed/가중치로 한 plan을 선택한다. plan/location/collected/refill은 run의 facility 상태이며 방별 재추첨이 없다. 검증은 후보·계획ID·양수 weight/refill·위치 중복·모든 후보 wing·ENTRY 도달성·비최종보스·조건 없는 지원 경로를 확인한다. 실제 실행 후보는 live loader에서 NormalBoss를 요구한다. 공식 미지원 본체의 예약은 명시 Unsupported로 남아 실행되지 않는다.

`facility_rooms.rs::sealed_golden_bough`는 현재 방의 원래 slot과 미회수 위치를 일치시키고 monolith_stage가 있는 부활 노드를 제외한다. 원래 승리만 collected insertion과 checked refill을 한 번 수행한다. 이 호출은 기존 terminal result 후보 안에서 settle에 앞서 실행돼 후속 실패 시 준비 후보 전체가 버려진다. `sealed_host001`은 실제 paired journal IO실패에서 모든 메모리·active/profile bytes가 그대로임을 확인했고 동일 tick 재시도로 승리와 보상이 한 번만 반영됐다. entry만으로는 지급하지 않으며 패배의 실제 전투에서도 가지/보충0을 확인했다.

공개 지도에는 위치·회수·현재 경로 도달성만 추가하고 U13의 hidden encounter 이름/ID 숨김을 유지한다. 접근가능성은 전방 하강과 같은 높이 양방향 연결을 반영하므로 지나친 분기의 가지를 접근가능으로 바꾸지 않는다. 모든 가지 회수를 강제하는 경로 보정은 없다.

## 검증 구성과 한계

COVERAGE.json은31수용/10차원/10통합 경계를 실제 원시 결과에 연결하며 TEST_RESULTS_INDEX.json은 실행한 test 이름과 수를 보유한다. 내부 trauma/balance fixture는 경계 전제 준비로 표시하고, 전투 승패는 실제 공격/시간으로 관측했다. 외부 public 진단과 승인 호스트를 구분했다. source admission 우회 없이 실제 host binary identity로 calibration/adoption 후 정상 명령을 실행했다. 최초 calibration의 unsupported Abnormality 항목은 기존 계측기 미지원 정의 종류를 잘못 요구한 것으로, byte-identical actual scenario와 외부 판정을 유지한 채 지원 PveEncounter/Employee/RunPolicy coverage로 교정했다.

full Core002의1940PASS와 baseline138실패는 숨기지 않는다. Gate/recovery/preview/codec 변경의 영향은 최종82집중 검사와 G11기본4/계측4, 서버29, 승인호스트30단계+승리IO10단계로 닫았다. XP 다중레벨/대상·기존 grant/monolith의 비영향 full-run PASS를 재사용한다. 새 성능 한도나 product API를 시험 편의상 도입하지 않았다.

공식85/확장169방 입력에서6 fresh process의 실제시간·CountingAllocator·RSS·native work를 구분해 기록했다. 측정은 한 방 현현과 전체 정적 그래프 query이며 모든 방 방문이나 최종 SLO가 아니다. 기존 한도는 normal/sealed review에서 여유가 있어 유지했다. G11은 exact/one-short 거부·취소·복원·재시도를 current pack으로 확인했다. raw0인 미계측 operation을 무비용으로 해석하지 않는다.

12전 성장 경로는 공식 적/무기/XP/보상 수치를 유지하고 전장 OpenHall/Medium과 경로만 지원 RON으로 작성했다. 최초 세 전투의 원래 전장 승리도 원문으로 남겼으며 Corridor에서 성립하지 않은 진행기 전략을 제품 의무로 바꾸지 않았다. 공식 네 고유보스/최종보스 본체, U16 전체, Unity와 배포·임의변조 전용 방어는 제외한다. 정식 Goal Complete는 별도 비참여 감사의 판정 뒤에만 가능하다.

## 2026-10-14 교정001 자체 검토

위 본문은 target001 구현 당시 이력이다. 후속 독립 감사 F-U14-001을 인수했고, 본 교정 중 C-U14-002를 추가 발견했다. 독립 감사 판정은 갱신하지 않는다.

facility_rooms의 현재방 wrapper는 current map에서 node를 얻어 공통 room/node helper를 호출한다. 전투 진입/정산의 호출 의미는 유지한다. facility_view는 순회 중인 방ID와 그 방 map의 node를 전달하여 미회수·원래 slot·monolith 제외 조건을 동일하게 적용한다. expedition_lifecycle의 current wrapper는 기존 current_map_view와 일반 모드를 보존하고 room-aware helper에서 sealed를 한 번 계산하여 두 공개 DTO의 같은 preview로 전달한다. 비용은 기존 activity metadata/receipt를 참조하며, hidden encounter ID/label 마스킹을 변경하지 않는다. 새 캐시·런타임 상태·schema·정책은 없다.

정상 public002 collected 경로가 초기12보다 높은16의 checkpoint를 생성하고 Continue에서 거절된 것이 C-U14-002다. node_flow 검증의 과거 초기값 상한을 이미 회수한 가지의 공급량까지 포함하도록 바꿨다. 보충 producer는 checked_add로 지급하고 본 변경은 검증에만 적용된다. u64 saturating 공급 계산으로 산술 wrap을 방지하며 실제 잔량은 기존 u32다. 복원 current와 SavePoint 각각의 저장 원장을 사용한다. 지원되지 않는 임의 변조 방어를 새 목표로 추가하지 않았다.

새 Core 왕복 사례와 기존 실제승리 사례의 balance12 대조, 공개 3상태(억제/무억제/회수) 및 각 왕복 전후 fresh-process 복원으로 공개와 저장 소비자를 확인한다. 승인호스트의 IO journal/재시도/원자성과 reward producer 자체는 수정하지 않았으므로 target001 원시 증거를 해당 메커니즘에 한해 재사용한다. 공개 현재 바이너리의 통과를 새 Normal 승인으로 해석하지 않는다. 최종 수치/원문은 correction/u14_correction_001/RESULT.json과 COVERAGE.json이 연결한다.
