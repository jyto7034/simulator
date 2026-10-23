# U16-2 상태 수명

| 상태 | 권한/수명 | 복원/실패 |
| --- | --- | --- |
| 후보pool·가중치·가루임계치 | validated FacilityLayout, static content generation | 정상loader 검증; 세대와 기존 저장계약 |
| reward_activity | U13 facility activity_metadata, 지도 생성 시 고정 | checkpoint/Continue에 고정; 재입장 재배정 없음 |
| 공개후보·선택·투자등급/누적·합성입력 | RewardSessionState optional activity, 노드 세션 | 같은process 유지; 미완료종료는입장전; fixed node seed로동일조건 재입장동일결과 |
| 사본/장착 | RelicInventory copy UUID와단일slot owner | 두입력미장착확인; 비용/결과/UUID원자; Normal신규사본 |
| 가루 | crafting_resources.equipment_dust, run | 다음임계차액만차감; 실패불변; checkpoint와동시복원 |
| 완료·노드소비 | node progression/Continue | 수령/융합/퇴장확정시1회; 완료후보존 |
| active/profile durable files | 정상host paired publish | 실제IO실패메모리/두파일불변·재시도 |

일반/전투보상은activity None이며 기존행동/완료시점을유지한다. 미래후보를세션조회에미리직렬화하지않는다. 후보효과공개는현재선택권에한정한다. 임의변조방어/새저장수명/별도자산권한을추가하지않는다.
