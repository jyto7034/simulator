# 프로젝트 종합 검토 보고서 — 2026-07-10

> 상태: 감사 스냅샷
>
> 작성일: 2026-07-10 (KST)
>
> 이 문서는 현재 구현과 live data를 기준으로 확인한 결함, 계약 불일치, 정책 질문을 보존하는 감사 보고서다. 새로운 gameplay 또는 wire 계약의 source of truth가 아니다. 정책이 결정되고 구현이 바뀌면 가장 가까운 canonical 문서와 테스트를 갱신해야 한다.

## 검토 범위와 판정 기준

검토 범위:

- Core runtime과 상태 전이
- 전투 lifecycle, 결과, 보상, 실패, retry, retreat, 원자성
- live RON schema, loader, startup validation
- Core ↔ game_server ↔ Unity DTO와 WebSocket 경계
- 중복 state, 우회 경로, fallback, compatibility, dead state
- 사용자-visible 또는 외부 계약을 보호하는 테스트
- durable 문서와 완료 goal 문서의 현재 구현 정합성

판정 근거의 우선순위:

1. 실제 runtime code
2. live RON/data
3. typed DTO와 game_server
4. Unity canonical 문서와 현재 ArkClient 구현
5. durable 문서
6. goal 및 과거 작업 문서

판정 상태:

- 확정·현재: 현재 live 또는 일반 사용자 경로에서 직접 성립한다.
- 확정: 코드 경로와 상태 전이로 성립하지만 특정 live 콘텐츠 또는 입력이 필요할 수 있다.
- 확정·잠재: 구현 결함은 확인됐지만 현재 live data가 그 조합을 사용하지 않는다.
- 정책 필요: 현재 동작은 확인됐지만 의도된 게임 규칙인지 결정이 필요하다.

이번 감사에서는 코드, live data, 서버, Unity를 수정하지 않았다. 기존 worktree 변경은 사용자 소유로 보존했고 이 보고서 파일만 추가했다.

## 1. 총평

현재 상태를 릴리스 준비 완료로 보기는 어렵다.

확인된 항목은 Critical 1건, High 9건, Medium 5건이다. 이 중 14건은 코드 경로로 확정했고, 1건은 현재 동작은 확정됐지만 결함 여부를 확정하려면 gameplay 정책 결정이 필요하다.

가장 위험한 문제는 전투 event sequence와 battle time이 변하지 않는 pause, speed, VN 상태 update를 Unity가 이미 처리한 update로 오인해 폐기하는 것이다. 코어는 pause되어 후속 tick을 내보내지 않는데 Unity는 pause 전 상태에 머무를 수 있다.

그 밖에 다음 live integration 결함이 즉시 사용자 흐름을 막을 수 있다.

- 맵 생성기가 보장하는 Event 노드에 Unity route와 command가 없다.
- Unity 소비 아이템 command field가 Rust DTO와 다르다.
- HQ candidate 및 reward option identity를 Unity가 다른 field로 읽는다.
- checkpoint와 battle setup 복구가 코어에 있지만 ArkClient 명령과 UI가 없다.
- 비권위 debug 파일 출력 실패가 권위 전투 전이를 중단할 수 있다.

Core 전체 테스트는 통과했지만 game_server 전체 테스트는 6건 실패했다. 이 실패들은 필수 RunStart VN trigger가 fixture에 없어서 reconnect, tick, ordering 관련 본래 assertion에 도달하지 못한다.

## 2. 현재 구조와 source of truth

| 영역 | 현재 실질적 source of truth | 파생 또는 소비 계층 |
| --- | --- | --- |
| 명령 gate와 상태 전이 | GameCore::execute, GameCoreState, GameState, RunState | server actor, Unity root snapshot |
| 전투 진행 | ActiveBattleSession, BattleCore, BattleExecutionState | battle event log, checkpoint, CombatBattleState |
| 전투 영속 결과 | participant_results를 적용하는 world/combat 경로 | employee changes, result stats, rewards, battle record |
| 운영 데이터 | GameDataBase::load_live_embedded와 GameDataBase::new validation | 개별 RON 파일 |
| wire 계약 | Rust typed behavior와 snapshot DTO | game_server serialization, Unity readers와 stores |
| 문서 | 설명과 정책 보존 자료 | runtime 및 live data보다 후순위 |

주요 진입점:

- [GameCore 명령 gate](../../../src/game/world.rs#L82)
- [GameCore dispatch](../../../src/game/world.rs#L128)
- [live embedded loader](../../../src/game/data/mod.rs#L745)
- [GameDataBase validation](../../../src/game/data/mod.rs#L848)
- [game_server production loader 호출](../../../../game_server/src/main.rs#L103)
- [map node definition loader](../../../src/game/map/types.rs#L207)
- [map generation policy](../../../src/game/map/generator.rs#L61)

맵 노드 정의, 맵 생성 정책, battlefield archetype/template이 별도 loader에 있는 것은 그 자체로 중복 source of truth가 아니다. 문제는 이들을 배포 번들 전체로 대조하는 최종 의미 검증 단계가 없다는 점이다. 도메인별 소유권은 유지하고, 모든 live source를 읽은 뒤 교차 검증하는 seam을 추가하는 편이 적절하다.

## 3. 심각도 요약

| ID | 심각도 | 상태 | 요약 |
| --- | --- | --- | --- |
| C-01 | Critical | 확정·현재 | 동일 seq/time의 전투 제어 update를 Unity가 폐기한다. |
| H-01 | High | 확정 | 재배치한 동일 직원에게 결과, XP, trauma가 중복 적용될 수 있다. |
| H-02 | High | 확정 | 전멸 판정보다 retry와 cancel이 먼저 평가된다. |
| H-03 | High | 확정 | debug battle record 파일 출력 실패가 전투 상태 전이를 중단한다. |
| H-04 | High | 확정·현재 | 보장 생성되는 Event 노드를 Unity가 처리하지 못한다. |
| H-05 | High | 확정·현재 | 소비 아이템 요청 field가 Rust 계약과 다르다. |
| H-06 | High | 확정·현재 | HQ candidate와 reward option identity를 Unity가 읽지 못한다. |
| H-07 | High | 확정 | checkpoint 및 setup 복구가 코어에는 있지만 Unity 명령이 없다. |
| H-08 | High | 확정·잠재 | Event effect가 startup에서 검증되지 않고 일부 variant는 runtime panic이다. |
| H-09 | High | 정책 필요 | retreat가 이미 발생한 사상자까지 되돌린다. |
| M-01 | Medium | 확정 | 필수 RunStart VN invariant가 늦게 검사되고 server 테스트 6건이 실패한다. |
| M-02 | Medium | 확정·잠재 | extra wave와 map event reference의 bundle validation이 없다. |
| M-03 | Medium | 확정 | abnormality 기준이라고 문서화된 battle record가 실제로 node UUID 기준이다. |
| M-04 | Medium | 확정 | allowed_actions와 capability 소유자가 여러 군데다. |
| M-05 | Medium | 확정·잠재 | WholeField와 TileArea 조합이 validation을 통과하지만 runtime에서 효과가 없다. |

## 4. 상세 발견 사항

### C-01. 전투 제어 update의 독립 revision 부재

판정: Critical / 확정·현재

문제:

pause, resume, speed 변경은 battle event sequence나 battle time을 증가시키지 않는다. Core는 동일 seq/time의 checkpoint update를 내보낼 수 있지만 Unity BattleRuntimeState는 동일 seq에서 server battle time도 새롭지 않으면 checkpoint를 적용하기 전에 성공으로 반환한다.

영향:

- Core는 pause되어 다음 tick을 내보내지 않는데 Unity는 paused=false 상태로 남을 수 있다.
- speed 변경이 UI와 runtime에 반영되지 않을 수 있다.
- 동일 seq/time에 전달되는 battle VN scene 변경 또는 VN 종료 null projection도 폐기될 수 있다.
- blocking battle VN이 live content에 추가되면 진행 정지로 발전할 수 있다.

근거:

- [pause 처리](../../../src/game/world/combat.rs#L1127)
- [resume 처리](../../../src/game/world/combat.rs#L1145)
- [speed 처리](../../../src/game/world/combat.rs#L1163)
- [event가 없을 때 동일 to_seq를 만드는 update drain](../../../src/game/world/state.rs#L504)
- [update DTO 작성](../../../src/game/world/state.rs#L542)
- Unity: /mnt/f/unity projects/ark/Assets/Scripts/ArkClient/Runtime/Core/BattleRuntimeState.cs:115
- [battle VN update 작성](../../../src/game/world/visual_novel_flow.rs#L159)

테스트 상태:

- [Core pause test](../../../src/game/world/tests/combat.rs#L1996)는 pause 후 simulation이 멈추는 것만 보호한다.
- Unity ArkClientContractTests.cs:81은 동일 seq라도 time이 증가한 경우만 보호한다.
- 동일 seq, 동일 time, 변경된 checkpoint 또는 VN projection을 적용하는 테스트가 없다.

권고:

event sequence, simulation time과 독립적인 단조 증가 update_revision 또는 checkpoint_revision을 wire DTO에 추가한다. Unity는 event 중복 제거와 sideband state 적용을 서로 다른 revision으로 판단해야 한다.

범위와 위험:

Core DTO, server transport, Unity runtime과 reconnect test가 함께 바뀌는 중간 이상 범위다. wire migration 위험이 있으므로 임시 time 증가 같은 우회보다 명시적 revision이 안전하다.

### H-01. 재배치 instance와 직원 영속 결과의 혼합

판정: High / 확정

문제:

withdraw 또는 death 후 같은 직원을 재배치하면 새 RuntimeUnit instance가 생기고 이전 inactive instance도 결과에 남는다. 전투 종료는 모든 instance를 participant result로 출력하지만 XP, trauma, 직원 상태는 employee UUID 단위다.

영향:

- 동일 직원이 survival XP를 여러 번 받을 수 있다.
- employee_changes와 outcome row가 중복될 수 있다.
- 여러 번 쓰러진 직원에게 trauma가 반복 적용될 수 있다.
- survived, incapacitated, final HP, MVP 상태가 deployment chronology가 아니라 UUID 정렬상 마지막 instance에 의해 덮어써질 수 있다.

근거:

- [재배치가 새 instance를 만드는 테스트](../../../src/game/world/tests/combat.rs#L2933)
- [inactive instance를 보존하는 테스트](../../../src/game/world/tests/combat.rs#L3102)
- [모든 RuntimeUnit participant result 출력](../../../src/game/battle/core/sim.rs#L1116)
- [participant별 trauma와 XP 적용](../../../src/game/world/combat.rs#L451)
- [participant별 employee_changes 작성](../../../src/game/world/combat.rs#L571)
- [employee 통계 덮어쓰기](../../../src/game/battle/result_stats.rs#L154)
- [live survival XP](../../../../game_resources/data/run/policy.ron#L7)
- [live MVP survived weight](../../../../game_resources/data/run/policy.ron#L76)

테스트 상태:

현재 redeploy 테스트는 전투 결과 적용 전까지만 확인한다. 같은 employee UUID가 여러 participant instance로 결과에 들어올 때 persistent 상태를 한 번만 계산하는 테스트가 없다.

권고:

instance별 replay와 통계 데이터는 보존하되, 영속 결과는 employee UUID별 resolution으로 한 번 집계한다. 집계 규칙에는 deployment chronology, 최악 상태, 마지막 생존 instance, XP 지급 횟수를 명시한다.

범위와 위험:

중간 규모다. 성장, trauma, MVP 및 결과 UI 밸런스가 바뀔 수 있으므로 정책 테스트가 필요하다.

### H-02. 전멸 판정보다 retry와 cancel이 먼저 평가됨

판정: High / 확정

문제:

패배 participant outcome을 roster에 적용한 뒤 remaining attempts가 있으면 즉시 NodeConfirm으로 반환한다. NoLiving failure reason은 그 뒤에서 계산된다.

영향:

전원이 죽은 1회차 또는 2회차 패배에서도 RunFailed 대신 retry/cancel이 열릴 수 있다. Cancel은 active battle을 지우고 ViewingMap으로 돌아가므로 죽은 roster와 해결되지 않은 node만 남은 비정상 run이 만들어질 수 있다.

근거:

- [participant outcome 적용](../../../src/game/world/combat.rs#L1672)
- [remaining attempts 조기 반환](../../../src/game/world/combat.rs#L1684)
- [NoLiving failure 계산](../../../src/game/world/combat.rs#L1697)
- NodeConfirm의 Cancel 허용 당시 경로: `src/game/managers/action_scheduler.rs:53`
- [Cancel이 active battle을 제거하는 전이](../../../src/game/world/node_flow.rs#L207)
- [map progression selectability](../../../src/game/map/progression.rs#L168)

테스트 상태:

[현재 전멸 테스트](../../../src/game/world/tests/combat.rs#L3883)는 빈 participant_results를 사용해 실제 roster 사망 적용과 분기 순서를 보호하지 않는다.

권고:

loadable checkpoint를 통한 복구 가능성을 먼저 계산한 뒤 NoLiving과 NoDeployable terminal condition을 retry보다 우선 처리한다. terminal 상태에서는 Cancel을 노출하지 않는다.

범위와 위험:

작은 범위에서 중간 범위다. checkpoint 복구 예외 의미를 유지해야 한다.

### H-03. debug export I/O가 권위 전투 전이에 포함됨

판정: High / 확정

문제:

자연 전투 종료와 retreat 처리 도중 battle record 디렉터리 생성 및 파일 쓰기 오류를 gameplay 오류로 반환한다.

영향:

- 권한 부족, read-only 배포, 디스크 부족 때문에 완료된 battle이 InBattle에 머물 수 있다.
- retreat는 active battle을 제거한 뒤 export가 실패해 InBattle이지만 active session이 없는 상태가 될 수 있다.
- game_server tick handler는 오류를 push한 뒤 해당 tick을 중단한다.

근거:

- [자연 종료의 record 저장과 export](../../../src/game/world/combat.rs#L868)
- [record 저장과 dedupe](../../../src/game/world/state.rs#L748)
- [filesystem export](../../../src/game/world/state.rs#L763)
- [retreat의 선행 상태 제거 후 fallible export](../../../src/game/world/combat.rs#L1573)
- [server tick 오류 처리](../../../../game_server/src/game/player_game_actor/handlers.rs#L225)

테스트 상태:

성공적인 파일 출력만 확인하며 실패 sink를 주입하는 테스트가 없다.

권고:

gameplay commit 뒤 best-effort telemetry sink로 이동한다. export 실패는 log와 metric으로 남기되 authoritative state transition을 실패시키지 않는다.

범위와 위험:

구현 범위와 gameplay 회귀 위험 모두 낮다.

### H-04. live Event 노드의 Unity 진행 경로 부재

판정: High / 확정·현재

문제:

맵 생성기는 최소 하나의 Event node를 보장하고 live data도 Event definition과 story를 포함한다. Core에는 Event state와 command가 있지만 Unity router, screen, command bus에 해당 흐름이 없다.

영향:

Event node 진입 후 Unity가 Unknown 화면으로 떨어지고 AdvanceEventScene 또는 SelectEventChoice를 보낼 수 없어 run이 진행 불능이 된다.

근거:

- [Event node 최소 하나 보장](../../../src/game/map/generator.rs#L536)
- [Event invariant test](../../../src/game/map/generator.rs#L793)
- [live Event node definition](../../../../game_resources/data/map/node_definitions.ron#L103)
- [live Event story](../../../../game_resources/data/events/story/base.ron#L1)
- [Core Event state DTO](../../../src/game/behavior.rs#L744)
- [Core Event commands](../../../src/game/behavior.rs#L1018)
- Unity: /mnt/f/unity projects/ark/Assets/Scripts/ArkClient/Runtime/Core/GameScreenRouter.cs:91
- Unity: /mnt/f/unity projects/ark/Assets/Scripts/ArkClient/Runtime/Core/GameActionNames.cs:5

테스트 상태:

Core event flow test는 있으나 ArkClient의 live Event route, reader, presenter, command 종단 테스트가 없다.

권고:

실제 Rust snapshot JSON을 fixture로 사용해 Event route, model, presenter, advance/choice command를 하나의 feature slice로 구현한다.

범위와 위험:

Unity 중간 이상 규모다. 문서 예시가 아니라 실제 typed output을 기준으로 해야 한다.

### H-05. 소비 아이템 command field 불일치

판정: High / 확정·현재

문제:

Rust PlayerBehavior는 target_employee_uuid를 요구하지만 Unity는 employee_uuid를 전송한다.

영향:

Safezone 소비 아이템 사용은 Core 도달 전에 deserialization 실패한다. server schema 오류에는 request_id가 없어 Unity pending request는 기본 15초 timeout까지 남는다.

근거:

- [Rust UseConsumableItem 계약](../../../src/game/behavior.rs#L925)
- Unity: /mnt/f/unity projects/ark/Assets/Scripts/ArkClient/Runtime/Core/GameCommandBus.cs:118
- Unity canonical: /mnt/f/unity projects/ark/docs/unity_core_contract.md:1830
- [server가 request_id 없이 deserialization 오류를 보내는 경로](../../../../game_server/src/game/player_game_actor/session.rs#L363)
- Unity: /mnt/f/unity projects/ark/Assets/Scripts/ArkClient/Runtime/Core/PendingCommandRegistry.cs:122
- Unity: /mnt/f/unity projects/ark/Assets/Scripts/ArkClient/Runtime/Core/GameCommandSender.cs:8

테스트 상태:

Unity item test는 fake gateway만 검사하며 실제 serialized wire JSON을 확인하지 않는다.

권고:

Unity field를 target_employee_uuid로 수정하고 exact serialized payload test를 추가한다. server는 typed behavior 전체를 읽기 전에 envelope request_id를 추출해 schema 오류에도 correlation을 보존한다.

범위와 위험:

작은 범위다.

### H-06. HQ candidate와 reward option identity 불일치

판정: High / 확정·현재

문제:

Core HQ candidate는 id를, reward option은 instance uuid를 식별자로 내보낸다. Unity reader는 candidate_id와 reward_id만 읽는다.

영향:

HQ 모집 후보가 0명으로 보이고 ChooseOne reward option도 비어 node를 완료할 수 없다.

근거:

- [Core candidate id](../../../src/game/employee.rs#L39)
- [candidate snapshot mapping](../../../src/game/world/snapshot.rs#L1069)
- [reward option uuid](../../../src/game/behavior.rs#L694)
- [reward snapshot mapping](../../../src/game/world/snapshot.rs#L1122)
- Unity: /mnt/f/unity projects/ark/Assets/Scripts/ArkClient/Runtime/Core/Reading/SelectedEventModel.cs:63
- Unity: /mnt/f/unity projects/ark/Assets/Scripts/ArkClient/Runtime/Core/Reading/SelectedEventModel.cs:107
- Unity test: /mnt/f/unity projects/ark/Assets/Tests/PlayMode/ArkClientContractTests.cs:4013
- Unity가 실제 id/uuid shape를 legacy로 거부하는 test: /mnt/f/unity projects/ark/Assets/Tests/PlayMode/ArkClientContractTests.cs:4044

테스트 상태:

Unity 성공 fixture가 실제 Rust DTO에 없는 field를 만들어 사용한다.

권고:

Unity reader, fixture, 문서를 typed Rust output에 맞춘다. wire DTO를 별도로 도입한다면 Core와 Unity 양쪽에서 동일한 명시적 shape를 사용하고 장기 alias는 두지 않는다.

범위와 위험:

작은 범위에서 중간 범위다. 기존 잘못된 fixture가 수정된다.

### H-07. checkpoint와 setup recovery의 ArkClient command 부재

판정: High / 확정

문제:

Core는 loadable checkpoint가 있으면 no-deployable 상태를 즉시 RunFailed로 만들지 않는다. Checkpoint load와 battle setup loss recovery handler도 존재하지만 ArkClient에는 해당 action과 command sender가 없다.

영향:

Core가 recovery를 위해 failure를 보류한 상태에서 사용자가 recovery action을 실행하지 못해 run이 고립될 수 있다. reconnect actor의 자동 복구는 active client action을 대체하지 않는다.

근거:

- [checkpoint recovery 가능성 판단](../../../src/game/world/combat.rs#L731)
- [checkpoint 예외가 failure를 막는 경로](../../../src/game/world/combat.rs#L789)
- [defeat recovery 분기](../../../src/game/world/combat.rs#L1711)
- [checkpoint handler](../../../src/game/world/node_flow.rs#L716)
- Unity canonical: /mnt/f/unity projects/ark/docs/unity_core_contract.md:242
- Unity canonical command: /mnt/f/unity projects/ark/docs/unity_core_contract.md:1401
- Unity: /mnt/f/unity projects/ark/Assets/Scripts/ArkClient/Runtime/Core/GameActionNames.cs:5
- Unity probe: /mnt/f/unity projects/ark/docs/probe/unity_savepoint_checkpoint_probe.py

테스트 상태:

raw WebSocket probe는 있으나 현재 ArkClient UI와 command path를 보호하지 않는다.

권고:

LoadRunCheckpoint와 RecoverBattleSetupLoss를 서로 다른 command와 UI로 구현한다. reconnect auto recovery, resync, player-triggered recovery를 별도 테스트한다.

범위와 위험:

Unity 중간 규모다.

### H-08. Event RewardEffect startup validation 부재와 panic variant

판정: High / 확정·잠재

문제:

Event RewardEffect schema는 여러 효과를 authoring할 수 있지만 validation은 일부 immediate StartCombat reference에 집중돼 있다. ApplyBossOmenStepResult는 runtime에서 명시적으로 panic한다.

영향:

- 잘못된 reward, item, fragment, encounter reference가 server startup을 통과할 수 있다.
- node completion에서 늦게 실패해 이미 진행한 상태를 복구하기 어렵다.
- 미구현 variant를 live content에 작성하면 server process가 panic할 수 있다.

근거:

- [RewardEffect schema](../../../src/game/data/event_data.rs#L63)
- [ApplyBossOmenStepResult variant](../../../src/game/data/event_data.rs#L125)
- [제한적인 event validation](../../../src/game/data/event_data.rs#L350)
- [immediate encounter reference validation](../../../src/game/data/validation.rs#L757)
- [runtime missing grant 오류](../../../src/game/world/event_node.rs#L291)
- [runtime panic](../../../src/game/world/event_node.rs#L316)
- [staged path panic](../../../src/game/world/event_node.rs#L493)

테스트 상태:

runtime panic test는 있으나 startup rejection test가 없다. 현재 live story effect 목록은 비어 있어 지금은 직접 발화하지 않는다.

권고:

모든 Event 위치가 공유하는 RewardEffect semantic validator를 만든다. 미구현 variant는 구현 전까지 live schema에서 거부한다.

범위와 위험:

중간 규모이며 fixture 오류가 더 일찍 발견되는 방향이다.

### H-09. retreat가 이미 발생한 사상자를 되돌림

판정: High / 정책 필요

문제:

attempt가 남은 retreat는 active battle을 버리고 NodeConfirm으로 돌아가며 participant outcome을 적용하지 않는다. exhausted retreat도 빈 participant_results의 Draw를 만든다.

영향:

직원이 이미 0 HP가 된 뒤 BattleEnd 전에 retreat하면 trauma, run HP, injury를 회피할 수 있다. 이미 발생한 사상자를 유지해야 하는 정책이라면 balance exploit이다.

근거:

- [attempt가 남은 retreat](../../../src/game/world/combat.rs#L1551)
- [빈 participant 결과를 만드는 exhausted retreat](../../../src/game/world/combat.rs#L1564)
- [retreat helper](../../../src/game/world/combat.rs#L1597)
- [일반 participant persistent resolution](../../../src/game/world/combat.rs#L439)
- [retreat 규칙 설명](../../game_rulebook.md#L678)
- [retreat 자체는 incapacitation을 만들지 않는다는 설명](../../game_rulebook.md#L694)

테스트 상태:

사상자가 발생한 뒤 retreat하는 테스트가 없다.

권고:

이미 발생한 casualty를 영속할지 battle 전체를 rewind할지 먼저 결정한다. casualty 유지 정책이면 retreat도 participant outcome을 원자적으로 정산해야 한다.

범위와 위험:

중간 규모이며 gameplay와 balance 위험이 크다.

### M-01. 필수 RunStart VN invariant의 늦은 검사

판정: Medium / 확정

문제:

run reset state를 일부 변경한 뒤에야 RunStart trigger를 요구한다. VisualNovelTriggerData validation은 Standard와 Endless 각각 정확히 한 개의 RunStart trigger를 startup에서 보장하지 않는다.

영향:

잘못된 bundle이 server boot는 통과하지만 모든 신규 run 시작을 실패시킬 수 있다. 현재 server regression test 6건도 같은 fixture 누락 때문에 본래 assertion에 도달하지 못한다.

근거:

- [run start trigger를 늦게 요구하는 경로](../../../src/game/world.rs#L424)
- [trigger validation](../../../src/game/data/visual_novel_trigger_data.rs#L120)
- [현재 올바른 live trigger](../../../../game_resources/data/visual_novel_triggers/base.ron#L3)
- [trigger를 넣지 않는 server fixture builder](../../../../game_server/src/game/player_game_actor/handlers.rs#L435)

테스트 상태:

game_server cargo test --all-targets에서 25건 중 6건이 이 원인으로 실패한다.

권고:

live bundle validation에서 각 GameMode의 RunStart trigger를 정확히 하나 요구하고 blocking, repeat, checkpoint 정책도 검증한다. server fixture는 같은 공식 invariant builder를 사용한다.

범위와 위험:

작은 범위에서 중간 범위다.

### M-02. extra wave와 map event reference의 bundle validation 부재

판정: Medium / 확정·잠재

문제:

extra wave는 실제 stage에서 runtime에 추가되지만 startup preview validation은 floor 0 중심이다. map node definition은 event ID가 비어 있지 않은지만 확인하고 GameDataBase event 존재를 대조하지 않는다.

영향:

높은 floor에서만 사용되는 누락 wave preset 또는 잘못된 map event ID가 server startup을 통과한 뒤 panic 또는 unusable node로 나타날 수 있다.

근거:

- [extra wave schema](../../../src/game/data/run_policy_data.rs#L91)
- [run policy validation 단계](../../../src/game/data/run_policy_data.rs#L330)
- [startup preview의 floor 0 사용](../../../src/game/combat_preview/mod.rs#L475)
- [실제 stage extra wave append](../../../src/game/combat_preview/mod.rs#L416)
- missing wave preset panic 당시 경로: `src/game/battle/wave_resolution.rs:41`
- [map node definition validation](../../../src/game/map/types.rs#L229)

테스트 상태:

현재 live extra_waves는 비어 있고 현재 map event reference는 유효하다. 미래 콘텐츠 authoring에 대한 잠재 결함이다.

권고:

GameDataBase와 domain builtin을 모두 받은 validate_live_bundle seam에서 모든 distinct stage × mission/template과 map definition × event DB를 교차 검증한다.

범위와 위험:

중간 이상 validation 범위다. loader ownership을 합칠 필요는 없다.

### M-03. battle record identity 의미 불일치

판정: Medium / 확정

문제:

battle_uuid에 selected node UUID를 넣고 CombatBattleState에서는 이를 abnormality_uuid로 다시 저장한다. battle record dedupe도 이 field를 사용한다.

영향:

같은 abnormality가 다른 node에 나오면 여러 대표 record가 생기고, 같은 node retry는 하나로 합쳐진다. abnormality별 대표 record라는 문서와 실제 의미가 다르다.

근거:

- [battle UUID에 selected node UUID 할당](../../../src/game/world/combat.rs#L269)
- [active battle 저장](../../../src/game/world/combat.rs#L318)
- [CombatBattleState abnormality_uuid 구성](../../../src/game/world/combat.rs#L857)
- [record dedupe](../../../src/game/world/state.rs#L748)
- [durable contract의 abnormality별 설명](../../core_runtime_contract.md#L50)

테스트 상태:

현재 test는 실제 반복 abnormality가 아니라 node ID를 사용해 잘못된 의미를 고정한다.

권고:

battle_id, node_id, encounter_id, primary_abnormality_id를 분리하고 codex 대표 key를 별도 정책으로 정한다.

범위와 위험:

중간 규모 DTO와 저장 데이터 migration이다.

### M-04. allowed_actions와 capability의 중복 소유

판정: Medium / 확정

문제:

Core scheduler의 상태별 정적 목록, GameCore gate의 재계산, GameCoreState의 사실상 write-only action_validator, Unity의 active battle 정적 whitelist가 동시에 존재한다.

영향:

- Core snapshot이 명령을 실행 가능하다고 광고하지만 선택 node 또는 domain state 때문에 나중에 실패할 수 있다.
- Unity가 Core가 허용한 recovery 또는 VN action을 숨길 수 있다.
- blocking VN과 reconnect에서 UI와 server gate가 서로 다른 capability를 가진다.

근거:

- Core InNode 정적 action 목록 당시 경로: `src/game/managers/action_scheduler.rs:66`
- [GameCore execute gate](../../../src/game/world.rs#L82)
- Unity: /mnt/f/unity projects/ark/Assets/Scripts/ArkClient/Runtime/Core/GameSnapshotStore.cs:10
- Unity canonical: /mnt/f/unity projects/ark/docs/unity_core_contract.md:236

테스트 상태:

state string별 action 목록 test는 있으나 선택 node content, VN blocking, checkpoint recovery까지 포함한 executable capability 계약 test가 없다.

권고:

현재 context에서 계산되는 단일 capability projection을 Core가 생산하고 server가 전송하며 Unity가 그대로 소비하도록 정리한다. 미사용 validator와 static client whitelist를 제거한다.

범위와 위험:

중간 규모의 cross-layer refactor다.

### M-05. WholeField와 TileArea 조합의 runtime no-op

판정: Medium / 확정·잠재

문제:

validation은 WholeField target에서 defense_tile_range가 없는 TileArea effect를 허용하지만 runtime은 defense tile range가 없으면 빈 대상으로 step을 종료한다.

영향:

향후 작성한 whole-field AoE skill이 load 성공 후 아무 효과도 내지 않는다.

근거:

- [문서상 WholeFieldValidTiles와 TileArea 허용](../../skill_target_contract.md#L159)
- [validation 허용 분기](../../../src/game/data/validation.rs#L259)
- [runtime 빈 대상 처리](../../../src/game/battle/core/sim.rs#L2087)

테스트 상태:

현재 live RON에는 이 조합이 없다.

권고:

battlefield의 전체 유효 tile set을 실제로 구현하거나 구현 전까지 이 조합을 startup validation에서 거부한다.

범위와 위험:

중간 규모다.

## 5. 코드, 데이터, 서버, Unity 불일치

| 경계 | 생산 측 계약 | 소비 측 현재 동작 | 관련 항목 |
| --- | --- | --- | --- |
| Core → Unity battle update | seq가 늘지 않아도 checkpoint가 바뀔 수 있다. | 같은 seq/time이면 checkpoint 적용 전에 반환한다. | C-01 |
| Core/data → Unity Event | Event node와 command가 live 경로에 있다. | route, reader, command가 없다. | H-04 |
| Unity → Core item use | target_employee_uuid가 필수다. | employee_uuid를 전송한다. | H-05 |
| Core → Unity HQ | candidate id를 보낸다. | candidate_id만 읽는다. | H-06 |
| Core → Unity reward | option instance uuid를 보낸다. | reward_id만 읽는다. | H-06 |
| Core → Unity recovery | checkpoint와 setup recovery command가 있다. | ArkClient action과 sender가 없다. | H-07 |
| Data → runtime Event effect | schema에서 authoring할 수 있다. | 일부는 late error 또는 panic이다. | H-08 |
| Docs → runtime record | abnormality별 대표 record다. | node UUID 기준으로 dedupe한다. | M-03 |
| Docs → capability | allowed_actions는 실행 가능한 권위 목록이다. | Core와 Unity가 별도 정적 목록을 파생한다. | M-04 |

## 6. 테스트와 문서 신뢰도

### 실행한 명령과 결과

| 명령 | 결과 |
| --- | --- |
| cd /mnt/f/work/simulator/core_enhanced && cargo test --all-targets | 성공: 681 passed, 0 failed |
| cd /mnt/f/work/simulator/game_server && cargo test --all-targets | 실패: 19 passed, 6 failed |
| cd /mnt/f/work/simulator && cargo check -p game_server | 성공 |
| cargo test --test ron_loading -- --test-threads=1 | 성공: 18 passed |
| data, VN, run-policy, event, item-skill 대상 filter test | 모두 성공 |
| combat lifecycle, retry, redeploy 대상 test | 모두 성공 |

game_server의 6개 실패 범주:

- active battle attach와 recovery
- execute behavior snapshot
- finished battle snapshot ordering
- live tick push
- future sequence resync rejection
- paused tick suppression

모두 server fixture의 필수 RunStart trigger 누락으로 본래 assertion 전에 실패한다. 따라서 compile은 되지만 핵심 reconnect, tick, ordering regression suite는 현재 보호막 역할을 하지 못한다.

Unity test는 review-only 조건에서 project Library와 log를 변경하지 않기 위해 실행하지 않고 정적 검사했다.

확인된 test gap:

- 동일 seq, 동일 time의 checkpoint 또는 VN 변경을 검사하지 않는다.
- 소비 아이템은 실제 wire JSON serialization을 검사하지 않는다.
- HQ/reward fixture가 Rust DTO에 없는 field를 사용한다.
- 실제 Rust id/uuid shape를 legacy로 간주한다.
- Event, checkpoint load, setup recovery의 ArkClient 종단 test가 없다.
- redeploy test가 persistent result aggregation 전에서 끝난다.
- 실제 roster 전멸과 retry/cancel 순서를 검사하지 않는다.
- casualty 발생 후 retreat test가 없다.
- debug export failure injection이 없다.
- positive live RON test는 현재 bundle만 보호하며 missing-reference negative gate를 충분히 보호하지 않는다.

### 문서 신뢰도

[docs/README.md](../../README.md)는 runtime과 live data를 문서보다 우선한다고 올바르게 명시한다.

현재 구현보다 앞서 있거나 범위를 과대 표현하는 항목:

- Unity canonical의 Event UI와 command
- Unity canonical의 checkpoint action
- allowed_actions가 실제 executable capability라는 설명
- core_runtime_contract의 abnormality-keyed battle record 설명
- data_loading_contract가 주는 전체 reference startup validation 인상
- save_point_checkpoint_support goal의 ArkClient 적용 범위

완료 goal 문서는 historical evidence로는 유효하지만 현재 사용자 경로의 완성을 자동으로 의미하지 않는다. 특히 raw WebSocket probe 성공과 ArkClient UI/command 구현 완료는 구분해야 한다.

## 7. 즉시 수정 권고

1. C-01의 독립 update_revision을 도입하고 pause, resume, speed, battle VN의 Core → server → Unity 계약 test를 추가한다.
2. H-05 소비 아이템 field와 H-06 HQ/reward reader 및 fixture를 실제 Rust DTO에 맞춘다.
3. H-03 battle record filesystem export를 권위 상태 전이 밖으로 이동한다.
4. official RunStart invariant를 server fixture에 적용해 실패 중인 6개 test를 본래 assertion까지 복구한다.
5. H-02의 NoLiving과 NoDeployable 판정 순서를 retry/cancel보다 앞으로 옮기되 checkpoint recovery 예외를 명시적으로 보존한다.
6. Event와 checkpoint recovery를 임시 fallback 화면이 아니라 완결된 Unity feature slice로 구현한다.

## 8. 별도 refactor 목표

### 8.1 직원 단위 전투 결과 projection

목표:

- instance별 replay 정보를 보존한다.
- employee별 persistent resolution은 한 번만 계산한다.
- redeploy chronology, worst outcome, survival XP, trauma, MVP 정책을 명시한다.

완료 기준:

- 같은 employee UUID의 다중 RuntimeUnit을 포함한 victory, defeat, retreat test가 있다.
- employee state와 reward가 정책대로 정확히 한 번 적용된다.

### 8.2 전투 sideband update protocol

목표:

- event sequence
- simulation time
- checkpoint 또는 update revision
- visual novel projection
- executable capability

위 개념을 서로 다른 field와 ordering rule로 분리한다.

완료 기준:

pause, speed, VN, empty delta, reconnect, resync가 같은 protocol test matrix를 통과한다.

### 8.3 Unity node와 recovery feature 완성

목표:

- Event route와 presenter
- Event scene/choice command
- HQ와 reward identity
- LoadRunCheckpoint
- RecoverBattleSetupLoss

완료 기준:

실제 Rust serialized fixture와 live-like WebSocket flow를 사용하는 ArkClient test가 통과한다.

### 8.4 Live bundle semantic validation

목표:

- GameDataBase 내부 reference
- map node definition → event DB
- stage와 mission/template → extra wave
- Event RewardEffect reference
- RunStart trigger cardinality
- WholeField TileArea support

를 startup에서 교차 검증한다.

완료 기준:

잘못된 콘텐츠가 runtime 진입 전 경로와 ID를 포함한 오류로 거부된다.

### 8.5 Battle record identity와 codex key

목표:

battle attempt, node, encounter, abnormality identity를 분리하고 대표 record의 dedupe key를 명시한다.

완료 기준:

같은 abnormality의 여러 node, 같은 node의 retry, 여러 encounter variant test가 정책과 일치한다.

### 8.6 Capability source of truth

목표:

Core context-derived capability 한 개를 server와 Unity가 소비한다. 정적 client whitelist와 중복 validator를 제거한다.

완료 기준:

snapshot이 광고한 action은 같은 revision에서 실행 가능하며 blocking VN과 recovery 상태도 동일한 capability를 사용한다.

## 9. 권하지 않는 변경

- DTO mismatch를 serde alias 또는 Unity 이중 field로 장기간 숨기지 않는다.
- redeploy 결과를 고치기 위해 inactive RuntimeUnit을 삭제하지 않는다. Replay와 통계 정보가 손실된다.
- debug export 경계만 분리해야 하며 gameplay error 전체를 catch-all로 삼키지 않는다.
- Unity가 reward 또는 event identity를 display text나 배열 index에서 재구성하게 하지 않는다.
- domain loader를 하나의 거대 loader로 합치지 않는다. 소유권은 유지하고 bundle validation seam을 추가한다.
- 파일 크기만을 이유로 combat code를 분할하지 않는다.
- 현재 잘못된 Unity fixture를 compatibility contract로 승격하지 않는다.

## 10. 정책 결정 질문

1. 전투 도중 이미 발생한 death와 HP loss는 retreat 후에도 남아야 하는가?
   - 권고 기본값: 이미 발생한 결과는 유지한다.

2. 같은 직원이 여러 deployment instance에서 여러 번 쓰러지면 trauma는 battle당 한 번인가, death instance당 한 번인가?
   - 권고 기본값: employee당 battle 한 번, worst outcome을 사용하며 XP도 한 번만 지급한다.

3. setup-loss recovery가 battle attempt를 소비해야 하는가?
   - reconnect와 restore 기술 실패를 정상 패배와 같은 attempt로 취급하지 않도록 명시해야 한다.

4. Event OnAnyResolution reward는 attempt마다 지급하는가, node 전체에서 한 번 지급하는가?
   - 권고 기본값: retry reward exploit을 막기 위해 node 최종 이탈 시 한 번 지급한다.

5. battle codex 대표 key는 abnormality, encounter, node, 개별 battle attempt 중 무엇인가?

6. extra wave는 stage만으로 결정하는가, stage × mission/template 조합으로 결정하는가?
   - validator는 runtime과 같은 key를 사용해야 한다.

## 11. 우선순위 실행안

| 우선순위 | 작업 | 완료 gate |
| --- | --- | --- |
| P0 | C-01 update revision 도입 | 동일 seq/time pause, speed, VN state가 Unity에 적용되고 reconnect 뒤에도 일치한다. |
| P0 | H-05와 H-06 wire 수정 | 실제 Rust JSON fixture로 item, HQ, reward Unity test가 통과한다. |
| P0 | H-03 debug export 격리 | filesystem sink 실패에도 battle state와 server tick이 정상 진행된다. |
| P0 | M-01 startup invariant와 fixture 수정 | game_server cargo test --all-targets가 전부 통과한다. |
| P1 | H-01 employee result aggregation | 다중 instance의 XP, trauma, MVP가 정책대로 한 번 계산된다. |
| P1 | H-02와 H-09 lifecycle 정리 | 전멸, retry, cancel, retreat, checkpoint 조합 test가 통과한다. |
| P1 | H-04와 H-07 Unity 기능 | live Event와 checkpoint/setup recovery 종단 test가 통과한다. |
| P2 | H-08, M-02, M-05 validation 강화 | 잘못된 live content가 server startup 전에 거부된다. |
| P2 | M-03과 M-04 구조 정리 | identity와 capability가 계층별 하나의 명시적 source of truth를 가진다. |
| 마무리 | canonical 문서와 goal 상태 갱신 | 구현, live data, test와 문서가 동일한 계약을 말한다. |

## 후속 처리 원칙

- 이 보고서의 발견 사항을 수정할 때는 ID를 goal 또는 pull request에 연결한다.
- 정책 질문을 임의로 구현하지 않는다.
- 수정 완료 판정은 code change가 아니라 사용자-visible 또는 외부 contract test 통과를 기준으로 한다.
- 구현이 바뀌면 이 감사 보고서를 source of truth로 갱신하기보다 해당 durable canonical 문서를 갱신한다.
- 모든 항목이 처리된 뒤에도 이 파일은 2026-07-10 당시 상태를 기록한 historical audit snapshot으로 유지한다.
