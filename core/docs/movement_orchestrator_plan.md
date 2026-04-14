# Movement Orchestrator Plan

## 목적

이 문서는 현재 `core` 전투 이동 시스템을 장기적으로 교체하기 위한 `MovementOrchestrator` 설계 계획서다.

대상 독자는 다음과 같다.

- 현재 코드베이스를 처음 읽는 다른 AI
- 기존 movement 시스템을 유지보수하던 개발자
- 전투 리플레이/타임라인 회귀를 검증해야 하는 개발자

이 문서는 구현 코드가 아니라, 리팩토링 방향과 설계 의도를 고정하기 위한 문서다.

## 한 줄 요약

현재 movement 시스템은 `유닛별 순차 계획 + 예약 충돌 + wait_repath` 모델이다.

장기 방향은 이를 `한 시각의 이동 후보를 전역적으로 수집 -> 충돌 조정 -> 최종 커밋`하는 `MovementOrchestrator` 모델로 교체하는 것이다.

전투 감각 기준은 "rigid lane 유지"가 아니라, TFT에 가까운 다음 규칙이다.

- 근/원거리 모두 기본 타겟은 `현재 위치 기준 가장 가까운 적`
- 공격 중인 타겟은 쉽게 바꾸지 않는 `sticky target`
- 근접 유닛이 현재 타일이나 현재 이동 세그먼트에서 이미 사실상 교전 가능하면,
  더 좋은 빈 공격 타일을 찾기보다 즉시 `Engage`를 우선
- 아군 혼잡이나 path 막힘이 생기면 짧게 기다린 뒤 적극적으로 `reposition`
- `거의 붙기 직전`에는 타겟을 바꾸지 않는 `near-engage lock`
- `WaitRepath`는 예외 상태로만 남기고, 정상 흐름은 `Hold/Yield/Blocked`로 흡수

핵심 목표는 다음이다.

- 근/원거리 모두 `nearest enemy` 기준으로 자연스럽게 타겟을 잡게 하기
- 공격 중에는 쉽게 흔들리지 않되, 재배치 중에는 더 자유롭게 갈아타게 하기
- 아군 혼잡과 외길 상황에서 멍하게 멈추기보다 짧은 wait 후 재배치하게 하기
- `wait_repath`를 예외로 축소하고 정상적인 `yield/hold/blocked` 상태를 모델에 포함하기
- 타일 기반 점유/결정론은 유지하기

## 현재 코드 구조

이동 관련 핵심 파일:

- [movement/mod.rs](/mnt/f/work/simulator/core/src/game/battle/core/movement/mod.rs)
- [movement/plan.rs](/mnt/f/work/simulator/core/src/game/battle/core/movement/plan.rs)
- [movement/execute.rs](/mnt/f/work/simulator/core/src/game/battle/core/movement/execute.rs)
- [sim.rs](/mnt/f/work/simulator/core/src/game/battle/core/sim.rs)

현재 구조의 역할:

- `movement/plan.rs`
  - 타겟 선정
  - 사거리 내 타겟 확인
  - BFS 생성
  - 목적지 후보 계산
  - 첫 스텝 예약
- `movement/execute.rs`
  - `MoveStep` 소비
  - 연속 좌표 적분
  - 타일 점유 전환
  - 예약 충돌 처리
  - `wait_repath` 진입
  - 근접 사거리 진입 시 이동 중단
- `movement/mod.rs`
  - 타일 중심/경계 좌표 계산
  - 이동 상태 타입
- `sim.rs`
  - 이벤트 루프에서 movement 관련 이벤트를 호출

## 현재 모델 요약

현재 movement 시스템은 다음 모델에 가깝다.

1. 유닛을 `uuid` 순으로 하나씩 본다.
2. 각 유닛이 현재 타겟과 사거리를 확인한다.
3. 사거리 안에 적이 없으면 BFS를 만들고 목적지 후보를 만든다.
4. 가능한 첫 스텝을 예약한다.
5. 이후 `MoveStep` 시각에 실제 이동을 적용한다.
6. 충돌이나 점유 문제를 만나면 `wait_repath`로 들어간다.

즉, `집단 이동 계획`이 아니라 `유닛별 로컬 계획 + 사후 충돌 해소` 모델이다.

## TFT 스타일 명세

이 문서가 목표로 하는 TFT 스타일 movement/targeting 규칙은 아래와 같다.

### 1. 공통 타겟 규칙

- 근/원거리 모두 기본 타겟은 `현재 위치 기준 nearest enemy`
- 거리 차이가 사실상 없으면 deterministic tie-break 사용
- 공격 중에는 현재 타겟을 유지
- 현재 타겟이 죽었거나 더 이상 때릴 수 없을 때만 nearest fallback

### 2. 근접 engage 우선 규칙

- 근접 유닛은 "빈 공격 타일 찾기"보다 "얼마나 빨리 실제 공격을 시작할 수 있는가"를 우선해야 한다
- 현재 타일에서 이미 melee engage 가능하면 lateral reposition보다 즉시 `Engage`
- 현재 이동 세그먼트 중간에서 melee reach에 들어오면, 다음 측면 타일 후보를 끝까지 밟기보다 거기서 `target_acquired`
- 즉 근접 유닛은 `attack tile optimizer`가 아니라 `engage optimizer`처럼 보여야 한다

### 3. sticky target / sticky approach

- `sticky target`은 유지한다
- 동시에 현재 타겟에게 붙고 있던 접근 축도 가능한 한 유지한다
- 현재 타겟이 살아 있고 거의 붙기 직전이면, 더 좋은 측면 attack tile이 보여도 lateral switch를 억제한다
- 현재 타겟이 죽었거나 장시간 공격 불가일 때만 접근 축을 크게 바꾼다

### 4. 재배치 규칙

- 아군 혼잡이나 lane 막힘은 즉시 `WaitRepath`로 보내지 않는다
- 짧은 wait 후 재평가하고, 여전히 답이 없으면 적극적으로 reposition
- 다만 engage 직전 lateral wiggle은 허용 폭이 작아야 한다
- 즉 `mid-fight 재참여는 자유롭게`, `near-engage 직전 흔들림은 보수적으로`가 목표다

### 5. 원거리 규칙

- 원거리도 nearest enemy 기준으로 타겟을 잡는다
- 사거리 안에 들어오면 즉시 정지 후 공격
- 카이팅은 하지 않는다
- 전선 붕괴 후 가까운 적이 붙으면 그대로 nearest rule로 처리한다

## 현재 문제가 생기는 이유

대표 사례는 `timeline_exports/tft_like_field_6v6_mixed_melee_ranged_battle.json` 이다.

문제 관찰:

- 근접 유닛이 처음에는 직진한다.
- 일정 시점부터 논리 타일은 직진인데, `target_acquired` 직전 연속 좌표가 옆으로 밀린다.
- 그 결과 시각적으로 "잘 가다가 갑자기 옆으로 꺾었다"처럼 보인다.

대표 사례:

- `65dbace7-8e6d-4875-b9ae-73ce79dd7425`
- `9323b27e-6cf6-44ea-9ea4-d98a9cc9caad`
- `9970ce67-9a30-46f0-a132-27df729b3414`

### 실제 원인

원인은 단일 버그가 아니라, 현재 모델의 구조적 특성이다.

1. 계획이 유닛별 순차 계산이다.
   - 먼저 처리된 유닛의 예약이 뒤 유닛의 BFS와 후보 선택에 영향을 준다.

2. 근접 목적지 후보가 "적 주변의 빈 타일" 중심이다.
   - 현재 위치 유지나 직진 유지보다, 새로운 빈 공격 타일 재선정이 먼저 일어난다.

3. tie-break가 좌우 대칭 상황에서 한쪽으로 쏠릴 수 있다.
   - `compare_destination_preference()`와 `compare_plan_preference()`의 마지막 tie-break는 `x` 작은 쪽을 선호한다.

4. 근접 교전 시작 판단보다 "빈 공격 타일 후보"가 앞설 수 있다.
   - 현재 타일에서 사실상 melee engage 가능한 상태여도,
     planner는 더 좋은 빈 공격 타일을 찾으려 한다.

5. 이동 목표가 타일 중심이 아니라 경계(`boundary_target_units`)다.
   - 그래서 다음 스텝이 측면으로 잡혀 있으면,
     continuous `RangeEnter` 정지가 그 lateral step 중간에서 발생할 수 있다.
   - 논리 타일은 직진인데, 연속 좌표는 옆 타일 경계 쪽으로 움직여 보인다.

6. 충돌 해소가 사후적이다.
   - 계획 단계에서 해결하지 못한 것을 실행 단계에서 `wait_repath`로 밀어낸다.

### 관련 코드 포인트

- 유닛별 순차 계획:
  - [movement/plan.rs](/mnt/f/work/simulator/core/src/game/battle/core/movement/plan.rs)
- 목적지 후보 정렬:
  - `compare_destination_preference`
  - `compare_plan_preference`
- 적 주변 빈 타일 후보 생성:
  - `destination_candidates_in_order`
- 첫 스텝 예약:
  - `compute_movement_intents`
- 충돌 시 `wait_repath`:
  - `handle_move_steps_at`

## 왜 부분 보정이 아니라 구조 교체가 필요한가

다음 수정은 임시방편이다.

- `repath` 딜레이만 늘리기
- 좌표 보간만 바꾸기
- 특정 케이스에서만 대각선 후보를 제거하기
- Unity 리플레이에서만 위치를 스냅하기

이유:

- 문제는 렌더링이 아니라 서버 타임라인에 이미 기록된다.
- 이동 의사결정이 유닛별로 흩어져 있고, 충돌 해소가 사후적으로 일어난다.
- tie-break 한두 개만 바꾸면 다른 모양의 쏠림이 다시 생길 수 있다.

따라서 장기적으로는 `MovementOrchestrator`를 도입해 `계획 -> 조정 -> 커밋` 모델로 바꿔야 한다.

## 제안하는 최종 아키텍처

### 핵심 개념

`MovementOrchestrator`는 "각 유닛이 바로 예약하고 이동하는 시스템"이 아니다.

역할은 다음과 같다.

1. 현재 시각의 이동 의도를 전부 수집한다.
2. 각 유닛별 후보 스텝을 여러 개 만든다.
3. 충돌을 전역적으로 조정한다.
4. 승인된 이동만 커밋한다.

즉, movement의 주체를 `개별 유닛`에서 `전역 coordinator`로 옮긴다.

### 유지할 것

- 타일 기반 점유 모델
- 결정론
- `Battlefield` 중심의 occupancy/reservation 개념
- 연속 좌표 기반 movement/timeline export
- 근접 `RangeEnter` 중단 개념

### 바꿀 것

- 유닛별 순차 계획
- 즉시 예약 방식
- `wait_repath` 중심 실패 모델
- 적 주변 빈 타일을 바로 확정하는 방식

## 새 시스템의 책임 분리

### 1. Intent Gathering

입력:

- 현재 시각
- 유닛 상태
- 타일 점유/예약 스냅샷
- 공격 정책
- 현재 타겟

출력:

- 유닛별 이동 의도

예시 의도:

- `Engage`
- `Advance`
- `Hold`
- `Yield`
- `Blocked`

### 2. Candidate Generation

기존 시스템은 유닛 하나당 사실상 "베스트 목적지 1개"를 빠르게 선택한다.

새 시스템은 유닛 하나당 우선순위 있는 후보 리스트를 만든다.

핵심은 "빈 공격 타일 집합"보다
`현재 타겟에게 얼마나 빨리 실제 공격을 시작할 수 있는가`와
`지금이 sticky target / sticky approach를 유지해야 하는 상태인가`를 함께 보는 것이다.

후보 생성은 최소한 다음 축을 반영해야 한다.

1. 현재 위치에서 이미 사거리 충족 시 `Engage`
2. 현재 이동 세그먼트 중간에서 melee reach에 들어오면 `Engage`
3. 현재 타겟 유지
4. 현재 타겟에게 가는 더 짧은 path
5. 이미 거의 붙은 상태라면 lateral attack tile보다 current-approach 유지
6. 아군 혼잡을 풀기 위한 작은 재배치
7. 현재 타겟이 장시간 막혀 있을 때의 retarget

즉 "교전선 슬롯을 고정"하는 것보다,
"nearest target + sticky target + sticky approach + 자유로운 재배치"를 우선하는 구조여야 한다.

### 3. Conflict Resolution

이 단계가 현재 시스템에 사실상 없는 부분이다.

해결해야 하는 것:

- 같은 타일 경쟁
- 같은 tick 동시 진입
- soft/hard reservation 충돌
- 정면 마주보기 충돌
- 좌우 대칭 분산
- 짧은 wait 후 재배치

출력:

- 승인된 이동
- 양보(`Yield`)
- 유지(`Hold`)
- 일시 차단(`Blocked`)

### 4. Commit

이 단계에서만 실제 부수효과를 적용한다.

- `battlefield.move_unit()`
- reservation 적용/취소
- `MovementStopped`
- `UnitMoved`
- 다음 `MoveStep` 예약

즉, 계획과 적용을 분리한다.

## 새 상태 모델 제안

현재 상태:

- `Idle`
- `Moving`
- `WaitRepath`
- `HardCCLocked`
- `Dead`

장기적으로는 다음 상태를 고려한다.

- `Idle`
- `MovingSegment`
- `Holding`
- `Yielding`
- `Blocked`
- `HardCCLocked`
- `Dead`

### 상태 의미

- `Holding`
  - 짧은 혼잡이나 near-engage 상태 때문에 지금은 유지하는 편이 더 좋음
- `Yielding`
  - 이번 조정 턴에서 전역 우선순위 때문에 양보
- `Blocked`
  - 타겟은 유효하지만 path나 공격 슬롯이 잠깐 막힘

`WaitRepath`는 장기적으로 "정말 계획이 깨졌을 때만 쓰는 예외"로 축소하는 것이 목표다.

## 새 입력/출력 타입 제안

### 입력 스냅샷

추천 타입:

- `MovementFrame`
- `UnitMoveContext`
- `OccupancySnapshot`
- `EngagementSnapshot`

필요한 필드:

- 현재 타일 위치
- 연속 좌표
- 현재 이동 세그먼트
- 현재 타겟
- owner
- basic attack range policy
- move speed
- action lock / hard cc
- battlefield occupancy/reservation 상태

### 출력 결정 타입

추천 타입:

- `MoveDecision::Advance`
- `MoveDecision::Engage`
- `MoveDecision::Hold`
- `MoveDecision::Yield`
- `MoveDecision::Blocked`

예시:

- `Advance { unit, from, to, segment_end }`
- `Engage { unit, target, stop_at_ms }`
- `Hold { unit }`
- `Yield { unit, until_ms }`
- `Blocked { unit, retry_at_ms }`

## 타겟 라이프사이클과 재배치 규칙

현재 movement를 TFT 느낌으로 바꾸려면, 핵심은 lane 고정보다 타겟 라이프사이클을 분명하게 만드는 것이다.

### 1. 타겟 획득

- 근/원거리 모두 기본 타겟은 `현재 위치 기준 nearest enemy`
- 거의 같은 거리라면 deterministic tie-break 사용
- 방향/분산은 보조 규칙일 뿐 첫 기준이 아니다

### 2. 타겟 유지

- 공격 중이면 현재 타겟을 유지한다
- 더 가까운 적이 보여도, 현재 타겟을 계속 때릴 수 있으면 바꾸지 않는다
- 이 sticky target 규칙은 근/원거리 모두에 적용한다

### 3. near-engage lock

- 이동 중이라도 현재 타겟에게 거의 도달한 상태라면 기존 타겟을 유지한다
- 막 붙기 직전에 옆 적으로 꺾는 것을 막기 위한 규칙이다

### 4. 재평가와 retarget

- 유닛은 이동 중에도 nearest enemy를 계속 재평가한다
- 다만 타겟 변경은 무조건 즉시가 아니라 조건부다
- 새 타겟이 이미 사거리 안이면 즉시 공격 전환 가능
- 새 타겟이 사거리 밖이면 ETA가 충분히 유리할 때만 변경
- 현재 타겟이 죽었거나, 일정 시간 이상 못 때리면 강제 재평가한다

### 5. 재배치

- 아군 혼잡이나 공격 슬롯 부족이 생기면, 짧은 wait 후 적극적으로 재배치한다
- 재배치는 잠깐 목표에서 멀어지는 이동이어도 허용한다
- 다만 전체 ETA가 나빠지지 않는 범위에서만 허용한다

### 6. queue-follow

- 외길에서 앞유닛이 전진 중이면 뒷유닛은 뒤따라간다
- 앞유닛이 교전에 들어가 길이 닫히면 뒷유닛은 대기한다
- 다른 실제 경로가 열리면 그때 retarget 또는 reposition 한다

`EngagementLane` 같은 개념은 필요하더라도 핵심 모델이 아니라 보조 heuristic으로 다뤄야 한다.
즉 "라인 유지"는 절대 규칙이 아니라, `sticky target`과 `reposition`을 보조하는 약한 우선순위에 가깝다.

## tie-break 규칙 재설계

현재 구조의 중요한 문제는 tie-break가 전체 전선 형태를 좌우한다는 점이다.

장기적으로는 다음 우선순위를 권장한다.

1. 현재 타겟을 더 빨리 공격할 수 있는 후보
2. 현재 타겟 유지와 near-engage lock
3. 아군 혼잡을 풀어주는 재배치
4. deterministic ordering

여기서 `lane`, 방향, 분산은 핵심 우선순위가 아니라 위 규칙을 보조하는 신호로만 사용한다.

## 이벤트 모델 계획

외부 이벤트는 최대한 유지하는 편이 안전하다.

유지 후보:

- `MovementIntent`
- `MoveStep`
- `UnitMoved`
- `MovementStopped`

단, 내부 의미는 바뀔 수 있다.

- `MovementIntent`
  - 유닛 단위 이동 요청이 아니라 orchestrator 실행 요청
- `MoveStep`
  - 승인된 세그먼트 종료 시각

추가 가능한 내부 이벤트:

- `MovementResolve`
- `MovementCommit`
- `EngagementRefresh`

내부 이벤트는 꼭 공개 타임라인에 노출할 필요는 없다.

## Continuous Spatial Layer 계획

장기적으로는 orchestrator 위에 별도의 `continuous spatial layer`를 올리는 것을 목표로 한다.

핵심 원칙:

- `MovementOrchestrator`는 여전히 authoritative decision layer다
- 타일 occupancy / reservation / BFS / 결정론은 유지한다
- continuous layer는 decision을 대체하지 않고, 더 세밀한 위치 표현과 일부 거리 판정을 담당한다
- 즉 `tile authority + continuous presentation/proximity`의 2계층 구조를 만든다

### 왜 별도 레이어가 필요한가

현재 core 내부에는 이미 `pos_x_units`, `pos_y_units`가 있지만,
의미가 약하고 `MovementStopped` 시점에만 강하게 드러난다.

이 구조의 한계:

- replay가 이동 중엔 타일 중심 위주로 보이고 마지막에만 내부 좌표가 드러난다
- `near-engage`, `AoE`, `blast`, `proximity` 같은 판정이 점점 어색해진다
- 디버깅 시 "실제로 언제 어디까지 움직였는가"를 보기가 어렵다

따라서 장기적으로는 continuous position을
"부수적인 표현값"이 아니라 spatial layer의 정식 데이터로 승격해야 한다.

### 레이어 분리

#### 1. Decision Layer

담당:

- 타겟 선정
- 이동 후보 생성
- 충돌 조정
- `Engage / Advance / Hold / Yield / Blocked / WaitRepath`
- 타일 occupancy / reservation

authoritative source:

- 타일 좌표
- 예약 상태
- 이동 path
- 세그먼트 승인/중단 결정

#### 2. Continuous Spatial Layer

담당:

- 유닛의 실제 2D 위치 추적
- 이동 세그먼트 보간
- 근접 engage 보조 판정
- 범위형 스킬, proximity, 폭발 반경 같은 연속 거리 기반 판정의 기반
- replay / visualizer가 사용할 더 풍부한 spatial 정보 제공

authoritative source:

- `pos_x_units`, `pos_y_units`
- 현재 이동 세그먼트의 start / target / start_ms / end_ms / end_kind

즉:

- "어디로 가야 하는가"는 orchestrator가 결정
- "그 결정이 연속 공간에서 어떻게 보이는가"는 continuous layer가 책임진다

### 유지할 것과 바꿀 것

유지:

- 타일 occupancy
- reservation
- BFS pathing
- deterministic tick ordering

변경:

- replay가 movement를 복원하는 방식
- 일부 거리 기반 판정의 기준
- movement timeline이 continuous segment를 더 많이 노출하는 방식

### 좌표 모델

현재 좌표계는 타일 중심 기반이다.

- `(x, y)` 타일 중심 = `(x * TILE_UNITS_PER_TILE, y * TILE_UNITS_PER_TILE)`
- 현재 `(0, 0)`은 월드 원점이 아니라 `(0, 0)` 타일의 중심이다

continuous layer에서도 이 기준은 유지한다.
즉 좌표계 자체를 갈아엎지 않고, 같은 기준 위에서 internal position을 더 적극적으로 사용한다.

### movement 세그먼트 모델

continuous layer의 핵심 데이터는 "현재 이동 세그먼트"다.

필요한 필드:

- `segment_start_x_units`
- `segment_start_y_units`
- `segment_target_x_units`
- `segment_target_y_units`
- `segment_started_at_ms`
- `segment_ends_at_ms`
- `segment_end_kind`

현재 `MovementState`가 이미 일부를 들고 있으므로,
새 레이어는 완전 신설이라기보다 이 정보를 더 정식화하는 방향이 맞다.

### replay / timeline 전략

장기적으로는 `MovementStopped`에만 continuous 좌표를 싣는 방식에서 벗어나야 한다.

권장 방향:

1. 매 프레임 좌표를 전부 기록하지 않는다
2. 대신 movement segment의 start/target/time window를 replay가 복원할 수 있게 기록한다
3. replay는 이 segment 정보를 바탕으로 렌더 프레임마다 continuous interpolation 한다

이 접근의 장점:

- timeline 폭증을 막는다
- core의 authoritative tick 모델을 유지한다
- visualizer/replayer는 훨씬 자연스럽게 보간할 수 있다

즉 "sample dump"보다 "segment reconstruction"이 목표다.

### 범위형 스킬과 continuous 판정

continuous layer를 도입하는 가장 큰 실익 중 하나는
범위형 스킬과 proximity 판정의 품질을 높일 수 있다는 점이다.

권장 적용 순서:

1. melee engage 보조 판정
2. 원형 AoE / blast radius
3. line / cone / sweep 판정
4. knockback, pull, displacement 기반 효과

중요한 점:

- 모든 스킬을 즉시 continuous로 바꾸지 않는다
- 타일 기반 판정이 충분한 스킬은 계속 타일 기반으로 둘 수 있다
- 어떤 판정이 tile-based인지 continuous-based인지 문서에 명확히 구분해야 한다

### 이번 설계에서 고정한 전제

#### 1. occupancy / attack slot

- 논리 점유 규칙은 계속 `1 tile = 1 unit`이다
- 근접 공격 가능 슬롯도 타일 기반으로 유지한다
- 즉 continuous layer는 타일 authority를 깨지 않는다

#### 2. visual / spatial separation

- 유닛은 visual 상으로도 서로 겹치면 안 된다
- 특히 근접 engage 상태에서는 최소한 근접 공격 거리만큼 spacing을 유지해야 한다
- continuous layer는 "같은 타일 위에 두 유닛을 겹쳐 보이게 하는 레이어"가 아니라
  "서로 다른 타일/경계 근처에서 더 자연스럽게 위치시키는 레이어"다

#### 3. melee engage anchor

- 근접 유닛의 기본 engage 위치는 타겟과 가장 가까운 경계점이다
- 타일 중심에 억지로 세우는 것보다
  current segment / current target 기준 가장 가까운 boundary anchor를 우선한다

#### 4. ranged / idle position

- 원거리 유닛도 전투 시작 후에는 타일 중심 고정을 전제로 하지 않는다
- 이동 중이 아니더라도 continuous spatial layer 기준 위치를 가진다
- 즉 "타일에 서 있다"와 "타일 중심에 있다"는 같은 의미가 아니다

#### 5. unit hitbox

- continuous 판정에서 유닛은 점(point)이 아니라 원(circle) hitbox로 본다
- 1차 설계에서는 전 유닛 공통 반지름의 고정 원 hitbox를 기본으로 한다
- 필요해지면 나중에 유닛 타입별 반지름으로 확장할 수 있다

#### 6. projectile / AoE / skill origin

- 투사체와 범위기의 hit/miss는 continuous 판정으로 간다
- 스킬의 시작점과 판정 기준점은 현재 continuous 좌표를 기준으로 한다
- 즉 장기적으로 스킬은 "타일 중심에서 발사/폭발"보다
  "실제 위치에서 발사/폭발"하는 모델을 쓴다

#### 7. 우선 적용 대상

- 1차 목표는 자연스러운 movement 표현이다
- 2차 목표는 projectile, blast, AoE가 유닛 옆을 아슬아슬하게 스쳐 지나갈 수 있는
  continuous hit/miss 모델이다
- 즉 continuous layer는 movement 표현과 skill 판정 개선을 동시에 위한 기반이다

### 장점

- replay 품질이 크게 좋아진다
- movement debug가 쉬워진다
- AoE / proximity / melee engage가 더 자연스러워진다
- 현재 타일 기반 authority를 버리지 않아도 된다

### 단점 / 리스크

- 좌표계가 2층이 되어 규칙이 헷갈릴 수 있다
- validation / replay / test 비용이 증가한다
- "어느 판정이 어느 레이어를 쓰는가"가 불명확하면 디버깅이 어려워진다

따라서 continuous layer는 기능 추가보다
`책임 분리`와 `판정 기준 명세`가 먼저다.

## 기존 코드에서 옮길 책임

### plan.rs 에서 이관할 것

- `compute_movement_intents`
- `destination_candidates_in_order`
- `formulate_enemy_chase_plan`
- 목적지 후보 tie-break
- 타겟/목적지 우선순위 규칙

### execute.rs 에서 이관할 것

- `handle_move_steps_at`의 충돌 해소
- `abort_to_wait_repath`
- 다음 스텝 재예약
- soft/hard reservation 교착 시 처리

### sim.rs 에서 바뀔 것

- movement 관련 이벤트 진입점이 orchestrator를 부르게 됨
- 공격 시작 전 movement 상태와의 결합이 단순화될 수 있음

## 단계별 마이그레이션 계획

### Phase 1: Skeleton

목표:

- `movement/orchestrator.rs` 추가
- orchestrator 입력/출력 타입 정의
- 기존 movement는 그대로 유지

주의:

- 이 단계에서는 동작을 바꾸지 않는다.
- 먼저 구조와 진입점을 고정한다.

### Phase 2: Candidate Generation 이관

목표:

- 목적지 후보/전진 후보 생성만 orchestrator로 옮긴다.
- 커밋은 기존 `execute.rs`가 계속 담당한다.

효과:

- 로컬 계획 코드를 전역 계획 코드로 옮기기 시작

### Phase 3: Conflict Resolution 이관

목표:

- 충돌 해소를 orchestrator가 담당
- `wait_repath` 발생 지점 축소
- `Yield`, `Blocked`, `Hold` 상태 도입

효과:

- "실패 후 재시도"가 아니라 "전역 조정 후 커밋" 구조가 자리 잡음

### Phase 4: Target Lifecycle / Reposition Rules

목표:

- nearest target / sticky target / near-engage lock 계약 추가
- 막힘 이후 짧은 wait + reposition 규칙 추가
- ETA 기반 retarget hysteresis 추가

효과:

- mid-fight 타겟 흔들림 감소
- 외길과 혼잡 상황에서 더 자연스러운 재배치
- lane 고집 때문에 멍하게 서 있는 시간 감소

### Phase 5: Continuous Spatial Layer

목표:

- orchestrator 위에 continuous spatial layer를 정식화
- movement segment를 replay가 복원 가능한 형태로 노출
- visualizer/replayer가 타일 중심 점프 대신 continuous interpolation을 사용하게 준비

세부 단계:

1. `MovementState`의 continuous segment 의미를 문서/코드에서 명시
2. replay가 필요한 segment metadata를 timeline에 노출
3. replayer를 tile hop + final snap 모델에서 continuous interpolation 모델로 전환
4. melee engage / AoE / proximity 판정을 continuous layer에 점진 도입

주의:

- occupancy, reservation, BFS authority는 유지
- continuous layer는 표현과 일부 거리 판정을 개선하는 레이어이지,
  타일 authority를 대체하지 않는다

### Phase 5 상세 구현 계획

이 phase는 한 번에 끝내지 않는다.
가장 중요한 원칙은 다음 두 가지다.

- orchestrator decision layer는 그대로 둔다
- continuous layer는 `movement interpolation -> projectile/AoE 판정` 순으로 점진 도입한다

즉 첫 구현 목표는 "movement를 더 자연스럽게 보이게 하는 것"이며,
projectile / AoE / proximity는 그 다음 단계다.

#### Step 5.1. Spatial 타입 정식화

우선 새 레이어의 공통 타입부터 만든다.

권장 타입:

- `ContinuousPosition`
  - `x_units`
  - `y_units`
- `MovementSegment`
  - `start_x_units`
  - `start_y_units`
  - `target_x_units`
  - `target_y_units`
  - `started_at_ms`
  - `ends_at_ms`
  - `end_kind`
- `UnitHitbox`
  - 1차는 `radius_units` 하나만 가지는 공통 circle

원칙:

- 이 타입들은 타일 pathing을 대체하지 않는다
- `MovementState`가 이미 가진 필드를 감싸거나 정규화하는 역할부터 시작한다
- 1차 구현에서는 `MovementState` 내부 field를 재사용해도 된다

1차 코드 진입점:

- `src/game/battle/core/movement/mod.rs`
- `src/game/battle/core/movement/execute.rs`

#### Step 5.2. MovementState를 segment-authoritative로 정리

현재도 movement는 내부적으로 continuous 좌표를 가진다.
다만 그 의미가 타입 차원에서 약하다.

1차 정리 목표:

- 현재 step의 start/target/time window를 `MovementSegment` 의미로 명확히 재구성
- `update_move_position_to()`와 `schedule_current_move_step()`가
  같은 segment 모델을 공유하게 정리
- `MovementStopped`가 찍히는 좌표가
  "그 시점 spatial layer의 authoritative continuous 위치"라는 점을 명시

구현 원칙:

- path cursor, tile occupancy, reservation 흐름은 건드리지 않는다
- continuous layer는 기존 movement 적분 로직을 래핑/명문화하는 수준에서 시작한다

#### Step 5.3. Timeline에 segment metadata 노출

replayer를 바꾸려면 timeline이 movement segment를 복원할 수 있어야 한다.
다만 매 프레임 좌표 dump는 금지한다.

권장 timeline 확장 방향:

1. 기존 `UnitMoved`는 유지
   - authoritative tile occupancy 전환 이벤트로 계속 사용
2. 새 movement segment 이벤트를 추가
   - 예시 이름:
     - `MovementSegmentStarted`
     - `MovementSegmentInterrupted`
   - 또는 기존 movement event에 segment metadata를 얹는 확장도 가능
3. `MovementStopped`는 최종 stop/spatial snapshot 이벤트로 유지

`MovementSegmentStarted`에 필요한 필드:

- `unit_instance_id`
- `segment_start_x_units`
- `segment_start_y_units`
- `segment_target_x_units`
- `segment_target_y_units`
- `segment_started_at_ms`
- `segment_ends_at_ms`
- `segment_end_kind`
- 선택적으로 `from_tile`, `to_tile`

원칙:

- timeline은 authoritative replay reconstruction에 필요한 정보만 담는다
- visual-only easing parameter 같은 것은 여기에 넣지 않는다

1차 코드 진입점:

- `src/game/battle/timeline.rs`
- `src/game/battle/core/movement/execute.rs`

#### Step 5.4. Replayer interpolation 계약

replayer는 더 이상
"타일 중심 hop -> 마지막에 `MovementStopped` snap"
모델로 움직이면 안 된다.

새 계약:

1. `MovementSegmentStarted`를 받으면 active segment를 만든다
2. 렌더 프레임마다 현재 시각 기준 선형 보간으로 위치를 계산한다
3. `UnitMoved`는 tile occupancy 갱신용으로만 쓴다
4. `MovementStopped` 또는 interrupt 이벤트가 오면
   segment를 종료하고 authoritative position으로 동기화한다

fallback 규칙:

- segment 이벤트가 없는 오래된 export는 기존 방식으로 재생 가능하게 둔다
- 즉 replayer는 `segment-aware path`와 `legacy path`를 둘 다 지원해야 한다

이 단계의 성공 조건:

- 이동 중에도 유닛이 타일 중심에만 고정돼 보이지 않는다
- `MovementStopped` 직전에만 갑자기 내부 좌표로 튀는 현상이 사라진다

#### Step 5.5. Continuous hitbox / 거리 판정 유틸

movement interpolation이 안정화되면,
그 다음에야 continuous 판정 유틸을 추가한다.

먼저 필요한 공통 유틸:

- point-to-point distance
- point-to-circle / circle-to-circle overlap
- segment-to-circle shortest distance
- tile center / boundary anchor -> continuous 좌표 변환 helper

1차 적용 대상:

- projectile hit / miss
- 원형 AoE
- blast / proximity

주의:

- melee attack slot은 계속 타일 기반이다
- "누가 어느 타일에 설 수 있는가"는 decision layer authority 그대로 유지
- continuous 판정은 hit / miss / range feel 개선용으로만 도입한다

#### Step 5.6. Projectile / AoE의 1차 continuous 전환

가장 먼저 바꿀 판정은 movement가 아니라 스킬 쪽이다.
이유:

- 현재 요구사항이 "아슬아슬하게 빗나가는 연출"을 포함하기 때문이다
- projectile/AoE는 continuous 효과가 가장 직관적으로 보인다

권장 순서:

1. projectile origin을 타일 중심이 아니라 현재 continuous 좌표로 변경
2. projectile hitbox와 unit circle hitbox 기반 collision 추가
3. 원형 AoE를 tile overlap이 아니라 circle distance 기반으로 전환
4. cone/line/sweep는 마지막 단계로 미룬다

1차 코드 진입점:

- projectile 생성/업데이트 로직
- skill range / AoE 계산 로직
- timeline/replayer projectile 표시 로직

#### Step 5.7. Validation / replay test 전략

continuous layer는 시각 품질만이 아니라
deterministic reconstruction이 중요하다.

따라서 테스트는 세 층으로 간다.

1. spatial unit test
   - 거리 계산
   - hitbox overlap
   - segment interpolation
2. battle-level export test
   - movement segment 이벤트가 누락되지 않는지
   - `MovementStopped` 이전 sudden snap이 줄어드는지
3. replay validation
   - 같은 export를 replayer가 다시 읽었을 때
     segment reconstruction 결과가 authoritative stop 위치와 일치하는지

추천 대표 시나리오:

- opening melee engage
- dense frontline collapse
- projectile가 유닛 옆을 스치며 miss하는 케이스
- circular AoE가 두 유닛 중 하나만 맞는 경계 케이스

#### Step 5.8. 구현 순서 요약

실제 구현은 아래 순서가 가장 안전하다.

1. `MovementSegment` / `ContinuousPosition` 타입 정식화
2. `MovementState`와 `execute.rs`를 segment-authoritative로 정리
3. timeline에 segment metadata 노출
4. replayer interpolation 추가
5. projectile / AoE용 continuous 거리 유틸 추가
6. projectile -> circular AoE 순서로 continuous 판정 도입

즉:

- 먼저 `movement visibility`
- 그 다음 `skill collision fidelity`

이 순서를 지키는 편이 장기적으로 가장 안전하다

### Phase 6: Legacy Movement 제거

목표:

- 기존 `plan.rs`, `execute.rs`의 중복 책임 제거
- orchestrator 기반 구조로 정리

주의:

- 제거는 반드시 회귀 테스트 확보 후 진행

## 테스트 계획

새 시스템에는 회귀 테스트가 필수다.

### 반드시 필요한 테스트

- 전투 시작 시 근/원거리 모두 nearest enemy를 첫 타겟으로 잡는지
- 공격 중에는 더 가까운 적이 보여도 타겟을 유지하는지
- 현재 타겟을 못 때리게 되면 nearest 기준으로 재선택하는지
- near-engage 상태에서는 retarget하지 않는지
- 외길에서 뒷유닛이 queue-follow 후 막히면 대기하는지
- 다른 실제 경로가 열리면 retarget 또는 reposition 하는지
- 원거리가 사거리 진입 즉시 멈추고 공격하는지
- 원거리가 붙잡혀도 카이팅하지 않는지
- 같은 seed 입력에서 타임라인이 항상 동일한지

### 회귀 픽스처로 유지할 것

- [tft_like_field_6v6_mixed_melee_ranged_battle.json](/mnt/f/work/simulator/core/timeline_exports/tft_like_field_6v6_mixed_melee_ranged_battle.json)

이 타임라인은 단순 샘플이 아니라 "현재 구조 문제가 실제로 드러난 재현 케이스"로 취급해야 한다.

## 리스크

### 1. 공격 개시와 이동 중단 불일치

movement를 새로 짜는 동안 가장 먼저 재발할 수 있는 리스크다.

예:

- 유닛은 멈췄는데 공격은 시작되지 않음
- 공격은 가능하다고 보는데 movement 쪽은 계속 전진함

대응:

- 공격 가능 판정 helper를 movement와 attack 시작이 공유해야 함

### 2. sticky target / near-engage 규칙이 너무 강한 경우

문제:

- 유닛이 과도하게 고정되어 재배치가 늦어짐

대응:

- `retarget_timeout_ms`, `no_progress_timeout_ms`, `retarget_eta_advantage_ms`
  같은 상수로 완화할 수 있어야 함

### 3. 새 tie-break가 다른 편향을 만드는 경우

문제:

- 기존에는 왼쪽 쏠림
- 새 시스템에서는 다른 방향 쏠림 가능

대응:

- 대칭 입력 시 대칭 결과를 기대하는 테스트 추가

### 4. 회귀 범위 확대

movement는 battle 전체와 강하게 연결돼 있다.

대응:

- 빅뱅 교체가 아니라 phase별 이관

## 구현 원칙

- 기존 movement를 한 번에 삭제하지 않는다.
- orchestrator를 먼저 추가하고 점진적으로 책임을 옮긴다.
- deterministic ordering은 절대 깨지지 않아야 한다.
- tile occupancy 모델은 유지한다.
- Unity 리플레이 보정보다 서버 타임라인 정합성을 우선한다.

## 다른 AI를 위한 시작 포인트

이 문서를 받은 AI가 바로 시작하려면 다음 순서가 좋다.

1. 현재 movement 구조 다시 읽기
   - [movement/mod.rs](/mnt/f/work/simulator/core/src/game/battle/core/movement/mod.rs)
   - [movement/plan.rs](/mnt/f/work/simulator/core/src/game/battle/core/movement/plan.rs)
   - [movement/execute.rs](/mnt/f/work/simulator/core/src/game/battle/core/movement/execute.rs)
2. 기존 문서 읽기
   - [movement_flow.md](/mnt/f/work/simulator/core/docs/movement_flow.md)
   - [battle_flow.md](/mnt/f/work/simulator/core/docs/battle_flow.md)
3. 재현 케이스 확인
   - [tft_like_field_6v6_mixed_melee_ranged_battle.json](/mnt/f/work/simulator/core/timeline_exports/tft_like_field_6v6_mixed_melee_ranged_battle.json)
4. `Phase 1`부터 들어간다
   - orchestrator 타입/입력/출력 정의
   - 아직 동작은 바꾸지 않음

## 현재 결론

이번 리팩토링은 "특정 유닛의 경로 버그 수정"이 아니다.

본질적으로는:

- 로컬 movement planning 제거
- 전역 movement coordination 도입
- nearest target / sticky target / 자유로운 재배치 정책 확립

을 목표로 하는 아키텍처 교체다.

따라서 구현도 "작은 버그 패치"가 아니라 "점진적 시스템 교체"로 접근해야 한다.
