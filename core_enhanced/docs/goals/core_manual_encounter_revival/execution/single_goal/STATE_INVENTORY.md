# 상태 소유와 수명

| 상태 | 소유·수명 | 저장·복원·검증 |
| --- | --- | --- |
| 최초 완료 원장 | EncounterProgressionState / 원정 | 보유량과 독립, 새 원정 reset, 결과 commit과 checkpoint |
| 알·다음 부활 단계 | 전용 revival state / 원정 | 성공 결과/hatch 원자 전이, 보관/귀환 제외 |
| 부화된 노드·stage·source | revival state + map/node payload / 해당 층 | 재방문 유지, 하강 시 미해결 방치, 새 방으로 이전 없음 |
| 촉매 | 소지 inventory 또는 profile vault | 기존 MaterialPayment와 warehouse_debits; 원정 delta 복원 |
| 전투 진입 복원점 | 기존 battle snapshot | 부화 이후 상태, 후퇴 비용 유지, 직원/소모품 복원 |
| 미완료 Continue root | ContinueState | 최초 입장 전 불변 root, live 후퇴와 분리 |
| 승리 장비 선택 권리 | 기존 RewardSession/CombatRewardContinuation | 승패·노드와 동시 저장, 선택 grant1회, 후보 고정 |
| 도감·영구 격파 | 기존 profile | 최초 보상 원장과 별개, tutorial 격리 |

실제 owner는 `EncounterProgressionState.completed_reward_source_ids`와 그 안의
`RevivalRunState { eggs, nodes }`다. `HatchedEncounter`는 source·stage·resolved·출처별
`MaterialInvestment`를 보관하고 map은 `RevivalNodeTuning`을 보관한다. `RunState`가 progression을
통해 이 상태에 접근하며 구 `MonolithRunState`를 소유하지 않는다. 구 DTO·standalone monolith 모델은
기존 wire/clone-provenance 회귀 호환용으로 남지만 활성 Core 생산자·저장·제안 경로에는 연결되지 않는다.

Continue와 독립 checkpoint는 V17, host active envelope도17이다. 구 모노리스 상태를 새 부화로
묵시 재해석하지 않고 typed version 오류로 원본을 보존한다. 결과 후보 생성자와 복원 reader는 동일한
`RevivalStagePolicy::equipment_reward_options`를 사용한다. 선택 entitlement는 결과 즉시 저장한다.

완료/패배로 소비된 node의 `combat_previews` cache는 완료 transaction에서 해제한다. 결과와
선택 권리의 durable owner는 유지하며 미해결 node의 preview만 다음 진입에 남긴다. 이 수명 교정으로
실제 fresh Continue의 기본16MiB 한도 초과를 해결했다. 원문·회귀는 `BASELINE_REUSE.json`에 연결한다.
