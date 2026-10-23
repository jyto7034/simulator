# U15 상태 소유·수명

| 상태 | owner | 수명/복원 |
|---|---|---|
| recipe/material purpose | validated catalog | generation identity, invalid authored input reject |
| 예약 equipment_dust 잔액 | RunCraftingResources | 전투/유물/일반재료grant→강화/환급 단일owner; query만투영; checkpoint동시/terminal소멸 |
| 실제 지불 bag/vault 원장 | 장비/파편 사본 | grant0; 이전/형태유지; 분쇄소멸; checkpoint동시 |
| 창고잔량/현재원정debit | profile.lifecycle | 실패잔량유지; 종료debit정리; source delta만 복원 |
| 목표 warehouse debit | run progression/checkpoint | SavePoint/Continue 해당시점 복원 |
| 분쇄 기본보상/환급 | 원정 inventory/crafting_resources/enkephalin | 사본소멸과동시; 투자재화환급0; 환급은소지 |
| 회수품 파편투자 | vault.fragment | 귀환/재대여유지; 다음원정분쇄1회 |
| 무기 일반강화투자 | owned weapon | 성공반출시 level/투자모두소거 |
| 명시출처/귀환선택 | command value | 현재자격/잔량재검증; command candidate원자 |
| 영구지식/무관창고사본 | existing profile | 재료복원의 덮어쓰기대상아님 |

저장에 수락되지 않은 명령의 ActiveRunTransactionRollback은 lifecycle 자산만 별도 controlled clone으로 보유한다. 복원은 현재 단조 지식을 유지하고 이 자산과 run roots를 되돌린다. SavePoint의 source delta와 수명/의미가 다르다. 네이티브 clone accounting은 map을 field별 shape로 방문하며 RunProgression은 serialized_leaf가 아니다.
