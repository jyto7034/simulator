# U15 코드 검토와 구현 인계

2026-10-14 Workspace Actual의 지정 경계 정적 검토다. 제품 실행/Released/Deployed 검증은 아니다. U14 독립 재감사002 Complete/High/open0와 기본 Core full001/socket002 총2169통과는 선행 근거이며 U15 구현 증거로 사용하지 않는다.

## 실제 경계와 delta

| 경계 | 현재 코드/동작 | 필요한 변경과 검증 |
|---|---|---|
| COPY | resources/inventory OwnedEquipment는 enhancement_level만, skill_fragment OwnedFragment는 forms/owner만 소유. VaultFragment도 투자 없음 | 사본별 실제 지불 재료 기록. 무료 grant는 빈 기록, 이전/형태 변경은 같은 기록, 분쇄 제거. clone_shape/serializer/실제 소비자가 함께 전환 |
| RECIPE | equipment_data enhancement validator는 어떤 장비/재료든 참조가 있으면 수용. base.ron은 standard_armor MaxHealth/Defense 강화만 작성 | 무기/소지 가루/공격력·주문력 강화로 제한. 현재 방어구 recipe를 무기용 초기 콘텐츠로 교체. 중복 합산·비용종류·양수/overflow·참조 검증. 기존 방어구 강화 증거를 새 무기 검증으로 재사용하지 않음 |
| MAINTENANCE | world/maintenance는 소지 재료만 개화에 소비, 투자는 없음. fragment 분쇄는 EconomyGrantState candidate, equipment 분쇄는 preflight 후 live remove/grant. 장비 확인 인자 없음 | 출처 선택 결제와 투자/분쇄를 동일 candidate에서 확정. 확인 거절 무변경과 합산 보상 overflow를 원자 검증. 기존 장착 transition/정상 grant 재사용 |
| VAULT | expedition_lifecycle vault는 회수품 사본 두 map만. Return은 전 장비/파편 반납, weapon level reset. 재료/선택회수 없음 | 별도 재료 map과 원정 소비 영향. 파츠합계2/촉매1 선택회수·실패미환급. 대여와 다른 수명을 유지. 파편 투자 보존/무기 투자 소거 |
| RESTORE | state RunCheckpointPayload는 run과 자산 snapshot, profile 제외. node_flow와 active_run/continuation의 두 restore_into_state caller. profile은 lifecycle exact adopt와 knowledge merge를 구분 | run에 누적 창고 소비, profile에 현행 소비를 함께 보관하고 restore 때 현행-목표 차이만 reconcile. 현재/SavePoint/미확정입장전 Continue를 모두 소비. 전체창고 snapshot이나 합집합 merge 금지 |
| PUBLIC | behavior 기존 AwakenSkillFragment/DismantleEquipment/ChooseExpeditionContinuation, facility/snapshot의 비용·형태·장비 preview | 명시 출처와 귀환 선택 입력, 장착분쇄 확인 입력, 보유/비용/투자/기본·환급 조회. 기존 command가 창고를 묵시 소비하지 않도록 유지. 정확한 variant/DTO 배치는 구현자 선택 |
| STORE | active_run current/Continue v15. player profile과 active는 application_intent의 paired durable publication. journal decode는 version부터 확인 | 새 필수 투자/창고/consumption shape는 successor version. 미지원 구형은 원본보호 명시거절. 실제 write failure/retry와 새프로세스 Continue 확인 |
| AUTHOR | equipment material 분류 Fragment/Core/Blueprint/Residue/Generic에는 환상체파츠/부활촉매의 명시 의미가 없음. 공식 equipment_dust는 Generic, 부활촉매 재료 없음 | authored purpose를 분명히 하는 분류와 정식 가루/파츠/촉매 fixture. 부활촉매는 침식촉매 RunCraftingResources와 구분. U17은 이 공통 결제/수량 seam을 소비; U15가 부화 전체를 구현하지 않음 |

## 권장 구조와 유효 중간 상태

위 표의 저장·사본·결제·환급·귀환은 하나의 Atomic Group이다. 저장 필드만 먼저 완성하거나 재료만 차감하고 복원을 다음 Phase로 남기는 분할은 불가하다. PUBLIC과 검증도 같은 Single Goal에 배정한다. 단독 실행하며 병렬 위임하지 않는다.

사본 투자는 재료별 실제 소지/창고 지불 합계를 보관하는 작은 ordered map이 적합하다. 출처를 추적하되 환급은 총량/2를 현재 소지분으로 지급한다. 개별 거래/형태마다 환급을 내림하지 않는다. 무기 일반강화의 투자는 반출 시 삭제하고 파편 개화 투자는 회수품 사본에 이동한다. 기본 분쇄 recipe와 투자를 합산한 결과를 사전 검증한다. 기존 recipe의 엔케팔린 기본 보상은 투자환급과 구분하며 투자 엔케팔린은 기록/환급하지 않는다.

원정별 warehouse_debits는 결과와 같은 rewind scope다. profile의 warehouse_debits는 현재까지 확정한 직접 소비를 식별한다. restore는 profile의 현행 debits와 checkpoint의 목표 debits 차이만 warehouse에 적용하고 목표를 채택한다. 수량은 checked 연산, 성공 시 함께 commit. 현재 원정 identity 일치를 기존 lifecycle gate가 검사한다. 실패는 남은 warehouse를 보존하고 debit 추적만 종료한다. 입장 전 Continue의 current checkpoint와 저장 profile이 다른 소비 시점일 수 있으므로 fresh-process에서도 같은 reconcile을 수행한다. profile 전체 clone rollback은 사용하지 않는다.

기존 EconomyGrantState/EquipmentLoadoutTransition를 재사용한다. 재료 지불은 요구량 합산→선택출처 합계/자격/보유량 검증→candidate 소비→투자 기록→성장→공개결과→application publish 순서다. 명시 선택을 다른 출처로 대체하지 않는다. 외부 public behavior는 기존 소지전용 경로와 새 명시 선택 경로를 구분할 수 있으며 두 경로가 같은 결제 authority를 쓴다. 정확한 DTO/모듈은 GEX에서 선택한다.

기반 개선 필요: 기존 equipment 분쇄의 live remove→grant를 candidate로 옮겨 환급 overflow 실패 때 사본만 없어지는 경계를 없앤다. 개선 범위는 U15 경제 조작에 한정한다. 일반 grant 엔진/다른 도메인의 추상화 재구성은 하지 않는다.

## 검증 준비와 기대값

- 가루 3+4 투자 후 분쇄 환급3, 무료 강화는0. 파츠A 소지3+창고2·파츠B3 투자면 환급 A2/B1이다. 엔케팔린 비용은 환급0. recipe 정가를 바꾼 다른 generation에서도 원장 기반 기대는 동일하다.
- 혼합 개화에서 원정 소지/창고 수량과 형태/투자 모두 비교한다. 부족/불일치/확인거절은 전후 전체 공개상태 동일. 같은 노드 여러 작업 뒤 최초입장 receipt/억제/유예 비교.
- SavePoint를 생성·이동·정비진입한 실제 경로에서 혼합개화→분쇄→복구. 저장 이전 투자/다른 창고자산/영구기록과 이후 거래의 차이를 따로 관측. 반복복구 최대3 기존 규칙 유지.
- 미확정 정비 종료 후 새 프로세스 Continue는 입장 전, 완료 후 Continue는 완료 거래. 같은프로세스 roundtrip으로 대체하지 않는다. 지원 host paired publish의 실제 IO 실패/재시도까지 확인.
- 깊이3 Gate/소진 Return 자격을 만족한 뒤 선택 파츠2+촉매1 회수. 한도2/3·1/2와 금지품 거절. 개화파편 반납→새 후보/직원 선발→대여→정비 분쇄를 실제 사본 identity로 검증. 실패/포기는 미사용창고 유지/사용분미환급.
- 정상 외부 RON과 invalid authored 조합을 실제 loader/pack로 검증. 공식 데이터 전체와 새 현재경로 fixture의 용도를 구분. 기존 무효 방어구강화/Generic 재료 fixture는 목적 보존 후 전환하며 과거 동결 fixture는 보존.
- 비용은 대표 정상/확장 재료·사본에서 wall time/실제 heap 또는 RSS peak/종료회수와 내부 work를 구분한다. profile 한도는 측정 후 판단. 오래된 승인 cache를 새 source의 Normal 승인으로 주장하지 않음.

정상 지원 경로만 검증한다. 파일/checksum/메모리 임의변조 전용 사례는 §1.9 제외. 배포 환경은 제공되지 않았고 현재 범위는 workspace 및 접근 가능한 host이며 배포 완료를 주장하지 않는다. Unity/공식 개별 보스 본체/U17 부화 전체는 후속 owner다.
