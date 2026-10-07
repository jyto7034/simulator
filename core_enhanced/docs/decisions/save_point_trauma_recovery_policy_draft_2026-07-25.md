# SavePoint·Trauma 회복 및 출전 불가 실패 정책 초안 — 2026-07-25

- 논의·작성일: 2026-07-25
- 마지막 정책 검토: 2026-07-27
- 상태: **Policy closed / P00·P01 구현 완료 (2026-07-27)**
- 문서 등급: 구현 이력·정책 근거 문서
- 주의: 현재 runtime, live RON, save/wire 계약의 source of truth는 `archive/game_rulebook_legacy_2026-10-04.md`,
  `core_runtime_contract.md`, `data_loading_contract.md`와 외부 Unity canonical이다. 이 문서는 P00·P01이
  구현한 승인 정책의 역사적 입력으로 유지한다. 완료 근거는
  `../goals/run_policy_phase_00_savepoint_trauma_vertical_convergence/COMPLETION_REVIEW.md`와
  `../goals/run_policy_phase_01_failure_checkpoint_active_run_persistence/COMPLETION_REVIEW.md`에 있다.

이 논의에서 범위가 커 독립 goal로 분리한 정책은 다음 문서가 소유한다.

- 시설 맵 템플릿·노드 배정·연결 topology 전환:
  [facility_map_and_node_manifestation_policy_2026-07-25_26.md](facility_map_and_node_manifestation_policy_2026-07-25_26.md)
- 체크포인트 밖 로컬 영구 조우 도감 저장·복구:
  [local_encounter_compendium_persistence_policy_draft_2026-07-25.md](local_encounter_compendium_persistence_policy_draft_2026-07-25.md)
- Elite 전용 모노리스, 강화 재현, 떠돌이 상인 교환과 숨겨진 도감 보스:
  [elite_monolith_remanifestation_compendium_policy_draft_2026-07-26.md](elite_monolith_remanifestation_compendium_policy_draft_2026-07-26.md)

## 1. 목적

Run HP 제거 이후 Trauma만 회복하는 독립 Rest 노드는 선택의 깊이에 비해 노드 종류와 화면·명령
계약을 불필요하게 늘린다. 기존 Rest의 역할을 진행상의 명확한 이정표인 SavePoint와 Gate에
통합하고, Physical Damage는 향후 치료 아이템 경제가 담당하도록 장기 리스크의 책임을 분리한다.

이 초안은 다음을 결정한다.

- Support/Rest 레거시 제거와 SavePoint 노드 정식화
- SavePoint/Gate의 공통 Trauma 회복 표현과 기본값
- 전원 출전 불가 시 회복 가능성 기반 Run Failed 판정과 모든 실패 후보의 checkpoint 사전 선택
- Physical Damage 치료 아이템의 기존 준비 화면 drag-and-drop UX 재사용
- Core command 허용 상태와 Unity UI 노출 위치의 분리

### 1.1 최소 개입 원칙

Core와 Unity는 플레이어의 전략적 선택이 유용한지, 이후 어떤 기회를 잃는지 또는 어떤 순서가
최적인지를 미리 판단해 경고하거나 정답 행동을 추천하지 않는다. 현재 상태, command 사용 가능성,
선택한 행동의 직접 효과만 제공하고 행동 이후 authoritative state에서 deployability와 실패 조건을
다시 계산한다.

별도 확인이나 blocking UI는 checkpoint load처럼 되돌릴 수 없는 데이터 복원이나 Run Failed 확정 전
필수 선택처럼 데이터 무결성과 직접 관련된 경우에만 둔다. 일반적인
아이템 낭비, modifier 교체와 회복 경로 선택에는 추가 경고·확인·추천
projection을 만들지 않는다.

## 2. 노드 분류

정식 맵 노드 category는 다음 열 가지다.

1. `Start`
2. `Combat`
3. `SavePoint`
4. `Maintenance`
5. `HeadquartersContact`
6. `Event`
7. `Shop`
8. `Reward`
9. `Boss`
10. `Gate`

`Support` category와 `Rest` node/effect type은 제거한다. SavePoint는 Support 아래의 subtype이나
선택지가 아니라 독립된 단일 category다.

따라서 다음 레거시 표면도 함께 제거하는 것이 구현 완료 조건이다.

- SavePoint/Rest 중 하나를 고르는 support mode와 choices
- support effect 선택 command와 capability
- Support/Rest 전용 selected-event/result DTO 및 admin fixture 명칭
- Unity의 SupportChoice/Rest route와 화면
- 현재 SavePoint 완료에 결합된 신뢰도 `+8`, `RecoveredAtSavePoint` memory/event
- 구버전 Support/Rest 직렬화 alias와 fallback

`RecoveredAtSavePoint` memory, recent event, processed identity를 포함한 기존 save/checkpoint의
migration이나 dual-read는 제공하지 않는다. 해당 event kind, producer, projection과 테스트를 새
구조에 남기지 않고 함께 제거한다.

## 3. 영속 리스크의 책임

직원의 영속 condition authority는 계속 다음 두 값이다.

- `physical_damage: 0..6`: 부상 및 신체 손상 진행도
- `trauma: 0..100`: 정신적 손상 진행도

Run HP는 다시 도입하지 않는다.

- SavePoint와 Gate는 Trauma만 회복한다.
- SavePoint와 Gate는 Physical Damage나 사망 상태를 바꾸지 않는다.
- Physical Damage 회복은 authored Treatment 소비 아이템이 담당한다.
- Trauma mitigation 소비 아이템은 기존 Trauma를 회복하지 않고 이후 획득량만 완화한다.

## 4. 공통 Trauma 회복 표현

SavePoint와 Gate는 동일한 tagged recovery value를 사용하되 각각 독립된 설정을 가진다.

```text
TraumaRecovery::Flat { amount }
TraumaRecovery::CurrentPercent { percent }
```

한 recovery에는 두 방식 중 하나만 authoring한다. Flat과 CurrentPercent를 동시에 적용하거나 두
field를 병렬로 두지 않는다.

Recovery는 개별 node definition이나 map template가 아니라 category별 RunPolicy가 소유한다.
모든 SavePoint는 하나의 `save_point_trauma_recovery`, 모든 Gate는 하나의
`gate_trauma_recovery`를 사용하며 node별 override는 허용하지 않는다. 특수 node별 차등 회복은
실제 콘텐츠 요구가 생겼을 때 별도 정책으로 추가한다.

### 4.1 Flat

`Flat { amount: N }`은 현재 Trauma에서 `N`을 포화 뺄셈한다.

```text
after = current.saturating_sub(amount)
```

### 4.2 CurrentPercent

`CurrentPercent { percent: P }`는 현재 Trauma를 기준으로 `P%`를 계산해 올림한 값을 줄인다.
최대 Trauma 기준 percent는 Flat과 의미가 중복되므로 사용하지 않는다.

```text
recovery = ceil(current * percent / 100)
after = current.saturating_sub(recovery)
```

현재 Trauma가 0보다 크고 percent가 유효하면 최소 1을 회복한다. `amount`와 `percent`의 authored
범위는 각각 `1..=100`이며 0으로 recovery를 비활성화하지 않는다.

### 4.3 초기 live 기본값

- SavePoint: `CurrentPercent { percent: 15 }`
- Gate: `CurrentPercent { percent: 10 }`

Flat 방식은 schema와 runtime에서 함께 지원하되 초기 live data에는 사용하지 않는다. 실제 Flat
전환과 값 선택은 후속 콘텐츠 밸런싱이 소유한다.

## 5. SavePoint 완료 정책

SavePoint 완료는 하나의 원자적 결과로 다음을 수행한다.

1. SavePoint node 완료와 모든 생존 직원의 authored Trauma recovery를 정확히 한 번 stage한다.
2. planning-return에 따른 모든 pending 연구 지급을 같은 transaction에 stage한다.
3. node 완료, 회복, pending 지급과 planning 복귀가 모두 반영된 최종 run 상태로 단일
   체크포인트 payload를 만든다.
4. 위 변화를 함께 commit하고 Safezone/planning으로 복귀한다.

어느 단계든 실패하면 회복, pending 지급, node 완료, planning 복귀와 checkpoint 교체를 모두
적용하지 않는다. 체크포인트 payload를 planning-return 이전의 중간 상태로 만들지 않는다.

추가 규칙:

- Critical과 Breakdown 직원도 생존해 있으면 회복 대상이다.
- 사망 직원은 제외한다.
- Trauma가 100 아래로 내려간 Breakdown 직원은 별도 command 없이 Stable이 된다.
- SavePoint 완료는 신뢰도나 Psychology memory/event를 변경하지 않는다.
- Rest 제거 뒤 현재 production에는 `trust_score`를 바꾸는 producer가 남지 않는 것을 허용한다.
  신뢰도 변화를 SavePoint에 옮기지 않으며, 새 producer는 Event·직원 상호작용 등 적합한 콘텐츠
  정책이 생길 때 별도로 결정한다.
- 체크포인트는 회복 후 상태와 완료된 SavePoint를 저장하므로 load로 같은 회복을 반복할 수 없다.
- 기존 체크포인트를 유지·병합하거나 둘 중 하나를 선택하는 경로는 없다. SavePoint 완료는
  회복과 node 완료가 반영된 현재 payload로 기존 payload 전체를 교체한다.
- Pending 연구 지급은 기존 planning-return 계약을 그대로 따른다. SavePoint 진입은 지급
  authority가 아니며, 완료 후 planning으로 복귀할 때 모든 pending을 원자적으로 지급하고
  그 지급과 pending 제거가 반영된 inventory를 체크포인트에 저장한다.
- 기존 run당 단일 체크포인트와 최대 3회 load 정책은 유지한다. 새 SavePoint가 payload를
  덮어써도 `loads_used`는 초기화하지 않으며, 새 run을 시작할 때만 load 사용 횟수를 0으로
  초기화한다.
- `loads_used`는 checkpoint가 되돌릴 수 없는 run-scoped monotonic metadata다. checkpoint load는
  저장 당시의 더 작은 값으로 이 수치를 복원하지 않으며, 성공한 load마다 현재 값에서 정확히
  1 증가한다.
- Elite 모노리스의 구매 이력, 보유/사용 횟수, 환상체별 격파 단계, 변환된 Combat node와 남은 시도,
  잔여/변질된 모노리스, 조건부 Shop 보장과 교환 VN 상태는 일반 run-local payload에 포함해 load 시
  checkpoint 값으로 함께 복원한다. 반면 local persistent 도감의 발견/격파 set은 payload 밖에 있어
  load로 되돌리지 않는다.
- SavePoint는 대상이나 실효 Trauma가 없어도 정상 완료되고 체크포인트를 갱신한다.
- SavePoint preview는 기존 checkpoint를 현재 완료 결과로 교체한다는 직접 효과를 표시하되 별도
  overwrite 경고 modal이나 2차 확인을 만들지 않는다.

정상적인 수동 checkpoint load는 `ViewingMap`과 아직 node 진입이 commit되지 않은 Core
`NodeConfirm` 상태에서 허용한다. recovery restriction과 `AwaitingRunFailureDecision`에서도
허용하며, active node session, 전투, Event 진행, 결과 확인과 원자적 node/Floor transition
중에는 허용하지 않는다. 일반 수동 load는 checkpoint 이후 run-local 진행이 사라지고 횟수 하나를
소비한다는 단일 확인을 거친다. 실패 사전 선택 modal은 그 자체가 확인이므로 2차 확인을 추가하지
않는다. checkpoint 복원과 `loads_used + 1`은 하나의 원자적 transition이며 성공한 load에서만
횟수를 소비한다.

### 5.1 로컬 Active-Run 저장과 이어하기

SavePoint checkpoint는 Core 세션 안에서만 유지되는 임시 clone이 아니라, 종료되지 않은 run의
로컬 durable continue source다.

- SavePoint 완료 transaction은 회복, pending 연구 지급, node 완료와 planning 복귀가 모두 반영된
  최종 checkpoint payload를 versioned active-run envelope로 로컬 persistent-data 경로에 저장한다.
- 저장은 같은 디렉터리의 temporary file을 완전히 serialize·검증한 뒤 canonical active-run file로
  원자 교체하고, 직전 정상 canonical 한 세대를 rolling backup으로 유지한다.
- Durable publish가 실패하면 SavePoint 완료, 회복, pending 지급과 in-memory checkpoint 교체를
  성공으로 보고하지 않는다. 이전 정상 active-run file은 그대로 유지하고 오류를 보고한다.
- 파일은 마지막으로 **완료된 SavePoint** 상태만 소유한다. SavePoint 이후의 전투·node 선택·보상과
  `AwaitingRunFailureDecision` 같은 진행 중 상태를 일반 종료 시 별도 autosave하지 않는다.
- 유효한 active-run file이 있으면 다음 실행의 시작 화면에서 `이어하기`를 제공하고, 마지막
  SavePoint의 run/checkpoint state를 복원한다. SavePoint를 한 번도 완료하지 않은 run은 이어하기
  대상이 아니다.
- 이어하기는 명시적 `LoadRunCheckpoint`가 아니므로 최대 3회 load 횟수를 소비하지 않는다. 강제 종료로
  SavePoint 이후 진행을 되돌리는 비정상 플레이는 막지 않는다.
- `loads_used`와 checkpoint load로 되감기지 않는 run-scoped monotonic metadata는 checkpoint payload와
  분리해 active-run envelope에 함께 저장한다. 성공한 명시적 load는 payload를 복원하고 현재
  `loads_used`를 1 증가시킨 뒤 그 monotonic metadata를 durable file에도 반영한다. 재실행이나
  이어하기로 load 횟수를 초기화할 수 없다.
- 영구 조우 도감의 발견·격파 set은 active-run envelope에 넣지 않는다. 별도 local profile file이
  계속 소유하고 checkpoint load와 이어하기로 되돌리지 않는다.
- `RunComplete`, `ConfirmRunFailure`로 확정된 `RunFailed`, 명시적 `AbandonRun`은 active-run continue를
  원자적으로 무효화한다. 이후 시작 화면은 해당 run의 이어하기를 제공하지 않는다.
- 새 run이 유효한 active-run file을 대체하려면 사용자 확인을 한 번 받는다.
- Canonical file이 손상됐지만 backup이 정상이면 backup을 자동 복구하고 비차단 알림을 한 번
  표시한다. 둘 다 손상된 경우 원본을 보존하고 명시적 초기화 또는 종료를 선택하게 하며, 손상된
  run을 조용히 빈 상태나 새 run으로 대체하지 않는다.

## 6. Gate 완료 정책

Gate 확인과 Floor 전환은 하나의 원자적 결과로 다음을 수행한다.

1. Gate node 완료와 모든 생존 직원의 authored Trauma recovery를 정확히 한 번 stage한다.
2. planning-return에 따른 모든 pending 연구 지급을 stage한다.
3. 다음 Floor 생성과 진입을 stage한다.
4. 위 변화를 함께 commit한다.

다음 Floor 생성·validation을 포함한 어느 단계든 실패하면 Gate 완료, Trauma 회복, pending 지급과
Floor 전환을 모두 적용하지 않는다.

Critical과 Breakdown 대상 규칙, 사망 제외와 Physical Damage 불변 규칙은 SavePoint와 같다. Gate는
체크포인트가 아니며 Gate 확인 외의 화면 진입이나 연출로 회복을 적용하지 않는다.

## 7. 독립 맵 정책 경계

SavePoint 배치 주기, 시설 맵 템플릿, slot 기반 node 배정과 connection topology 전환은 이 정책의
구현 범위가 아니다. 해당 정책과 구현 완료 조건은
[facility_map_and_node_manifestation_policy_2026-07-25_26.md](facility_map_and_node_manifestation_policy_2026-07-25_26.md)가
독립적으로 소유한다.

## 8. Breakdown과 출전 가능성

Breakdown은 `trauma == 100`인 살아 있는 직원의 정신적 출전 불가 상태다. 사망이 아니며 Trauma가
100 아래로 회복되면 즉시 Stable로 돌아간다.

Combat, Event와 high-risk 활동에는 다음 조건을 만족하는 직원만 참여할 수 있다.

- `physical_damage: 0..=4`
- `trauma: 0..=99`

Critical(`physical_damage == 5`)과 Breakdown도 loadout과 모든 employee-targeted 소비 아이템의
대상이 될 수 있고, 살아 있다면 Trauma recovery 대상이다. 소비 아이템 사용 가능성은 현재
condition에서 그 효과가 유용한지와 분리한다. Dead(`physical_damage == 6`)는 loadout, 소비 아이템과
모든 recovery 대상에서 제외한다.

## 9. 출전 불가와 Run Failed 사전 선택

Core가 어떤 사유로든 Run Failed를 확정하려 할 때는 실제 `RunFailed` 전환보다 먼저 checkpoint
load 가능 여부를 검사한다. 특정 전투, 보스 또는 실패 사유에만 두는 예외가 아니라 모든 gameplay
Run Failed 후보에 동일하게 적용한다.

1. 실패를 일으킨 전투·노드 결과 transaction을 먼저 원자적으로 커밋하고 typed failure reason을
   계산한다. 전투·노드가 직접 만든 reason이 있으면 roster condition에서 파생된 reason보다
   primary로 유지하고 파생 condition은 보조 정보로 투영한다.
2. 현재 run에 속하고 구조적·의미적으로 검증되어 전체 복원이 가능한 Run checkpoint payload가
   있으며 `loads_used < 3`이면 `RunFailed`로 전환하지 않고
   `AwaitingRunFailureDecision { reason }`에 진입한다.
3. Unity는 이 authoritative state에서 checkpoint를 불러올지 묻고 남은 횟수를 표시한다.
4. 플레이어가 불러오기를 선택하면 기존 `LoadRunCheckpoint` command를 한 번 전송한다. 성공 시에만
   checkpoint 전체 상태 복원과 `loads_used + 1`을 원자적으로 commit한다.
5. 플레이어가 거절하면 새 `ConfirmRunFailure` command로 보관된 reason의 `RunFailed`를 확정한다.
   거절 자체는 load 횟수를 소비하지 않는다.
6. checkpoint payload가 없거나 load 횟수를 모두 사용했다면 중간 상태와 prompt 없이 즉시 해당
   reason의 `RunFailed`로 전환한다.

`AwaitingRunFailureDecision`은 해결된 node에서 `ViewingMap` planning으로 돌아온 상태가 아니다.
따라서 이 상태 진입은 pending 연구 지급 authority가 아니며, 실패 결과 때문에 새로 생긴 pending을
지급하거나 제거하지 않는다. Checkpoint를 불러오면 저장된 pending 상태를 포함한 payload 전체를
복원하고, 실패를 확정하면 미지급 pending과 함께 run이 종료된다.

전투·node가 직접 만든 typed failure reason이 primary다. 같은 결과에서 roster condition으로 파생된
`NoLivingEmployees` 또는 `NoDeployableEmployees`가 함께 성립하면 gameplay 전환을 다시 결정하지 않는
표시용 contributing reason으로만 투영한다.

Checkpoint metadata는 존재하지만 payload가 손상되었거나 복원 불가능하면 이를 단순한 checkpoint
부재로 숨기지 않고 system issue를 한 번 보고한다. 이 경우 decision state에서 load capability를
disabled reason과 함께 비활성화하고 `ConfirmRunFailure`는 허용해 플레이어를 막힌 상태에 두지
않는다.

`AwaitingRunFailureDecision` 중 gameplay mutation command는 `LoadRunCheckpoint`와
`ConfirmRunFailure`만 허용한다. load command가 실패하면 횟수를 소비하지 않고 같은 decision
state에 남아 최신 capability를 다시 투영한다. reconnect나 snapshot 재표시 뒤에도 Core state를
authority로 같은 선택을 복원하며, Unity의 로컬 prompt 표시 이력으로 수락·거절을 추론하지 않는다.

Unity의 선택창은 반드시 응답해야 하는 blocking modal이다.

- `체크포인트 불러오기 — 남은 횟수 N/3`은 `LoadRunCheckpoint`를 전송한다.
- `불러오지 않고 실패 확정`은 `ConfirmRunFailure`를 전송한다.
- 실패 reason과 복원 대상 checkpoint의 Floor, SavePoint/room 표시 이름을 함께 보여주고, 해당
  checkpoint 이후의 run-local 진행이 되돌아간다는 직접 결과를 표시한다. Unity가 이를 추론하지
  않도록 Core snapshot/projection이 안정된 identity와 표시용 metadata를 제공한다.
- 이 modal 자체를 load 확인 절차로 간주하며 `체크포인트 불러오기` 선택 뒤 같은 내용의 2차
  확인창을 추가하지 않는다.
- 창 밖 클릭, 닫기 버튼, ESC와 뒤로가기로 modal을 dismiss하지 않는다.
- command 처리 중에는 중복 입력을 막고, 실패 응답을 받으면 최신 snapshot/capability에 맞춰 같은
  modal을 다시 활성화한다.

한 번 load한 뒤 다시 실패 조건에 도달해도 payload가 유효하고 run 전체 load 횟수가 남아 있다면
같은 절차를 다시 거친다. 자발적인 run 포기, 정상 RunComplete와 protocol/system error는 gameplay
Run Failed 후보가 아니므로 이 절차의 대상이 아니다.

`ConfirmRunFailure`가 성공한 뒤의 `RunFailed`는 terminal이며 checkpoint load capability를 노출하지
않는다. 기존 강제 BossOmen `RunFailed`에서만 checkpoint load를 허용하던 예외는 제거하고 FinalBoss,
BossOmen capstone, 숨겨진 보스와 condition failure를 모두 이 공통 사전 선택 상태로 통합한다.

살아 있는 직원이 한 명도 없으면 현재 Floor의 HeadquartersContact와 실제 recruitment 후보가
있어도 recovery restriction을 만들지 않고 `NoLivingEmployees` 실패 후보를 만든 뒤 위 공통
절차를 적용한다. 전멸 뒤 신규 채용으로 run을 이어갈 수 없으며 checkpoint load만 과거 상태로
복귀할 수 있다.

살아 있는 직원은 있지만 출전 가능한 직원이 한 명도 없으면 즉시 실패시키지 않고 Core가 실제
회복 경로의 존재 여부를 내부 계산한다. 계산 범위는 현재 Floor의 준비 행동과 현재 progression에서
이미 즉시 선택 가능한 다음 node 최대 한 단계다.

회복 경로는 다음 authority만 사용할 수 있다.

- inventory에 있고 살아 있는 대상에게 지금 사용할 수 있으며 실제 condition 변화로 deployability에
  기여할 수 있는 즉시 효과 소비 아이템
- 즉시 선택 가능한 다음 SavePoint
- 즉시 선택 가능한 다음 Gate
- 즉시 선택 가능하고 command 비용·session 조건을 만족하며 채용 후 실제 출전 가능한 후보가 있는
  HeadquartersContact recruitment

Core는 현재 inventory에서 0회 이상의 즉시 효과 소비 아이템 사용과 현재 선택 가능한 recovery
node 최대 하나를 유효한 순서로 조합했을 때 최소 한 명이 출전 가능해지는지를 실제 효과 값과
command 가능성으로 계산한다. 단순 label, category 추측이나 아이템 보유 여부만으로 회복 가능하다고
판정하지 않는다. Gate 뒤 새 Floor에서 생성·해금될 node, 현재 잠긴 node, Shop offer, Reward, Event,
Maintenance와 그 밖의 미래 획득 가능성은 계산하지 않는다. Gate 자체의 직접 회복으로 출전 가능한
직원이 생길 때만 Gate를 회복 경로로 센다.

Core는 이 계산에서 찾은 아이템·대상·행동 순서, 최적 또는 최소 비용 경로를 snapshot/wire에
노출하지 않는다. Unity는 `출전 가능한 직원이 없지만 현재 Floor에서 회복 가능`이라는 restriction
사유와 각 행동의 기존 직접 preview만 표시하며 플레이어가 조합을 직접 판단한다.

예:

- 전원 사망이면 `NoLivingEmployees` 실패 후보가 되며, checkpoint load가 가능하면 실패 확정 전에
  선택을 제공
- 전원 Breakdown이고 다음 SavePoint/Gate recovery가 한 명 이상을 99 이하로 만들면 Run 유지
- 전원 Critical이고 사용할 수 있는 즉시 Physical Damage 감소 아이템이 없다면 SavePoint/Gate만으로는
  Physical Damage가 남으므로 실패 후보가 되고 공통 checkpoint 절차 적용
- 전원 Critical + Breakdown이어도 현재 즉시 회복 아이템 사용 후 다음 SavePoint/Gate로 한 명 이상이
  양쪽 조건을 벗어나면 Run 유지
- 실제 출전 가능한 후보가 있는 HeadquartersContact recruitment가 즉시 가능하면 Run 유지
- 남은 load 횟수가 있는 checkpoint가 존재하면 recovery restriction 중에도 load를 허용한다.
  Load 후에는 복원된 전체 상태에서 deployability와 현재 Floor 회복 경로를 다시 계산한다.

Checkpoint load 자체는 forward recovery path의 존재 근거로 세지 않는다. 유효한 즉시 효과·회복
node·채용 경로가 없으면 typed failure 후보를 만들고 공통 checkpoint 절차를 적용한다. 유효한
forward recovery path가 있으면 실패를 보류하고 recovery restriction을 건다.

restriction 중에는 다음만 허용한다.

- 남은 횟수를 소비하는 `LoadRunCheckpoint`
- 명시적인 자발적 `AbandonRun`
- 준비 화면에서 평상시 사용할 수 있는 모든 employee-targeted 소비 아이템 사용
- 현재 즉시 선택 가능한 SavePoint 또는 Gate 선택·완료
- 실제 recruitment 후보가 있는 HeadquartersContact 진입, 취소와 authored 선택 전체

Shop, Reward, Maintenance, Combat, Event 등 회복 경로에 포함하지 않는 node와 행동은 출전 가능한
직원이 다시 생길 때까지 허용하지 않는다. 각 허용 행동 후 Core가 같은 source of truth로
deployability와 현재 Floor 회복 경로를 다시 계산한다. 소비 아이템 낭비, 회복에 불필요한
SavePoint/Gate 사용, HeadquartersContact에서 recruitment를 거절하거나 다른 선택으로 node를
소비하는 것도 막거나 별도 경고하지 않는다. 그 결과 회복 경로가 사라지면 즉시 typed failure
후보와 공통 checkpoint 절차를 적용한다.

## 10. 소비 아이템 UX와 Physical Treatment

Physical Damage를 즉시 줄이는 Treatment와 이후 부상·Trauma·전투 결과를 완화하는 지속형 아이템은
별도 치료 화면이나 자동 소비 흐름을 만들지 않고 Unity 준비 화면의 기존 consumable UX를 그대로
사용한다. `사용 가능한가`는 구조적 command 조건이고 `현재 상태에서 유용한가`는 플레이어 판단이다.

- 우측 floating bag에서 consumable을 spatial employee에게 drag-and-drop한다.
- item click은 preview일 뿐 사용하지 않는다.
- employee click만으로 대상을 선택하거나 아이템을 사용하지 않는다.
- inventory identity와 quantity가 있고 item target policy에 맞는 살아 있는 직원에게 drop하면
  `use_consumable_item` command를 전송한다. Dead는 대상이 아니다.
- Core는 inventory identity/quantity, target 존재·생존, target policy와 authored effect만
  authoritative하게 검증한다. 현재 condition에서 실효 이득이 있는지를 eligibility나 capability로
  사용 금지 사유로 만들지 않는다.
- 즉시 Physical Damage 감소 효과는 포화 뺄셈한다. 현재 Physical Damage가 0이거나 authored
  회복량이 남아도 아이템은 정상 소비하며 별도 낭비 경고를 표시하지 않는다.
- 부상 방지, Trauma mitigation과 전투 modifier는 현재 condition과 무관하게 적용할 수 있지만
  기존 Trauma나 Physical Damage를 소급 회복하지 않는다.
- 직원은 지속형 consumable modifier를 최대 하나 가진다. 같은 definition을 다시 사용하면 기존
  modifier를 authored 초기 duration으로 갱신하고, 다른 지속형 consumable을 사용하면 기존 modifier를
  제거하고 새 modifier로 교체한다. 교체를 막거나 별도 확인 modal을 만들지 않는다.
- 즉시 Physical Damage 감소 아이템은 지속형 modifier slot을 사용하거나 기존 modifier를 교체하지
  않는다.
- consumable definition 하나는 우선 단일 authored effect만 가진다. 여러 효과를 합친 복합 아이템은
  실제 콘텐츠 요구가 생겼을 때 별도 schema와 합성 규칙을 추가한다.
- 성공한 command는 아이템을 즉시 소비하고 condition, active modifier와 deployability를 다시
  투영한다. 자동 소비하지 않으며 대상과 아이템은 플레이어가 직접 선택한다.

현재 live data에 즉시 Physical Treatment 아이템이 없으므로 구체 아이템, 획득처, 가격과 회복량은
후속 콘텐츠 authoring 범위다.

## 11. NodeConfirm과 command 허용 범위

Unity UI 노출 위치와 Core command 허용 상태를 분리한다.

- Unity의 일반 플레이어 UX는 inventory가 보이는 준비 화면에서만 consumable drag-and-drop을
  노출한다.
- Core `NodeConfirm` 상태를 표시하는 Node Exploration map의 preview panel에는 inventory나 아이템
  사용 control을 추가하지 않는다.
- Core는 기존처럼 `ViewingMap`과 `NodeConfirm`에서 `use_consumable_item`을 허용한다.
- NodeConfirm이라는 이유만으로 Core validation이나 capability에서 아이템 사용을 금지하지 않는다.
- 일반적인 Unity 접근에서 NodeConfirm 사용이 불가능한 것은 UI affordance가 없기 때문이며 gameplay
  금지 규칙이 아니다.
- command가 도달하면 Core는 다른 허용 상태와 동일하게 authoritative validation을 수행한다.

`NodeConfirm`은 별도 Unity screen/route가 아니라 node click 뒤 같은 Node Exploration map에서
preview가 준비된 Core 상태다. 다른 진입 가능 node를 선택하면 preview를 교체할 수 있고 진입 전
취소는 node를 소비하지 않는다. SavePoint와 Gate의 preview panel에는 Core가 계산한 다음 직접
효과를 노출한다.

- recovery mode와 authored value
- 대상 직원별 현재 Trauma와 예상 Trauma
- 회복 후 Stable/Breakdown 상태
- Physical Damage는 변하지 않는다는 설명

Unity는 percent 계산이나 eligibility를 재구현하지 않고 Core projection을 표시한다.

## 12. 구현 전 동기화 대상

이 초안을 구현하기 전 별도 goal/plan에서 최소 다음 표면을 함께 감사하고 갱신한다.

- `archive/game_rulebook_legacy_2026-10-04.md`: 노드 목록, Safe Node, SavePoint/Gate, condition과 failure 규칙
- `core_runtime_contract.md`: completion atomicity, checkpoint, recovery restriction,
  `AwaitingRunFailureDecision`과 capability
- `data_loading_contract.md`: tagged Trauma recovery schema와 validation
- live run-policy/node definition RON: SavePoint category, category 공통 recovery 기본값과
  node/template override 금지
- Core map/session/result/snapshot/admin command와 save/checkpoint schema
- host/game server의 versioned active-run bootstrap, atomic current/backup 저장, 이어하기와 terminal
  invalidation
- 외부 Unity `unity_core_contract.md`: category/payload/result/snapshot/preview 계약
- 외부 Unity `docs/ui/screen_spec.md`: SavePoint/Gate preview, 기존 준비 drag-and-drop의 즉시
  Treatment·단일 지속 modifier 교체, modifier 교체 확인 modal 제거, 모든 Run Failed 후보의
  checkpoint 사전 선택 prompt
- Unity route/composer/mock/test: SupportChoice/Rest와 별도 NodeConfirm route 제거, Node Exploration
  map의 SavePoint/Gate preview 통합
- game server deserialize/result mapping과 non-combat/admin probe

## 13. 이 초안의 비목표

- Physical Treatment live 아이템의 이름, 획득처, 가격과 수치 확정
- SavePoint/Gate Flat 값으로의 실제 밸런스 전환
- SavePoint/Gate의 최종 아트, 연출과 사운드
- Run HP 재도입
- 현재 run state에서 사망 직원을 되살리는 resurrection/Treatment. checkpoint의 과거 전체 상태
  복원은 여기에 포함하지 않는다.
- NodeConfirm inventory UI 추가

## 14. 승인 후 완료 조건

이 초안이 최종 확인되면 다음 순서로 진행한다.

1. 영향 감사와 구현 goal을 작성한다.
2. durable canonical 문서와 외부 Unity 계약을 먼저 같은 정책으로 갱신한다.
3. Core schema/runtime/live data를 변경한다.
4. game server와 Unity reader/router/presentation을 변경한다.
5. active-run current/backup 저장, 이어하기, monotonic `loads_used`, 손상 복구와 terminal invalidation을
   구현한다.
6. focused Core test, strict live loading, host persistence, server mapping/probe와 Unity
   EditMode/PlayMode를 검증한다.
7. 잔여 `Support`, `Rest`, 선택형 support, BossOmen post-`RunFailed` load 예외와 stale 문구가 없는지
   감사한다.
8. 완료 review 뒤 이 문서를 historical decision input으로 전환한다.

## 15. 다른 작업자에게 넘길 때의 기준

이 문서의 2~11절은 이번 논의에서 승인된 target policy다. 현재 runtime, live RON과 기존 canonical이
다르게 동작하는 부분은 이 결정을 무효화하는 근거가 아니라 구현 시 함께 전환해야 할 current-state
차이다. 오래된 goal/experiment 문서는 당시 이력이며 이 문서보다 새로운 정책 source로 사용하지
않는다.

구현 작업자는 다음 경계를 유지한다.

- SavePoint·Gate 회복, checkpoint overwrite, pending 연구 planning-return 지급과 node completion을
  부분 commit으로 나누지 않는다.
- Support/Rest, 선택형 support, SavePoint 신뢰도/memory와 구 save alias를 새 구조에 남기지 않는다.
- run당 load 3회, monotonic `loads_used`, 모든 gameplay Run Failed 후보의 사전 load 선택과 명시적
  실패 확정을 하나의 상태·capability 계약으로 구현한다.
- Recovery restriction은 현재 Floor의 보유 자원과 즉시 선택 가능한 node만으로 존재 여부를
  계산하고 구체 경로를 wire에 공개하지 않는다. 전략적 유용성, 아이템 낭비, modifier 교체와
  회복 경로 선택을 이유로 별도 경고·확인·추천 계약을 만들지 않는다.
- 소비 아이템 사용 가능성은 현재 condition에서의 유용성과 분리하고, 살아 있는 target에게
  구조적으로 유효한 사용을 허용한다. 지속형 modifier는 직원당 하나이며 새 지속형 item이 기존
  modifier를 교체한다.

정책 검토 결과 현재 정책을 막는 추가 gameplay 또는 persistence 결정은 없다. Treatment 아이템의
실제 콘텐츠 수치, Flat 전환 밸런스와 최종 아트·연출·사운드는 명시적인 후속 콘텐츠 범위다. 구현
감사에서 이 문서의 의미를 바꿔야만 해결할 수 있는 충돌이 새로 확인될 때만 사용자 결정을 다시
요청한다.
