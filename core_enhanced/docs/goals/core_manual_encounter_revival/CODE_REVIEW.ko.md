# U17 현재 경계와 구현 설계

정적 검토다. 제품 구현·런타임 검증 완료를 뜻하지 않는다. 사용자 U17 착수 요청과 2026-10-15 추가 요청으로 다시 읽은 연속 구현 지시서 §1을 적용한다. 기존 dirty 작업과 U16 XP 비차단 기록은 보존한다.

| 경계 | 확인한 현재 동작 | U17 변경과 책임 |
| --- | --- | --- |
| 최초 보상 | `encounter_progression.rs::plan_encounter_reward`는 source 완료를 보유량과 독립 관리한다. 그러나 `world/combat.rs::plan_victorious_encounter_progression`은 엘리트의 모노리스 3승 뒤 파편을 지급한다. sweep은 파편 보유0이면 재지급한다. | 최초 보상 원장은 유지하되 종류 identity와 source를 검증한다. 수동 승리 plan이 최초/반복/부활 단계 보상을 만들며 구 숙달 보유량 분기를 제거한다. |
| 자연 배정 | `world/map_encounters.rs`는 미숙달/숙달 pool과 모노리스 자연승리를 조회한다. 기존 `assigned_source_ids`는 예약/해제 수명이다. | 자연 엘리트 배정에서 숙달 조건을 제거한다. 이미 배정한 노드·U13 예약·전조 identity를 보존하며 최초 보상과 배정 가중치를 혼합하지 않는다. |
| 모노리스 구조 | `monolith.rs` 하나에 자연승리, 구매, 사용횟수, 재현노드, 잔재교환, 다음층 상인 보장이 함께 있다. `node_flow`, `shop`, `visual_novel_flow`, snapshot이 직접 참조한다. | 새 부화를 구매/숙달 상태에 덧붙이지 않는다. 원정별 알/부활단계/노드 ledger는 전용 owner로 분리한다. 구 활성 명령·상점·VN·query 경로를 제거/명시 거절한다. 관련 없는 VN/shop 구조는 유지한다. |
| 작성과 강화 | `PveEncounterRewardSource`는 fragment와 고정 FragmentDust 5/10/20만 허용한다. `RunPolicyData::resolved_tuning_for_manifested_node`는 모노리스 단계와 전역 배율을 결합한다. | source별 최초 장비/반복 효과와 일반 보스의 단계별 encounter·촉매 recipe·파츠·장비 후보를 명시 작성한다. 부활 강화의 단일 owner와 노드의 typed stage를 연결하고 무억제 곱셈은 기존 U14를 사용한다. 구 reward dust 강제값은 새 작성 정책으로 대체한다. |
| 부화와 진입 | `handle_convert_map_node_with_monolith`는 NodeConfirm에서 노드를 전환한다. `SelectMapNode`는 preview/session, `ConfirmEnterNode`와 expedition entry는 별도 비용/Continue 경계다. | 별도 hatch 명령은 후보 검사→비용 candidate→알/노드 전환을 원자 적용하고 NodeConfirm을 유지한다. 전투/후퇴를 시작하지 않는다. node-session payload와 preview cache도 같은 transaction에서 갱신한다. |
| 촉매 출처 | U15 `material_economy::pay_materials`는 정확한 소지/창고 선택·부족 거절·warehouse_debits를 제공한다. U16 상점에 RevivalCatalyst 상품이 존재한다. | 이 지불 owner를 재사용한다. bag와 profile vault/debit 및 run debit을 함께 publish한다. 새 지불/환불 알고리즘을 복제하지 않는다. |
| 결과·장비 선택 | `plan_player_victory_combat_result_completion`은 fixed 효과와 ChooseOne session을 구분한다. terminal commit은 노드·진행·grant와 `CombatRewardContinuation`을 같이 확정한다. | 단계의 고정 파츠/알과 장비 후보를 각각 같은 결과 plan에 공급한다. 이미 있는 U04 권리·후보 고정·선택 처리·재접속 경로를 재사용한다. reward 선택이 진행 기록의 두 번째 commit이 되지 않게 한다. |
| Continue와 후퇴 | `active_run/continuation.rs`는 미완료 노드의 pre-entry root를 보존하고 후퇴횟수만 overlay한다. NodeConfirm은 unfinished가 없으면 stable이다. | 입장 전 hatch는 stable root에 저장하고 미완료 노드 안 hatch는 pre-entry root에 포함시키지 않는다. 실제 battle retreat는 hatch 후 snapshot을 사용한다. Continue overlay와 live retreat를 혼동하지 않는다. |
| SavePoint·창고 | run checkpoint의 progression과 U15 warehouse_debits/profile reconcile이 함께 복원된다. 현재 active payload version16이다. | 신규 부화 ledger를 checkpoint/clone/계량에 편입하고 형식 successor를 발행한다. USER11에 따라 구형을 원본 보존/명시 거절한다. vault 전체 rollback은 금지한다. |
| 공개 소비자 | `behavior.rs`, `world.rs`, `world/application.rs`, action capability, snapshot과 서버는 typed 명령/결과를 공유한다. | hatch 조회/preview/commit과 단계·비용·알·보상 후보를 동일 authority로 제공한다. 구명령에는 명시 retired 오류를 반환하고 새 상태에서는 제안하지 않는다. |

원정 수명과 단계 상태는 `EncounterProgressionState`의 전용 하위 owner가 소유한다. node/session, 지불, 결과, 저장은 이 owner의 plan을 소비한다. 구현은 이 경계 안에서 기존 불필요한 모노리스 결합을 제거하며, 무관한 전체 전투/상점 재설계는 하지 않는다.

일반 보스 종류별 첫 승리→알1→2단계 승리→알1→3단계 승리→알0을 작은 독립 기대값으로 검증한다. 패배는 노드 소비/승리 기록 불변/환급0, 수동 후퇴는 node 미해결/후퇴+1/알·촉매 불변이다. 전투 진입 전, 미확정 노드 내부, 수동 후퇴, 결과 확정, 선택 대기의 저장 경계를 각각 관측한다. 정상 원정의 Event/상점/전투·장비 장착으로 입력을 준비하며 목표 연산 직전 자격과 잔액을 확인한다. 지원 RON 오류·IO 실패는 범위 안이며 임의 저장 편집은 제외한다.

실행 계획은 ledger/작성, 부화/결과, 공개/복원, 통합 검증을 한 Phase에서 함께 전환한다. 구저장과 구명령을 명시 거절하고 새 정상 경로가 완성되기 전 부분 배포하지 않는다. 전역 도감·새 원정·튜토리얼·다른 보상 활동의 선행 회귀도 영향 범위에서 검증한다.
