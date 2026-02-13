# Movement & Targeting Flow (Tick-Based, Deterministic)

이 문서는 전투 중 유닛의 **타겟 선정**, **이동**, **타일 점유/예약**, **동시성 처리**를 1ms 고정 tick 기반으로
결정론적으로 정의한다.

> 구현 메모: 실제 엔진은 매 1ms를 모두 순회하지 않고, “의미 있는 시간선(time_ms)”으로 점프하여 처리할 수 있다.
> (예: 다음 공격/투사체 명중/이동 경계 도달 시각). 단, 결과는 “1ms tick을 모두 돌린 것”과 동일해야 한다.

## Goals

- 리플레이 시 매번 동일한 결과(결정론)
- 유닛 겹침 없음: “한 타일 = 최대 1유닛”(겹침/약한 점유 없음)
- 스왑/회전 없음: 다른 유닛의 타일로는 교환 없이 한 명만 진입 가능
- 점유 전환 타이밍은 “타일 경계(boundary) 통과” 시점
- 예약은 기본적으로 장애물(=hard)이나, “예약 주인이 충분히 멀면 soft로 간주하여 통과 허용”

## 1. Time & Units

- 시간 단위: `ms`
- tick: `1 tick = 1ms`
- 거리 단위(공통): `tile_units` (고정소수점)
  - `1 tile = 1_000_000 tile_units`
  - 모든 속도는 `tile_units_per_ms`로 표현 (유닛 이동 속도, 투사체 속도 등)
  - 소요 시간 계산은 부동소수점 없이: `time_ms = ceil(distance_units / speed_units_per_ms)`

### 1.1 Time-Jump 등가 규칙(권장 구현)

이동은 “타일 1칸 이동”이 아니라, **현재 타일에서 다음 타일로 넘어가는 경계(boundary) 지점**을 목표로 하는
연속 좌표 이동으로 표현한다.

- 각 유닛은 연속 좌표 `pos_x_units`, `pos_y_units`를 가진다. (타일 중심은 `tile_units`의 정수 배)
- `MoveStep(time_ms)` 이벤트는 “다음 경계 지점에 도달(또는 도달해야 하는) 시각”을 의미한다.
- 이벤트 노이즈(중단 후 남아있는 미래 `MoveStep`)를 줄이기 위해, `MoveStep`은 `expected_move_epoch`를 포함하며
  유닛의 현재 `move_epoch`와 불일치하면 무효 이벤트로 간주하고 드롭한다.
- 이벤트가 발생하면:
  1. 연속 좌표를 `time_ms`까지 적분(=속도 * Δt만큼 clamp 이동)하여 갱신한다.
  2. 아직 목표 경계에 도달하지 못했으면, **같은 스텝을 더 뒤로 재스케줄**한다.
  3. 목표 경계에 도달했다면, 그 순간 **타일 점유를 다음 타일로 전환 시도**한다.

## 2. Field Model

- 필드는 직사각형 격자 `width x height` 이다.
  - 최소 크기: `min(width, height) >= 3` 그리고 `max(width, height) >= 4` (즉 최소 3x4)
  - 라운드별 크기 변경 없음(매치 중 고정)
- 좌표계:
  - `x ∈ [0, width-1]`, `y ∈ [0, height-1]`
  - 한 좌표를 “타일(tile)”이라 부른다.

## 3. Distances

### 3.1 Chebyshev Distance (사거리/시간 기본)

- `dist_cheb(a, b) = max(|ax-bx|, |ay-by|)`
- 공격 사거리 및 “사거리 내” 판정은 `dist_cheb`를 사용한다.
- 이동/투사체의 시간 계산에서 기본 거리로 `dist_cheb * 1 tile`을 사용한다.
  - 대각 이동도 1 tile로 취급한다.

### 3.2 BFS Path Distance (추적/이동 목표 선택)

- “가장 가까운 적(추적 대상)”은 **BFS 경로 길이**로 정의한다.
- BFS는 8방향 이웃을 사용하며, **코너컷을 허용**한다.
  - 상하좌우가 막혀 있어도 대각 이동 가능

## 4. Determinism Rules (필수)

### 4.1 Global Order

- 같은 tick에서 유닛 처리 순서는 `uuid` 오름차순이다.
  - `uuid` 비교는 `Uuid::as_bytes()`의 lexicographic 오름차순 기준으로 정의한다.

### 4.2 Tile Order

- 타일 정렬은 `(y, x)` 오름차순(위→아래, 좌→우)이다.

### 4.3 BFS Neighbor Order

BFS에서 이웃 확장 순서는 아래 고정 순서를 사용한다.

1. `NW (-1,-1)`
2. `N  ( 0,-1)`
3. `NE ( 1,-1)`
4. `W  (-1, 0)`
5. `E  ( 1, 0)`
6. `SW (-1, 1)`
7. `S  ( 0, 1)`
8. `SE ( 1, 1)`

### 4.4 Tie-Breakers (동률 깨기)

명시되지 않은 모든 동률은 아래 우선순위를 적용한다.

1. 최단 거리(해당 규칙이 사용하는 거리: `dist_cheb` 또는 BFS 길이)
2. 타일 정렬 `(y, x)` 오름차순
3. `uuid` 오름차순

## 5. Tile States (점유/예약)

타일은 아래 상태 중 하나로 해석한다.

- `Empty`: 비어있음
- `Occupied(unit)`: 유닛이 점유
- `Reserved(unit)`: 특정 유닛의 **목적지**로 예약됨 (기본 장애물)

### 5.1 Occupancy Rules

- **한 타일에는 오직 하나의 유닛만** 존재할 수 있다. (겹침 없음)
- 어떤 진영이든 **점유 중인 타일은 점유할 수 없다.**
- `Reserved`는 기본적으로 장애물이나, “soft 예약”은 통과가 허용된다. (5.2 참고)
- 스왑(서로의 위치 교환)은 허용하지 않는다.

### 5.2 Soft/Hard Reservation (거리 기반)

예약은 기본적으로 **hard(막힘)** 이다. 단, 특정 타일 `T`가 `Reserved(R)`일 때,
이 예약의 주인 `R`이 **현재 위치에서 `T`까지 충분히 멀면** soft로 간주한다.

- 거리: `dist_cheb(position_of(R), T)`
- 상수: `K = RESERVATION_HARD_DISTANCE_TILES` (예: 3)
- 규칙:
  - `dist_cheb(position_of(R), T) <= K` 이면 hard(다른 유닛은 진입 불가)
  - `dist_cheb(position_of(R), T) > K` 이면 soft(다른 유닛은 진입 가능)
  - soft 예약 타일을 다른 유닛이 실제로 점유하게 되면, 그 예약은 즉시 취소된다. (예약 보장하지 않음)

## 6. Unit States (이동 관점)

- `Idle`: 이동하지 않는 상태(필요 시 이 상태에서 계획을 수립)
- `Moving`: 목적지 예약 + 경로를 따라 이동 중
- `WaitRepath(until_ms)`: 목적지/경로를 찾지 못해 재탐색 대기
- `Dead`

추가 메모:
- 공격/스킬은 이벤트 기반으로 별도 처리된다. “이동 중 공격 금지”를 원하면 공격 처리에서 상태 게이트가 필요하다.

### 6.1 Hard CC (예외 규칙)

하드 CC(예: Stun/Freeze)는 아래처럼 이동/점유와 상호작용한다.

- CC 적용 시:
  - 현재 이동/예약을 즉시 취소하고 이동을 중단한다.
  - `next_action_time`을 CC 만료 시각(+1ms)으로 설정하여, 그 시각 전에는 이동 계획을 수립하지 않는다.
  - 중단 시 기존 `MoveStep` 이벤트는 `move_epoch` 갱신으로 무효화된다.
- CC 해제 시(`until_ms` 도달): `MovementIntent`로 재탐색/이동 재개를 유도한다.

## 7. Targeting

### 7.1 Attackable Set

유닛 `U`의 공격 사거리 `range`는 `dist_cheb(U.pos, enemy.pos) <= range`로 판정한다.

- 근거리(사거리 1): 인접 8칸에 적이 있을 때만 공격 가능
- 원거리(사거리 2 이상): 사거리 내 적을 공격 가능
  - 지형/유닛에 의한 line-of-sight 차단 없음(유닛 너머로 공격 가능)

### 7.2 Choose Attack Target

공격 가능한 적이 있다면 다음 규칙으로 1명을 선택한다.

1. `dist_cheb` 최소
2. 동률이면 `enemy_uuid` 오름차순

## 8. Movement Planning (Idle 단계)

공격 가능한 적이 없다면, 추적 대상과 목적지를 아래처럼 선택한다.

### 8.1 Choose Chase Target (가장 가까운 적)

- 추적 대상은 “**공격 가능한 위치(목적지 후보)**까지의 BFS 경로 길이”가 최소인 적을 선택한다.
  - 즉, 적 `E` 자체의 타일이 아니라, 유닛 `U`가 **서서 공격할 수 있는 빈 타일** `T`들 중 하나까지의 거리로 “가까움”을 정의한다.
  - 이 규칙은 특히 **근거리(range=1)**에서 중요하다. (적을 둘러싼 빈 타일이 없으면, 그 적은 “가까워도 공격 위치가 없어서” 추적 대상으로 적합하지 않음)
- BFS에서 장애물:
  - `Occupied`는 항상 장애물
  - `Reserved`는 hard일 때 장애물(soft는 통과 가능)
- 적 `E`에 대한 “추적 거리”는 다음으로 정의한다.
  - 후보 타일 집합 `C(E)`:
    - `dist_cheb(T, E.pos) <= range`
    - `T`는 `Empty`
    - `T`가 `Occupied / Reserved` 이면 제외
    - `U.pos`에서 `T`까지 BFS로 도달 가능
  - `chase_dist(E) = min_{T ∈ C(E)} bfs_len(U.pos -> T)`
  - `C(E)`가 비어있으면 `chase_dist(E) = ∞` (해당 적은 추적 대상 후보에서 제외)
- 동률이면 `enemy_uuid` 오름차순

### 8.2 Choose Destination Tile (사거리 걸치는 지점)

추적 대상의 위치 `E`에 대해 목적지 후보 `T`는:

- `dist_cheb(T, E) <= range` 를 만족
- `T`가 `Empty` 여야 함
- `T`가 `Occupied / Reserved` 이면 제외
- `U.pos`에서 `T`까지 BFS로 도달 가능해야 함

후보 중 다음으로 1개를 선택한다.

1. `U.pos -> T` BFS 길이 최소
2. 동률이면 `(y, x)` 오름차순

### 8.3 Reserve Destination

- 선택된 목적지 타일은 `Reserved(U)`로 예약한다.
- 예약 충돌 시(이미 `Reserved/Occupied`):
  - 같은 BFS 결과로 생성 가능한 다음 후보를 순서대로 시도한다.
  - 후보가 모두 실패하면 “다음으로 가까운 적”으로 추적 대상을 바꿔 8.2/8.3을 다시 시도한다.
    - “다음으로 가까운 적”은 `chase_dist(E)` 오름차순, 동률 `enemy_uuid` 오름차순으로 정렬한 다음 후보를 의미한다.
  - 모든 적에 대해 시도가 실패하면 `WaitRepath`로 전환한다.

## 9. Movement Execution (Moving 단계)

### 9.1 Continuous Position

- 각 유닛은 연속 좌표 `pos_x_units`, `pos_y_units`를 가진다.
- 타일 중심 좌표는 `(tile.x * TILE_UNITS, tile.y * TILE_UNITS)`이다.
- 스폰 시점에는 타일 중심에서 시작한다.

### 9.2 Boundary Crossing & Occupancy Switch

타일 점유는 경계 통과 순간에만 바뀐다.

- 현재 타일을 `from`, 다음 타일을 `to`라고 할 때,
  목표 경계 좌표는 `from` 타일 중심에서 `to` 방향으로 `HALF_TILE`만큼 이동한 지점이다.
- “경계선은 다음 타일에 속한다”(half-open):
  - 경계 지점에 도달한 tick에 `from -> to` 점유 전환을 시도한다.
  - 대각(NE 등)도 한 번에 `(tx+1, ty+1)`로 전환한다.

### 9.3 Step Validity (충돌/막힘)

`to`로 점유 전환이 성공하려면:

- `to`가 필드 내부
- `to`가 `Occupied`가 아님
- `to`가 `Reserved`일 때:
  - hard면 실패(WaitRepath)
  - soft면 성공 가능(성공 시 기존 예약 취소)

### 9.4 Same-Tick Conflicts (동시성)

같은 `time_ms`에 여러 유닛이 이동 스텝을 수행할 수 있다.

- 이동 의도(intent)는 `(unit_id, from, to)`로 구성된다.
- 처리 순서: `unit_id(uuid)` 오름차순.
- 같은 tick에서 **한 `to` 타일에는 최대 1명만 성공**한다.
  - 이미 성공해서 “claim”된 `to`로 가려는 나머지는 `WaitRepath`.
- 이동 적용은 순차적으로 `Battlefield`를 갱신한다.
  - 즉, 앞선 이동이 점유를 바꾸면 뒤의 이동은 바뀐 상태를 본다(결정론적).

### 9.5 Arrive at Destination

유닛이 예약한 목적지에 도착하면:

- `Reserved(U)`를 해제하고 목적지 타일을 `Occupied(U)`로 전환한다.
- 각 1스텝 이동은 리플레이/클라이언트 재현을 위해 타임라인에 `UnitMoved(from, to)`로 기록하는 것을 권장한다.

## 10. Repath / Waiting

목적지 후보를 찾지 못했거나(도달 불가 포함), 모든 후보 예약이 실패하면:

- `WaitRepath(now_ms + 100ms + jitter_ms)`로 전환한다.
- `jitter_ms`는 리플레이 일관성을 위해 결정적으로 생성한다.
  - 예: `jitter_ms = hash(run_seed, unit_uuid, repath_counter) % 17`

`WaitRepath` 만료 시 `Idle`로 돌아가 재탐색한다. (횟수 제한 없음)

## 11. Reservation Cancellation

`Moving` 중이라도 아래 조건이 발생하면 즉시 예약을 취소한다.

### 11.1 Cancellation Causes

- 사망(`Dead`)
- 하드 CC로 행동 불가(`next_action_time` lock)
- 타일 점유 전환(`MoveStep` 성공) 직후, 사거리 내 적을 포착(사거리 내 적 존재)

### 11.2 After Cancellation

- 사망: `Reserved` 해제 후 유닛 관련 자원 정리
- CC: 즉시 정지하며 현재 타일을 `Occupied`로 취급(고정)
- 적 포착: 즉시 정지하며 예약을 해제하고 현재 타일을 계속 점유한다

## 12. Tick Loop Integration (권장 순서)

동일한 결과를 위해 처리 순서를 고정한다. (Time-Jump 구현에서도 동일)

1. 같은 `time_ms`에 발생한 **전투 이벤트(피격/버프/디버프/공격 등)** 를 먼저 처리한다.
   - 이동 중이라도 무적이 아니므로, 이 단계가 이동보다 우선한다.
2. 의도 계산(Intent):
   - `uuid` 오름차순으로 각 유닛을 처리
   - `Idle` 또는 `WaitRepath` 만료 상태의 유닛만 BFS/목적지 계산을 수행
   - 예약은 이 단계에서만 생성/갱신
3. 이동 적용(경계 통과 시점):
   - 이번 `time_ms`에 due인 `MoveStep`들을 모아 intent를 만들고, `uuid` 오름차순으로 적용한다.
   - 같은 tick에서 한 `to` 타일에는 1명만 성공한다(나머지는 `WaitRepath`).
   - 각 유닛의 타일 점유 전환(`move_unit`)이 성공한 직후, 해당 타일 기준으로 사거리 내 적을 검사한다.
     - 포착 시 즉시 `Idle`로 전환하고(예약 해제), 같은 tick에 재탐색을 위해 `MovementIntent(time_ms)`를 푸시할 수 있다.
4. (후속) 같은 `time_ms`에 이벤트가 더 생기면, 같은 순서로 다시 처리한다.

## 13. Deterministic Test Scenarios (체크리스트)

- 근접 유닛이 적을 포위한 상태에서 추적/대기 반복이 결정론적으로 유지되는가
- 동일 거리의 적 2명 존재 시 uuid 정렬로 동일 타겟을 선택하는가
- 목적지 후보가 여러 개인 경우 `(y,x)` 기준으로 동일 후보를 고르는가
- 예약 충돌/스텝 충돌(여러 유닛이 같은 `to`)이 `uuid` 우선권으로 결정론적으로 해결되는가
- hard 예약은 막히고, soft 예약은 통과 가능한가(K 거리 기준)
- soft 예약을 통과하여 점유하게 되면 예약이 취소되는가
- `Reserved`(hard)가 BFS 경로에서 장애물로 동작하는가
- 코너컷 허용(상하좌우 막힘에도 대각 이동)이 실제로 가능하고 결정론적인가
